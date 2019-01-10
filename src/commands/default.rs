use crate::config::Config;
use crate::database::models::{Factoid, FactoidEnum};
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;

pub fn user_defined(command: Command, _config: &Config, db: &Db) -> Result<Response, Error> {
    let num_args = command.arguments.len();

    let full_command = std::iter::once(command.command_str)
        .chain(command.arguments)
        .collect::<Vec<_>>();

    let (name, label) = if full_command.len() > 2 && full_command[full_command.len() - 2] == "@" {
        (
            Some(full_command[full_command.len() - 1]),
            full_command[0..full_command.len() - 2].join(" "),
        )
    } else {
        (None, full_command.join(" "))
    };

    println!("command is: '{}'", label);
    let response = match db.get_factoid(&label)? {
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
    };

    Ok(match (name, response) {
        (None, response) => response,
        (Some(_), Response::None) => Response::None,
        (Some(name), Response::Say(description)) => {
            Response::Say(format!("{}: {}", name, description))
        }
        (Some(name), Response::Act(description)) => {
            Response::Act(format!("{}: {}", name, description))
        }
        (Some(name), Response::Notice(description)) => {
            Response::Notice(format!("{}: {}", name, description))
        }
    })
}

fn process_alias(mut factoid: Factoid, db: &Db) -> Result<Response, Error> {
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
