//! トレイト（Javaの interface）とジェネリクス
//! 実行：cargo run --example 10_traits

use std::fmt;

trait Describe {
    fn describe(&self) -> String;

    // 既定の実装も持てる（Javaの default メソッド）
    fn shout(&self) -> String {
        format!("{}！", self.describe())
    }
}

struct Room {
    name: String,
}

struct Desk {
    number: u32,
}

impl Describe for Room {
    fn describe(&self) -> String {
        format!("会議室{}", self.name)
    }
}

impl Describe for Desk {
    fn describe(&self) -> String {
        format!("座席{}番", self.number)
    }
}

// 標準のトレイト Display を実装すると {} で表示できる（toString 相当）
impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Room({})", self.name)
    }
}

// ジェネリクス：Describe を実装した型なら何でも受け取る
fn print_item(item: &impl Describe) {
    println!("{}", item.describe());
}

// Into<String>：&str と String のどちらでも受け取れる
fn make_room(name: impl Into<String>) -> Room {
    Room { name: name.into() }
}

// derive：よく使うトレイトを自動で実装する
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let a = make_room("A");
    let b = make_room(String::from("B"));
    let d = Desk { number: 3 };

    print_item(&a);
    print_item(&d);
    println!("{}", b.shout());
    println!("{a}");

    // 違う型をまとめるときは Box<dyn トレイト>（Javaの List<Describe> 相当）
    let items: Vec<Box<dyn Describe>> = vec![Box::new(b), Box::new(d)];
    for item in &items {
        println!("- {}", item.describe());
    }

    let p = Point { x: 1, y: 2 };
    let q = p.clone(); // Clone
    println!("{p:?} == {q:?} : {}", p == q); // Debug と PartialEq
}
