FROM rust:1-slim-bookworm as builder

WORKDIR /app
RUN apt-get update
RUN apt-get install protobuf-compiler -y
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim as runner

WORKDIR /app
COPY --from=builder /app/target/release/pomegranate .

EXPOSE 50051

CMD ["./pomegranate"]
