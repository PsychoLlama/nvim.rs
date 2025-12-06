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

# Run all tests
test:
    cmake --build build --target test

# Run functional tests
functionaltest:
    cmake --build build --target functionaltest

# Run unit tests
unittest:
    cmake --build build --target unittest

# Run the built nvim
run *ARGS:
    ./build/bin/nvim {{ARGS}}

# Show nvim version
version:
    ./build/bin/nvim --version

# === Rust Commands ===

# Build Rust components
rust-build:
    cargo build --release

# Build Rust components (debug)
rust-build-debug:
    cargo build

# Run Rust tests
rust-test:
    cargo test

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
rust-ffi-test: rust-build
    cc -o /tmp/compare_math src/nvim-rs/test/compare_math.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_math
    cc -o /tmp/compare_path src/nvim-rs/test/compare_path.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_path
    cc -o /tmp/compare_strings src/nvim-rs/test/compare_strings.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_strings
    cc -o /tmp/compare_mbyte src/nvim-rs/test/compare_mbyte.c -L target/release -lnvim_rs -lpthread -ldl -lm
    /tmp/compare_mbyte

# Full build: Rust + C
build-all: rust-build build

# Full check: all Rust checks + C tests
check-all: rust-check test
