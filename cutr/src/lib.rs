use crate::Extract::*;
use clap::{App, Arg};
use std::{error::Error, ops::Range};

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
        .version("0.0.1")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty cut")
        .arg(
            Arg::with_name("files")
                .multiple(true)
                .help("Input file(s)")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("fields")
                .short("f")
                .long("field")
                .value_name("FIELDS")
                .help("Selected fields"),
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("delim")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("b")
                .long("bytes")
                .value_name("BYTES")
                .help("Selected bytes"),
        )
        .arg(
            Arg::with_name("chars")
                .short("c")
                .long("chars")
                .value_name("CHARS")
                .help("Selected characters"),
        )
        .get_matches();
    let files = matches.values_of_lossy("files").unwrap();
    let delimiter = matches.value_of("delimiter").unwrap().as_bytes().to_owned()[0];
    let extract = if matches.is_present("bytes") {
        let cut_ranges = matches.values_of_lossy("bytes").unwrap();
        let position_list = cut_ranges
            .iter()
            .map(|r| {
                let cut_range: Vec<_> = r.split("-").collect();
                Range {
                    start: cut_range[0].parse::<usize>().unwrap(),
                    end: cut_range[1].parse::<usize>().unwrap(),
                }
            })
            .collect();
        Bytes(position_list)
    } else if matches.is_present("chars") {
        let cut_ranges = matches.values_of_lossy("chars").unwrap();
        let position_list = cut_ranges
            .iter()
            .map(|r| {
                let cut_range: Vec<_> = r.split("-").collect();
                Range {
                    start: cut_range[0].parse::<usize>().unwrap(),
                    end: cut_range[1].parse::<usize>().unwrap(),
                }
            })
            .collect();
        Chars(position_list)
    } else {
        let cut_ranges = matches.values_of_lossy("fields").unwrap();
        let position_list = cut_ranges
            .iter()
            .map(|r| {
                let cut_range: Vec<_> = r.split("-").collect();
                Range {
                    start: cut_range[0].parse::<usize>().unwrap(),
                    end: cut_range[1].parse::<usize>().unwrap(),
                }
            })
            .collect();
        Fields(position_list)
    };

    Ok(Config {
        files,
        delimiter,
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut position_lists = vec![];
    for range in range.replace("0", "").split(",") {
        let mut start = 0;
        let mut end = 0;

        let bound: Vec<&str> = range.split("-").collect();
        if bound.len() == 2 {
            start = match bound[0].parse::<usize>() {
                Ok(v) => v,
                Err(_) => return Err(From::from(format!("illegal list value \"{}\"", range))),
            };
            end = match bound[1].parse::<usize>() {
                Ok(v) => v,
                Err(_) => return Err(From::from(format!("illegal list value \"{}\"", range))),
            };

            if start >= end {
                return Err(From::from(format!(
                    "First number in range ({}) must be lower than second number ({})",
                    start, end
                )));
            }
        } else if bound.len() == 1 {
            end = match bound[0].parse::<usize>() {
                Ok(v) => v,
                Err(_) => return Err(From::from(format!("illegal list value \"{}\"", range))),
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

        position_lists.push(Range { start, end });
    }

    Ok(position_lists)
}

#[cfg(test)]
mod unit_tests {
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
}