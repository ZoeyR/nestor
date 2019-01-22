use failure::Error;
use irc_bot::handler::Command;
use irc_bot_codegen::command;

#[command("error")]
pub async fn rustc_error<'a>(command: &'a Command<'a>) -> Result<String, Error> {
    if command.arguments.len() != 1 {
        return Ok("Invalid command format, please use ~error <error code>".into());
    }

    Ok(match command.arguments[0].parse::<u32>() {
        Ok(num) if num <= 9999 => format!("https://doc.rust-lang.org/error-index.html#E{:04}", num),
        _ => "Error code must be between 0000 and 9999.".to_string(),
    })
}
