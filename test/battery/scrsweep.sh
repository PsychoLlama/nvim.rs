#!/usr/bin/env bash
# scrsweep — screen-pipeline differential driver.
#
#   scrsweep.sh <nvim-binary> <outdir> <label>
#
# Reproduces run-tests.sh's sandbox (XDG dirs, a SHORT $TMPDIR — nvim's server
# sockets live there and sun_path caps at ~107 bytes) and runs scrsweep.lua
# through `nvim -ll`. The binary under test is the *child* nvim ($NVIM_PRG);
# the host that runs the Lua is the same binary, which is fine because the
# host only speaks RPC.
#
# Writes <outdir>/<label>.{txt,attrs,vals}. Diff all three: the grid alone is
# not an oracle (cells carry attribute ids, not values) and neither is the
# attr dump (it cannot show where a cell went).
set -uo pipefail

nvim_bin=$(realpath "$1")
outdir=$(realpath -m "$2")
label=$3

here=$(dirname "$(realpath "$0")")
root=${SCRSWEEP_REPO:-$(dirname "$(dirname "$here")")}

mkdir -p "$outdir"

build_dir=$root/target/scrsweep
xdg=$build_dir/xdg
tmpdir=${XDG_RUNTIME_DIR:-/tmp}/nvim.rs-scrsweep-$$
chmod -R u+rwX "$xdg" 2>/dev/null || true
rm -rf "$xdg"
mkdir -p "$xdg" "$tmpdir"
ln -sfn "$root/runtime" "$xdg/runtime"
ln -sfn "$root/test" "$xdg/test"

# Syntax fixtures for the runtime/syntax scenario; generated fresh so the
# corpus travels with the script instead of living in the repo.
fixtures=$build_dir/fixtures
mkdir -p "$fixtures"
cat > "$fixtures/fixture.vim" <<'EOF'
" a vim script fixture
let s:name = 'value'   " trailing comment
function! s:Fn(a, ...) abort
  if a:a ==# 'x' | return [1, 2.5, v:true] | endif
  for i in range(10)
    echo printf('%d %s', i, s:name)
  endfor
  return {'k': 'v'}
endfunction
augroup Fix | autocmd! | autocmd BufRead * call s:Fn('x') | augroup END
EOF
cat > "$fixtures/fixture.c" <<'EOF'
/* a C fixture */
#include <stdio.h>
#define MAX(a, b) ((a) > (b) ? (a) : (b))
typedef struct { int x; char *s; } pair_t;
static int fn(const char *s, unsigned n)
{
  // line comment
  for (unsigned i = 0; i < n; i++) {
    if (s[i] == '\n') return -1;
  }
  return MAX(0, (int)n);
}
EOF
cat > "$fixtures/fixture.lua" <<'EOF'
-- a lua fixture
local M = {}
--[[ block
     comment ]]
function M.go(a, b)
  local t = { 1, 2.5, 'three', [4] = true }
  for k, v in pairs(t) do
    print(('%s=%s'):format(k, tostring(v)))
  end
  return a and b or nil
end
return M
EOF
cat > "$fixtures/fixture.sh" <<'EOF'
#!/bin/sh
# a shell fixture
set -eu
name="world"
for i in 1 2 3; do
  printf 'hello %s %d\n' "$name" "$i"
done
case "${1:-}" in
  -h|--help) echo usage; exit 0 ;;
  *) : ;;
esac
EOF
cat > "$fixtures/fixture.diff" <<'EOF'
diff --git a/x b/x
index 1234567..89abcde 100644
--- a/x
+++ b/x
@@ -1,4 +1,4 @@
 context line
-removed line
+added line
 tail
EOF
# `:syntax include` sources a file of `:syntax` commands with an inclusion
# tag pushed, which is the only way to reach `syn_incl_toplevel`,
# HL_INCLUDED_TOPLEVEL and the `inc_tag` half of `in_id_list`.
cat > "$fixtures/fixture.syn" <<'EOF'
syntax keyword incKey INCKEY
syntax match incMatch /incm/
syntax region incReg start=/INCBEG/ end=/INCEND/ contains=incInner
syntax match incInner /deep/ contained
hi link incKey Statement
hi link incMatch Constant
hi link incReg Comment
hi link incInner Todo
EOF

cd "$xdg" || exit 1
NVIMRS_ROOT=$root \
SCRSWEEP_FIXTURES=$fixtures \
NVIM_TEST=1 \
LC_ALL=en_US.UTF-8 \
VIMRUNTIME=$root/runtime \
XDG_CONFIG_HOME=$xdg/config \
XDG_DATA_HOME=$xdg/share \
XDG_STATE_HOME=$xdg/state \
NVIM_LOG_FILE=$build_dir/nvim.log \
NVIM_PRG=$nvim_bin \
TMPDIR=$tmpdir \
HISTFILE=/dev/null \
SHELL=sh \
SYSTEM_NAME=$(uname -s) \
  timeout -k 5 "${SCRSWEEP_TIMEOUT:-180}" \
  "$nvim_bin" -ll "$here/scrsweep.lua" "$outdir" "$label"
rc=$?
# A clean run is ~5 s. The timeout is not belt-and-braces: a mutant that makes
# a decoration provider throw on every redraw wedges the child nvim and the
# RPC read blocks forever, which hung a whole mutation run.
if [ "$rc" -ge 124 ]; then
  echo "scrsweep $label: TIMED OUT after ${SCRSWEEP_TIMEOUT:-180}s" >&2
  pkill -f "scrsweep.lua $outdir $label" 2>/dev/null
fi

rm -rf "$tmpdir"
exit $rc
