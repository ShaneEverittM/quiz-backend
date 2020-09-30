use crate::schema::*;

/* -------------------------------------------------------------------------- */
/*        Models for query results, analagous to the records in the db.       */
/* -------------------------------------------------------------------------- */

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Queryable)]
pub struct AuthInfo {
    pub id: i32,
    pub uid: i32,
    pub password_hash: String,
}

/* -------------------------------------------------------------------------- */
/*         Models for data to be inserted. Adds calculated db fields.         */
/* -------------------------------------------------------------------------- */

#[derive(Insertable, Debug)]
#[table_name = "user"]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(Insertable, Debug)]
#[table_name = "auth_info"]
pub struct NewAuthInfo {
    pub uid: i32,
    pub password_hash: String,
}
