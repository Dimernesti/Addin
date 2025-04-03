use addin1c::{AddinResult, MethodInfo, Methods, PropInfo, SimpleAddin, Variant, name};

use crate::gitlib::GitLib;

pub struct GitAddin {
    gitlib: GitLib,
}

impl GitAddin {
    pub fn new() -> Self {
        Self { gitlib: GitLib::default() }
    }

    fn clone_repo(&mut self, url: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let message = self.gitlib.clone_repo_str(&url.get_string()?);
        ret_value.set_str1c(message)?;
        Ok(())
    }

    fn get_branches(&mut self, ret_value: &mut Variant) -> AddinResult {
        let branches = self.gitlib.get_branches_str();
        ret_value.set_str1c(branches)?;
        Ok(())
    }

    fn add_all(&mut self, ret_value: &mut Variant) -> AddinResult {
        let message = self.gitlib.add_all_str();
        ret_value.set_str1c(message)?;
        Ok(())
    }

    fn commit(&mut self, message: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let result = self.gitlib.commit_str(&message.get_string()?);
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn checkout(&mut self, branch: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let result = self.gitlib.checkout_str(&branch.get_string()?);
        ret_value.set_str1c(result)?;
        Ok(())
    }

    fn push(&mut self, ret_value: &mut Variant) -> AddinResult {
        let result = self.gitlib.push_str();
        ret_value.set_str1c(result)?;
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

    fn get_email(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.gitlib.email.clone())?;
        Ok(())
    }

    fn set_email(&mut self, email: &Variant) -> AddinResult {
        self.gitlib.email = email.get_string()?;
        Ok(())
    }

    fn get_catalog(&mut self, ret_value: &mut Variant) -> AddinResult {
        ret_value.set_str1c(self.gitlib.get_catalog())?;
        Ok(())
    }

    fn set_catalog(&mut self, catalog: &Variant) -> AddinResult {
        self.gitlib.set_catalog(&catalog.get_string()?);
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
