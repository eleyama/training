//! serde：構造体と JSON・フォームの相互変換（Javaの Jackson 相当）
//! server の Form<ReserveForm> はこの仕組みでフォームを構造体に変換している
//! 実行：cargo run --example 24_serde

use serde::{Deserialize, Serialize};

// derive するだけで変換できるようになる
#[derive(Debug, Serialize, Deserialize)]
struct ReserveForm {
    slot_id: u32,
    name: String,
}

fn main() {
    // r#"..."# は生文字列。中の " をエスケープしなくてよい
    let json = r#"{"slot_id": 1, "name": "山田"}"#;

    let form: ReserveForm = serde_json::from_str(json).unwrap();
    println!("JSON → 構造体: {form:?}");

    let back = serde_json::to_string(&form).unwrap();
    println!("構造体 → JSON: {back}");

    // 項目が足りない・型が違う場合は Err になる
    let bad = r#"{"slot_id": "abc"}"#;
    match serde_json::from_str::<ReserveForm>(bad) {
        Ok(form) => println!("{form:?}"),
        Err(e) => println!("変換失敗: {e}"),
    }
}
