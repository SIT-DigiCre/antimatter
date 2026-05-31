[![SUSHI-WARE LICENSE](https://img.shields.io/badge/license-SUSHI--WARE%F0%9F%8D%A3-blue.svg)](https://github.com/MakeNowJust/sushi-ware)

# Antimatter

## 事前準備

user_full_profiles(ビュー)を作成しておく。\
ビューの作成方法については、NotionのSysDev>知見 内 部員名簿の作成 か、\
[部員名簿を半自動で作成するやつ](https://github.com/SIT-DigiCre/gen_student_list)のREADME.mdに従うこと。

デジコアのDB上で、\
``SELECT active_limit, student_number FROM `user_full_profiles` WHERE is_member = 1 AND is_graduated = 0 AND active_limit >= CURRENT_TIMESTAMP;``\
を実行する。\
"CSV"としてエクスポートする。

## 実行

nixがなくとも、cargoが入っていればなんとかなります\
まずは`cargo run -- --help`とかで説明を読んでください。\
次に指示に従って実行。\
やさしい人は`cargo build -r`して、ビルド成果物を後輩に引き継ぐなりなんなりしてあげてください\
実行すると認証情報を聞かれたりするので、ノリでなんとかしましょう。

## おわり

おわりだよ

