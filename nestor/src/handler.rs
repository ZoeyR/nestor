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

#[derive(Debug, PartialEq, Eq, Clone)]
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

        Command::from_command_str(source_nick, command_str)
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
    use crate as nestor;
    use crate::command;
    use crate::request::State;
    use crate::Outcome;
    use crate::Response;

    #[command("foo")]
    fn foo() -> &'static str {
        "foo"
    }

    #[command("failed")]
    fn failed(_n: State<u32>) -> &'static str {
        "failure"
    }

    #[command]
    fn default() -> &'static str {
        "default"
    }

    fn run(config: &str, command: &str, source: &str) -> Outcome {
        use super::{Command, Request};
        use crate::handler::{CommandHandler, CommandRouter};
        use crate::inventory;
        use futures_preview::FutureExt;
        use state::Container;
        use tokio::prelude::Future;
        use tokio_async_await::compat::backward;

        let config = toml::de::from_str(config).unwrap();

        let mut router = CommandRouter::new();
        let routes = inventory::iter::<Box<dyn CommandHandler>>
            .into_iter()
            .map(|route| (route.route_id(), route.as_ref()))
            .collect::<Vec<_>>();
        router.add_handlers(routes);
        let container = Container::new();
        let command = Command::from_command_str(source, command).unwrap();
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };

        let result: Result<Outcome, ()> =
            backward::Compat::new(router.route(&request).map(Ok)).wait();
        result.unwrap()
    }

    #[test]
    fn command_routes() {
        let result = run(
            r##"
			blacklisted_users = []
			command_indicator = ["!"]
			alias_depth = 3
		"##,
            "foo",
            "",
        );

        match result {
            Outcome::Success(res) => assert_eq!(res, Response::Notice("foo".into())),
            _ => panic!("unexpected outcome"),
        }
    }

    #[test]
    fn ignore_blacklist() {
        let result = run(
            r##"
			blacklisted_users = ["ignored_user"]
			command_indicator = ["!"]
			alias_depth = 3
		"##,
            "foo",
            "ignored_user",
        );

        match result {
            Outcome::Success(res) => assert_eq!(res, Response::None),
            _ => panic!("unexpected outcome"),
        }
    }

    #[test]
    fn allow_other_users() {
        let result = run(
            r##"
			blacklisted_users = ["ignored_user"]
			command_indicator = ["!"]
			alias_depth = 3
		"##,
            "foo",
            "normal_user",
        );

        match result {
            Outcome::Success(res) => assert_eq!(res, Response::Notice("foo".into())),
            _ => panic!("unexpected outcome"),
        }
    }

    #[test]
    fn failure_on_invalid_param() {
        let result = run(
            r##"
			blacklisted_users = []
			command_indicator = ["!"]
			alias_depth = 3
		"##,
            "failed",
            "",
        );

        match result {
            Outcome::Failure(_) => {}
            _ => panic!("unexpected outcome"),
        }
    }

    #[test]
    fn default_handler() {
        let result = run(
            r##"
			blacklisted_users = []
			command_indicator = ["!"]
			alias_depth = 3
		"##,
            "not_present",
            "",
        );

        match result {
            Outcome::Success(res) => assert_eq!(res, Response::Notice("default".into())),
            _ => panic!("unexpected outcome"),
        }
    }

    #[test]
    fn parse_empty_command() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "~", &config).unwrap();

        assert_eq!(command.command_str, "");
        assert!(command.arguments.is_empty());
    }

    #[test]
    fn parse_non_empty_command() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "~foo", &config).unwrap();

        assert_eq!(command.command_str, "foo");
        assert!(command.arguments.is_empty());
    }

    #[test]
    fn parse_command_with_args() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "~foo bar baz", &config).unwrap();

        assert_eq!(command.command_str, "foo");
        assert_eq!(command.arguments, ["bar", "baz"]);
    }

    #[test]
    fn message_with_no_command() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "foo ~bar baz", &config);

        assert!(command.is_none());
    }

    #[test]
    fn parse_interpolated_command() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "test {~foo bar} baz", &config).unwrap();

        assert_eq!(command.command_str, "foo");
        assert_eq!(command.arguments, ["bar"]);
    }

    #[test]
    fn parse_command_trims_whitespace() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "~   foo bar baz  ", &config).unwrap();

        assert_eq!(command.command_str, "foo");
        assert_eq!(command.arguments, ["bar", "baz"]);
    }

    #[test]
    fn parse_command_uses_bot_name() {
        use super::Command;
        let config = toml::de::from_str(
            r##"
			blacklisted_users = []
			command_indicator = ["~"]
			alias_depth = 3
		"##,
        )
        .unwrap();
        let command = Command::try_parse("bot", "user", "bot: foo bar baz", &config).unwrap();

        assert_eq!(command.command_str, "foo");
        assert_eq!(command.arguments, ["bar", "baz"]);
    }
}
