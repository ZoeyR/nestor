use crate::database::models::WinErrorVariant;
use crate::database::Db;

use anyhow::Result;
use nestor::command;
use nestor::handler::Command;
use nestor::request::State;

#[command("hresult")]
pub fn hresult(command: &Command, db: State<Db>) -> Result<String> {
    generic_error(command, WinErrorVariant::HResult, &db)
}

#[command("ntstatus")]
pub fn nt_status(command: &Command, db: State<Db>) -> Result<String> {
    generic_error(command, WinErrorVariant::NtStatus, &db)
}

#[command("win32")]
pub fn win32(command: &Command, db: State<Db>) -> Result<String> {
    generic_error(command, WinErrorVariant::Win32, &db)
}

fn generic_error(command: &Command, variant: WinErrorVariant, db: &Db) -> Result<String> {
    if command.arguments.len() != 1 {
        return Ok(format!(
            "Invalid command format, please use ~{} <code>. <code> can either be hex, decimal, or the symbol name.",
            command.command_str
        ));
    }

    let error = match slice_to(&command.arguments[0], 2) {
        "0x" => {
            if let Ok(code) = u32::from_str_radix(&command.arguments[0][2..], 16) {
                db.get_error_by_code(code, variant)?
            } else {
                return Ok("Invalid hex number.".to_string());
            }
        }
        _ => {
            if let Ok(code) = command.arguments[0].parse::<u32>() {
                db.get_error_by_code(code, variant)?
            } else {
                db.get_error_by_name(&command.arguments[0], variant)?
            }
        }
    };

    Ok(match error {
        Some(error) => format!(
            "[{:#X}] '{}' {}",
            error.code.parse::<u32>()?,
            error.name,
            error.description
        ),
        None => format!("Error '{}' not found.", command.arguments[0]),
    })
}

pub fn slice_to(slice: &str, mut n: usize) -> &str {
    if n >= slice.len() {
        return slice;
    }

    if !slice.is_char_boundary(n) && n > 0 {
        n -= 1;
    }

    &slice[..n]
}
