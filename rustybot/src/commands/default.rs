use crate::database::models::FactoidEnum;
use crate::database::Db;

use nestor::command;
use nestor::handler::Command;
use nestor::request::State;
use nestor::response::{Outcome, Response};

#[command]
pub fn user_defined(command: &Command, db: State<Db>) -> Outcome {
    let num_args = command.arguments.len();

    let full_command: Vec<_> = std::iter::once(&command.command_str)
        .chain(command.arguments.as_slice())
        .map(|s| s.as_ref())
        .collect();

    let (name, label) = if full_command.len() > 2 && full_command[full_command.len() - 2] == "@" {
        (
            Some(full_command[full_command.len() - 1]),
            full_command[0..full_command.len() - 2].join(" "),
        )
    } else {
        (None, full_command.join(" "))
    };

    println!("command is: '{}'", label);
    let response = match db.get_factoid(&label) {
        Ok(Some(factoid)) => match factoid.intent {
            FactoidEnum::Forget => {
                Response::Notice(format!("unknown factoid '{}'", command.command_str))
            }
            FactoidEnum::Alias => {
                if let Some(name) = name {
                    return Outcome::Forward(format!("{} @ {}", factoid.description, name));
                } else {
                    return Outcome::Forward(factoid.description);
                }
            }
            _ => factoid.intent.to_response(factoid.description),
        },
        Ok(None) if num_args == 0 => {
            Response::Notice(format!("unknown factoid '{}'", command.command_str))
        }
        Ok(None) => Response::None,
        Err(err) => return Outcome::Failure(err.into()),
    };

    Outcome::Success(match (name, response) {
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
