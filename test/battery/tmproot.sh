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
# fixed path for every run), and where they scrub it nvim may have elided a
# message wider than the headless screen from the middle first (README.md,
# "Paths and formatting"). So baseline.sh keys its cache by slot: a cut is
# compared only with a head run in the same root. A lone battery always runs
# in /tmp/nvb1; a second one at once cuts its own baselines in nvb2, once.
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
