//! Option：値が「ある（Some）」か「ない（None）」か。Rustには null がない
//! 実行：cargo run --example 07_option

fn find_name(id: u32) -> Option<String> {
    match id {
        1 => Some("山田".to_string()),
        2 => Some("佐藤".to_string()),
        _ => None,
    }
}

fn main() {
    // match で両方の場合を書く
    match find_name(1) {
        Some(name) => println!("見つかった: {name}"),
        None => println!("いない"),
    }

    // if let：ある場合だけ処理する
    if let Some(name) = find_name(2) {
        println!("id=2 は {name}");
    }

    // ない場合の既定値（|| ... はクロージャ。ない場合だけ実行される）
    let name = find_name(9).unwrap_or_else(|| "未登録".to_string());
    println!("id=9 は {name}");

    // map：ある場合だけ中身を変換する
    let len = find_name(1).map(|n| n.chars().count());
    println!("文字数 = {len:?}"); // Some(2)

    println!("id=9 はいない？ {}", find_name(9).is_none());

    // unwrap：None なら異常終了（panic）。確実にある場合とテストでだけ使う
    let sure = find_name(1).unwrap();
    println!("unwrap = {sure}");
}
