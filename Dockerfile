FROM rust:1-slim-bookworm as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim as runner

WORKDIR /app
COPY --from=builder /app/target/release/pomegranate .

CMD ["./pomegranate"]
