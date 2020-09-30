use diesel::{self, prelude::*}; //common diesel things

use rocket::request::Outcome;
use rocket::response::status::{Conflict, NotFound}; // Response types
use rocket_contrib::json::Json; // Easy Json coercion

use crate::models::quiz_models::*; // Models needed for pulling or pushing data
use crate::utils::sql_utils::last_insert_id; //utility for getting around mysql being bad
use crate::DbConn; // The state managed DB connection

use super::quiz_functions::*;
use super::quiz_types::*;

//TODO: Think about error handling improvements

//TODO: Create an Edit route
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

#[get("/quizzes?<user_id>")]
pub fn get_quizzes_by_user_id(
    user_id: LoggedInUserID,
    conn_ptr: DbConn,
) -> Result<Json<Vec<Quiz>>, NotFound<RouteError>> {
    let ref conn = *conn_ptr;
    use crate::schema::quiz::dsl::{quiz as quiz_table, u_id};
    let quizzes: Vec<Quiz> = quiz_table
        .filter(u_id.eq(user_id.0))
        .load::<Quiz>(conn)
        .map_err(|e| NotFound(e.into()))?;

    Ok(Json(quizzes))
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
// This route handles adding new quizzes to the db. Takes a large amount of data in the body
// and destructures it into its fields for insertion into their respective tables.
#[post("/quiz", format = "json", data = "<f_quiz>")]
pub fn insert_quiz(
    f_quiz: Json<IncomingFullQuiz>,
    conn_ptr: DbConn,
) -> Result<Json<i32>, Conflict<RouteError>> {
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
    conn.transaction::<Json<i32>, diesel::result::Error, _>(|| {
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

        Ok(Json(last_qz_id as i32))
    })
    .map_err(|msg| {
        Conflict(Some(RouteError {
            error: msg.to_string(),
        }))
    })
}
#[allow(unused_variables)]
#[delete("/quiz?<quiz_id>&<user_id>")]
pub fn delete(
    quiz_id: i32,
    user_id: LoggedInUserID,
    conn_ptr: DbConn,
) -> Result<(), Outcome<(), String>> {
    use crate::schema::quiz::dsl::{id, quiz as quiz_table};
    let ref conn = *conn_ptr;
    let res = diesel::delete(quiz_table.filter(id.eq(quiz_id))).execute(conn);
    match res {
        Ok(_) => Ok(()),
        Err(msg) => Err(Outcome::Failure((
            rocket::http::Status::InternalServerError,
            msg.to_string(),
        ))),
    }
}
