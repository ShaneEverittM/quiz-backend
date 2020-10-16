#[macro_use]
extern crate rocket; // framework
#[macro_use]
extern crate rocket_contrib;

use rocket::http::Method;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedOrigins, CorsOptions}; // must appease our CORS overlords

use quizzes_backend::routes::{auth_routes::*, quiz_routes::*};

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

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use serde_json::{Result, Value};
    #[test]
    fn test_index() {
        let client = Client::new(rocket()).expect("failed to create a rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        eprintln!("{}", response.body_string().unwrap());
    }
    #[test]
    fn test_insert() {
        let client = Client::new(rocket()).expect("failed to create a rocket instance");
        let raw_json = r#"
            {
                "quiz": {
                    "name": "test_insert_quiz",
                    "description": "test",
                    "u_id": 1
                },
                "questions": [
                    {
                        "description": "description"
                    },
                    {
                        "description": "description"
                    }
                ],
                "answers": [
                    [
                        {
                            "description": "description",
                            "val": 1
                        }
                    ],
                    [
                        {
                            "description": "description",
                            "val": 2
                        }
                    ]  
                ],
                "results": [
                    {
                        "num": 1,
                        "header": "header",
                        "description":"description"
                    }
                ]
            }
"#;
        let json: Value = serde_json::from_str(raw_json).unwrap();
        eprintln!("The json is: {}", json);
        let mut response = client
            .post("/quiz")
            .header(ContentType::JSON)
            .body(raw_json)
            .dispatch();
        let mut returned_quiz = client.get("/quiz/2").dispatch();
        let returned_json: Value =
            serde_json::from_str(&returned_quiz.body_string().unwrap()).unwrap();
        // dbg!(json);
        // dbg!(returned_json);
        assert_eq!(json, returned_json);
        // dbg!(response);
    }
}
