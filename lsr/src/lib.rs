use chrono::{DateTime, Local};
use clap::{App, Arg};
use std::error::Error;
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use tabular::{Row, Table};

type MyReuslt<T> = Result<T, Box<dyn Error>>;

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
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
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

fn format_output(paths: &[PathBuf]) -> MyReuslt<String> {
    //         1   2     3     4     5     6     7     8
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let path_metadata = path.metadata()?;
        let position_1 = format!(
            "{}{}",
            if path_metadata.is_dir() { "d" } else { "-" },
            format_mode(path_metadata.mode())
        );
        let path_usr = format!(
            "{}",
            users::get_user_by_uid(path_metadata.uid())
                .unwrap()
                .name()
                .to_str()
                .unwrap()
        );
        let length = path_metadata.len();
        let path_grp = format!(
            "{}",
            users::get_group_by_gid(path_metadata.gid())
                .unwrap()
                .name()
                .to_str()
                .unwrap()
        );
        let modified_time: DateTime<Local> = DateTime::from(path_metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(position_1)
                .with_cell("")
                .with_cell(path_metadata.nlink())
                .with_cell(path_usr)
                .with_cell(path_grp)
                .with_cell(length)
                .with_cell(modified_time.format("%b %d %y %H:%M"))
                .with_cell(path.display().to_string()),
        );
    }

    Ok(format!("{}", table))
}

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
fn format_mode(mode: u32) -> String {
    let mode_map = [0o400, 0o040, 0o004];
    let mut perms = String::new();

    for each_mask in mode_map {
        // check read permission
        if mode & each_mask == each_mask {
            perms.push('r')
        } else {
            perms.push('-')
        }

        // check write permission
        if mode & each_mask >> 1 == each_mask >> 1 {
            perms.push('w')
        } else {
            perms.push('-')
        }

        // check execute permission
        if mode & each_mask >> 2 == each_mask >> 2 {
            perms.push('x')
        } else {
            perms.push('-')
        }
    }

    perms
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::format_output;

    use super::{find_files, format_mode};

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10);

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

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

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_lines = lines.remove(0);
        long_match(
            &empty_lines,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None)
    }
}
