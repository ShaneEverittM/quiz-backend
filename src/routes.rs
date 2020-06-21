use diesel::{self, prelude::*};

use rocket_contrib::json::Json;

use crate::models::Quiz;
use crate::schema::*;
use crate::DbConn;

#[get("/")]
pub fn index(conn: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quizzes::dsl::*;
    quizzes
        .load(&conn.0)
        .map_err(|err| -> String {
            println!("Error");
            "Error".into()
        })
        .map(Json)
}
