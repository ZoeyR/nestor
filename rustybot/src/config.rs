use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "irc.config.toml"
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
