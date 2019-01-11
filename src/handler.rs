use crate::commands;
use crate::config::Config;
use crate::database::models::FactoidEnum;
use crate::database::Db;

use failure::Error;
use irc::client::ext::ClientExt;
use irc::client::IrcClient;
use irc::error::IrcError;
use irc::proto::Message;
use std::rc::Rc;

pub struct Command<'a> {
    pub source_nick: &'a str,
    pub command_str: &'a str,
    pub arguments: Vec<&'a str>,
}

impl<'a> Command<'a> {
    pub fn try_parse(
        our_nick: &'a str,
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

impl Response {
    pub fn from_intent(intent: FactoidEnum, message: String) -> Self {
        match intent {
            FactoidEnum::Act => Response::Act(message),
            FactoidEnum::Say => Response::Say(message),
            _ => Response::None,
        }
    }
}

pub struct Handler {
    db: Db,
}

impl Handler {
    pub fn new(db: Db) -> Self {
        Handler { db }
    }

    pub async fn handle<'a>(
        &'a self,
        command: Command<'a>,
        config: &'a Config,
    ) -> Result<Response, Error> {
        match command.command_str {
            "learn" => await!(commands::learn(command, config, &self.db)),
            "forget" => await!(commands::forget(command, config, &self.db)),
            "lock" => await!(commands::lock(command, config, &self.db)),
            "unlock" => await!(commands::unlock(command, config, &self.db)),
            "crate" => await!(commands::crate_info(command, config, &self.db)),
            "error" => await!(commands::rustc_error(command, config, &self.db)),
            "qotd" => await!(commands::qotd(command, config, &self.db)),
            _ => await!(commands::user_defined(command, config, &self.db)),
        }
    }
}

pub async fn handle_message<'a>(
    client: IrcClient,
    message: Message,
    config: Rc<Config>,
    handler: Rc<Handler>,
) -> Result<(), IrcError> {
    println!("{:?}", message);
    let (target, msg) = match message.command {
        irc::proto::command::Command::PRIVMSG(ref target, ref msg) => (target, msg),
        _ => return Ok(()),
    };

    let user = message.source_nickname().unwrap();
    if config.bot_settings.blacklisted_users.contains(&user.into()) {
        return Ok(());
    }

    if let Some(command) = Command::try_parse(client.current_nickname(), user, msg, &config) {
        let result = match await!(handler.handle(command, &config)) {
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

    Ok(())
}
