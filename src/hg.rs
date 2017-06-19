//! Get Mercurial (hg) status
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::io::prelude::*;

use util::Status;


/// Get the status for the cwd
pub fn status(rootdir: PathBuf) -> Status {
    let status_str = get_status();
    let mut status = parse_status(&status_str);
    status.branch = get_branch(&rootdir) + &get_bookmark(&rootdir);
    status
}

/// Run `hg status` and return its output.
fn get_status() -> String {
    let result = Command::new("hg")
                .arg("status")
                .arg("--color=false")
                .arg("--pager=false")
                .output()
                .expect("Failed to execute \"hg\"");

    let output = String::from_utf8_lossy(&result.stdout).into_owned();

    if !result.status.success() {
        panic!("git status failed: {}", &output);
    }

    output
}

/// Parse the output string of `get_status()`.
fn parse_status(status: &str) -> Status {
    let mut result = Status::new("hg", "☿");

    for line in status.lines() {
        match line.split(" ").next() {
            Some("M") | Some("A") | Some("R") | Some("!") => result.staged += 1,
            Some("?") => result.untracked += 1,
            _ => (),
        }
    }

    result
}

/// Return the current branch
fn get_branch(rootdir: &PathBuf) -> String {
    let mut path = rootdir.clone();
    path.push(".hg/branch");
    match File::open(path) {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            contents.trim().to_string()
        },
        Err(_) => "default".to_string(),
    }
}

/// Return the current bookmark or an empty string
fn get_bookmark(rootdir: &PathBuf) -> String {
    let mut path = rootdir.clone();
    path.push(".hg/bookmarks.current");
    match File::open(path) {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            "*".to_string() + contents.trim()
        },
        Err(_) => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_full() {
        let output = "
M modified.txt
A added.txt
R removed.txt
C clean.txt
? untracked.txt
! deleted.txt
I ignored.txt
";
        let mut expected = Status::new("hg", "☿");
        expected.branch = "<unknown>".to_string();
        expected.ahead = 0;
        expected.behind = 0;
        expected.staged = 4;
        expected.changed = 0;
        expected.untracked = 1;
        expected.conflicts = 0;
        assert_eq!(parse_status(output), expected);
    }

    #[test]
    fn parse_status_clean() {
        assert_eq!(parse_status(""), Status::new("hg", "☿"));
    }
}
