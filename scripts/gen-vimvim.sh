#!/usr/bin/env bash
# Generate the vimscript syntax keyword lists (syntax/vim/generated.vim) the
# way upstream's build did: src/gen/gen_vimvim.lua over the option/command/
# function metadata, with funcs_data.mpack coming from the reconstructed
# upstream build (scripts/prep-unit-headers.sh).
#
# Like upstream, the file lands in the *build* runtime (target/runtime), not
# the source runtime: test expectations (e.g. startup_spec) assume the default
# runtime does not carry it, and only specs that opt in via
# add_builddir_to_rtp() see it.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"

out=$root/target/runtime/syntax/vim/generated.vim
if [[ -f $out ]]; then
  exit 0
fi

"$root/scripts/prep-unit-headers.sh"

up=$root/target/upstream
mkdir -p "$(dirname "$out")"
"$NVIM_DEPS_PREFIX/bin/luajit" \
  "$up/src/src/gen/preload_nlua.lua" \
  "$up/src" \
  "$up/build/lib/libnlua0.so" \
  "$up/build" \
  "$up/src/src/gen/gen_vimvim.lua" \
  "$out" \
  "$up/build/funcs_data.mpack" \
  "$up/src/src/nvim/options.lua" \
  "$up/src/src/nvim/auevents.lua" \
  "$up/src/src/nvim/ex_cmds.lua" \
  "$up/src/src/nvim/vvars.lua"
