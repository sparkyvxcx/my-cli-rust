use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
    chars: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let lines_help = "Prints ? number of lines of given file(s)";
    let bytes_help = "Prints ? number of bytes of given file(s)";
    let chars_help = "Prints ? number of chars of given file(s)";
    let matches = App::new("headr")
        .version("0.1.0")
        .author("KenYoens-Clar<yclar@gmail.com>")
        .about("Rusty Head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help(lines_help)
                .takes_value(true)
                .value_name("LINES")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help(bytes_help)
                .takes_value(true)
                .value_name("BYTES")
                .conflicts_with("lines"),
        )
        .arg(
            Arg::with_name("chars")
                .short("a")
                .long("chars")
                .help(chars_help)
                .takes_value(true)
                .value_name("CHARS")
                .conflicts_with_all(&["bytes", "lines"]),
        )
        .get_matches();
    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();
    // TODO: implement read `-c 1K` to print the first 1024 bytes of the file, and negative number
    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    // TODO: add an option for selecting characters in addition to bytes
    let chars = matches
        .value_of("chars")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal char count -- {}", e))?;

    Ok(Config {
        files,
        lines,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?} {:?} {:?}", config.files, config.lines, config.bytes);
    let num_files = config.files.len();
    let multiple = num_files > 1;
    for (count, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Ok(file) => {
                if multiple {
                    println!("{}==> {} <==", if count > 0 { "\n" } else { "" }, filename);
                }
                if let Some(size) = config.bytes {
                    let mut handle = file.take(size as u64);
                    let mut buffer: Vec<u8> = vec![0; size];
                    let bytes_read = handle.read(&mut buffer)?;
                    let utf8_content = String::from_utf8_lossy(&buffer[..bytes_read]);
                    print!("{}", utf8_content);
                } else if let Some(size) = config.chars {
                    let mut reader = BufReader::new(file);
                    let mut count = 0;
                    while count != size {
                        let mut buffer = String::new();
                        reader.read_line(&mut buffer)?;
                        for c in buffer.chars() {
                            print!("{}", c);
                            count += 1;
                            if count == size {
                                break;
                            }
                        }
                    }
                } else {
                    let mut reader = BufReader::new(file);
                    for _ in 0..config.lines {
                        let mut buffer = String::new();
                        reader.read_line(&mut buffer)?;
                        print!("{}", buffer);
                    }
                }
            }
            Err(e) => eprintln!("{}: {}", filename, e),
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    // return file handler here
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    if val.contains("K") {
        let new_val = val.replace("K", "");
        match new_val.parse::<usize>() {
            Ok(num) if num > 0 => {
                let num = num * 1024;
                // println!("K in value: {}", num);
                return Ok(num);
            }
            _ => return Err(From::from(val)),
        }
    }
    match val.parse::<usize>() {
        Ok(num) if num > 0 => Ok(num),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");

    // valid positive number is accepted
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");

    // non-number is rejected
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");

    // zero is rejected
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
