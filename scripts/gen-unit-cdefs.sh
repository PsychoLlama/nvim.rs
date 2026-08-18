#!/usr/bin/env bash
# Generate the unit tests' LuaJIT ffi.cdef chunk from the Rust crate.
#
# tools/ffigen parses crates/nvim/src/**/*.rs and emits a C declaration chunk.
# test/unit/testutil.lua cdefs it and the C fixture helpers compile against it
# (test/unit/fixtures/shim.h), so the declarations the specs run against always
# describe the code actually built.
#
# The chunk lives in two places. tools/ffigen/unit-cdefs.h is the committed
# golden: the assertion that ffigen still runs and still says the same thing,
# in a tool that otherwise has no test at all. target/ffi/unit-cdefs.h is the
# copy the harness compiles and cdefs, regenerated on demand.
#
# Usage: gen-unit-cdefs.sh [--if-stale|--check]
#
#   (no args)   regenerate and write both the golden and the harness copy.
#               `just ffigen`; part of `just refresh`.
#   --if-stale  the harness's mode: refresh target/ffi/unit-cdefs.h only when
#               it is older than the crate, the tool or the deny list. Never
#               writes the golden — a suite run must not silently absorb drift,
#               that is what gives --check its teeth.
#   --check     regenerate into a scratch file and diff it against the golden,
#               failing on any difference. Deliberately unconditional: the
#               mtime skip above must not be able to make this pass.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
golden=$root/tools/ffigen/unit-cdefs.h
out=$root/target/ffi/unit-cdefs.h

mode=write
case ${1:-} in
  --if-stale | --check) mode=${1#--} ;;
  "") ;;
  *) exit_usage=1 ;;
esac
if [[ -n ${exit_usage:-} || $# -gt 1 ]]; then
  echo "usage: $(basename "$0") [--if-stale|--check]" >&2
  exit 2
fi

if [[ $mode == if-stale && -f $out ]]; then
  newer=$(find "$root/crates/nvim" "$root/tools/ffigen/src" \
    -name '*.rs' -newer "$out" -print -quit 2>/dev/null)
  deny_newer=$(find "$root/tools/ffigen/deny.txt" -newer "$out" -print -quit)
  if [[ -z $newer && -z $deny_newer ]]; then
    exit 0
  fi
fi

# The tool builds with the dev-shell toolchain. RUSTFLAGS from the dev shell
# (-D warnings) is fine here.
cargo build --release --quiet --manifest-path "$root/tools/ffigen/Cargo.toml"

if [[ $mode == if-stale ]]; then
  dest=$out
else
  dest=$(mktemp "${TMPDIR:-/tmp}/unit-cdefs.XXXXXX.h")
  trap 'rm -f "$dest"' EXIT
fi
mkdir -p "$(dirname "$out")"

echo "generating unit-test cdefs from the crate" >&2
# --root is the crate dir, not the repo root: ffigen walks <root>/src and
# derives its keys ("src/...") relative to it. Those keys are internal — the
# cimport paths the unit specs pass are labels testutil rewrites away — so
# they stayed stable across the crates/nvim move and the src/nvim flatten.
"$root/tools/ffigen/target/release/ffigen" \
  --root "$root/crates/nvim" \
  --deny "$root/tools/ffigen/deny.txt" \
  --out "$dest" \
  2> >(grep -Ev '^ffigen: (parsing|[0-9]+ type names|emitted)' >&2 || true)

case $mode in
  check)
    if ! diff -u --label "tools/ffigen/unit-cdefs.h" --label "regenerated" \
      "$golden" "$dest"; then
      echo "gen-unit-cdefs: the committed cdefs chunk is stale." >&2
      echo "Run \`just ffigen\` and commit tools/ffigen/unit-cdefs.h." >&2
      exit 1
    fi
    ;;
  write)
    # install, not cp: mktemp made $dest 0600 and the golden is a tracked file.
    install -m 644 "$dest" "$golden"
    install -m 644 "$dest" "$out"
    ;;
esac
