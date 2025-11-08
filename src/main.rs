use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;

use anyhow::Result;
use clap::{ArgAction, Parser as ClapParser};
use log::{info};
use regex::{Regex, RegexBuilder};


#[derive(ClapParser, Default)]
#[command(version, about, long_about = None, disable_help_flag=true)]
pub struct Cli {
    /// Show header
    #[arg(short='H', long, value_name = "HEADER")]
    pub show_header: bool,

    /// Don't show header
    #[arg(short='h', long, value_name = "NO HEADER")]
    pub no_header: bool,

    /// Case insenstive search
    #[arg(short, long, value_name = "CASE INSENSITIVE")]
    pub insensitive: bool,

    /// Invert match
    #[arg(short='v', long, value_name = "INVERT MATCH")]
    pub invert_match: bool,

    /// Show line numbers
    #[arg(short='n', long, value_name = "LINE NUMBERS")]
    pub show_line_numbers: bool,

    /// Count matching lines
    #[arg(short, long, value_name = "COUNT MATCHING LINES")]
    pub count_matching_lines: bool,

    /// Regex to search for
    #[arg(value_name = "REGEX", required = true)]
    pub regex: String,

    /// One or more files to check
    #[arg(value_name = "FILE", required = true)]
    pub file_names: Vec<String>,

    /// Manually restore --help
    #[arg(long = "help", action = ArgAction::Help, help = "Print help information")]
    help: Option<bool>,
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Rusty Curl");

    let cli = Cli::parse();

    let regex = build_regex(&cli.regex, cli.insensitive)?;
    let show_header = cli.show_header || cli.file_names.len() > 1;


    for file_name in cli.file_names.iter() {
        process_file_name(&file_name, &regex, show_header, cli.no_header, cli.invert_match, cli.show_line_numbers, cli.count_matching_lines, io::stdout())?;
    }

    Ok(())
}

fn build_regex(regex_str: &str, insensitive: bool) -> Result<Regex, regex::Error> {
    RegexBuilder::new(regex_str)
        .case_insensitive(insensitive)
        .build()
}

/// Returns `Ok(())` on success, writes matches to `out`.
fn process_file_name<P: AsRef<Path>, W: Write>(
    file_name: P,
    regex: &Regex,
    show_header: bool,
    no_header: bool,
    invert_match: bool,
    show_line_numbers: bool,
    count_matching_lines: bool,
    mut out: W,
) -> io::Result<()> {
    let file_path = file_name.as_ref();
    let file_name_str = file_path.to_str().unwrap_or_default(); // safe fallback

    let reader = open_reader(file_name.as_ref())?;
    let mut line_number: u32 = 0;
    let mut matching_lines: u32 = 0;

    for line_result in reader.lines() {
        line_number += 1;
        let line = line_result?;
        let is_match = regex.is_match(&line);

        if is_match {
            matching_lines += 1;
        }

        if should_write_line(is_match, invert_match, count_matching_lines) {
            let prefix = build_prefix(file_name_str, show_header, no_header, show_line_numbers, line_number);
            writeln!(out, "{}{}", prefix, line)?;
        }
    }

    if count_matching_lines {
        if show_header {
            writeln!(out, "{}:{}", file_name_str, matching_lines)?;
        }
        else {
            writeln!(out, "{}", matching_lines)?;
        }
    }

    Ok(())
}

fn should_write_line(is_match: bool, invert_match: bool, count_matching_lines: bool) -> bool {
    is_match != invert_match && !count_matching_lines
}

fn build_prefix(file_name: &str, show_header: bool, no_header: bool, show_line_numbers: bool, line_number: u32) -> String {
    let mut prefix = String::new();

    if show_header && !no_header {
        prefix.push_str(&format!("{}:", file_name));
    }

    if show_line_numbers {
        prefix.push_str(&format!("{}:", line_number));
    }

    prefix
}

fn open_reader<P: AsRef<Path>>(path: P) -> io::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Write, BufRead};
    use tempfile::NamedTempFile;

    #[test]
    fn test_build_regex_without_insensitive() -> Result<()> {
        let regex = build_regex("hello", false)?;

        assert_eq!(regex.is_match("some text HELLO more text"), false);

        Ok(())
    }

    #[test]
    fn test_build_regex_with_insensitive() -> Result<()> {
        let regex = build_regex("hello", true)?;

        assert_eq!(regex.is_match("some text HELLO more text"), true);

        Ok(())
    }

    #[test]
    fn test_build_prefix_with_header_without_line_numbers() -> Result<()> {
        let prefix_with_header = build_prefix("some_file", true, false, false, 22);

        assert_eq!(prefix_with_header, "some_file:");

        Ok(())
    }

    #[test]
    fn test_build_prefix_without_header_without_line_numbers() -> Result<()> {
        let prefix_with_header = build_prefix("some_file", false, false, false, 22);

        assert_eq!(prefix_with_header, "");

        Ok(())
    }

    #[test]
    fn test_build_prefix_with_header_with_line_numbers() -> Result<()> {
        let prefix_with_header = build_prefix("some_file", true, false, true, 22);

        assert_eq!(prefix_with_header, "some_file:22:");

        Ok(())
    }

    #[test]
    fn test_build_prefix_without_header_with_line_numbers() -> Result<()> {
        let prefix_with_header = build_prefix("some_file", false, false, true, 22);

        assert_eq!(prefix_with_header, "22:");

        Ok(())
    }

    #[test]
    fn test_open_reader_reads_file() -> io::Result<()> {
        // 1. Create a temporary file
        let mut tmpfile = NamedTempFile::new()?;

        // 2. Write some content to it
        writeln!(tmpfile, "hello world")?;
        writeln!(tmpfile, "goodbye world")?;

        // 3. Re-open the file through your function
        let reader = open_reader(tmpfile.path())?;

        // 4. Collect the lines and verify the content
        let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
        assert_eq!(lines, vec!["hello world", "goodbye world"]);

        Ok(())
    }

    #[test]
    fn test_open_reader_nonexistent_file() {
        // 1. Pick a definitely-nonexistent file path
        let bogus_path = "this_file_should_not_exist_12345.txt";

        // 2. Call your function
        let result = open_reader(bogus_path);

        // 3. Verify it failed
        assert!(result.is_err(), "Expected error for nonexistent file, got Ok");

        // 4. Optionally: check the specific error kind
        if let Err(err) = result {
            assert_eq!(err.kind(), io::ErrorKind::NotFound);
        }
    }

    #[test]
    fn test_process_file_name_matches_lines() -> std::io::Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "hello")?;
        writeln!(tmp, "world")?;
        writeln!(tmp, "HELLO")?;
        // Flush/close the file handle so reads see it
        let path = tmp.path().to_path_buf();

        let regex = build_regex("hello", false).unwrap(); // case-sensitive

        let mut buf: Vec<u8> = Vec::new();
        process_file_name(&path, &regex, false, false, false, false, false, &mut buf)?;

        let out = String::from_utf8(buf).expect("output was not valid UTF-8");
        assert!(out.contains("hello"));
        assert!(!out.contains("world"));
        // "HELLO" only matches if case-insensitive; here it should not.
        assert!(!out.contains("HELLO"));
        Ok(())
    }

    #[test]
    fn test_process_file_name_matches_lines_count_matching() -> std::io::Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "hello")?;
        writeln!(tmp, "world")?;
        writeln!(tmp, "HELLO")?;
        // Flush/close the file handle so reads see it
        let path = tmp.path().to_path_buf();

        let regex = build_regex("hello", false).unwrap(); // case-sensitive

        let mut buf: Vec<u8> = Vec::new();
        process_file_name(&path, &regex, false, false, false, false, true, &mut buf)?;

        let out = String::from_utf8(buf).expect("output was not valid UTF-8");
        println!("out = {}", out);
        assert!(!out.contains("hello"));
        assert!(!out.contains("world"));
        // "HELLO" only matches if case-insensitive; here it should not.
        assert!(!out.contains("HELLO"));
        assert!(out.contains("1"));
        Ok(())
    }

    #[test]
    fn test_process_file_name_matches_lines_count_matching_with_header() -> std::io::Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "hello")?;
        writeln!(tmp, "world")?;
        writeln!(tmp, "HELLO")?;
        // Flush/close the file handle so reads see it
        let path = tmp.path().to_path_buf();

        let regex = build_regex("hello", false).unwrap(); // case-sensitive

        let mut buf: Vec<u8> = Vec::new();
        process_file_name(&path, &regex, true, false, false, false, true, &mut buf)?;

        let out = String::from_utf8(buf).expect("output was not valid UTF-8");

        let filename = path.to_str().unwrap();

        println!("out = {}", out);
        assert!(!out.contains("hello"));
        assert!(!out.contains("world"));
        // "HELLO" only matches if case-insensitive; here it should not.
        assert!(!out.contains("HELLO"));
        assert!(out.contains("1"));
        assert!(out.contains(&format!("{}:1", filename)));
        Ok(())
    }

    #[test]
    fn test_process_file_name_with_header() -> std::io::Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "foo")?;
        writeln!(tmp, "bar")?;
        let path = tmp.path().to_path_buf();

        let regex = build_regex("foo", false).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        process_file_name(&path, &regex, true, false, false, false, false, &mut buf)?; // show_header = true

        let out = String::from_utf8(buf).unwrap();
        // Expect the prefix (filename:) and the matched line
        let filename = path.to_str().unwrap();
        assert!(out.contains(&format!("{}:foo", filename)));
        Ok(())
    }

    #[test]
    fn test_process_file_name_no_matches_outputs_nothing() -> std::io::Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "alpha")?;
        writeln!(tmp, "beta")?;
        let path = tmp.path().to_path_buf();

        let regex = build_regex("zzz", false).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        process_file_name(&path, &regex, false, false, false, false, false, &mut buf)?;

        assert!(buf.is_empty());
        Ok(())
    }

    /// Should write tests
    #[test]
    fn test_should_write_line_match_and_no_invert_without_count() -> Result<()> {
        let invert = false;
        let is_match = true;
        let count_matching_lines = false;

        assert_eq!(should_write_line(is_match, invert, count_matching_lines), true);

        Ok(())
    }

    #[test]
    fn test_should_write_line_match_and_invert_without_count() -> Result<()> {
        let invert = true;
        let is_match = true;
        let count_matching_lines = false;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }

    #[test]
    fn test_should_write_line_no_match_and_invert_without_count() -> Result<()> {
        let invert = false;
        let is_match = true;
        let count_matching_lines = false;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), true);

        Ok(())
    }

    #[test]
    fn test_should_write_line_no_match_and_no_invert_without_count() -> Result<()> {
        let invert = false;
        let is_match = false;
        let count_matching_lines = false;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }

    #[test]
    fn test_should_write_line_match_and_no_invert_with_count() -> Result<()> {
        let invert = false;
        let is_match = true;
        let count_matching_lines = true;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }

    #[test]
    fn test_should_write_line_match_and_invert_with_count() -> Result<()> {
        let invert = true;
        let is_match = true;
        let count_matching_lines = true;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }

    #[test]
    fn test_should_write_line_no_match_and_invert_with_count() -> Result<()> {
        let invert = false;
        let is_match = true;
        let count_matching_lines = true;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }

    #[test]
    fn test_should_write_line_no_match_and_no_invert_with_count() -> Result<()> {
        let invert = false;
        let is_match = false;
        let count_matching_lines = true;

        assert_eq!(should_write_line(is_match, invert,  count_matching_lines), false);

        Ok(())
    }
}
