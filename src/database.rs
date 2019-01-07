use crate::models::*;
use chrono::offset::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use failure::Error;

pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, Error> {
        let connection = SqliteConnection::establish(path)?;

        Ok(Db { connection })
    }

    pub fn get_factoid(&self, factoid: &str) -> Result<Option<String>, Error> {
        use crate::schema::factoids::dsl::*;

        let factoid = factoids
            .filter(label.eq(factoid))
            .order(timestamp.desc())
            .first::<Factoid>(&self.connection)
            .optional()?;

        Ok(factoid.map(|f| f.description))
    }

    pub fn create_factoid(
        &self,
        nickname: &str,
        intent: FactoidEnum,
        factoid: &str,
        description: &str,
    ) -> Result<(), Error> {
        use crate::schema::factoids;

        let timestamp = Utc::now().naive_utc();
        let new_factoid = NewFactoid {
            label: factoid,
            intent,
            description,
            nickname,
            timestamp,
        };

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&self.connection)?;

        Ok(())
    }
}
