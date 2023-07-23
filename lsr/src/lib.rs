use clap::{App, Arg};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

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

fn find_files(paths: &[String], show_hidden: bool) -> MyReuslt<Vec<PathBuf>> {
    let mut valid_paths = vec![];
    for each_path in paths {
        let path = Path::new(each_path);
        path.metadata()
            .map_err(|e| format!("{}: {}", path.display(), e))?;

        if path.is_file() {
            valid_paths.push(path.to_owned());
        } else {
            // let mut entries = vec![];
            // for entry in new_path.read_dir()? {
            //     if let Ok(entry) = entry {
            //         // println!("{:?}", entry.path());
            //         valid_paths.push(entry.path());
            //     }
            // }
            valid_paths.extend(
                path.read_dir()?
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| show_hidden || !e.file_name().to_str().unwrap().starts_with("."))
                    .map(|e| e.path().into()),
            );
        }
    }
    valid_paths.sort();
    valid_paths.dedup();

    Ok(valid_paths)
}

#[cfg(test)]
mod tests {
    use super::find_files;

    #[test]
    fn test_find_files() {
        // Find all nonhidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());

        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();

        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Find all entries in a directory
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());

        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();

        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }
}
