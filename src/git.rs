//! Get Git status
use std::process::Command;

use util::Status;


/// Get the status for `cwd`
pub fn git(cwd: &str) -> Option<Status> {
    let status = get_status(cwd);
    match status {
        Some(s) => Some(parse_status(&s)),
        None => None,
    }
}

/// Run `git status` and return its output if we are in a Git repo.
fn get_status(cwd: &str) -> Option<String> {
    let result = Command::new("git")
                .arg("status")
                .arg("--porcelain=2")
                .arg("--branch")
                .arg("--untracked-files")
                // .arg("--ignored")
                .current_dir(cwd)
                .output()
                .expect("failed to execute process");

    if !result.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&result.stdout).into_owned())
}

/// Parse the output string `get_status()`.
fn parse_status(status: &str) -> Status {
    let mut result = Status::new();

    for line in status.lines() {
        let parts: Vec<&str> = line.split(" ").collect();
        // See https://git-scm.com/docs/git-status
        match parts[0] {
            "#" => match parts[1] {
                "branch.head" => result.branch = parts[2].to_string(),
                "branch.ab" => {
                    result.ahead = parts[2].parse::<i32>().unwrap().abs() as u32;
                    result.behind = parts[3].parse::<i32>().unwrap().abs() as u32;
                },
                _ => (),
            },
            "1" | "2" => {
                // submodule state has len 4
                if parts[1].len() == 2 {
                    if !parts[1].starts_with(".") {
                        result.staged += 1;
                    }
                    if !parts[1].ends_with(".") {
                        result.changed += 1;
                    }
                }
            },
            "u" => result.conflicts += 1,
            "?" => result.untracked += 1,
            _ => (),
        }

    }

    result
}
