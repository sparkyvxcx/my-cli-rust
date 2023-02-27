use crate::Extract::*;
use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::NonZeroUsize;
use std::ops::Range;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .short("d")
                .long("delim")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .help("Selected fields")
                .short("f")
                .long("field")
                .conflicts_with_all(&["bytes", "chars"]),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .help("Selected bytes")
                .short("b")
                .long("bytes")
                .conflicts_with_all(&["chars", "fields"]),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .help("Selected characters")
                .short("c")
                .long("chars")
                .conflicts_with_all(&["bytes", "fields"]),
        )
        .get_matches();

    let delimiter = matches.value_of("delimiter").unwrap();
    let delimiter_bytes = delimiter.as_bytes();
    let delimiter = if delimiter_bytes.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            delimiter.to_string(),
        )));
    } else {
        *delimiter_bytes.first().unwrap()
    };

    let extract = if matches.is_present("bytes") {
        let cut_ranges = matches.value_of("bytes").unwrap();
        Bytes(parse_pos(cut_ranges)?)
    } else if matches.is_present("chars") {
        let cut_ranges = matches.value_of("chars").unwrap();
        Chars(parse_pos(cut_ranges)?)
    } else if matches.is_present("fields") {
        let cut_ranges = matches.value_of("fields").unwrap();
        Fields(parse_pos(cut_ranges)?)
    } else {
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    };

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        delimiter,
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    for filename in &config.files {
        match open(filename) {
            Ok(buf) => {
                println!("Opened {}", filename);
                match &config.extract {
                    Bytes(byte_pos) => {
                        for line in buf.lines() {
                            let line = line.unwrap();
                            println!("{}", extract_bytes(&line, &byte_pos))
                        }
                    }
                    Chars(char_pos) => {
                        for line in buf.lines() {
                            let line = line.unwrap();
                            println!("{}", extract_bytes(&line, &char_pos))
                        }
                    }
                    Fields(_) => todo!(),
                }
            }
            Err(err) => eprintln!("{}: {}", filename, err),
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    char_pos
        .iter()
        .map(|pos| {
            let mut x = String::new();
            for (i, c) in line.chars().enumerate() {
                if pos.contains(&i) {
                    x.push(c);
                }
            }
            x
        })
        .collect::<String>()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut buf = vec![];
    byte_pos.iter().for_each(|pos| {
        for (i, c) in line.bytes().enumerate() {
            if pos.contains(&i) {
                buf.push(c);
            }
        }
    });
    String::from_utf8_lossy(&buf[..]).into_owned()
}

#[warn(deprecated)]
fn parse_pos_mine(range: &str) -> MyResult<PositionList> {
    let mut position_lists = vec![];
    for range in range.split(",") {
        let mut start = 0;
        let mut end = 0;

        println!("{} {}", start, end);

        let bound: Vec<&str> = range.split("-").collect();
        if bound.len() == 2 {
            start = match bound[0].parse::<usize>() {
                Ok(v) => v,
                _ => return Err(From::from(format!("illegal list value: \"{}\"", range))),
            };
            end = match bound[1].parse::<usize>() {
                Ok(v) => v,
                _ => return Err(From::from(format!("illegal list value: \"{}\"", range))),
            };
        } else if bound.len() == 1 {
            end = match bound[0].parse::<usize>() {
                Ok(v) => v,
                _ => return Err(From::from(format!("illegal list value: \"{}\"", range))),
            };
            start = end - 1;
        } else {
            return Err(From::from("wonky error"));
        }

        if start >= end {
            return Err(From::from(format!(
                "First number in range ({}) must be lower than second number ({})",
                start, end
            )));
        }
        position_lists.push(start..end);
    }

    Ok(position_lists)
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();

    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!(
                            "First number in range ({}) must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        ));
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: \"{}\"", input);
    input
        .starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

#[cfg(test)]
mod unit_tests {
    use super::extract_bytes;
    use super::extract_chars;
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"");

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"");

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"");

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"");

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
    }
}
