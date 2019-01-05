use crate::schema::factoids;

#[derive(Queryable)]
pub struct Factoid {
    pub label: String,
    pub description: String,
}

#[derive(Insertable)]
#[table_name = "factoids"]
pub struct NewFactoid<'a> {
    pub label: &'a str,
    pub description: &'a str,
}
