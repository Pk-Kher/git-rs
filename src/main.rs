use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{self, Read, Write};

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
        /// lists test values
        #[arg(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
}
fn main() -> io::Result<()> {
    let args = Args::parse();
    println!("{:?}", args);
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
        } => {
            let f = fs::read(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..],
            ))
            .unwrap();
            let mut z = ZlibDecoder::new(&f[..]);
            let mut s = String::new();
            z.read_to_string(&mut s)?;
            print!("{}", s);
            io::stdout().flush().expect("Failed to flush stdout");
        }
    }
    Ok(())
}
