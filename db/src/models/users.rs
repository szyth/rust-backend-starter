use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, serde::Serialize, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

/// Inserts a user and returns the stored row (id and created_at come from Postgres).
pub fn insert(name: &str, email: &str) -> Result<User, crate::DBError> {
    use crate::schema::users;
    let mut conn = crate::pg::get_conn()?;
    Ok(diesel::insert_into(users::table)
        .values(NewUser { name, email })
        .returning(User::as_returning())
        .get_result(&mut conn)?)
}
