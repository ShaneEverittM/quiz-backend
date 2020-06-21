use diesel::debug_query;
use diesel::{self, prelude::*};
use rocket_contrib::json::Json; //Easy Json coercion

use crate::models::{NewQuiz, Quiz};
use crate::DbConn;

#[get("/")]
pub fn index(conn: DbConn) -> Result<Json<Vec<Quiz>>, String> {
    use crate::schema::quizzes::dsl::*;
    let conn = &conn.0;
    use diesel::insert_into;

    // let insert_query = insert_into(quizzes).values(name.eq("Test5"));
    // match insert_query.execute(conn) {
    //     Ok(rows_changed) => println!("{} rows changed", rows_changed),
    //     Err(msg) => println!(
    //         "Query: {} \n failed with error: {}",
    //         debug_query::<diesel::mysql::Mysql, _>(&insert_query),
    //         msg
    //     ),
    // }
    // let data = quizzes
    //     .select(name)
    //     .filter(name.eq("Test4"))
    //     .load::<String>(conn)
    //     .unwrap();
    // println!(
    //     "Select statement: {} \n returned {}",
    //     debug_query::<diesel::mysql::Mysql, _>(&quizzes.select(name).filter(name.eq("Test4"))),
    //     data[0]
    // );

    let new_quiz = NewQuiz {
        name: "struct_quiz".into(),
        num_questions: 10,
    };

    insert_into(quizzes)
        .values(&new_quiz)
        .execute(conn)
        .expect("Error inserting quiz");

    quizzes
        .load::<Quiz>(conn)
        .map_err(|msg| -> String {
            println!("{}", msg);
            "Error".into()
        })
        .map(Json)
}
