FROM rust:1.92 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/smart-loan-recovery /usr/local/bin/
EXPOSE 3000
ENV SERVER_HOST=0.0.0.0
CMD ["smart-loan-recovery"]