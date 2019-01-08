#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use crate::handler::Response;

use irc::client::prelude::*;

mod commands;
mod config;
mod database;
mod handler;
mod models;
mod schema;

fn main() {
    let db = database::Db::open("rustbot.sqlite").unwrap();
    let mut handler = handler::Handler::new(db);
    handler.register_default(commands::user_defined);
    handler.register("learn", commands::learn);
    handler.register("forget", commands::forget);

    let config = config::Config::load("irc.config.toml").unwrap();

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor
        .prepare_client_and_connect(&config.irc_config)
        .unwrap();
    client.identify().unwrap();
    reactor.register_client_with_handler(client, move |client, message| {
        handle_message(client, message, &config, &handler);
        Ok(())
    });

    reactor.run().unwrap();
}

fn handle_message(
    client: &IrcClient,
    message: Message,
    config: &config::Config,
    handler: &handler::Handler,
) {
    println!("{:?}", message);
    let (target, msg) = match message.command {
        Command::PRIVMSG(ref target, ref msg) => (target, msg),
        _ => return,
    };

    let user = message.source_nickname().unwrap();
    if config.bot_settings.blacklisted_users.contains(&user.into()) {
        return;
    }

    if let Some(command) = handler::Command::try_parse(user, msg) {
        let result = match handler.handle(command, config) {
            Ok(response) => response,
            Err(err) => {
                println!("{:?}", err);
                handler::Response::Say("unexpected error when executing command".into())
            }
        };

        let target = message.response_target().unwrap_or(target);
        match result {
            Response::Say(message) => client.send_privmsg(target, &message).unwrap(),
            Response::Act(message) => client.send_action(target, &message).unwrap(),
            Response::Notice(message) => client.send_notice(target, &message).unwrap(),
            Response::None => {}
        }
    }
}
