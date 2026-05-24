# Montana node — Docker runtime image.
#
# Two-stage build:
#   1. rust:1.95-bookworm-slim builds montana-node release binary
#   2. debian:bookworm-slim runs the binary as user montana
#
# Build context is the Code/ directory of the Montana repository.
# Build:
#   docker build -t montana-node:local -f Code/docker/runtime/Dockerfile Code/
# Run via docker-compose.yml in this directory.

FROM rust:1.95-bookworm AS builder
WORKDIR /build
COPY . /build/
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false
# Single-job build to fit small VPS (~1 GB RAM). Slow but reliable.
RUN cargo build --release -j 1 -p montana-node 2>&1 | tail -50 && \
    strip target/release/montana-node && \
    ls -lh target/release/montana-node

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tini && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -m -d /var/lib/montana -s /usr/sbin/nologin montana && \
    mkdir -p /etc/montana && chmod 0755 /etc/montana
COPY --from=builder /build/target/release/montana-node /usr/local/bin/montana-node
COPY scripts/genesis-manifest.json /etc/montana/genesis-manifest.json
COPY docker/runtime/entrypoint.sh /usr/local/bin/montana-entrypoint
RUN chmod +x /usr/local/bin/montana-entrypoint
USER montana
WORKDIR /var/lib/montana
VOLUME ["/var/lib/montana"]
EXPOSE 8444
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/montana-entrypoint"]
