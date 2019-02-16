use self::import_models::RFactoid;
use self::models::{
    Factoid, FactoidEnum, NewFactoid, NewQuote, NewWinError, Quote, WinError, WinErrorVariant,
};

use chrono::offset::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use failure::Error;

pub mod import_models;
pub mod models;
pub mod schema;

embed_migrations!();

pub struct Db {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(path);
        let pool = Pool::builder().build(manager)?;
        embedded_migrations::run(&pool.get()?)?;

        Ok(Db { pool })
    }

    pub fn get_factoid(&self, key: &str) -> Result<Option<Factoid>, Error> {
        use self::schema::factoids::dsl::*;

        let connection = self.pool.get()?;
        factoids
            .filter(label.eq(key))
            .order(timestamp.desc())
            .first::<Factoid>(&connection)
            .optional()
            .map_err(From::from)
    }

    pub fn all_factoids(&self) -> Result<Vec<Factoid>, Error> {
        use self::schema::factoids;

        let connection = self.pool.get()?;
        Ok(factoids::table.load(&connection)?)
    }

    pub fn create_from_rfactoid(&self, rfactoid: &RFactoid) -> Result<(), Error> {
        use self::schema::factoids;

        let connection = self.pool.get()?;
        let new_factoid = NewFactoid::from_rfactoid(rfactoid)?;

        diesel::insert_into(factoids::table)
            .values(&new_factoid)
            .execute(&connection)?;

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

        let connection = self.pool.get()?;
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
            .execute(&connection)?;

        Ok(())
    }

    pub fn all_quotes(&self) -> Result<Vec<Quote>, Error> {
        use self::schema::qotd;

        let connection = self.pool.get()?;
        Ok(qotd::table.load(&connection)?)
    }

    pub fn create_quote(&self, quote: &str) -> Result<(), Error> {
        use self::schema::qotd;

        let connection = self.pool.get()?;
        let new_quote = NewQuote { quote };

        diesel::insert_into(qotd::table)
            .values(&new_quote)
            .execute(&connection)?;

        Ok(())
    }

    pub fn get_error_by_code(
        &self,
        error_code: u32,
        variant: WinErrorVariant,
    ) -> Result<Option<WinError>, Error> {
        use self::schema::winerrors::dsl::*;

        let connection = self.pool.get()?;
        winerrors
            .filter(error_type.eq(variant))
            .filter(code.eq(error_code.to_string()))
            .first::<WinError>(&connection)
            .optional()
            .map_err(From::from)
    }

    pub fn get_error_by_name(
        &self,
        error_name: &str,
        variant: WinErrorVariant,
    ) -> Result<Option<WinError>, Error> {
        use self::schema::winerrors::dsl::*;

        let connection = self.pool.get()?;
        winerrors
            .filter(error_type.eq(variant))
            .filter(name.eq(error_name))
            .first::<WinError>(&connection)
            .optional()
            .map_err(From::from)
    }

    pub fn create_error(
        &self,
        error_code: u32,
        error_type: WinErrorVariant,
        name: &str,
        description: &str,
    ) -> Result<(), Error> {
        use self::schema::winerrors;

        let connection = self.pool.get()?;
        let new_error = NewWinError {
            code: &error_code.to_string(),
            error_type,
            name,
            description,
        };

        diesel::insert_into(winerrors::table)
            .values(&new_error)
            .execute(&connection)?;

        Ok(())
    }
}
