# Build neovim using system dependencies from nix (with Rust implementations)
build:
    cargo build --release
    cmake -B build -G Ninja -DUSE_BUNDLED=OFF -DUSE_RUST=ON
    cmake --build build

# Build with bundled dependencies (downloads deps)
build-bundled:
    cmake -B build -G Ninja
    cmake --build build

# Configure only (no build, with Rust implementations)
configure:
    cmake -B build -G Ninja -DUSE_BUNDLED=OFF -DUSE_RUST=ON

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
    cargo test \
      -p nvim-api \
      -p nvim-ascii \
      -p nvim-charset \
      -p nvim-cmdhist \
      -p nvim-collections \
      -p nvim-encoding \
      -p nvim-eval \
      -p nvim-fileio \
      -p nvim-fuzzy \
      -p nvim-help \
      -p nvim-indent \
      -p nvim-keycodes \
      -p nvim-linematch \
      -p nvim-mark \
      -p nvim-math \
      -p nvim-memutil \
      -p nvim-menu \
      -p nvim-ops \
      -p nvim-os \
      -p nvim-profile \
      -p nvim-register \
      -p nvim-spell \
      -p nvim-strings

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
# Note: Only compare_math works standalone. Other tests need Neovim's C code
# (utf8proc, global variables, accessor functions) - run them via `just test`
rust-ffi-test: rust-build
    cc -o /tmp/compare_math src/nvim-rs/test/compare_math.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_math

# Full build: Rust + C
build-all: rust-build build

# Full check: all Rust checks + C tests
check-all: rust-check test
