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
