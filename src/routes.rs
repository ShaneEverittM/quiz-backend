use diesel::{self, prelude::*}; //common diesel things

use rocket::response::status::{Conflict, NotFound}; // Response types
use rocket_contrib::json::Json; // Easy Json coercion

use crate::models::*; // Models needed for pulling or pushing data
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

// Test route.
#[get("/")]
pub fn index(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quiz::dsl::*; //convenience re-exports from 'table!' macro codegen
    let ref conn = *conn_ptr; //Pull a connection out of the connection pool
    match quiz.load::<Quiz>(conn) {
        Ok(quizzes) => Ok(Json(quizzes)),
        Err(msg) => Err(format!("Error loading quiz: {}", msg).into()),
    }
}

// This route handles retrieval of all of the constituant parts of a quiz from their
// tables and assembles them into a large struct and sends it as JSON.
#[get("/quiz/<quiz_id>")]
pub fn get_full_quiz(quiz_id: i32, conn_ptr: DbConn) -> Result<Json<FullQuiz>, NotFound<String>> {
    // Could do some of these concurrently
    let quiz = get_quiz(quiz_id, &*conn_ptr)?;
    let questions = get_questions(quiz_id, &*conn_ptr)?;
    let answers = get_answers(&questions, &*conn_ptr)?;
    let results = get_results(quiz_id, &*conn_ptr)?;
    Ok(Json(FullQuiz {
        quiz,
        questions,
        answers,
        results,
    }))
}

fn get_quiz(quiz_id: i32, conn: &diesel::MysqlConnection) -> Result<Quiz, NotFound<String>> {
    use crate::schema::quiz::dsl::quiz as quiz_table;
    Ok(quiz_table
        .find(quiz_id)
        .first::<Quiz>(conn)
        .map_err(|msg| NotFound(msg.to_string()))?)
}

fn get_questions(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<Question>, NotFound<String>> {
    use crate::schema::question::dsl::{question as question_table, qz_id};
    Ok(question_table
        .filter(qz_id.eq(quiz_id))
        .load::<Question>(conn)
        .map_err(|msg| NotFound(msg.to_string()))?)
}

fn get_answers(
    questions: &Vec<Question>,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<Vec<Answer>>, NotFound<String>> {
    use crate::schema::answer::dsl::{answer as answer_table, q_id};
    let mut answers: Vec<Vec<Answer>> = Vec::new();
    for cur_question in questions {
        let inner_answers = answer_table
            .filter(q_id.eq(cur_question.id))
            .load::<Answer>(conn)
            .map_err(|msg| NotFound(msg.to_string()))?;
        answers.push(inner_answers);
    }
    Ok(answers)
}

fn get_results(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<QuizResult>, NotFound<String>> {
    use crate::schema::result::dsl::{qz_id, result as result_table};
    Ok(result_table
        .filter(qz_id.eq(quiz_id))
        .load::<QuizResult>(conn)
        .map_err(|msg| NotFound(msg.to_string()))?)
}

// no_arg_sql_function!(function_name, return_type)
// Generates a FFI of a specific signature for db_name.function_name()
// In this case its quizzes_db.last_insert_id() -> sql::BigInt
no_arg_sql_function!(
    last_insert_id,
    diesel::sql_types::Unsigned<diesel::sql_types::BigInt>
);
// This route handles adding new quizzes to the db. Takes a large amount of data in the body
// and destructures it into its fields for insertion into their respective tables.
#[post("/quiz", format = "json", data = "<f_quiz>")]
pub fn insert_quiz(
    f_quiz: Json<IncomingFullQuiz>,
    conn_ptr: DbConn,
) -> Result<String, Conflict<String>> {
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

    let _rows_changed = diesel::insert_into(quiz_table)
        .values(NewQuiz::from(quiz))
        .execute(conn)
        .map_err(|msg| Conflict(Some(msg.to_string())))?;
    let last_qz_id: u64 = diesel::select(last_insert_id)
        .first(conn)
        .map_err(|msg| Conflict(Some(msg.to_string())))?;
    let mut cur_question = 0;
    for qs in questions {
        let question_to_add = NewQuestion {
            description: qs.description.clone(),
            qz_id: last_qz_id as i32,
        };
        let _row_changed = diesel::insert_into(question_table)
            .values(question_to_add)
            .execute(conn)
            .map_err(|msg| Conflict(Some(msg.to_string())))?;
        let last_q_id: u64 = diesel::select(last_insert_id)
            .first(conn)
            .map_err(|msg| Conflict(Some(msg.to_string())))?;
        for ans in &answers[cur_question] {
            let answer_to_add = NewAnswer {
                description: ans.description.clone(),
                val: ans.val,
                q_id: last_q_id as i32,
            };
            let _rows_changed = diesel::insert_into(answer_table)
                .values(answer_to_add)
                .execute(conn)
                .map_err(|msg| Conflict(Some(msg.to_string())))?;
            cur_question += 1;
        }
    }

    let new_results: Vec<NewQuizResult> = results
        .iter()
        .map(|q| NewQuizResult {
            num: q.num,
            header: q.header.clone(),
            description: q.description.clone(),
            qz_id: last_qz_id as i32,
        })
        .collect();
    diesel::insert_into(result_table)
        .values(new_results)
        .execute(conn)
        .map_err(|msg| Conflict(Some(msg.to_string())))?;
    Ok("Inserted".into())
}
