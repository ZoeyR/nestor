use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::config::Config;
use crate::request::Request;
use crate::Nestor;

use failure::Error;
use irc::client::prelude::Message;
use irc::client::IrcClient;

pub(crate) struct CommandRouter {
    commands: HashMap<&'static str, Box<dyn CommandHandler>>,
}

impl CommandRouter {
    pub fn new() -> Self {
        CommandRouter {
            commands: HashMap::new(),
        }
    }

    pub async fn route<'r>(&'r self, request: Request<'r>) -> Result<Response, Error> {
        if request
            .config
            .bot_settings
            .blacklisted_users
            .contains(&request.command.source_nick.into())
        {
            return Ok(Response::None);
        }

        if let Some(handler) = self.commands.get(request.command.command_str) {
            await!(handler.handle(&request))
        } else {
            Ok(Response::None)
        }
    }
}

pub trait CommandHandler {
    fn handle<'a, 'r>(
        &'a self,
        request: &'a Request<'r>,
    ) -> Pin<Box<Future<Output = Result<Response, Error>> + 'a>>;
}

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub arguments: Vec<&'a str>,
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

        let mut parts = command_str.trim().split(' ');
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
