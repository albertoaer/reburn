use std::{path::{PathBuf, Path}, sync::mpsc::{channel, Sender}, fs::{self, DirEntry}};
use notify::RecursiveMode::{self, *};
use crate::parsing::selector::Selector;
use super::reduce::*;
use super::common::*;

struct Walker(Sender<(PathBuf, RecursiveMode)>);

fn files_in<P: AsRef<Path>>(path: &P) -> impl Iterator<Item = DirEntry> {
  fs::read_dir(path)
  .and_then(|d| Ok(d.collect::<Vec<_>>()))
  .unwrap_or_default()
  .into_iter().filter_map(move |f| f.and_then(|e| Ok(e)).ok())
}

impl Walker {
  fn send(&self, path: PathBuf, mode: RecursiveMode) {
    self.0.send((path, mode)).ok();
  }

  fn match_route(&self, remain: &[RouteItem], path: PathBuf) {
    //TODO: Include Name(".") and Name("..")
    match remain {
      [RouteItem::AnySubRoute] => self.send(path, Recursive),
      [n, last @ ..] => {
        let files: Vec<_> = files_in(&path)
          .filter(|p| n.matches(p.file_name().to_str().unwrap())).collect();
        files.iter().for_each(|f| self.match_route(last, f.path()));
        if n.omittable() {
          files.iter().for_each(|f| self.match_route(remain, f.path()));
          self.match_route(last, path);
        }
      }
      [] => self.send(path, NonRecursive)
    }
  }

  fn match_all(&self, routes: impl Iterator<Item = Route>) {
    routes.for_each(|route| self.match_route(&route[..], PathBuf::from(".")));
  }
}

pub fn matches(selector: Selector) -> Result<impl Iterator<Item = (PathBuf, RecursiveMode)>, &'static str> {
  // channels will be useful for future parallelization
  let (sender, receiver) = channel();
  // sender will be closed once dropped
  Walker(sender).match_all(recursive_join(&selector)?.into_iter());
  Ok(receiver.into_iter())
}