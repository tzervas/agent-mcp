# syntax=docker/dockerfile:1
#
# agent-mcp — multi-agent orchestration MCP server (rmcp SDK, stdio transport).
#
# Multi-stage: a Rust builder compiles the release binary; a slim Debian runtime
# carries just the binary + TLS roots. The `agent_*` prompt tools drive a browser
# via webpuppet at call time — a headless Chromium must be provided by the host /
# mounted in for those tools; the MCP server itself (handshake, tools/list, and the
# browser-free tools) runs without one.

# ---- builder ---------------------------------------------------------------
# rmcp 0.8.5 is an edition-2024 crate ⇒ rustc >= 1.85 (see Cargo.toml rust-version).
FROM rust:1-slim-bookworm AS builder

# ring/rustls need a C toolchain; webpuppet's git dep is fetched over https.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential pkg-config ca-certificates git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build --release --locked
RUN strip target/release/agent-mcp || true

# ---- runtime ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 agent

COPY --from=builder /build/target/release/agent-mcp /usr/local/bin/agent-mcp

USER agent
WORKDIR /home/agent

# MCP is spoken over stdio; keep stdout clean (logs go to stderr).
ENTRYPOINT ["/usr/local/bin/agent-mcp"]
