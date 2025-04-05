use std::path::{Path, PathBuf};

use git2::{
    Cred,
    FetchOptions,
    IndexAddOption,
    ObjectType,
    Oid,
    PushOptions,
    RemoteCallbacks,
    ResetType,
    Signature,
    StatusOptions,
    build::{CheckoutBuilder, RepoBuilder},
};
use itertools::Itertools;

use crate::git_status::{FileStatus, StatusSummary};

#[derive(Default)]
pub struct Git {
    pub login: String,
    pub password: String,
    pub email: String,
    pub catalog: PathBuf,
}

impl Git {
    fn clone_repo(&mut self, url: &str) -> Result<(), git2::Error> {
        RepoBuilder::new().fetch_options(self.fetch_options()).clone(url, &self.catalog).map(|_repo| ())
    }

    pub fn clone_repo_str(&mut self, url: &str) -> String {
        match self.clone_repo(url) {
            Ok(()) => "Ok".to_string(),
            Err(e) => e.to_string(),
        }
    }

    fn get_branches<'a>(
        &self,
        repo: &'a git2::Repository,
    ) -> Result<Vec<(git2::Branch<'a>, git2::BranchType)>, git2::Error> {
        self.fetch_all(repo)?;
        let branches = repo.branches(None)?.flatten().collect();
        Ok(branches)
    }

    pub fn get_branches_str(&self) -> String {
        let repo = match self.open_repo() {
            Ok(repo) => repo,
            Err(e) => return e.to_string(),
        };

        let branches = match self.get_branches(&repo) {
            Ok(branches) => branches,
            Err(e) => return e.to_string(),
        };

        branches
            .into_iter()
            .map(|(branch, branch_type)| {
                let branch_type = match branch_type {
                    git2::BranchType::Local => "Local",
                    git2::BranchType::Remote => "Remote",
                };

                let branch_name = match branch.name() {
                    Ok(Some(name)) => name,
                    Ok(None) => "INVALID UTF-8",
                    Err(e) => &e.to_string(),
                };

                format!("{branch_type} {branch_name}")
            })
            .join("\n")
    }

    pub fn checkout_str(&self, branch_name: &str) -> String {
        match self.checkout(branch_name) {
            Ok(()) => format!("switched to branch {branch_name}"),
            Err(error) => error.to_string(),
        }
    }

    fn checkout(&self, branch_name: &str) -> Result<(), git2::Error> {
        let repo = self.open_repo()?;

        let (branch, _) = repo
            .branches(None)?
            .flatten()
            .find(|(branch, _)| branch.name() == Ok(Some(branch_name)))
            .ok_or(git2::Error::from_str("no branch with this name"))?;

        let commit = branch.get().resolve()?.peel(ObjectType::Commit)?;
        let mut checkout = CheckoutBuilder::new();

        repo.reset(&commit, ResetType::Hard, Some(checkout.force())).and_then(|()| repo.set_head(branch_name))
    }

    fn add_all(&self) -> Result<git2::Index, git2::Error> {
        let repo = self.open_repo()?;
        let mut index = repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(index)
    }

    pub fn add_all_str(&self) -> String {
        match self.add_all() {
            Ok(index) => format!("{} files to be committed", index.len()),
            Err(e) => e.to_string(),
        }
    }

    fn commit(&self, message: &str) -> Result<Oid, git2::Error> {
        let repo = self.open_repo()?;
        let mut index = repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;
        let parent_commit = Self::find_last_commit(&repo)?;

        let author = Signature::now(&self.login, &self.email)?;
        repo.commit(Some("HEAD"), &author, &author, message, &tree, &[&parent_commit])
    }

    pub fn commit_str(&self, message: &str) -> String {
        match self.commit(message) {
            Ok(_oid) => "Ok".to_string(),
            Err(e) => e.to_string(),
        }
    }

    fn fetch_all(&self, repo: &git2::Repository) -> Result<(), git2::Error> {
        for remote_name in repo.remotes()?.iter().flatten() {
            let mut remote = repo.find_remote(remote_name)?;
            let refspecs = remote.fetch_refspecs()?;
            let refspecs = refspecs.iter().flatten().collect_vec();
            let mut opts = self.fetch_options();
            remote.fetch(&refspecs, Some(&mut opts), None)?;
        }
        Ok(())
    }

    pub fn push_str(&self) -> String {
        match self.push() {
            Ok(()) => "Ok".to_string(),
            Err(e) => e.to_string(),
        }
    }

    fn push(&self) -> Result<(), git2::Error> {
        let repo = self.open_repo()?;
        let mut origin = repo.find_remote("origin")?;
        let repo_head = repo.head()?;
        let branch_name = repo_head.name().ok_or_else(|| git2::Error::from_str("no branch name"))?;
        let callbacks = self.register_credentials(RemoteCallbacks::new());
        let mut options = PushOptions::new();
        options.remote_callbacks(callbacks);
        origin.push(&[branch_name], Some(&mut options))?;

        Ok(())
    }

    pub fn status_str(&self) -> String {
        let StatusSummary {
            branch_name,
            staged,
            not_staged,
            untracked,
        } = match self.status() {
            Ok(status) => status,
            Err(e) => return e.to_string(),
        };

        let mut res = format!("on branch {branch_name}");
        if staged.is_empty() && not_staged.is_empty() && untracked.is_empty() {
            res.push_str("\nnothing to commit, working tree clean");
            return res;
        }

        if !staged.is_empty() {
            res.push_str("\nChanges to be committed:\n\t");
            res.push_str(&staged.iter().map(FileStatus::to_string).join("\n\t"));
        }

        if !not_staged.is_empty() {
            res.push_str("\nChanges not staged for commit:\n\t");
            res.push_str(&not_staged.iter().map(FileStatus::to_string).join("\n\t"));
        }

        if !untracked.is_empty() {
            res.push_str("\nUntracked files:\n\t");
            res.push_str(&untracked.iter().map(FileStatus::to_string).join("\n\t"));
        }

        res
    }

    fn status(&self) -> Result<StatusSummary, git2::Error> {
        let repo = self.open_repo()?;
        let branch_name = repo.head()?.name().ok_or_else(|| git2::Error::from_str("no branch name"))?.to_string();

        let mut options = StatusOptions::new();
        options.include_untracked(true).renames_from_rewrites(true).renames_head_to_index(true);

        let summary =
            repo.statuses(Some(&mut options))?.iter().fold(StatusSummary::new(branch_name), |mut summary, entry| {
                summary.add_entry(&entry);
                summary
            });

        Ok(summary)
    }

    pub fn get_catalog(&self) -> &str {
        self.catalog.to_str().unwrap_or("")
    }

    pub fn set_catalog(&mut self, catalog: &str) {
        self.catalog = Path::new(catalog).to_path_buf();
    }

    fn open_repo(&self) -> Result<git2::Repository, git2::Error> {
        git2::Repository::open(&self.catalog)
    }

    fn fetch_options(&self) -> FetchOptions {
        let callbacks = self.register_credentials(RemoteCallbacks::new());
        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks);
        options
    }

    fn register_credentials<'a, 'b>(&'a self, mut callbacks: RemoteCallbacks<'b>) -> RemoteCallbacks<'b>
    where
        'a: 'b,
    {
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&self.login, &self.password)
        });
        callbacks
    }

    fn find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
        repo.head()?
            .resolve()?
            .peel(ObjectType::Commit)?
            .into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
}
