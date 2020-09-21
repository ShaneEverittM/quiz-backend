#![feature(proc_macro_hygiene, decl_macro)] // decl_macro is for route annotations, proc_macro_hygiene is for better error messages

#[macro_use]
extern crate diesel; // ORM and query builder

#[macro_use]
extern crate rocket; // framework

#[macro_use]
extern crate rocket_contrib; // useful community libraries

// Because everyone needs serde
extern crate serde;
extern crate serde_json;

extern crate crypto;

#[macro_use]
extern crate serde_derive; // to be able to derive

use rocket_cors::CorsOptions; // must appease our CORS overlords

// apparently this is how you declare modules, at the crate root
pub mod models;
pub mod routes;
pub mod schema;

// Managed struct that holds the db connection, specifically to a database called 'quizzes_db'
#[database("quizzes_db")]
pub struct DbConn(diesel::MysqlConnection);

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::index,
                routes::get_full_quiz_route,
                routes::insert_quiz,
                routes::browse,
                routes::search,
                routes::create,
                routes::login,
                routes::fetch_info_by_user_id,
            ],
        )
        .attach(DbConn::fairing())
        .attach(CorsOptions::default().to_cors().unwrap())
        .launch();
}
