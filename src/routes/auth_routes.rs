use diesel::{self, prelude::*};

use rocket::http::{Cookie, Cookies};
use rocket_contrib::json::Json; // Easy Json coercion

use super::auth_functions::*;
use super::auth_types::*;
use crate::models::auth_models::*; // Models needed for pulling or pushing data
use crate::utils::sql_utils::last_insert_id; //utility for getting around mysql being bad
use crate::DbConn; // The state managed DB connection

#[post("/users/create", format = "json", data = "<create_info>")]
pub fn create(conn_ptr: DbConn, create_info: Json<CreateInfo>) -> Json<i32> {
    use crate::schema::auth_info::dsl::auth_info as auth_info_table;
    use crate::schema::user::dsl::user as user_table;
    let user = NewUser {
        name: create_info.name.clone(),
        email: create_info.email.clone(),
    };
    let ref conn = *conn_ptr;
    let _rows_changed = diesel::insert_into(user_table)
        .values(user)
        .execute(conn)
        .expect("error");

    let password_hash = hash_password(&create_info.password);
    let last_uid: u64 = diesel::select(last_insert_id).first(conn).expect("Error");
    let auth_info = NewAuthInfo {
        uid: last_uid as i32,
        password_hash: password_hash,
    };
    let _rows_changed = diesel::insert_into(auth_info_table)
        .values(auth_info)
        .execute(conn)
        .expect("Error");
    Json(last_uid as i32)
}
#[post("/users/login", format = "json", data = "<login_info>")]
pub fn login(
    conn_ptr: DbConn,
    login_info: Json<LoginInfo>,
    mut cookies: Cookies,
) -> Json<Option<User>> {
    let ref conn = *conn_ptr;
    let user_opt = fetch_user_by_email(conn, &login_info.username);
    match user_opt {
        Some(user) => {
            let auth_opt = fetch_auth_info_by_user_id(conn, user.id);
            match auth_opt {
                Some(auth_info) => {
                    let hash = hash_password(&login_info.password);
                    if hash == auth_info.password_hash {
                        cookies.add_private(Cookie::new("user_id", user.id.to_string()));
                        Json(fetch_user_by_id(conn, user.id))
                    } else {
                        Json(None)
                    }
                }
                None => Json(None),
            }
        }
        None => Json(None),
    }
}

#[post("/users/logout", format = "json")]
pub fn logout(mut cookies: Cookies) -> () {
    cookies.remove_private(Cookie::named("user_id"));
}

#[get("/users/cookies/<uid>")]
pub fn fetch_info_by_user_id(
    conn_ptr: DbConn,
    uid: i32,
    mut cookies: Cookies,
) -> Json<Option<User>> {
    let ref conn = *conn_ptr;
    if logged_in(uid, &mut cookies) {
        Json(fetch_user_by_id(conn, uid))
    } else {
        Json(None)
    }
}
