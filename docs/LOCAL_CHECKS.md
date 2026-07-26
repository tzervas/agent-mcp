# Local checks (CI parity)

Day-to-day quality gates run **locally** via `./scripts/check.sh`. Remote GitHub
Actions also run on push/PR to trunk branches (and on `workflow_dispatch`).

> **Correction (2026-07-25):** older wording claimed workflows were “manual only.”
> That was true after early polish (PR #12 era) but is **false** now:
> `ci.yml` and `fleet-ci.yml` trigger on `push` / `pull_request` to `main` /
> `dev` (and related), not only `workflow_dispatch`. Prefer live workflow files
> and [CURRENT-STATE.md](CURRENT-STATE.md) over memory.

## Run everything the local gate runs

```bash
export CARGO_BUILD_JOBS=3   # recommended on shared builders
./scripts/check.sh
```

Steps (see `scripts/check.sh`):

1. `cargo fmt --check` (or `fmt` with `--fix`)
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-features`
4. `cargo test --all-features --verbose`

Optional:

```bash
./scripts/check.sh --fix  # apply rustfmt instead of --check
```

The script still documents a `--quick` mode in older prose; the checked-in
`scripts/check.sh` only special-cases `--fix` vs default. If you need a faster
loop, run individual cargo commands yourself.

## Tero index

```bash
# from a checkout that can see the generator (sibling tero-mcp recommended):
python3 ../tero-mcp/scripts/generate_lite_index.py --root "$(pwd)"
```

Artifacts land in `docs/tero-index/` (`index.json`, `INDEX.md`, `MANIFEST.toml`, …).

## Remote

| Workflow | Typical trigger |
|----------|-----------------|
| `fleet-ci.yml` | push/PR to main\|dev + dispatch |
| `fleet-security.yml` | push/PR + schedule |
| `ci.yml` | push/PR to main\|dev\|develop + dispatch |
| issue close/reopen | PR closed/merged events |

Badge status on the README reflects **trunk** (`main`) Actions SVG, not a static green image.
See [FLEET_STANDARDS.md](FLEET_STANDARDS.md).
