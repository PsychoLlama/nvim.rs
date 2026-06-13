#!/usr/bin/env bash
# Kill ONLY orphaned test-nvim processes (the repo build/bin/nvim binary).
# NEVER touches the user's editor (/nix/store/.../bin/nvim) — it matches by the
# resolved executable path of each PID, not by command-line text.
set -u
REPO_NVIM="/home/overlord/projects/neovim/neovim/build/bin/nvim"
killed=0
for pid in $(pgrep -x nvim 2>/dev/null); do
  exe="$(readlink -f "/proc/$pid/exe" 2>/dev/null)" || continue
  if [ "$exe" = "$REPO_NVIM" ]; then
    kill -9 "$pid" 2>/dev/null && killed=$((killed+1))
  fi
done
echo "swept $killed orphaned test-nvim process(es)"
