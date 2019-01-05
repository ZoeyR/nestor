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

    pub fn get_factoid(&self, factoid: &str) -> Result<String, Error> {
        use crate::schema::factoids::dsl::*;

        let results = factoids
            .filter(label.eq(factoid))
            .limit(1)
            .load::<Factoid>(&self.connection)?;

        Ok(results
            .into_iter()
            .map(|f| f.description)
            .nth(0)
            .unwrap_or(format!("no factoid: {}", factoid)))
    }

    pub fn create_factoid(&self, factoid: &str, description: &str) -> Result<String, Error> {
        use crate::schema::factoids;

        let results = factoids::table
            .filter(factoids::label.eq(factoid))
            .limit(1)
            .load::<Factoid>(&self.connection)?;

        if !results.is_empty() {
            return Ok(format!(
                "cannot replace factoid {} since it already exists",
                factoid
            ));
        }

        let new_factoid = NewFactoid {
            label: factoid,
            description,
        };

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&self.connection)?;

        Ok(format!("learned factoid '{}'", factoid))
    }
}
