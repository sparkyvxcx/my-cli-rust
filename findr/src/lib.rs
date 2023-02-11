use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

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
                .help("Search paths")
                .default_value(".")
                .multiple(true),
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
                .help("Entry type")
                .possible_values(&["f", "d", "l"])
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let paths: Vec<String> = matches.values_of_lossy("path").unwrap();
    /*
    let names: Vec<Regex> = matches
        .values_of_lossy("name")
        .map(|vals| {
            vals.into_iter()
                .map(|name| Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    */
    let names: Vec<Regex> = matches
        .values_of_lossy("name")
        .unwrap_or(vec![])
        .into_iter()
        .map(|name| Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name)))
        .collect::<Result<Vec<_>, _>>()?;

    let entry_types = matches
        .values_of_lossy("type")
        .unwrap_or(vec![])
        .iter()
        .map(|t| match &t[..] {
            "f" => File,
            "d" => Dir,
            "l" => Link,
            _ => unreachable!("Invalid type"),
        })
        .collect();

    Ok(Config {
        paths,
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?}", config);

    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    if !config.names.is_empty()
                        && config
                            .names
                            .iter()
                            .filter(|name| name.is_match(entry.file_name().to_str().unwrap()))
                            .count()
                            == 0
                    {
                        continue;
                    }
                    if config.entry_types.is_empty()
                        || config
                            .entry_types
                            .iter()
                            .any(|each_entry| match each_entry {
                                File => entry.file_type().is_file(),
                                Dir => entry.file_type().is_dir(),
                                Link => entry.file_type().is_symlink(),
                            })
                    {
                        println!("{}", entry.path().display());
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }
    Ok(())
}
