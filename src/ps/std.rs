use std::{process::{Command, Child}, ffi::OsStr, io};

pub struct Process(Child);

impl Process {
	pub fn run<S : AsRef<OsStr>>(args: &[S]) -> io::Result<Self> {
		Ok(Process(Command::new(&args[0]).args(&args[1..]).spawn()?))
	}
	
	pub fn kill(&mut self) {
		self.0.kill().ok();
	}
}