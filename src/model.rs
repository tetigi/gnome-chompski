use eyre::Result;
use regex::Regex;

use crate::gpt::Conversation;

const COMMAND_REGEX: &str = r"^!(\w+)\s+(.+)$";
const NO_ARG_COMMAND_REGEX: &str = r"^!(\w+)$";

const CONVERSATION_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Let's have a conversation at A2 level in Polish. Do not provide any translations.";
const TEACH_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please correct any grammar or mistakes I make in the following sentences, in English. Please only speak in English. Do not patronise me with complements.";
const DEFINE_PROMPT: &str =
    "I am learning to speak Polish. You are a Polish teacher. What does this word mean?";
const CASES_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please provide me with all of the cases for the following Polish word.";
const EXAMPLES_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please provide me with 3 example sentences and translations containing the following Polish word.";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Chat(String),
    Ask(String),
    Define(String),
    Cases(String),
    Example(String),
    Undo,
}

impl Command {
    pub fn read(s: &str) -> Option<Command> {
        let cmd_regex = Regex::new(COMMAND_REGEX).expect("implementation error - invalid regex");
        let no_arg_cmd_regex =
            Regex::new(NO_ARG_COMMAND_REGEX).expect("implementation error - invalid regex");
        if let Some(cap) = cmd_regex.captures(s) {
            let command = &cap[1];
            let arg = &cap[2];

            match command {
                "chat" => Some(Command::Chat(arg.to_string())),
                "ask" => Some(Command::Ask(arg.to_string())),
                "def" => Some(Command::Define(arg.to_string())),
                "cases" => Some(Command::Cases(arg.to_string())),
                "ex" => Some(Command::Example(arg.to_string())),
                _ => None,
            }
        } else if let Some(cap) = no_arg_cmd_regex.captures(s) {
            let command = &cap[1];

            match command {
                "undo" => Some(Command::Undo),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct TeachBot {
    conversation: Conversation,
}

pub struct MessageReply {
    // What should be attached to the original message
    pub reply: Option<String>,

    // What should be sent to the channel
    pub channel: Option<String>,
}

impl MessageReply {
    pub fn channel(msg: impl Into<String>) -> Self {
        Self {
            channel: Some(msg.into()),
            reply: None,
        }
    }

    pub fn reply(msg: impl Into<String>) -> Self {
        Self {
            reply: Some(msg.into()),
            channel: None,
        }
    }

    pub fn message_and_reply(msg: impl Into<String>, reply: impl Into<String>) -> Self {
        Self {
            reply: Some(reply.into()),
            channel: Some(msg.into()),
        }
    }
}

impl TeachBot {
    pub async fn handle(&mut self, message: &str) -> Result<MessageReply> {
        if let Some(command) = Command::read(message) {
            let msg = match command {
                Command::Chat(new_prompt) => {
                    self.conversation = Conversation::new(CONVERSATION_PROMPT);
                    self.conversation.message(new_prompt).await
                }
                Command::Ask(question) => Conversation::ask(TEACH_PROMPT, question).await,
                Command::Define(question) => Conversation::ask(DEFINE_PROMPT, question).await,
                Command::Cases(word) => Conversation::ask(CASES_PROMPT, word).await,
                Command::Example(word) => Conversation::ask(EXAMPLES_PROMPT, word).await,
                Command::Undo => return self.undo_reply(),
            }?;

            Ok(MessageReply::channel(msg))
        } else {
            self.chat_response(message).await
        }
    }

    fn undo_reply(&mut self) -> Result<MessageReply> {
        let message_last = self.conversation.forget_last();
        let message_last_but_one = self.conversation.forget_last();

        if message_last_but_one.is_none() && message_last.is_none() {
            Ok(MessageReply::reply(
                "There are no messages in the history to undo.",
            ))
        } else {
            let mut reply = String::new();
            reply.push_str("Forgot about the following messages:\n");

            if let Some(msg) = message_last_but_one {
                reply.push_str(&format!("> {msg}"));
            }

            if let Some(msg) = message_last {
                reply.push_str(&format!("\n> {msg}"));
            }

            Ok(MessageReply::reply(reply))
        }
    }

    async fn chat_response(&mut self, message: &str) -> Result<MessageReply> {
        let (chat_response, teach_response) = tokio::join!(
            self.conversation.message(message),
            Self::fetch_teacher_thoughts(message)
        );

        Ok(MessageReply::message_and_reply(
            chat_response?,
            teach_response?,
        ))
    }

    async fn fetch_teacher_thoughts(message: &str) -> Result<String> {
        Conversation::new(TEACH_PROMPT).message(message).await
    }
}

impl Default for TeachBot {
    fn default() -> Self {
        Self {
            conversation: Conversation::new(CONVERSATION_PROMPT),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands() {
        assert_eq!(
            Command::read("!chat foo bar"),
            Some(Command::Chat("foo bar".to_string()))
        );

        assert_eq!(
            Command::read("!ask bar baz"),
            Some(Command::Ask("bar baz".to_string()))
        );
        assert_eq!(
            Command::read("!cases quup"),
            Some(Command::Cases("quup".to_string()))
        );
        assert_eq!(
            Command::read("!ex quux!"),
            Some(Command::Example("quux!".to_string()))
        );
        assert_eq!(Command::read("!undo foo bar"), Some(Command::Undo));
    }

    #[test]
    fn test_bad_commands() {
        assert_eq!(Command::read("chat foo"), None);
        assert_eq!(Command::read("!foo"), None);
        assert_eq!(Command::read(""), None);
        assert_eq!(Command::read("!chat"), None);
        assert_eq!(Command::read("!chat "), None);
    }
}
