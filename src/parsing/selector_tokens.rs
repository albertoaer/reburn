use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub(super) enum Token {
  /// *
  WildCard,
  /// **
  WildCardDepth,
  Word(String),
  /// {
  Open,
  /// }
  Close,
  /// ,
  Comma,
  /// /
  Slash,
  /// !
  Not
}

impl Token {
  fn next(c: char, tk: &Option<Token>) -> (Option<Token>, Token) {
    match (c, tk) {
      ('/', _) => (tk.clone(), Self::Slash),
      ('{', _) => (tk.clone(), Self::Open),
      ('}', _) => (tk.clone(), Self::Close),
      (',', _) => (tk.clone(), Self::Comma),
      ('*', Some(Self::WildCard)) => (None, Self::WildCardDepth),
      ('*', _) => (tk.clone(), Self::WildCard),
      ('!', _) => (tk.clone(), Self::Not),
      (_, Some(Self::Word(s))) => (None, Self::Word(format!("{}{}", s, c))),
      (_, _) => (tk.clone(), Self::Word(c.to_string()))
    }
  }

  pub(super) fn many_from(iter: Chars) -> Option<Vec<Token>> {
    let mut v = Vec::new();
    let mut tk = None;
    for c in iter {
      let (push, next) = Self::next(c, &tk);
      if let Some(push) = push {
        v.push(push)
      }
      tk = Some(next)
    }
    if let Some(tk) = tk {
      v.push(tk);
    }
    if v.is_empty() {
      return None
    }
    Some(v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use Token::*;

  macro_rules! assert_tk {
    ($str: expr, None) => {
      assert_eq!(Token::many_from($str.chars()), None);
    };
    ($str: expr, $($e: expr),+) => {
      assert_eq!(Token::many_from($str.chars()), Some(vec![$($e,)+]))
    };
  }

  #[test]
  fn test_tokens() {
    assert_tk!("word.rs", Word("word.rs".to_string()));
    assert_tk!("***", WildCardDepth, WildCard);
    assert_tk!("./.rs", Word(".".to_string()), Slash, Word(".rs".to_string()));
    assert_tk!("../..", Word("..".to_string()), Slash, Word("..".to_string()));
    assert_tk!("..test{.rs,.go}",
      Word("..test".to_string()), Open, Word(".rs".to_string()), Comma, Word(".go".to_string()), Close);
    assert_tk!("", None);
    assert_tk!("/**/something/file.*",
      Slash, WildCardDepth, Slash, Word("something".to_string()), Slash, Word("file.".to_string()), WildCard);
    assert_tk!("{Cargo.toml,*.rs}/",
      Open, Word("Cargo.toml".to_string()), Comma, WildCard, Word(".rs".to_string()), Close, Slash);
    assert_tk!("!test!{c,!d}",
      Not, Word("test".to_string()), Not, Open, Word("c".to_string()), Comma, Not, Word("d".to_string()), Close);
  }
}