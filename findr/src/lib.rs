use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.0.1")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty Find")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .help("Search paths [default: .]"),
        )
        .arg(
            Arg::with_name("name")
                .value_name("NAME")
                .short("n")
                .long("name")
                .help("Name")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("type")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .help("Entry type [possible values: f, d, l]")
                .possible_values(&["f", "d", "l"])
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let paths: Vec<String> = matches
        .values_of_lossy("path")
        .unwrap_or(vec![".".to_string()]);
    let names: Vec<Regex> = matches
        .values_of_lossy("name")
        .unwrap_or(vec![])
        .iter()
        .map(|p| Regex::new(&p).unwrap())
        .collect();
    let entry_types = matches
        .values_of_lossy("type")
        .unwrap_or(vec![])
        .iter()
        .map(|t| match &t[..] {
            "f" => File,
            "d" => Dir,
            "l" => Link,
            _ => panic!("unexpected input error"),
        })
        .collect();

    Ok(Config {
        paths,
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
