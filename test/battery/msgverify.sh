#!/usr/bin/env bash
# Build the working tree and diff its msgsweep against the baseline cut
# from the binary `test/battery/BASE` pins.  Run from anywhere.
#
#   msgverify.sh [label]          # default label: head
#
# All three artifacts are compared (see msgsweep.sh for what each holds).
#
# THE CASE COUNT IS AN ASSERTION OF ITS OWN.  `.vals` ends with
# `## LAST <n>`, the number of the last case the driver reached, and this
# script fails unless it is EXPECTED_CASES below.  A driver that returns
# early on both sides (the `expr` row lost half its cases that way for four
# slices while it read IDENTICAL) diffs clean; it cannot also reach the
# last case.  A slice that adds cases bumps the number in the same commit.
#
# TAKES ~30 s debug per side; a cold run cuts the base first.
set -uo pipefail

EXPECTED_CASES=697

here=$(dirname "$(realpath "$0")")
root=${MSGSWEEP_REPO:-$(dirname "$(dirname "$here")")}
label=${1:-head}
out=${MSGVERIFY_OUT:-$root/target/msgsweep/verify}
cut=
if [[ ${1:-} == --cut ]]; then cut=$2; out=$3; label=base; fi
nvim_bin=${cut:-$root/target/debug/nvim}

mkdir -p "$out"

if [[ -z $cut ]]; then
  echo "== build"
  ( cd "$root" && just build ) > "$out/build.log" 2>&1 || {
    echo "BUILD FAILED — see $out/build.log"; exit 1;
  }
fi

echo "== sweep"
"$here/msgsweep.sh" "$nvim_bin" "$out" "$label" || exit 1
if [[ -n $cut ]]; then exit 0; fi

base=${MSG_BASELINE:-$("$here/baseline.sh" msg)}

rc=0
for ext in txt attrs vals; do
  if diff -q "$base/base.$ext" "$out/$label.$ext" > /dev/null 2>&1; then
    echo "  $ext: IDENTICAL"
  else
    echo "  $ext: DIFFERS"
    # `|| true`: a non-zero diff through `head` must not end the loop
    # before the later artifacts are compared.
    diff -a "$base/base.$ext" "$out/$label.$ext" | head -60 || true
    rc=1
  fi
done
last=$(grep -aoE '^## LAST [0-9]+$' "$out/$label.vals" | awk '{print $3}')
if [[ $last != "$EXPECTED_CASES" ]]; then
  echo "  cases: DIFFERS (reached ${last:-none}, expected $EXPECTED_CASES)"
  rc=1
else
  echo "  cases: IDENTICAL ($last)"
fi
exit $rc
