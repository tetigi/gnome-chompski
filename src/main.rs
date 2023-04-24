use eyre::Result;
use serenity::{
    async_trait,
    framework::StandardFramework,
    model::prelude::{Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::env;

use dotenv::dotenv;

pub mod gpt;

const DISCORD_API_TOKEN: &str = "DISCORD_API_TOKEN";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// Taken from https://github.com/serenity-rs/serenity/blob/current/examples/e01_basic_ping_bot/src/main.rs
async fn do_chat_bot() {
    let framework = StandardFramework::new().configure(|c| c.prefix("~")); // set the bot's prefix to ~";

    // Configure the client with your Discord bot token in the environment.
    let token =
        env::var("DISCORD_API_TOKEN").expect("Expected a DISCORD_API_TOKEN in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Bot client error: {:?}", why);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("could not instantiate dotenv");

    do_chat_bot().await;

    /*
    let mut state = ConversationState::default();
    state.history.push(MessageContent {
        role: Role::System,
        content: CONVERSATION_PROMPT.into(),
    });

    prompt_new_reply(&mut state).await?;
    */

    Ok(())
}
