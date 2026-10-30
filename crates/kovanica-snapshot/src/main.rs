use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kovanica_storage::Storage;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "kovanica-snapshot",
    about = "State snapshot export/verify/import"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Export {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Verify {
        #[arg(long)]
        file: PathBuf,
    },
    Import {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        force: bool,
    },
}

fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("state.redb")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Export { data_dir, out } => {
            let storage = Storage::new(db_path(&data_dir)).context("open storage")?;
            let txn = storage.begin_read().context("begin read")?;
            storage
                .state()
                .export_snapshot(&txn, &out)
                .context("export snapshot")?;
            println!("exported {}", out.display());
        }
        Cmd::Verify { file } => {
            let bytes = std::fs::read(&file).context("read file")?;
            let data = kovanica_storage::state_store::StateStore::decode_snapshot(&bytes)
                .context("decode snapshot")?;
            println!("height={} state_root={:?}", data.height, data.state_root);
        }
        Cmd::Import {
            data_dir,
            file,
            force,
        } => {
            if !force {
                anyhow::bail!("import requires --force");
            }
            std::fs::create_dir_all(&data_dir).context("create data dir")?;
            let storage = Storage::new(db_path(&data_dir)).context("open storage")?;
            let txn = storage.begin_write().context("begin write")?;
            storage
                .state()
                .import_snapshot(&txn, &file)
                .context("import snapshot")?;
            txn.commit().context("commit")?;
            println!("imported into {}", data_dir.display());
        }
    }
    Ok(())
}
