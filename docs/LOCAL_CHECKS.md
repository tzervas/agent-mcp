# Local checks (CI parity)

Primary quality gate is `./scripts/check.sh` — the same entrypoint the remote **CI** job runs
(`.github/workflows/ci.yml` → `Build and Test chromium (local parity)`). Fleet gates
(`fleet-ci`, `fleet-security`) are additional self-hosted checks on PRs/pushes.

## Run everything the remote job would run

```bash
./scripts/check.sh          # fmt --check + clippy -D warnings + build + test
./scripts/check.sh --fix  # apply rustfmt instead of --check
```

Env defaults set by the script (override as needed):

| Variable | Default | Why |
| --- | --- | --- |
| `CARGO_BUILD_JOBS` | `1` | Serial rustc on shared self-hosted hosts (avoids SIGKILL under mem pressure / C5) |
| `CARGO_PROFILE_*_CODEGEN_UNITS` | `1` | Serial codegen units (16 still OOM'd tokio on medium host-homelab) |
| `CARGO_INCREMENTAL` | `0` | Lower peak disk+RAM on chromiumoxide-heavy graphs |
| `CARGO_PROFILE_DEV_DEBUG` / `CARGO_PROFILE_TEST_DEBUG` | `0` | Strip debuginfo in dev/test for smaller objects |
| `CI` | unset locally | When `true` (Actions): wait until `MemAvailable` ≥ ~4 GiB, then **skip** a full `cargo build` and run `cargo test` only (test rebuilds what it needs) |

Locally you almost always want the default path (build + test). Setting `CI=true` on a quiet
laptop is fine for a dry-run of the Actions path; on a busy shared host it is the safer gate.

```bash
# Optional: force the Actions-shaped path locally
CI=true ./scripts/check.sh
```

## Shared-host / OOM notes (C5)

Compiling the `chromium` feature graph (`chromiumoxide_cdp` metadata) is peak-RAM heavy. On fleet
CPU workers that also run other MCP crates concurrently:

1. Prefer `CARGO_BUILD_JOBS=1` (script default).
2. Do not start a second full workspace compile while another agent job is mid-rustc.
3. Under `CI=true`, `check.sh` polls `/proc/meminfo` for ~15 minutes max before starting cargo.

If a remote job dies with exit 101 / SIGKILL mid-`rustc`, treat it as host memory pressure first —
re-queue after quieter slots rather than assuming a logic regression.

## Tero index

```bash
# from a checkout that can see the generator (sibling tero-mcp recommended):
python3 ../tero-mcp/scripts/generate_lite_index.py --root "$(pwd)"
# or:
python3 scripts/generate_tero_index.sh   # if present as a thin wrapper
```

Artifacts land in `docs/tero-index/` (`index.json`, `INDEX.md`, `MANIFEST.toml`, `README.md`).

## Remote

- **CI** (local parity): pull_request / workflow_dispatch — https://github.com/tzervas/agent-mcp/actions/workflows/ci.yml
- **fleet-ci** / **fleet-security**: fleet self-hosted stack on PRs and `main`
