# Shared driver for the A/B benches beside it. Sourced, never run.
#
# Each bench defines a `run <binary>` function that prints "phase<TAB>ms"
# lines for one pass -- with `$RUNNER` in front of the binary, so the same
# invocation can be handed to valgrind -- and then calls `ab_run <name>`.
#
# Two modes:
#
#   bench.sh <nvim-A> <nvim-B> [rounds]   wall clock, interleaved
#   bench.sh --cachegrind <nvim>          instructions retired, one run
#
# What the first shares is the part the drift rule cares about: one untimed
# warm-up, then the two sides alternating inside a single session, and the
# MINIMUM per phase reported rather than the mean -- the machine is noisy and
# the minimum is the only statistic that is stable across runs. Even so, a
# wall-clock claim needs both orders and a same-binary floor; see README.md.
#
# The second answers with instruction counts instead, which are deterministic
# where wall clock is not, and needs no repetition: it prints one number for
# one binary, and two of those are compared by hand.
#
# The runtime is not a positional argument in either mode; set VIMRUNTIME to
# measure against a tree other than the one this file lives in.

RUNNER=""
MODE=wall

if [ "${1:-}" = "--cachegrind" ]; then
  if [ "$#" -ne 2 ]; then
    echo "usage: $(basename "$0") --cachegrind <nvim>" >&2
    exit 64
  fi
  MODE=cachegrind
  A=$(realpath "$2")
  B=$A
  ROUNDS=1
elif [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $(basename "$0") <nvim-A> <nvim-B> [rounds]" >&2
  echo "       $(basename "$0") --cachegrind <nvim>" >&2
  exit 64
else
  A=$(realpath "$1")
  B=$(realpath "$2")
  ROUNDS=${3:-${ROUNDS_DEFAULT:-7}}
  case "$ROUNDS" in
    '' | *[!0-9]*)
      echo "$(basename "$0"): rounds must be a number, got '$ROUNDS'." >&2
      echo "The runtime is not positional; set VIMRUNTIME instead." >&2
      exit 64
      ;;
  esac
fi

# The tree this file lives in, unless the caller names another runtime.
RUNTIME=${VIMRUNTIME:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)/runtime}
if [ ! -d "$RUNTIME" ]; then
  echo "$(basename "$0"): no runtime at $RUNTIME" >&2
  exit 66
fi

ab_run() {
  local name=$1
  local out
  out=$(mktemp -d)
  trap 'rm -rf "$out"' EXIT

  # One untimed, uninstrumented warm-up, so the measured run pays neither for
  # faulting the binary in nor for building a fixture it then reuses.
  run "$A" >/dev/null

  if [ "$MODE" = cachegrind ]; then
    RUNNER="valgrind --tool=cachegrind --cache-sim=no --branch-sim=no
      --cachegrind-out-file=$out/cachegrind.out --log-file=$out/log"
    run "$A" >/dev/null
    local refs
    refs=$(sed -n 's/^==[0-9]*== *I *refs: *//p' "$out/log" | tr -d ' ,')
    if [ -z "$refs" ]; then
      echo "$name: cachegrind printed no I refs; log follows" >&2
      cat "$out/log" >&2
      exit 70
    fi
    printf '%s\tIr\t%s\n' "$name" "$refs"
    return
  fi

  local _
  for _ in $(seq 1 "$ROUNDS"); do
    run "$A" >>"$out/a"
    run "$B" >>"$out/b"
  done

  python3 - "$name" "$out/a" "$out/b" <<'PY'
import collections
import sys


def mins(path):
    """The minimum time seen for each phase, in first-seen order."""
    best = collections.OrderedDict()
    for line in open(path):
        if "\t" not in line:
            continue
        name, value = line.rstrip("\n").split("\t")
        value = float(value)
        if name not in best or value < best[name]:
            best[name] = value
    return best


bench, a, b = sys.argv[1], mins(sys.argv[2]), mins(sys.argv[3])
if not a or not b:
    sys.exit(f"{bench}: no samples -- did the binary print its result lines?")
print(f"{'phase':16s} {'A (ms)':>10s} {'B (ms)':>10s} {'change':>9s}")
for name in a:
    if name not in b:
        continue
    change = (b[name] / a[name] - 1) * 100
    print(f"{name:16s} {a[name]:10.1f} {b[name]:10.1f} {change:8.1f}%")
total_a = sum(v for k, v in a.items() if k in b)
total_b = sum(v for k, v in b.items() if k in a)
change = (total_b / total_a - 1) * 100
print(f"{'TOTAL':16s} {total_a:10.1f} {total_b:10.1f} {change:8.1f}%")
PY
}
