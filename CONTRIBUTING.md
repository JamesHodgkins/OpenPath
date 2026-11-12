# Contributing to OpenPath

Thanks for helping. OpenPath is one of a family of desktop engineering tools (OpenDraft, OpenCalc, OpenPath) that share conventions, so a change here should read like the others.

## Set up

Install the Rust stable toolchain (`rust-toolchain.toml` selects it). On Linux, install the window-system libraries the CI workflow lists. Then:

```
cargo run -p openpath-app
```

## Conventions

- Crates are `openpath-<role>`: `-core` (model, no UI), `-ui` (theme and widgets), `-app` (the program), `-cli`, and a format crate named after its format. Never a short prefix.
- No `unwrap`, `todo!` or `dbg!` in production code; `unsafe` is forbidden.
- Files stay under about 400 lines of production code.
- Commit as yourself; keep commits focused and describe the change in the first line.

## Before you open a pull request

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Open the pull request against `main`; the `check` jobs must pass.

## Conduct and security

Please read the [Code of Conduct](CODE_OF_CONDUCT.md). Report security problems as described in [SECURITY.md](SECURITY.md), not in a public issue.
