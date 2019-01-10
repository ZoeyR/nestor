use crate::config::{Config, is_admin};
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;

pub fn lock(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(Response::Notice("Only an admin can lock a factoid".into()));
    }

    if command.arguments.is_empty() {
        return Ok(Response::Notice(
            "Invalid command format, please use ~lock <factoid>".into(),
        ));
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
            Response::Notice(format!("locked factoid '{}'", factoid.label))
        }
        None => Response::Notice(format!(
            "cannot lock factoid '{}' because it doesn't exist",
            actual_factoid
        )),
    })
}

pub fn unlock(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(Response::Notice(
            "Only an admin can unlock a factoid".into(),
        ));
    }

    if command.arguments.is_empty() {
        return Ok(Response::Notice(
            "Invalid command format, please use ~unlock <factoid>".into(),
        ));
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
            Response::Notice(format!("unlocked factoid '{}'", factoid.label))
        }
        None => Response::Notice(format!(
            "cannot unlock factoid '{}' because it doesn't exist",
            actual_factoid
        )),
    })
}
