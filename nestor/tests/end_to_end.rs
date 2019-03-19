#![feature(await_macro, async_await, futures_api)]

use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Read, Write};

use nestor::command;
use nestor::response::{Outcome, Response};
use nestor::Nestor;

#[command("speak")]
fn speak() -> Outcome {
    Outcome::Success(Response::Say("hello".into()))
}

#[command("act")]
fn act() -> Outcome {
    Outcome::Success(Response::Act("action".into()))
}

#[command("note")]
fn note() -> Outcome {
    Outcome::Success(Response::Notice("notice".into()))
}

#[test]
fn bot_say() {
    let mut stream = setup(44444);
    stream.write_all(b":Testhost PRIVMSG test :~speak\r\n").unwrap();

    let reader = BufReader::new(stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "PRIVMSG Testhost :hello");
}

#[test]
fn bot_act() {
    let mut stream = setup(44445);
    stream.write_all(b":Testhost PRIVMSG test :~act\r\n").unwrap();

    let reader = BufReader::new(stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "PRIVMSG Testhost :\u{1}ACTION action\u{1}");
}

#[test]
fn bot_notice() {
    let mut stream = setup(44446);
    stream.write_all(b":Testhost PRIVMSG test :~note\r\n").unwrap();

    let reader = BufReader::new(stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "NOTICE Testhost :notice");
}


fn setup(port: u16) -> TcpStream {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    thread::spawn(move || {
        let config_str = format!(
            r##"
                blacklisted_users = []
                command_indicator = ["~", "&&"]
                alias_depth = 2
                nickname = "test"
                server = "127.0.0.1"
                port = {}
            "##,
            port,
        );
        let config = toml::de::from_str(&config_str).unwrap();

        Nestor::with_config(config).activate();
        
    });
    
    listener.incoming().next().unwrap().unwrap()
}