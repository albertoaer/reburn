use super::selector_tokens::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
  /// a/b/c
  Route(Vec<Selector>),
  /// {a,b}
  Option(Vec<Selector>),
  /// abc
  Concat(Vec<Selector>),
  /// *
  WildCard,
  /// **
  WildCardDepth,
  Word(String)
}

struct Parser {
  stack: Vec<(Vec<Selector>, Vec<Selector>, Vec<Selector>)>,
  current: (Vec<Selector>, Vec<Selector>, Vec<Selector>)
}

impl Parser {
  fn new() -> Self {
    Parser {
      stack: Vec::new(),
      current: (Vec::new(), Vec::new(), Vec::new())
    }
  }

  fn push_to_concat(&mut self, atom: Selector) {
    self.current.2.push(atom)
  }

  fn base_push<F: FnOnce(&[Selector]) -> Selector>(
    from: &mut Vec<Selector>, into: &mut Vec<Selector>, force: bool, compose: F
  ) -> Option<()> {
    match &from[..] {
      [a] => into.push(a.clone()),
      [] if force => return None,
      [] => (),
      o => into.push(compose(o))
    }
    *from = Vec::new();
    Some(())
  }

  fn push_to_route(&mut self, force: bool) -> Result<(), &'static str> {
    Self::base_push(&mut self.current.2, &mut self.current.1,
      force, |o| Selector::Concat(o.to_vec())
    ).ok_or("Nothing before the slash")
  }

  fn push_to_option(&mut self, force: bool) -> Result<(), &'static str> {
    self.push_to_route(false)?;
    Self::base_push(&mut self.current.1, &mut self.current.0,
      force, |o| Selector::Route(o.to_vec())
    ).ok_or("Nothing before the comma")
  }

  fn push_to_stack(&mut self) {
    self.stack.push(self.current.clone());
    self.current = (Vec::new(), Vec::new(), Vec::new());
  }

  fn collect(&mut self) -> Result<Selector, &'static str> {
    self.push_to_option(false)?;
    Ok(match &self.current.0[..] {
      [a] => a.clone(),
      [] => return Err("Empty"),
      o => Selector::Option(o.to_vec())
    })
  }

  fn pop_from_stack(&mut self) -> Result<(), &'static str> {
    if let Some(mut top) = self.stack.pop() {
      self.push_to_option(false)?;
      top.2.push(self.collect()?);
      self.current = top;
      return Ok(())
    }
    Err("No opened group")
  }

  fn append_token(&mut self, tk: Token) -> Result<(), &'static str> {
    Ok(match tk {
        Token::WildCard => self.push_to_concat(Selector::WildCard),
        Token::WildCardDepth => self.push_to_concat(Selector::WildCardDepth),
        Token::Word(n) => self.push_to_concat(Selector::Word(n.clone())),
        Token::Open => self.push_to_stack(),
        Token::Close => self.pop_from_stack()?,
        Token::Comma => self.push_to_option(true)?,
        Token::Slash => self.push_to_route(true)?,
    })
  }

  fn get_valid_selector(&mut self) -> Result<Selector, &'static str> {
    if !self.stack.is_empty() {
      return Err("Unclosed group");
    }
    // if stack is empty we are in the first level where no group is open
    self.collect()
  }
}

pub fn parse_selector<'a>(pattern: &'a str) -> Result<Selector, &'static str> {
  let mut parser = Parser::new();
  for token in Token::many_from(pattern.chars()).ok_or("Invalid pattern")? {
    parser.append_token(token)?;
  }
  parser.get_valid_selector()
}

#[cfg(test)]
mod tests {
  use super::*;
  use Selector::*;

  macro_rules! w {
    ($e:expr) => {
      Word($e.to_string())
    };
  }
  macro_rules! make {
    [rt $($e:expr),+] => {
      Route(vec![$($e),+])
    };
    [cc $($e:expr),+] => {
      Concat(vec![$($e),+])
    };
    [op $($e:expr),+] => {
      Option(vec![$($e),+])
    };
  }

  #[test]
  fn test_selector() {
    assert_eq!(parse_selector("word.rs"), Ok(w!("word.rs")));
    assert_eq!(parse_selector("word.rs/*"), Ok(make![rt w!("word.rs"), WildCard]));
    assert_eq!(parse_selector("**/*"), Ok(make![rt WildCardDepth, WildCard]));
    assert!(matches!(parse_selector("**//*"), Err(_)));
    assert!(matches!(parse_selector("{,a}"), Err(_)));
    assert!(matches!(parse_selector(""), Err(_)));
    assert!(matches!(parse_selector("{}"), Err(_)));
    assert_eq!(parse_selector("{ab,cd}/c"), Ok(make![rt make![op w!("ab"), w!("cd")], w!("c")]));
    assert_eq!(parse_selector("{a,b}"), Ok(make![op w!("a"), w!("b")]));
    assert_eq!(parse_selector("a,b/c"), Ok(make![op w!("a"), make![rt w!("b"), w!("c")]]));
    assert_eq!(parse_selector("**/{{ab,cd},*d}.rs"), Ok(
      make![rt WildCardDepth, make![cc make![op make![op w!("ab"), w!("cd")], make![cc WildCard, w!("d")]], w!(".rs")]]
    ));
    assert_eq!(parse_selector("ab/{cd,ef}/{g}"), Ok(
      make![rt w!("ab"), make![op w!("cd"), w!("ef")], w!("g")]
    ));
    assert_eq!(parse_selector("{ab/cd/*,./**}"), Ok(
      make![op make![rt w!("ab"), w!("cd"), WildCard], make![rt w!("."), WildCardDepth]]
    ));
    assert_eq!(parse_selector("a/b{c,d}"), Ok(
      make![rt w!("a"), make![cc w!("b"), make![op w!("c"), w!("d")]]]
    ));
    assert_eq!(parse_selector("a/b/{c,d}"), Ok(
      make![rt w!("a"), w!("b"), make![op w!("c"), w!("d")]]
    ));
  }
}