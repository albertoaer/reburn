use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(help = "Changes to watch")]
    pub watch: String,

    #[arg(help = "Script to run")]
    pub script: Option<String>,

    #[arg(last = true, help = "Command to run")]
    pub cmd: Option<Vec<String>>
}