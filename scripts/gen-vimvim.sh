#!/usr/bin/env bash
# Generate the vimscript syntax keyword lists (syntax/vim/generated.vim) the
# way upstream's build did: src/gen/gen_vimvim.lua over the option/command/
# event/variable metadata (src/nvim/*.lua), run by the built nvim itself
# (which also supplies the builtin-function list, via getcompletion).
#
# Like upstream, the file lands in the *build* runtime (target/runtime), not
# the source runtime: test expectations (e.g. startup_spec) assume the default
# runtime does not carry it, and only specs that opt in via
# add_builddir_to_rtp() see it.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

out=$root/target/runtime/syntax/vim/generated.vim
nvim=$root/target/bin/nvim

# Regenerate when any input is newer. A missing binary is not an error here:
# prep-test-tree.sh calls this unconditionally, and the harness reports the
# missing build with a better message ("run `just build` first").
if [[ ! -x $nvim ]]; then
  exit 0
fi
if [[ -f $out ]]; then
  stale=0
  for src in "$nvim" "$root/src/gen/gen_vimvim.lua" "$root/src/nvim"/{options,auevents,ex_cmds,vvars}.lua; do
    if [[ $src -nt $out ]]; then
      stale=1
    fi
  done
  if [[ $stale == 0 ]]; then
    exit 0
  fi
fi

mkdir -p "$(dirname "$out")"
"$nvim" -u NONE -i NONE --headless \
  -l "$root/src/gen/gen_vimvim.lua" \
  "$out" \
  "$root/src/nvim/options.lua" \
  "$root/src/nvim/auevents.lua" \
  "$root/src/nvim/ex_cmds.lua" \
  "$root/src/nvim/vvars.lua"
