use crate::config::Config;
use crate::handler::Command;
use crate::Nestor;
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

pub struct State<'r, T: Send + Sync + 'static>(&'r T);

pub trait FromRequest<'a, 'r>: Sized {
    fn from_request(request: &'a Request<'r>) -> Option<Self>;
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Config {
    fn from_request(request: &'a Request<'r>) -> Option<Self> {
        Some(&request.config)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Command<'r> {
    fn from_request(request: &'a Request<'r>) -> Option<Self> {
        Some(&request.command)
    }
}

impl<'a, 'r, T: Send + Sync + 'static> FromRequest<'a, 'r> for State<'r, T> {
    fn from_request(request: &'a Request<'r>) -> Option<Self> {
        request.state.try_get::<T>().map(State)
    }
}
