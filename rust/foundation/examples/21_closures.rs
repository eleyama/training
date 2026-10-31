//! クロージャ（Javaのラムダ式）
//! 予約システムでは filter・any・find・retain・unwrap_or_else に使っている
//! 実行：cargo run --example 21_closures

fn main() {
    let double = |x: i32| x * 2;
    println!("double(4) = {}", double(4));

    // 外の変数を取り込める
    let limit = 12;
    let is_afternoon = |hour: u32| hour >= limit;
    println!("13時は午後？ {}", is_afternoon(13));

    // 条件をクロージャで渡す（core の available_slots と同じ形）
    let reserved = [1, 3];
    let slots = [1, 2, 3, 4];
    let available: Vec<u32> = slots
        .iter()
        .copied() // &u32 を u32 に変換
        .filter(|id| !reserved.contains(id))
        .collect();
    println!("空き = {available:?}");

    // 関数の引数にクロージャを受け取る
    println!("apply = {}", apply(5, |n| n + 100));

    // 環境変数がなければ既定値（server の main と同じ形）
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    println!("port = {port}");

    // move：変数の所有権ごと取り込む。スレッドに渡すときに必要（22で使う）
    let name = String::from("山田");
    let greet = move || format!("こんにちは {name}");
    println!("{}", greet());
}

// impl Fn(i32) -> i32：「i32 を受け取り i32 を返す関数」なら何でもよい
fn apply(n: i32, f: impl Fn(i32) -> i32) -> i32 {
    f(n)
}
