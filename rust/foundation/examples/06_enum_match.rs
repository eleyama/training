//! enum と match（Javaのenumより強力で、種類ごとに値を持てる）
//! 実行：cargo run --example 06_enum_match

enum Status {
    Available,
    Reserved(String),          // 予約者名を持つ
    Closed { reason: String }, // 名前付きの値を持つ
}

fn describe(status: &Status) -> String {
    // match はすべての種類を書かないとコンパイルエラーになる（書き忘れを防げる）
    match status {
        Status::Available => "空き".to_string(),
        Status::Reserved(name) => format!("{name} 様が予約済み"),
        Status::Closed { reason } => format!("休止中（{reason}）"),
    }
}

fn main() {
    let list = vec![
        Status::Available,
        Status::Reserved("山田".to_string()),
        Status::Closed {
            reason: "点検".to_string(),
        },
    ];
    for status in &list {
        println!("{}", describe(status));
    }

    // if let：1つの種類だけ確認したいとき
    if let Status::Reserved(name) = &list[1] {
        println!("予約者は {name}");
    }

    // match は数値や範囲にも使える。_ は「それ以外」
    let hour = 13;
    let part = match hour {
        0..=11 => "午前",
        12 => "正午",
        _ => "午後",
    };
    println!("{hour}時は{part}");
}
