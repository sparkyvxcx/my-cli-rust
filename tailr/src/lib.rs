use crate::TakeValue::*;
use clap::{App, Arg};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pub files: Vec<String>,
    pub lines: TakeValue,
    pub bytes: Option<TakeValue>,
    pub quiet: bool,
}

#[derive(Debug)]
pub enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.0.1")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty tail")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-")
                .required(true),
        )
        .arg(
            Arg::with_name("bytes")
                .help("Number of bytes")
                .short("c")
                .long("bytes")
                .conflicts_with("lines")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("lines")
                .help("Number of lines")
                .short("n")
                .long("lines")
                .conflicts_with("bytes")
                .takes_value(true)
                .default_value("10"),
        )
        .arg(
            Arg::with_name("quiet")
                .help("Suppress headers")
                .short("q")
                .long("quiet"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let bytes = matches
        .value_of("bytes")
        .map(parse_input_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    let lines = matches
        .value_of("lines")
        .map(parse_input_num)
        .unwrap()
        .map_err(|e| format!("illegal line count -- {}", e))?;
    let quiet = matches.is_present("quiet");

    Ok(Config {
        files,
        bytes,
        lines,
        quiet,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

fn parse_input_num(val: &str) -> MyResult<TakeValue> {
    if val.contains("+") {
        let new_val = val.replace("+", "");
        match new_val.parse::<i64>() {
            Ok(num) => {
                if num == 0 {
                    return Ok(PlusZero);
                }
                return Ok(TakeNum(num));
            }
            _ => return Err(From::from(val)),
        }
    }
    match val.parse::<i64>() {
        Ok(num) => Ok(TakeNum(num * -1)),
        _ => Err(From::from(val)),
    }
}
