use eyre::Result;
use regex::Regex;

use crate::gpt::Conversation;

const COMMAND_REGEX: &str = r"^!(\w+)\s+(.+)$";

const CONVERSATION_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Let's have a conversation at A2 level in Polish.";
const TEACH_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please correct any grammar or mistakes I make in the following sentence, in English. Please only speak in English.";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Chat(String),
    Ask(String),
    Cases(String),
    Example(String),
}

impl Command {
    pub fn read(s: &str) -> Option<Command> {
        let regex = Regex::new(COMMAND_REGEX).expect("implementation error - invalid regex");
        if let Some(cap) = regex.captures(s) {
            let command = &cap[1];
            let arg = &cap[2];

            match command {
                "chat" => Some(Command::Chat(arg.to_string())),
                "ask" => Some(Command::Ask(arg.to_string())),
                "cases" => Some(Command::Cases(arg.to_string())),
                "ex" => Some(Command::Example(arg.to_string())),
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

impl TeachBot {
    pub async fn handle(&mut self, message: &str) -> Result<String> {
        if let Some(command) = Command::read(message) {
            match command {
                Command::Chat(new_prompt) => {
                    self.conversation = Conversation::new(CONVERSATION_PROMPT);

                    self.chat_response(&new_prompt).await
                }
                _ => todo!(),
            }
        } else {
            self.chat_response(message).await
        }
    }

    async fn chat_response(&mut self, message: &str) -> Result<String> {
        let chat_response = self.conversation.message(message).await?;
        let teach_response = Self::fetch_teacher_thoughts(message).await?;

        Ok(format!("{chat_response}\n\n(_{teach_response}_)"))
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
