use clap::{Args, Parser, Subcommand};
use git_core::{AuthType, Config, Repo};
use git2::BranchType;
use itertools::Itertools;


fn main() -> Result<(), git2::Error> {
    let config = Config {
        username: "RUST".to_string(),
        auth: AuthType::None,
        email: "rust@rust.rs".to_string(),
        path: "/home/vasich/projects/smolin/test_repo".into(),
    };

    let repo = Repo::open(&config).expect("failed to open repository");

    match Cli::parse().command {
        Commands::Add(AddArgs { files: _files }) => {
            let index = repo.add_all()?;
            println!("{} files added", index.len());
        },
        Commands::Commit(CommitArgs { message }) => {
            let oid = repo.commit(&message)?;
            println!("made commit {oid}");
        },
        Commands::Status => {
            let summary = repo.status()?;
            println!("{summary:?}");
        },
        Commands::Branches => {
            let branches = repo
                .branches()?
                .map(|(branch, branch_type)| format!("{branch_type:?} -- {}", branch_name(&branch)))
                .join("\n");

            println!("{branches:?}");
        },
        Commands::CurrentBranch => {
            let (local, upstream) = repo.current_branch()?;
            let local = branch_name(&local);
            let upstream = branch_name(&upstream);
            println!("{local} -- {upstream}");
        },
        Commands::Checkout(checkout_args) => todo!(),
    }

    Ok(())
}


#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    Commit(CommitArgs),
    Status,
    Branches,
    #[command(name = "current-branch")]
    CurrentBranch,
    Checkout(CheckoutArgs),
}

#[derive(Args)]
struct AddArgs {
    files: Vec<String>,
}

#[derive(Args)]
struct CommitArgs {
    message: String,
}

#[derive(Args)]
struct CheckoutArgs {
    branch_name: String,
}

const INVALID_UTF8: &str = "INVALID UTF-8";

fn branch_name(branch: &git2::Branch) -> String {
    match branch.name() {
        Ok(Some(name)) => name.to_string(),
        Ok(None) => INVALID_UTF8.to_string(),
        Err(e) => e.to_string(),
    }
}
