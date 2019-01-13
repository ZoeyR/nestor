use crate::database::Db;

use failure::Error;
use irc_bot::config::Config;
use irc_bot::handler::{Command, Response};
use irc_bot_codegen::command;
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use tokio::await;

#[derive(Deserialize)]
struct CratesApi {
    #[serde(rename = "crate")]
    info: Crate,
}

#[derive(Deserialize)]
struct Crate {
    name: String,
    max_version: String,
    description: Option<String>,
    documentation: Option<String>,
}

#[command]
pub async fn crate_info<'a>(
    command: &'a Command<'a>,
    config: &'a Config,
) -> Result<Response, Error> {
    if command.arguments.len() != 1 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~crate <crate>".into(),
        ));
    }

    let client = Client::builder().build()?;
    let mut response = await!(client
        .get(&format!(
            "https://crates.io/api/v1/crates/{}",
            command.arguments[0]
        ))
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
            ),
        )
        .send())?;

    match response.status() {
        StatusCode::OK => {
            let api: CratesApi = await!(response.json())?;

            let crate_url = format!("https://crates.io/crates/{}", command.arguments[0]);

            if let Some(description) = api.info.description {
                Ok(Response::Notice(format!(
                    "{} ({}) - {} -> {} <{}>",
                    api.info.name,
                    api.info.max_version,
                    description,
                    crate_url,
                    api.info
                        .documentation
                        .unwrap_or(format!("https://docs.rs/{}", api.info.name))
                )))
            } else {
                Ok(Response::Notice(format!(
                    "{} ({}) -> {} <{}>",
                    api.info.name,
                    api.info.max_version,
                    crate_url,
                    api.info
                        .documentation
                        .unwrap_or(format!("https://docs.rs/{}", api.info.name))
                )))
            }
        }
        StatusCode::NOT_FOUND => Ok(Response::Notice(format!(
            "crate {} does not exist",
            command.arguments[0]
        ))),
        code => Ok(Response::Notice(format!(
            "crates.io returned error code: {}",
            code.as_u16()
        ))),
    }
}
