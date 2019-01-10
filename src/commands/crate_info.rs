use crate::config::Config;
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use serde::Deserialize;
use std::ops::Deref;
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
    description: String,
    documentation: String,
}

pub async fn crate_info<'a>(
    command: Command<'a>,
    config: &'a Config,
    _: &'a Db,
) -> Result<Response, Error> {
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

    let api: CratesApi = await!(response.json())?;

    let crate_url = format!("https://crates.io/crates/{}", command.arguments[0]);
    Ok(Response::Notice(format!(
        "{} ({}) - {} -> {} <{}>",
        api.info.name,
        api.info.max_version,
        api.info.description.replace("\n", "").replace("\r", ""),
        crate_url,
        api.info.documentation
    )))
}
