#!/usr/bin/env bash
# Write the third-party licence notices that ship with the programs:
# `scripts/third_party.sh <output file>`. The Rust dependencies come from
# `cargo about` (settings in about.toml, layout in about.hbs); the fonts and
# icons that are not Rust crates are appended from the licence files kept
# beside them. Needs cargo-about (`cargo install cargo-about --locked --features cli`).
set -euo pipefail
cd "$(dirname "$0")/.."

out=${1:?usage: scripts/third_party.sh <output file>}

cargo about generate --workspace --locked --fail about.hbs > "$out"

append() {
  {
    printf '\n================================================================================\n'
    printf '%s\n' "$1"
    printf '================================================================================\n\n'
    cat "$2"
    printf '\n'
  } >> "$out"
}

append "Inter (the interface font), crates/openpath-ui/assets/fonts" crates/openpath-ui/assets/fonts/LICENSE-Inter.txt
append "Lucide (the icon font), crates/openpath-ui/assets/fonts" crates/openpath-ui/assets/fonts/LICENSE-Lucide.txt

echo "$out"
