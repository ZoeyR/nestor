use crate::database::Db;
use failure::Error;
use irc::client::prelude::*;
use std::collections::HashMap;

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub argument: &'a str,
}

impl<'a> Command<'a> {
    pub fn try_parse(source_nick: &'a str, message: &'a str) -> Option<Command<'a>> {
        if !message.starts_with('~') {
            return None;
        }

        let parts: Vec<&str> = message.splitn(2, ' ').collect();

        Some(Command {
            source_nick: source_nick,
            command_str: parts[0],
            argument: parts.get(1).unwrap_or(&""),
        })
    }
}

pub struct Handler {
    db: Db,
    commands: HashMap<&'static str, fn(Command, &Db) -> Result<String, Error>>,
    default: Option<fn(Command, &Db) -> Result<String, Error>>,
}

impl Handler {
    pub fn new(db: Db) -> Self {
        Handler {
            db,
            commands: HashMap::new(),
            default: None,
        }
    }

    pub fn register(
        &mut self,
        label: &'static str,
        handler: fn(Command, &Db) -> Result<String, Error>,
    ) {
        self.commands.insert(label, handler);
    }

    pub fn register_default(&mut self, handler: fn(Command, &Db) -> Result<String, Error>) {
        self.default = Some(handler);
    }

    pub fn handle(&self, command: Command) -> Result<String, Error> {
        if self.commands.contains_key(command.command_str) {
            self.commands[command.command_str](command, &self.db)
        } else if let Some(default) = self.default {
            default(command, &self.db)
        } else {
            Ok(format!("command '{}' not found", command.command_str))
        }
    }
}
