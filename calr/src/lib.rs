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

#[allow(deprecated)]
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

fn parse_year(year: &str) -> MyResult<i32> {
    match parse_int(year) {
        Ok(year) => {
            if year < 1 || year > 9999 {
                return Err(From::from(format!(
                    "year \"{}\" not in the range 1 through 9999",
                    year
                )));
            } else {
                Ok(year)
            }
        }
        Err(e) => Err(e),
    }
}

use chrono::Month;

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(month_num) => {
            if month_num < 1 || month_num > 12 {
                return Err(From::from(format!(
                    "month \"{}\" not in the range 1 through 12",
                    month
                )));
            } else {
                Ok(month_num)
            }
        }
        Err(_) => {
            let month = month.parse::<Month>().map_err(|_| -> Box<dyn Error> {
                From::from(format!("Invalid month \"{}\"", month))
            })?;

            Ok(month.number_from_month())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_int, parse_month, parse_year};

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

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999_i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12_u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }
}
