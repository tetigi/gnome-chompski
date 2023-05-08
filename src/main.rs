use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::{Parser, ValueHint};
use discord::do_chat_bot;
use dotenvy::dotenv;
use eyre::{bail, Result};
use log::LevelFilter;
use store::Store;

mod discord;
mod gpt;
mod model;
mod store;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Location for application-related data. Defaults to var/data
    #[arg(long, value_hint = ValueHint::DirPath, value_parser)]
    data_dir: Option<PathBuf>,

    /// Location of a tokens file to ensure is loaded. Does nothing if not provided.
    #[arg(long, value_hint = ValueHint::DirPath, value_parser)]
    tokens_file: Option<PathBuf>,
}

const DEFAULT_DATA_DIR: &str = "var/data";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Warn)
        .init();
    dotenv().expect("could not instantiate dotenv");

    let args = Args::parse();

    let store = Store::connect(&args.data_dir.unwrap_or(DEFAULT_DATA_DIR.into())).await?;

    if let Some(tokens_file) = args.tokens_file {
        store
            .ensure_tokens(&read_tokens_file(&tokens_file)?)
            .await?;
    }

    do_chat_bot(store).await?;

    Ok(())
}

fn read_tokens_file(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        bail!("Tokens file {path:?} does not exist");
    }

    let file = File::open(path)?;
    let lines = BufReader::new(file).lines();

    let mut out = vec![];
    for line in lines {
        out.push(line?.trim().to_string());
    }

    Ok(out)
}
