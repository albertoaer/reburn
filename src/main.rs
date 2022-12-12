mod ps;
mod cli;
mod watcher;
mod parsing;
mod walk;

use std::time::SystemTime;

use parsing::selector::parse_selector;

fn aux_to_str<E : ToString>(e: E) -> String {
  e.to_string()
}

const ELAPSE_TIME: u128 = 50;

fn main() -> Result<(), String> {
  let parsed_args = cli::Cli::new();

  let command = parsed_args.command()?;
  let update = || ps::Process::run(command.as_slice()).map_err(aux_to_str);
  let mut active = update()?;
  
  let targets = walk::matches(parse_selector(&parsed_args.watch)?)?;
  let mut last_run = SystemTime::now();

  for _ in watcher::WatchingChannel::try_new(targets).map_err(aux_to_str)? {
    if last_run.elapsed().and_then(|x| Ok(x.as_millis())).unwrap_or(ELAPSE_TIME) >= ELAPSE_TIME {
      last_run = SystemTime::now();
      active.kill();
      active = update()?;
    }
  }
  Ok(())
}