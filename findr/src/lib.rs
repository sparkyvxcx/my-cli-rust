use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

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
    max_depth: Option<usize>,
    min_depth: Option<usize>,
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
        .arg(
            Arg::with_name("max_depth")
                .value_name("MAX DEPTH")
                .long("max_depth")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min_depth")
                .value_name("MIN DEPTH")
                .long("min_depth")
                .takes_value(true),
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

    let str_to_usize = |v: &str| {
        let v = v.to_owned();
        Some(v.parse::<usize>().unwrap())
    };

    let max_depth = matches.value_of("max_depth").and_then(str_to_usize);
    let min_depth = matches.value_of("min_depth").and_then(str_to_usize);

    Ok(Config {
        paths,
        names,
        entry_types,
        max_depth,
        min_depth,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    let type_filter = |entry: &DirEntry| {
        config.entry_types.is_empty()
            || config
                .entry_types
                .iter()
                .any(|entry_type| match entry_type {
                    Link => entry.path_is_symlink(),
                    File => entry.file_type().is_file(),
                    Dir => entry.file_type().is_dir(),
                })
    };

    let name_filter = |entry: &DirEntry| {
        config.names.is_empty()
            || config
                .names
                .iter()
                .any(|name| name.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in &config.paths {
        let entries = WalkDir::new(path)
            .min_depth(config.min_depth.unwrap_or(0)) // follow WalkDir's default min depth value
            .max_depth(config.max_depth.unwrap_or(10)) // follow WalkDir's default max depth value
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"));
    }
    Ok(())
}
