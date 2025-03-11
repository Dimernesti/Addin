use addin1c::{AddinResult, MethodInfo, Methods, PropInfo, SimpleAddin, Variant, name};

use crate::gitlib::GitLib;

pub struct GitAddin {
    gitlib: GitLib,
}

impl GitAddin {
    pub fn new() -> Self {
        Self { gitlib: GitLib::default() }
    }

    fn clone_repo(&mut self, url: &mut Variant, catalog: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let message = self.gitlib.clone_repo_str(&url.get_string()?, &catalog.get_string()?);
        ret_value.set_str1c(message)?;
        Ok(())
    }

    fn get_branches(&mut self, catalog: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let branches = self.gitlib.get_branches_str(&catalog.get_string()?);
        ret_value.set_str1c(branches)?;
        Ok(())
    }

    fn get_login(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.gitlib.login.clone())?;
        Ok(())
    }

    fn set_login(&mut self, login: &Variant) -> AddinResult {
        self.gitlib.login = login.get_string()?;
        Ok(())
    }

    fn get_password(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.gitlib.password.clone())?;
        Ok(())
    }

    fn set_password(&mut self, password: &Variant) -> AddinResult {
        self.gitlib.password = password.get_string()?;
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
                method: Methods::Method2(Self::clone_repo),
            },
            MethodInfo {
                name: name!("GetBranches"),
                method: Methods::Method1(Self::get_branches),
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
        ]
    }
}
