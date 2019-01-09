use std::io::Write;
use std::str::FromStr;

use super::rustbot_model::RFactoid;
use super::schema::factoids;

use chrono::naive::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use failure::format_err;
use failure::Error;

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
}

impl<'a> NewFactoid<'a> {
    fn from_rfactoid(factoid: &'a RFactoid) -> Result<Self, Error> {
        Ok(NewFactoid {
            label: &factoid.key,
            intent: FactoidEnum::from_str(&factoid.val.intent)?,
            description: &factoid.val.message,
            nickname: &factoid.val.editor.nickname,
            timestamp: factoid.val.time,
        })
    }
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Text"]
pub enum FactoidEnum {
    Act,
    Say,
    Alias,
    Forget,
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
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "act" => Ok(FactoidEnum::Act),
            "say" => Ok(FactoidEnum::Say),
            "alias" => Ok(FactoidEnum::Alias),
            "forget" => Ok(FactoidEnum::Forget),
            _ => Err(format_err!("Unrecognized enum variant")),
        }
    }
}
