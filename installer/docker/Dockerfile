# Kovanica Protocol — Docker Deployment
#
# Run a Kovanica node in a container.
# Image is built from the official Rust base image for reproducibility.

FROM rust:1.98-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --release -p kovanica-node

# ─── Runtime ─────────────────────────────────────────────────────────────────

FROM debian:bookworm-slim

RUN apt-get update -qq && \
    apt-get install -y -qq --no-install-recommends \
        ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kovanica-node /usr/local/bin/kovanica-node

RUN useradd --system --no-create-home --shell /usr/sbin/nologin kovanica && \
    mkdir -p /var/lib/kovanica && \
    chown kovanica:kovanica /var/lib/kovanica

USER kovanica
WORKDIR /var/lib/kovanica

ENV KOVANICA_DATA=/var/lib/kovanica
ENV KOVANICA_P2P_PORT=9000
ENV KOVANICA_HTTP_PORT=8080
ENV KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000

EXPOSE 9000 8080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -sf http://localhost:${KOVANICA_HTTP_PORT}/api/head || exit 1

ENTRYPOINT ["kovanica-node"]
CMD ["serve"]
