use diesel::{self, prelude::*}; //common diesel things

use rocket::http::{Cookie, Cookies};
use rocket::response::status::{Conflict, NotFound}; // Response types
use rocket_contrib::json::Json; // Easy Json coercion

use crypto::digest::Digest;
use crypto::sha3::Sha3;

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

#[derive(Deserialize)]
pub struct CreateInfo {
    name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

// enum LoginError {
//     InvalidData,
//     UsernameDoesNotExist,
//     WrongPassword,
// }

// Test route.
#[get("/")]
pub fn index(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quiz::dsl::quiz as quiz_table; //convenience re-exports from 'table!' macro codegen
    let ref conn = *conn_ptr; //Pull a connection out of the connection pool
    Ok(Json(
        quiz_table
            .limit(6)
            .load::<Quiz>(conn)
            .map_err(|msg| msg.to_string())?, //TODO create a from trait impl to avoid this everywhere
    ))
}

#[post("/users/create", format = "json", data = "<create_info>")]
pub fn create(conn_ptr: DbConn, create_info: Json<CreateInfo>) -> Json<i32> {
    use crate::schema::auth_info::dsl::auth_info as auth_info_table;
    use crate::schema::user::dsl::user as user_table;
    let user = NewUser {
        name: create_info.name.clone(),
        email: create_info.email.clone(),
    };
    let ref conn = *conn_ptr;
    let _rows_changed = diesel::insert_into(user_table)
        .values(user)
        .execute(conn)
        .expect("error");

    let password_hash = hash_password(&create_info.password);
    let last_uid: u64 = diesel::select(last_insert_id).first(conn).expect("Error");
    let auth_info = NewAuthInfo {
        uid: last_uid as i32,
        password_hash: password_hash,
    };
    let _rows_changed = diesel::insert_into(auth_info_table)
        .values(auth_info)
        .execute(conn)
        .expect("Error");
    Json(last_uid as i32)
}
#[post("/users/login", format = "json", data = "<login_info>")]
pub fn login(
    conn_ptr: DbConn,
    login_info: Json<LoginInfo>,
    mut cookies: Cookies,
) -> Json<Option<i32>> {
    let ref conn = *conn_ptr;
    let user_opt = fetch_user_by_email(conn, &login_info.username);
    match user_opt {
        Some(user) => {
            let auth_opt = fetch_auth_info_by_user_id(conn, user.id);
            match auth_opt {
                Some(auth_info) => {
                    let hash = hash_password(&login_info.password);
                    if hash == auth_info.password_hash {
                        cookies.add_private(Cookie::new("user_id", user.id.to_string()));
                        Json(Some(user.id))
                    } else {
                        Json(None)
                    }
                }
                None => Json(None),
            }
        }
        None => Json(None),
    }
}

#[get("/users/cookies/<uid>")]
pub fn fetch_info_by_user_id(
    conn_ptr: DbConn,
    uid: i32,
    mut cookies: Cookies,
) -> Json<Option<User>> {
    let logged_in_user = cookies.get_private("user_id");
    let ref conn = *conn_ptr;
    match logged_in_user {
        Some(c) => {
            let logged_in_uid = c.value().parse::<i32>().unwrap();
            if logged_in_uid == uid {
                Json(fetch_user_by_id(conn, uid))
            } else {
                Json(None)
            }
        }
        None => Json(None),
    }
}

fn fetch_auth_info_by_user_id(conn: &diesel::MysqlConnection, input_uid: i32) -> Option<AuthInfo> {
    use crate::schema::auth_info::dsl::*;
    let mut auth_infos_by_uid: Vec<AuthInfo> = auth_info
        .filter(uid.eq(input_uid))
        .load::<AuthInfo>(conn)
        .expect("Error");
    if auth_infos_by_uid.len() == 0 {
        None
    } else {
        let first = auth_infos_by_uid.remove(0);
        Some(first)
    }
}

fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}

fn fetch_user_by_email(conn: &diesel::MysqlConnection, input_email: &String) -> Option<User> {
    use crate::schema::user::dsl::*;
    let mut users_by_id: Vec<User> = user
        .filter(email.eq(input_email))
        .load::<User>(conn)
        .expect("Error");
    if users_by_id.len() == 0 {
        None
    } else {
        let first = users_by_id.remove(0);
        Some(first)
    }
}

fn fetch_user_by_id(conn: &diesel::MysqlConnection, input_id: i32) -> Option<User> {
    use crate::schema::user::dsl::*;
    let mut users_by_id: Vec<User> = user
        .filter(id.eq(input_id))
        .load::<User>(conn)
        .expect("Error");
    if users_by_id.len() == 0 {
        None
    } else {
        Some(users_by_id.remove(0))
    }
}

#[get("/browse")]
pub fn browse(conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quiz::dsl::{name, quiz as quiz_table};
    let ref conn = *conn_ptr;
    Ok(Json(
        quiz_table
            .order(name.asc())
            .load::<Quiz>(conn)
            .map_err(|msg| msg.to_string())?,
    ))
}

#[get("/search?<query>")]
pub fn search(query: String, conn_ptr: DbConn) -> Result<Json<Vec<Quiz>>, NotFound<String>> {
    let ref conn = *conn_ptr;
    let sql_query_string = query.replace(" ", "%");
    let sql_query_string = String::from("%") + &sql_query_string + "%";
    let quizzes: Vec<Quiz> = diesel::sql_query(
        "SELECT * from quiz where match(name, description) against (? in boolean mode)",
    )
    .bind::<diesel::sql_types::Text, _>(sql_query_string)
    .load(conn)
    .unwrap();

    Ok(Json(quizzes))
}

// This route handles retrieval of all of the constituant parts of a quiz from their
// tables and assembles them into a large struct and sends it as JSON.
#[get("/quiz/<quiz_id>")]
pub fn get_full_quiz_route(
    quiz_id: i32,
    conn_ptr: DbConn,
) -> Result<Json<FullQuiz>, NotFound<String>> {
    get_full_quiz(quiz_id, &*conn_ptr)
}
pub fn get_full_quiz(
    quiz_id: i32,
    conn: &diesel::MysqlConnection,
) -> Result<Json<FullQuiz>, NotFound<String>> {
    // TODO Could do some of these concurrently
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

    Ok(conn
        .transaction::<String, diesel::result::Error, _>(|| {
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
        .map_err(|msg| Conflict(Some(msg.to_string())))?)
}
