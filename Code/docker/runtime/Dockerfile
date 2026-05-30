# Montana node + VPN endpoint — all-in-one Docker runtime image (v1.0.2).
#
# Single container, single docker run, full join to the Montana fleet:
#   docker run -d --name montana-node --network host --restart unless-stopped \
#     -v montana-data:/var/lib/montana \
#     ghcr.io/efir369999/montana-node:latest
#
# Inside the container:
#   - tini supervisor → /usr/local/bin/montana-entrypoint
#   - nginx :80   (decoy landing page)
#   - xray :443   (Reality VLESS with embedded universal pbk/sid + per-host UUID)
#   - montana-node :8444 (TimeChain consensus, foreground process)
#   - auto-register POST /register to Moscow orchestrator on first start
#
# Two-stage build:
#   1. rust:1.95-bookworm — builds montana-node release binary
#   2. debian:trixie-slim — runs node + nginx + xray + supervisor
#
# Build context = REPO ROOT (parent of Code/) because mt-mnemonic uses
# include_str!("../../../../Montana wordlist.txt").

FROM rust:1.95-bookworm AS builder
WORKDIR /build
COPY . /build/
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false
WORKDIR /build/Code
RUN cargo build --release -j 1 -p montana-node && \
    strip target/release/montana-node && \
    ls -lh target/release/montana-node

FROM debian:trixie-slim
ARG XRAY_VERSION=v1.8.24
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates tini util-linux curl unzip jq nginx iproute2 openssl && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -m -d /var/lib/montana -s /usr/sbin/nologin montana && \
    mkdir -p /etc/montana /etc/montana-vpn /usr/local/share/xray /var/log/xray /var/www/decoy && \
    curl -sL "https://github.com/XTLS/Xray-core/releases/download/${XRAY_VERSION}/Xray-linux-64.zip" -o /tmp/xray.zip && \
    unzip -o /tmp/xray.zip -d /usr/local/share/xray && \
    install -m 0755 /usr/local/share/xray/xray /usr/local/bin/xray && \
    rm -rf /tmp/xray.zip /usr/local/share/xray && \
    chmod 0755 /etc/montana && \
    chmod 0700 /etc/montana-vpn

COPY --from=builder /build/Code/target/release/montana-node /usr/local/bin/montana-node
COPY Code/scripts/genesis-manifest.json /etc/montana/genesis-manifest.json
COPY Code/docker/runtime/entrypoint.sh /usr/local/bin/montana-entrypoint
COPY Code/docker/runtime/decoy-index.html /var/www/decoy/index.html
COPY Code/docker/runtime/nginx-decoy.conf /etc/nginx/sites-available/default
RUN chmod +x /usr/local/bin/montana-entrypoint && \
    ln -sf /etc/nginx/sites-available/default /etc/nginx/sites-enabled/default

WORKDIR /var/lib/montana
VOLUME ["/var/lib/montana"]
# host network mode — exposes node :8444, xray :443, nginx :80 on host.
EXPOSE 80 443 8444

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/montana-entrypoint"]
