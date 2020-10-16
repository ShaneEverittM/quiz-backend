use diesel::prelude::*;
use diesel::MysqlConnection;
use quizzes_backend;
fn get_connection() -> diesel::MysqlConnection {
    MysqlConnection::establish("mysql://rocket:password@localhost/quizzes_db")
        .expect("Could not establish connection")
}

#[test]
fn test_get_full_quiz() {
    let conn = get_connection();
    let res = quizzes_backend::routes::quiz_functions::get_full_quiz(100, &conn);
    dbg!(&res);
}
