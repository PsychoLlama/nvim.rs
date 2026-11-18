#!/usr/bin/env bash
# Summarise the AddressSanitizer reports in one suite's log directory and fail
# when there is any. The ASan recipes point `log_path` into a directory of
# their own, emptied before the run, so every report here came from this run:
# every nvim it spawned, on every worker of a sharded run.
#
# Usage: asan-reports.sh <log dir>
set -euo pipefail

dir=${1:?usage: asan-reports.sh <log dir>}

shopt -s nullglob
reports=("$dir"/asan.log.*)

echo
echo "== AddressSanitizer reports in $dir =="
if ((${#reports[@]} == 0)); then
  echo "none"
  exit 0
fi
for report in "${reports[@]}"; do
  # The first ERROR line names the class (heap-use-after-free, SEGV, ...);
  # a file without one (a runtime warning) is still shown, and still fails.
  headline=$(grep -m1 -h 'ERROR: AddressSanitizer' "$report" || head -n1 "$report")
  printf '%s\t%s\n' "$report" "$headline"
done
echo "${#reports[@]} report(s): read each file whole; resolve nvim+0x... frames with addr2line"
exit 1
