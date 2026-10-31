//! クリニック予約のWebサーバー。今月と来月のカレンダーを表示し、フォームから予約を受け付ける。
//! 予約はSQLiteのファイルに保存するため、サーバーを止めても残る。

mod db;
mod security;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use askama::Template;
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, Extension, Query, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{from_fn, from_fn_with_state},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{Datelike, FixedOffset, NaiveDate, Utc};
use reservation_core::{
    Book, BookError, CAPACITY, TIMES, days_of_month, first_of_next_month, is_closed,
};
use rusqlite::Connection;
use security::{RateLimited, RateLimiter};
use serde::Deserialize;
use tower_http::{set_header::SetResponseHeaderLayer, timeout::TimeoutLayer};

/// 全リクエストで共有するDB接続。
/// Arc：複数のリクエストから同じ接続を参照する
/// Mutex：同じ接続を同時に使わないよう、1リクエストずつ順番に使う
type SharedConn = Arc<Mutex<Connection>>;

const MAX_NAME_CHARS: usize = 20;

/// 同じ送信元から、この時間内に予約できる回数
const MAX_RESERVATIONS: usize = 5;
const RESERVATION_WINDOW: Duration = Duration::from_secs(10 * 60);

// ここが最初に動く場所（Javaの main）
#[tokio::main]
async fn main() {
    // 環境変数 DATABASE_PATH がなければ、起動したフォルダに reservations.db を作る
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "reservations.db".to_string());
    let conn = db::open(&db_path).expect("DBを開けませんでした");
    // Renderで動かすときは TRUST_PROXY=1 にして、送信元をプロキシのヘッダーから取る
    let trust_proxy = std::env::var("TRUST_PROXY").is_ok_and(|v| v == "1");
    let limiter = RateLimiter::new(MAX_RESERVATIONS, RESERVATION_WINDOW, trust_proxy);
    let app = app(conn, limiter);

    // Renderでは環境変数 HOST=0.0.0.0、PORT=割り当て番号 を使う
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("ポートを開けませんでした");
    println!("http://{addr}/ で起動しました（Ctrl+Cで停止）");
    // into_make_service_with_connect_info：送信元のIPアドレスを処理の中で使えるようにする
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("サーバーが異常終了しました");
}

/// URLと処理する関数の対応表に、攻撃への対策を重ねる
fn app(conn: Connection, limiter: RateLimiter) -> Router {
    let state: SharedConn = Arc::new(Mutex::new(conn));
    let mut router = Router::new()
        .route("/", get(index))
        // 予約の送信だけ、送信元ごとの回数制限をかける
        .route(
            "/reserve",
            post(reserve).layer(from_fn_with_state(Arc::new(limiter), security::limit_rate)),
        )
        .route("/style.css", get(style))
        .with_state(state)
        .layer(from_fn(security::reject_cross_site))
        // 送信できるデータは1KBまで。名前20文字なら十分に収まる
        .layer(DefaultBodyLimit::max(1024))
        // 10秒で終わらない処理は打ち切る
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ));
    for (name, value) in security::HEADERS {
        router = router.layer(SetResponseHeaderLayer::overriding(
            name,
            HeaderValue::from_static(value),
        ));
    }
    router
}

/// DB接続を借りる。別の処理が異常終了していても、止まらずに使い続ける
fn lock(conn: &SharedConn) -> MutexGuard<'_, Connection> {
    conn.lock().unwrap_or_else(PoisonError::into_inner)
}

/// GET /style.css ：画面のCSS。CSPでインラインのCSSを禁止しているため、別ファイルで返す
async fn style() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css"),
    )
}

/// 日本時間の今日。Renderのサーバーは協定世界時（UTC）で動くため、9時間ずらす
fn today() -> NaiveDate {
    let jst = FixedOffset::east_opt(9 * 3600).expect("+9時間は有効な時差");
    Utc::now().with_timezone(&jst).date_naive()
}

/// URLのクエリ。例：/?date=2026-10-28&done=true
#[derive(Deserialize)]
struct PageQuery {
    /// 選んだ日。日付の形式が正しくなければ、axum が 400 を返す
    date: Option<NaiveDate>,
    /// 予約が完了した直後なら true
    #[serde(default)]
    done: bool,
}

/// フォームの送信内容。項目名はHTMLの name 属性と一致させる
#[derive(Deserialize)]
struct ReserveForm {
    date: NaiveDate,
    /// 時間を選ばずに送信された場合は None
    time: Option<String>,
    name: String,
}

/// GET / ：カレンダーと、選んだ日の枠を表示する
async fn index(State(conn): State<SharedConn>, Query(query): Query<PageQuery>) -> Response {
    let conn = lock(&conn);
    match db::load_book(&conn, today()) {
        Ok(book) => {
            let notice = query.done.then_some("予約しました");
            Html(render_page(&book, query.date, None, notice)).into_response()
        }
        Err(e) => server_error(e),
    }
}

/// POST /reserve ：予約する。成功したら、選んだ日の画面へ戻す
/// 流れ：DBから読む → core のルールで判定 → DBに書く
async fn reserve(
    State(conn): State<SharedConn>,
    // 回数制限を超えていれば Some（security::limit_rate が付ける印）
    limited: Option<Extension<RateLimited>>,
    Form(form): Form<ReserveForm>,
) -> Response {
    let conn = lock(&conn);
    let mut book = match db::load_book(&conn, today()) {
        Ok(book) => book,
        Err(e) => return server_error(e),
    };
    let date = form.date;

    if limited.is_some() {
        return error_page(
            &book,
            date,
            StatusCode::TOO_MANY_REQUESTS,
            "短時間に予約が続いたため、10分ほどしてから再度お試しください",
        );
    }

    let name = form.name.trim();
    if name.is_empty() {
        return error_page(
            &book,
            date,
            StatusCode::BAD_REQUEST,
            "名前を入力してください",
        );
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return error_page(
            &book,
            date,
            StatusCode::BAD_REQUEST,
            "名前は20文字以内で入力してください",
        );
    }
    // 改行などの制御文字や、文字の向きを変える記号・ゼロ幅の文字は受け付けない
    if name.chars().any(|c| c.is_control() || is_invisible(c)) {
        return error_page(
            &book,
            date,
            StatusCode::BAD_REQUEST,
            "名前に使えない文字が含まれています",
        );
    }
    let Some(time) = form.time.as_deref() else {
        return error_page(&book, date, StatusCode::BAD_REQUEST, "時間を選んでください");
    };

    if let Err(e) = book.reserve(date, time, name) {
        return error_page(&book, date, StatusCode::BAD_REQUEST, &e.to_string());
    }
    match db::insert_reservation(&conn, date, time, name) {
        // 再読み込みで二重送信しないよう、成功時は GET へリダイレクトする
        Ok(true) => Redirect::to(&format!("/?date={date}&done=true#day")).into_response(),
        // core の判定のあとに、別のプロセスが同じ枠を埋めた場合。DBの条件付き追加で止まる
        Ok(false) => match db::load_book(&conn, today()) {
            Ok(book) => error_page(
                &book,
                date,
                StatusCode::BAD_REQUEST,
                &BookError::Full.to_string(),
            ),
            Err(e) => server_error(e),
        },
        Err(e) => server_error(e),
    }
}

/// 画面に見えないが、表示や並びを変えてしまう文字
fn is_invisible(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}

/// 選んだ日の見出しの右に、エラーを表示した画面
fn error_page(book: &Book, date: NaiveDate, status: StatusCode, message: &str) -> Response {
    (
        status,
        Html(render_page(book, Some(date), Some(message), None)),
    )
        .into_response()
}

/// DBの読み書きに失敗したとき。詳細はログにだけ出し、画面には出さない
fn server_error(e: rusqlite::Error) -> Response {
    eprintln!("DBエラー：{e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>サーバーでエラーが発生しました。時間をおいて再度お試しください。</p>"),
    )
        .into_response()
}

/// 画面のテンプレート。templates/index.html と項目名を一致させる
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    /// 今月と来月のカレンダー
    months: Vec<MonthView>,
    /// 選んだ日。選んでいなければ None
    day: Option<DayView>,
    error: Option<&'a str>,
    notice: Option<&'a str>,
    capacity: usize,
    max_name_chars: usize,
}

/// 1か月分のカレンダー
struct MonthView {
    label: String,
    /// 週ごとの行。月の前後の空きマスは None
    weeks: Vec<Vec<Option<DayCell>>>,
}

/// カレンダーの1マス
struct DayCell {
    date: NaiveDate,
    status: DayStatus,
    selected: bool,
}

enum DayStatus {
    /// 予約できる。値は残り人数
    Open(usize),
    Full,
    Closed,
    /// 過去の日
    Past,
}

/// 選んだ日の枠の一覧
struct DayView {
    date: NaiveDate,
    label: String,
    bookable: bool,
    slots: Vec<SlotView>,
}

struct SlotView {
    time: &'static str,
    remaining: usize,
}

const WEEKDAYS: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];

/// 画面のHTMLを組み立てる
fn render_page(
    book: &Book,
    selected: Option<NaiveDate>,
    error: Option<&str>,
    notice: Option<&str>,
) -> String {
    let today = book.today();
    let page = IndexTemplate {
        months: [today, first_of_next_month(today)]
            .into_iter()
            .map(|month| MonthView {
                label: format!("{}年{}月", month.year(), month.month()),
                weeks: calendar(book, month, selected),
            })
            .collect(),
        day: selected.map(|date| day_view(book, date)),
        error,
        notice,
        capacity: CAPACITY,
        max_name_chars: MAX_NAME_CHARS,
    };
    // テンプレートの書き間違いはコンパイル時に検出されるため、ここで失敗することはまずない
    page.render().expect("画面の組み立てに失敗しました")
}

/// month を含む月のカレンダーを、日曜始まりの週ごとに並べる
fn calendar(
    book: &Book,
    month: NaiveDate,
    selected: Option<NaiveDate>,
) -> Vec<Vec<Option<DayCell>>> {
    let days = days_of_month(month);
    // 1日が何曜日かに合わせて、最初の週の前を空ける
    let blanks = days[0].weekday().num_days_from_sunday() as usize;
    let mut cells: Vec<Option<DayCell>> = (0..blanks).map(|_| None).collect();

    for date in days {
        let status = if is_closed(date) {
            DayStatus::Closed
        } else if !book.is_bookable_day(date) {
            DayStatus::Past
        } else {
            match book.remaining_on(date) {
                0 => DayStatus::Full,
                remaining => DayStatus::Open(remaining),
            }
        };
        cells.push(Some(DayCell {
            date,
            status,
            selected: selected == Some(date),
        }));
    }
    // 最後の週の後ろも空けて、7マスにそろえる
    while !cells.len().is_multiple_of(7) {
        cells.push(None);
    }

    // 7マスずつ取り出して、週の行にする
    let mut weeks = Vec::new();
    let mut iter = cells.into_iter();
    loop {
        let week: Vec<_> = iter.by_ref().take(7).collect();
        if week.is_empty() {
            break;
        }
        weeks.push(week);
    }
    weeks
}

fn day_view(book: &Book, date: NaiveDate) -> DayView {
    let weekday = WEEKDAYS[date.weekday().num_days_from_sunday() as usize];
    DayView {
        date,
        label: format!("{}月{}日（{weekday}）", date.month(), date.day()),
        bookable: book.is_bookable_day(date),
        slots: TIMES
            .iter()
            .map(|&time| SlotView {
                time,
                remaining: book.remaining(date, time),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use chrono::{Days, Weekday};
    use tower::ServiceExt;

    /// メモリ上のDBで、すべての対策を重ねたアプリを作る
    fn test_app(max_reservations: usize) -> Router {
        let conn = db::open(":memory:").unwrap();
        app(
            conn,
            RateLimiter::new(max_reservations, RESERVATION_WINDOW, false),
        )
    }

    /// 予約できる日（今日が日曜なら翌日）
    fn open_day() -> NaiveDate {
        let today = today();
        if today.weekday() == Weekday::Sun {
            today + Days::new(1)
        } else {
            today
        }
    }

    fn reserve_request(time: &str, site: &'static str) -> Request<Body> {
        Request::post("/reserve")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("sec-fetch-site", site)
            .body(Body::from(format!(
                "date={}&time={time}&name=a",
                open_day()
            )))
            .unwrap()
    }

    #[tokio::test]
    async fn 応答にセキュリティ用のヘッダーが付く() {
        let res = test_app(5)
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        for (name, value) in security::HEADERS {
            assert_eq!(res.headers()[name], value);
        }
    }

    #[tokio::test]
    async fn 画面に練習用のデモだと表示する() {
        let res = test_app(5)
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("実在のクリニックではなく"));
        assert!(html.contains("本名や個人情報は入力しないでください"));
    }

    #[tokio::test]
    async fn 同じサイトからの予約は受け付ける() {
        let res = test_app(5)
            .oneshot(reserve_request("16:00", "same-origin"))
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn 別のサイトからの予約は断る() {
        let res = test_app(5)
            .oneshot(reserve_request("16:00", "cross-site"))
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 短時間に回数を超えた予約は断る() {
        let app = test_app(2);
        for time in ["09:00", "10:00"] {
            let res = app
                .clone()
                .oneshot(reserve_request(time, "same-origin"))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::SEE_OTHER);
        }

        let res = app
            .oneshot(reserve_request("11:00", "same-origin"))
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        // カレンダー付きの画面に、エラーが表示される
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("class=\"calendar\""));
        assert!(html.contains("登録できませんでした。短時間に予約が続いたため"));
    }

    #[tokio::test]
    async fn 大きすぎる送信は断る() {
        let body = format!("date={}&time=09:00&name={}", open_day(), "a".repeat(2000));
        let res = test_app(5)
            .oneshot(
                Request::post("/reserve")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn 見えない文字を見分けられる() {
        assert!(is_invisible('\u{202E}'));
        assert!(is_invisible('\u{200B}'));
        assert!(!is_invisible('山'));
        assert!(!is_invisible(' '));
    }
}
