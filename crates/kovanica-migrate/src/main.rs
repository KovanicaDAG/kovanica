use clap::{Parser, Subcommand};
use kovanica_migrate::Registry;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kovanica-migrate")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    List,
    Status {
        #[arg(long)]
        data_dir: PathBuf,
    },
    Apply {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reg = Registry::default();
    match cli.cmd {
        Cmd::List => {
            for m in &reg.migrations {
                println!("{} - {}", m.id(), m.description());
            }
        }
        Cmd::Status { data_dir } => {
            println!("data_dir: {}", data_dir.display());
            println!("known migrations: {}", reg.migrations.len());
        }
        Cmd::Apply { data_dir, dry_run } => {
            for m in &reg.migrations {
                if dry_run {
                    println!("DRY-RUN: would apply {}", m.id());
                } else {
                    m.up(&data_dir, false)?;
                    println!("applied {}", m.id());
                }
            }
        }
    }
    Ok(())
}
