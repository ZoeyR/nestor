#![feature(await_macro, async_await, futures_api)]

use std::rc::Rc;

use crate::config::Config;
use crate::handler::{Command, CommandHandler, CommandRouter};
use crate::request::Request;
use crate::response::{Outcome, Response};

use futures::future::Future;
use irc::client::ext::ClientExt;
use irc::client::reactor::IrcReactor;
use irc::client::IrcClient;
use irc::proto::Message;
use state::Container;
use tokio_async_await::compat::backward;

pub use failure::Error;

#[doc(hidden)]
pub use inventory;

pub use nestor_codegen::command;

pub mod config;
pub mod handler;
pub mod request;
pub mod response;

inventory::collect!(Box<dyn CommandHandler>);

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

    pub fn manage<T: Send + Sync + 'static>(self, state: T) -> Self {
        self.state.set(state);

        self
    }

    pub fn activate(mut self) {
        let routes = inventory::iter::<Box<dyn CommandHandler>>
            .into_iter()
            .map(|route| (route.route_id(), route.as_ref()))
            .collect();
        self.router.add_handlers(routes);

        let nestor = Rc::new(self);
        let mut reactor = IrcReactor::new().unwrap();
        let handle = reactor.inner_handle();
        let client = reactor
            .prepare_client_and_connect(&nestor.config.irc_config)
            .unwrap();
        client.identify().unwrap();
        reactor.register_client_with_handler(client, move |client, message| {
            let future = handle_message(nestor.clone(), client.clone(), message);
            handle.spawn(backward::Compat::new(future).map_err(|_| ()));
            Ok(())
        });

        reactor.run().unwrap();
    }
}

async fn handle_message(
    nestor: Rc<Nestor>,
    client: IrcClient,
    message: Message,
) -> Result<(), Error> {
    if let Some((responder, mut request)) = Request::from_message(&nestor, &client, &message) {
        for _ in 0..nestor.config.bot_settings.alias_depth {
            let response = await!(nestor.router.route(&request));
            let response = match response {
                Outcome::Forward(c) => {
                    request = Request {
                        config: &nestor.config,
                        state: &nestor.state,
                        command: Command::from_command_str(request.command.source_nick, &c)
                            .ok_or(failure::err_msg("Internal error with command alias"))?,
                    };
                    continue;
                }
                Outcome::Success(response) => response,
                Outcome::Failure(err) => {
                    println!("{:?}", err);
                    Response::Say("Unexpected error executing command".into())
                }
            };

            match response {
                Response::Say(message) => client.send_privmsg(responder, &message)?,
                Response::Act(message) => client.send_action(responder, &message)?,
                Response::Notice(message) => client.send_notice(responder, &message)?,
                Response::None => {}
            }

            return Ok(());
        }

        client.send_notice(responder, "alias depth too deep")?;
    }

    Ok(())
}
