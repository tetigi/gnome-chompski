use regex::Regex;

const COMMAND_REGEX: &str = r"^!(\w+)\s+(.+)$";

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
