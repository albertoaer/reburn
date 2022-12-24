#[derive(Debug, Clone, PartialEq)]
pub(super) enum NameMatch {
  Literal(String),
  NegatedLiteral(String),
  Any
}

impl NameMatch {
  pub(super) fn negate(self) -> Result<NameMatch, &'static str> {
    match self {
      Self::Literal(n) => Ok(Self::NegatedLiteral(n)),
      Self::NegatedLiteral(n) => Ok(Self::Literal(n)),
      Self::Any => Err("Can not negate any")
    }
  }
}

#[derive(Debug, Clone)]
pub(super) enum RouteItem {
  Name(Vec<NameMatch>),
  AnySubRoute
}

fn do_name_match(name: &Vec<NameMatch>, mut remain: &str) -> bool {
  use NameMatch::*;
  let mut free_begin = false;
  let mut black_list = Vec::<&String>::new();
  let black_list_chk = |list: &Vec<_>, free: bool| !list.iter().any(|v| match remain.find(*v) {
    Some(i) if i == 0 || free => true,
    _ => false
  });
  for name in name {
    match name {
      NegatedLiteral(n) => black_list.push(n),
      Any => free_begin = true,
      Literal(n) => {
        remain = match remain.find(n) {
          Some(i) if i == 0 || (free_begin && black_list_chk(&black_list, free_begin)) => &remain[(i + n.len())..],
          _ => return false
        };
        black_list.clear();
        free_begin = false;
      },
    }
  }
  return remain.is_empty() || free_begin && black_list_chk(&black_list, free_begin)
}

impl RouteItem {
  pub(super) fn matches(&self, src: &str) -> bool {
    if let RouteItem::Name(name) = self {
      return do_name_match(name, src.clone())
    }
    return true
  }

  pub(super) fn omittable(&self) -> bool {
    matches!(self, Self::AnySubRoute)
  }

  pub(super) fn is_name(&self, matcher: &[NameMatch]) -> bool {
    if let Self::Name(name) = self {
      return &name[..] == matcher
    }
    false
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
    let pattern = Name(vec![Any, Literal("smt".to_string()), Any]);
    assert!(pattern.matches("smt"));
    assert!(pattern.matches("_smt"));
    assert!(pattern.matches("smt_"));
    assert!(!pattern.matches("sm"));
    assert!(!pattern.matches("mt"));
    let pattern = Name(vec![Any, NegatedLiteral("_gen".to_string())]);
    assert!(pattern.matches("afile"));
    assert!(pattern.matches("afile_other"));
    assert!(!pattern.matches("afile_gen"));
    assert!(!pattern.matches("afile_gen_other"));
    let pattern = Name(vec![Any, NegatedLiteral("_gen".to_string()), Literal(".rs".to_string())]);
    assert!(pattern.matches("afile.rs"));
    assert!(pattern.matches("afile_other.rs"));
    assert!(!pattern.matches("afile_gen.rs"));
    assert!(!pattern.matches("afile"));
    assert!(!pattern.matches("afile_other"));
    let pattern = Name(vec![NegatedLiteral("private_".to_string()), Any]);
    assert!(pattern.matches("afile.rs"));
    assert!(pattern.matches("afile_other.rs"));
    assert!(pattern.matches("anything"));
    assert!(pattern.matches("priv_smt"));
    assert!(!pattern.matches("private_anything"));
    assert!(!pattern.matches("private_"));
    let pattern = Name(vec![NegatedLiteral("private_".to_string()), Literal("file".to_string())]);
    assert!(pattern.matches("file"));
    //makes no sense to use a negated without a wildcard
    assert!(!pattern.matches("private_file"));
    assert!(!pattern.matches("anything_file"));
    let pattern = Name(vec![Any, NegatedLiteral("avoid".to_string()), Literal("_".to_string()), Any]);
    assert!(pattern.matches("_avoid"));
    assert!(pattern.matches("_avoid_smt"));
    assert!(!pattern.matches("avoid_"));
    assert!(!pattern.matches("avoid"));
  }
}