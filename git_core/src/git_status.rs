use std::path::Path;

use git2::{Delta, DiffDelta, StatusEntry};

use crate::INVALID_UTF8;

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub status: Delta,
    pub old_file: String,
    pub new_file: String,
}

impl FileStatus {
    fn from_delta(delta: &DiffDelta) -> Self {
        Self {
            status: delta.status(),
            old_file: delta
                .old_file()
                .path()
                .and_then(Path::to_str)
                .unwrap_or(INVALID_UTF8)
                .to_string(),
            new_file: delta
                .new_file()
                .path()
                .and_then(Path::to_str)
                .unwrap_or(INVALID_UTF8)
                .to_string(),
        }
    }
}


/// From <https://git-scm.com/docs/git-status> :
///
/// Displays paths that have differences between the index file and the current HEAD commit,
/// paths that have differences between the working tree and the index file,
/// and paths in the working tree that are not tracked by Git (and are not ignored by gitignore[5]).
/// The first are what you would commit by running git commit;
/// the second and third are what you could commit by running git add before running git commit.
#[derive(Debug, Clone)]
pub struct StatusSummary {
    pub branch_name: String,
    pub staged: Vec<FileStatus>,
    pub not_staged: Vec<FileStatus>,
    pub untracked: Vec<FileStatus>,
}

impl StatusSummary {
    pub fn new(branch_name: String) -> Self {
        Self {
            branch_name,
            staged: Vec::new(),
            not_staged: Vec::new(),
            untracked: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &StatusEntry) {
        if let Some(ref delta) = entry.head_to_index() {
            self.staged.push(FileStatus::from_delta(delta));
        }

        match entry.index_to_workdir().as_ref().map(FileStatus::from_delta) {
            Some(status @ FileStatus { status: Delta::Untracked, .. }) =>
                self.untracked.push(status),
            Some(status) => self.not_staged.push(status),
            None => {},
        }
    }
}


impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = format!("{:10?}", self.status).to_lowercase();

        if Delta::Renamed == self.status {
            write!(f, "{status}: {} --> {}", self.old_file, self.new_file)
        } else {
            write!(f, "{status}: {}", self.old_file)
        }
    }
}
