use crate::schema::factoids;

#[derive(Queryable, Identifiable)]
pub struct Factoid {
    pub id: String,
    pub description: String,
    pub locked: bool,
}

#[derive(Insertable)]
#[table_name = "factoids"]
pub struct NewFactoid<'a> {
    pub id: &'a str,
    pub description: &'a str,
}
