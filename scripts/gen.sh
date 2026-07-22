#!/usr/bin/env bash
# Run the src/gen/ generators. A generator is a Lua script invoked as
# `nvim -u NONE -i NONE --headless -l src/gen/<script> <output> <inputs...>`:
# running under the built nvim keeps outputs honest where the binary itself
# is a source of truth (e.g. gen_vimvim's builtin-function keywords come from
# getcompletion). This harness owns generator invocation end to end — binary
# resolution, output layout, mtime staleness — so callers don't re-derive any
# of it. (The one exception is scripts run by build.rs, e.g.
# src/gen/compile_lua_modules.lua: build.rs runs before any nvim exists, so
# it drives the deps-prefix luajit itself.)
#
# Usage: gen.sh [--nvim BIN] [--runtime DIR]
#
#   --nvim BIN     nvim to run the generators under. Default: target/bin/nvim
#                  (the hard link prep-test-tree.sh maintains); with the
#                  default, a missing binary is a silent no-op, because
#                  prep-test-tree.sh calls this unconditionally and the test
#                  harness reports a missing build with a better message. An
#                  explicit --nvim that is missing is an error: packagers must
#                  not silently ship without the generated files.
#   --runtime DIR  runtime tree to generate into. Default: target/runtime
#                  (the build runtime the test suites see). Packagers point
#                  this at their staged share/nvim/runtime.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

nvim=
runtime=$root/target/runtime
while [[ $# -gt 0 ]]; do
  case $1 in
    --nvim)
      nvim=$2
      shift 2
      ;;
    --runtime)
      runtime=$2
      shift 2
      ;;
    *)
      echo "gen.sh: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -n $nvim ]]; then
  if [[ ! -x $nvim ]]; then
    echo "gen.sh: not an executable nvim: $nvim" >&2
    exit 1
  fi
else
  nvim=$root/target/bin/nvim
  if [[ ! -x $nvim ]]; then
    exit 0
  fi
fi

# run <script> <output> <inputs...>: regenerate <output> from <inputs> with
# src/gen/<script>, unless it is already newer than every input (and the
# binary, and the generator itself).
run() {
  local script=$root/src/gen/$1 out=$2
  shift 2
  if [[ -f $out ]]; then
    local stale=0
    for dep in "$nvim" "$script" "$@"; do
      if [[ $dep -nt $out ]]; then
        stale=1
      fi
    done
    if [[ $stale == 0 ]]; then
      return
    fi
  fi
  mkdir -p "$(dirname "$out")"
  "$nvim" -u NONE -i NONE --headless -l "$script" "$out" "$@"
}

# The vimscript syntax keyword tables: options/commands/events/vvars from the
# vendored metadata, builtin functions from the binary itself.
run gen_vimvim.lua "$runtime/syntax/vim/generated.vim" \
  "$root/src/nvim"/{options,auevents,ex_cmds,vvars}.lua
