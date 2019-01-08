#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use crate::handler::{handle_message, Response};

use irc::client::prelude::*;

mod commands;
mod config;
mod database;
mod handler;

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
