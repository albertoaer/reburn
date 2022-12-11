mod exec;
mod cli;
mod watcher;
mod parsing;
mod walk;

use parsing::selector::parse_selector;

fn aux_to_str<E : ToString>(e: E) -> String {
  e.to_string()
}

fn main() -> Result<(), String>{
  let parsed_args = cli::Cli::new();

  let command = parsed_args.command()?;
  let update = || exec::run_command(command.as_slice()).map_err(aux_to_str);
  let mut active = update()?;
  
  let targets = walk::matches(parse_selector(&parsed_args.watch)?)?;

  for _ in watcher::WatchingChannel::try_new(targets).map_err(aux_to_str).unwrap() {
    active.kill().ok();
    active = update()?;
  }
  Ok(())
}