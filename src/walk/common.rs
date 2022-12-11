#[derive(Debug, Clone)]
pub(super) enum NameMatch {
  Literal(String),
  Any
}

#[derive(Debug, Clone)]
pub(super) enum RouteItem {
  Name(Vec<NameMatch>),
  AnySubRoute
}

impl RouteItem {
  pub(super) fn matches<'a>(&self, src: &'a str) -> bool {
    if let RouteItem::Name(name) = self {
      let (mut prev_wildcard, mut remain) = (false, src);
      for name in name {
        if let NameMatch::Literal(n) = name {
          remain = match remain.find(n) {
            Some(i) if i == 0 || prev_wildcard => &remain[(i + n.len())..],
            _ => return false
          }
        }
        prev_wildcard = matches!(name, NameMatch::Any);
      }
      return prev_wildcard || remain.is_empty()
    }
    return true
  }

  pub(super) fn omittable(&self) -> bool {
    matches!(self, Self::AnySubRoute)
  }
}

pub(super) type Route = Vec<RouteItem>;

#[cfg(test)]
mod tests {
  use super::*;
  use NameMatch::*;
  use RouteItem::*;

  #[test]
  fn test_matches() {
    assert!(AnySubRoute.matches("anything i want"));
    let pattern = Name(vec![Literal("sm".to_string()), Any, Literal("t".to_string())]);
    assert!(pattern.matches("smt"));
    assert!(pattern.matches("sm t"));
    assert!(pattern.matches("sm__t"));
    assert!(!pattern.matches("sm__t_"));
    assert!(!pattern.matches("_sm__t"));
    let pattern2 = Name(vec![Any, Literal("smt".to_string()), Any]);
    assert!(pattern2.matches("smt"));
    assert!(pattern2.matches("_smt"));
    assert!(pattern2.matches("smt_"));
    assert!(!pattern2.matches("sm"));
    assert!(!pattern2.matches("mt"));
  }
}