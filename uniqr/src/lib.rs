use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.0.1")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty Uniq")
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Show counts")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .takes_value(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file")
                .takes_value(true),
        )
        .get_matches();

    let count = matches.is_present("count");
    let in_file = matches.value_of("in_file").unwrap().to_string();
    let out_file = matches.value_of("out_file").map(String::from);

    Ok(Config {
        count,
        in_file,
        out_file,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut previous = String::new();
    let mut count = 0;
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            if config.count {
                print!("{:>4} {}", count, previous);
            } else {
                print!("{}", previous);
            }
            break;
        }
        /*
        if let Some(&count) = hm.get_mut(&line) {
            count += 1;
        } else {
            hm.insert(line, 1);
        }
        */
        if previous == "" {
            previous = line.clone();
            count += 1;
        } else if previous.trim() != line.trim() {
            if config.count {
                print!("{:>4} {}", count, previous);
            } else {
                print!("{}", previous);
            }

            previous = line.clone();
            count = 0;
        } else {
            count += 1;
        }
        line.clear();
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
