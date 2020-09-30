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

pub mod models;
pub mod routing;
pub mod schema;
pub mod utils;

// Managed struct that holds the db connection, specifically to a database called 'quizzes_db'
#[database("quizzes_db")]
pub struct DbConn(diesel::MysqlConnection);

fn make_cors() -> rocket_cors::Cors {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000/*"]);

    // You can also deserialize this
    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap()
}
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                routing::quiz_routes::index,
                routing::quiz_routes::get_full_quiz_route,
                routing::quiz_routes::insert_quiz,
                routing::quiz_routes::browse,
                routing::quiz_routes::search,
                routing::quiz_routes::get_quizzes_by_user_id,
                routing::quiz_routes::delete,
                routing::auth_routes::create,
                routing::auth_routes::login,
                routing::auth_routes::fetch_info_by_user_id,
                routing::auth_routes::logout,
            ],
        )
        .attach(DbConn::fairing())
        .attach(make_cors())
}
fn main() {
    rocket().launch();
}
