use std::error::Error;

use clap::{Args, Parser, Subcommand};
use git_core::{AuthType, Config, Repo, StatusSummary, git::branch_name};


fn main() -> Result<(), Box<dyn Error>> {
    let repo_name = std::env::var("REPO_NAME").expect("repo name is not set");
    let repos_dir = std::env::var("REPOS_DIR").expect("repo root path is not set");

    let config = Config {
        username: "RUST".to_string(),
        auth: AuthType::None,
        email: "rust@rust.rs".to_string(),
        path: format!("{repos_dir}/{repo_name}").into(),
    };

    match Cli::parse().command {
        Commands::Clone(CloneArgs { url }) => {
            let _repo = Repo::clone_from(&url, &config)?;
            config
                .path
                .read_dir()?
                .flatten()
                .for_each(|file| println!("{}", file.file_name().to_string_lossy()));
        },
        Commands::Add(AddArgs { files }) => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let _index = repo.add(files)?;
            println!("files added");
        },
        Commands::Commit(CommitArgs { message }) => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let oid = repo.commit(&message)?;
            println!("made commit {oid}");
        },
        Commands::Status => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let summary = repo.status()?;
            print_status_summary(&summary);
            // println!("{summary:?}");
        },
        Commands::Branches => {
            let repo = Repo::open(&config).expect("failed to open repository");
            repo.branches()?.for_each(|(branch, branch_type)| {
                let branch_name = branch_name(&branch);
                println!("{:6} -- {branch_name}", format!("{branch_type:?}"))
            });
        },
        Commands::CurrentBranch => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let current_branch = repo.current_branch()?;

            let local = current_branch.local_name();
            let upstream = current_branch
                .upstream_name()
                .unwrap_or_else(|| "[No upstream branch tracked]".to_string());

            println!("{local}:{upstream}");
        },
        Commands::Checkout(CheckoutArgs { branch_name }) => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let res = repo.checkout(&branch_name);
            println!("{res:?}");
        },
        Commands::Push => {
            let repo = Repo::open(&config).expect("failed to open repository");
            let res = repo.push();
            println!("{res:?}")
        },
    }

    Ok(())
}


#[derive(Parser)]
#[command(
    flatten_help = true,
    disable_help_flag = true,
    disable_help_subcommand = false,
    help_template = "usage: {usage}"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clone(CloneArgs),
    Add(AddArgs),
    Commit(CommitArgs),
    Status,
    Branches,
    #[command(name = "current-branch")]
    CurrentBranch,
    Checkout(CheckoutArgs),
    Push,
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

#[derive(Args)]
struct CloneArgs {
    url: String,
}

fn print_status_summary(summary: &StatusSummary) {
    let StatusSummary {
        branch_name,
        staged,
        not_staged,
        untracked,
    } = summary;

    println!("on branch {branch_name}");

    if staged.is_empty() && not_staged.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return;
    }

    let print_section = |header, contents: &[_]| {
        if contents.is_empty() {
            return;
        }
        println!("{header}");
        for file in contents {
            println!("\t{file}");
        }
    };

    print_section("Changes to be committed:", &staged);
    print_section("Changes not staged for commit:", &not_staged);
    print_section("Untracked files:", &untracked);
}
