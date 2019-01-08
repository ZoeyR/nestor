use std::marker::PhantomData;
use std::ops::Deref;

use crate::config::is_admin;
use crate::config::Config;
use crate::database::Db;
use crate::handler::Command;
use crate::handler::Response;
use crate::models::FactoidEnum;

use failure::Error;

pub fn learn(command: Command, config: &Config, db: &Db) -> Result<Response, Error> {
    if !is_admin(command.source_nick, config) {
        return Ok(Response::Say("Shoo! I'm testing this right now".into()));
    }

    let operation_index = match command
        .arguments
        .iter()
        .enumerate()
        .find(|(_, arg)| {
            **arg == "="
                || **arg == ":="
                || **arg == "+="
                || **arg == "f="
                || **arg == "!="
                || **arg == "@="
                || **arg == "~="
        })
        .map(|(idx, _)| idx)
    {
        Some(index) => index,
        None => {
            return Ok(Response::Notice(
                "Invalid command format, please use ~learn <factoid> = <description>".into(),
            ));
        }
    };

    if command.arguments.len() < 3 || operation_index == command.arguments.len() - 1 {
        return Ok(Response::Notice(
            "Invalid command format, please use ~learn <factoid> = <description>".into(),
        ));
    }

    let operation = command.arguments[operation_index];
    let actual_factoid = command.arguments[0..operation_index].join(" ");
    let existing_factoid = db.get_factoid(&actual_factoid)?;
    let raw_description = command.arguments[operation_index + 1..].join(" ");

    Ok(match operation {
        "=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Say,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&crate::models::Factoid) -> String>,
            ),
        )?,
        ":=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Say,
            EditOptions::MustNot(
                || format!("{} is {}", actual_factoid, raw_description),
                PhantomData::<fn(&crate::models::Factoid) -> String>,
            ),
        )?,
        "+=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Say,
            EditOptions::Must(
                |factoid: &crate::models::Factoid| {
                    format!("{} {}", factoid.description, raw_description.trim())
                },
                PhantomData::<fn() -> String>,
            ),
        )?,
        "f=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Say,
            EditOptions::Optional(
                |_: &crate::models::Factoid| raw_description.trim(),
                || raw_description.trim(),
            ),
        )?,
        "!=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Act,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&crate::models::Factoid) -> String>,
            ),
        )?,
        "@=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            existing_factoid,
            db,
            FactoidEnum::Alias,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&crate::models::Factoid) -> String>,
            ),
        )?,
        format @ "~=" => Response::Notice(format!(
            "learn format {} is currently unimplemented",
            format
        )),
        _ => Response::Notice(
            "Invalid command format, please use ~learn <factoid> = <description>".into(),
        ),
    })
}

enum EditOptions<E, F> {
    Must(E, PhantomData<F>),
    MustNot(F, PhantomData<E>),
    Optional(E, F),
}

fn learn_helper<E, F, T, G>(
    nick: &str,
    label: &str,
    mut factoid: Option<crate::models::Factoid>,
    db: &Db,
    intent: FactoidEnum,
    editor: EditOptions<E, F>,
) -> Result<Response, Error>
where
    E: for<'factoid> Fn(&'factoid crate::models::Factoid) -> G,
    F: Fn() -> T,
    T: Deref<Target = str>,
    G: Deref<Target = str>,
{
    if let Some(FactoidEnum::Forget) = factoid.as_ref().map(|f| &f.intent) {
        factoid = None;
    }

    Ok(match (factoid, editor) {
        (None, EditOptions::MustNot(fresh, _)) | (None, EditOptions::Optional(_, fresh)) => {
            db.create_factoid(nick, intent, label, &fresh())?;
            Response::Notice(format!("learned factoid: '{}'.", label))
        }
        (None, EditOptions::Must(_, _)) => {
            Response::Notice(format!("cannot edit: '{}'. Factoid does not exist.", label))
        }
        (Some(_), EditOptions::MustNot(_, _)) => Response::Notice(format!(
            "cannot rewrite '{}' since it already exists.",
            label
        )),
        (Some(factoid), EditOptions::Must(editor, _))
        | (Some(factoid), EditOptions::Optional(editor, _)) => {
            let description = editor(&factoid);
            db.create_factoid(nick, factoid.intent, &label, &description)?;
            Response::Notice(format!("edited factoid: '{}'.", label))
        }
    })
}
