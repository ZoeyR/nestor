use crate::config::{is_admin, Config};
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;

pub fn rustc_error(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if command.arguments.len() != 1 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~error <factoid>".into(),
        ));
    }

    Ok(match command.arguments[0].parse::<u32>() {
        Ok(num) if num >= 0 && num <= 9999 => Response::Notice(format!(
            "https://doc.rust-lang.org/error-index.html#E{:04}",
            num
        )),
        _ => Response::Notice("Error code must be between 0000 and 9999.".to_string()),
    })
}
