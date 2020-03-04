use crate::config::{is_admin, RustybotSettings};
use crate::database::models::FactoidEnum;
use crate::database::Db;

use anyhow::Result;
use nestor::command;
use nestor::handler::Command;
use nestor::request::State;

#[command("forget")]
pub fn forget(command: &Command, config: State<RustybotSettings>, db: State<Db>) -> Result<String> {
    if command.arguments.is_empty() {
        return Ok("Invalid command format, please use ~forget <factoid>".into());
    }

    let actual_factoid = command.arguments.join(" ");
    Ok(match db.get_factoid(&actual_factoid)? {
        Some(ref factoid)
            if factoid.intent != FactoidEnum::Forget
                && (!factoid.locked || is_admin(command.source_nick, &config)) =>
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
