use std::path::{Path, PathBuf};

use git2::{
    Branch,
    BranchType,
    Commit,
    Cred,
    FetchOptions,
    IndexAddOption,
    ObjectType,
    Oid,
    RemoteCallbacks,
    Repository,
    ResetType,
    Signature,
    build::{CheckoutBuilder, RepoBuilder},
};
use itertools::Itertools;

#[derive(Default)]
pub struct GitLib {
    pub login: String,
    pub password: String,
    pub email: String,
    pub catalog: PathBuf,
}

impl GitLib {
    fn clone_repo(&mut self, url: &str, catalog: &str) -> Result<(), git2::Error> {
        self.set_catalog(catalog);

        RepoBuilder::new().fetch_options(self.fetch_options()).clone(url, &self.catalog).map(|_repo| ())
    }

    pub fn clone_repo_str(&mut self, url: &str, catalog: &str) -> String {
        match self.clone_repo(url, catalog) {
            Ok(()) => "Ok".to_string(),
            Err(e) => e.to_string(),
        }
    }

    fn get_branches<'a>(&self, repo: &'a Repository) -> Result<Vec<(Branch<'a>, BranchType)>, git2::Error> {
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
                    BranchType::Local => "Local",
                    BranchType::Remote => "Remote",
                };

                let branch_name = match branch.name() {
                    Ok(Some(name)) => name,
                    Ok(None) => "INVALID UTF-8",
                    Err(e) => &e.to_string(),
                };

                format!("{branch_type} {branch_name}",)
            })
            .join("\n")
    }

    pub fn checkout_str(&self, branch: &str) -> String {
        match self.checkout(branch) {
            Ok(_) => "ok".to_string(),
            Err(error) => error.to_string(),
        }
    }

    fn checkout(&self, branch_name: &str) -> Result<(), git2::Error> {
        let repo = self.open_repo()?;
        let mut branches = repo.branches(None)?.flatten();
        let my_branch = branches.find(|(branch, _)| match branch.name() {
            Ok(Some(name)) => name == branch_name,
            _ => false,
        });

        let (my_branch, _) = match my_branch {
            Some(branch) => branch,
            None => return Err(git2::Error::from_str("no branch with this name")),
        };

        let my_commit = my_branch.get().resolve()?.peel(ObjectType::Commit)?;

        let mut checkout = CheckoutBuilder::new();

        repo.reset(&my_commit, ResetType::Hard, Some(checkout.force()))
    }

    fn add_all(&self) -> Result<(), git2::Error> {
        let repo = self.open_repo()?;
        let mut index = repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()
    }

    pub fn add_all_str(&self) -> String {
        match self.add_all() {
            Ok(()) => "Ok".to_string(),
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

    fn fetch_all(&self, repo: &Repository) -> Result<(), git2::Error> {
        for remote_name in repo.remotes()?.iter().flatten() {
            let mut remote = repo.find_remote(remote_name)?;
            let refspecs = remote.fetch_refspecs()?;
            let refspecs = refspecs.iter().flatten().collect_vec();
            let mut opts = self.fetch_options();
            remote.fetch(&refspecs, Some(&mut opts), None)?;
        }
        Ok(())
    }

    pub fn get_catalog(&self) -> &str {
        self.catalog.to_str().unwrap_or("")
    }

    pub fn set_catalog(&mut self, catalog: &str) {
        self.catalog = Path::new(catalog).to_path_buf();
    }

    fn open_repo(&self) -> Result<Repository, git2::Error> {
        Repository::open(&self.catalog)
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

    fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
        repo.head()?
            .resolve()?
            .peel(ObjectType::Commit)?
            .into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
}
