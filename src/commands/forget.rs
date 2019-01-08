use crate::config::is_admin;
use crate::config::Config;
use crate::database::models::FactoidEnum;
use crate::database::Db;
use crate::handler::Command;
use crate::handler::Response;

use failure::Error;

pub fn forget(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(Response::Say("Shoo! I'm testing this right now".into()));
    }

    if command.arguments.len() != 1 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~forget <factoid>".into(),
        ));
    }

    let actual_factoid = command.arguments[0];
    Ok(match db.get_factoid(&actual_factoid)? {
        Some(ref factoid) if factoid.intent != FactoidEnum::Forget => {
            db.create_factoid(
                command.source_nick,
                FactoidEnum::Forget,
                &factoid.label,
                &factoid.description,
            )?;
            Response::Notice(format!("forgot factoid '{}'", factoid.label))
        }
        Some(_) | None => Response::Notice(format!(
            "cannot forget factoid '{}' because it doesn't exist",
            actual_factoid
        )),
    })
}
