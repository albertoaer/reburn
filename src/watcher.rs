use std::{sync::mpsc::{channel, Receiver}, path::{Path, PathBuf}};

use notify::{Watcher, recommended_watcher, RecommendedWatcher, RecursiveMode};

#[derive(Debug)]
pub struct WatchingChannel<T> {
	_watcher: RecommendedWatcher,
	receiver: Receiver<T>
}

impl WatchingChannel<Vec<PathBuf>> {
	pub fn try_new<P: AsRef<Path>, I: Iterator<Item = (P, RecursiveMode)>>(targets: I) -> Result<Self, notify::Error> {
		let (sender, receiver) = channel::<Vec<PathBuf>>();
		let mut watcher = recommended_watcher(
			move |res: Result<notify::Event, notify::Error>| match res {
				Ok(event) => {
					sender.send(event.paths).ok();
				},
				Err(e) => panic!("Watch error: {:?}", e),
			}
		)?;
		for (path, recursive) in targets {
			watcher.watch(path.as_ref(), recursive)?;
		}
		Ok(WatchingChannel{ _watcher: watcher, receiver })
	}
}

impl<T> Iterator for WatchingChannel<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match self.receiver.recv() {
			Ok(n) => Some(n),
			Err(_) => None
		}
	}
}