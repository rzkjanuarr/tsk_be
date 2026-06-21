FROM rust:latest as builder

WORKDIR /app

COPY Cargo.toml ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/task_backend .

EXPOSE 8080

ENV RUST_LOG=info

CMD ["./task_backend"]
