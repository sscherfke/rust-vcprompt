extern crate clap;

use clap::{Arg, App};
use std::process::Command;


fn main() {
    let matches = App::new("vcprompt")
        .version("0.1")
        .about("Version control information in your prompt")
        .arg(Arg::with_name("cwd")
             .short("c")
             .long("cwd")
             .help("Use this directory as CWD")
             .takes_value(true))
        .get_matches();
    let cwd = matches.value_of("cwd").unwrap_or(".");
    println!("Cwd: {}", cwd);

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
        return;
    }

    let output = String::from_utf8_lossy(&result.stdout);
    let mut branch = "<unknown>";
    let mut ahead = 0;
    let mut behind = 0;
    let mut conflicts = 0;
    let mut untracked = 0;
    let mut changed = 0;
    let mut staged = 0;

    for line in output.lines() {
        let parts: Vec<&str> = line.split(" ").collect();
        // See https://git-scm.com/docs/git-status
        match parts[0] {
            "#" => match parts[1] {
                "branch.head" => branch = parts[2],
                "branch.ab" => {
                    ahead = parts[2].parse::<i32>().unwrap().abs();
                    behind = parts[3].parse::<i32>().unwrap().abs();
                },
                _ => (),
            },
            "1" | "2" => {
                // submodule state has len 4
                if parts[1].len() == 2 {
                    if !parts[1].starts_with(".") {
                        staged += 1;
                    }
                    if !parts[1].ends_with(".") {
                        changed += 1;
                    }
                }
            },
            "u" => conflicts += 1,
            "?" => untracked += 1,
            _ => (),
        }

    }
    println!("Branch: {}", branch);
    println!("Ahead: {}, behind: {}", ahead, behind);
    println!("Conflicts: {}", conflicts);
    println!("Untracked: {}", untracked);
    println!("Changed:   {}", changed);
    println!("Staged:    {}", staged);
}
