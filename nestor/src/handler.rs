use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::config::Config;
use crate::request::Request;
use crate::response::{Outcome, Response};

use failure::Error;

pub(crate) struct CommandRouter {
    commands: HashMap<&'static str, &'static dyn CommandHandler>,
    default: Option<&'static dyn CommandHandler>,
}

impl CommandRouter {
    pub fn new() -> Self {
        CommandRouter {
            commands: HashMap::new(),
            default: None,
        }
    }

    pub fn add_handlers(
        &mut self,
        handlers: Vec<(Option<&'static str>, &'static dyn CommandHandler)>,
    ) {
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
            match handler.handle(&request) {
                Ok(fut) => await!(fut),
                Err(err) => return Outcome::Failure(err),
            }
        } else if let Some(handler) = &self.default {
            match handler.handle(&request) {
                Ok(fut) => await!(fut),
                Err(err) => return Outcome::Failure(err),
            }
        } else {
            Outcome::Success(Response::None)
        }
    }
}

pub trait CommandHandler {
    fn route_id(&self) -> Option<&'static str>;

    fn handle<'a, 'r>(
        &'a self,
        request: &'a Request<'r>,
    ) -> Result<Pin<Box<Future<Output = Outcome> + 'a>>, Error>;
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

#[cfg(test)]
mod test {
    #[test]
    fn command_routes() {
        use super::{Command, Request};
        use crate as nestor;
        use crate::command;
        use crate::handler::{CommandHandler, CommandRouter};
        use crate::inventory;
        use crate::Outcome;
        use crate::Response;
        use futures_preview::FutureExt;
        use state::Container;
        use tokio::prelude::Future;
        use tokio_async_await::compat::backward;

        let config = toml::de::from_str(
            r##"
        blacklisted_users = [""]
        command_indicator = ["!"]
        alias_depth = 3
        "##,
        )
        .unwrap();

        #[command("foo")]
        fn foo() -> &'static str {
            "foo"
        }

        let mut router = CommandRouter::new();
        let routes = inventory::iter::<Box<dyn CommandHandler>>
            .into_iter()
            .map(|route| (route.route_id(), route.as_ref()))
            .collect::<Vec<_>>();
        router.add_handlers(routes);
        let container = Container::new();
        let command = Command {
            source_nick: "test_user",
            command_str: "foo".into(),
            arguments: vec![],
        };
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };
        let result = backward::Compat::new(router.route(&request).map(|res| match res {
            Outcome::Failure(_) => Err(()),
            Outcome::Success(res) => Ok(res),
            Outcome::Forward(_) => Err(()),
        }))
        .wait();

        assert_eq!(result, Ok(Response::Notice("foo".into())));
    }
}
