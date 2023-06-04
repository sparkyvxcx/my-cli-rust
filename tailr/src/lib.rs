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

#[derive(Debug, PartialEq)]
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
        Ok(num) if num < 0 => Ok(TakeNum(num)),
        Ok(num) if num >= 0 => Ok(TakeNum(num * -1)),
        _ => Err(From::from(val)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_input_num;
    use super::TakeValue::*;

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_input_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_input_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" should result in a negative number
        let res = parse_input_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_input_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_input_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundries
        let res = parse_input_num(&(i64::MAX).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_input_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_input_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_input_num(&i64::MIN.to_string());
        // assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_input_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // A non-integer string is invalid
        let res = parse_input_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
