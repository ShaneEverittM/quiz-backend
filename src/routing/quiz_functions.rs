use super::quiz_types::*;
use crate::models::quiz_models::*;
use diesel::{self, prelude::*};
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
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

pub fn get_quiz(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Quiz, NotFound<RouteError>> {
    use crate::schema::quiz::dsl::quiz as quiz_table;
    quiz_table
        .find(quiz_id)
        .first::<Quiz>(conn)
        .map_err(|msg| NotFound(msg.into()))
}

pub fn get_questions(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<Question>, NotFound<RouteError>> {
    use crate::schema::question::dsl::{question as question_table, qz_id};
    question_table
        .filter(qz_id.eq(quiz_id))
        .load::<Question>(conn)
        .map_err(|msg| NotFound(msg.into()))
}

pub fn get_answers(
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

pub fn get_results(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Vec<QuizResult>, NotFound<RouteError>> {
    use crate::schema::result::dsl::{qz_id, result as result_table};
    result_table
        .filter(qz_id.eq(quiz_id))
        .load::<QuizResult>(conn)
        .map_err(|msg| NotFound(msg.into()))
}
