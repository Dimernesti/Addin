use std::path::PathBuf;

use git2::{
    Branch,
    BranchType,
    Cred,
    FetchOptions,
    IndexAddOption,
    ObjectType,
    Oid,
    PushOptions,
    RemoteCallbacks,
    Repository,
    ResetType,
    Signature,
    StatusOptions,
    build::{CheckoutBuilder, RepoBuilder},
};

use crate::git_status::StatusSummary;

#[derive(Clone, Default)]
pub enum AuthType {
    Password(String),
    #[default]
    None,
}

#[derive(Clone, Default)]
pub struct Config {
    pub username: String,
    pub auth: AuthType,
    pub email: String,
    pub path: PathBuf,
}

pub struct Repo<'a> {
    repo: Repository,
    config: &'a Config,
}

impl<'a> Repo<'a> {
    pub fn open(config: &'a Config) -> Result<Self, git2::Error> {
        Ok(Self {
            repo: Repository::open(&config.path)?,
            config,
        })
    }

    pub fn clone_from(url: &str, config: &'a Config) -> Result<Self, git2::Error> {
        let repo = RepoBuilder::new()
            .fetch_options(Self::fetch_options(config))
            .clone(url, &config.path)?;

        Ok(Self { repo, config })
    }

    pub fn branches(
        &self,
    ) -> Result<impl Iterator<Item = (git2::Branch, git2::BranchType)>, git2::Error> {
        self.fetch_all()?;
        Ok(self.repo.branches(None)?.flatten())
    }

    pub fn current_branch(&self) -> Result<(Branch, Branch), git2::Error> {
        let head = self.repo.head()?;
        let head_shorthand = head.shorthand().unwrap_or("HEAD");

        let local_branch = self.repo.find_branch(head_shorthand, BranchType::Local)?;
        let upstream_branch = local_branch.upstream()?;

        Ok((local_branch, upstream_branch))
    }

    pub fn status(&self) -> Result<StatusSummary, git2::Error> {
        let branch_name = self
            .repo
            .head()?
            .shorthand()
            .ok_or_else(|| git2::Error::from_str("no branch name"))?
            .to_string();

        let mut options = StatusOptions::new();
        options
            .renames_from_rewrites(true) // not sure if this line is needed
            .include_untracked(true)
            .renames_head_to_index(true);

        let summary = self.repo.statuses(Some(&mut options))?.iter().fold(
            StatusSummary::new(branch_name),
            |mut summary, entry| {
                summary.add_entry(&entry);
                summary
            },
        );

        Ok(summary)
    }

    pub fn add_all(&self) -> Result<git2::Index, git2::Error> {
        let mut index = self.repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(index)
    }

    pub fn commit(&self, message: &str) -> Result<Oid, git2::Error> {
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        let parent_commit = self.find_last_commit()?;

        let author = Signature::now(&self.config.username, &self.config.email)?;
        self.repo.commit(Some("HEAD"), &author, &author, message, &tree, &[&parent_commit])
    }

    pub fn checkout(&self, branch_name: &str) -> Result<(), git2::Error> {
        self.fetch_all()?;

        let remote_branch_name = format!("origin/{branch_name}");

        let (branch, _brach_type) = self
            .repo
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
        let refname =
            reference.shorthand().ok_or(git2::Error::from_str("cannot obtain branch refname"))?;

        self.repo
            .reset(&commit, ResetType::Hard, Some(checkout.force()))
            .and_then(|()| self.repo.set_head(&format!("refs/heads/{refname}")))
    }

    pub fn push(&self) -> Result<(), git2::Error> {
        let mut origin = self.repo.find_remote("origin")?;
        let repo_head = self.repo.head()?;
        let branch_name =
            repo_head.name().ok_or_else(|| git2::Error::from_str("no branch name"))?;
        let mut options = Self::push_options(self.config);
        origin.push(&[branch_name], Some(&mut options))?;

        Ok(())
    }

    fn fetch_all(&self) -> Result<(), git2::Error> {
        for remote_name in self.repo.remotes()?.iter().flatten() {
            let mut remote = self.repo.find_remote(remote_name)?;
            let mut opts = Self::fetch_options(self.config);
            remote.fetch(&[] as &[&str], Some(&mut opts), None)?;
        }
        Ok(())
    }

    fn push_options<'b>(config: &'a Config) -> PushOptions<'b>
    where
        'a: 'b,
    {
        let callbacks = Self::register_credentials(config, RemoteCallbacks::new());
        let mut options = PushOptions::new();
        options.remote_callbacks(callbacks);
        options
    }

    fn fetch_options<'b>(config: &'a Config) -> FetchOptions<'b>
    where
        'a: 'b,
    {
        let callbacks = Self::register_credentials(config, RemoteCallbacks::new());
        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks);
        options
    }

    fn register_credentials<'b>(
        config: &'a Config,
        mut callbacks: RemoteCallbacks<'b>,
    ) -> RemoteCallbacks<'b>
    where
        'a: 'b,
    {
        match &config.auth {
            AuthType::Password(password) => {
                callbacks.credentials(|_url, _username_from_url, _allowed_types| {
                    Cred::userpass_plaintext(&config.username, password)
                });
            },
            AuthType::None => {},
        }
        callbacks
    }

    fn find_last_commit(&self) -> Result<git2::Commit, git2::Error> {
        self.repo
            .head()?
            .resolve()?
            .peel(ObjectType::Commit)?
            .into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find last commit"))
    }
}
