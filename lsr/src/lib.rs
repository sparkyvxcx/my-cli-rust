mod owner;

use chrono::{DateTime, Local};
use clap::{App, Arg};
use owner::Owner;
use std::error::Error;
use std::fs;
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

#[allow(dead_code)]
fn find_files_og(paths: &[String], show_hidden: bool) -> MyReuslt<Vec<PathBuf>> {
    let mut results = vec![];

    for name in paths {
        match fs::metadata(name) {
            Ok(file_metadata) => {
                if file_metadata.is_dir() {
                    results.extend(
                        fs::read_dir(name)?
                            .into_iter()
                            .filter_map(Result::ok)
                            .filter(|e| {
                                show_hidden || !e.file_name().to_str().unwrap().starts_with(".")
                            })
                            .map(|e| e.path().into()),
                    );
                } else {
                    results.push(PathBuf::from(name));
                }
            }
            Err(e) => eprintln!("{}: {}", name, e),
        }
    }

    Ok(results)
}

fn find_files(paths: &[String], show_hidden: bool) -> MyReuslt<Vec<PathBuf>> {
    let mut valid_paths = vec![];
    for each_path in paths {
        let path = Path::new(each_path);
        if let Err(e) = path.metadata() {
            eprintln!("{}: {}", path.display(), e);
            continue;
        }

        if path.is_file() {
            valid_paths.push(path.to_owned());
        } else {
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
        let file_type = format!("{}", if path_metadata.is_dir() { "d" } else { "-" },);
        let uid = path_metadata.uid();
        let path_usr = users::get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());
        let length = path_metadata.len();
        let gid = path_metadata.gid();
        let path_grp = users::get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());
        let perms = format_mode(path_metadata.mode());
        let modified_time: DateTime<Local> = DateTime::from(path_metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(file_type)
                .with_cell(perms)
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

#[allow(dead_code)]
fn formate_mode_og(mode: u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other),
    )
}

/// Given a file mode in octal formate like 0o751,
/// return a string like "rwxr-x--x"
pub fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Owner;
    use super::{find_files, format_mode, format_output, mk_triple};

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

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::User), "rw-");
        assert_eq!(mk_triple(0o600, Owner::Group), "---");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }
}
