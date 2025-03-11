use std::path::Path;

use git2::{Cred, Direction, FetchOptions, RemoteCallbacks, RemoteHead, Repository, build::RepoBuilder};
use itertools::Itertools;

#[derive(Default)]
pub struct GitLib {
    pub login: String,
    pub password: String,
}

impl GitLib {
    pub fn clone_repo(&self, url: &str, catalog: &str) -> Result<(), git2::Error> {
        let callbacks = self.register_credentials(RemoteCallbacks::new());
        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks);

        RepoBuilder::new().fetch_options(options).clone(url, Path::new(catalog)).map(|_repo| ())
    }

    pub fn clone_repo_str(&self, url: &str, catalog: &str) -> String {
        self.clone_repo(url, catalog).map_or_else(|e| e.to_string(), |()| "ok".to_string())
    }

    pub fn get_branches(&self, catalog: &str) -> Result<String, git2::Error> {
        let repo = Repository::open(catalog)?;
        let remotes = repo.remotes()?;
        let remote = remotes.get(0).unwrap();
        let mut remote = repo.find_remote(remote)?;

        let callbacks = self.register_credentials(RemoteCallbacks::new());
        let connection = remote.connect_auth(Direction::Fetch, Some(callbacks), None)?;

        let branches = connection.list()?.iter().map(RemoteHead::name).join("\n");
        Ok(branches)
    }

    pub fn get_branches_str(&self, catalog: &str) -> String {
        self.get_branches(catalog).unwrap_or_else(|e| e.to_string())
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
}
