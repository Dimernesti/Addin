use clap::{Args, Parser, Subcommand};
use git_core::{AuthType, Config, Repo, git::branch_name};
use itertools::Itertools;


fn main() -> Result<(), git2::Error> {
    let repo_name = std::env::var("REPO_NAME").expect("repo name is not set");
    let root_path = std::env::var("REPO_ROOT_PATH").expect("repo root path is not set");

    let config = Config {
        username: "RUST".to_string(),
        auth: AuthType::None,
        email: "rust@rust.rs".to_string(),
        path: format!("{root_path}/test_repo/{repo_name}").into(),
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
                .map(|(branch, branch_type)| {
                    let branch_name = branch_name(&branch);
                    format!("{branch_type:?} -- {branch_name}")
                })
                .join("\n");

            println!("{branches}");
        },
        Commands::CurrentBranch => {
            let current_branch = repo.current_branch()?;

            let local = current_branch.local_name();
            let upstream = current_branch
                .upstream_name()
                .unwrap_or_else(|| "[No upstream branch tracked]".to_string());
            println!("{local}:{upstream}");
        },
        Commands::Checkout(CheckoutArgs { branch_name }) => {
            let res = repo.checkout(&branch_name);
            println!("{res:?}");
        },
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
