# FindRustNvim.cmake
# Find the nvim-rs Rust static library
#
# This module sets:
#   RUST_NVIM_FOUND - True if the library was found
#   RUST_NVIM_LIBRARY - Path to the static library
#   RUST_NVIM_INCLUDE_DIR - Path to the generated headers

# Build the Rust library first
execute_process(
  COMMAND cargo build --release -p nvim-rs
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  RESULT_VARIABLE CARGO_RESULT
)

if(NOT CARGO_RESULT EQUAL 0)
  message(FATAL_ERROR "Failed to build Rust library: cargo build failed")
endif()

# Find the static library
find_library(RUST_NVIM_LIBRARY
  NAMES nvim_rs libnvim_rs
  PATHS ${CMAKE_SOURCE_DIR}/target/release
  NO_DEFAULT_PATH
)

# Generate headers using cbindgen
execute_process(
  COMMAND cbindgen --config src/nvim-rs/cbindgen.toml --crate nvim-rs --output ${CMAKE_BINARY_DIR}/nvim_rs.h
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  RESULT_VARIABLE CBINDGEN_RESULT
)

# Also generate per-crate headers
execute_process(
  COMMAND cbindgen --crate nvim-math --output ${CMAKE_BINARY_DIR}/nvim_rs_math.h
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/src/nvim-rs/math
)

set(RUST_NVIM_INCLUDE_DIR ${CMAKE_BINARY_DIR})

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(RustNvim
  REQUIRED_VARS RUST_NVIM_LIBRARY RUST_NVIM_INCLUDE_DIR
)

if(RUST_NVIM_FOUND)
  message(STATUS "Found Rust nvim library: ${RUST_NVIM_LIBRARY}")

  # Create imported target
  add_library(rust_nvim STATIC IMPORTED GLOBAL)
  set_target_properties(rust_nvim PROPERTIES
    IMPORTED_LOCATION ${RUST_NVIM_LIBRARY}
  )

  # Link dependencies required by Rust
  target_link_libraries(rust_nvim INTERFACE pthread dl m)
endif()

mark_as_advanced(RUST_NVIM_LIBRARY RUST_NVIM_INCLUDE_DIR)
