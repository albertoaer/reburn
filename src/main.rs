use clap::Parser;
use notify::RecursiveMode;

mod exec;
mod cli;
mod watcher;
mod parsing;

fn main() -> Result<(), String>{
  let args = cli::Cli::parse().command()?;
  let w = watcher::WatchingChannel::try_new([(".", RecursiveMode::Recursive)]
    .into_iter()).unwrap();
  let update = || exec::run_command(args.as_slice())
    .map_err(|e| e.to_string());
  let mut active = update()?;
  for _ in w {
    active.kill().ok();
    active = update()?;
  }
  Ok(())
}