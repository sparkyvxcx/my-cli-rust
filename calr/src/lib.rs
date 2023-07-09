use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use itertools::izip;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
                .takes_value(true),
        )
        .get_matches();

    let today = Local::today();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;

    if matches.is_present("show_current_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.naive_local(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?}", config);
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:>32}", config.year);
            let months: Vec<_> = (1..=12)
                .into_iter()
                .map(|month| format_month(config.year, month, false, config.today))
                .collect();
            for (index, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) {
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    // print the next row of months
                    if index < 3 {
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_month_mine(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let mut month_vec = vec![];
    let line2 = "Su Mo Tu We Th Fr Sa ";
    let line1 = if print_year {
        format!("{} {}", MONTH_NAMES[(month - 1) as usize], year)
    } else {
        format!("{}", MONTH_NAMES[(month - 1) as usize])
    };
    let spaces_len = 20 - line1.len();
    let width = spaces_len / 2;
    if spaces_len % 2 == 0 {
        month_vec.push(format!(
            "{}{}{}",
            " ".repeat(width),
            line1,
            " ".repeat(width + 1)
        ))
    } else {
        month_vec.push(format!(
            "{}{}{}",
            " ".repeat(width),
            line1,
            " ".repeat(width + 2)
        ))
    }
    month_vec.push(line2.to_owned());
    month_vec
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .into_iter()
        .map(|_| "  ".to_string())
        .collect();

    let is_today =
        |days: u32| year == today.year() && month == today.month() && days == today.day();

    let last = last_day_in_month(year, month).unwrap();
    days.extend((first.day()..=last.day()).into_iter().map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    let mut lines = Vec::with_capacity(8);
    let month_name = MONTH_NAMES[month as usize - 1];

    // lines.push(format!(
    //     "{:^20}  ",
    //     if print_year {
    //         format!("{} {}", month_name, year)
    //     } else {
    //         month_name.to_string()
    //     }
    // ));

    let line1 = if print_year {
        format!("{} {}", month_name, year)
    } else {
        format!("{}", month_name)
    };
    let spaces_len = 20 - line1.len();
    let width = spaces_len / 2;
    if spaces_len % 2 == 0 {
        lines.push(format!(
            "{}{}{}  ",
            " ".repeat(width),
            line1,
            " ".repeat(width)
        ))
    } else {
        lines.push(format!(
            "{}{}{}  ",
            " ".repeat(width),
            line1,
            " ".repeat(width + 1)
        ))
    }

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        lines.push(format!("{:width$}  ", week.join(" "), width = 22 - 2));
    }

    while lines.len() < 8 {
        lines.push(" ".repeat(22));
    }

    lines
}

#[allow(dead_code)]
fn last_day_in_month_mine(year: i32, month: u32) -> Option<NaiveDate> {
    let normal_months: HashMap<u32, u32> = HashMap::from([
        (1, 31),
        (3, 31),
        (4, 30),
        (5, 31),
        (6, 31),
        (7, 31),
        (8, 31),
        (9, 30),
        (10, 31),
        (11, 30),
        (12, 31),
    ]);
    if let Some(&day) = normal_months.get(&month) {
        NaiveDate::from_ymd_opt(year, month, day)
    } else if year % 4 == 0 {
        NaiveDate::from_ymd_opt(year, month, 29)
    } else {
        NaiveDate::from_ymd_opt(year, month, 28)
    }
}

fn last_day_in_month(year: i32, month: u32) -> Option<NaiveDate> {
    // The first day of the next month...
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap().pred_opt()
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
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            } else {
                Ok(month_num)
            }
        }
        _ => {
            let month_lowercase = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(index, name)| {
                    if name.to_lowercase().starts_with(month_lowercase) {
                        Some(index + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month};
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
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
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
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31)
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29)
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30)
        );
        assert_eq!(
            last_day_in_month(2019, 2),
            NaiveDate::from_ymd_opt(2019, 2, 28)
        );
    }
}
