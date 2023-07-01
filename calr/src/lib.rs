use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use std::error::Error;
use std::str::FromStr;

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
            Arg::with_name("whole_year")
                .help("Show whole current year")
                .short("y")
                .long("year")
                .takes_value(false)
                .conflicts_with_all(&["month", "year"]),
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
                .takes_value(true)
                .conflicts_with("month"),
        )
        .get_matches();

    // dummy placeholder data
    let mut month = matches
        .value_of("month")
        .map(|m| m.parse::<u32>())
        .transpose()?;
    let whole_year = matches.is_present("whole_year");
    let today = Local::today();

    let mut year = matches
        .value_of("year")
        .map(|y| y.parse::<i32>())
        .transpose()?
        .unwrap_or(today.year());

    if whole_year {
        month = None;
        year = today.year();
    }

    Ok(Config {
        month,
        year,
        today: today.naive_local(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse::<T>()
        .map_err(|_| From::from(format!("Invalid integer \"{}\"", val)))
}

#[cfg(test)]
mod tests {
    use super::parse_int;

    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_usize);

        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1_i32);

        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }
}
