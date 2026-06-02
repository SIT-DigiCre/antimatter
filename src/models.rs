use std::fmt;

use serde::Deserialize;
use unicode_width::UnicodeWidthStr;

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
        fn pad(s: &String, len: usize) -> String {
            s.to_owned() + &" ".repeat(len.saturating_sub(s.width_cjk()))
        }
        write!(
            f,
            "ユーザー名: {}, ニックネーム: {}, メールアドレス: {}",
            pad(&self.username, 25),
            pad(&self.nickname, 20),
            pad(&self.email, 20),
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
