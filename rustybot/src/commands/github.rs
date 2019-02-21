use crate::config::RustybotSettings;

use failure::Error;
use futures::compat::Future01CompatExt;
use nestor::command;
use nestor::config::Config;
use nestor::handler::Command;
use nestor::request::State;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::r#async::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use std::ops::Deref;

#[derive(Deserialize)]
struct PullRequest {
    pub number: u32,
    pub html_url: String,
    pub state: String,
    pub title: String,
    pub merged: bool,
}

#[command("rfc")]
pub async fn rfc<'a>(
    command: &'a Command<'a>,
    config: &'a Config,
    r_config: State<'a, RustybotSettings>,
) -> Result<String, Error> {
    let rfc = match command.arguments.get(0).map(|arg| arg.parse::<u32>()) {
        Some(Ok(rfc)) => rfc,
        Some(Err(_)) => return Ok("RFC must be a number.".into()),
        None => {
            return Ok("Invalid command format, please use ~rfc <number>.".into());
        }
    };

    let client = Client::builder().build()?;
    let mut response = await!(client
        .get(&format!(
            "https://api.github.com/repos/rust-lang/rfcs/pulls/{}",
            rfc
        ))
        .basic_auth(
            &r_config.github_auth.username,
            Some(&r_config.github_auth.password)
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
                r_config.contact
            )
        )
        .send()
        .compat())?;

    match response.status() {
        StatusCode::OK => {
            let pull_request: PullRequest = await!(response.json().compat())?;
            let state = if pull_request.merged {
                "merged"
            } else {
                &pull_request.state
            };

            Ok(format!(
                "[PR {}] <{}> {} <{}>",
                pull_request.number, state, pull_request.title, pull_request.html_url
            ))
        }
        StatusCode::NOT_FOUND => Ok(format!("RFC {} does not exist", command.arguments[0])),
        code => Ok(format!("github.com returned error code: {}", code.as_u16())),
    }
}
