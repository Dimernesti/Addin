pub mod git;
pub mod git_status;

pub use git::{AuthType, Config, Repo};
pub use git_status::{FileStatus, StatusSummary};

pub const INVALID_UTF8: &str = "INVALID UTF-8";
