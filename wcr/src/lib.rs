use clap::{App, Arg};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    for file in config.files {
        println!("{}", file);
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let lines_help = "Show line count";
    let bytes_help = "Show byte count";
    let chars_help = "Show character count";
    let words_help = "Show word count";

    let matches = App::new("wcr")
        .version("0.1.0")
        .author("KenYoens-Clar<yclar@gmail.com>")
        .about("Rusty Word Count")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help(lines_help),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help(words_help),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help(chars_help),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help(bytes_help)
                .conflicts_with("chars"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches.is_present("lines");
    let words = matches.is_present("words");
    let bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    Ok(Config {
        files,
        lines,
        words,
        bytes,
        chars,
    })
}
