use std::collections::HashMap;

use crate::config::Config;
use crate::database::models::FactoidEnum;
use crate::database::Db;

use failure::Error;
use irc::client::ext::ClientExt;
use irc::client::IrcClient;
use irc::proto::Message;

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub arguments: Vec<&'a str>,
}

impl<'a> Command<'a> {
    pub fn try_parse(
        source_nick: &'a str,
        message: &'a str,
        config: &Config,
    ) -> Option<Command<'a>> {
        let command_str = if !message.starts_with(&config.bot_settings.command_indicator) {
            let start = message.find(&format!("{{{}", config.bot_settings.command_indicator))?
                + config.bot_settings.command_indicator.len()
                + 1;
            let end = message.split_at(start).1.find('}')?;

            &message[start..(start + end)]
        } else {
            message
                .split_at(config.bot_settings.command_indicator.len())
                .1
        };

        let mut parts = command_str.split(' ');
        let command = parts.next()?;
        let args = parts.collect();
        Some(Command {
            source_nick,
            command_str: command,
            arguments: args,
        })
    }
}

#[derive(Debug)]
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

type MessageHandler = fn(Command, &Config, &Db) -> Result<Response, Error>;

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

    pub fn handle(&self, command: Command, config: &Config) -> Result<Response, Error> {
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

pub fn handle_message(client: &IrcClient, message: &Message, config: &Config, handler: &Handler) {
    println!("{:?}", message);
    let (target, msg) = match message.command {
        irc::proto::command::Command::PRIVMSG(ref target, ref msg) => (target, msg),
        _ => return,
    };

    let user = message.source_nickname().unwrap();
    if config.bot_settings.blacklisted_users.contains(&user.into()) {
        return;
    }

    if let Some(command) = Command::try_parse(user, msg, config) {
        let result = match handler.handle(command, config) {
            Ok(response) => response,
            Err(err) => {
                println!("{:?}", err);
                Response::Say("unexpected error when executing command".into())
            }
        };

        let target = message.response_target().unwrap_or(target);
        println!("{:?}", result);
        match result {
            Response::Say(message) => client.send_privmsg(target, &message).unwrap(),
            Response::Act(message) => client.send_action(target, &message).unwrap(),
            Response::Notice(message) => client.send_notice(target, &message).unwrap(),
            Response::None => {}
        }
    }
}
