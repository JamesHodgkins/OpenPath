# CLAUDE.md

OpenPath: a Rust desktop project scheduler on egui/eframe with Microsoft Project interoperability. Windows and Linux are the targets. Licence GPL-3.0-or-later. One of three sibling apps with OpenDraft and OpenCalc (`../OpenDraft`, `../OpenCalc`); keep the three consistent.

## Read first
- `private/specification-v1.1.md`: the specification (section numbers are cited in code comments). Appendix G logs decisions; Appendix H is the shared house style.
- `private/phase0/STATUS.md`: what Phase 0 has done and what is open.

## Crates (dependencies point down)
`openpath-core` (model; no I/O, parsers or UI) <- `openpath-schedule` (engine), `openpath-files` (native `.opath`), `openpath-mpp` (`.mpp` import via `mpxj-rs`), `openpath-mspdi` (MSPDI) <- `openpath-app` (binary `openpath`) and `openpath-cli` (binary `openpath-cli`). `openpath-ui` is the theme and widget library the app draws with.

## Naming (shared by all three apps)
- Crates are `<product>-<role>` with the full lowercase product name, never a short prefix (a later app would collide). Roles: `-core`, `-ui` (widgets only), `-app` (the program), `-cli`, `-files`, and a format crate named after its format.
- Files: "o" plus the product word: `.opath` here (`.odraft`, `.ocalc` in the siblings). Environment variables are `OPENPATH_*`.

## Conventions
- No `unwrap`, `todo!`, `dbg!` or prints in production code; `unsafe` is forbidden (the lints are in the workspace `Cargo.toml`; clippy does not yet enforce `unwrap` here, but new code should not add them).
- Files stay under about 400 production lines.
- Rust types from `mpxj-rs` (`Rc<RefCell<..>>`) never leave `openpath-mpp`.

## Checks (CI runs these on Windows and Linux)
```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
Gate commits on the real exit codes (do not pipe cargo into grep and then commit).

## Working with the user (James)
- Commit as James <133605871+JamesHodgkins@users.noreply.github.com> with NO Co-Authored-By trailer. Never use any other email address; check `git config user.email` first.
- Work on a feature branch and push only when asked. Stage with `git add -u` plus named new files; check `git status` first.
- Finish with a short report: what changed, what is not done, and what to take next. Do not start new feature work without confirmation.
