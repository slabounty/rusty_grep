use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Result;
use clap::{Parser as ClapParser};
use log::{info};
use regex::Regex;


#[derive(ClapParser, Default)]
#[command(version, about, long_about = None)]
pub struct Cli {
    // One or more URLs to fetch
    #[arg(value_name = "REGEX", required = true)]
    pub regex: String,

    // One or more URLs to fetch
    #[arg(value_name = "FILE", required = true)]
    pub file_name: String,
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Rusty Curl");

    let cli = Cli::parse();

    info!("Regex = {}", cli.regex);
    info!("File = {}", cli.file_name);

    let regex = Regex::new(&cli.regex)?;

    let file_path = Path::new(&cli.file_name);

    let file = File::open(&file_path)?;

    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        if regex.is_match(&line) {
            println!("{}", line);
        }
    }

    Ok(())
}
