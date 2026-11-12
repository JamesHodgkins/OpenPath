# OpenPath

A project scheduler written in Rust, using egui for the interface. It opens Microsoft Project files, lets you edit the plan on a task sheet beside a Gantt chart, and schedules it with Microsoft Project-compatible rules.

**Status:** Phase 0, an engineering spike. The scheduling engine (calendars, critical path, constraints, slack), the native `.opath` file, MSPDI export, `.mpp` import and a first task sheet with Gantt chart work. Most of the planned feature set is not built yet. The specification is in `private/specification-v1.1.md`.

## Layout

```
crates/
  openpath-core/      project model: tasks, calendars, dependencies (no I/O, no UI)
  openpath-schedule/  calendar arithmetic, critical path, constraints, slack
  openpath-files/     native .opath file: checksummed container, atomic save
  openpath-mpp/       Microsoft Project .mpp import (through mpxj-rs)
  openpath-mspdi/     Microsoft Project XML (MSPDI) reading and writing
  openpath-ui/        theme and widgets (egui)
  openpath-app/       desktop application (eframe / egui)  -> binary: openpath
  openpath-cli/       headless tool (sample, schedule, export)  -> binary: openpath-cli
tests/corpus/         sample projects and how they are licensed
```

## Build and run

Requires the Rust stable toolchain (`rust-toolchain.toml` sets it up).

```
cargo run -p openpath-app          # desktop app
cargo run -p openpath-cli -- sample
cargo test --workspace             # all tests
cargo clippy --workspace           # lints
```

Windows and Linux are the targets. GitHub Actions (`.github/workflows/ci.yml`) runs the checks on both.

## Part of a family

OpenPath shares its conventions, look and licence with [OpenDraft](https://github.com/JamesHodgkins/OpenDraft) (2D CAD) and [OpenCalc](https://github.com/JamesHodgkins/OpenCalc) (engineering worksheets): crates are `<product>-<role>`, files use "o" plus the product word (`.odraft`, `.ocalc`, `.opath`), and the interface follows one house style.

## Licence

GNU General Public License v3.0 or later (GPL-3.0-or-later). See `LICENSE`. The `.mpp` reader, `mpxj-rs`, is LGPL-2.1-or-later and is linked under that licence's section 3.
