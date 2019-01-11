use crate::config::{is_admin, Config};
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub async fn qotd<'a>(
    command: Command<'a>,
    _config: &'a Config,
    db: &'a Db,
) -> Result<Response, Error> {
    let mut rng = thread_rng();

    Ok(match command.arguments.len() {
        0 => {
            if let Some(quote) = db.all_quotes()?.choose(&mut rng) {
                Response::Notice(quote.quote.clone())
            } else {
                Response::Notice("I don't have any quotes :(".to_string())
            }
        }
        _ if command.arguments[0] == "add" => {
            let quote = command.arguments[1..].join(" ");
            db.create_quote(&quote)?;

            Response::Notice("Added new quote".to_string())
        }
        _ => Response::Notice(
            "Invalid command format, please use `~qotd` or `~qotd add <quote>`".to_string(),
        ),
    })
}
