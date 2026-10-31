//! SQLiteへの保存。予約ルールの判定は core に任せ、ここでは読み書きだけを行う。
//!
//! SQLインジェクション対策：SQLに値を文字列としてつなげない。値は必ず ?1 などの
//! プレースホルダーに params! で渡す（JavaのPreparedStatementと同じ）。

use chrono::{Datelike, NaiveDate};
use reservation_core::{Book, CAPACITY, Reservation, last_bookable_day};
use rusqlite::{Connection, params};

/// DBファイルを開き、テーブルがなければ作る
pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    create_table(&conn)?;
    Ok(conn)
}

fn create_table(conn: &Connection) -> rusqlite::Result<()> {
    // execute_batch：複数のSQLをまとめて実行する
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS appointments (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            time TEXT NOT NULL,
            name TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS appointments_slot ON appointments (date, time);",
    )
}

/// 今月と来月の予約を読み込み、予約台帳を組み立てる
pub fn load_book(conn: &Connection, today: NaiveDate) -> rusqlite::Result<Book> {
    let first = today.with_day(1).expect("1日は必ず存在する");
    let last = last_bookable_day(today);

    let mut stmt = conn.prepare(
        "SELECT date, time, name FROM appointments WHERE date BETWEEN ?1 AND ?2 ORDER BY id",
    )?;
    // query_map：1行ごとにクロージャで値を取り出す（JDBCの ResultSet を while で回すのに近い）
    // 日付は rusqlite の chrono 機能で、TEXT（2026-10-28）と NaiveDate を自動で変換する
    let reservations = stmt
        .query_map(params![first, last], |row| {
            Ok(Reservation {
                date: row.get(0)?,
                time: row.get(1)?,
                name: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Book::new(today, reservations))
}

/// 予約を保存する。枠が定員に達していれば保存せず false を返す
pub fn insert_reservation(
    conn: &Connection,
    date: NaiveDate,
    time: &str,
    name: &str,
) -> rusqlite::Result<bool> {
    // 「数える」と「追加する」を1つのSQLで行うので、間に別の予約が割り込めない（core のルールと同じ内容をDBでも守る）
    let inserted = conn.execute(
        "INSERT INTO appointments (date, time, name)
         SELECT ?1, ?2, ?3
         WHERE (SELECT COUNT(*) FROM appointments WHERE date = ?1 AND time = ?2) < ?4",
        params![date, time, name, CAPACITY as i64],
    )?;
    Ok(inserted == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        // テストではファイルを作らず、メモリ上のDBを使う
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();
        conn
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn 保存した予約を読み込める() {
        let conn = conn();
        insert_reservation(&conn, date(2026, 10, 15), "09:00", "山田").unwrap();

        let book = load_book(&conn, date(2026, 10, 14)).unwrap();

        assert_eq!(book.reservations().len(), 1);
        assert_eq!(book.remaining(date(2026, 10, 15), "09:00"), 1);
    }

    #[test]
    fn 定員を超える保存はdbでも止まる() {
        let conn = conn();

        assert!(insert_reservation(&conn, date(2026, 10, 15), "09:00", "山田").unwrap());
        assert!(insert_reservation(&conn, date(2026, 10, 15), "09:00", "山田").unwrap());
        assert!(!insert_reservation(&conn, date(2026, 10, 15), "09:00", "山田").unwrap());
    }

    #[test]
    fn sqlとして危険な名前も文字のまま保存される() {
        let conn = conn();
        let name = "x'); DROP TABLE appointments; --";

        assert!(insert_reservation(&conn, date(2026, 10, 15), "09:00", name).unwrap());

        // テーブルは消えておらず、名前は入力したとおりに保存されている
        let book = load_book(&conn, date(2026, 10, 14)).unwrap();
        assert_eq!(book.reservations()[0].name, name);
    }

    #[test]
    fn 今月と来月以外の予約は読み込まない() {
        let conn = conn();
        insert_reservation(&conn, date(2026, 9, 30), "09:00", "山田").unwrap();
        insert_reservation(&conn, date(2026, 12, 1), "09:00", "山田").unwrap();

        assert_eq!(
            load_book(&conn, date(2026, 10, 14))
                .unwrap()
                .reservations()
                .len(),
            0
        );
    }
}
