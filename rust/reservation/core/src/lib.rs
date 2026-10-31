//! 予約ルール。DB・HTTPなどの入出力を持たず、Render版とCloudflare版で共有する。
//!
//! ルール：
//! - 予約できるのは、今日から来月末までの診療日（日曜は休診）
//! - 1つの枠の定員は2人。予約が入るたびに残りが減る

use std::fmt;

use chrono::{Datelike, Months, NaiveDate, Weekday};

/// 1つの枠の定員
pub const CAPACITY: usize = 2;

/// 1日の予約枠（開始時刻）
pub const TIMES: [&str; 6] = ["09:00", "10:00", "11:00", "14:00", "15:00", "16:00"];

/// いつ・どの枠を・誰が予約したか
#[derive(Debug, Clone, PartialEq)]
pub struct Reservation {
    pub date: NaiveDate,
    pub time: String,
    pub name: String,
}

/// 予約に失敗した理由
#[derive(Debug, PartialEq)]
pub enum BookError {
    UnknownTime,
    OutOfRange,
    Closed,
    Full,
}

impl fmt::Display for BookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            BookError::UnknownTime => "その時間の枠はありません",
            BookError::OutOfRange => "予約できるのは今日から来月末までです",
            BookError::Closed => "休診日です",
            BookError::Full => "その枠は満員です",
        };
        write!(f, "{message}")
    }
}

/// 休診日なら true
pub fn is_closed(date: NaiveDate) -> bool {
    date.weekday() == Weekday::Sun
}

/// 指定した日を含む月の翌月1日
pub fn first_of_next_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1)
        .and_then(|first| first.checked_add_months(Months::new(1)))
        .expect("翌月1日は必ず存在する")
}

/// 予約を受け付ける最後の日（来月末）
pub fn last_bookable_day(today: NaiveDate) -> NaiveDate {
    first_of_next_month(first_of_next_month(today))
        .pred_opt()
        .expect("前日は必ず存在する")
}

/// 指定した日を含む月の、1日から末日まで
pub fn days_of_month(date: NaiveDate) -> Vec<NaiveDate> {
    let first = date.with_day(1).expect("1日は必ず存在する");
    first
        .iter_days()
        .take_while(|d| d.month() == first.month())
        .collect()
}

/// 予約台帳（今月と来月の分）
#[derive(Debug)]
pub struct Book {
    today: NaiveDate,
    reservations: Vec<Reservation>,
}

impl Book {
    /// 保存済みの予約から台帳を作る。today は「今日」の日付（テストで日付を固定できるよう外から渡す）
    pub fn new(today: NaiveDate, reservations: Vec<Reservation>) -> Self {
        Book {
            today,
            reservations,
        }
    }

    pub fn today(&self) -> NaiveDate {
        self.today
    }

    /// すべての予約
    pub fn reservations(&self) -> &[Reservation] {
        &self.reservations
    }

    /// 枠の残り人数
    pub fn remaining(&self, date: NaiveDate, time: &str) -> usize {
        let count = self
            .reservations
            .iter()
            .filter(|r| r.date == date && r.time == time)
            .count();
        CAPACITY.saturating_sub(count)
    }

    /// その日の残り人数（全枠の合計）
    pub fn remaining_on(&self, date: NaiveDate) -> usize {
        TIMES.iter().map(|time| self.remaining(date, time)).sum()
    }

    /// その日に予約を受け付けるか
    pub fn is_bookable_day(&self, date: NaiveDate) -> bool {
        self.in_range(date) && !is_closed(date)
    }

    pub fn reserve(&mut self, date: NaiveDate, time: &str, name: &str) -> Result<(), BookError> {
        if !TIMES.contains(&time) {
            return Err(BookError::UnknownTime);
        }
        if !self.in_range(date) {
            return Err(BookError::OutOfRange);
        }
        if is_closed(date) {
            return Err(BookError::Closed);
        }
        if self.remaining(date, time) == 0 {
            return Err(BookError::Full);
        }
        self.reservations.push(Reservation {
            date,
            time: time.to_string(),
            name: name.to_string(),
        });
        Ok(())
    }

    /// 今日から来月末までなら true
    fn in_range(&self, date: NaiveDate) -> bool {
        self.today <= date && date <= last_bookable_day(self.today)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// 2026-10-14（水）を「今日」とする
    fn book() -> Book {
        Book::new(date(2026, 10, 14), Vec::new())
    }

    #[test]
    fn 予約すると枠の残りが減る() {
        let mut book = book();
        assert_eq!(book.remaining(date(2026, 10, 15), "09:00"), 2);

        assert_eq!(book.reserve(date(2026, 10, 15), "09:00", "山田"), Ok(()));
        assert_eq!(book.remaining(date(2026, 10, 15), "09:00"), 1);
    }

    #[test]
    fn 定員に達した枠は予約できない() {
        let mut book = book();
        book.reserve(date(2026, 10, 15), "09:00", "山田").unwrap();
        book.reserve(date(2026, 10, 15), "09:00", "山田").unwrap();

        assert_eq!(
            book.reserve(date(2026, 10, 15), "09:00", "山田"),
            Err(BookError::Full)
        );
        // 別の枠は予約できる
        assert_eq!(book.reserve(date(2026, 10, 15), "10:00", "山田"), Ok(()));
    }

    #[test]
    fn 今日は予約できる() {
        let mut book = book();

        assert_eq!(book.reserve(date(2026, 10, 14), "16:00", "山田"), Ok(()));
    }

    #[test]
    fn 過去の日は予約できない() {
        let mut book = book();

        assert_eq!(
            book.reserve(date(2026, 10, 13), "09:00", "山田"),
            Err(BookError::OutOfRange)
        );
    }

    #[test]
    fn 来月末まで予約できる() {
        let mut book = book();

        assert_eq!(book.reserve(date(2026, 11, 30), "09:00", "山田"), Ok(()));
    }

    #[test]
    fn 再来月は予約できない() {
        let mut book = book();

        assert_eq!(
            book.reserve(date(2026, 12, 1), "09:00", "山田"),
            Err(BookError::OutOfRange)
        );
    }

    #[test]
    fn 年をまたいで来月を予約できる() {
        let mut book = Book::new(date(2026, 12, 10), Vec::new());

        assert_eq!(last_bookable_day(book.today()), date(2027, 1, 31));
        assert_eq!(book.reserve(date(2027, 1, 5), "09:00", "山田"), Ok(()));
    }

    #[test]
    fn 日曜は休診で予約できない() {
        let mut book = book();

        assert_eq!(
            book.reserve(date(2026, 10, 18), "09:00", "山田"),
            Err(BookError::Closed)
        );
    }

    #[test]
    fn 枠にない時間は予約できない() {
        let mut book = book();

        assert_eq!(
            book.reserve(date(2026, 10, 15), "12:00", "山田"),
            Err(BookError::UnknownTime)
        );
    }

    #[test]
    fn 一日の残りは全枠の合計() {
        let mut book = book();
        assert_eq!(book.remaining_on(date(2026, 10, 15)), 12);

        book.reserve(date(2026, 10, 15), "09:00", "山田").unwrap();

        assert_eq!(book.remaining_on(date(2026, 10, 15)), 11);
    }

    #[test]
    fn 月の日数を数えられる() {
        assert_eq!(days_of_month(date(2026, 10, 14)).len(), 31);
        assert_eq!(days_of_month(date(2026, 2, 10)).len(), 28);
        assert_eq!(days_of_month(date(2028, 2, 10)).len(), 29);
    }
}
