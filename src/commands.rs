use crate::config::is_admin;
use crate::config::Config;
use crate::database::Db;
use crate::handler::Command;
use failure::Error;

pub fn user_defined(command: Command, _config: &Config, db: &Db) -> Result<Option<String>, Error> {
    if command.arguments.len() > 1 {
        return Ok(None);
    }

    Ok(Some(match db.get_factoid(command.command_str)? {
        Some(factoid) => factoid,
        None => format!("unknown factoid '{}'", command.command_str),
    }))
}

pub fn learn(command: Command, config: &Config, db: &Db) -> Result<String, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(format!("{}", "Shoo! I'm testing this right now"));
    }

    if command.arguments.len() < 3 {
        return Ok(format!(
            "{}",
            "Invalid command format, please use ~learn <factoid> = <description>"
        ));
    }

    let actual_factoid = format!("~{}", command.arguments[0]);
    let existing_factoid = db.get_factoid(&actual_factoid)?;

    Ok(match command.arguments[1] {
        "=" => {
            if existing_factoid.is_some() {
                format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                )
            } else {
                let description = command.arguments[2..].join(" ");
                let description = description.trim();
                db.create_factoid(&actual_factoid, description)?;
                format!("learned factoid: {}", actual_factoid)
            }
        }
        ":=" => {
            if existing_factoid.is_some() {
                format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                )
            } else {
                let description = format!(
                    "{} is {}",
                    command.arguments[0],
                    command.arguments[2..].join(" ").trim()
                );
                db.create_factoid(&actual_factoid, &description)?;
                format!("learned factoid: {}", actual_factoid)
            }
        }
        "+=" => {
            if let Some(existing_factoid) = existing_factoid {
                let description = format!(
                    "{} {}",
                    existing_factoid,
                    command.arguments[2..].join(" ").trim()
                );
                db.update_factoid_description(&actual_factoid, &description)?;
                format!("learned factoid: {}", actual_factoid)
            } else {
                format!("cannot ammend '{}' since it doesn't exist.", actual_factoid)
            }
        }
        "f=" => {
            let description = command.arguments[2..].join(" ");
            let description = description.trim();
            if let Some(existing_factoid) = existing_factoid {
                db.update_factoid_description(&actual_factoid, &description)?;
                format!("rewrote factoid: {}", actual_factoid)
            } else {
                db.create_factoid(&actual_factoid, &description)?;
                format!("learned factoid: {}", actual_factoid)
            }
        }
        "~=" | "@=" | "!=" | _ => format!("{}", "learn format is currently unimplemented"),
    })
}
