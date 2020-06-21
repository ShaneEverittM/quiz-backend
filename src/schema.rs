table! {
    answer (id) {
        id -> Integer,
        description -> Varchar,
        val -> Integer,
        q_id -> Integer,
    }
}

table! {
    question (id) {
        id -> Integer,
        description -> Varchar,
        qz_id -> Integer,
    }
}

table! {
    quiz (id) {
        id -> Integer,
        name -> Varchar,
        num_questions -> Integer,
    }
}

joinable!(answer -> question (q_id));
joinable!(question -> quiz (qz_id));

allow_tables_to_appear_in_same_query!(
    answer,
    question,
    quiz,
);
