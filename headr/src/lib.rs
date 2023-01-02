use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let lines_help = "Prints ? number of lines of given file(s)";
    let bytes_help = "Prints ? number of bytes of given file(s)";
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
        .get_matches();
    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();
    // TODO: implement read `-c 1K` to print the first 1024 bytes of the file
    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    // TODO: add an option for selecting characters in addition to bytes

    Ok(Config {
        files,
        lines,
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?} {:?} {:?}", config.files, config.lines, config.bytes);
    let num_files = config.files.len();
    let multiple = num_files > 1;
    for (count, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Ok(mut file) => {
                if multiple {
                    println!("{}==> {} <==", if count > 0 { "\n" } else { "" }, filename);
                }
                if let Some(size) = config.bytes {
                    /*
                    let reader = BufReader::with_capacity(size, file);
                    let byte_vec: Vec<u8> = reader.bytes().map(|b| b.unwrap()).collect();
                    */
                    let mut buffer: Vec<u8> = vec![0; size];
                    if let Err(_) = file.read_exact(&mut buffer) {
                        continue;
                    }
                    // let utf8_content = String::from_utf8_lossy(&byte_vec);
                    let utf8_content = String::from_utf8_lossy(&buffer);
                    print!("{}", utf8_content);
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
