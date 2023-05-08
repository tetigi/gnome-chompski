use eyre::Result;
use log::{error, warn};
use regex::Regex;
use serenity::{
    async_trait,
    framework::StandardFramework,
    futures::lock::Mutex,
    model::prelude::{Message, Ready, UserId},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::{collections::HashMap, env, sync::Arc};

use crate::{model::TeachBot, store::Store};

const DISCORD_API_TOKEN: &str = "DISCORD_API_TOKEN";
const TOKEN_REGEX: &str = r"^!token\s+(.+)$";

struct Handler {
    state: Arc<Mutex<HashMap<UserId, TeachBot>>>,
    store: Store,
}

impl Handler {
    fn new(store: Store) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            store,
        }
    }

    async fn authenticate_user(&self, user: &UserId, ctx: &Context, msg: &Message) -> Result<bool> {
        let user_id = user.0.to_string();
        let reply = if !self.store.is_allocated(&user_id).await? {
            let token_regex =
                Regex::new(TOKEN_REGEX).expect("implementation error - invalid regex");
            if let Some(cap) = token_regex.captures(&msg.content) {
                let token = &cap[1];
                if self.store.is_token_valid(token).await? {
                    self.store.allocate(&user_id, token).await?;
                    "Looks good! Your user is now authenticated :D"
                } else {
                    "Unfortunately, your token appears to be invalid or has already been used before.\n\nAre you sure you entered it correctly?"
                }
            } else {
                "Hey there!\n\nUnfortunately, you are not authenticated yet. Please paste in your authentication token in the following format:\n\n`!token YOUR_TOKEN`"
            }
        } else {
            return Ok(true);
        };

        msg.reply(&ctx.http, reply).await?;

        Ok(false)
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if !self
            .authenticate_user(&msg.author.id, &ctx, &msg)
            .await
            .unwrap()
        {
            return;
        };

        let mut all_bots = self.state.lock().await;
        let state = all_bots.entry(msg.author.id).or_default();

        let typing = match msg.channel_id.start_typing(&ctx.http) {
            Err(why) => {
                error!("Error starting typing: {:?}", why);
                None
            }
            Ok(typing) => Some(typing),
        };

        let message_and_reply = state
            .handle(&msg.content)
            .await
            .expect("could not interact with bot");

        if let Some(typing) = typing {
            let _ = typing.stop();
        }

        if let Some(reply) = message_and_reply.reply {
            if let Err(why) = msg.reply(&ctx.http, reply).await {
                error!("Error sending reply: {:?}", why);
            }
        }

        if let Some(message) = message_and_reply.channel {
            if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        warn!("{} is connected!", ready.user.name);
    }
}

pub async fn do_chat_bot(store: Store) -> Result<()> {
    let framework = StandardFramework::new().configure(|c| c.prefix("~"));

    let token =
        env::var(DISCORD_API_TOKEN).expect("Expected a DISCORD_API_TOKEN in the environment");

    let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler::new(store))
        .await?;

    if let Err(why) = client.start().await {
        error!("Bot client error: {:?}", why);
    }

    Ok(())
}
