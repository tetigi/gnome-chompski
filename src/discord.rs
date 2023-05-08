use eyre::Result;
use log::{error, warn};
use serenity::{
    async_trait,
    framework::StandardFramework,
    futures::lock::Mutex,
    model::{
        prelude::{Message, Ready, UserId},
        user::User,
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::{collections::HashMap, env, sync::Arc};

use crate::{
    authentication::{AuthResult, AuthenticationStrategy},
    model::TeachBot,
};

const DISCORD_API_TOKEN: &str = "DISCORD_API_TOKEN";

struct Handler {
    state: Arc<Mutex<HashMap<UserId, TeachBot>>>,
    auth_strategy: AuthenticationStrategy,
}

impl Handler {
    fn new(auth_strategy: AuthenticationStrategy) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            auth_strategy,
        }
    }

    async fn authenticate_user(&self, user: &User, ctx: &Context, msg: &Message) -> Result<bool> {
        let user_id = user.id.0.to_string();

        if self.auth_strategy.is_user_authenticated(&user_id).await? {
            return Ok(true);
        }

        let reply = match self
            .auth_strategy
            .add_auth_for_new_user(&user_id, &msg.content)
            .await?
        {
            AuthResult::Success => {
                warn!(
                    "User {} ({user_id}) was successfully authenticated",
                    user.name
                );
                "Looks good! Your user is now authenticated :D"
            }
            AuthResult::InvalidToken => {
                warn!(
                    "User {} ({user_id}) provided an invalid token: {}",
                    user.name, msg.content
                );
                "Unfortunately, your token appears to be invalid or has already been used before.\n\nAre you sure you entered it correctly?"
            }
            AuthResult::MalformedTokenRequest => {
                warn!(
                    "User {} ({user_id}) wrote {}, but was not authenticated",
                    user.name, msg.content
                );
                "Hey there!\n\nUnfortunately, you are not authenticated yet. Please paste in your authentication token in the following format:\n\n`!token YOUR_TOKEN`"
            }
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

        if self.auth_strategy.auth_required()
            && !self
                .authenticate_user(&msg.author, &ctx, &msg)
                .await
                .unwrap()
        {
            return;
        };

        warn!(
            "User {} ({}) is chatting with Gnome Chompski.",
            msg.author, msg.author.id.0
        );

        let mut all_bots = self.state.lock().await;

        let state = if let Some(state) = all_bots.get_mut(&msg.author.id) {
            state
        } else {
            if let Err(why) = msg
                .reply(
                    &ctx.http,
                    "_This is your first message of the session. Did Gnome Chompski just wake up?_",
                )
                .await
            {
                error!("Error sending reply: {:?}", why);
            }
            all_bots.entry(msg.author.id).or_default()
        };

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

pub async fn do_chat_bot(auth_strategy: AuthenticationStrategy) -> Result<()> {
    let framework = StandardFramework::new().configure(|c| c.prefix("~"));

    let token =
        env::var(DISCORD_API_TOKEN).expect("Expected a DISCORD_API_TOKEN in the environment");

    let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler::new(auth_strategy))
        .await?;

    if let Err(why) = client.start().await {
        error!("Bot client error: {:?}", why);
    }

    Ok(())
}
