use std::path::PathBuf;

use git2::{
    Branch,
    BranchType,
    Cred,
    FetchOptions,
    FetchPrune,
    IndexAddOption,
    IntoCString,
    ObjectType,
    Oid,
    PushOptions,
    RemoteCallbacks,
    Repository,
    Signature,
    StatusOptions,
    build::{CheckoutBuilder, RepoBuilder},
};

use crate::{INVALID_UTF8, git_status::StatusSummary};

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

    pub fn current_branch(&self) -> Result<TrackedBranch, git2::Error> {
        let head = self.repo.head()?;
        let head_shorthand = head.shorthand().unwrap_or("HEAD");

        let local = self.repo.find_branch(head_shorthand, BranchType::Local)?;
        let upstream = local.upstream().ok();

        Ok(TrackedBranch { local, upstream })
    }

    pub fn status(&self) -> Result<StatusSummary, git2::Error> {
        let branch_name = self
            .repo
            .head()?
            .shorthand()
            .ok_or_else(|| {
                git2::Error::from_str(&format!("Current branch name is {INVALID_UTF8}"))
            })?
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

    pub fn add<T, I>(&self, pathspecks: I) -> Result<git2::Index, git2::Error>
    where
        T: IntoCString,
        I: IntoIterator<Item = T>,
    {
        let mut index = self.repo.index()?;
        index.add_all(pathspecks, IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(index)
    }

    pub fn add_all(&self) -> Result<git2::Index, git2::Error> {
        self.add(["."])
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

        let (branch, brach_type) = self
            .repo
            .branches(None)?
            .flatten()
            .find(|(branch, branch_type)| match branch_type {
                BranchType::Local => Ok(Some(branch_name)) == branch.name(),
                BranchType::Remote => Ok(Some(remote_branch_name.as_str())) == branch.name(),
            })
            .ok_or(git2::Error::from_str("no branch with this name"))?;

        let commit = branch
            .get()
            .resolve()?
            .peel(ObjectType::Commit)?
            .into_commit()
            .map_err(|_e| git2::Error::from_str("Failed to obtain commit"))?;

        if let BranchType::Remote = brach_type {
            self.repo
                .branch(branch_name, &commit, false)?
                .set_upstream(Some(&remote_branch_name))?;
        }

        self.repo.set_head(&format!("refs/heads/{branch_name}"))?;
        self.repo.checkout_head(Some(CheckoutBuilder::default().allow_conflicts(true).force()))?;

        Ok(())
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

    pub fn pull(&self, branch_name: &str) -> Result<PullResult, git2::Error> {
        let mut local_branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let remote_branch = local_branch.upstream()?;
        let old_id = local_branch.get().peel_to_commit()?.id();

        let remote_commit = remote_branch.get().peel_to_commit()?;
        let annotated_commit = self.repo.find_annotated_commit(remote_commit.id())?;
        let (analisis, _preference) =
            self.repo.merge_analysis_for_ref(local_branch.get(), &[&annotated_commit])?;

        if analisis.is_none() {
            Ok(PullResult::None)
        } else if analisis.is_normal() {
            Ok(PullResult::Normal)
        } else if analisis.is_up_to_date() {
            Ok(PullResult::UpToDate)
        } else if analisis.is_fast_forward() {
            let referense = local_branch.get_mut().set_target(
                remote_commit.id(),
                &format!("fast forward branch '{branch_name}' tip"),
            )?;
            let new_id = referense.peel_to_commit()?.id();
            Ok(PullResult::FastForwarded { old_id, new_id })
        } else if analisis.is_unborn() {
            Ok(PullResult::Unborn)
        } else {
            unreachable!("Invalid pull analisis value {:b}", analisis.bits())
        }
    }

    pub fn merge(&self, _branch_from: &str, _branch_to: Option<&str>) -> Result<(), git2::Error> {
        // self.repo.
        // self.repo.merge(annotated_commits, merge_opts, checkout_opts)

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
        options.prune(FetchPrune::On);
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

pub fn branch_name(branch: &git2::Branch) -> String {
    match branch.name() {
        Ok(Some(name)) => name.to_string(),
        Ok(None) => INVALID_UTF8.to_string(),
        Err(e) => e.to_string(),
    }
}

pub struct TrackedBranch<'repo> {
    pub local: Branch<'repo>,
    pub upstream: Option<Branch<'repo>>,
}

impl TrackedBranch<'_> {
    pub fn local_name(&self) -> String {
        branch_name(&self.local)
    }

    pub fn upstream_name(&self) -> Option<String> {
        self.upstream.as_ref().map(branch_name)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PullResult {
    /// No merge is possible.
    None,
    /// A "normal" merge; both HEAD and the given merge input have diverged
    /// from their common ancestor. The divergent commits must be merged.
    Normal,
    /// All given merge inputs are reachable from HEAD, meaning the
    /// repository is up-to-date and no merge needs to be performed.
    UpToDate,
    /// The given merge input is a fast-forward from HEAD and no merge
    /// needs to be performed. Check out the given merge input.
    FastForwarded { old_id: Oid, new_id: Oid },
    /// The HEAD of the current repository is "unborn" and does not point to
    /// a valid commit. No merge can be performed, but the caller may wish
    /// to simply set HEAD to the target commit(s).
    Unborn,
}
