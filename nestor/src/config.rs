use std::fs;
use std::path::Path;

use anyhow::Result;
use irc::client::data::Config as IrcConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub irc_config: IrcConfig,
    #[serde(flatten)]
    pub bot_settings: NestorSettings,
}

#[derive(Deserialize)]
pub struct NestorSettings {
    pub blacklisted_users: Vec<String>,
    pub command_indicator: Vec<String>,
    pub alias_depth: u32,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        // Load entries via serde
        let conf = fs::read_to_string(path.as_ref())?;
        let conf: Config = toml::de::from_str(&conf)?;
        Ok(conf)
    }
}
