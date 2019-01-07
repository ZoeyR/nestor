use crate::database::Db;
use crate::models::FactoidEnum;
use failure::Error;
use std::collections::HashMap;

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub arguments: Vec<&'a str>,
}

impl<'a> Command<'a> {
    pub fn try_parse(source_nick: &'a str, message: &'a str) -> Option<Command<'a>> {
        if !message.starts_with('~') {
            return None;
        }

        let mut parts = message.split(' ');
        let (_, command) = parts.next()?.split_at(1);
        Some(Command {
            source_nick,
            command_str: command,
            arguments: parts.collect(),
        })
    }
}

pub enum Response {
    Say(String),
    Act(String),
    Notice(String),
    None,
}

impl Response {
    pub fn from_intent(intent: FactoidEnum, message: String) -> Self {
        match intent {
            FactoidEnum::Act => Response::Act(message),
            FactoidEnum::Say => Response::Say(message),
            _ => Response::None,
        }
    }
}

type MessageHandler = fn(Command, &crate::config::Config, &Db) -> Result<Response, Error>;

pub struct Handler {
    db: Db,
    commands: HashMap<&'static str, MessageHandler>,
    default: Option<MessageHandler>,
}

impl Handler {
    pub fn new(db: Db) -> Self {
        Handler {
            db,
            commands: HashMap::new(),
            default: None,
        }
    }

    pub fn register(&mut self, label: &'static str, handler: MessageHandler) {
        self.commands.insert(label, handler);
    }

    pub fn register_default(&mut self, handler: MessageHandler) {
        self.default = Some(handler);
    }

    pub fn handle(
        &self,
        command: Command,
        config: &crate::config::Config,
    ) -> Result<Response, Error> {
        if self.commands.contains_key(command.command_str) {
            self.commands[command.command_str](command, config, &self.db)
        } else if let Some(default) = self.default {
            default(command, config, &self.db)
        } else {
            Ok(Response::Say(format!(
                "command '{}' not found",
                command.command_str
            )))
        }
    }
}
