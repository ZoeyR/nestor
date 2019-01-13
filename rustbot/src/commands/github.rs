use crate::database::Db;
use irc_bot::config::Config;
use irc_bot::handler::{Command, Response};

use failure::Error;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::r#async::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use std::ops::Deref;
use tokio::await;

#[derive(Deserialize)]
struct PullRequest {
    pub number: u32,
    pub html_url: String,
    pub state: String,
    pub title: String,
    pub merged: bool,
}

pub async fn rfc<'a>(
    command: Command<'a>,
    config: &'a Config,
    _: &'a Db,
) -> Result<Response, Error> {
    let rfc = match command.arguments.get(0).map(|arg| arg.parse::<u32>()) {
        Some(Ok(rfc)) => rfc,
        Some(Err(_)) => return Ok(Response::Notice("RFC must be a number.".into())),
        None => {
            return Ok(Response::Notice(
                "Invalid command format, please use ~rfc <number>.".into(),
            ));
        }
    };

    let client = Client::builder().build()?;
    let mut response = await!(client
        .get(&format!(
            "https://api.github.com/repos/rust-lang/rfcs/pulls/{}",
            rfc
        ))
        .basic_auth(
            &config.bot_settings.github_auth.username,
            Some(&config.bot_settings.github_auth.password)
        )
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(
            USER_AGENT,
            format!(
                "{} ({})",
                config
                    .irc_config
                    .nickname
                    .as_ref()
                    .map(|s| s.deref())
                    .unwrap_or("rustybot"),
                config.bot_settings.contact
            )
        )
        .send())?;

    match response.status() {
        StatusCode::OK => {
            let pull_request: PullRequest = await!(response.json())?;
            let state = if pull_request.merged {
                "merged"
            } else {
                &pull_request.state
            };

            Ok(Response::Notice(format!(
                "[PR {}] <{}> {} <{}>",
                pull_request.number, state, pull_request.title, pull_request.html_url
            )))
        }
        StatusCode::NOT_FOUND => Ok(Response::Notice(format!(
            "RFC {} does not exist",
            command.arguments[0]
        ))),
        code => Ok(Response::Notice(format!(
            "github.com returned error code: {}",
            code.as_u16()
        ))),
    }
}
