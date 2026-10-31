//! 非同期処理：async / await と Tokio
//! server の #[tokio::main] と async fn の仕組み
//! 実行：cargo run --example 23_async

use std::time::{Duration, Instant};

// async fn は「後で実行される処理（Future）」を返す。.await で完了を待つ
async fn fetch(name: &str, ms: u64) -> String {
    // 待っている間、スレッドを他の処理に譲る（Thread.sleep と違い、スレッドを止めない）
    tokio::time::sleep(Duration::from_millis(ms)).await;
    format!("{name}（{ms}ms）")
}

// main を非同期にする。Tokio の実行基盤（スレッドプール）を起動する
#[tokio::main]
async fn main() {
    // 順番に待つ：合計 約300ms
    let start = Instant::now();
    let a = fetch("A", 100).await;
    let b = fetch("B", 200).await;
    println!("順番: {a}, {b} → {:?}", start.elapsed());

    // 同時に待つ：約200ms
    let start = Instant::now();
    let (a, b) = tokio::join!(fetch("A", 100), fetch("B", 200));
    println!("同時: {a}, {b} → {:?}", start.elapsed());

    // spawn：別のタスクとして実行する。サーバーは1リクエストごとにこうして処理している
    let task = tokio::spawn(fetch("C", 50));
    println!("spawn: {}", task.await.unwrap());
}
