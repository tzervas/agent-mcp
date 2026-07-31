#!/usr/bin/env bash
# Local parity with .github/workflows/ci.yml.
# Primary quality gate — see docs/LOCAL_CHECKS.md and AGENTS.md.
set -euo pipefail
cd "$(dirname "$0")/.."
MODE="${1:-}"
export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
# Prefer serial compile on shared self-hosted hosts (avoids SIGKILL under mem pressure).
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
# Thin rustc units — chromiumoxide_cdp is a single huge crate; lower peak RSS (C5).
export CARGO_PROFILE_DEV_CODEGEN_UNITS="${CARGO_PROFILE_DEV_CODEGEN_UNITS:-16}"
export CARGO_PROFILE_TEST_CODEGEN_UNITS="${CARGO_PROFILE_TEST_CODEGEN_UNITS:-16}"

TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
CARGO=(cargo)
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy --toolchain "$TOOLCHAIN" >/dev/null 2>&1 || true
  CARGO=(cargo "+$TOOLCHAIN")
fi

# Host flock: only one heavy agent-mcp cargo graph on the box at a time (shared fleet).
# Prevents concurrent chromiumoxide_cdp compiles from SIGKILL (signal 9).
LOCK_FD=
if [[ "${CI:-}" == "true" ]]; then
  LOCK_PATH="${AGENT_MCP_CARGO_LOCK:-/tmp/gha-cargo-heavy.lock}"
  exec {LOCK_FD}>"$LOCK_PATH" || true
  if [[ -n "${LOCK_FD}" ]]; then
    echo "acquiring host cargo lock ${LOCK_PATH} (C5 serialize)"
    flock -w 900 "${LOCK_FD}" || echo "WARN: flock timeout — proceeding without exclusive lock"
  fi
  # Wait for quieter host before peak compile (xlarge claim still shares host RAM).
  for _ in $(seq 1 60); do
    avail_kb=$(awk '/MemAvailable:/ {print $2}' /proc/meminfo 2>/dev/null || echo 99999999)
    if [[ "${avail_kb}" -ge 6291456 ]]; then
      echo "MemAvailable=${avail_kb}kB — starting cargo gates"
      break
    fi
    echo "low MemAvailable=${avail_kb}kB — waiting for quieter host (C5)"
    sleep 15
  done
fi

if [[ "$MODE" == "--fix" ]]; then
  "${CARGO[@]}" fmt
else
  "${CARGO[@]}" fmt --check
fi
"${CARGO[@]}" clippy --all-targets --all-features -- -D warnings
# Under CI, skip a full `cargo build` — `cargo test` rebuilds what it needs and
# a second full graph compile doubles peak RAM risk for chromiumoxide_cdp.
if [[ "${CI:-}" == "true" ]]; then
  "${CARGO[@]}" test --all-features --verbose
else
  "${CARGO[@]}" build --all-features
  "${CARGO[@]}" test --all-features --verbose
fi
echo "OK: checks passed ($(basename "$PWD"))"
