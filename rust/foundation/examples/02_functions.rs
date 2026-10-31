//! 関数と制御構文（if・for・while・loop）
//! 実行：cargo run --example 02_functions

fn main() {
    println!("2 + 3 = {}", add(2, 3));
    println!("{}", label(10));
    println!("{}", label(15));

    // if は式なので値を返せる（三項演算子の代わり）
    let n = 7;
    let kind = if n % 2 == 0 { "偶数" } else { "奇数" };
    println!("{n} は{kind}");

    // for：範囲。10..13 は 10, 11, 12（13 は含まない）
    for hour in 10..13 {
        println!("{hour}時");
    }

    // for：配列。enumerate で番号も取れる
    let names = ["山田", "佐藤"];
    for (i, name) in names.iter().enumerate() {
        println!("{i}: {name}");
    }

    // while
    let mut left = 3;
    while left > 0 {
        left -= 1;
    }
    println!("left = {left}");

    // loop は無限ループ。break で値を返せる
    let mut tries = 0;
    let result = loop {
        tries += 1;
        if tries == 3 {
            break tries * 10;
        }
    };
    println!("result = {result}");
}

fn add(a: i32, b: i32) -> i32 {
    a + b // 最後の式（セミコロンなし）が戻り値。return は不要
}

fn label(hour: u32) -> String {
    if hour < 12 {
        return format!("午前{hour}時"); // 途中で返すときは return
    }
    format!("午後{}時", hour - 12)
}
