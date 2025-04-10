use git_core::{
    FileStatus,
    INVALID_UTF8,
    StatusSummary,
    git::{Config, Repo},
};
use itertools::Itertools;

#[derive(Default)]
pub struct Git {
    pub config: Config,
}

impl Git {
    pub fn clone_repo(&self, url: &str) -> String {
        Repo::clone_from(url, &self.config)
            .map_or_else(|e| e.to_string(), |_repo| "Repository cloned".to_string())
    }

    pub fn branches(&self) -> String {
        self.branches_().unwrap_or_else(|e| e.to_string())
    }

    pub fn current_branch(&self) -> String {
        self.current_branch_().unwrap_or_else(|e| e.to_string())
    }

    pub fn status(&self) -> String {
        self.status_().unwrap_or_else(|e| e.to_string())
    }

    pub fn add_all(&self) -> String {
        self.add_all_().unwrap_or_else(|e| e.to_string())
    }

    pub fn commit(&self, message: &str) -> String {
        self.commit_(message).unwrap_or_else(|e| e.to_string())
    }

    pub fn checkout(&self, branch_name: &str) -> String {
        self.checkout_(branch_name)
            .map_or_else(|e| e.to_string(), |()| format!("Switched to branch {branch_name}"))
    }

    pub fn push(&self) -> String {
        self.push_()
            .map_or_else(|e| e.to_string(), |()| "Successfully pushed the branch".to_string())
    }

    pub fn merge(&self) -> String {
        self.merge_()
            .map_or_else(|e| e.to_string(), |()| "Successfully merged the branch".to_string())
    }

    fn branches_(&self) -> Result<String, git2::Error> {
        let repo = self.open_repo()?;
        let branches = repo.branches()?;

        let res = branches
            .into_iter()
            .map(|(branch, branch_type)| {
                let branch_type = match branch_type {
                    git2::BranchType::Local => "Local",
                    git2::BranchType::Remote => "Remote",
                };

                let branch_name = match branch.name() {
                    Ok(Some(name)) => name,
                    Ok(None) => INVALID_UTF8,
                    Err(e) => &e.to_string(),
                };

                format!("{branch_type} {branch_name}")
            })
            .join("\n");
        Ok(res)
    }

    fn current_branch_(&self) -> Result<String, git2::Error> {
        let repo = self.open_repo()?;
        let current_branch = repo.current_branch()?;
        let local = current_branch.local_name();
        let upstream = current_branch
            .upstream_name()
            .unwrap_or_else(|| "[No upstream branch tracked]".to_string());

        Ok(format!("{local}:{upstream}"))
    }

    fn status_(&self) -> Result<String, git2::Error> {
        let StatusSummary {
            branch_name,
            staged,
            not_staged,
            untracked,
        } = self.open_repo().and_then(|repo| repo.status())?;

        let mut res = format!("on branch {branch_name}");
        if staged.is_empty() && not_staged.is_empty() && untracked.is_empty() {
            res.push_str("\nnothing to commit, working tree clean");
            return Ok(res);
        }

        let mut write_section = |header, contents: &[_]| {
            if !contents.is_empty() {
                res.push_str(header);
                res.push_str(&contents.iter().map(FileStatus::to_string).join("\n\t"));
            }
        };

        write_section("\nChanges to be committed:\n\t", &staged);
        write_section("\nChanges not staged for commit:\n\t", &not_staged);
        write_section("\nUntracked files:\n\t", &untracked);

        Ok(res)
    }

    fn add_all_(&self) -> Result<String, git2::Error> {
        let index = self.open_repo()?.add_all()?;
        Ok(format!("{} files to be committed", index.len()))
    }

    fn commit_(&self, message: &str) -> Result<String, git2::Error> {
        self.open_repo()?.commit(message).map(|oid| oid.to_string())
    }

    fn checkout_(&self, branch_name: &str) -> Result<(), git2::Error> {
        self.open_repo()?.checkout(branch_name)
    }

    fn push_(&self) -> Result<(), git2::Error> {
        self.open_repo()?.push()
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn merge_(&self) -> Result<(), git2::Error> {
        Ok(())
    }

    fn open_repo(&self) -> Result<Repo, git2::Error> {
        Repo::open(&self.config)
    }
}
