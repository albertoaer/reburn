use crate::parsing::selector::Selector;
use super::common::*;

fn reduce_routes<F: Fn(Route, Route) -> Result<Route, &'static str>>(routes: Vec<Vec<Route>>, combine: F)
-> Result<Vec<Route>, &'static str> {
  routes.into_iter().map(|i| Ok(i)).reduce(|a, b| match (a, b) {
    (a @ Err(_), _) | (_, a @ Err(_)) => a,
    (Ok(ref a), Ok(ref b)) => a.into_iter().map(
      |a| b.into_iter().map(|b| combine(a.clone(), b.clone()))
    ).flatten().collect()
  }).unwrap()
}

fn route_combine(a: &[RouteItem], b: &[RouteItem]) -> Result<Route, &'static str> {
  use RouteItem::*;

  match (a, b) {
    ([.., AnySubRoute], _) | (_, [AnySubRoute, ..]) => Err("Trying to concat with **"),
    (_, []) | ([], _) => Err("Empty concat"),
    ([init @ .., Name(a_i)], [Name(b_i), tail @ ..]) => Ok({
      let mut v = Vec::new();
      v.extend(init.iter().map(|x| x.clone()));
      v.push({
        let mut aux = Vec::new();
        aux.extend(a_i.clone());
        aux.extend(b_i.clone());
        Name(aux)
      });
      v.extend(tail.iter().map(|x| x.clone()));
      v
    })
  }
}

pub(super) fn recursive_join(selector: &Selector) -> Result<Vec<Route>, &'static str> {
  use RouteItem::*;
  use NameMatch::*;
  use Selector::*;

  Ok(match selector {
    Route(e) | Option(e) | Concat(e) if e.is_empty() => return Err("Empty group"),
    Route(v) => {
      let mut ret = Vec::new();
      for s in v {
        ret.push(recursive_join(s)?);
      }
      reduce_routes(ret, |a, b| {
        let mut v = Vec::new();
        v.extend(a.iter().map(|x| x.clone()));
        v.extend(b.iter().map(|x| x.clone()));
        Ok(v)
      })?
    }
    Option(v) => {
      let mut ret = Vec::new();
      for s in v {
        ret.append(&mut recursive_join(s)?);
      }
      ret
    }
    Concat(v) => {
      let mut ret = Vec::new();
      for s in v {
        ret.push(recursive_join(s)?);
      }
      reduce_routes(ret, |a, b| route_combine(&a[..], &b[..]))?
    }
    WildCard => vec![vec![Name(vec![Any])]],
    WildCardDepth => vec![vec![AnySubRoute]],
    Word(n) => vec![vec![Name(vec![Literal(n.clone())])]],
  })
}