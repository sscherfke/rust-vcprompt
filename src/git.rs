//! Get Git status
use std::path::PathBuf;
use std::process::Command;

use util::Status;


static OPERATIONS: [(&str, &str); 6] = [
    ("rebase-merge", "REBASE"),
    ("rebase-apply", "AM/REBASE"),
    ("MERGE_HEAD", "MERGING"),
    ("CHERRY_PICK_HEAD", "CHERRY-PICKING"),
    ("REVERT_HEAD", "REVERTING"),
    ("BISECT_LOG", "BISECTING"),
];


/// Get the status for the cwd
pub fn status(rootdir: PathBuf) -> Status {
    let status = get_status();
    let mut result = parse_status(&status);
    get_operations(&mut result.operations, &rootdir);
    result
}

/// Run `git status` and return its output.
fn get_status() -> String {
    let result = Command::new("git")
                .arg("status")
                .arg("--porcelain=2")
                .arg("--branch")
                .arg("--untracked-files")
                .output()
                .expect("Failed to execute \"git\"");

    let output = String::from_utf8_lossy(&result.stdout).into_owned();

    if !result.status.success() {
        panic!("git status failed: {}", &output);
    }

    output
}

/// Parse the output string of `get_status()`.
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


/// Look for files that indicate an ongoing operation (e.g., a merge)
/// and update *list* accordingly
fn get_operations(list: &mut Vec<&str>, rootdir: &PathBuf) {
    let mut gitdir = rootdir.clone();
    gitdir.push(".git");
    for &(fname, op) in OPERATIONS.iter() {
        let mut file = gitdir.clone();
        file.push(fname);
        if file.exists() {
            list.push(op);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs::{DirBuilder,File};

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

    #[test]
    fn detect_merge() {
        let mut result = Vec::<&str>::new();
        let mut rootdir = temp_dir();
        rootdir.push("test-vcprompt");

        let mut path = rootdir.clone();
        path.push(".git");
        DirBuilder::new().recursive(true).create(path.clone()).unwrap();
        path.push("MERGE_HEAD");
        File::create(path).unwrap();

        get_operations(&mut result, &rootdir);

        assert_eq!(result, vec!["MERGING"]);
    }
}
