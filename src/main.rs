#![feature(proc_macro_hygiene, decl_macro)] // decl_macro is for route annotations, proc_macro_hygiene is for better error messages

#[macro_use]
extern crate diesel; // ORM and query builder

#[macro_use]
extern crate rocket; // framework

#[macro_use]
extern crate rocket_contrib; // useful community libraries
use rocket::http::Method;
use rocket_cors::AllowedOrigins;

// Because everyone needs serde
extern crate serde;
extern crate serde_json;

extern crate crypto;

#[macro_use]
extern crate serde_derive; // to be able to derive

use rocket_cors::CorsOptions; // must appease our CORS overlords

// apparently this is how you declare modules, at the crate root
pub mod auth_routes;
pub mod models;
pub mod quiz_routes;
pub mod schema;
pub mod sql_utils;

// Managed struct that holds the db connection, specifically to a database called 'quizzes_db'
#[database("quizzes_db")]
pub struct DbConn(diesel::MysqlConnection);

fn main() {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000/*"]);

    // You can also deserialize this
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();
    rocket::ignite()
        .mount(
            "/",
            routes![
                quiz_routes::index,
                quiz_routes::get_full_quiz_route,
                quiz_routes::insert_quiz,
                quiz_routes::browse,
                quiz_routes::search,
                quiz_routes::get_quizzes_by_user_id,
                auth_routes::create,
                auth_routes::login,
                auth_routes::fetch_info_by_user_id,
                auth_routes::logout,
            ],
        )
        .attach(DbConn::fairing())
        .attach(cors)
        .launch();
}
