use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    source: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty fortune")
        .arg(
            Arg::with_name("source")
                .value_name("FILES")
                .help("Input files or directories")
                .multiple(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pattern")
                .help("Pattern")
                .value_name("PATTERN")
                .short("m")
                .long("pattern")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .help("Random seed")
                .short("s")
                .long("seed")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive pattern matching")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .get_matches();

    let source = matches.values_of_lossy("source").unwrap();
    let pattern = match matches.value_of("pattern") {
        Some(s) => {
            let pattern = RegexBuilder::new(s)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", s))?;
            Some(pattern)
        }
        _ => None,
    };
    let seed = matches
        .value_of("seed")
        .map(parse_seed_num)
        .transpose()
        .map_err(|e| format!("illegal seed number -- {}", e))?;

    Ok(Config {
        source,
        pattern,
        seed,
    })
}

fn parse_seed_num(val: &str) -> MyResult<u64> {
    match val.parse::<u64>() {
        Ok(num) => Ok(num),
        _ => Err(From::from(val)),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
