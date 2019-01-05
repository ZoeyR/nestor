use crate::database::Db;
use irc::client::prelude::*;
use std::collections::HashMap;

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub argument: &'a str,
}

impl<'a> Command<'a> {
    pub fn try_parse(message: &str) -> Option<Command> {
        if !message.starts_with('~') || message.len() <= 1 {
            return None;
        }

        let (_, remaining) = message.split_at(1);
        let parts: Vec<&str> = remaining.splitn(2, ' ').collect();

        Some(Command {
            source_nick: "",
            command_str: parts[0],
            argument: parts.get(1).unwrap_or(&""),
        })
    }
}

pub struct Handler {
    db: Db,
    commands: HashMap<&'static str, fn(Command, &Db) -> String>,
    default: Option<fn(Command, &Db) -> String>,
}

impl Handler {
    pub fn new(db: Db) -> Self {
        Handler {
            db,
            commands: HashMap::new(),
            default: None,
        }
    }

    pub fn register(&mut self, label: &'static str, handler: fn(Command, &Db) -> String) {
        self.commands.insert(label, handler);
    }

    pub fn register_default(&mut self, handler: fn(Command, &Db) -> String) {
        self.default = Some(handler);
    }

    pub fn handle(&self, command: Command) -> String {
        if self.commands.contains_key(command.command_str) {
            self.commands[command.command_str](command, &self.db)
        } else if let Some(default) = self.default {
            default(command, &self.db)
        } else {
            format!("command '{}' not found", command.command_str)
        }
    }
}
