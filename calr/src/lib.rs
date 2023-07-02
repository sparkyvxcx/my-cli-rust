use chrono::{Datelike, Local, Month, NaiveDate};
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
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .help("Month name or number (1-12)")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("show_current_year")
                .short("y")
                .long("year")
                .help("Show whole current year")
                .conflicts_with_all(&["month", "year"])
                .takes_value(false),
        )
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1-9999)")
                .conflicts_with("month")
                .takes_value(true),
        )
        .get_matches();

    let today = Local::today();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches
        .value_of("year")
        .map(parse_year)
        .transpose()?
        .unwrap_or(today.year());

    if matches.is_present("show_current_year") {
        month = None;
        year = today.year();
    } else if month.is_none() {
        month = Some(today.month())
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

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    unimplemented!();
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| From::from(format!("Invalid integer \"{}\"", val)))
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

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
            let month = month
                .parse::<Month>()
                .map_err(|_| -> Box<dyn Error> { format!("Invalid month \"{}\"", month).into() })?;

            Ok(month.number_from_month())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::format_month;

    use super::{parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

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

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd(0, 1, 1);
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            " May ",
            "Su Mo Tu We Th Fr Sa ",
            "                1  2 ",
            " 3  4  5  6  7  8  9 ",
            "10 11 12 13 14 15 16 ",
            "17 18 19 20 21 22 23 ",
            "24 25 26 27 28 29 30 ",
            "31 ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            " April 2021 ",
            "Su Mo Tu We Th Fr Sa ",
            " 1 2 3 ",
            " 4 5 6 \u{1b}[7m 7\u{1b}[0m 8 9 10 ",
            "11 12 13 14 15 16 17 ",
            "18 19 20 21 22 23 24 ",
            "25 26 27 28 29 30 ",
            " ",
        ];
        let today = NaiveDate::from_ymd(2021, 4, 7);
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
}
