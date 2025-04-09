use std::path::{Path, PathBuf};

use git2::{
    BranchType,
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
    pub fn clone_repo_str(&mut self, url: &str) -> String {
        match self.clone_repo(url) {
            Ok(()) => "Repository cloned".to_string(),
            Err(e) => e.to_string(),
        }
    }

    fn clone_repo(&mut self, url: &str) -> Result<(), git2::Error> {
        RepoBuilder::new().fetch_options(self.fetch_options()).clone(url, &self.catalog).map(|_repo| ())
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
                    Ok(None) => INVALID_UTF8,
                    Err(e) => &e.to_string(),
                };

                format!("{branch_type} {branch_name}")
            })
            .join("\n")
    }

    fn get_branches<'a>(
        &self,
        repo: &'a git2::Repository,
    ) -> Result<Vec<(git2::Branch<'a>, git2::BranchType)>, git2::Error> {
        self.fetch_all(repo)?;
        let branches = repo.branches(None)?.flatten().collect();
        Ok(branches)
    }

    pub fn checkout_str(&self, branch_name: &str) -> String {
        match self.checkout(branch_name) {
            Ok(()) => format!("switched to branch {branch_name}"),
            Err(error) => error.to_string(),
        }
    }

    fn checkout(&self, branch_name: &str) -> Result<(), git2::Error> {
        let repo = self.open_repo()?;
        self.fetch_all(&repo)?;

        let remote_branch_name = format!("origin/{branch_name}");

        let (branch, _brach_type) = repo
            .branches(None)?
            .flatten()
            .find(|(branch, branch_type)| match branch_type {
                BranchType::Local => Ok(Some(branch_name)) == branch.name(),
                BranchType::Remote => Ok(Some(remote_branch_name.as_str())) == branch.name(),
            })
            .ok_or(git2::Error::from_str("no branch with this name"))?;

        let commit = branch.get().resolve()?.peel(ObjectType::Commit)?;
        let mut checkout = CheckoutBuilder::new();

        let reference = branch.into_reference();
        let refname = reference.shorthand().ok_or(git2::Error::from_str("cannot obtain branch refname"))?;

        repo.reset(&commit, ResetType::Hard, Some(checkout.force()))
            .and_then(|()| repo.set_head(&format!("refs/heads/{refname}")))
    }

    pub fn add_all_str(&self) -> String {
        match self.add_all() {
            Ok(index) => format!("{} files to be committed", index.len()),
            Err(e) => e.to_string(),
        }
    }

    fn add_all(&self) -> Result<git2::Index, git2::Error> {
        let repo = self.open_repo()?;
        let mut index = repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(index)
    }

    pub fn commit_str(&self, message: &str) -> String {
        match self.commit(message) {
            Ok(oid) => format!("commit {oid}"),
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

    pub fn get_current_branch_str(&self) -> String {
        match self.get_current_branch() {
            Ok((local_branch_name, upstream_branch_name)) => format!("{local_branch_name}:{upstream_branch_name}"),
            Err(error) => error.to_string(),
        }
    }

    fn get_current_branch(&self) -> Result<(String, String), git2::Error> {
        let repo = self.open_repo()?;

        let head = repo.head()?;
        let head_shorthand = head.shorthand().unwrap_or("HEAD");

        let local_branch = repo.find_branch(head_shorthand, BranchType::Local)?;
        let local_branch_name = local_branch.name()?.ok_or(git2::Error::from_str(INVALID_UTF8))?;

        let upstream_branch = local_branch.upstream()?;
        let upstream_branch_name = upstream_branch.name()?.ok_or(git2::Error::from_str(INVALID_UTF8))?;

        Ok((local_branch_name.to_string(), upstream_branch_name.to_string()))
    }

    pub fn merge_str(&self) -> String {
        match self.merge() {
            Ok(current_branch) => current_branch,
            Err(error) => error.to_string(),
        }
    }

    fn merge(&self) -> Result<String, git2::Error> {
        let repo = self.open_repo()?;
        self.fetch_all(&repo)?;
        // self.checkout(branch_name)?;

        let branch_name =
            repo.head()?.name().ok_or_else(|| git2::Error::from_str("no branch name in HEAD"))?.to_string();

        Ok(branch_name)
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

const INVALID_UTF8: &str = "INVALID UTF-8";
