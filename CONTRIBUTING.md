# Contributing to tronz

Thank you for your interest in contributing!

## Getting started

1. Fork the repository and clone it locally.
2. Make sure you have a recent stable Rust toolchain (`rustup update stable`).
3. Build and test everything:

```bash
cargo build --workspace
cargo test  --workspace
```

## Code style

We use `rustfmt` with the configuration in [`rustfmt.toml`](./rustfmt.toml):

```bash
cargo fmt --all
```

Linting via Clippy:

```bash
cargo clippy --workspace --all-features -- -D warnings
```

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/). Examples:

- `feat(provider): add get_block_by_hash`
- `fix(primitives): correct Address checksum encoding`
- `chore: bump alloy-primitives to 1.1`

## Pull requests

- One logical change per PR.
- Add or update tests for any new behaviour.
- Update `CHANGELOG.md` under `[Unreleased]` with a brief description.
- PRs are squash-merged; the PR title becomes the commit message.

## License

By contributing, you agree that your contributions will be dual-licensed under
[MIT](./LICENSE-MIT) and [Apache-2.0](./LICENSE-APACHE), matching the project licence.
