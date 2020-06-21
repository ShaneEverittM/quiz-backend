use diesel::{self, prelude::*};
use rocket::response::status::NotFound;
use rocket_contrib::json::Json; //Easy Json coercion

use crate::models::*; //Models needed for pulling or pushing data
use crate::DbConn; // The state managed DB connection

// use diesel::debug_query;
#[derive(Serialize)]
pub struct FullQuiz {
    quiz: Quiz,
    questions: Vec<Question>,
    answers: Vec<Vec<Answer>>,
}

#[get("/")]
pub fn index(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quiz::dsl::*; //convenience re-exports from 'table!' macro codegen
    let ref conn = *conn_ptr; //Pull a connection out of the connection pool
    match quiz.load::<Quiz>(conn) {
        Ok(quizzes) => Ok(Json(quizzes)),
        Err(msg) => Err(format!("Error loading quiz: {}", msg).into()),
    }
}

#[get("/quiz/<target_id>")]
pub fn get_quiz(target_id: i32, conn_ptr: DbConn) -> Result<Json<FullQuiz>, NotFound<String>> {
    use crate::schema::answer::dsl::{answer, q_id};
    use crate::schema::question::dsl::{question, qz_id};
    use crate::schema::quiz::dsl::quiz;
    let ref conn = *conn_ptr;
    let qz = quiz
        .find(target_id)
        .first::<Quiz>(conn)
        .map_err(|msg| NotFound(msg.to_string()))?;
    let questions = question
        .filter(qz_id.eq(target_id))
        .load::<Question>(conn)
        .map_err(|msg| NotFound(msg.to_string()))?;
    let mut answers: Vec<Vec<Answer>> = Vec::new();
    for cur_question in &questions {
        let inner_answers = answer
            .filter(q_id.eq(cur_question.id))
            .load::<Answer>(conn)
            .map_err(|msg| NotFound(msg.to_string()))?;
        answers.push(inner_answers);
    }

    let full_quiz = FullQuiz {
        quiz: qz,
        questions,
        answers,
    };
    Ok(Json(full_quiz))
}

// let insert_query = insert_into(quiz).values(name.eq("Test5"));
// match insert_query.execute(conn) {
//     Ok(rows_changed) => println!("{} rows changed", rows_changed),
//     Err(msg) => println!(
//         "Query: {} \n failed with error: {}",
//         debug_query::<diesel::mysql::Mysql, _>(&insert_query),
//         msg
//     ),
// }
