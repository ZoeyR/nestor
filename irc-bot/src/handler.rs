use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::config::Config;
use crate::request::Request;
use crate::response::{Outcome, Response};

use failure::Error;

pub(crate) struct CommandRouter {
    commands: HashMap<&'static str, Box<dyn CommandHandler>>,
    default: Option<Box<dyn CommandHandler>>,
}

impl CommandRouter {
    pub fn new() -> Self {
        CommandRouter {
            commands: HashMap::new(),
            default: None,
        }
    }

    pub fn add_handlers(&mut self, handlers: Vec<(Option<&'static str>, Box<dyn CommandHandler>)>) {
        for (label, handler) in handlers {
            if let Some(label) = label {
                self.commands.insert(label, handler);
            } else if self.default.is_none() {
                self.default = Some(handler);
            }
        }
    }

    pub async fn route<'r>(&'r self, request: &'r Request<'r>) -> Outcome {
        if request
            .config
            .bot_settings
            .blacklisted_users
            .contains(&request.command.source_nick.into())
        {
            return Outcome::Success(Response::None);
        }

        let c: &str = request.command.command_str.as_ref();
        if let Some(handler) = self.commands.get(c) {
            await!(handler.handle(&request))
        } else if let Some(handler) = &self.default {
            await!(handler.handle(&request))
        } else {
            Outcome::Success(Response::None)
        }
    }
}

pub trait CommandHandler {
    fn handle<'a, 'r>(
        &'a self,
        request: &'a Request<'r>,
    ) -> Pin<Box<Future<Output = Outcome> + 'a>>;
}

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: String,
    pub arguments: Vec<String>,
}

impl<'a> Command<'a> {
    pub fn try_parse<'u>(
        our_nick: &'u str,
        source_nick: &'a str,
        message: &'a str,
        config: &Config,
    ) -> Option<Command<'a>> {
        let command_str = config
            .bot_settings
            .command_indicator
            .iter()
            .chain(std::iter::once(&format!("{}:", our_nick)))
            .filter_map(|indicator| {
                if !message.starts_with(indicator) {
                    message.find(&format!("{{{}", indicator)).and_then(|start| {
                        let start = start + indicator.len() + 1;
                        let end = message.split_at(start).1.find('}')?;

                        Some(&message[start..(start + end)])
                    })
                } else {
                    Some(message.split_at(indicator.len()).1)
                }
            })
            .nth(0)?;

        let mut parts = command_str.trim().split(' ').map(String::from);
        let command = parts.next()?;
        let args = parts.collect();
        Some(Command {
            source_nick,
            command_str: command,
            arguments: args,
        })
    }

    pub fn from_command_str(source_nick: &'a str, command_str: &str) -> Option<Command<'a>> {
        let mut parts = command_str.trim().split(' ').map(String::from);
        let command = parts.next()?;
        let args = parts.collect();
        Some(Command {
            source_nick,
            command_str: command,
            arguments: args,
        })
    }
}
