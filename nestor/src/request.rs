use std::ops::Deref;

use crate::config::Config;
use crate::handler::Command;
use crate::Nestor;

use failure::Error;
use irc::client::prelude::Message;
use irc::client::IrcClient;
use state::Container;

pub struct Request<'r> {
    pub(crate) config: &'r Config,
    pub(crate) command: Command<'r>,
    pub(crate) state: &'r Container,
}

impl<'r> Request<'r> {
    pub fn from_message<'c>(
        nestor: &'r Nestor,
        client: &'c IrcClient,
        message: &'r Message,
    ) -> Option<(&'r str, Self)> {
        let (default_target, msg) = match message.command {
            irc::proto::command::Command::PRIVMSG(ref target, ref msg) => (target, msg),
            _ => return None,
        };

        let user = message.source_nickname().unwrap();

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

pub struct State<'r, T: Send + 'static>(&'r T);

impl<'r, T: Send + 'static> State<'r, T> {
    pub fn inner(&self) -> &'r T {
        self.0
    }
}

impl<'r, T: Send + 'static> Deref for State<'r, T> {
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

impl<'a, 'r, T: Send + 'static> FromRequest<'a, 'r> for State<'r, T> {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Error> {
        request
            .state
            .try_get_local::<T>()
            .map(State)
            .ok_or(failure::err_msg("State object not managed."))
    }
}
