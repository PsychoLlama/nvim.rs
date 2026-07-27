#!/usr/bin/env bash
# Regenerate the msgpack-RPC dispatch wrappers from the Rust API signatures.
#
# tools/apigen parses crates/nvim/src/nvim/api/*.rs plus the attribute spec
# (tools/apigen/functions.txt) and writes the module directory
# crates/nvim/src/nvim/api/private/dispatch_wrappers/ — a root holding the
# shared support code and one child per API source file. Unlike the unit-test
# cdefs, the output is committed: it is ordinary crate source that has to
# compile, be formatted and be measured by the ratchet like everything else.
#
# `--check` regenerates in memory and fails if the committed file differs, so
# a signature change that nobody re-generated for is a CI failure rather than
# a wrapper that quietly disagrees with the function it calls. It runs in
# `just minimal-ci` and `just refresh` regenerates for real.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

# Built with the dev-shell toolchain; RUSTFLAGS from the dev shell (-D
# warnings) is fine here.
cargo build --release --quiet --manifest-path "$root/tools/apigen/Cargo.toml"

"$root/tools/apigen/target/release/apigen" \
  --root "$root/crates/nvim" \
  --spec "$root/tools/apigen/functions.txt" \
  --out-dir "$root/crates/nvim/src/nvim/api/private/dispatch_wrappers" \
  --rustfmt-config "$root/rustfmt.toml" \
  "$@"
