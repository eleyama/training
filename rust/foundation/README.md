# Rust基礎練習

Rustの基本的な動作を試すためのフォルダ。テーマごとに `examples/` に1ファイルずつ置いている。

```powershell
cargo run --example 03_ownership    # 例を実行
cargo test --example 12_testing     # テストを実行
cargo run                           # src/main.rs（自由に試す場所）
```

コメントアウトしてある「← コンパイルエラー」の行を戻すと、コンパイラのエラー文を確認できる。

## 1. 根本的な基礎

| ファイル | 内容 | Javaでいうと |
| --- | --- | --- |
| 01_variables | let・mut・型・as・シャドーイング・String と &str | final、プリミティブ型 |
| 02_functions | 関数・戻り値・if 式・for・while・loop | メソッド、制御構文 |
| 03_ownership | 所有権の移動・clone・Copy 型・自動解放 | （対応なし。GCの代わり） |
| 04_borrowing | 参照 & と &mut・借用のルール | （対応なし） |
| 05_struct | 構造体・impl・&self と &mut self・関連関数 | クラス、static メソッド |
| 06_enum_match | 値を持つ enum・match・if let | enum、switch |
| 07_option | Option・Some/None・unwrap_or_else・map | null、Optional |
| 08_result | Result・Ok/Err・?・独自エラー型・Display | 例外、throws |
| 09_collections | Vec・HashMap・iter/map/filter/any/find/sum/retain | ArrayList、HashMap、Stream |
| 10_traits | トレイト・既定実装・impl Trait・dyn・derive・Into | interface、default メソッド |
| 11_modules | mod・pub・use | package、public、import |
| 12_testing | #[test]・assert!・assert_eq!・should_panic | JUnit |

## 2. 予約システムを動かすための基礎

| ファイル | 内容 | 予約システムでの使用箇所 |
| --- | --- | --- |
| 21_closures | クロージャ・外の変数の取り込み・move | core の filter/map、server の unwrap_or_else |
| 22_shared_state | Arc・Mutex・スレッド | server の `Arc<Mutex<Connection>>` |
| 23_async | async/await・Tokio・join!・spawn | server の `#[tokio::main]`・`async fn` |
| 24_serde | 構造体と JSON の相互変換・derive(Deserialize) | server の `Form<ReserveForm>` |
| 25_axum_hello | axum の最小サーバー（GET/POST・State・Form・Redirect） | server/src/main.rs 全体 |

25 は http://127.0.0.1:3001/ で起動する（予約システムの 3000 番と重ならない）。

## おすすめの順番

01 → 08 を順に読んで実行する。core（rust/reservation/core/src/lib.rs）はここまでの知識で読める。
21 → 25 を読むと server（rust/reservation/server/src/main.rs）が読める。
