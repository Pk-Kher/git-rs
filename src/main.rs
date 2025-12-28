use anyhow::Context;
use clap::{Parser, Subcommand, command};
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
        Commands::Commit { message } => {
            // to commit we need value same as the commit-tree
            //value of head=ref: refs/heads/master
            let head_ref =
                std::fs::read_to_string(".git/HEAD").context("Failed to read the head")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("Refusing to commit onto detached HEAD")
            };
            let head_ref = head_ref.trim();
            let parent_sha = std::fs::read_to_string(format!("./.git/{head_ref}"))
                .with_context(|| format!("Failed to read the parent commit hash:{}", head_ref))?;
            let parent_sha = parent_sha.trim();

            let Some(tree_hash) = commands::write_tree::write_tree_for(&PathBuf::from("."))? else {
                eprintln!("Not commiting the empty tree");
                return Ok(());
            };
            let tree_hash = hex::encode(tree_hash);

            let commit_hash =
                commands::commit_tree::write_commit(&tree_hash, Some(parent_sha), &message)
                    .with_context(|| format!("Failed to generate commit hash"))?;
            let commit_hash = hex::encode(commit_hash);

            std::fs::write(format!(".git/{head_ref}"), &commit_hash)
                .with_context(|| format!("Failed to update the HEAD ref at :{}", head_ref))?;
            eprintln!("HEAD is now at {}", commit_hash);
        }
    }
    Ok(())
}
//
// [u8; 20] is the real SHA-1 value
// bS/sFN*% is just a human-readable representation of those bytes
