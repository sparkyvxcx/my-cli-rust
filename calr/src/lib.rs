use chrono::NaiveDate;
use clap::{App, Arg};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty cal")
        .arg(
            Arg::with_name("current_year")
                .help("Show whole current year")
                .short("y")
                .long("year")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("month")
                .help("Month name or number (1-12)")
                .value_name("MONTH")
                .short("m")
                .required(false),
        )
        .arg(
            Arg::with_name("year")
                .help("Year (1-9999)")
                .value_name("YEAR")
                .takes_value(true),
        )
        .get_matches();

    // dummy placeholder data
    let month = Some(1);
    let year = 1;
    let today = NaiveDate::parse_from_str(matches.value_of("year").unwrap(), "%Y")?;
    Ok(Config { month, year, today })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
