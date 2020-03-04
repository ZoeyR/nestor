use std::io::Write;
use std::str::FromStr;

use super::import_models::RFactoid;
use super::schema::factoids;
use super::schema::qotd;
use super::schema::winerrors;

use anyhow::{anyhow, Error, Result};
use chrono::naive::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use nestor::response::Response;

#[derive(Queryable)]
pub struct Factoid {
    pub id: i32,
    pub label: String,
    pub intent: FactoidEnum,
    pub description: String,
    pub nickname: String,
    pub timestamp: NaiveDateTime,
    pub locked: bool,
}

#[derive(Insertable)]
#[table_name = "factoids"]
pub struct NewFactoid<'a> {
    pub label: &'a str,
    pub intent: FactoidEnum,
    pub description: &'a str,
    pub nickname: &'a str,
    pub timestamp: NaiveDateTime,
    pub locked: bool,
}

#[derive(Queryable)]
pub struct Quote {
    pub id: i32,
    pub quote: String,
}

#[derive(Insertable)]
#[table_name = "qotd"]
pub struct NewQuote<'a> {
    pub quote: &'a str,
}

#[derive(Queryable)]
pub struct WinError {
    pub id: i32,
    pub code: String,
    pub error_type: WinErrorVariant,
    pub name: String,
    pub description: String,
}

#[derive(Insertable)]
#[table_name = "winerrors"]
pub struct NewWinError<'a> {
    pub code: &'a str,
    pub error_type: WinErrorVariant,
    pub name: &'a str,
    pub description: &'a str,
}

impl<'a> NewFactoid<'a> {
    pub fn from_rfactoid(factoid: &'a RFactoid) -> Result<Self> {
        let intent = match &factoid.val.intent {
            Some(intent) => FactoidEnum::from_str(intent)?,
            None => FactoidEnum::Forget,
        };

        Ok(NewFactoid {
            label: &factoid.key,
            intent,
            description: &factoid.val.message,
            nickname: &factoid.val.editor.nickname,
            timestamp: NaiveDateTime::parse_from_str(&factoid.val.time, "%+")?,
            locked: factoid.val.frozen,
        })
    }
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "Text"]
pub enum FactoidEnum {
    Act,
    Say,
    Alias,
    Forget,
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "Text"]
pub enum WinErrorVariant {
    HResult,
    NtStatus,
    Win32,
}

impl FactoidEnum {
    pub fn to_response(&self, description: String) -> Response {
        match self {
            FactoidEnum::Act => Response::Act(description),
            FactoidEnum::Say => Response::Say(description),
            _ => Response::None,
        }
    }
}

impl ToSql<Text, Sqlite> for FactoidEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let write = match *self {
            FactoidEnum::Act => "act",
            FactoidEnum::Say => "say",
            FactoidEnum::Alias => "alias",
            FactoidEnum::Forget => "forget",
        };

        <str as ToSql<Text, Sqlite>>::to_sql(write, out)
    }
}

impl FromSql<Text, Sqlite> for FactoidEnum {
    fn from_sql(value: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let text = <String as FromSql<Text, Sqlite>>::from_sql(value)?;
        match text.as_ref() {
            "act" => Ok(FactoidEnum::Act),
            "say" => Ok(FactoidEnum::Say),
            "alias" => Ok(FactoidEnum::Alias),
            "forget" => Ok(FactoidEnum::Forget),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToString for FactoidEnum {
    fn to_string(&self) -> String {
        match self {
            FactoidEnum::Act => "act",
            FactoidEnum::Say => "say",
            FactoidEnum::Alias => "alias",
            FactoidEnum::Forget => "forget",
        }
        .into()
    }
}

impl FromStr for FactoidEnum {
    type Err = Error;
    fn from_str(val: &str) -> Result<Self> {
        match val {
            "act" => Ok(FactoidEnum::Act),
            "say" => Ok(FactoidEnum::Say),
            "alias" => Ok(FactoidEnum::Alias),
            "forget" => Ok(FactoidEnum::Forget),
            _ => Err(anyhow!("Unrecognized enum variant")),
        }
    }
}

impl ToSql<Text, Sqlite> for WinErrorVariant {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let write = match *self {
            WinErrorVariant::HResult => "hresult",
            WinErrorVariant::NtStatus => "ntstatus",
            WinErrorVariant::Win32 => "win32",
        };

        <str as ToSql<Text, Sqlite>>::to_sql(write, out)
    }
}

impl FromSql<Text, Sqlite> for WinErrorVariant {
    fn from_sql(value: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let text = <String as FromSql<Text, Sqlite>>::from_sql(value)?;
        match text.as_ref() {
            "hresult" => Ok(WinErrorVariant::HResult),
            "ntstatus" => Ok(WinErrorVariant::NtStatus),
            "win32" => Ok(WinErrorVariant::Win32),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToString for WinErrorVariant {
    fn to_string(&self) -> String {
        match self {
            WinErrorVariant::HResult => "hresult",
            WinErrorVariant::NtStatus => "ntstatus",
            WinErrorVariant::Win32 => "win32",
        }
        .into()
    }
}

impl FromStr for WinErrorVariant {
    type Err = Error;
    fn from_str(val: &str) -> Result<Self> {
        match val {
            "hresult" => Ok(WinErrorVariant::HResult),
            "ntstatus" => Ok(WinErrorVariant::NtStatus),
            "win32" => Ok(WinErrorVariant::Win32),
            _ => Err(anyhow!("Unrecognized enum variant")),
        }
    }
}
