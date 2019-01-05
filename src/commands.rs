use crate::config::Config;
use crate::database::Db;
use crate::handler::Command;
use failure::Error;

pub fn user_defined(command: Command, _config: &Config, db: &Db) -> Result<String, Error> {
    Ok(match db.get_factoid(command.command_str)? {
        Some(factoid) => factoid,
        None => format!("unknown factoid '{}'", command.command_str),
    })
}

pub fn learn(command: Command, config: &Config, db: &Db) -> Result<String, Error> {
    if !config
        .bot_settings
        .admins
        .contains(&command.source_nick.into())
    {
        return Ok(format!("{}", "Shoo! I'm testing this right now"));
    }

    Ok(match db.get_factoid(command.command_str)? {
        Some(_) => format!(
            "cannot rewrite '{}' since it already exists.",
            command.command_str
        ),
        None => {
            let parts: Vec<&str> = command.argument.splitn(2, "=").collect();
            if parts.len() < 2 {
                return Ok(format!(
                    "Invalid format for ~learn, format is: ~learn name=description"
                ));
            }

            let actual_factoid = format!("~{}", parts[0]);
            db.create_factoid(&actual_factoid, parts[1])?;
            format!("learned factoid: {}", actual_factoid)
        }
    })
}
