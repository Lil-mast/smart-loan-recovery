FROM rust:1.92-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# bookworm-slim does not include OpenSSL; reqwest/native-tls links libssl.so.3
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lendwise-recovery /usr/local/bin/
COPY --from=builder /app/frontend /usr/local/share/lendwise-frontend
EXPOSE 3000
ENV SERVER_HOST=0.0.0.0
ENV FRONTEND_DIR=/usr/local/share/lendwise-frontend
CMD ["lendwise-recovery"]