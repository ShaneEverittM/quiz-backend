// password hashing
use crate::models::*;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use diesel::{self, prelude::*};
use rocket::http::Cookies;

pub fn logged_in(uid: i32, cookies: &mut Cookies) -> bool {
    let logged_in_user = cookies.get_private("user_id");
    match logged_in_user {
        Some(c) => {
            let logged_in_uid = c.value().parse::<i32>().unwrap();
            if logged_in_uid == uid {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

pub fn fetch_auth_info_by_user_id(
    conn: &diesel::MysqlConnection,
    input_uid: i32,
) -> Option<AuthInfo> {
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

pub fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}

pub fn fetch_user_by_email(conn: &diesel::MysqlConnection, input_email: &String) -> Option<User> {
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

pub fn fetch_user_by_id(conn: &diesel::MysqlConnection, input_id: i32) -> Option<User> {
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
