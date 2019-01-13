#![feature(await_macro, async_await, futures_api)]

use std::rc::Rc;

use crate::config::Config;
use crate::handler::{CommandRouter, Response};
use crate::request::Request;

use futures::future::Future;
use irc::client::ext::ClientExt;
use irc::client::reactor::IrcReactor;
use state::Container;
use tokio_async_await::compat::backward;

mod config;
mod handler;
mod request;

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

    pub fn activate(self) {
        let nestor = Rc::new(self);
        let mut reactor = IrcReactor::new().unwrap();
        let handle = reactor.inner_handle();
        let client = reactor
            .prepare_client_and_connect(&nestor.config.irc_config)
            .unwrap();
        client.identify().unwrap();
        reactor.register_client_with_handler(client, move |client, message| {
            let nestor = nestor.clone();
            let client = client.clone();
            if let Some(request) = Request::from_message(&nestor, &client, message) {
                let future = async {
                    let result = match await!(self.router.route(request)) {
                        Ok(response) => response,
                        Err(err) => {
                            println!("{:?}", err);
                            Response::Say("unexpected error when executing command".into())
                        }
                    };

                    match result {
                        Response::Say(message) => {
                            client.send_privmsg(request.response, &message).unwrap()
                        }
                        Response::Act(message) => {
                            client.send_action(request.response, &message).unwrap()
                        }
                        Response::Notice(message) => {
                            client.send_notice(request.response, &message).unwrap()
                        }
                        Response::None => {}
                    }

                    Ok(())
                };

                handle.spawn(backward::Compat::new(future));
            }

            Ok(())
        });
    }
}
