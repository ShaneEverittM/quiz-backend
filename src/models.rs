//use crate::schema::*;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Quiz {
    name: String,
}
