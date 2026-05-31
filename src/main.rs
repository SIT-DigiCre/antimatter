use mattermost_api::prelude::*;
use std::{
    error::Error,
    io::{BufRead, Write, stdin, stdout}, path::PathBuf,
};
use clap::Parser;

use rpassword::read_password;

mod models;
mod mattermost;
mod csv;

/// -----------------------------------------------
/// デジクリ 退部者Mattermostアカウント無効化ツール
///
///                    Antimatter
/// -----------------------------------------------
#[derive(Parser)]
#[command(version, verbatim_doc_comment)]
struct Args {
    /// デジコアDBからダンプしてきた、部員データが格納されたCSVへのパス(README.mdを参照のこと)
    input_csv: PathBuf,
    #[arg(long, help_heading = "dry-runモード", help="dry-runモードを使用する\n実際のアカウント無効化を実行せずに、DMにて該当者への告知を行う")]
    dm: bool,
    /// DMに送信する文面。改行はがんばってください。
    #[arg(long, help_heading = "dry-runモード", default_value="これはテスト用文字列です。正式に文章が確定したら置き換えてください。")]
    dm_text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let active_student_numbers = csv::load_active_students(
        &args.input_csv
    )?;
    println!(
        "CSVを読み込みました。
有効な部員の総数は{}です。\n",
        active_student_numbers.len()
    );

    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();

    print!("操作を行うシステム管理者として、ユーザー名 または メールアドレスを入力してください: ");
    let _ = stdout.flush();
    let mut login_id = String::new();
    stdin
        .read_line(&mut login_id)
        .expect("IDの読み取りに失敗しました");
    login_id = login_id.trim_end().to_string();
    println!();

    print!("パスワードを入力してください: ");
    let _ = stdout.flush();
    let password = read_password().expect("パスワードの読み取りに失敗しました");
    println!();

    let auth = AuthenticationData::from_password(login_id, password);
    let mut api = Mattermost::new("https://mm.digicre.net", auth)?;
    api.store_session_token().await?;

    println!(
        "デジクリ Mattermostサーバへのセッションを確立しました。
ユーザー一覧を取得します。\n"
    );

    let users = mattermost::fetch_all_active_users(&api).await.expect("ユーザ一覧の取得に失敗しました。");

    println!("ユーザ一覧を取得しました。");
    // emailの文字列が"@shibaura-it.ac.jp"で終わるかどうかで、normalとabnormalに分ける。
    let (normal_users, abnormal_users): (Vec<_>, _) = users
        .into_iter()
        .partition(|u| u.email.ends_with("@shibaura-it.ac.jp"));

    if !abnormal_users.is_empty() {
        println!("@shibaura-it.ac.jp以外のドメインで登録しているユーザの一覧を表示します。\n");
        abnormal_users.iter().for_each(|u| println!("{:?}", u));
        // 後で、それぞれのアカウントについて個別に削除するか確認する画面でも用意する？
        // というか、printlnでこれをやるのはあんまりなぁ…
        println!("管理画面等で、個別に対応を行ってください。\n");
    }

    let suspend_list = normal_users
        .iter()
        .filter(|user| {
            // 上のpartitionにおける条件から、"@shibaura-it.ac.jp"で終わることが保証されているので、email内に"@"を含むことが保証できる
            let (student_number, _) = user.email.split_once("@").unwrap();
            // メールアドレスを"@"で2つに分割したうち先頭の方(ユーザー名, 大学のメールアドレスであれば学籍番号と同一)が、
            // active_student_numbersに含まれてい"ない"ユーザを抽出する。
            !active_student_numbers.contains(student_number)
        })
        .collect::<Vec<_>>();
    println!("無効化対象者の一覧\n");
    suspend_list.iter().for_each(|u| {
        println!(
            "ユーザー名: {}, ニックネーム: {}, 学籍番号: {}",
            u.username,
            u.nickname,
            u.email.split_once("@").unwrap().0
        )
    });
    println!("実行しますか？(y/n): ");
    let _= stdout.flush();

    Ok(())
}
