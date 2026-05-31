use std::{collections::HashSet, fs::File, path::Path};

use chrono::{Local, NaiveDate};
use csv::Reader;

use crate::models;

pub fn load_active_students(file_path: &Path) -> Result<HashSet<String>, std::io::Error> {
    let mut reader = Reader::from_reader(File::open(file_path)?);
    // "実行環境における"現在の日付を取得
    let today = Local::now().date_naive();
    Ok(reader
        .deserialize::<models::DBRecord>()
        .map(|r| {
            if let Ok(r) = r {
                r
            } else {
                // ここも下記のunreachableと同じ理由でunreachableです(読む側が見る順番を考えないカスのコメント)
                unreachable!("不正な行を検出しました。 {:?}", r);
            }
        })
        // active_limitが本日以降の日付であれば、有効な部員に含める
        .filter(|r| {
            match NaiveDate::parse_from_str(&r.active_limit, "%Y-%m-%d") {
                Ok(date) => date >= today,
                // そんなわけない(DBから持ってきているため、手動でCSVをいじらないかぎり、上記のフォーマットに従うはず)
                Err(_) => unreachable!("日付文字列のパースエラー: {}", r.active_limit),
            }
        })
        // これ以降active_limitを保持する必要がないので、student_numberのみを抽出
        .map(|u| u.student_number)
        // HashSetにすることで、containsの(平均)時間計算量をO(1)に
        .collect::<HashSet<_>>())
}
