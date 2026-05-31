use serde::Deserialize;


#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MMUser {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub roles: String,
}

#[derive(Debug, Deserialize)]
pub struct DBRecord {
    pub active_limit: String,
    pub student_number: String,
}

