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
    let mut result = Status::new("git", "±");

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
                // We can ignore the submodule state as it is also indicated
                // by ".M", so we already track it as a change.
                if !parts[1].starts_with(".") {
                    result.staged += 1;
                }
                if !parts[1].ends_with(".") {
                    result.changed += 1;
                }
            },
            "u" => result.conflicts += 1,
            "?" => result.untracked += 1,
            _ => (),
        }

    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_full() {
        let output = "
# branch.oid dc716b061d9a0bc6a59f4e02d72b9952cce28927
# branch.head master
# branch.upstream origin/master
# branch.ab +1 -2
1 .M <sub> <mH> <mI> <mW> <hH> <hI> modified.txt
1 .D <sub> <mH> <mI> <mW> <hH> <hI> deleted.txt
1 M. <sub> <mH> <mI> <mW> <hH> <hI> staged.txt
1 MM <sub> <mH> <mI> <mW> <hH> <hI> staged_modified.txt
1 MD <sub> <mH> <mI> <mW> <hH> <hI> staged_deleted.txt
1 A. <sub> <mH> <mI> <mW> <hH> <hI> added.txt
1 AM <sub> <mH> <mI> <mW> <hH> <hI> added_modified.txt
1 AD <sub> <mH> <mI> <mW> <hH> <hI> added_deleted.txt
1 D. <sub> <mH> <mI> <mW> <hH> <hI> deleted.txt
1 DM <sub> <mH> <mI> <mW> <hH> <hI> deleted_modified.txt
2 R. <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 RM <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 RD <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 C. <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 CM <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 CD <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
u UU <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
? untracked.txt
! ignored.txt
";
        let mut expected = Status::new("git", "±");
        expected.branch = "master".to_string();
        expected.ahead = 1;
        expected.behind = 2;
        expected.staged = 14;
        expected.changed = 11;
        expected.untracked = 1;
        expected.conflicts = 1;
        assert_eq!(parse_status(output), expected);
    }

    #[test]
    fn parse_status_clean() {
        let output = "
# branch.oid dc716b061d9a0bc6a59f4e02d72b9952cce28927
# branch.head master
";
        let mut expected = Status::new("git", "±");
        expected.branch = "master".to_string();
        assert_eq!(parse_status(output), expected);
    }

    #[test]
    fn parse_status_emty() {
        assert_eq!(parse_status(""), Status::new("git", "±"));
    }
}
