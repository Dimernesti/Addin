use addin1c::{AddinResult, MethodInfo, Methods, PropInfo, SimpleAddin, Variant, name};
use git_core::AuthType;
use log::debug;

use crate::git::Git;

pub struct GitAddin {
    git: Git,
}

impl GitAddin {
    pub fn new() -> Self {
        debug!("GitAdding::new()");
        Self { git: Git::default() }
    }

    fn clone_repo(&mut self, url: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        debug!("clone_repo()");
        let message = self.git.clone_repo(&url.get_string()?);
        ret_value.set_str1c(message)?;
        Ok(())
    }

    fn get_branches(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("get_branches()");
        let branches = self.git.branches();
        ret_value.set_str1c(branches)?;
        Ok(())
    }

    fn get_current_branch(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("get_current_branch()");
        let result = self.git.current_branch();
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn status(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("status()");
        let status = self.git.status();
        ret_value.set_str1c(status)?;
        Ok(())
    }

    fn add_all(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("add_all()");
        let message = self.git.add_all();
        ret_value.set_str1c(message)?;
        Ok(())
    }

    fn commit(&mut self, message: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        debug!("commit()");
        let result = self.git.commit(&message.get_string()?);
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn checkout(&mut self, branch_name: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        debug!("checkout()");
        let result = self.git.checkout(&branch_name.get_string()?);
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn push(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("push()");
        let result = self.git.push();
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn pull(&mut self, branch_name: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        debug!("pull()");
        let result = self.git.pull(&branch_name.get_string()?);
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn merge(&mut self, ret_value: &mut Variant) -> AddinResult {
        debug!("merge()");
        let result = self.git.merge();
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn get_login(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.git.config.username.clone())?;
        Ok(())
    }

    fn set_login(&mut self, login: &Variant) -> AddinResult {
        self.git.config.username = login.get_string()?;
        Ok(())
    }

    fn get_password(&mut self, ret_value: &mut Variant) -> AddinResult {
        let password = match &self.git.config.auth {
            AuthType::Password(password) => password,
            AuthType::None => "",
        };

        ret_value.set_str1c(password)?;
        Ok(())
    }

    fn set_password(&mut self, password: &Variant) -> AddinResult {
        self.git.config.auth = AuthType::Password(password.get_string()?);
        Ok(())
    }

    fn get_email(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.git.config.email.as_str())?;
        Ok(())
    }

    fn set_email(&mut self, email: &Variant) -> AddinResult {
        self.git.config.email = email.get_string()?;
        Ok(())
    }

    fn get_catalog(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.git.config.path.to_str().unwrap_or(""))?;
        Ok(())
    }

    fn set_catalog(&mut self, catalog: &Variant) -> AddinResult {
        self.git.config.path = catalog.get_string()?.into();
        Ok(())
    }
}

impl SimpleAddin for GitAddin {
    fn name() -> &'static [u16] {
        name!("GitAddin")
    }

    fn methods() -> &'static [MethodInfo<Self>] {
        &[
            MethodInfo {
                name: name!("CloneRepo"),
                method: Methods::Method1(Self::clone_repo),
            },
            MethodInfo {
                name: name!("GetBranches"),
                method: Methods::Method0(Self::get_branches),
            },
            MethodInfo {
                name: name!("Status"),
                method: Methods::Method0(Self::status),
            },
            MethodInfo {
                name: name!("AddAll"),
                method: Methods::Method0(Self::add_all),
            },
            MethodInfo {
                name: name!("Commit"),
                method: Methods::Method1(Self::commit),
            },
            MethodInfo {
                name: name!("Checkout"),
                method: Methods::Method1(Self::checkout),
            },
            MethodInfo {
                name: name!("Push"),
                method: Methods::Method0(Self::push),
            },
            MethodInfo {
                name: name!("GetCurrentBranch"),
                method: Methods::Method0(Self::get_current_branch),
            },
            MethodInfo {
                name: name!("Pull"),
                method: Methods::Method1(Self::pull),
            },
            MethodInfo {
                name: name!("Merge"),
                method: Methods::Method0(Self::merge),
            },
        ]
    }

    fn properties() -> &'static [PropInfo<Self>] {
        &[
            PropInfo {
                name: name!("Login"),
                getter: Some(Self::get_login),
                setter: Some(Self::set_login),
            },
            PropInfo {
                name: name!("Password"),
                getter: Some(Self::get_password),
                setter: Some(Self::set_password),
            },
            PropInfo {
                name: name!("Email"),
                getter: Some(Self::get_email),
                setter: Some(Self::set_email),
            },
            PropInfo {
                name: name!("Catalog"),
                getter: Some(Self::get_catalog),
                setter: Some(Self::set_catalog),
            },
        ]
    }
}
