# Test corpus

This directory holds the Phase 0 import corpus (P0-01) and the parity fixtures (§9.2).
It contains real and synthetic project files; **do not commit a file here without a
permission-log entry** (§9.3).

## Layout

```text
corpus/
  mpp/        # MPP14 files (Project 2010–365), one manifest per file
  mspdi/      # MSPDI XML files (real exports + synthetic)
  parity/     # 120 differential scenarios, each with an .xml and manifest
  permission-log.csv   # source, consent, and redaction record for every file
  README.md            # this file
```

## Rules

- Every fixture has a row in `permission-log.csv`. No row, no commit.
- Redact names, rates, notes, and personal data; keep structure, dates, calendars, links.
- Store large binaries with Git LFS; record a checksum in the manifest.
- The MPXJ test corpus is LGPL and contains third-party files — verify redistribution
  rights before committing any of it; use it only as a local development aid.

## Manifest format

Each fixture is accompanied by a sidecar manifest (see `manifest-schema.md`). The
manifest records the producing tool/version, all §5.2 project options, the expected
fields, and a checksum. CI compares the fidelity report against the checked-in
expectation per fixture.
