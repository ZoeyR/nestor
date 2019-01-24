use self::import_models::RFactoid;
use self::models::{
    Factoid, FactoidEnum, NewFactoid, NewQuote, NewWinError, Quote, WinError, WinErrorVariant,
};

use chrono::offset::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use failure::Error;
use nestor::request::{FromRequest, Request, State};

pub mod import_models;
pub mod models;
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

    pub fn all_quotes(&self) -> Result<Vec<Quote>, Error> {
        use self::schema::qotd;

        Ok(qotd::table.load(&self.connection)?)
    }

    pub fn create_quote(&self, quote: &str) -> Result<(), Error> {
        use self::schema::qotd;

        let new_quote = NewQuote { quote };

        diesel::insert_into(qotd::table)
            .values(&new_quote)
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn get_error_by_code(
        &self,
        error_code: u32,
        variant: WinErrorVariant,
    ) -> Result<Option<WinError>, Error> {
        use self::schema::winerrors::dsl::*;

        winerrors
            .filter(error_type.eq(variant))
            .filter(code.eq(error_code.to_string()))
            .first::<WinError>(&self.connection)
            .optional()
            .map_err(From::from)
    }

    pub fn get_error_by_name(
        &self,
        error_name: &str,
        variant: WinErrorVariant,
    ) -> Result<Option<WinError>, Error> {
        use self::schema::winerrors::dsl::*;

        winerrors
            .filter(error_type.eq(variant))
            .filter(name.eq(error_name))
            .first::<WinError>(&self.connection)
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

        let new_error = NewWinError {
            code: &error_code.to_string(),
            error_type,
            name,
            description,
        };

        diesel::insert_into(winerrors::table)
            .values(&new_error)
            .execute(&self.connection)?;

        Ok(())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Db {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> Result<Self, Self::Error> {
        let db: State<'a, Result<Db, Error>> = FromRequest::from_request(request)?;
        match db.inner() {
            Err(_) => Err(failure::err_msg("Failed to create db connection")),
            Ok(db) => Ok(&db),
        }
    }
}
