set indentation := "  "
set default-list

# Recipes run from the module file's directory by default; the `scripts/...`
# and `test/...` paths below are relative to the project root.
set working-directory := ".."

# AddressSanitizer builds (the phase-1 safety net from the migration plan).
# All heap allocation flows through libc malloc, so ASan's interposer catches
# use-after-free/double-free/out-of-bounds across the whole binary even though
# the C deps are not (yet) instrumented. An explicit --target keeps RUSTFLAGS
# off host build scripts; -Zbuild-std instruments std's own allocations; a
# separate target dir keeps ASan artifacts out of the normal build cache.
triple := `rustc -vV | sed -n 's/^host: //p'`
bin := justfile_directory() / "target/asan" / triple / "debug/nvim"

# ASan reports go to files (nvim is a TUI; stderr is often a terminal the
# harness owns). Leak checking is off in the default options because a leak
# is not a corruption and every report would be noise on a run looking for
# one; `leaktest` below turns it on. Callers may override by exporting
# ASAN_OPTIONS.
options := "detect_leaks=0:log_path=" + justfile_directory() / "target/asan/asan.log"

# The same, with LeakSanitizer on.
#
# `exitcode=0` is not optional. LSan exits non-zero when it reports, and the
# functional harness reads a child nvim's exit status: with the default
# exitcode the whole run dies before the first test, with no output that says
# why. Measured.
leakoptions := "detect_leaks=1:exitcode=0:log_path=" + justfile_directory() / "target/asan/asan.log"

# Compile the nvim binary with AddressSanitizer. Stack-redzone checking is
# on again: the 2023 nightly's C-ABI lowering over-read by-value aggregates
# (12-byte pos_T read with a 16-byte load), which forced -asan-stack=0; the
# current toolchain's cast-ABI fixes lower those loads exactly.
build:
  RUSTFLAGS="-Zsanitizer=address" cargo build -Zbuild-std --target {{ triple }} --target-dir target/asan

# setarch -R disables ASLR (inherited by every spawned nvim): this toolchain's
# ASan runtime predates LLVM's fix for kernels with 32-bit mmap entropy, so
# with ASLR on, ~1/3 of processes randomly land a mapping inside ASan's fixed
# shadow regions and SIGSEGV in the dynamic loader before main.
#
# Reports land in target/asan/asan.log.<pid>.
#
# Run functional tests against the ASan build. Args as for `functionaltest`.
functionaltest *args: build
  #!/usr/bin/env bash
  set -euo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ options }}}"
  export NVIM_TEST_ASAN=1
  # ASan frames are several times larger and the debug GlobalCell checks add
  # non-inlined calls, so the deliberately-recursive specs (E169) overflow
  # the default 8MB stack; give the whole tree headroom.
  ulimit -s 65536
  # Tests run target/bin/nvim (the layout specs assert); prep-test-tree.sh
  # points it at $NVIM_BIN.
  export NVIM_BIN={{ bin }}
  exec setarch "$(uname -m)" -R scripts/run-tests.sh functional {{ args }}

# Reports land in target/asan/asan.log.<pid>.
#
# Run old (Vim) tests against the ASan build. Args as for `oldtest`, whose
# mode (`all`, test names, or `clean`) is likewise required.
oldtest +args: build
  #!/usr/bin/env bash
  set -euo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ options }}}"
  export NVIM_TEST_ASAN=1
  ulimit -s 65536
  export NVIM_BIN={{ bin }}
  export NVIM_PRG={{ justfile_directory() }}/target/bin/nvim
  exec setarch "$(uname -m)" -R scripts/run-oldtest.sh {{ args }}

# Run functional tests with LEAK CHECKING ON, then summarise every report.
# Args as for `functionaltest`.
#
# THE NOISE FLOOR IS A CONSTANT, and that is what makes this worth having:
# on a clean tree every nvim process reports exactly
#
#   SUMMARY: AddressSanitizer: 1055 byte(s) leaked in 2 allocation(s).
#
# ... which is two process-lifetime singletons that are never freed because
# the Lua state outlives them: `cjson::lua_cjson::lua_cjson_new`'s 1,023-byte
# encode buffer (registered once by `nlua_state_add_stdlib`) and
# `luv_work_init`'s 32-byte work queue (once, from `nlua_init`). Measured
# identical for a bare `-u NONE -c qa`, for a 269-line Vimscript decode
# corpus, and for all 77 cases of `vimscript/json_functions_spec.lua` — so
# ANY OTHER SUMMARY LINE IS A REAL LEAK, and the arithmetic is that simple.
#
# There is no suppression file because there is no `llvm-symbolizer` in this
# environment: LSan matches suppressions on symbolized frames, so the reports
# come back as raw module offsets and a suppression could not match. Resolve
# a frame by hand with
#   addr2line -e target/asan/<triple>/debug/nvim -f -C -p 0x<offset>
# where the offset is the `nvim+0x...` in the report.
leaktest *args: build
  #!/usr/bin/env bash
  set -uo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ leakoptions }}}"
  export NVIM_TEST_ASAN=1
  ulimit -s 65536
  export NVIM_BIN={{ bin }}
  rm -f {{ justfile_directory() }}/target/asan/asan.log.*
  setarch "$(uname -m)" -R scripts/run-tests.sh functional {{ args }}
  rc=$?
  echo
  echo "== leak reports =="
  shopt -s nullglob
  reports=({{ justfile_directory() }}/target/asan/asan.log.*)
  if [ ${#reports[@]} -eq 0 ]; then
    echo "none"
  else
    for f in "${reports[@]}"; do
      printf '%s\t%s\n' "$(basename "$f")" "$(grep -h SUMMARY "$f" | tail -1)"
    done | sort -k2 | uniq -c -f1
  fi
  exit $rc
