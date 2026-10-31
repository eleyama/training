//! 借用（参照）：所有権を移さずに値を貸す
//! 実行：cargo run --example 04_borrowing

fn main() {
    let name = String::from("山田");

    // & で貸す。所有権は移らないので、後でも使える
    let n = length(&name);
    println!("{name} は {n} 文字");

    // &mut で「変更してよい」として貸す
    let mut greeting = String::from("こんにちは");
    add_suffix(&mut greeting);
    println!("{greeting}");

    // ルール：読み取り用の参照（&）は何個でも、変更用の参照（&mut）は1個だけ。同時には持てない
    let r1 = &greeting;
    let r2 = &greeting;
    println!("{r1} / {r2}");
    // r1・r2 をこの後も使うなら、次の行はコンパイルエラーになる
    let w = &mut greeting;
    w.push('！');
    println!("{greeting}");

    // &str の引数には、String の参照も文字列リテラルも渡せる
    greet("佐藤");
    greet(&name);
}

fn length(s: &str) -> usize {
    s.chars().count() // len() はバイト数。文字数は chars().count()
}

fn add_suffix(s: &mut String) {
    s.push_str("、ねこさん");
}

fn greet(name: &str) {
    println!("ようこそ {name} さん");
}
