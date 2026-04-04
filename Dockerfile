FROM rust:1.92 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/lendwise-recovery /usr/local/bin/
COPY --from=builder /app/frontend /usr/local/share/lendwise-frontend
EXPOSE 3000
ENV SERVER_HOST=0.0.0.0
ENV FRONTEND_DIR=/usr/local/share/lendwise-frontend
CMD ["lendwise-recovery"]