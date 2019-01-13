#![feature(await_macro, async_await, futures_api)]

use failure::Error;
use irc::client::IrcClient;
use irc::proto::Message;
use std::rc::Rc;

use crate::config::Config;
use crate::handler::{CommandHandler, CommandRouter, Response};
use crate::request::Request;

use futures::future::Future;
use irc::client::ext::ClientExt;
use irc::client::reactor::IrcReactor;
use state::Container;
use tokio_async_await::compat::backward;

pub mod config;
pub mod handler;
pub mod request;

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

    pub fn manage<T: Send + Sync + 'static>(self, state: T) -> Self {
        self.state.set::<T>(state);

        self
    }

    pub fn route(mut self, handlers: Vec<(&'static str, Box<dyn CommandHandler>)>) -> Self {
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
    if let Some((responder, request)) = Request::from_message(&nestor, &client, &message) {
        let response = match await!(nestor.router.route(request)) {
            Ok(response) => response,
            Err(_) => Response::Say("unexpected error when executing command".into()),
        };

        match response {
            Response::Say(message) => client.send_privmsg(responder, &message).unwrap(),
            Response::Act(message) => client.send_action(responder, &message).unwrap(),
            Response::Notice(message) => client.send_notice(responder, &message).unwrap(),
            Response::None => {}
        }
    }

    Ok(())
}
