use crate::config;
use crate::sys::libjudger;
use std::ffi::CString;
use std::ptr::null_mut;

#[derive(Debug, Copy, Clone)]
pub struct RunConfig<'a> {
    pub time_limit: u32,
    pub memory_limit: u64,
    pub exec_path: &'a str,
    pub input_path: &'a str,
    pub output_path: &'a str,
    pub env: &'a [&'a [u8]],
    pub args: &'a [&'a [u8]],
}

#[derive(Debug, Copy, Clone)]
pub struct RunStatistics {
    pub time: u32,
    pub memory: u64,
    pub code: u32,
    pub status: RunStatus,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RunStatus {
    Success,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
}

#[derive(Debug, Copy, Clone)]
pub enum RunError {
    Internal,
}

pub fn run(c: &RunConfig) -> Result<RunStatistics, RunError> {
    let mut arg2;
    unsafe {
        const CONST_NONE_CSTRING: Option<CString> = None;
        let ffi_exe_path = CString::new(c.exec_path).map_err(|_| RunError::Internal)?;
        let ffi_input_path = CString::new(c.input_path).map_err(|_| RunError::Internal)?;
        let ffi_output_path = CString::new(c.output_path).map_err(|_| RunError::Internal)?;
        let ffi_null_path = CString::new("/dev/null").map_err(|_| RunError::Internal)?;
        let ffi_seccomp_rule_name = CString::new("c_cpp").map_err(|_| RunError::Internal)?;
        let mut ffi_env = [CONST_NONE_CSTRING; 256];
        let mut ffi_args = [CONST_NONE_CSTRING; 256];
        for (i, &j) in c.env.iter().enumerate().take(256) {
            ffi_env[i] = Some(CString::new(j).map_err(|_| RunError::Internal)?);
        }
        for (i, &j) in c.args.iter().enumerate().take(256) {
            ffi_args[i] = Some(CString::new(j).map_err(|_| RunError::Internal)?);
        }
        let mut arg1 = libjudger::config {
            max_cpu_time: c.time_limit as i32,
            max_real_time: c.time_limit as i32,
            max_memory: c.memory_limit as i64,
            max_stack: 8 << 20,
            max_process_number: 0,
            max_output_size: config().judger.output_limit as i64,
            memory_limit_check_only: 0,
            exe_path: ffi_exe_path.as_ptr() as *mut i8,
            input_path: ffi_input_path.as_ptr() as *mut i8,
            output_path: ffi_output_path.as_ptr() as *mut i8,
            error_path: ffi_null_path.as_ptr() as *mut i8,
            args: ffi_args.map(|x| x.map(|y| y.as_ptr() as *mut i8).unwrap_or(null_mut())),
            env: ffi_env.map(|x| x.map(|y| y.as_ptr() as *mut i8).unwrap_or(null_mut())),
            log_path: ffi_null_path.as_ptr() as *mut i8,
            seccomp_rule_name: ffi_seccomp_rule_name.as_ptr() as *mut i8,
            uid: 65534,
            gid: 65534,
        };
        arg2 = libjudger::result {
            cpu_time: 0,
            real_time: 0,
            memory: 0,
            signal: 0,
            exit_code: 0,
            error: 0,
            result: 0,
        };
        libjudger::run(&mut arg1, &mut arg2);
    }
    if arg2.error != 0 {
        return Err(RunError::Internal);
    }
    Ok(RunStatistics {
        time: arg2.real_time as u32,
        memory: arg2.memory as u64,
        code: arg2.exit_code as u32,
        status: match arg2.result {
            0 => RunStatus::Success,
            1 | 2 => RunStatus::TimeLimitExceeded,
            3 => RunStatus::MemoryLimitExceeded,
            4 => RunStatus::RuntimeError,
            _ => return Err(RunError::Internal),
        },
    })
}
