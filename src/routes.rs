use diesel::{self, insert_into, prelude::*};

use rocket::response::status::{Conflict, NotFound};
use rocket_contrib::json::Json; //Easy Json coercion

use crate::models::*; //Models needed for pulling or pushing data
use crate::DbConn; // The state managed DB connection

// use diesel::debug_query;
#[derive(Serialize, Deserialize, Debug)]
pub struct FullQuiz {
    quiz: Quiz,
    questions: Vec<Question>,
    answers: Vec<Vec<Answer>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct NewFullQuiz {
    quiz: NewQuiz,
    questions: Vec<IncomingQuestion>,
    answers: Vec<Vec<IncomingAnswer>>,
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

no_arg_sql_function!(
    last_insert_id,
    diesel::sql_types::Unsigned<diesel::sql_types::BigInt>
);

#[post("/quiz", format = "json", data = "<f_quiz>")]
pub fn insert_quiz(
    f_quiz: Json<NewFullQuiz>,
    conn_ptr: DbConn,
) -> Result<String, Conflict<String>> {
    use crate::schema::answer::dsl::answer;
    use crate::schema::question::dsl::question;
    use crate::schema::quiz::dsl::quiz;
    let ref conn = *conn_ptr;

    let f_quiz_struct = f_quiz.into_inner();
    let q = f_quiz_struct.quiz;

    let _ = insert_into(quiz)
        .values(q)
        .execute(conn)
        .map_err(|msg| Conflict(Some(msg.to_string())))?;

    let last_qz_id: u64 = diesel::select(last_insert_id)
        .first(conn)
        .map_err(|msg| Conflict(Some(msg.to_string())))?;
    let cur_question = 0;
    for qs in &f_quiz_struct.questions {
        let question_to_add = NewQuestion {
            description: qs.description.clone(),
            qz_id: last_qz_id as i32,
        };
        let _ = insert_into(question)
            .values(question_to_add)
            .execute(conn)
            .map_err(|msg| Conflict(Some(msg.to_string())))?;
        let last_q_id: u64 = diesel::select(last_insert_id)
            .first(conn)
            .map_err(|msg| Conflict(Some(msg.to_string())))?;
        for ans in &f_quiz_struct.answers[cur_question] {
            let answer_to_add = NewAnswer {
                description: ans.description.clone(),
                val: ans.val,
                q_id: last_q_id as i32,
            };
            let _ = insert_into(answer)
                .values(answer_to_add)
                .execute(conn)
                .map_err(|msg| Conflict(Some(msg.to_string())))?;
        }
    }
    Ok("Inserted".into())
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
