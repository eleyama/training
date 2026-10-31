//! 変数と基本の型
//! 実行：cargo run --example 01_variables

fn main() {
    // let は既定で変更不可（Javaの final 相当）
    let x = 5;
    // x = 6; // ← コンパイルエラー。変更するには mut が必要
    let mut count = 0;
    count += 1;
    println!("x = {x}, count = {count}");

    // 型は推論される。明示もできる
    let price: u32 = 1200; // 符号なし32ビット整数
    let rate: f64 = 0.1; // 浮動小数点
    let ok: bool = true;
    let initial: char = 'ね'; // 1文字（Unicode）
    println!("{price} {rate} {ok} {initial}");

    // 整数と小数は自動で変換されない。as で明示する
    let tax = price as f64 * rate;
    println!("税 = {tax}");

    // シャドーイング：同じ名前で新しい変数を作る（型が変わってもよい）
    let input = "42";
    let input: i32 = input.parse().unwrap();
    println!("input + 1 = {}", input + 1);

    // 文字列は2種類
    let borrowed: &str = "借りている文字列"; // 文字列リテラル。変更不可
    let mut owned: String = String::from("所有する文字列"); // 変更できる
    owned.push_str("です");
    owned.push('！'); // 1文字なら push
    println!("{borrowed} / {owned}");

    // タプルと配列
    let pair = (1, "山田");
    let hours = [10, 11, 13, 14];
    println!(
        "{} {} / 先頭 {} / 件数 {}",
        pair.0,
        pair.1,
        hours[0],
        hours.len()
    );
}
