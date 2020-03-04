use std::sync::Arc;

use crate::config::Config;
use crate::handler::{Command, CommandHandler, CommandRouter};
use crate::request::Request;
use crate::response::{Outcome, Response};

use futures::prelude::*;
use irc::client::prelude::*;
use state::Container;

use anyhow::anyhow;
pub use anyhow::Error;
pub use anyhow::Result;

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

        let nestor = Arc::new(self);

        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut client = Client::from_config(nestor.config.irc_config.clone())
                .await
                .unwrap();
            client.identify().unwrap();
            let mut stream = client.stream().unwrap();
            let client = Arc::new(client);
            while let Some(message) = stream.next().await.transpose().unwrap() {
                let nestor = nestor.clone();
                let client = client.clone();
                tokio::spawn(async move {
                    let _ = handle_message(nestor, client, message).await;
                });
            }
        });
    }
}

async fn handle_message(
    nestor: Arc<Nestor>,
    client: Arc<Client>,
    message: Message,
) -> Result<(), Error> {
    if let Some((responder, mut request)) = Request::from_message(&nestor, &client, &message) {
        for _ in 0..nestor.config.bot_settings.alias_depth {
            let response = nestor.router.route(&request).await;
            let response = match response {
                Outcome::Forward(c) => {
                    request = Request {
                        config: &nestor.config,
                        state: &nestor.state,
                        command: Command::from_command_str(request.command.source_nick, &c)
                            .ok_or(anyhow!("Internal error with command alias"))?,
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
