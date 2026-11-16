# Sourced by the test harnesses: catch a test child that logs into the user's
# own Nvim log instead of the run's.
#
# A debug build logs DBG lines from startup on, and a child started with a
# fresh environment (or before the harness exported its own NVIM_LOG_FILE)
# falls back to $XDG_STATE_HOME/nvim/nvim.log -- the file the user's own
# editor writes. Unnoticed, the old tests once grew it to 60 GB.
#
#   user_log_mark    before the run (and before the harness moves XDG_*)
#   user_log_check   after it: prints what a debug build appended and
#                    returns 1; the user's own release Nvim logs no DBG
#                    lines, so its writes during the run do not count.
#
# The file is only read (from the mark on), never written.

user_log_path=${XDG_STATE_HOME:-$HOME/.local/state}/nvim/nvim.log

user_log_size() {
  stat -c %s "$user_log_path" 2>/dev/null || echo 0
}

user_log_mark() {
  user_log_start=$(user_log_size)
}

user_log_check() {
  local now appended
  now=$(user_log_size)
  ((now > user_log_start)) || return 0
  # At most 16 MiB of what was appended: enough to name the culprits.
  appended=$(tail -c +"$((user_log_start + 1))" "$user_log_path" | head -c 16777216 |
    grep -a '^DBG ' || true)
  [[ -n $appended ]] || return 0
  {
    echo "error: a test child wrote $((now - user_log_start)) bytes of debug log to $user_log_path"
    echo "(pass it the run's NVIM_LOG_FILE). The processes, by their first lines:"
    awk '!seen[$3]++' <<<"$appended" | cut -c1-200 | head -20
  } >&2
  return 1
}
