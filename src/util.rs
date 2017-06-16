//! Commonly used utilities

/// The current VC status
#[derive(PartialEq, Debug)]
pub struct Status {
    /// The branch name
    pub branch: String,
    /// Number of revisions we are ahead of upstream
    pub ahead: u32,
    /// Number of revisions we are behind upstream
    pub behind: u32,
    /// Number of staged files
    pub staged: u32,
    /// Number of modified/added/removed files
    pub changed: u32,
    /// Number of untracked files
    pub untracked: u32,
    /// Number of conflicts
    pub conflicts: u32,
}

impl Status {
    /// Create a new instance with all values set to `<unknown>` branch and `0`.
    pub fn new() -> Status {
        Status {
            branch: "<unknown>".to_string(),
            ahead: 0,
            behind: 0,
            staged: 0,
            changed: 0,
            untracked: 0,
            conflicts: 0,
        }
    }
}
