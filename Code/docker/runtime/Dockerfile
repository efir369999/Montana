# Montana node — Docker runtime image.
#
# Two-stage build:
#   1. rust:1.95-bookworm — builds montana-node release binary
#   2. debian:trixie-slim — runs the binary (newer glibc covers symbols the
#      rust image links against; bookworm-slim is too old)
#
# Build context is the Code/ directory of the Montana repository.
# Standalone build:
#   docker build -t montana-node:local -f Code/docker/runtime/Dockerfile Code/
# Compose orchestration:
#   docker compose -f Code/docker/runtime/docker-compose.yml up -d --build

FROM rust:1.95-bookworm AS builder
WORKDIR /build
COPY . /build/
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false
RUN cargo build --release -j 1 -p montana-node 2>&1 | tail -50 && \
    strip target/release/montana-node && \
    ls -lh target/release/montana-node

FROM debian:trixie-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tini util-linux && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -m -d /var/lib/montana -s /usr/sbin/nologin montana && \
    mkdir -p /etc/montana && chmod 0755 /etc/montana
COPY --from=builder /build/target/release/montana-node /usr/local/bin/montana-node
COPY scripts/genesis-manifest.json /etc/montana/genesis-manifest.json
COPY docker/runtime/entrypoint.sh /usr/local/bin/montana-entrypoint
RUN chmod +x /usr/local/bin/montana-entrypoint
WORKDIR /var/lib/montana
VOLUME ["/var/lib/montana"]
EXPOSE 8444
# Run as root so the entrypoint can chown the named volume, then drop to montana.
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/montana-entrypoint"]
