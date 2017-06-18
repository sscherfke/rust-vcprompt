mod git;
mod util;

extern crate clap;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;


use clap::{Arg, App};

use util::Status;


/// Supported version control systems
#[derive(Clone)]
enum VCS {
    Git,
    Hg,
    None,
}

impl VCS {
    fn get_status(self, cwd: &str) -> Option<Status> {
        match self {
            VCS::Git => git::git(cwd),
            VCS::Hg => None,
            VCS::None => None,
        }
    }
}


/// Determine the inner most VCS.
///
/// This functions works for nest (sub) repos and always returns
/// the most inner repository type.
fn get_vcs() -> VCS {
    let vcs_files = [
        (VCS::Git, ".git/HEAD"),
        (VCS::Hg, ".hg/00changelog.i"),
    ];

    let mut cwd = Some(env::current_dir().unwrap());
    while let Some(path) = cwd {
        for &(ref vcs, vcs_file) in vcs_files.iter() {
            let mut fname = path.clone();
            fname.push(vcs_file);
            if fname.exists() {
                return (*vcs).clone();
            }
        }
        cwd = path.parent().map(|p| PathBuf::from(p));
    }
    VCS::None
}


/// Format and print the current VC status
fn print_result(status: &Status) {
    let colors: HashMap<&str, &str> = [
        ("{reset}", "\x1B[00m"),
        ("{bold}", "\x1B[01m"),
        ("{black}", "\x1B[30m"),
        ("{red}", "\x1B[31m"),
        ("{green}", "\x1B[32m"),
        ("{yellow}", "\x1B[33m"),
        ("{blue}", "\x1B[34m"),
        ("{magenta}", "\x1B[35m"),
        ("{cyan}", "\x1B[36m"),
        ("{white}", "\x1B[37m"),
    ].iter().cloned().collect();

    let mut variables: HashMap<&str, String> = [
        ("VCP_PREFIX", "("),
        ("VCP_SUFFIX", "{reset})"),
        ("VCP_SEPARATOR", "|"),
        ("VCP_NAME", "{green}{bold}{symbol}{reset}"),  // value|symbol
        ("VCP_BRANCH", "{value}"),
        ("VCP_BEHIND", "↓{value}"),
        ("VCP_AHEAD", "↑{value}"),
        ("VCP_STAGED", "{red}●{value}"),
        ("VCP_CONFLICTS", "{red}✖{value}"),
        ("VCP_CHANGED", "{blue}✚{value}"),
        ("VCP_UNTRACKED", "{reset}…{value}"),
        ("VCP_CLEAN", "{green}{bold}✔"),
    ].iter().map(|&(k, v)| (k, v.to_string())).collect();

    for (k, v) in variables.iter_mut() {
        match env::var(k) {
            Ok(val) => *v = val,
            _ => (),
        };
    }

    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
    output.push_str(&variables.get("VCP_NAME").unwrap()
                    .replace("{value}", &status.name)
                    .replace("{symbol}", &status.symbol));
    output.push_str(&variables.get("VCP_BRANCH").unwrap()
                    .replace("{value}", &status.branch));
    if status.behind > 0 {
        output.push_str(&variables.get("VCP_BEHIND").unwrap()
                        .replace("{value}", &status.behind.to_string()));
    }
    if status.ahead > 0 {
        output.push_str(&variables.get("VCP_AHEAD").unwrap()
                        .replace("{value}", &status.ahead.to_string()));
    }
    output.push_str(&variables.get("VCP_SEPARATOR").unwrap());
    if status.staged > 0 {
        output.push_str(&variables.get("VCP_STAGED").unwrap()
                        .replace("{value}", &status.staged.to_string()));
    }
    if status.conflicts > 0 {
        output.push_str(&variables.get("VCP_CONFLICTS").unwrap()
                        .replace("{value}", &status.conflicts.to_string()));
    }
    if status.changed > 0 {
        output.push_str(&variables.get("VCP_CHANGED").unwrap()
                        .replace("{value}", &status.changed.to_string()));
    }
    if status.untracked > 0 {
        output.push_str(&variables.get("VCP_UNTRACKED").unwrap()
                        .replace("{value}", &status.untracked.to_string()));
    }
    if status.staged == 0 && status.conflicts == 0 && status.changed == 0 && status.untracked == 0 {
        output.push_str(&variables.get("VCP_CLEAN").unwrap());
    }
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    for (k, v) in colors.iter() {
        output = output.replace(k, v);
    }
    println!("{}", output);
}


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

    get_vcs().get_status(cwd).map(|r| print_result(&r));
}
