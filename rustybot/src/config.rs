use std::fs;
use std::path::Path;
use std::path::PathBuf;

use nestor::Error;
use serde::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "nestor.toml"
    )]
    pub config: PathBuf,
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "import")]
    Import {
        #[structopt(short = "f", long = "file", parse(from_os_str))]
        file: PathBuf,
        #[structopt(subcommand)]
        import_type: ImportType,
    },
    #[structopt(name = "export")]
    Export {
        #[structopt(short = "f", long = "file", parse(from_os_str))]
        file: PathBuf,
    },
    #[structopt(name = "launch")]
    Launch {},
}

#[derive(StructOpt)]
pub enum ImportType {
    #[structopt(name = "factoid")]
    Factoid,
    #[structopt(name = "hresult")]
    HResult,
    #[structopt(name = "ntresult")]
    NtResult,
    #[structopt(name = "win32")]
    Win32,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "connection")]
    pub nestor: nestor::config::Config,
    pub rustybot: RustybotSettings,
}

#[derive(Deserialize)]
pub struct RustybotSettings {
    pub admins: Vec<String>,
    pub database_url: String,
    pub contact: String,
    pub github_auth: GithubAuth,
}

#[derive(Deserialize)]
pub struct GithubAuth {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        // Load entries via serde
        let conf = fs::read_to_string(path.as_ref())?;
        let conf: Config = toml::de::from_str(&conf)?;
        Ok(conf)
    }
}

pub fn is_admin(nick: &str, config: &RustybotSettings) -> bool {
    config.admins.contains(&nick.into())
}
