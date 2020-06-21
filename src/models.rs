use crate::schema::*;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Answer {
    pub id: i32,
    pub description: String,
    pub val: i32,
    pub q_id: i32,
}
pub struct NewAnswer {}

#[derive(Serialize, Deserialize, Queryable)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub qz_id: i32,
}
#[derive(Serialize, Deserialize, Queryable)]
pub struct Quiz {
    pub id: i32,
    pub name: String,
    pub num_questions: i32,
}
#[derive(Insertable)]
#[table_name = "quiz"]
pub struct NewQuiz {
    pub name: String,
    pub num_questions: i32,
}
