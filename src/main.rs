use eyre::Result;
use inquire::Text;
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

const API_TOKEN: &str = "API_TOKEN";
const COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";
const TEACH_TOKEN: &str = "!";
const CONVERSATION_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Let's have a conversation at A2 level in Polish.";
const TEACH_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please correct any grammar or mistakes I make in the following sentences, in English. If there are no mistakes, just say 'All good'. Please only speak in English.";

async fn do_request(state: &mut ConversationState) -> Result<MessageContent> {
    let token = env::var(API_TOKEN).expect("no API_TOKEN found in env");

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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("could not instantiate dotenv");

    let mut state = ConversationState::default();
    state.history.push(MessageContent {
        role: Role::System,
        content: CONVERSATION_PROMPT.into(),
    });

    prompt_new_reply(&mut state).await?;

    Ok(())
}
