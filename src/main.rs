use eyre::Result;
use inquire::Text;
use serenity::{
    async_trait,
    framework::StandardFramework,
    model::prelude::{Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::env;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Clone)]
enum Model {
    #[serde(rename(serialize = "gpt-3.5-turbo"))]
    Gpt35Turbo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MessageContent {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletion {
    model: Model,
    messages: Vec<MessageContent>,
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatChoice {
    message: MessageContent,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Default)]
struct ConversationState {
    history: Vec<MessageContent>,
}

const OPENAI_API_TOKEN: &str = "OPENAI_API_TOKEN";
const DISCORD_API_TOKEN: &str = "DISCORD_API_TOKEN";
const COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";
const TEACH_TOKEN: &str = "!";
const CONVERSATION_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Let's have a conversation at A2 level in Polish.";
const TEACH_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please correct any grammar or mistakes I make in the following sentences, in English. If there are no mistakes, just say 'All good'. Please only speak in English.";

async fn do_request(state: &mut ConversationState) -> Result<MessageContent> {
    let token = env::var(OPENAI_API_TOKEN).expect("no OPENAI_API_TOKEN found in env");

    let body = ChatCompletion {
        model: Model::Gpt35Turbo,
        messages: state.history.clone(),
        temperature: None,
    };
    let ser_body = serde_json::to_string(&body)?;

    let client = reqwest::Client::new();
    let res = client
        .post(COMPLETIONS_URL)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(ser_body)
        .send()
        .await?;

    let response: ChatCompletionResponse = res.json().await?;
    let message = response
        .choices
        .first()
        .expect("no message returned")
        .message
        .clone();

    state.history.push(message.clone());

    Ok(message)
}

fn teach_state(input: impl Into<String>) -> ConversationState {
    ConversationState {
        history: vec![
            MessageContent {
                role: Role::System,
                content: TEACH_PROMPT.into(),
            },
            MessageContent {
                role: Role::User,
                content: input.into(),
            },
        ],
    }
}

async fn prompt_new_reply(state: &mut ConversationState) -> Result<()> {
    loop {
        let user_reply = Text::new("").prompt()?;

        if user_reply.starts_with(TEACH_TOKEN) {
            let (_, rest) = user_reply.split_at(TEACH_TOKEN.len());
            let teaching_reply = do_request(&mut teach_state(rest)).await?;
            println!("Teacher: {}", teaching_reply.content);
        } else {
            state.history.push(MessageContent {
                role: Role::User,
                content: user_reply.clone(),
            });
            let teaching_reply = do_request(&mut teach_state(user_reply)).await?;
            let conversation_reply = do_request(state).await?;

            println!("Teacher: {}", teaching_reply.content);
            println!("---");
            println!("Partner: {}", conversation_reply.content);
        }

        println!();
    }
}

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
