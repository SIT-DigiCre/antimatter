use mattermost_api::prelude::*;
use std::io::{BufRead, Write, stdin, stdout};

use rpassword::read_password;

fn main() {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    print!("ユーザー名 または メールアドレスを入力してください: ");
    let _ = stdout.flush();
    let mut login_id = String::new();
    stdin
        .read_line(&mut login_id)
        .expect("idの読み取りに失敗しました");
    login_id = login_id.trim_end().to_string();
    println!();

    print!("パスワードを入力してください: ");
    let _ = stdout.flush();
    let password = read_password().expect("パスワードの読み取りに失敗しました");
    println!();

    let auth = AuthenticationData::from_password(login_id, password);
    println!("{:?}", auth);
}
