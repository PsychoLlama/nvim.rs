_:
  just --list

# Build neovim using system dependencies from nix
build:
    cargo build --release
    @test -f build/build.ninja || cmake -B build -G Ninja -DUSE_BUNDLED=OFF
    @# Force relink when Rust library is newer than the binary (cmake doesn't track imported libs)
    @if [ -f build/bin/nvim ] && [ target/release/libnvim_rs.a -nt build/bin/nvim ]; then rm -f build/bin/nvim; fi
    cmake --build build

# Build with bundled dependencies (downloads deps)
build-bundled:
    cmake -B build -G Ninja
    cmake --build build

# Configure only (no build)
configure:
    cmake -B build -G Ninja -DUSE_BUNDLED=OFF

# Build only (after configure)
compile:
    cmake --build build

# Clean build directory
clean:
    rm -rf build
    cargo clean

# Run all tests (unit + functional)
test: unittest functionaltest

# Run functional tests
functionaltest:
    cmake --build build --target functionaltest

# Run unit tests
unittest:
    cmake --build build --target unittest

# Run the built nvim
run *ARGS:
    VIMRUNTIME=runtime ./build/bin/nvim {{ARGS}}

# Smoke test: verify nvim starts, exits cleanly, and can do basic regexp work.
# The regexp smoke test has a 30s timeout to catch infinite loops without
# leaving zombie processes. It exercises buffer search (vim_regexec_multi),
# substitution, and syntax highlighting — paths the baseline test doesn't cover.
smoke-test:
    @just smoke-test-run
    @just smoke-test-regexp

smoke-test-run:
    NVIM=./build/bin/nvim VIMRUNTIME=./runtime timeout 30 expect scripts/open_file.exp justfile just || { echo "FAIL: nvim startup smoke test timed out or failed (exit $?)"; exit 1; }

smoke-test-regexp:
    timeout 30 bash -c 'VIMRUNTIME=runtime ./build/bin/nvim --headless --clean -S test/regexp_smoke.vim 2>&1' || { echo "FAIL: regexp smoke test timed out or failed (exit $?)"; exit 1; }

# Show nvim version
version:
    VIMRUNTIME=runtime ./build/bin/nvim --version

# === Rust Commands ===

# Build Rust components
rust-build:
    cargo build --release

# Build Rust components (debug)
rust-build-debug:
    cargo build

# Run Rust tests for pure Rust crates (no FFI linking needed)
# These crates don't call into C code, only export functions to C
rust-test:
    cargo nextest run \
      --show-progress=none \
      --status-level=fail \
      --cargo-quiet \
      -p nvim-arglist \
      -p nvim-api \
      -p nvim-arabic \
      -p nvim-ascii \
      -p nvim-autocmd \
      -p nvim-bufwrite \
      -p nvim-change \
      -p nvim-charset \
      -p nvim-clipboard \
      -p nvim-cmdexpand \
      -p nvim-cmdline \
      -p nvim-cmdhist \
      -p nvim-collections \
      -p nvim-compositor \
      -p nvim-context \
      -p nvim-cursor \
      -p nvim-cursor-shape \
      -p nvim-debugger \
      -p nvim-decoration \
      -p nvim-drawline \
      -p nvim-edit \
      -p nvim-encoding \
      -p nvim-eval \
      -p nvim-event \
      -p nvim-ex-cmds \
      -p nvim-ex-cmds2 \
      -p nvim-ex-docmd \
      -p nvim-ex-eval \
      -p nvim-extmark \
      -p nvim-fileio \
      -p nvim-fuzzy \
      -p nvim-getchar \
      -p nvim-grid \
      -p nvim-help \
      -p nvim-input \
      -p nvim-highlight \
      -p nvim-indent \
      -p nvim-insexpand \
      -p nvim-keycodes \
      -p nvim-linematch \
      -p nvim-lua \
      -p nvim-main \
      -p nvim-mapping \
      -p nvim-mark \
      -p nvim-marktree \
      -p nvim-match \
      -p nvim-math \
      -p nvim-memfile \
      -p nvim-memline \
      -p nvim-memory \
      -p nvim-memutil \
      -p nvim-menu \
      -p nvim-mouse \
      -p nvim-msgpack \
      -p nvim-normal \
      -p nvim-ops \
      -p nvim-option \
      -p nvim-os \
      -p nvim-plines \
      -p nvim-profile \
      -p nvim-regexp \
      -p nvim-register \
      -p nvim-rs \
      -p nvim-runtime \
      -p nvim-search \
      -p nvim-session \
      -p nvim-shada \
      -p nvim-statusline \
      -p nvim-strings \
      -p nvim-syntax \
      -p nvim-testing \
      -p nvim-state \
      -p nvim-tui \
      -p nvim-ugrid \
      -p nvim-unpacker \
      -p nvim-version \
      -p nvim-viewport \
      -p nvim-usercmd \
      -p nvim-viml-parser

# Run Rust linter
rust-clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format Rust code
rust-fmt:
    cargo fmt

# Check Rust formatting
rust-fmt-check:
    cargo fmt -- --check

# Run all Rust checks (clippy + fmt + test)
rust-check: rust-fmt-check rust-clippy rust-test

# Test Rust FFI from C (compares Rust vs C implementations)
# Note: Only compare_math and compare_regexp work standalone. Other tests need
# Neovim's C code (utf8proc, global variables, accessor functions) - run them via `just test`
rust-ffi-test: rust-build
    cc -o /tmp/compare_math src/nvim-rs/test/compare_math.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_math
    cc -o /tmp/compare_regexp src/nvim-rs/test/compare_regexp.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_regexp

# Generate regexp baseline corpus from the C engine
regexp-baseline: build
    VIMRUNTIME=runtime ./build/bin/nvim --headless --clean -S test/regexp_baseline.vim

# Validate regexp corpus matches committed baseline (catches regressions)
regexp-validate: build
    VIMRUNTIME=runtime ./build/bin/nvim --headless --clean -S test/regexp_baseline.vim
    @git diff --exit-code src/nvim-rs/test/regexp_corpus.json || (echo 'ERROR: regexp corpus diverged from committed baseline' && exit 1)

# Fuzz test regexp engine with random patterns (catches crashes/hangs)
regexp-fuzz duration='30': build
    timeout {{duration}} bash -c 'VIMRUNTIME=runtime ./build/bin/nvim --headless --clean -S test/regexp_fuzz.vim 2>&1' || { echo "FAIL: regexp fuzz test crashed or timed out (exit $?)"; exit 1; }

# Full build: Rust + C
build-all: rust-build build

# Full pre-commit check: build, smoke-test, and all Rust checks
check: build smoke-test rust-check

# Full check: all Rust checks + C tests
check-all: rust-check test
