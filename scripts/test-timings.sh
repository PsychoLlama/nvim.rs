# Sourced by the test harnesses: the per-file timing table each run writes,
# so the slowest files are a list and a split across workers can balance by
# time.
#
# A table is TSV, slowest first, with a header row:
#
#   seconds  tests  file
#
# Each run writes its own (in its run root) and folds it into a stable copy,
# <build dir>/timings/<suite>.tsv: rows for the files that ran replace their
# old rows, the rest are kept, so a run of one spec does not shrink the table
# to one line.

timings_header=$'seconds\ttests\tfile'

# timings_sort <rows-file>: header + rows (no header), slowest first.
timings_sort() {
  echo "$timings_header"
  sort -t$'\t' -k1,1gr -k3,3 "$1"
}

# timings_fold <run table> <stable table>: merge the run's rows into the
# stable table (the run's row wins for a file in both). Under a lock: the
# workers of a sharded run each fold their own rows as they finish.
timings_fold() {
  local run=$1 stable=$2
  [[ -s $run ]] || return 0
  mkdir -p "$(dirname "$stable")"
  (
    flock 9
    timings_fold_locked "$run" "$stable"
  ) 9>"$stable.lock"
}

timings_fold_locked() {
  local run=$1 stable=$2 rows
  rows=$(mktemp "$stable.XXXXXX")
  local inputs=("$run")
  [[ -f $stable ]] && inputs+=("$stable")
  awk -F'\t' 'FNR == 1 { next } NR == FNR { seen[$3] = 1; print; next } !($3 in seen)' \
    "${inputs[@]}" >"$rows"
  timings_sort "$rows" >"$rows.sorted"
  mv -f "$rows.sorted" "$stable"
  rm -f "$rows"
}
