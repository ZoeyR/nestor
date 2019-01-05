use crate::database::Db;
use crate::handler::Command;
use failure::Error;

pub fn user_defined(command: Command, db: &Db) -> String {
    db.get_factoid(command.command_str)
        .unwrap_or(format!("{}", "uhh"))
}

pub fn learn(_command: Command, db: &Db) -> String {
    format!("{}", "learn is not yet implemented")
}
