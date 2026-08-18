#!/usr/bin/env bash
# Regenerate the API dispatch layer from the Rust API signatures.
#
# tools/apigen syn-parses the `nvim_*` signatures under crates/nvim/src/api/
# (bar private/) and the `KeyDict_*` structs in crates/nvim/src/types/
# keysets.rs — no C headers are involved — plus the attribute spec
# (tools/apigen/functions.txt), and writes five module directories — the
# msgpack-RPC wrappers, the dispatch tables, the `vim.api` Lua binding, the
# option table and the Vimscript builtin table — plus the packed api-info
# metadata. Each directory is a root holding the shared support code and one
# child per API source file. Unlike
# the unit-test cdefs, the output is committed: it is ordinary crate source
# that has to compile, be formatted and be measured by the ratchet like
# everything else.
#
# Three of them read metadata rather than Rust: the vendored
# crates/nvim/src/{options,eval,ex_cmds}.lua, the same files upstream fed
# to src/gen/gen_options.lua, src/gen/gen_eval.lua and src/gen/gen_ex_cmds.lua.
# ex_cmds.lua yields two single modules rather than a directory --- the Ex
# command table with its two lookup indices, and the CMD_* names that index it
# --- which have to be generated together or they drift apart.
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
  --metadata-spec "$root/tools/apigen/metadata.txt" \
  --out-dir "$root/crates/nvim/src/api/private/dispatch_wrappers" \
  --tables-dir "$root/crates/nvim/src/api/private/dispatch" \
  --lua-dir "$root/crates/nvim/src/lua/api_wrappers" \
  --options-lua "$root/crates/nvim/src/options.lua" \
  --options-dir "$root/crates/nvim/src/options" \
  --eval-lua "$root/crates/nvim/src/eval.lua" \
  --eval-dir "$root/crates/nvim/src/eval/funcs/table" \
  --metadata-file "$root/crates/nvim/src/api/private/metadata.rs" \
  --ex-cmds-lua "$root/crates/nvim/src/ex_cmds.lua" \
  --cmdtable-file "$root/crates/nvim/src/ex_docmd/cmdtable.rs" \
  --cmdidx-file "$root/crates/nvim/src/types/cmdidx.rs" \
  --rustfmt-config "$root/rustfmt.toml" \
  "$@"
