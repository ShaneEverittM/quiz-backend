use crate::schema::*;

/* -------------------------------------------------------------------------- */
/*        Models for query results, analagous to the records in the db.       */
/* -------------------------------------------------------------------------- */

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Answer {
    pub id: i32,
    pub description: String,
    pub val: i32, // value used for determining overall result pub q_id: i32,
    pub q_id: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub qz_id: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Quiz {
    pub id: i32,
    pub name: String,
    pub num_questions: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct QuizResult {
    pub id: i32,
    pub num: i32, // the corresponding field to 'val' in Answer. 'val' is used to calculate which result 'num'.
    pub header: String,
    pub description: String,
    pub qz_id: i32,
}

/* -------------------------------------------------------------------------- */
/*         Models for data to be inserted. Adds calculated db fields.         */
/* -------------------------------------------------------------------------- */

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "answer"]
pub struct NewAnswer {
    pub description: String,
    pub val: i32,
    pub q_id: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "question"]
pub struct NewQuestion {
    pub description: String,
    pub qz_id: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "quiz"]
pub struct NewQuiz {
    pub name: String,
    pub num_questions: i32,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[table_name = "result"]
pub struct NewQuizResult {
    pub num: i32,
    pub header: String,
    pub description: String,
    pub qz_id: i32,
}
