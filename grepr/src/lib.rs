use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("recursive")
                .help("Recursive search")
                .short("r")
                .long("recursive"),
        )
        .arg(
            Arg::with_name("count")
                .help("Count occurrences")
                .short("c")
                .long("count"),
        )
        .arg(
            Arg::with_name("invert")
                .help("Invert match")
                .short("v")
                .long("invert-match"),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive")
                .short("i")
                .long("insensitive"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let pattern = matches
        .value_of("pattern")
        .map(|p| Regex::new(&p).map_err(|_| format!("Invalid pattern \"{}\"", p)))
        .transpose()?
        .unwrap();
    let recursive = matches.is_present("recursive");
    let invert_match = matches.is_present("invert");
    let count = matches.is_present("count");
    Ok(Config {
        pattern,
        files,
        recursive,
        count,
        invert_match,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
