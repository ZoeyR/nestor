#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use std::fs::File;
use std::io::BufRead;
use std::io::{BufReader, Write};

use crate::config::{Args, Command, Config};
use crate::database::rustbot_model::RFactoid;
use crate::handler::{handle_message, Response};

use chrono::{offset, DateTime, NaiveDateTime, Utc};
use irc::client::ext::ClientExt;
use irc::client::reactor::IrcReactor;
use structopt::StructOpt;

mod commands;
mod config;
mod database;
mod handler;

fn main() {
    let args = Args::from_args();
    let config = config::Config::load(args.config).unwrap();
    let db = database::Db::open(&config.bot_settings.database_url).unwrap();

    match args.command {
        Command::Export { file } => {
            let mut output = File::create(file).unwrap();
            let factoids = db.all_factoids().unwrap();
            for factoid in factoids {
                let factoid_json = serde_json::to_string(&RFactoid::from(factoid)).unwrap();
                writeln!(output, "{}", factoid_json).unwrap();
            }
        }
        Command::Import { file } => {
            let input = File::open(file).unwrap();
            let input = BufReader::new(input);

            for line in input.lines() {
                let line = line.unwrap();
                let rfactoid: RFactoid = match serde_json::from_str(&line) {
                    Ok(rfactoid) => rfactoid,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                db.create_from_rfactoid(&rfactoid).unwrap();
            }
        }
        Command::Launch {} => {
            let mut handler = handler::Handler::new(db);
            handler.register_default(commands::user_defined);
            handler.register("learn", commands::learn);
            handler.register("forget", commands::forget);
            handler.register("lock", commands::lock);
            handler.register("unlock", commands::unlock);

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
    }
}
