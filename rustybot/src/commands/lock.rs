use crate::database::Db;

use failure::Error;
use nestor::command;
use nestor::config::{is_admin, Config};
use nestor::handler::Command;

#[command("lock")]
pub fn lock(command: &Command, config: &Config, db: &Db) -> Result<String, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok("Only an admin can lock a factoid".into());
    }

    if command.arguments.is_empty() {
        return Ok("Invalid command format, please use ~lock <factoid>".into());
    }

    let actual_factoid = command.arguments.join(" ");
    Ok(match db.get_factoid(&actual_factoid)? {
        Some(factoid) => {
            db.create_factoid(
                command.source_nick,
                factoid.intent,
                &factoid.label,
                &factoid.description,
                true,
            )?;
            format!("locked factoid '{}'", factoid.label)
        }
        None => format!(
            "cannot lock factoid '{}' because it doesn't exist",
            actual_factoid
        ),
    })
}

#[command("unlock")]
pub fn unlock(command: &Command, config: &Config, db: &Db) -> Result<String, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok("Only an admin can unlock a factoid".into());
    }

    if command.arguments.is_empty() {
        return Ok("Invalid command format, please use ~unlock <factoid>".into());
    }

    let actual_factoid = command.arguments.join(" ");
    Ok(match db.get_factoid(&actual_factoid)? {
        Some(factoid) => {
            db.create_factoid(
                command.source_nick,
                factoid.intent,
                &factoid.label,
                &factoid.description,
                false,
            )?;
            format!("unlocked factoid '{}'", factoid.label)
        }
        None => format!(
            "cannot unlock factoid '{}' because it doesn't exist",
            actual_factoid
        ),
    })
}
