//! テスト（JUnit相当）
//! 実行：cargo run --example 12_testing
//! テスト：cargo test --example 12_testing

fn is_morning(hour: u32) -> bool {
    hour < 12
}

fn main() {
    println!("10時は午前？ {}", is_morning(10));
}

// cfg(test)：テストのときだけコンパイルする
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの関数を使う

    #[test]
    fn 午前を判定できる() {
        assert!(is_morning(10));
    }

    #[test]
    fn 正午は午前ではない() {
        assert!(!is_morning(12));
    }

    #[test]
    fn 値を比べる() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    #[should_panic] // 異常終了することを期待する
    fn 変換できない文字列でunwrapすると異常終了する() {
        "abc".parse::<u32>().unwrap();
    }
}
