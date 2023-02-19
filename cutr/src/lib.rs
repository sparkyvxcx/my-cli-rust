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
