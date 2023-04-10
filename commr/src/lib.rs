use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty comm")
        .arg(
            Arg::with_name("file1")
                .value_name("FLIE1")
                .help("Input file 1")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("file2")
                .value_name("FLIE2")
                .help("Input file 2")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("suppress_col1")
                .short("1")
                .help("Suppress printing of column 1")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("suppress_col2")
                .short("2")
                .help("Suppress printing of column 2")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("suppress_col3")
                .short("3")
                .help("Suppress printing of column 3")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive comparison of lines")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delimiter")
                .help("Output delimiter")
                .short("d")
                .long("output-delimiter")
                .value_name("DELIM")
                .default_value("\t"),
        )
        .get_matches();

    let file1 = matches.value_of("file1").unwrap().to_owned();
    let file2 = matches.value_of("file2").unwrap().to_owned();

    let show_col1 = !matches.is_present("show_col1");
    let show_col2 = !matches.is_present("show_col2");
    let show_col3 = !matches.is_present("show_col3");

    let insensitive = matches.is_present("insensitive");
    let delimiter = matches.value_of("delimiter").unwrap().to_owned();

    Ok(Config {
        file1,
        file2,
        show_col1,
        show_col2,
        show_col3,
        insensitive,
        delimiter,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:#?}", config);
    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
