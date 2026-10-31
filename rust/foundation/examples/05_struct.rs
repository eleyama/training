//! 構造体とメソッド（Javaのクラスに近い。継承はない）
//! 実行：cargo run --example 05_struct

#[derive(Debug)] // {:?} で中身を表示できるようにする
struct Slot {
    id: u32,
    label: String,
    capacity: u32,
}

impl Slot {
    // 関連関数：self がない。Javaの static メソッド／コンストラクタの代わり
    fn new(id: u32, label: &str) -> Self {
        Slot {
            id,
            label: label.to_string(),
            capacity: 1,
        }
    }

    // &self：読むだけのメソッド
    fn describe(&self) -> String {
        format!("[{}] {}（定員{}）", self.id, self.label, self.capacity)
    }

    // &mut self：変更するメソッド
    fn expand(&mut self, n: u32) {
        self.capacity += n;
    }
}

fn main() {
    let mut slot = Slot::new(1, "10:00-11:00");
    println!("{}", slot.describe());

    slot.expand(2); // slot が mut でないとコンパイルエラー
    println!("{}", slot.describe());

    println!("{slot:?}"); // Debug 表示
    println!("{slot:#?}"); // 整形した Debug 表示

    // フィールド名と変数名が同じなら省略して書ける
    let id = 2;
    let label = String::from("11:00-12:00");
    let other = Slot {
        id,
        label,
        capacity: 3,
    };
    println!("{}", other.describe());
}
