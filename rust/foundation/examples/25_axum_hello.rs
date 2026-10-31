//! axum の最小構成：名前を追加して一覧表示する
//! server/src/main.rs を小さくしたもの
//! 実行：cargo run --example 25_axum_hello → http://127.0.0.1:3001/ を開く（Ctrl+Cで停止）

use std::sync::{Arc, Mutex};

use axum::{
    Form, Router,
    extract::State,
    response::{Html, Redirect},
    routing::{get, post},
};
use serde::Deserialize;

type Shared = Arc<Mutex<Vec<String>>>;

#[derive(Deserialize)]
struct NameForm {
    name: String, // HTML の <input name="name"> と対応
}

#[tokio::main]
async fn main() {
    let state: Shared = Arc::new(Mutex::new(Vec::new()));

    // URL と関数の対応（Spring の @GetMapping / @PostMapping）
    let app = Router::new()
        .route("/", get(index))
        .route("/add", post(add))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    println!("http://127.0.0.1:3001/ で起動しました");
    axum::serve(listener, app).await.unwrap();
}

// GET /
async fn index(State(state): State<Shared>) -> Html<String> {
    let names = state.lock().unwrap();
    let items: String = names
        .iter()
        .map(|name| format!("<li>{}</li>", escape(name)))
        .collect();
    Html(format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>練習</title>
<form method="post" action="/add">
<input name="name" required aria-label="名前"><button>追加</button>
</form>
<ul>{items}</ul>"#
    ))
}

// POST /add
async fn add(State(state): State<Shared>, Form(form): Form<NameForm>) -> Redirect {
    state.lock().unwrap().push(form.name);
    Redirect::to("/")
}

// 入力された文字をHTMLのタグとして解釈させない
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
