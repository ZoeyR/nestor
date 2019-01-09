use self::models::{Factoid, FactoidEnum, NewFactoid};
use self::rustbot_model::RFactoid;

use chrono::offset::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use failure::Error;

pub mod models;
pub mod rustbot_model;
pub mod schema;

pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, Error> {
        let connection = SqliteConnection::establish(path)?;

        Ok(Db { connection })
    }

    pub fn get_factoid(&self, key: &str) -> Result<Option<Factoid>, Error> {
        use self::schema::factoids::dsl::*;

        factoids
            .filter(label.eq(key))
            .order(timestamp.desc())
            .first::<Factoid>(&self.connection)
            .optional()
            .map_err(From::from)
    }

    pub fn all_factoids(&self) -> Result<Vec<Factoid>, Error> {
        use self::schema::factoids;

        Ok(factoids::table.load(&self.connection)?)
    }

    pub fn create_from_rfactoid(&self, rfactoid: &RFactoid) -> Result<(), Error> {
        use self::schema::factoids;

        let new_factoid = NewFactoid::from_rfactoid(rfactoid)?;

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn create_factoid(
        &self,
        nickname: &str,
        intent: FactoidEnum,
        factoid: &str,
        description: &str,
        locked: bool,
    ) -> Result<(), Error> {
        use self::schema::factoids;

        let timestamp = Utc::now().naive_utc();
        let new_factoid = NewFactoid {
            label: factoid,
            intent,
            description,
            nickname,
            timestamp,
            locked,
        };

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&self.connection)?;

        Ok(())
    }
}
