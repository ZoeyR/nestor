use crate::config::is_admin;
use crate::config::Config;
use crate::database::Db;
use crate::handler::Command;
use crate::handler::Response;
use crate::models::FactoidEnum;

use failure::Error;

pub fn user_defined(command: Command, _config: &Config, db: &Db) -> Result<Response, Error> {
    if command.arguments.len() > 1 {
        return Ok(Response::None);
    }

    Ok(match db.get_factoid(command.command_str)? {
        Some(factoid) => match factoid.intent {
            FactoidEnum::Forget => {
                Response::Notice(format!("unknown factoid '{}'", command.command_str))
            }
            FactoidEnum::Alias => process_alias(factoid, db)?,
            _ => Response::from_intent(factoid.intent, factoid.description),
        },
        None => Response::Notice(format!("unknown factoid '{}'", command.command_str)),
    })
}

fn process_alias(mut factoid: crate::models::Factoid, db: &Db) -> Result<Response, Error> {
    for _ in 0..3 {
        match factoid.intent {
            FactoidEnum::Alias => match db.get_factoid(&factoid.description)? {
                Some(next_level) => factoid = next_level,
                None => {
                    return Ok(Response::Notice(format!(
                        "unknown factoid '{}'",
                        factoid.description
                    )))
                }
            },
            _ => return Ok(Response::from_intent(factoid.intent, factoid.description)),
        }
    }

    Ok(Response::Notice("alias depth too deep".into()))
}

pub fn learn(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(Response::Say("Shoo! I'm testing this right now".into()));
    }

    if command.arguments.len() < 3 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~learn <factoid> = <description>".into(),
        ));
    }

    let actual_factoid = command.arguments[0];
    let existing_factoid = db.get_factoid(&actual_factoid)?;

    Ok(match command.arguments[1] {
        "=" => match existing_factoid {
            Some(ref factoid) if factoid.intent != FactoidEnum::Forget => {
                Response::Notice(format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                ))
            }
            Some(_) | None => {
                let description = command.arguments[2..].join(" ");
                let description = description.trim();
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Say,
                    &actual_factoid,
                    description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            }
        },
        ":=" => {
            if existing_factoid.is_some() {
                Response::Notice(format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                ))
            } else {
                let description = format!(
                    "{} is {}",
                    command.arguments[0],
                    command.arguments[2..].join(" ").trim()
                );
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Say,
                    &actual_factoid,
                    &description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            }
        }
        "+=" => {
            if let Some(existing_factoid) = existing_factoid {
                let description = format!(
                    "{} {}",
                    existing_factoid.description,
                    command.arguments[2..].join(" ").trim()
                );
                db.create_factoid(
                    command.source_nick,
                    existing_factoid.intent,
                    &actual_factoid,
                    &description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            } else {
                Response::Notice(format!(
                    "cannot ammend '{}' since it doesn't exist.",
                    actual_factoid
                ))
            }
        }
        "f=" => {
            let description = command.arguments[2..].join(" ");
            let description = description.trim();
            if let Some(_) = existing_factoid {
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Say,
                    &actual_factoid,
                    &description,
                )?;
                Response::Notice(format!("rewrote factoid: {}", actual_factoid))
            } else {
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Say,
                    &actual_factoid,
                    &description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            }
        }
        "!=" => match existing_factoid {
            Some(ref factoid) if factoid.intent != FactoidEnum::Forget => {
                Response::Notice(format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                ))
            }
            Some(_) | None => {
                let description = command.arguments[2..].join(" ");
                let description = description.trim();
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Act,
                    &actual_factoid,
                    description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            }
        },
        "@=" => match existing_factoid {
            Some(ref factoid) if factoid.intent != FactoidEnum::Forget => {
                Response::Notice(format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                ))
            }
            Some(_) | None => {
                let description = command.arguments[2..].join(" ");
                let description = description.trim();
                db.create_factoid(
                    command.source_nick,
                    FactoidEnum::Alias,
                    &actual_factoid,
                    description,
                )?;
                Response::Notice(format!("learned factoid: {}", actual_factoid))
            }
        },
        format @ "~=" => Response::Notice(format!(
            "learn format {} is currently unimplemented",
            format
        )),
        _ => Response::Notice(
            "Invalid command format, please use ~learn <factoid> = <description>".into(),
        ),
    })
}

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
