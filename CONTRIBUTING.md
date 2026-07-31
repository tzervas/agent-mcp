# Contributing to This Project

Thank you for your interest in contributing!

## Development Setup

1. Clone the repository
2. Ensure you have Rust 1.84+ installed
3. Run `cargo build` to build
4. Run `cargo test` to run tests

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## Code Style

- Use `cargo fmt` for formatting
- No clippy warnings (`cargo clippy -- -D warnings`)
- Add tests for new functionality
- Update documentation as needed

## Versioning

`embeddenator-agent-mcp` is versioned **0.x.y** and stays there. Commitizen enforces this with
`major_version_zero = true` in [`.cz.toml`](.cz.toml). Moving to **1.x.x requires an explicit
human authorization** — full production readiness, hardening, and a maintainer decision. No
agent, and no automation, may cut or propose a 1.x.x release.

### Under `major_version_zero`, MINOR is the breaking position

This is the detail most often got wrong. While the major is pinned at 0:

| Change                        | Bump      | Example           |
| ----------------------------- | --------- | ----------------- |
| `fix:`                        | PATCH     | 0.2.1 → 0.2.2     |
| `feat:`                       | PATCH     | 0.2.1 → 0.2.2     |
| `feat!:` / `BREAKING CHANGE:` | **MINOR** | 0.2.1 → **0.3.0** |

A consumer pinning "latest compatible" therefore pins the **minor** — `"0.2"`, or a moving tag
`v0.2`. Never `"1"`, and never a bare `v1` tag: under this scheme `0.2` and `0.3` are
*incompatible* releases, exactly as `1.x` and `2.x` would be after a 1.0 cut.

With `major_version_zero` **absent**, commitizen treats a breaking change as MAJOR and mints
`1.0.0` on the first `feat!:` — a version nobody authorized.

### Version files

The version appears in more than one place. [`.cz.toml`](.cz.toml) lists each one under
`version_files` so `cz bump` moves them together and they cannot drift:

- `Cargo.toml` — `[package] version`
- `README.md` — the `**Status (vX.Y.Z).**` line
- `.cz.toml` itself — `version`

`Cargo.lock` records this package's own version too; refresh it with `cargo build` after a bump.
Do not hand-edit any of these — run the tool:

```bash
cz bump --yes --dry-run     # show what would happen, change nothing
cz bump                     # move every version file + create the tag
cz version --project        # what this project currently claims to be
```

The `version` key in `.cz.toml` is the version cz bumps *from*, so it must track the newest
released tag. If it lags behind the tag list, the next `cz bump` re-mints a version that already
has a tag.

## Release process (maintainers)

**A GitHub Release is not a registry publication.** A git tag with notes attached publishes
nothing consumable; only a crates.io publish produces an artifact a dependent can resolve.
`embeddenator-agent-mcp` has **never been published to crates.io**, by design (see below). Say
*where* a version shipped when you claim it shipped.

Releases publish to a single channel: GitHub Releases, tagged `vX.Y.Z`. A maintainer runs the
`Release` workflow (`.github/workflows/release.yml`, manual `workflow_dispatch` — never
auto-triggered on tag push) which builds `cargo build --release`, computes a `sha256` checksum
of the resulting binary, and attaches both as release assets. There is no crates.io publish and
no container-registry publish for this project; the `Dockerfile` is provided for optional local/
service use only.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
