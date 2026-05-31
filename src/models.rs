use std::fmt;

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
impl fmt::Display for MMUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}, ユーザー名: {}, ニックネーム: {}, メールアドレス: {}",
            self.id, self.username, self.nickname, self.email
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DBRecord {
    pub active_limit: String,
    pub student_number: String,
}
