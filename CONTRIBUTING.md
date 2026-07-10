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

## Release process (maintainers)

Releases publish to a single channel: GitHub Releases, tagged `vX.Y.Z`. A maintainer runs the
`Release` workflow (`.github/workflows/release.yml`, manual `workflow_dispatch` — never
auto-triggered on tag push) which builds `cargo build --release`, computes a `sha256` checksum
of the resulting binary, and attaches both as release assets. There is no crates.io publish and
no container-registry publish for this project; the `Dockerfile` is provided for optional local/
service use only.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
