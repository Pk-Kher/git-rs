use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    // plumbing command
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,
        file_path: PathBuf,
    },
    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,
        tree_object: String,
    },
    WriteTree,
    CommitTree {
        tree_sha: String,
        #[arg(short = 'p', value_name = "PARENT_COMMIT")]
        parent_commit_sha: Option<String>,
        #[arg(short = 'm', value_name = "COMMIT_MESSAGE")]
        commit_message: String,
    },
    LsFiles {
        #[arg(short = 's', long = "stage")]
        stage: bool,
        #[arg(short = 'c', long = "cached")]
        cached: bool,
    },
    UpdateIndex {
        #[arg(long = "add")]
        add: bool,
        file_path: Option<String>,
    },
    // general commands
    Commit {
        #[arg(short = 'm')]
        message: String,
    },
    // implement the git config user.name and user.email
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, object_hash)?,
        Commands::HashObject { write, file_path } => {
            commands::hash_object::invoke(write, &file_path)?;
        }
        Commands::LsTree {
            name_only,
            tree_object,
        } => commands::ls_tree::invoke(name_only, tree_object)?,
        Commands::WriteTree => {
            commands::write_tree::invoke(&PathBuf::from("."))?;
        }
        Commands::CommitTree {
            tree_sha,
            parent_commit_sha,
            commit_message,
        } => {
            commands::commit_tree::invoke(tree_sha, parent_commit_sha, commit_message)?;
        }
        Commands::LsFiles { stage, cached } => commands::ls_file::invoke(stage, cached)?,
        Commands::Commit { message } => commands::commit::invoke(&message)?,
        Commands::UpdateIndex { add, file_path } => commands::update_index::invoke(add, file_path)?,
    }
    Ok(())
}
//
// [u8; 20] is the real SHA-1 value
// bS/sFN*% is just a human-readable representation of those bytes
