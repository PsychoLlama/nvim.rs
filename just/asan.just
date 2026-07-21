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
# harness owns). Leak checking stays off until the corruption classes are
# triaged. Callers may override by exporting ASAN_OPTIONS.
options := "detect_leaks=0:log_path=" + justfile_directory() / "target/asan/asan.log"

# Stack-redzone checking is off (-asan-stack=0): this nightly's C-ABI lowering
# over-reads by-value aggregates whose size is not a multiple of 8 (e.g. the
# 12-byte pos_T is read with a 16-byte load), tripping stack-buffer-overflow
# on ~every by-value struct call in the transpiled code. The over-read never
# leaves the caller's frame, so it is benign outside ASan. Revisit when the
# pinned toolchain moves past the cast-ABI fixes.
#
# Compile the nvim binary with AddressSanitizer.
build:
  RUSTFLAGS="-Zsanitizer=address -Cllvm-args=-asan-stack=0" cargo build -Zbuild-std --target {{ triple }} --target-dir target/asan

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
# Run old (Vim) tests against the ASan build. Args as for `oldtest`.
oldtest *args: build
  #!/usr/bin/env bash
  set -euo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ options }}}"
  export NVIM_TEST_ASAN=1
  ulimit -s 65536
  export NVIM_BIN={{ bin }}
  scripts/prep-test-tree.sh
  exec setarch "$(uname -m)" -R make -C test/old/testdir NVIM_PRG={{ justfile_directory() }}/target/bin/nvim {{ args }}
