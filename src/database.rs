use crate::models::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use failure::Error;
use std::path::Path;

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
            .find(factoid)
            .first::<Factoid>(&self.connection)
            .optional()?;

        Ok(factoid.map(|f| f.description))
    }

    pub fn create_factoid(&self, factoid: &str, description: &str) -> Result<(), Error> {
        use crate::schema::factoids;

        let new_factoid = NewFactoid {
            id: factoid,
            description,
        };

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn lock_factoid(&self, factoid: &str) -> Result<(), Error> {
        use crate::schema::factoids;

        let factoid = factoids::table
            .find(factoid)
            .first::<Factoid>(&self.connection)?;

        diesel::update(&factoid)
            .set(factoids::locked.eq(true))
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn unlock_factoid(&self, factoid: &str) -> Result<(), Error> {
        use crate::schema::factoids;

        let factoid = factoids::table
            .find(factoid)
            .first::<Factoid>(&self.connection)?;

        diesel::update(&factoid)
            .set(factoids::locked.eq(false))
            .execute(&self.connection)?;

        Ok(())
    }
}
