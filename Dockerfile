FROM rust:1.81 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/smart-loan-recovery /usr/local/bin/
CMD ["smart-loan-recovery"]