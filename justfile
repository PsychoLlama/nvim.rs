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
