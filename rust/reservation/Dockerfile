# ビルド用。Rustとコンパイラが入った大きなイメージ
FROM rust:1.99.1-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY core core
COPY server server
RUN cargo build --release --locked -p reservation-server

# 実行用。完成したプログラムだけを入れた小さなイメージ
FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 app && mkdir /data && chown app /data
COPY --from=build /app/target/release/reservation-server /usr/local/bin/reservation-server
# 管理者権限を使わずに動かす
USER app
ENV HOST=0.0.0.0 PORT=10000 DATABASE_PATH=/data/reservations.db
EXPOSE 10000
CMD ["reservation-server"]
