use clap::Parser;
use inquire::{Confirm, MultiSelect, Password, Text};
use itertools::Itertools;
use mattermost_api::{models::PostBody, prelude::*};
use std::{error::Error, path::PathBuf};

use crate::models::MMUser;

mod csv;
mod mattermost;
mod models;

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
    #[arg(
        long,
        help_heading = "告知モード",
        help = "告知モードで実行します。アカウント無効化を実行せずに、DMにて該当者への告知を行うモードです。"
    )]
    dm: bool,
    /// DMに送信する文面。改行はがんばってください。
    #[arg(
        long,
        requires = "dm",
        help_heading = "告知モード",
        default_value_t = String::from("
# 【デジクリ】退部処理のお知らせ

※このDMは、現在デジクリへの在籍が確認できない方に送信しています。
デジクリ　インフラ担当です。
このDMを受け取った方につきましては、6/9 0:00を持ちまして、Discordサーバ, Mattermostへのアクセス権を無効化させていただきます。
何かご不明な点・異議申し立てがありましたら、[インフラサポートチケット](https://forms.gle/a1jWvcvosP6jAdfM9)、もしくはデジクリ メールアドレス(contact@digicre.net)までお知らせください。")
    )]
    dm_text: String,
    /// 接続するMattermostサーバのアドレス
    #[arg(long, default_value_t=String::from("https://mm.digicre.net"))]
    server_addr: String,
    /// メールアドレスのドメインを指定する
    #[arg(long, default_value_t=String::from("shibaura-it.ac.jp"))]
    domain: String,
    /// Botアカウントではなく、あなたのアカウントで操作を実行します。DM送信も。IDとパスワードでログインできるので楽です。
    #[arg(long)]
    with_my_account: bool,
}

fn filter_users(users: Vec<MMUser>, domain: &str) -> (Vec<MMUser>, Vec<MMUser>, Vec<MMUser>) {
    // Botアカウントの類については、emailが@localhostになっているのでそれを見る
    let (bots, users): (Vec<_>, _) = users
        .into_iter()
        .partition(|u| u.email.ends_with("@localhost"));

    // emailの文字列が"@<ドメイン>"で終わるかどうかで、normalとabnormalに分ける。
    let (normal_users, abnormal_users): (Vec<_>, _) =
        users.into_iter().partition(|u| u.email.ends_with(domain));

    (bots, normal_users, abnormal_users)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let active_student_numbers = csv::load_active_students(&args.input_csv)?;
    println!("\nCSVを読み込みました。\n");
    println!("有効な部員の総数は{}です。\n", active_student_numbers.len());

    let auth = if args.with_my_account {
        let login_id = Text::new(
            "操作を行うシステム管理者として、ユーザー名 または メールアドレスを入力してください: ",
        )
        .prompt()?;

        let password = Password::new("パスワードを入力してください:")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()?;

        AuthenticationData::from_password(login_id, password)
    } else {
        AuthenticationData::from_access_token(
            Password::new("アクセストークンを入力してください。")
                .without_confirmation()
                .prompt()?,
        )
    };

    let mut api = Mattermost::new(args.server_addr, auth)?;
    if args.with_my_account {
        api.store_session_token().await?;
    }

    println!("\nMattermostサーバとのセッションを確立しました。\n");
    let me = mattermost::get_my_info(&api)
        .await
        .expect("ログイン中のユーザ情報の取得に失敗しました。");
    println!("ログイン中のユーザー情報: {me:#?}\n");
    println!("ユーザー一覧を取得します。\n");

    let users = mattermost::fetch_all_active_users(&api)
        .await
        .expect("ユーザ一覧の取得に失敗しました。");

    println!("ユーザ一覧を取得しました。");
    let domain = format!("@{}", args.domain);

    let (bots, normal_users, abnormal_users) = filter_users(users, &domain);

    if !bots.is_empty() {
        println!(
            "\n以下のアカウントについては、メールアドレスのドメインがlocalhostであるため除外します。\n"
        );
        println!("{}", bots.iter().format("\n"));
        println!("\n---------------------------------------------------");
    }

    let mut suspend_list = Vec::new();
    if !abnormal_users.is_empty() {
        println!(
            "{}以外のドメインで登録しているユーザが存在します。\n",
            args.domain
        );
        suspend_list.extend(loop {
            let suspend_list = MultiSelect::new(
                "この中に無効化すべきアカウントが存在する場合、選択してください。",
                abnormal_users.clone(),
            )
            .with_page_size(20)
            .prompt()?;
            println!("\n--------------------------------------------------");
            for u in &suspend_list {
                println!("{u:#?}");
            }
            if Confirm::new("以上のアカウントを選択しますか？")
                .with_default(false)
                .prompt()?
            {
                break suspend_list;
            }
        });
    }

    let suspend_list_norm = normal_users
        .into_iter()
        .filter(|user| {
            // 上のpartitionにおける条件から、"@<ドメイン>"で終わることが保証されているので、email内に"@"を含むことが保証できる
            let (student_number, _) = user.email.split_once('@').unwrap();
            // メールアドレスを"@"で2つに分割したうち先頭の方(ユーザー名, 大学のメールアドレスであれば学籍番号と同一)が、
            // active_student_numbersに含まれてい"ない"ユーザを抽出する。
            !active_student_numbers.contains(student_number)
        })
        .collect::<Vec<_>>();
    suspend_list.extend(loop {
        let suspend_list = MultiSelect::new(
            "無効化すべきでないアカウントが存在する場合、チェックを外してください。",
            suspend_list_norm.clone(),
        )
        .with_default(&(0..suspend_list_norm.len()).collect::<Vec<_>>())
        .with_page_size(20)
        .prompt()?;
        println!("\n--------------------------------------------------");
        for u in &suspend_list {
            println!("{u}");
        }
        if Confirm::new("以上のアカウントを選択しますか？")
            .with_default(false)
            .prompt()?
        {
            break suspend_list;
        }
    });
    println!("\n--------------------------------------------------\n");
    println!("無効化対象者の一覧\n");
    for u in &suspend_list {
        println!(
            "ユーザー名: {}, ニックネーム: {}, 学籍番号?: {}",
            u.username,
            u.nickname,
            // abnormalに、まともなメールアドレスを持っていないユーザが存在する可能性を捨て切れない
            u.email.split_once('@').unwrap_or(("None", "")).0
        );
    }

    println!("\n対象の総数: {}\n", suspend_list.len());
    if args.dm {
        println!("DM送信を実行します。");
        if args.with_my_account {
            println!("!!Botアカウント(トークン)を利用せずに実行しています!!");
            println!("!!あなたのアカウントでDMを送信することになります!!");
        }
        if !Confirm::new("続行しますか？")
            .with_default(false)
            .prompt()?
        {
            return Ok(());
        }

        let mut body = PostBody {
            channel_id: "a".into(),
            message: args.dm_text,
            root_id: None,
        };
        for user in suspend_list {
            let ids = [&me.id, &user.id];
            match mattermost::get_or_create_dm_channel_id(&api, &ids).await {
                Ok(channel) => {
                    body.channel_id = channel.id;
                    if let Err(e) = api.create_post(&body).await {
                        eprintln!("------------------------");
                        eprintln!("{e:?}");
                        eprintln!("{user}へのDM送信に失敗しました。");
                    }
                }
                Err(e) => {
                    eprintln!("------------------------");
                    eprintln!("{e:?}");
                    eprintln!("{user}とのDMチャンネルの作成, IDの取得に失敗しました。");
                }
            }
        }
    } else {
        println!("アカウント無効化処理を実行します。");
        if !Confirm::new("続行しますか？")
            .with_default(false)
            .prompt()?
        {
            return Ok(());
        }
        for user in suspend_list {
            if let Err(e) = mattermost::set_user_inactive(&api, &user.id).await {
                eprintln!("-----------------------------");
                eprintln!("{e:?}");
                eprintln!("{user}の無効化に失敗しました。");
            }
        }
    }
    Ok(())
}
