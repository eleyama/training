//! モジュール（Javaの package）と公開範囲（pub）
//! 実行：cargo run --example 11_modules
//!
//! ここでは1ファイルの中に書く。別ファイルに分けるときは
//! `mod booking;` と書き、中身を booking.rs に置く

mod booking {
    // pub がないものは、モジュールの外から見えない（既定は private）
    pub struct Book {
        pub title: String,
        count: u32,
    }

    impl Book {
        pub fn new(title: &str) -> Self {
            Book {
                title: title.to_string(),
                count: 0,
            }
        }

        pub fn add(&mut self) {
            self.count += 1;
        }

        pub fn count(&self) -> u32 {
            self.count
        }
    }

    pub mod util {
        pub fn hello() -> &'static str {
            "booking::util::hello" // 文字列リテラルはプログラム終了まで有効（'static）
        }
    }
}

// use：長い名前を短く呼べるようにする（Javaの import）
use booking::Book;
use booking::util;

fn main() {
    let mut book = Book::new("会議室");
    book.add();
    // book.count = 5; // ← コンパイルエラー。count は private
    println!("{} の予約数 = {}", book.title, book.count());
    println!("{}", util::hello());
}
