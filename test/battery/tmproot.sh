# Sourced by the rows: $BATTERY_TMP, the root this run keeps its fixed-name
# work directories under (`$BATTERY_TMP/ausweep`, `$BATTERY_TMP/ksweep`, ...).
#
# The first script of a run to source this (battery.sh, or a row run by hand)
# takes the lowest free slot, /tmp/nvb1 ... /tmp/nvb9, holding
# /tmp/nvb<N>.lock for as long as it runs, and exports the root to every row
# under it: two batteries at once never share a work directory.
#
# A slot, not a `mktemp -d`: the root's NAME reaches the reports. Most of
# these rows print their work directory's path unscrubbed (it used to be one
# fixed path for every run), and nvim may elide a message wider than the
# headless screen (README.md, "Paths and formatting"), so the path must not
# vary in length. Every slot root is the same length, so a message lays out
# alike in any slot and only the digit differs; slotscrub.py masks it, and
# baseline.sh keeps one cut cache for all nine slots.
# Rows that take a `mktemp -d` of their own (constant length, scrubbed whole)
# keep it.
if [[ -z ${BATTERY_TMP:-} ]]; then
  for slot in 1 2 3 4 5 6 7 8 9; do
    exec {battery_lock}>"/tmp/nvb$slot.lock"
    if flock -n "$battery_lock"; then
      BATTERY_TMP=/tmp/nvb$slot
      break
    fi
    exec {battery_lock}>&-
  done
  if [[ -z ${BATTERY_TMP:-} ]]; then
    echo "tmproot: all nine battery slots (/tmp/nvb1..9) are in use" >&2
    exit 1
  fi
  unset slot
fi
export BATTERY_TMP
mkdir -p "$BATTERY_TMP"
