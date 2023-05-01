use eyre::Result;
use regex::Regex;

use crate::gpt::Conversation;

const COMMAND_REGEX: &str = r"^!(\w+)\s+(.+)$";

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
                "def" => Some(Command::Define(arg.to_string())),
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
                    self.conversation.message(new_prompt).await
                }
                Command::Ask(question) => Conversation::ask(TEACH_PROMPT, question).await,
                Command::Define(question) => Conversation::ask(DEFINE_PROMPT, question).await,
                Command::Cases(word) => Conversation::ask(CASES_PROMPT, word).await,
                Command::Example(word) => Conversation::ask(EXAMPLES_PROMPT, word).await,
            }
        } else {
            self.chat_response(message).await
        }
    }

    async fn chat_response(&mut self, message: &str) -> Result<String> {
        let (chat_response, teach_response) = tokio::join!(
            self.conversation.message(message),
            Self::fetch_teacher_thoughts(message)
        );

        Ok(format!("(_{}_)\n\n{}", teach_response?, chat_response?))
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
