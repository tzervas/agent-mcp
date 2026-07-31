#!/usr/bin/env bash
# Local parity with .github/workflows/ci.yml (manual-only remote).
# Primary quality gate — see docs/LOCAL_CHECKS.md and CLAUDE.md.
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
# Use stable for fmt/clippy/test unless caller overrides
TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
CARGO=(cargo)
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy --toolchain "$TOOLCHAIN" >/dev/null 2>&1 || true
  CARGO=(cargo "+$TOOLCHAIN")
fi

# Shared fleet hosts run concurrent agent jobs; wait for ~4GiB free before heavy rustc
# so chromiumoxide_cdp metadata compile is less likely to be OOM-killed (C5).
if [[ "${CI:-}" == "true" ]]; then
  for _ in $(seq 1 45); do
    avail_kb=$(awk '/MemAvailable:/ {print $2}' /proc/meminfo 2>/dev/null || echo 99999999)
    if [[ "${avail_kb}" -ge 4194304 ]]; then
      echo "MemAvailable=${avail_kb}kB — starting cargo gates"
      break
    fi
    echo "low MemAvailable=${avail_kb}kB — waiting for quieter host (C5)"
    sleep 20
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
