use failure::Error;
use irc::client::data::Config as IrcConfig;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "connection")]
    pub irc_config: IrcConfig,
    #[serde(rename = "bot")]
    pub bot_settings: BotSettings,
}

#[derive(Deserialize)]
pub struct BotSettings {
    pub admins: Vec<String>,
    pub blacklisted_users: Vec<String>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        // Load entries via serde
        let conf = fs::read_to_string(path.as_ref())?;
        let mut conf: Config = toml::de::from_str(&conf)?;
        Ok(conf)
    }
}
