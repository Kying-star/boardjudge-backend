use anyhow::{Context, Result};
use boardjudge_backend::Config;
use clap::{ArgEnum, Parser};
use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Parser, Serialize, Deserialize)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "./config.toml")]
    config: String,
    #[clap(short, long, arg_enum, default_value = "error")]
    level: ArgsLevel,
}

#[derive(ArgEnum, Clone, Copy, Serialize, Deserialize)]
pub enum ArgsLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[tokio::main]
async fn main() -> Result<()> {
    use ArgsLevel::*;
    let args = Args::parse();
    let direct = std::fs::read_to_string(&args.config).context("failed to read config")?;
    let xonfig = toml::from_str::<Config>(&direct).context("failed to parse config")?;
    let fmt = tracing_subscriber::fmt();
    match args.level {
        Trace => fmt.with_max_level(Level::TRACE).with_test_writer().init(),
        Debug => fmt.with_max_level(Level::DEBUG).with_test_writer().init(),
        Info => fmt.with_max_level(Level::INFO).init(),
        Warn => fmt.with_max_level(Level::WARN).init(),
        Error => fmt.with_max_level(Level::ERROR).init(),
    };
    let mut dir = std::env::current_dir().context("failed to read current dir")?;
    dir.push(&args.config);
    dir.pop();
    std::env::set_current_dir(dir).context("failed to set current dir")?;
    boardjudge_backend::main(xonfig).await?;
    Ok(())
}
