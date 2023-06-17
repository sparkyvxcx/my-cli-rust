use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::{error::Error, path::PathBuf, unimplemented};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty fortune")
        .arg(
            Arg::with_name("sources")
                .value_name("FILES")
                .help("Input files or directories")
                .multiple(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Pattern")
                .short("m")
                .long("pattern")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .value_name("SEED")
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

    let sources = matches.values_of_lossy("source").unwrap();
    let pattern = matches
        .value_of("pattern")
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", val))
        })
        .transpose()?;
    let seed = matches.value_of("seed").map(parse_seed_num).transpose()?;

    Ok(Config {
        sources,
        pattern,
        seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

fn parse_seed_num(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

fn find_files(path: &[String]) -> MyResult<Vec<PathBuf>> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_seed_num};

    #[test]
    fn test_parse_seed_num() {
        let res = parse_seed_num("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_seed_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_seed_num("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file knonw to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
    }
}
