use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn run(args: &[&str], expected_file: &str) -> TestResult {
    let expected = fs::read_to_string(expected_file)?;
    Command::cargo_bin("echor")?.args(args).assert().success().stdout(expected);
    Ok(())
}

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.assert().failure().stderr(predicate::str::contains("USAGE"));
    Ok(())
}

#[test]
fn runs() {
    let mut cmd = Command::cargo_bin("echor").unwrap();
    cmd.arg("hello world").assert().success();
}

#[test]
fn hello1() -> TestResult {
    let outfile = "tests/expected/hello1.txt";
    run(&["Hello there"], outfile)
}

#[test]
fn hello2() -> TestResult {
    let outfile = "tests/expected/hello2.txt";
    run(&["Hello", "there"], outfile)
}

#[test]
fn hello1_no_newline() -> TestResult {
    run(&["Hello  there", "-n"], "tests/expected/hello1.n.txt")
}

#[test]
fn hello2_no_newline() -> TestResult {
    run(&["-n", "Hello", "there"], "tests/expected/hello2.n.txt")
}
