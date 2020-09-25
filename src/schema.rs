table! {
    answer (id) {
        id -> Integer,
        description -> Varchar,
        val -> Integer,
        q_id -> Integer,
    }
}

table! {
    auth_info (id) {
        id -> Integer,
        uid -> Integer,
        password_hash -> Text,
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
        description -> Varchar,
        u_id -> Integer,
    }
}

table! {
    result (id) {
        id -> Integer,
        num -> Integer,
        header -> Varchar,
        description -> Varchar,
        qz_id -> Integer,
    }
}

table! {
    user (id) {
        id -> Integer,
        name -> Varchar,
        email -> Varchar,
    }
}

joinable!(answer -> question (q_id));
joinable!(question -> quiz (qz_id));
joinable!(quiz -> user (u_id));
joinable!(result -> quiz (qz_id));

allow_tables_to_appear_in_same_query!(
    answer,
    auth_info,
    question,
    quiz,
    result,
    user,
);
