#[macro_use]
extern crate rocket; // framework

use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions}; // must appease our CORS overlords

use quizzes_backend::routing::{auth_routes::*, quiz_routes::*};

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
                index,
                get_full_quiz_route,
                insert_quiz,
                browse,
                search,
                get_quizzes_by_user_id,
                delete,
                create,
                login,
                fetch_info_by_user_id,
                logout,
            ],
        )
        .attach(quizzes_backend::DbConn::fairing())
        .attach(make_cors())
}
fn main() {
    rocket().launch();
}
