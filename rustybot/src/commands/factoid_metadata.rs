use crate::database::Db;

use anyhow::Result;
use nestor::command;
use nestor::handler::Command;
use nestor::request::State;

#[command("factoid-metadata")]
fn metadata(command: &Command, db: State<Db>) -> Result<String> {
    let factoid = command.arguments.join(" ");

    Ok(match db.get_factoid(&factoid)? {
        Some(factoid) => format!(
            "Factoid metadata for '{}': Intent={:?}; Locked={}; LastEdit={} on {}",
            factoid.label, factoid.intent, factoid.locked, factoid.nickname, factoid.timestamp
        ),
        None => format!("Factoid '{}' does not exist", factoid),
    })
}
