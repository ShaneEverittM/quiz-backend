#![feature(proc_macro_hygiene, decl_macro)] // decl_macro is for route annotations, proc_macro_hygiene is for better error messages?

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use rocket_cors::CorsOptions;

pub mod models;
pub mod routes;
pub mod schema;

// Managed struct that holds the db connection, specifically to a database called 'quizzes_db'
#[database("quizzes_db")]
pub struct DbConn(diesel::MysqlConnection);

fn main() {
    rocket::ignite()
        .mount("/", routes![routes::index])
        .attach(DbConn::fairing())
        .attach(CorsOptions::default().to_cors().unwrap())
        .launch();
}
