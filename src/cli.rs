use clap::Parser;

use crate::parsing::interpreter::get_interpreter;

#[derive(Parser, Debug)]
pub struct Cli {
	#[arg(help = "Changes to watch")]
	pub watch: String,

	#[arg(help = "Script to run")]
	pub script: Option<String>,

	#[arg(last = true, help = "Command to run")]
	pub cmd: Option<Vec<String>>
}

impl Cli {
	pub fn new() -> Self {
		Self::parse()
	}

	pub fn command(&self) -> Result<Vec<String>, String> {
		match (&self.cmd, &self.script) {
			(Some(_), Some(_)) | (None, None) => Err("Expected either a script or a command".to_string()),
			(Some(v), None) => Ok(v.clone()),
			(None, Some(script)) => match get_interpreter(script) {
				Ok(mut v) => {
					v.push(script.clone());
					Ok(v)
				},
				Err(e) => Err(e.to_string()),
			}
		}
	}
}