//! コレクション（Vec・HashMap）とイテレーター
//! 実行：cargo run --example 09_collections

use std::collections::HashMap;

fn main() {
    // Vec：可変長の配列（Javaの ArrayList）
    let mut hours = vec![10, 11, 13];
    hours.push(14);
    println!("{hours:?} 件数 = {}", hours.len());

    // イテレーター（Javaの Stream に近い）
    // iter() は要素への参照（&i32）を順に渡す
    let labels: Vec<String> = hours.iter().map(|h| format!("{h}:00")).collect();
    println!("{labels:?}");

    // filter には「参照の参照」が渡るため、** で値を取り出す
    let afternoon: Vec<&i32> = hours.iter().filter(|h| **h >= 12).collect();
    println!("午後 = {afternoon:?}");

    let has_ten = hours.contains(&10); // 値が含まれるか
    let has_evening = hours.iter().any(|h| *h >= 17); // 1つでも条件に合うか
    let first_after_11 = hours.iter().find(|h| **h > 11); // 最初の1つ（Option）
    let total: i32 = hours.iter().sum();
    println!("10時あり = {has_ten}, 17時以降あり = {has_evening}");
    println!("11時より後 = {first_after_11:?}, 合計 = {total}");

    // retain：条件に合う要素だけ残す（removeIf の逆）
    hours.retain(|h| *h != 11);
    println!("11時を削除 = {hours:?}");

    // HashMap（Javaの HashMap）。表示順は決まっていない
    let mut reservations: HashMap<u32, String> = HashMap::new();
    reservations.insert(10, "山田".to_string());
    reservations.insert(13, "佐藤".to_string());

    match reservations.get(&10) {
        Some(name) => println!("10時は {name}"),
        None => println!("10時は空き"),
    }
    reservations.remove(&13);
    for (hour, name) in &reservations {
        println!("{hour}時: {name}");
    }

    // entry：キーがなければ追加し、あれば更新する
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for name in ["山田", "佐藤", "山田"] {
        *counts.entry(name).or_insert(0) += 1;
    }
    println!("{counts:?}");
}
