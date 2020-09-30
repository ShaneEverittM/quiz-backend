use super::auth_functions::logged_in;
use crate::models::quiz_models::*;
use rocket::http::RawStr;
use rocket::request::{FromFormValue, FromRequest, Outcome, Request};
// Aggregate struct to represent an entire quiz coming out of the db.
#[derive(Serialize, Debug)]
pub struct FullQuiz {
    pub quiz: Quiz,
    pub questions: Vec<Question>,
    pub answers: Vec<Vec<Answer>>,
    pub results: Vec<QuizResult>,
}

// Aggregate struct to represent an entire incoming quiz to be processed before going into the db.
#[derive(Deserialize, Debug)]
pub struct IncomingFullQuiz {
    pub quiz: IncomingQuiz,
    pub questions: Vec<IncomingQuestion>,
    pub answers: Vec<Vec<IncomingAnswer>>,
    pub results: Vec<IncomingQuizResult>,
}

#[derive(Debug)]
pub struct RouteError {
    pub error: String,
}

impl RouteError {
    pub fn new(err: &str) -> Self {
        Self {
            error: String::from(err),
        }
    }
}

impl From<diesel::result::Error> for RouteError {
    fn from(err: diesel::result::Error) -> Self {
        RouteError {
            error: err.to_string(),
        }
    }
}

impl<'r> rocket::response::Responder<'r> for RouteError {
    fn respond_to(self, _: &rocket::request::Request) -> rocket::response::Result<'r> {
        rocket::Response::build()
            .header(rocket::http::ContentType::Binary)
            .sized_body(std::io::Cursor::new(self.error))
            .ok()
    }
}

pub struct LoggedInUserID(pub i32);

impl<'a, 'r> FromRequest<'a, 'r> for LoggedInUserID {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> Outcome<LoggedInUserID, ()> {
        let id_opt = request.headers().get_one("x-api-key");
        match id_opt {
            Some(id_str) => {
                let uid = id_str.parse().unwrap();
                if logged_in(uid, &mut request.cookies()) {
                    Outcome::Success(LoggedInUserID(uid))
                } else {
                    Outcome::Failure((rocket::http::Status::Unauthorized, ()))
                }
            }
            None => Outcome::Failure((rocket::http::Status::NotFound, ())),
        }
    }
}

impl<'v> FromFormValue<'v> for LoggedInUserID {
    type Error = ();

    fn from_form_value(form_value: &'v RawStr) -> Result<LoggedInUserID, ()> {
        match form_value.parse::<i32>() {
            Ok(id) => Ok(LoggedInUserID(id)),
            _ => Err(()),
        }
    }
}
