#[derive(Deserialize)]
pub struct CreateInfo {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}
