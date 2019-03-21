#![feature(await_macro, async_await, futures_api)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use nestor::command;
use nestor::response::{Outcome, Response};
use nestor::Nestor;

#[command("die")]
fn die() {
    panic!()
}

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

#[command("forward")]
fn forward() -> Outcome {
    Outcome::Forward("forward-to".into())
}

#[command("forward-to")]
fn forward_to() -> Outcome {
    Outcome::Success(Response::Say("forwarded".into()))
}

#[command("forward-loop")]
fn forward_loop() -> Outcome {
    Outcome::Forward("forward-loop".into())
}

#[test]
fn bot_say() {
    let mut stream = setup(44444);
    stream
        .write_all(b":Testhost PRIVMSG test :~speak\r\n")
        .unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "PRIVMSG Testhost :hello");

    stream
        .write_all(b":Testhost PRIVMSG test :~die\r\n")
        .unwrap();
}

#[test]
fn bot_act() {
    let mut stream = setup(44445);
    stream
        .write_all(b":Testhost PRIVMSG test :~act\r\n")
        .unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "PRIVMSG Testhost :\u{1}ACTION action\u{1}");

    stream
        .write_all(b":Testhost PRIVMSG test :~die\r\n")
        .unwrap();
}

#[test]
fn bot_notice() {
    let mut stream = setup(44446);
    stream
        .write_all(b":Testhost PRIVMSG test :~note\r\n")
        .unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "NOTICE Testhost :notice");

    stream
        .write_all(b":Testhost PRIVMSG test :~die\r\n")
        .unwrap();
}

#[test]
fn bot_forward() {
    let mut stream = setup(44447);
    stream
        .write_all(b":Testhost PRIVMSG test :~forward\r\n")
        .unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "PRIVMSG Testhost :forwarded");

    stream
        .write_all(b":Testhost PRIVMSG test :~die\r\n")
        .unwrap();
}

#[test]
fn bot_forward_loop() {
    let mut stream = setup(44449);
    stream
        .write_all(b":Testhost PRIVMSG test :~forward-loop\r\n")
        .unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().skip(3).next().unwrap().unwrap();

    assert_eq!(line, "NOTICE Testhost :alias depth too deep");

    stream
        .write_all(b":Testhost PRIVMSG test :~die\r\n")
        .unwrap();
}

fn setup(port: u16) -> TcpStream {
    std::panic::set_hook(Box::new(|_| {}));

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
        let _ = std::panic::catch_unwind(|| {
            Nestor::with_config(config).activate();
        });
    });

    listener.incoming().next().unwrap().unwrap()
}
