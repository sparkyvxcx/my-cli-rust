use clap::{App, Arg};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{eprintln, println};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rusty fortune")
        .arg(
            Arg::with_name("sources")
                .value_name("FILES")
                .help("Input files or directories")
                .multiple(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Pattern")
                .short("m")
                .long("pattern")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .value_name("SEED")
                .help("Random seed")
                .short("s")
                .long("seed")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive pattern matching")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .get_matches();

    let sources = matches.values_of_lossy("sources").unwrap();
    let pattern = matches
        .value_of("pattern")
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", val))
        })
        .transpose()?;
    let seed = matches.value_of("seed").map(parse_seed_num).transpose()?;

    Ok(Config {
        sources,
        pattern,
        seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    // println!("{:#?}", config);
    // println!("{:#?}", files);
    // println!("{:#?}", fortunes.last());
    if fortunes.is_empty() {
        println!("No fortunes found");
        return Ok(());
    }

    if let Some(pattern) = config.pattern {
        let mut sources = vec![];
        for fortune in fortunes {
            // Print all the fortunes matching the pattern
            if pattern.is_match(&fortune.text) {
                println!("{}\n%", fortune.text);
                sources.push(format!("({})", fortune.source));
            }
        }
        if !sources.is_empty() {
            sources.dedup();
            eprintln!("{}\n%", sources.join("\n%\n"));
        }
    } else {
        println!("{}", pick_fortune(&fortunes, config.seed).unwrap());
    }
    Ok(())
}

fn parse_seed_num(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    // unimplemented!();
    let mut valid_paths = vec![];
    for each_path in paths {
        let new_path = Path::new(each_path);
        new_path
            .metadata()
            .map_err(|e| format!("{}: {}", new_path.display(), e))?;

        if new_path.is_file() {
            match new_path.extension() {
                Some(ext) => {
                    if ext.to_str().unwrap() == "dat" {
                        continue;
                    }
                }
                None => {}
            }
            valid_paths.push(new_path.to_owned());
        } else {
            let mut entries = vec![];
            for entry in new_path.read_dir()? {
                if let Ok(entry) = entry {
                    // println!("{:?}", entry.path());
                    entries.push(entry.path().to_str().unwrap().to_string());
                }
            }
            for each_path in find_files(&entries).unwrap() {
                valid_paths.push(each_path);
            }
        }
    }
    valid_paths.sort();
    valid_paths.dedup();

    Ok(valid_paths)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = vec![];

    for path in paths {
        let mut file = BufReader::new(File::open(path)?);
        let mut buf = String::new();
        loop {
            let bytes_read = file.read_line(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
        }
        buf.split("%")
            .map(|f| f.trim())
            .filter(|f| !f.is_empty())
            .for_each(|f| {
                fortunes.push(Fortune {
                    source: path.file_name().unwrap().to_string_lossy().to_string(),
                    text: f.to_string(),
                })
            })
    }

    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let fortune = match seed {
        Some(state) => {
            let mut rng = rand::rngs::StdRng::seed_from_u64(state);
            fortunes.choose(&mut rng)
        }
        None => {
            let mut rng = rand::thread_rng();
            fortunes.choose(&mut rng)
        }
    };
    if let Some(f) = fortune {
        Some(f.text.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_seed_num, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_parse_seed_num() {
        let res = parse_seed_num("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_seed_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_seed_num("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file knonw to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string());
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string());
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\nA. Collared greens."
            );
        }

        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
