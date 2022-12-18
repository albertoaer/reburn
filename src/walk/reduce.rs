use crate::parsing::selector::Selector;
use super::common::*;

fn reduce_routes<F: Fn(Route, Route) -> Result<Route, &'static str>>(routes: Vec<Vec<Route>>, combine: F)
-> Result<Vec<Route>, &'static str> {
  routes.into_iter().map(|i| Ok(i)).reduce(|a, b| match (a, b) {
    (a @ Err(_), _) | (_, a @ Err(_)) => a,
    (Ok(ref a), Ok(ref b)) => a.into_iter()
      .map( |a| b.into_iter().map(|b| combine(a.clone(), b.clone())) ).flatten().collect()
  }).unwrap()
}

fn route_combine(a: &[RouteItem], b: &[RouteItem]) -> Result<Route, &'static str> {
  use RouteItem::*;

  match (a, b) {
    ([.., AnySubRoute], _) | (_, [AnySubRoute, ..]) => Err("Trying to concat with **"),
    (_, []) | ([], _) => Err("Empty concat"),
    ([init @ .., Name(left)], [Name(right), tail @ ..]) => Ok({
      let mut v = Vec::new();
      v.extend(init.iter().map(|x| x.clone()));
      v.push(Name(
        vec![left.clone(), right.clone()].into_iter().flatten().collect::<Vec<_>>()
      ));
      v.extend(tail.iter().map(|x| x.clone()));
      v
    })
  }
}

#[inline]
fn recursive_join_many(v: &Vec<Selector>) -> Result<Vec<Vec<Route>>, &'static str> {
  v.into_iter().map(recursive_join).collect::<Result<Vec<_>, _>>()
}

#[inline]
fn negate(v: Vec<RouteItem>) -> Result<Route, &'static str> {
  use RouteItem::*;

  if v.len() == 1 {
    match &v[0] {
      RouteItem::Name(n) if n.len() == 1 => return Ok(vec![Name(vec![n[0].clone().negate()?])]),
      _ => ()
    }
  }
  return Err("Only literals can be negated")
}

pub(super) fn recursive_join(selector: &Selector) -> Result<Vec<Route>, &'static str> {
  use Selector::*;
  use RouteItem::*;
  use NameMatch::*;

  Ok(match selector {
    Option(v) | Route(v) | Concat(v) if v.is_empty() => return Err("Empty group"),
    Option(v) => recursive_join_many(v)?.into_iter().flatten().collect(),
    Route(v) => reduce_routes(recursive_join_many(v)?, |a, b| {
      let mut v = Vec::new();
      v.extend(a.iter().map(|x| x.clone()));
      v.extend(b.iter().map(|x| x.clone()));
      Ok(v)
    })?,
    Concat(v) => reduce_routes(recursive_join_many(v)?,
      |a, b| route_combine(&a[..], &b[..])
    )?,
    Not(v) => recursive_join(v)?.into_iter()
      .map(negate).collect::<Result<Vec<_>, _>>()?,
    WildCard => vec![vec![Name(vec![Any])]],
    WildCardDepth => vec![vec![AnySubRoute]],
    Word(n) => vec![vec![Name(vec![Literal(n.clone())])]],
  })
}