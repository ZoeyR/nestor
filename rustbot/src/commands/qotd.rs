use crate::database::Db;

use failure::Error;
use irc_bot::handler::Command;
use irc_bot::request::State;
use irc_bot_codegen::command;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[command("qotd")]
pub async fn qotd<'a>(command: &'a Command<'a>, db: State<'a, Db>) -> Result<String, Error> {
    let mut rng = thread_rng();

    Ok(match command.arguments.len() {
        0 => {
            if let Some(quote) = db.all_quotes()?.choose(&mut rng) {
                quote.quote.clone()
            } else {
                "I don't have any quotes :(".to_string()
            }
        }
        _ if command.arguments[0] == "add" => {
            let quote = command.arguments[1..].join(" ");
            db.create_quote(&quote)?;

            "Added new quote".to_string()
        }
        _ => "Invalid command format, please use `~qotd` or `~qotd add <quote>`".to_string(),
    })
}
