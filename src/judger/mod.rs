pub mod run;

use self::run::RunStatistics;
use self::run::{run, RunConfig, RunError, RunStatus};
use crate::config;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use tokio::sync::mpsc::unbounded_channel as mpsc_channel;
use tokio::sync::mpsc::UnboundedSender as MpscSender;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender as OneshotSender;
use uuid::Uuid;
use Status::*;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    JudgeFailed,
    TestdataError,
    CompilationError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    WrongAnswer,
    Accepted,
    Skipped,
}

impl From<Status> for &'static str {
    fn from(s: Status) -> Self {
        match s {
            JudgeFailed => "judge_failed",
            TestdataError => "testdata_error",
            CompilationError => "compilation_error",
            RuntimeError => "runtime_error",
            TimeLimitExceeded => "time_limit_exceeded",
            MemoryLimitExceeded => "memory_limit_exceeded",
            WrongAnswer => "wrong_answer",
            Accepted => "accepted",
            Skipped => "skipped",
        }
    }
}

impl Serialize for Status {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(Into::<&'static str>::into(*self))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Test {
    pub name: String,
    pub status: Status,
    pub time: u32,
    pub memory: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Tests {
    Message(String),
    Tests(Vec<Test>),
}

impl From<&str> for Tests {
    fn from(e: &str) -> Self {
        Self::Message(e.to_string())
    }
}

pub fn judge(
    Judge {
        record_id: _,
        problem_id,
        time_limit,
        memory_limit,
        language,
        code,
    }: Judge,
) -> Result<(Status, Tests)> {
    let _ = std::fs::remove_dir_all("/tmp/boardjudge/judge");
    let testdata =
        match std::fs::read_dir(format!("{}/{}/testdata", config().judger.root, problem_id)) {
            Err(_) => return Ok((TestdataError, "{}".into())),
            Ok(x) => x,
        }
        .into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|x| x.file_name().into_string().ok())
        .collect::<Vec<_>>();
    std::fs::create_dir_all("/tmp/boardjudge/judge/code")?;
    std::fs::write("/tmp/boardjudge/judge/code", code)?;
    match run(&RunConfig {
        time_limit: 10_000,
        memory_limit: 256 << 20,
        exec_path: match language.as_str() {
            "c" => &config().judger.compiler_c,
            "cxx" => &config().judger.compiler_cxx,
            _ => return Ok((CompilationError, "{}".into())),
        },
        input_path: "/dev/null",
        output_path: "/dev/null",
        env: &[],
        args: &[
            "/tmp/boardjudge/judge/code".as_bytes(),
            "-o".as_bytes(),
            "/tmp/boardjudge/judge/a.out".as_bytes(),
            "-g".as_bytes(),
            "-Wall".as_bytes(),
            "-static-libgcc".as_bytes(),
            "-fexec-charset=UTF-8".as_bytes(),
            "-std=c++2b".as_bytes(),
            "-march=native".as_bytes(),
        ],
    }) {
        Ok(t) => {
            if t.status != RunStatus::Success || t.code != 0 {
                return Ok((CompilationError, "{}".into()));
            }
        }
        Err(RunError::Internal) => return Ok((CompilationError, "{}".into())),
    }
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    for name in testdata {
        if name.ends_with(".in") {
            inputs.insert(name);
        } else if name.ends_with(".out") {
            outputs.insert(name);
        }
    }
    let mut xests = inputs
        .intersection(&outputs)
        .cloned()
        .map(|name| Test {
            name,
            status: Skipped,
            time: 0,
            memory: 0,
        })
        .collect::<Vec<Test>>();
    let mut xtatus = Accepted;
    for test in xests.iter_mut() {
        match run(&RunConfig {
            time_limit,
            memory_limit: memory_limit as u64,
            exec_path: "/tmp/boardjudge/judge/a.out",
            input_path: format!(
                "{}/{}/testdata/{}.in",
                config().judger.root,
                problem_id,
                test.name
            )
            .as_str(),
            output_path: "/tmp/boardjudge/judge/output",
            env: &[],
            args: &[],
        }) {
            Ok(RunStatistics {
                time,
                memory,
                code: _,
                status: RunStatus::Success,
            }) => {
                let answer = std::fs::read(format!(
                    "{}/{}/testdata/{}.in",
                    config().judger.root,
                    problem_id,
                    test.name
                ))?;
                let output = std::fs::read("/tmp/boardjudge/judge/a.out")?;
                *test = Test {
                    name: std::mem::replace(&mut test.name, "".to_string()),
                    status: if ojcmp::Comparison::AC
                        == ojcmp::try_normal_compare(
                            &mut answer.as_slice(),
                            &mut output.as_slice(),
                        )? {
                        Accepted
                    } else {
                        WrongAnswer
                    },
                    time,
                    memory: memory as u32,
                };
            }
            Ok(RunStatistics {
                time,
                memory,
                code: _,
                status: RunStatus::RuntimeError,
            }) => {
                *test = Test {
                    name: std::mem::replace(&mut test.name, "".to_string()),
                    status: RuntimeError,
                    time,
                    memory: memory as u32,
                };
                xtatus = RuntimeError;
                break;
            }
            Ok(RunStatistics {
                time,
                memory,
                code: _,
                status: RunStatus::TimeLimitExceeded,
            }) => {
                *test = Test {
                    name: std::mem::replace(&mut test.name, "".to_string()),
                    status: MemoryLimitExceeded,
                    time,
                    memory: memory as u32,
                };
                xtatus = RuntimeError;
                break;
            }
            Ok(RunStatistics {
                time,
                memory,
                code: _,
                status: RunStatus::MemoryLimitExceeded,
            }) => {
                *test = Test {
                    name: std::mem::replace(&mut test.name, "".to_string()),
                    status: MemoryLimitExceeded,
                    time,
                    memory: memory as u32,
                };
                xtatus = RuntimeError;
                break;
            }
            Err(RunError::Internal) => {
                *test = Test {
                    name: std::mem::replace(&mut test.name, "".to_string()),
                    status: JudgeFailed,
                    time: 0,
                    memory: 0,
                };
                xtatus = RuntimeError;
                break;
            }
        }
    }
    std::fs::remove_dir_all("/tmp/boardjudge/judge")?;
    Ok((xtatus, Tests::Tests(xests)))
}

pub struct Judge {
    pub record_id: Uuid,
    pub problem_id: Uuid,
    pub time_limit: u32,
    pub memory_limit: u32,
    pub language: String,
    pub code: String,
}

#[derive(Clone)]
pub struct Judger {
    sender: MpscSender<(Judge, OneshotSender<(Status, Tests)>)>,
}

impl Judger {
    pub fn daemon() -> Judger {
        let (tx, mut rx) = mpsc_channel::<(Judge, OneshotSender<(Status, Tests)>)>();
        tokio::spawn(async move {
            while let Some((j, s)) = rx.recv().await {
                let (status, result) = tokio::task::block_in_place(move || judge(j))
                    .unwrap_or((JudgeFailed, "{}".into()));
                let _ = s.send((status, result));
            }
        });
        Judger { sender: tx }
    }
    pub async fn judge(&self, j: Judge) -> (Status, Tests) {
        let (tx, rx) = oneshot_channel();
        let _ = self.sender.send((j, tx));
        rx.await.unwrap_or((JudgeFailed, "{}".into()))
    }
}
