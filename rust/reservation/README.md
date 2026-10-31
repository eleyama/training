# クリニック予約（Rust学習用）

Rustを学ぶための予約システム。今月と来月のカレンダーから日付と時間枠を選び、名前を入力して予約する。

## 構成

| ディレクトリ | 内容 | 状態 |
| --- | --- | --- |
| core/ | 予約ルール。入出力なし | 作成済み |
| server/ | axumでHTMLを返す。ローカル開発とRender公開用 | 作成済み（SQLite保存） |
| worker/ | workers-rs + D1。Cloudflare公開用 | 未作成 |

## 予約ルール

- 診療時間は 9:00〜11:00 と 14:00〜16:00 の各1時間枠（1日6枠）。日曜は休診
- 予約できるのは、今日から来月末までの診療日
- 1つの枠の定員は2人。予約が入るたびに残りが減る
- 画面には残り人数だけを出し、予約者の名前は表示しない
- 取り消し・本人確認はない（学習用のため）

## セキュリティ対策

server/src/security.rs と server/src/main.rs の `app()` にまとめている。

| 攻撃 | 対策 |
| --- | --- |
| SQLインジェクション | SQLに値をつなげず、プレースホルダー（`?1`）で渡す |
| XSS | テンプレートの自動エスケープ。CSPでスクリプトを一切動かさない |
| CSRF（他サイトからの送信） | `Sec-Fetch-Site` / `Origin` ヘッダーで送信元を確認し、別サイトなら403 |
| 枠の買い占め | 同じIPアドレスからの予約は10分に5回まで。超えたら429 |
| クリックジャッキング | CSPの `frame-ancestors 'none'` と `X-Frame-Options: DENY` |
| 巨大・低速な送信 | 送信は1KBまで（413）、処理は10秒まで（408） |
| 見えない文字 | 名前に制御文字・文字の向きを変える記号・ゼロ幅の文字があればエラー |

Renderなどプロキシの後ろで動かすときは、環境変数 `TRUST_PROXY=1` を設定する。設定しないと、全員がプロキシの1つのIPアドレスとして数えられる。

## サーバーの起動

```powershell
cargo run -p reservation-server
```

ブラウザで http://127.0.0.1:3000/ を開く。Ctrl+Cで停止。

予約は起動したフォルダの `reservations.db`（SQLite）に保存され、停止しても残る。保存先は環境変数 `DATABASE_PATH` で変えられる。予約を全部消すときは、停止してからこのファイルを削除する。

## 公開（Render）

リポジトリ直下の `render.yaml` で設定する。mainへのpushでGitHub Actionsの検査が成功すると、自動的に公開される。

無料枠ではファイルが残らないため、再起動や再公開で予約は消える。画面にも、練習用のデモであることと、本名を入力しないことを表示している。

```powershell
docker build -t reservation-demo .
docker run --rm -p 3000:10000 reservation-demo
```

## テストの実行

```powershell
cargo test
```

Rustを入れずにDockerで実行する場合：

```powershell
docker run --rm -v "${PWD}:/app" -w /app rust:1 cargo test
```

## 依存ライブラリの検査

```powershell
cargo install cargo-audit cargo-deny --locked   # 初回だけ
cargo audit        # 既知の脆弱性を調べる
cargo deny check   # deny.toml のルール（脆弱性・ライセンス・取得元）を調べる
```

GitHub Actions でも、pushのたびと週に1回、`cargo deny check` を実行する（`.github/workflows/reservation.yml`）。

## 進め方

1. core：予約ルールとテスト（Rustの基本）
2. server：axumでHTML画面とAPI（非同期処理・トレイト）
3. server：SQLiteで永続化し、二重予約をDBでも防ぐ（済）
4. Dockerイメージを作り、Renderの無料枠で公開
5. worker：coreを共有したままCloudflare Workers + D1へ移植
