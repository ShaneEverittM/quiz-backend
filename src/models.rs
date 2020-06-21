use crate::schema::*;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Quiz {
    pub name: String,
    pub num_questions: i32,
    pub id: i32,
}
#[derive(Insertable)]
#[table_name = "quizzes"]
pub struct NewQuiz {
    pub name: String,
    pub num_questions: i32,
}
