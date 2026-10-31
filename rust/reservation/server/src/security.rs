//! サイバー攻撃への対策。予約の処理とは分けて、ルーターに重ねて使う。

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};

/// すべての応答に付けるヘッダー
pub const HEADERS: [(header::HeaderName, &str); 4] = [
    // CSP：スクリプトは一切動かさず、CSSと送信先は同じサイトだけに限る（XSS対策）
    // frame-ancestors：他のサイトの枠の中に表示させない（クリックジャッキング対策）
    (
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; style-src 'self'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'",
    ),
    // CSPに対応していない古いブラウザ向けのクリックジャッキング対策
    (header::X_FRAME_OPTIONS, "DENY"),
    // ファイルの種類をブラウザに推測させない
    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    // 他のサイトへ移動するとき、URL（選んだ日付など）を渡さない
    (header::REFERRER_POLICY, "same-origin"),
];

/// 他のサイトからのフォーム送信を断る（CSRF対策）
pub async fn reject_cross_site(req: Request, next: Next) -> Response {
    let is_read = matches!(*req.method(), Method::GET | Method::HEAD);
    if !is_read && is_cross_site(req.headers()) {
        return (
            StatusCode::FORBIDDEN,
            Html("<p>このページ以外からは送信できません。</p>"),
        )
            .into_response();
    }
    next.run(req).await
}

/// ブラウザが付けるヘッダーで、送信元が別のサイトかを判定する
fn is_cross_site(headers: &HeaderMap) -> bool {
    // 新しいブラウザは Sec-Fetch-Site を付ける。同じサイトからなら same-origin
    if let Some(site) = headers.get("sec-fetch-site") {
        return site != "same-origin";
    }
    // 古いブラウザ向け：Origin（送信元）と Host（このサーバー）を比べる
    // どちらもない場合（curl など、ブラウザ以外）は通す
    match (headers.get(header::ORIGIN), headers.get(header::HOST)) {
        (Some(origin), Some(host)) => {
            let origin = origin.to_str().unwrap_or_default();
            let authority = origin.split_once("://").map_or("", |(_, rest)| rest);
            authority.as_bytes() != host.as_bytes()
        }
        _ => false,
    }
}

/// 同じ送信元からの予約を、一定時間に決まった回数までに制限する（枠の買い占め対策）
pub struct RateLimiter {
    max: usize,
    window: Duration,
    /// Render などのプロキシの後ろで動くとき true。送信元を X-Forwarded-For から取る
    trust_proxy: bool,
    hits: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration, trust_proxy: bool) -> Self {
        RateLimiter {
            max,
            window,
            trust_proxy,
            hits: Mutex::new(HashMap::new()),
        }
    }

    /// 受け付けてよければ true を返し、受け付けた時刻を記録する
    fn allow(&self, ip: IpAddr, now: Instant) -> bool {
        // 別の処理が異常終了していても、記録はそのまま使い続ける
        let mut hits = self.hits.lock().unwrap_or_else(PoisonError::into_inner);
        // 期限切れの記録を消す（メモリが増え続けないように）
        hits.retain(|_, times| {
            times.retain(|t| now.duration_since(*t) < self.window);
            !times.is_empty()
        });
        let times = hits.entry(ip).or_default();
        if times.len() >= self.max {
            return false;
        }
        times.push(now);
        true
    }

    /// 送信元のIPアドレス
    fn client_ip(&self, req: &Request) -> Option<IpAddr> {
        if self.trust_proxy {
            // 右端の値は、プロキシが付けたもの。左側は送信者が偽れるので使わない
            let forwarded = req
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.rsplit(',').next())
                .and_then(|v| v.trim().parse().ok());
            if forwarded.is_some() {
                return forwarded;
            }
        }
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
    }
}

/// 回数制限を超えた送信に付ける印。予約の処理側で、カレンダー付きのエラー画面を出す
#[derive(Clone)]
pub struct RateLimited;

/// 回数制限を超えた送信に、印（RateLimited）を付ける
pub async fn limit_rate(
    State(limiter): State<Arc<RateLimiter>>,
    mut req: Request,
    next: Next,
) -> Response {
    // 送信元が分からないときは、まとめて1つの送信元として数える
    let ip = limiter
        .client_ip(&req)
        .unwrap_or(IpAddr::from([0, 0, 0, 0]));
    if !limiter.allow(ip, Instant::now()) {
        req.extensions_mut().insert(RateLimited);
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn 同じサイトからの送信は通す() {
        assert!(!is_cross_site(&headers(&[(
            "sec-fetch-site",
            "same-origin"
        )])));
        assert!(!is_cross_site(&headers(&[
            ("origin", "http://127.0.0.1:3000"),
            ("host", "127.0.0.1:3000"),
        ])));
    }

    #[test]
    fn 別のサイトからの送信は断る() {
        assert!(is_cross_site(&headers(&[("sec-fetch-site", "cross-site")])));
        assert!(is_cross_site(&headers(&[
            ("origin", "https://evil.example"),
            ("host", "127.0.0.1:3000"),
        ])));
    }

    #[test]
    fn 回数を超えると断り時間がたてば戻る() {
        let limiter = RateLimiter::new(2, Duration::from_secs(600), false);
        let ip = IpAddr::from([192, 0, 2, 1]);
        let now = Instant::now();

        assert!(limiter.allow(ip, now));
        assert!(limiter.allow(ip, now));
        assert!(!limiter.allow(ip, now));
        // 別の送信元は数が別
        assert!(limiter.allow(IpAddr::from([192, 0, 2, 2]), now));
        // 時間がたてば、また受け付ける
        assert!(limiter.allow(ip, now + Duration::from_secs(600)));
    }
}
