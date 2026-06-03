use std::fmt::{self, Write as _};

use serde::Deserialize;
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug, Deserialize, Clone)]
#[expect(unused)]
pub struct MMUser {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub roles: String,
}
impl fmt::Display for MMUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Pad<'a>(&'a str, usize);
        impl fmt::Display for Pad<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let width = self.0.width_cjk();
                let padding = self.1.saturating_sub(width);
                f.write_str(self.0)?;
                for _ in 0..padding {
                    f.write_char(' ')?;
                }
                Ok(())
            }
        }
        write!(
            f,
            "ユーザー名: {}, ニックネーム: {}, メールアドレス: {}",
            Pad(&self.username, 25),
            Pad(&self.nickname, 20),
            Pad(&self.email, 20),
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
