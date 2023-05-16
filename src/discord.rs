use eyre::Result;
use log::{error, warn};
use serenity::{
    async_trait,
    framework::StandardFramework,
    futures::lock::Mutex,
    model::{
        prelude::{Channel, Message, Ready, UserId},
        user::User,
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::{collections::HashMap, env, sync::Arc, time::Duration};

use crate::{
    authentication::{AuthResult, AuthenticationStrategy},
    model::{MessageReply, TeachBot},
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

macro_rules! just_log_error {
    ($context:tt, $x:expr) => {{
        match $x {
            Err(e) => {
                error!("Error while $context: {e:?}");
                return;
            }
            Ok(res) => res,
        }
    }};
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // We don't reply to bots
        if msg.author.bot {
            return;
        }

        // We don't reply on public / non-private channels
        match msg.channel(&ctx.http).await {
            Ok(channel) => {
                if !matches!(channel, Channel::Private(_)) {
                    just_log_error!(
                        "sending reply",
                        msg.reply(
                            &ctx.http,
                            "I'm shy, so I don't talk in public! Message me directly to chat :)",
                        )
                        .await
                    );
                    return;
                }
            }
            Err(why) => {
                error!("Could not fetch channel due to {why:?}");
                return;
            }
        }

        // Authenticate if necessary
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
            msg.author.name, msg.author.id.0
        );

        // Find the relevant bot for this user
        let mut all_bots = self.state.lock().await;

        let state = if let Some(state) = all_bots.get_mut(&msg.author.id) {
            state
        } else {
            just_log_error!(
                "sending reply",
                msg.reply(
                    &ctx.http,
                    "_This is your first message of the session. Did Gnome Chompski just wake up?_",
                )
                .await
            );
            all_bots.entry(msg.author.id).or_default()
        };

        // Start typing, indicating to the user that we're doing some work
        let typing = just_log_error!("starting typing", msg.channel_id.start_typing(&ctx.http));

        // Get the relevant reply from bot. We timeout after 10 seconds, as ChatGPT is probably
        // overloaded and just won't reply to us.
        let message_and_reply = match tokio::time::timeout(
            Duration::from_secs(20),
            state.handle(&msg.content),
        )
        .await
        {
            Ok(maybe_msg) => match maybe_msg {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error while interacting with bot: {e:?}");
                    MessageReply::reply("I'm sorry, I don't understand :(

_Something went wrong whilst communicating with Gnome Chompski. Please restate your reply and try again._")
                }
            },
            Err(_) => MessageReply::reply(
                "...I'm sorry, I wasn't paying attention. What were we talking about?

_ChatGPT request timed out. Please write your reply again._
",
            ),
        };

        // Stop typing before sending the message back
        let _ = typing.stop();

        // Send the reply as a reply (wow!)
        if let Some(reply) = message_and_reply.reply {
            just_log_error!("sending reply", msg.reply(&ctx.http, reply).await);
        }

        // Send the channel message
        if let Some(message) = message_and_reply.channel {
            just_log_error!("sending reply", msg.reply(&ctx.http, message).await);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_just_log_error_passes_ok() {
        let x: Result<usize, String> = Ok(123);
        let res = just_log_error!("doing foo", x);

        assert_eq!(res, 123);
    }
}
