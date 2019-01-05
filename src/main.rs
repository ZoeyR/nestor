#[macro_use]
extern crate diesel;

use irc::client::prelude::*;

mod commands;
mod database;
mod handler;
mod models;
mod schema;

fn main() {
    let db = database::Db::open("rustbot.sqlite").unwrap();
    let mut handler = handler::Handler::new(db);
    handler.register_default(commands::user_defined);
    handler.register("learn", commands::learn);

    let config = Config {
        nickname: Some("dgriffen_bot".into()),
        server: Some("irc.mozilla.org".into()),
        port: Some(6667),
        channels: Some(vec!["#rust-offtopic".into()]),
        ..Config::default()
    };
    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();
    reactor.register_client_with_handler(client, move |client, message| {
        handle_message(client, message, &handler);
        Ok(())
    });

    reactor.run().unwrap();
}

fn handle_message(client: &IrcClient, message: Message, handler: &handler::Handler) {
    println!("{:?}", message);
    let (target, msg) = match message.command {
        Command::PRIVMSG(ref target, ref msg) => (target, msg),
        _ => return,
    };

    let user = message.source_nickname().unwrap();
    if let Some(command) = handler::Command::try_parse(msg) {
        let result = handler.handle(command);
        let target = message.response_target().unwrap_or(target);
        client.send_notice(target, &result).unwrap();
    }
}
