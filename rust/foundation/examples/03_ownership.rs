//! 所有権：値の持ち主は常に1人。持ち主がいなくなると自動で解放される（GCはない）
//! 実行：cargo run --example 03_ownership

fn main() {
    // 1. 代入すると所有権が移る（move）
    let a = String::from("山田");
    let b = a;
    // println!("{a}"); // ← コンパイルエラー。a の中身は b に移った
    println!("b = {b}");

    // 2. clone で複製すれば両方使える
    let c = b.clone();
    println!("b = {b}, c = {c}");

    // 3. 関数に渡しても所有権が移る
    take(c);
    // println!("{c}"); // ← コンパイルエラー

    // 4. 整数・bool・char などは Copy 型。代入しても元が使える
    let x = 1;
    let y = x;
    println!("x = {x}, y = {y}");

    // 5. 関数から返すと、所有権が呼び出し側へ移る
    let made = make_name();
    println!("made = {made}");
} // ここで b と made が解放される

fn take(name: String) {
    println!("受け取った: {name}");
} // ここで name が解放される

fn make_name() -> String {
    String::from("佐藤")
}
