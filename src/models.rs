use crate::schema::*;

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Answer {
    pub id: i32,
    pub description: String,
    pub val: i32,
    pub q_id: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingAnswer {
    pub description: String,
    pub val: i32,
}
#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "answer"]
pub struct NewAnswer {
    pub description: String,
    pub val: i32,
    pub q_id: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub qz_id: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingQuestion {
    pub description: String,
}
#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "question"]
pub struct NewQuestion {
    pub description: String,
    pub qz_id: i32,
}
#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Quiz {
    pub id: i32,
    pub name: String,
    pub num_questions: i32,
}
#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "quiz"]
pub struct NewQuiz {
    pub name: String,
    pub num_questions: i32,
}
#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct QuizResult {
    pub id: i32,
    pub num: i32,
    pub header: String,
    pub description: String,
    pub qz_id: i32,
}
#[derive(Serialize, Deserialize, Insertable, Debug)]
#[table_name = "result"]
pub struct NewQuizResult {
    pub num: i32,
    pub header: String,
    pub description: String,
    pub qz_id: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingQuizResult {
    pub num: i32,
    pub header: String,
    pub description: String,
}
