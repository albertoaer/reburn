use std::{process::{Command, Child}, ffi::OsStr, io};

pub fn run_command<S : AsRef<OsStr>>(args: &[S]) -> io::Result<Child> {
	let mut cmd = Command::new(&args[0]);
	cmd.args(&args[1..]).spawn()
}