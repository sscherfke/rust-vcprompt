mod git;
mod hg;
mod util;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use util::Status;


/// Available formatting styles
enum OutputStyle {
    Detailed,
    Minimal,
}

/// Supported version control systems
#[derive(Clone)]
enum VCS {
    Git,
    Hg,
    None,
}

impl VCS {
    fn get_status(self, rootdir: Option<PathBuf>) -> Option<Status> {
        match self {
            VCS::Git => Some(git::status(rootdir.unwrap())),
            VCS::Hg => Some(hg::status(rootdir.unwrap())),
            VCS::None => None,
        }
    }
}


/// Determine the inner most VCS.
///
/// This functions works for nest (sub) repos and always returns
/// the most inner repository type.
fn get_vcs() -> (VCS, Option<PathBuf>) {
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
                return ((*vcs).clone(), Some(path));
            }
        }
        cwd = path.parent().map(|p| PathBuf::from(p));
    }
    (VCS::None, None)
}


/// Format and print the current VC status
fn print_result(status: &Status, style: OutputStyle) {
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
        ("VCP_PREFIX", " "),
        ("VCP_SUFFIX", "{reset}"),
        ("VCP_SEPARATOR", "|"),
        ("VCP_NAME", "{symbol}"),  // value|symbol
        ("VCP_BRANCH", "{blue}{value}{reset}"),
        ("VCP_OPERATION", "{red}{value}{reset}"),
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

    let mut output = match style {
        OutputStyle::Detailed => format_full(&status, &variables),
        OutputStyle::Minimal => format_minimal(&status, &variables),
    };

    for (k, v) in colors.iter() {
        output = output.replace(k, v);
    }
    println!("{}", output);
}

/// Format *status* in detailed style
/// (`{name}{branch}{branch tracking}|{local status}`).
fn format_full(status: &Status, variables: &HashMap<&str, String>) -> String {
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
    for op in status.operations.iter() {
        output.push_str(&variables.get("VCP_SEPARATOR").unwrap());
        output.push_str(&variables.get("VCP_OPERATION").unwrap()
                        .replace("{value}", op));
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
    if status.is_clean() {
        output.push_str(&variables.get("VCP_CLEAN").unwrap());
    }
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    output
}

/// Format *status* in minimal style
/// (`{branch}{colored_symbol}`).
fn format_minimal(status: &Status, variables: &HashMap<&str, String>) -> String {
    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
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
    if status.is_clean() {
        output.push_str("{bold}{green}");
    } else if status.staged > 0 {
        output.push_str("{bold}{red}");
    } else {
        output.push_str("{bold}{yellow}");
    }
    output.push_str(&status.symbol);
    output.push_str("{reset}");
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    output
}

/// Print vcprompt's help message.
///
/// *name* is the name with which the program has been invoked.
fn print_help(name: &str) {
    println!("Usage: {} [OPTIONS]

    Print version control information for use in your shell prompt.

Options:
  -h, --help        Show this message and exit.
  -m, --minimal     Use minimal format instead of full format.",
  name);
}


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && ["-h", "--help"].contains(&args[1].as_str()) {
        print_help(&args[0]);
        return
    }

    let style = if args.len() > 1 && args[1] == "--minimal" {
        OutputStyle::Minimal
    } else {
        OutputStyle::Detailed
    };

    let (vcs, rootdir) = get_vcs();
    vcs.get_status(rootdir).map(|r| print_result(&r, style));
}
