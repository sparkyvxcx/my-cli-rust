use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty Cat")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Prints content with line number")
                .takes_value(false)
                .conflicts_with("number_nonblank"),
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Prints content with line number, ignore blank line")
                .takes_value(false),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let number_lines = matches.is_present("number");
    let number_nonblank_lines = matches.is_present("number_nonblank");

    Ok(Config {
        files,
        number_lines,
        number_nonblank_lines,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // dbg!(config);

    for filename in config.files {
        // println!("{}", filename);
        match open(&filename) {
            Ok(file_handler) => {
                let lines = file_handler.lines();
                let mut last_num = 0;
                for (line_num, line) in lines.enumerate() {
                    let line = line?;
                    if config.number_lines {
                        println!("{:>6}\t{}", line_num + 1, line);
                    } else if config.number_nonblank_lines {
                        if line.is_empty() {
                            println!();
                        } else {
                            last_num += 1;
                            println!("{:>6}\t{}", last_num, line);
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
            Err(e) => eprintln!("Failed to open {}: {}", filename, e),
        }
    }
    Ok(())
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
