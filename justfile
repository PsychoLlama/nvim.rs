# Build neovim using system dependencies from nix
build:
    cmake -B build -G Ninja -DUSE_BUNDLED=OFF
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

# Full build: Rust + C
build-all: rust-build build

# Full check: all Rust checks + C tests
check-all: rust-check test
