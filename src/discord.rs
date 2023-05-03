use eyre::Result;
use serenity::{
    async_trait,
    framework::StandardFramework,
    futures::lock::Mutex,
    model::prelude::{Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::{env, sync::Arc};

use crate::model::TeachBot;

const DISCORD_API_TOKEN: &str = "DISCORD_API_TOKEN";

struct Handler {
    state: Arc<Mutex<TeachBot>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TeachBot::default())),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let mut state = self.state.lock().await;

        let message_and_reply = state
            .handle(&msg.content)
            .await
            .expect("could not interact with bot");

        if let Some(reply) = message_and_reply.reply {
            if let Err(why) = msg.reply(&ctx.http, reply).await {
                println!("Error sending reply: {:?}", why);
            }
        }

        if let Some(message) = message_and_reply.channel {
            if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub async fn do_chat_bot() -> Result<()> {
    let framework = StandardFramework::new().configure(|c| c.prefix("~"));

    let token =
        env::var(DISCORD_API_TOKEN).expect("Expected a DISCORD_API_TOKEN in the environment");

    let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler::new())
        .await?;

    if let Err(why) = client.start().await {
        println!("Bot client error: {:?}", why);
    }

    Ok(())
}
