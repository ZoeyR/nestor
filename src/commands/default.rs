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
