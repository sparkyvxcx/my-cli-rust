use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

type MyResult<T> = Result<T, Box<dyn Error>>;

static NUM_RE: OnceCell<Regex> = OnceCell::new();

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
                .required(true),
        )
        .arg(
            Arg::with_name("bytes")
                .help("Number of bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines"),
        )
        .arg(
            Arg::with_name("lines")
                .help("Number of lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
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
        .map(parse_num)
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
    // println!("{:#?}", config);
    let multiple = config.files.len() > 1;
    let mut files_left = config.files.len();
    for filename in config.files {
        match File::open(&filename) {
            Ok(file_handle) => {
                // println!("Opened {}", filename);
                if multiple && !config.quiet {
                    println!("==> {} <==", filename);
                    files_left -= 1;
                }
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                let file = BufReader::new(file_handle);
                if let Some(bytes) = &config.bytes {
                    print_bytes(file, &bytes, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                }
                if multiple && !config.quiet && files_left > 0 {
                    println!()
                }
            }
            Err(err) => eprintln!("{}: {}", filename, err),
        }
    }
    Ok(())
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());
    match num_re.captures(val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |m| m.as_str());
            let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
            if let Ok(val) = num.parse() {
                if sign == "+" && val == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(val))
                }
            } else {
                Err(From::from(val))
            }
        }
        _ => Err(From::from(val)),
    }
}

#[allow(dead_code)]
fn parse_num_normal(val: &str) -> MyResult<TakeValue> {
    let signs: &[char] = &['+', '-'];
    let res = val
        .starts_with(signs)
        .then(|| val.parse())
        .unwrap_or_else(|| val.parse().map(i64::wrapping_neg));

    match res {
        Ok(num) => {
            if num == 0 && val.starts_with('+') {
                Ok(PlusZero)
            } else {
                Ok(TakeNum(num))
            }
        }
        _ => Err(From::from(val)),
    }
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

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();

    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total == 0 {
                None
            } else {
                Some(0)
            }
        }
        TakeNum(num) => {
            if total == 0 || num == &0 || num > &total {
                None
            } else if num > &0 {
                Some((num - 1) as u64)
            } else {
                let start = num + total;
                if start > 0 {
                    Some(start as u64)
                } else {
                    Some(0)
                }
            }
        }
    }
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if start == line_num as u64 {
                print!("{}", String::from_utf8_lossy(&buf));
            } else {
                line_num += 1;
            }
            buf.clear();
        }
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        if !buf.is_empty() {
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::get_start_index;

    use super::TakeValue::*;
    use super::{count_lines_bytes, parse_input_num, parse_num};

    #[test]
    fn test_parse_input_num() {
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
        assert!(res.is_ok());
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

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundries
        let res = parse_num(&(i64::MAX).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // A non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When staring line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
