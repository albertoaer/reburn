use std::{path::Path, fs::File, io::{Result, BufReader, BufRead, Error, ErrorKind}, vec};

fn get_reburn_name() -> String {
  "reburn".to_string() // Should it be get dynamically
}

#[derive(Debug, PartialEq)]
enum ShebangLine {
  Shebang(String, Option<String>),
  ReburnShebang,
  NoShebang
}

use ShebangLine::*;

fn shebang_of<'a>(line: &'a str, reburn_name: &'a str) -> ShebangLine {
  if !line.starts_with("#!") {
    return NoShebang;
  }
  let mut chars = line[2..].trim_start().chars();
  let name: String = chars.by_ref().take_while(|s| *s != ' ').collect::<String>().trim().to_string();
  if name.is_empty() {
    return NoShebang;
  }
  if name == *reburn_name {
    return ReburnShebang;
  }
  let remain: String = chars.collect();
  let trimmed = remain.trim().to_string();
  Shebang(name, if trimmed.is_empty() { None } else { Some(trimmed) })
}

pub fn get_interpreter<S: AsRef<Path>>(source: S) -> Result<Vec<String>> {
  let file = File::open(source.as_ref())?;
  let mut buf = BufReader::new(file);
  let reburn_name = get_reburn_name();
  let mut current: ShebangLine;
  loop {
    let mut line = String::new();
    let read = buf.read_line(&mut line)?;
    current = shebang_of(&line, &reburn_name);
    if read == 0 || current != ReburnShebang {
      break;
    }
  }
  match current {
    Shebang(a, b) => if let Some(b) = b {
      Ok(vec![a, b])
    } else {
      Ok(vec![a])
    },
    ReburnShebang => Err(Error::new(ErrorKind::NotFound, "Only the reburn shebang found")),
    NoShebang => Err(Error::new(ErrorKind::NotFound, "No shebang found")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_shebang_of() {
    assert_eq!(shebang_of("nothing", "reburn"), NoShebang);
    assert_eq!(shebang_of("#!reburn", "reburn"), ReburnShebang);
    assert_eq!(shebang_of("#!  reburn arg", "reburn"), ReburnShebang);
    assert_eq!(shebang_of("#!smt", "reburn"), Shebang("smt".to_string(), None));
    assert_eq!(shebang_of("#!smt else", "reburn"), Shebang("smt".to_string(), Some("else".to_string())));
    assert_eq!(shebang_of("#!  smt  else  ", "reburn"), Shebang("smt".to_string(), Some("else".to_string())));
  }
}