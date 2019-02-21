use std::ops::Deref;

use crate::config::RustybotSettings;

use failure::Error;
use futures::compat::Future01CompatExt;
use nestor::command;
use nestor::config::Config as NestorConfig;
use nestor::handler::Command;
use nestor::request::State;
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use reqwest::StatusCode;
use serde::Deserialize;

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

#[command("crate")]
pub async fn crate_info<'a>(
    command: &'a Command<'a>,
    nestor_config: &'a NestorConfig,
    r_config: State<'a, RustybotSettings>,
) -> Result<String, Error> {
    if command.arguments.len() != 1 {
        return Ok("Invalid command format, please use ~crate <crate>".into());
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
                nestor_config
                    .irc_config
                    .nickname
                    .as_ref()
                    .map(|s| s.deref())
                    .unwrap_or("rustybot"),
                r_config.contact
            ),
        )
        .send()
        .compat())?;

    match response.status() {
        StatusCode::OK => {
            let api: CratesApi = await!(response.json().compat())?;

            let crate_url = format!("https://crates.io/crates/{}", command.arguments[0]);

            if let Some(description) = api.info.description {
                Ok(format!(
                    "{} ({}) - {} -> {} <{}>",
                    api.info.name,
                    api.info.max_version,
                    description,
                    crate_url,
                    api.info
                        .documentation
                        .unwrap_or(format!("https://docs.rs/{}", api.info.name))
                ))
            } else {
                Ok(format!(
                    "{} ({}) -> {} <{}>",
                    api.info.name,
                    api.info.max_version,
                    crate_url,
                    api.info
                        .documentation
                        .unwrap_or(format!("https://docs.rs/{}", api.info.name))
                ))
            }
        }
        StatusCode::NOT_FOUND => Ok(format!("crate {} does not exist", command.arguments[0])),
        code => Ok(format!("crates.io returned error code: {}", code.as_u16())),
    }
}
