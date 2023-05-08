use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use authentication::AuthenticationStrategy;
use clap::{Parser, ValueHint};
use discord::do_chat_bot;
use dotenvy::dotenv;
use eyre::{bail, Result};
use log::{warn, LevelFilter};
use store::Store;

mod authentication;
mod discord;
mod gpt;
mod model;
mod store;

const DEFAULT_DATA_DIR: &str = "var/data";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Location for application-related data. Defaults to var/data
    #[arg(long, value_hint = ValueHint::DirPath, value_parser)]
    data_dir: Option<PathBuf>,

    /// Location of a tokens file. If provided, enables the token-based auth strategy.
    #[arg(long, value_hint = ValueHint::DirPath, value_parser)]
    tokens_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Warn)
        .init();
    dotenv().expect("could not instantiate dotenv");

    let args = Args::parse();

    let auth_strategy = if let Some(tokens_file) = args.tokens_file {
        warn!("Tokens file provided. Starting with auth-strategy=TOKEN_LIST");
        let store = Store::connect(&args.data_dir.unwrap_or(DEFAULT_DATA_DIR.into())).await?;
        store
            .ensure_tokens(&read_tokens_file(&tokens_file)?)
            .await?;
        AuthenticationStrategy::TokenList(store)
    } else {
        warn!("No tokens file provided. Starting with auth-strategy=ALLOW_ALL");
        AuthenticationStrategy::NoAuthentication
    };

    do_chat_bot(auth_strategy).await?;

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
