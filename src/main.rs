use discord::do_chat_bot;
use dotenv::dotenv;
use eyre::Result;

pub mod discord;
pub mod gpt;
pub mod model;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("could not instantiate dotenv");

    do_chat_bot().await?;

    Ok(())
}
