use diesel::{self, prelude::*}; //common diesel things

use rocket::response::status::{Conflict, NotFound}; // Response types
use rocket_contrib::json::Json; // Easy Json coercion

use crate::models::*; // Models needed for pulling or pushing data
use crate::sql_utils::last_insert_id; //utility for getting around mysql being bad
use crate::DbConn; // The state managed DB connection

// Aggregate struct to represent an entire quiz coming out of the db.
#[derive(Serialize, Debug)]
pub struct FullQuiz {
    quiz: Quiz,
    questions: Vec<Question>,
    answers: Vec<Vec<Answer>>,
    results: Vec<QuizResult>,
}

// Aggregate struct to represent an entire incoming quiz to be processed before going into the db.
#[derive(Deserialize, Debug)]
pub struct IncomingFullQuiz {
    quiz: IncomingQuiz,
    questions: Vec<IncomingQuestion>,
    answers: Vec<Vec<IncomingAnswer>>,
    results: Vec<IncomingQuizResult>,
}

#[derive(std::fmt::Debug)]
pub struct RouteError {
    error: String,
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
// Test route.
#[get("/")]
pub fn index(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, RouteError> {
    use crate::schema::quiz::dsl::quiz as quiz_table; //convenience re-exports from 'table!' macro codegen
    let ref conn = *conn_ptr; //Pull a connection out of the connection pool
    Ok(Json(quiz_table.limit(6).load::<Quiz>(conn)?))
}

#[get("/browse")]
pub fn browse(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, RouteError> {
    use crate::schema::quiz::dsl::{name, quiz as quiz_table};
    let ref conn = *conn_ptr;
    Ok(Json(quiz_table.order(name.asc()).load::<Quiz>(conn)?))
}

#[get("/search?<query>")]
pub fn search(query: String, conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, NotFound<RouteError>> {
    let ref conn = *conn_ptr;
    let sql_query_string = query.replace(" ", "*");
    let sql_query_string = String::from("*") + &sql_query_string + "*";
    diesel::sql_query(
        "SELECT * from quiz where match(name, description) against (? in boolean mode)",
    )
    .bind::<diesel::sql_types::Text, _>(sql_query_string)
    .load(conn)
    .map_err(|e| NotFound(e.into()))
    .map(|val| Json(val))
}

// This route handles retrieval of all of the constituant parts of a quiz from their
// tables and assembles them into a large struct and sends it as JSON.
#[get("/quiz/<quiz_id>")]
pub fn get_full_quiz_route(
    quiz_id: i32,
    conn_ptr: DbConn,
) -> Result<Json<FullQuiz>, NotFound<RouteError>> {
    get_full_quiz(quiz_id, &*conn_ptr)
}
pub fn get_full_quiz(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Json<FullQuiz>, NotFound<RouteError>> {
    // Cannot do these concurrently, because they are all using the same db connection
    let quiz = get_quiz(quiz_id, conn)?;
    let questions = get_questions(quiz_id, conn)?;
    let answers = get_answers(&questions, conn)?;
    let results = get_results(quiz_id, conn)?;
    Ok(Json(FullQuiz {
        quiz,
        questions,
        answers,
        results,
    }))
}

fn get_quiz(quiz_id: i32, conn: &diesel::MysqlConnection) -> Result<Quiz, NotFound<RouteError>> {
    use crate::schema::quiz::dsl::quiz as quiz_table;
    quiz_table
        .find(quiz_id)
        .first::<Quiz>(conn)
        .map_err(|msg| NotFound(msg.into()))
}

fn get_questions(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<Question>, NotFound<RouteError>> {
    use crate::schema::question::dsl::{question as question_table, qz_id};
    question_table
        .filter(qz_id.eq(quiz_id))
        .load::<Question>(conn)
        .map_err(|msg| NotFound(msg.into()))
}

fn get_answers(
    questions: &Vec<Question>,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<Vec<Answer>>, NotFound<RouteError>> {
    use crate::schema::answer::dsl::{answer as answer_table, q_id};
    let mut answers: Vec<Vec<Answer>> = Vec::new();
    for cur_question in questions {
        let inner_answers = answer_table
            .filter(q_id.eq(cur_question.id))
            .load::<Answer>(conn)
            .map_err(|msg| NotFound(msg.into()))?;
        answers.push(inner_answers);
    }
    Ok(answers)
}

fn get_results(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<QuizResult>, NotFound<RouteError>> {
    use crate::schema::result::dsl::{qz_id, result as result_table};
    result_table
        .filter(qz_id.eq(quiz_id))
        .load::<QuizResult>(conn)
        .map_err(|msg| NotFound(msg.into()))
}

// This route handles adding new quizzes to the db. Takes a large amount of data in the body
// and destructures it into its fields for insertion into their respective tables.
#[post("/quiz", format = "json", data = "<f_quiz>")]
pub fn insert_quiz(
    f_quiz: Json<IncomingFullQuiz>,
    conn_ptr: DbConn,
) -> Result<String, Conflict<RouteError>> {
    use crate::schema::answer::dsl::answer as answer_table;
    use crate::schema::question::dsl::question as question_table;
    use crate::schema::quiz::dsl::quiz as quiz_table;
    use crate::schema::result::dsl::result as result_table;
    let ref conn = *conn_ptr;

    let IncomingFullQuiz {
        quiz,
        questions,
        answers,
        results,
    } = f_quiz.into_inner();

    // Attempts to insert and associate all the new records under a transaction, rolling back under failure
    conn.transaction::<String, diesel::result::Error, _>(|| {
        diesel::insert_into(quiz_table)
            .values(NewQuiz::from(quiz))
            .execute(conn)?;
        let last_qz_id: u64 = diesel::select(last_insert_id).first(conn)?;
        let mut cur_question = 0;
        for qs in questions {
            let question_to_add = NewQuestion {
                description: qs.description.clone(),
                qz_id: last_qz_id as i32,
            };
            let _row_changed = diesel::insert_into(question_table)
                .values(question_to_add)
                .execute(conn)?;
            let last_q_id: u64 = diesel::select(last_insert_id).first(conn)?;
            for ans in &answers[cur_question] {
                let answer_to_add = NewAnswer {
                    description: ans.description.clone(),
                    val: ans.val,
                    q_id: last_q_id as i32,
                };
                let _rows_changed = diesel::insert_into(answer_table)
                    .values(answer_to_add)
                    .execute(conn)?;
            }
            cur_question += 1;
        }

        let new_results: Vec<NewQuizResult> = results
            .iter()
            .enumerate()
            .map(|(i, q)| NewQuizResult {
                num: i as i32,
                header: q.header.clone(),
                description: q.description.clone(),
                qz_id: last_qz_id as i32,
            })
            .collect();
        diesel::insert_into(result_table)
            .values(new_results)
            .execute(conn)?;

        Ok("Inserted".into())
    })
    .map_err(|msg| Conflict(Some(msg.into())))
}
