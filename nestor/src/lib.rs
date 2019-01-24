#![feature(await_macro, async_await, futures_api)]

use std::rc::Rc;

use crate::config::Config;
use crate::handler::{Command, CommandHandler, CommandRouter};
use crate::request::Request;
use crate::response::{Outcome, Response};

use irc::client::ext::ClientExt;
use irc::client::reactor::IrcReactor;
use irc::client::IrcClient;
use irc::proto::Message;
use state::Container;
use tokio_async_await::compat::backward;

pub use futures_preview::FutureExt;
pub use failure::Error;

pub mod config;
pub mod handler;
pub mod request;
pub mod response;

pub struct Nestor {
    state: Container,
    config: Config,
    router: CommandRouter,
}

impl Nestor {
    pub fn build() -> Self {
        let config = Config::load("nestor.toml").unwrap();
        Nestor {
            state: Container::new(),
            config,
            router: CommandRouter::new(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Nestor {
            state: Container::new(),
            config,
            router: CommandRouter::new(),
        }
    }

    pub fn manage<T: Send + 'static, F>(self, state: F) -> Self
    where
        T: Send + 'static,
        F: Fn() -> T + 'static,
    {
        self.state.set_local(state);

        self
    }

    pub fn route(mut self, handlers: Vec<(Option<&'static str>, Box<dyn CommandHandler>)>) -> Self {
        self.router.add_handlers(handlers);

        self
    }

    pub fn activate(self) {
        let nestor = Rc::new(self);
        let mut reactor = IrcReactor::new().unwrap();
        let handle = reactor.inner_handle();
        let client = reactor
            .prepare_client_and_connect(&nestor.config.irc_config)
            .unwrap();
        client.identify().unwrap();
        reactor.register_client_with_handler(client, move |client, message| {
            let future = handle_message(nestor.clone(), client.clone(), message);
            handle.spawn(backward::Compat::new(future));
            Ok(())
        });

        reactor.run().unwrap();
    }
}

async fn handle_message(nestor: Rc<Nestor>, client: IrcClient, message: Message) -> Result<(), ()> {
    if let Some((responder, mut request)) = Request::from_message(&nestor, &client, &message) {
        for _ in 0..nestor.config.bot_settings.alias_depth {
            let response = await!(nestor.router.route(&request));
            let response = match response {
                Outcome::Forward(c) => {
                    request = Request {
                        config: &nestor.config,
                        state: &nestor.state,
                        command: Command::from_command_str("", &c).unwrap(),
                    };
                    continue;
                }
                Outcome::Success(response) => response,
                Outcome::Failure(_) => Response::Say("Unexpected error executing command".into()),
            };

            match response {
                Response::Say(message) => client.send_privmsg(responder, &message).unwrap(),
                Response::Act(message) => client.send_action(responder, &message).unwrap(),
                Response::Notice(message) => client.send_notice(responder, &message).unwrap(),
                Response::None => {}
            }

            return Ok(());
        }

        client.send_notice(responder, "alias depth too deep").unwrap();
    }

    Ok(())
}
