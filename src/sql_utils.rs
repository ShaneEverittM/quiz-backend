// no_arg_sql_function!(function_name, return_type)
// Generates a FFI of a specific signature for db_name.function_name()
// In this case its quizzes_db.last_insert_id() -> sql::BigInt
no_arg_sql_function!(
    last_insert_id,
    diesel::sql_types::Unsigned<diesel::sql_types::BigInt>
);
