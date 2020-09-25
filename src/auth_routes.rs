use diesel::{self, prelude::*};

use rocket::http::{Cookie, Cookies};
use rocket_contrib::json::Json; // Easy Json coercion

// password hashing
use crypto::digest::Digest;
use crypto::sha3::Sha3;

use crate::models::*; // Models needed for pulling or pushing data
use crate::sql_utils::last_insert_id; //utility for getting around mysql being bad
use crate::DbConn; // The state managed DB connection

#[derive(Deserialize)]
pub struct CreateInfo {
    name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

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
) -> Json<Option<i32>> {
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
                        Json(Some(user.id))
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

#[post("/users/logout", format="json")]
pub fn logout(mut cookies: Cookies) -> () {
    cookies.remove_private(Cookie::named("user_id"));

}

#[get("/users/cookies/<uid>")]
pub fn fetch_info_by_user_id(
    conn_ptr: DbConn,
    uid: i32,
    mut cookies: Cookies,
) -> Json<Option<User>> {
    let logged_in_user = cookies.get_private("user_id");
    let ref conn = *conn_ptr;
    match logged_in_user {
        Some(c) => {
            let logged_in_uid = c.value().parse::<i32>().unwrap();
            if logged_in_uid == uid {
                Json(fetch_user_by_id(conn, uid))
            } else {
                Json(None)
            }
        }
        None => Json(None),
    }
}

fn fetch_auth_info_by_user_id(conn: &diesel::MysqlConnection, input_uid: i32) -> Option<AuthInfo> {
    use crate::schema::auth_info::dsl::*;
    let mut auth_infos_by_uid: Vec<AuthInfo> = auth_info
        .filter(uid.eq(input_uid))
        .load::<AuthInfo>(conn)
        .expect("Error");
    if auth_infos_by_uid.len() == 0 {
        None
    } else {
        let first = auth_infos_by_uid.remove(0);
        Some(first)
    }
}

fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}

fn fetch_user_by_email(conn: &diesel::MysqlConnection, input_email: &String) -> Option<User> {
    use crate::schema::user::dsl::*;
    let mut users_by_id: Vec<User> = user
        .filter(email.eq(input_email))
        .load::<User>(conn)
        .expect("Error");
    if users_by_id.len() == 0 {
        None
    } else {
        let first = users_by_id.remove(0);
        Some(first)
    }
}

fn fetch_user_by_id(conn: &diesel::MysqlConnection, input_id: i32) -> Option<User> {
    use crate::schema::user::dsl::*;
    let mut users_by_id: Vec<User> = user
        .filter(id.eq(input_id))
        .load::<User>(conn)
        .expect("Error");
    if users_by_id.len() == 0 {
        None
    } else {
        Some(users_by_id.remove(0))
    }
}
