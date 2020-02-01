use std::ops::Deref;

use crate::config::Config;
use crate::handler::Command;
use crate::Nestor;

use anyhow::anyhow;
use anyhow::Error;
use irc::client::prelude::Message;
use irc::client::Client;
use state::Container;

pub struct Request<'r> {
    pub(crate) config: &'r Config,
    pub(crate) command: Command<'r>,
    pub(crate) state: &'r Container,
}

impl<'r> Request<'r> {
    pub fn from_message<'c>(
        nestor: &'r Nestor,
        client: &'c Client,
        message: &'r Message,
    ) -> Option<(&'r str, Self)> {
        let (default_target, msg) = match message.command {
            irc::proto::command::Command::PRIVMSG(ref target, ref msg) => (target, msg),
            _ => return None,
        };

        let user = message.source_nickname()?;

        let command = Command::try_parse(client.current_nickname(), user, msg, &nestor.config)?;

        let response = message.response_target().unwrap_or(default_target);

        Some((
            response,
            Request {
                config: &nestor.config,
                command,
                state: &nestor.state,
            },
        ))
    }
}

pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
    pub fn inner(&self) -> &'r T {
        self.0
    }
}

impl<'r, T: Send + Sync + 'static> Deref for State<'r, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pub trait FromRequest<'a, 'r>: Sized {
    type Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Self::Error>;
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Config {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Self::Error> {
        Ok(&request.config)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Command<'r> {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Self::Error> {
        Ok(&request.command)
    }
}

impl<'a, 'r, T: Send + Sync + 'static> FromRequest<'a, 'r> for State<'r, T> {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Error> {
        request
            .state
            .try_get::<T>()
            .map(State)
            .ok_or(anyhow!("State object not managed."))
    }
}

#[cfg(test)]
mod test {
    use super::State;
    use crate::request::FromRequest;
    use crate::Command;
    use crate::Config;
    use crate::Request;
    use state::Container;

    #[test]
    fn config_from_request() {
        let config = toml::de::from_str(
            r##"
            blacklisted_users = []
            command_indicator = ["~", "&&"]
            alias_depth = 2
        "##,
        )
        .unwrap();
        let container = Container::new();
        let command = Command::from_command_str("user", "foo bar baz").unwrap();
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };

        let config = <&Config as FromRequest>::from_request(&request).unwrap();

        assert_eq!(config.bot_settings.command_indicator, ["~", "&&"]);
    }

    #[test]
    fn command_from_request() {
        let config = toml::de::from_str(
            r##"
            blacklisted_users = []
            command_indicator = ["~", "&&"]
            alias_depth = 2
        "##,
        )
        .unwrap();
        let container = Container::new();
        let command = Command::from_command_str("user", "foo bar baz").unwrap();
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };

        let command = <&Command as FromRequest>::from_request(&request).unwrap();

        assert_eq!(command.source_nick, "user");
        assert_eq!(command.command_str, "foo");
        assert_eq!(command.arguments, ["bar", "baz"]);
    }

    #[test]
    fn state_from_request_success() {
        let config = toml::de::from_str(
            r##"
            blacklisted_users = []
            command_indicator = ["~", "&&"]
            alias_depth = 2
        "##,
        )
        .unwrap();
        let container = Container::new();
        container.set(42u32);
        let command = Command::from_command_str("user", "foo bar baz").unwrap();
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };

        let state = <State<u32> as FromRequest>::from_request(&request).unwrap();

        assert_eq!(*state, 42u32);
    }

    #[test]
    fn state_from_request_failure() {
        let config = toml::de::from_str(
            r##"
            blacklisted_users = []
            command_indicator = ["~", "&&"]
            alias_depth = 2
        "##,
        )
        .unwrap();
        let container = Container::new();
        let command = Command::from_command_str("user", "foo bar baz").unwrap();
        let request = Request {
            config: &config,
            command: command,
            state: &container,
        };

        let state = <State<u32> as FromRequest>::from_request(&request);

        assert!(state.is_err());
    }
}
