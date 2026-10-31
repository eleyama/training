//! Result：成功（Ok）か失敗（Err）か。Rustには例外がなく、失敗は戻り値で返す
//! 実行：cargo run --example 08_result

use std::fmt;

// 失敗の理由を enum で表す
#[derive(Debug)]
enum NameError {
    Empty,
    TooLong(usize),
}

// Display を実装すると {} でメッセージを表示できる
impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => write!(f, "名前が空です"),
            NameError::TooLong(n) => write!(f, "名前が長すぎます（{n}文字）"),
        }
    }
}

fn check_name(name: &str) -> Result<String, NameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    let n = name.chars().count();
    if n > 5 {
        return Err(NameError::TooLong(n));
    }
    Ok(name.to_string())
}

// ? ：Err ならその場で呼び出し元へ返す（Javaの throws に近い）
fn register(name: &str) -> Result<String, NameError> {
    let name = check_name(name)?;
    Ok(format!("{name} を登録しました"))
}

fn main() {
    for input in ["山田", "  ", "とても長い名前です"] {
        match register(input) {
            Ok(message) => println!("成功: {message}"),
            Err(e) => println!("失敗: {e}"),
        }
    }

    // 標準ライブラリも Result を返す
    for text in ["13", "abc"] {
        match text.parse::<u32>() {
            Ok(hour) => println!("{hour}時"),
            Err(e) => println!("「{text}」は変換できません: {e}"),
        }
    }
}
