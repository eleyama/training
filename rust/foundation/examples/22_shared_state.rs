//! 共有データ：Arc と Mutex
//! server の Arc<Mutex<Book>> と同じ仕組み（Javaの synchronized で守る共有オブジェクト）
//! 実行：cargo run --example 22_shared_state

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Arc：複数の持ち主で1つの値を共有する（参照カウント）
    // Mutex：同時に1つのスレッドだけが変更できるようにする
    let reservations = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for i in 1..=5 {
        // 中身は複製せず、持ち主の数を1つ増やす
        let shared = Arc::clone(&reservations);
        let handle = thread::spawn(move || {
            let mut list = shared.lock().unwrap(); // ロックを取る
            list.push(format!("予約{i}"));
        }); // list がスコープを抜けると、ロックが自動で外れる
        handles.push(handle);
    }

    // すべてのスレッドの終了を待つ（Javaの Thread.join）
    for handle in handles {
        handle.join().unwrap();
    }

    let list = reservations.lock().unwrap();
    println!("{} 件: {list:?}", list.len()); // 順番は実行ごとに変わる

    // Arc を使わずに reservations をそのまま move すると、
    // 1回目のスレッドに所有権が移り、2回目でコンパイルエラーになる
}
