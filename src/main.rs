#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use crate::config::{Args, Config};
use crate::handler::{handle_message, Response};

use irc::client::prelude::*;
use structopt::StructOpt;

mod commands;
mod config;
mod database;
mod handler;

fn main() {
    let args = Args::from_args();
    let config = config::Config::load(args.config).unwrap();

    let db = database::Db::open(&config.bot_settings.database_url).unwrap();
    let mut handler = handler::Handler::new(db);
    handler.register_default(commands::user_defined);
    handler.register("learn", commands::learn);
    handler.register("forget", commands::forget);

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
