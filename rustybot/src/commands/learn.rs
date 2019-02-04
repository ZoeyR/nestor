use std::marker::PhantomData;
use std::ops::Deref;

use crate::config::{is_admin, RustybotSettings};
use crate::database::models::{Factoid, FactoidEnum};
use crate::database::Db;

use failure::Error;
use nestor::command;
use nestor::handler::Command;
use nestor::request::State;

#[command("learn")]
pub fn learn(
    command: &Command,
    config: State<RustybotSettings>,
    db: State<Db>,
) -> Result<String, Error> {
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
            return Ok("Invalid command format, please use ~learn <factoid> = <description>".into());
        }
    };

    if command.arguments.len() < 3 || operation_index == command.arguments.len() - 1 {
        return Ok("Invalid command format, please use ~learn <factoid> = <description>".into());
    }

    let operation = command.arguments[operation_index].as_ref();
    let actual_factoid = command.arguments[0..operation_index].join(" ");
    let existing_factoid = db.get_factoid(&actual_factoid)?;
    let raw_description = command.arguments[operation_index + 1..].join(" ");

    if actual_factoid.contains('@') {
        return Ok("factoid keys cannot contain an @ symbol".into());
    }

    Ok(match operation {
        "=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Say,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&Factoid) -> String>,
            ),
        )?,
        ":=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Say,
            EditOptions::MustNot(
                || format!("{} is {}", actual_factoid, raw_description),
                PhantomData::<fn(&Factoid) -> String>,
            ),
        )?,
        "+=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Say,
            EditOptions::Must(
                |factoid: &Factoid| format!("{} {}", factoid.description, raw_description.trim()),
                PhantomData::<fn() -> String>,
            ),
        )?,
        "f=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Say,
            EditOptions::Optional(
                |_: &Factoid| raw_description.trim(),
                || raw_description.trim(),
            ),
        )?,
        "!=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Act,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&Factoid) -> String>,
            ),
        )?,
        "@=" => learn_helper(
            command.source_nick,
            &actual_factoid,
            &config,
            existing_factoid,
            &db,
            FactoidEnum::Alias,
            EditOptions::MustNot(
                || raw_description.trim(),
                PhantomData::<fn(&Factoid) -> String>,
            ),
        )?,
        format @ "~=" => format!("learn format {} is currently unimplemented", format),
        _ => "Invalid command format, please use ~learn <factoid> = <description>".into(),
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
    config: &RustybotSettings,
    mut factoid: Option<Factoid>,
    db: &Db,
    intent: FactoidEnum,
    editor: EditOptions<E, F>,
) -> Result<String, Error>
where
    E: for<'factoid> Fn(&'factoid Factoid) -> G,
    F: Fn() -> T,
    T: Deref<Target = str>,
    G: Deref<Target = str>,
{
    if let Some(FactoidEnum::Forget) = factoid.as_ref().map(|f| &f.intent) {
        factoid = None;
    }

    Ok(match (factoid, editor) {
        (None, EditOptions::MustNot(fresh, _)) | (None, EditOptions::Optional(_, fresh)) => {
            db.create_factoid(nick, intent, label, &fresh(), false)?;
            format!("learned factoid: '{}'.", label)
        }
        (None, EditOptions::Must(_, _)) => {
            format!("cannot edit: '{}'. Factoid does not exist.", label)
        }
        (Some(_), EditOptions::MustNot(_, _)) => {
            format!("cannot rewrite '{}' since it already exists.", label)
        }
        (Some(factoid), EditOptions::Must(editor, _))
        | (Some(factoid), EditOptions::Optional(editor, _)) => {
            if factoid.locked && !is_admin(nick, config) {
                format!("cannot edit factoid: '{}' because it is locked.", label)
            } else {
                let description = editor(&factoid);
                db.create_factoid(nick, factoid.intent, &label, &description, false)?;
                format!("edited factoid: '{}'.", label)
            }
        }
    })
}
