use crate::database::models::FactoidEnum;
use crate::database::Db;

use failure::Error;
use nestor::config::{is_admin, Config};
use nestor::handler::Command;
use nestor_codegen::command;

#[command("forget")]
pub async fn forget<'a>(
    command: &'a Command<'a>,
    config: &'a Config,
    db: &'a Db,
) -> Result<String, Error> {
    if command.arguments.is_empty() {
        return Ok("Invalid command format, please use ~forget <factoid>".into());
    }

    let actual_factoid = command.arguments.join(" ");
    Ok(match db.get_factoid(&actual_factoid)? {
        Some(ref factoid)
            if factoid.intent != FactoidEnum::Forget
                && (!factoid.locked || is_admin(command.source_nick, config)) =>
        {
            db.create_factoid(
                command.source_nick,
                FactoidEnum::Forget,
                &factoid.label,
                &factoid.description,
                false,
            )?;
            format!("forgot factoid '{}'", factoid.label)
        }
        Some(_) | None => format!(
            "cannot forget factoid '{}' because it doesn't exist",
            actual_factoid
        ),
    })
}
