use std::path::{Path, PathBuf};

fn main() {
    // Locate CMakeLists.txt relative to this crate's manifest directory.
    // Path: src/nvim-rs/version/ -> ../../.. -> repo root
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    let cmake_path = PathBuf::from(&manifest_dir)
        .join("../../../CMakeLists.txt")
        .canonicalize()
        .unwrap_or_else(|_| {
            panic!(
                "Cannot find CMakeLists.txt relative to crate at {manifest_dir}. \
                 Expected path: ../../../CMakeLists.txt"
            )
        });

    println!("cargo:rerun-if-changed={}", cmake_path.display());

    let (major, minor, patch) = parse_cmake_versions(&cmake_path);

    // Write a generated file with literal const declarations.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set by cargo");
    let out_path = Path::new(&out_dir).join("version_constants.rs");
    let content = format!(
        "pub const NVIM_VERSION_MAJOR: i32 = {major};\n\
         pub const NVIM_VERSION_MINOR: i32 = {minor};\n\
         pub const NVIM_VERSION_PATCH: i32 = {patch};\n"
    );
    std::fs::write(&out_path, content)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", out_path.display()));
}

fn parse_cmake_versions(cmake_path: &Path) -> (u32, u32, u32) {
    let content = std::fs::read_to_string(cmake_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", cmake_path.display()));

    let major = parse_cmake_var(&content, "NVIM_VERSION_MAJOR", cmake_path);
    let minor = parse_cmake_var(&content, "NVIM_VERSION_MINOR", cmake_path);
    let patch = parse_cmake_var(&content, "NVIM_VERSION_PATCH", cmake_path);

    (major, minor, patch)
}

fn parse_cmake_var(content: &str, var: &str, cmake_path: &Path) -> u32 {
    // Match: set(NVIM_VERSION_MAJOR <number>)
    let prefix = format!("set({var} ");
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            if let Some(value) = rest.strip_suffix(')') {
                return value.trim().parse::<u32>().unwrap_or_else(|_| {
                    panic!(
                        "Failed to parse {var} value '{value}' in {}",
                        cmake_path.display()
                    )
                });
            }
        }
    }
    panic!(
        "Could not find '{prefix}...) in {}. \
         Expected format: set({var} <number>)",
        cmake_path.display()
    );
}
