use clap::{App, Arg};
use std::error::Error;

type MyReuslt<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyReuslt<Config> {
    let matches = App::new("lsr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty ls")
        .arg(
            Arg::with_name("paths")
                .help("Files and/or directories")
                .value_name("PATH")
                .takes_value(true)
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("long")
                .help("Long listing")
                .long("long")
                .short("l"),
        )
        .arg(
            Arg::with_name("show_all")
                .help("Show all files")
                .long("all")
                .short("a"),
        )
        .get_matches();

    let paths = matches.values_of_lossy("paths").unwrap();
    let long = matches.is_present("long");
    let show_hidden = matches.is_present("show_all");

    Ok(Config {
        paths,
        long,
        show_hidden,
    })
}

pub fn run(config: Config) -> MyReuslt<()> {
    println!("{:?}", config);
    Ok(())
}
