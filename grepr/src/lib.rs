use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("recursive")
                .help("Recursive search")
                .short("r")
                .long("recursive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("count")
                .help("Count occurrences")
                .short("c")
                .long("count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("invert")
                .help("Invert match")
                .short("v")
                .long("invert-match")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let pattern = matches.value_of("pattern").unwrap();
    let pattern = RegexBuilder::new(pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", pattern))?;
    let recursive = matches.is_present("recursive");
    let invert_match = matches.is_present("invert");
    let count = matches.is_present("count");
    Ok(Config {
        pattern,
        files,
        recursive,
        count,
        invert_match,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:#?}", config);
    // println!("pattern \"{}\"", config.pattern);

    let entries = find_files(&config.files, config.recursive);
    let multiple = entries.len() > 1;
    for (idx, entry) in entries.iter().enumerate() {
        match entry {
            Ok(filename) => match open(&filename) {
                Ok(file) => {
                    let matches = find_lines(file, &config.pattern, config.invert_match)?;
                    // println!("Found {:?}", matches);
                    if config.count {
                        if multiple {
                            println!("{}:{}", filename, matches.len());
                        } else {
                            println!("{}", matches.len());
                        }
                        continue;
                    }

                    for each_match in matches {
                        if each_match.len() > 0 {
                            if multiple {
                                print!("{}:{}", filename, each_match);
                            } else {
                                print!("{}", each_match);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("{}: {}", filename, e),
            },
            Err(e) => eprintln!("{}: {}", config.files[idx], e),
        }
    }

    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    // unimplemented!()
    let mut results = vec![];
    for path in paths {
        if path == "-" {
            results.push(Ok(path.clone()));
            continue;
        }
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                results.push(Err(From::from(format!("{}", e))));
                continue;
            }
        };
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            if recursive {
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(entry) => Some(entry),
                        Err(_) => None,
                    })
                    .filter(|entry| entry.file_type().is_file())
                    .for_each(|entry| results.push(Ok(entry.path().display().to_string())));
            } else {
                results.push(Err(From::from(format!("{} is a directory", path))));
            }
        } else if file_type.is_file() {
            results.push(Ok(path.clone()));
        }
    }

    results
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut results = vec![];
    loop {
        let mut buffer = String::new();
        let size = file.read_line(&mut buffer)?;
        if size == 0 {
            break;
        }
        if pattern.is_match(&buffer) {
            if !invert_match {
                results.push(buffer);
            }
        } else {
            if invert_match {
                results.push(buffer);
            }
        }
    }

    Ok(results)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use super::find_lines;
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt"
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
