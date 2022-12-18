use std::rc::Rc;

use super::selector_tokens::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
  /// {a,b}
  Option(Vec<Selector>),
  /// a/b/c
  Route(Vec<Selector>),
  /// abc
  Concat(Vec<Selector>),
  /// *
  WildCard,
  /// **
  WildCardDepth,
  /// !
  Not(Rc<Selector>),
  Word(String)
}

#[derive(Debug, Clone)]
struct ParserLevel {
  option: Vec<Selector>,
  route: Vec<Selector>,
  concat: Vec<Selector>,
  negate_next: bool
}

impl ParserLevel {
  fn new() -> Self {
    ParserLevel {
      option: Vec::new(), route: Vec::new(), concat: Vec::new(), negate_next: false
    }
  }

  fn negate(&mut self) {
    self.negate_next = true;
  }

  fn push_to_concat(&mut self, atom: Selector) {
    self.concat.push(if self.negate_next {
      self.negate_next = false;
      Selector::Not(Rc::new(atom))
    } else {
      atom
    });
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
    if self.negate_next {
      return Err("Unused negation mark")
    }
    Self::base_push(&mut self.concat, &mut self.route,
      force, |o| Selector::Concat(o.to_vec())
    ).ok_or("Nothing before the slash")
  }

  fn push_to_option(&mut self, force: bool) -> Result<(), &'static str> {
    self.push_to_route(false)?;
    Self::base_push(&mut self.route, &mut self.option,
      force, |o| Selector::Route(o.to_vec())
    ).ok_or("Nothing before the comma")
  }
  
  fn collect(&mut self) -> Result<Selector, &'static str> {
    self.push_to_option(false)?;
    Ok(match &self.option[..] {
      [a] => a.clone(),
      [] => return Err("Empty"),
      o => Selector::Option(o.to_vec())
    })
  }
}

struct Parser {
  stack: Vec<ParserLevel>,
  current: ParserLevel
}

impl Parser {
  fn new() -> Self {
    Parser {
      stack: Vec::new(),
      current: ParserLevel::new()
    }
  }

  fn push_to_stack(&mut self) {
    self.stack.push(self.current.clone());
    self.current = ParserLevel::new();
  }

  fn pop_from_stack(&mut self) -> Result<(), &'static str> {
    if let Some(mut top) = self.stack.pop() {
      self.current.push_to_option(false)?;
      top.push_to_concat(self.current.collect()?);
      self.current = top;
      return Ok(())
    }
    Err("No opened group")
  }

  fn append_token(&mut self, tk: Token) -> Result<(), &'static str> {
    Ok(match tk {
        Token::WildCard => self.current.push_to_concat(Selector::WildCard),
        Token::WildCardDepth => self.current.push_to_concat(Selector::WildCardDepth),
        Token::Not => self.current.negate(),
        Token::Word(n) => self.current.push_to_concat(Selector::Word(n.clone())),
        Token::Open => self.push_to_stack(),
        Token::Close => self.pop_from_stack()?,
        Token::Comma => self.current.push_to_option(true)?,
        Token::Slash => self.current.push_to_route(true)?,
    })
  }

  fn get_valid_selector(&mut self) -> Result<Selector, &'static str> {
    if !self.stack.is_empty() {
      return Err("Unclosed group");
    }
    // if stack is empty we are in the first level where no group is open
    self.current.collect()
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
  macro_rules! n {
    ($e:expr) => {
      Not(Rc::new($e))
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
    assert!(matches!(parse_selector("a/!b/!{c,d}!"), Err(_)));
    assert_eq!(parse_selector("a/!b/!{c,d}"), Ok(
      make![rt w!("a"), n!(w!("b")), n!(make![op w!("c"), w!("d")])]
    ));
    assert_eq!(parse_selector("!{a,*c}"), Ok(n!(make![op w!("a"), make![cc WildCard, w!("c")]])));
  }
}