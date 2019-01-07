use crate::config::is_admin;
use crate::config::Config;
use crate::database::Db;
use crate::handler::Command;
use crate::handler::Response;
use crate::models::FactoidEnum;

use failure::Error;

pub fn user_defined(command: Command, _config: &Config, db: &Db) -> Result<Response, Error> {
    let num_args = command.arguments.len();
    let full_command = std::iter::once(command.command_str)
        .chain(command.arguments)
        .collect::<Vec<_>>()
        .join(" ");
    println!("command is: '{}'", full_command);
    Ok(match db.get_factoid(&full_command)? {
        Some(factoid) => match factoid.intent {
            FactoidEnum::Forget => {
                Response::Notice(format!("unknown factoid '{}'", command.command_str))
            }
            FactoidEnum::Alias => process_alias(factoid, db)?,
            _ => Response::from_intent(factoid.intent, factoid.description),
        },
        None if num_args == 0 => {
            Response::Notice(format!("unknown factoid '{}'", command.command_str))
        }
        None => Response::None,
    })
}

fn process_alias(mut factoid: crate::models::Factoid, db: &Db) -> Result<Response, Error> {
    for _ in 0..3 {
        match factoid.intent {
            FactoidEnum::Alias => match db.get_factoid(&factoid.description)? {
                Some(next_level) => factoid = next_level,
                None => {
                    return Ok(Response::Notice(format!(
                        "unknown factoid alias '{}'",
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

    let operation_index = match command
        .arguments
        .iter()
        .enumerate()
        .find(|(_, arg)| {
            **arg == "="
                || **arg == ":="
                || **arg == "+="
                || **arg == "f="
                || **arg == "!="
                || **arg == "@="
                || **arg == "~="
        })
        .map(|(idx, _)| idx)
    {
        Some(index) => index,
        None => {
            return Ok(Response::Notice(
                "Invalid command format, please use ~learn <factoid> = <description>".into(),
            ));
        }
    };

    if command.arguments.len() < 3 || operation_index == command.arguments.len() - 1 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~learn <factoid> = <description>".into(),
        ));
    }

    let operation = command.arguments[operation_index];
    let actual_factoid = command.arguments[0..operation_index].join(" ");
    let existing_factoid = db.get_factoid(&actual_factoid)?;
    let raw_description = command.arguments[operation_index + 1..].join(" ");

    Ok(match operation {
        "=" => match existing_factoid {
            Some(ref factoid) if factoid.intent != FactoidEnum::Forget => {
                Response::Notice(format!(
                    "cannot rewrite '{}' since it already exists.",
                    actual_factoid
                ))
            }
            Some(_) | None => {
                let description = raw_description.trim();
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
                let description = format!("{} is {}", actual_factoid, raw_description);
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
                    raw_description.trim()
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
            let description = raw_description.trim();
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
                let description = raw_description.trim();
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
                let description = raw_description.trim();
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
