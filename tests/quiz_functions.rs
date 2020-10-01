#[test]
fn test_get_full_quiz() {
    let conn: diesel::MysqlConnection =
        diesel::connection::Connection::establish("mysql://rocket:password@localhost/quizzes_db")
            .unwrap();
    let res = quizzes_backend::routing::quiz_functions::get_full_quiz(100, &conn);
    dbg!(&res);
}
