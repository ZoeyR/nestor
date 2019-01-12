use crate::config::Config;
use crate::database::models::WinErrorVariant;
use crate::database::Db;
use crate::handler::{Command, Response};

use failure::Error;

pub async fn hresult<'a>(
    command: Command<'a>,
    _config: &'a Config,
    db: &'a Db,
) -> Result<Response, Error> {
    await!(generic_error(command, WinErrorVariant::HResult, db))
}

pub async fn nt_status<'a>(
    command: Command<'a>,
    _config: &'a Config,
    db: &'a Db,
) -> Result<Response, Error> {
    await!(generic_error(command, WinErrorVariant::NtStatus, db))
}

pub async fn win32<'a>(
    command: Command<'a>,
    _config: &'a Config,
    db: &'a Db,
) -> Result<Response, Error> {
    await!(generic_error(command, WinErrorVariant::Win32, db))
}

async fn generic_error<'a>(
    command: Command<'a>,
    variant: WinErrorVariant,
    db: &'a Db,
) -> Result<Response, Error> {
    if command.arguments.len() != 1 {
        return Ok(Response::Notice(format!(
            "Invalid command format, please use ~{} <code>. <code> can either be hex, decimal, or the symbol name.",
            command.command_str
        )));
    }

    let error = match slice_to(command.arguments[0], 2) {
        "0x" => {
            if let Ok(code) = u32::from_str_radix(&command.arguments[0][2..], 16) {
                db.get_error_by_code(code, variant)?
            } else {
                return Ok(Response::Notice("Invalid hex number.".to_string()));
            }
        }
        _ => {
            if let Ok(code) = command.arguments[0].parse::<u32>() {
                db.get_error_by_code(code, variant)?
            } else {
                db.get_error_by_name(command.arguments[0], variant)?
            }
        }
    };

    Ok(match error {
        Some(error) => Response::Notice(format!(
            "[{:#X}] '{}' {}",
            error.code.parse::<u32>()?,
            error.name,
            error.description
        )),
        None => Response::Notice(format!("Error '{}' not found.", command.arguments[0])),
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
