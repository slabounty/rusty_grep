use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use anyhow::Result;
use clap::{Parser as ClapParser};
use log::{info};
use regex::Regex;


#[derive(ClapParser, Default)]
#[command(version, about, long_about = None)]
pub struct Cli {
    // Show header
    #[arg(short='H', long, value_name = "HEADER")]
    pub show_header: bool,

    // One or more URLs to fetch
    #[arg(value_name = "REGEX", required = true)]
    pub regex: String,

    // One or more URLs to fetch
    #[arg(value_name = "FILE", required = true)]
    pub file_names: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Rusty Curl");

    let cli = Cli::parse();

    let regex = build_regex(&cli.regex)?;
    let show_header = cli.show_header || cli.file_names.len() > 1;

    for file_name in cli.file_names.iter() {
        process_file_name(&file_name, &regex, show_header)?;
    }

    Ok(())
}

fn build_regex(regex_str: &str) -> Result<Regex, regex::Error> {
    Regex::new(regex_str)
}

fn process_file_name(file_name: &str, regex: &Regex, show_header: bool) -> Result<()> {
    let reader = open_reader(file_name)?;

    let prefix = build_prefix(&file_name, show_header);

    for line_result in reader.lines() {
        let line = line_result?;
        if regex.is_match(&line) {
            println!("{}{}", prefix, line);
        }
    }

    Ok(())
}

fn build_prefix(file_name: &str, show_header: bool) -> String {
    if show_header {
        format!("{}:", file_name)
    } else {
        String::new()
    }
}

fn open_reader<P: AsRef<Path>>(path: P) -> io::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}
