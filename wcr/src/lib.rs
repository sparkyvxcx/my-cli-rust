use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?}", config);
    //
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        // println!("{}", filename);

        match open(&filename) {
            Ok(file) => {
                if let Ok(FileInfo {
                    num_lines,
                    num_words,
                    num_bytes,
                    num_chars,
                }) = count(file)
                {
                    total_lines += num_lines;
                    total_words += num_words;
                    total_bytes += num_bytes;
                    total_chars += num_chars;

                    println!(
                        "{}{}{}{}{}",
                        format_field(num_lines, config.lines),
                        format_field(num_words, config.words),
                        format_field(num_bytes, config.bytes),
                        format_field(num_chars, config.chars),
                        if filename == "-" {
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    )
                }
            }
            Err(e) => eprintln!("{}: {}", filename, e),
        }
    }

    if config.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.chars),
        );
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let lines_help = "Show line count";
    let bytes_help = "Show byte count";
    let chars_help = "Show character count";
    let words_help = "Show word count";

    let matches = App::new("wcr")
        .version("0.1.0")
        .author("KenYoens-Clar<yclar@gmail.com>")
        .about("Rusty Word Count")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help(lines_help),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help(words_help),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help(bytes_help),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help(chars_help)
                .conflicts_with("bytes"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches.is_present("lines");
    let words = matches.is_present("words");
    let bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if !lines && !words && !bytes && !chars {
        return Ok(Config {
            files,
            lines: true,
            words: true,
            bytes: true,
            chars,
        });
    }

    Ok(Config {
        files,
        lines,
        words,
        bytes,
        chars,
    })
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();

    loop {
        let bytes_read = file.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{count, format_field, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());

        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
