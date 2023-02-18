use clap::{App, Arg};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Config {
    path: String,
    depth: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("treer")
        .version("0.0.1")
        .author("John Doe <sparkyvxcx@gmail.com>")
        .about("Rusty tree")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .help("Display path")
                .default_value("."),
        )
        .arg(
            Arg::with_name("level")
                .value_name("Level")
                .short("L")
                .long("level")
                .help("directory level")
                .takes_value(true),
        )
        .get_matches();

    let path = matches.value_of("path").unwrap().to_string();
    Ok(Config { path, depth: 6 })
}

pub fn run(config: Config) -> MyResult<()> {
    // echo "│   ├──"
    let mut depth = 0;
    let mut padding = String::from("");
    for entry in WalkDir::new(config.path).max_depth(2) {
        match entry {
            Ok(entry) => {
                // println!("{} {}", entry.depth(), entry.path().display());
                if entry.depth() == 0 {
                    println!("{}", entry.path().display());
                } else if entry.depth() == 1 {
                    println!("├── {}", entry.file_name().to_str().unwrap());
                } else {
                    depth = entry.depth();
                    padding = "   ".repeat(depth - 1);
                    println!("│{}├── {}", padding, entry.file_name().to_str().unwrap());
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(())
}

fn display() {}
