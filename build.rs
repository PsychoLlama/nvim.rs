//! Build neovim's bundled C dependencies and link the transpiled objects
//! against them.
//!
//! The transpiled Rust supplies every symbol that used to come from
//! neovim's `.c` sources; everything else (LuaJIT, libuv, tree-sitter,
//! unibilium, utf8proc, lpeg, luv) lives in the static archives neovim's
//! `cmake.deps` build produces under `.deps/usr/lib{,64}`. We build those
//! archives (if needed) and reproduce neovim's link line: the same
//! libraries, in the same order, with the same `--export-dynamic` so
//! dlopened Lua C modules can resolve back into the binary.
//!
//! cmake and ninja come from the flake devShell; nothing else is assumed.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let prefix = manifest.join(".deps/usr");

    // Build the bundled C deps if they aren't there yet — the same two-step
    // cmake build as BUILD.md's "Third-party dependencies". It runs only when
    // `.deps` is absent, so ordinary rebuilds just relink; `rm -rf .deps` to
    // force a fresh build (e.g. after bumping a version in `cmake.deps/deps.txt`).
    if !prefix.join("lib/libluajit-5.1.a").exists() {
        println!(
            "cargo:warning=building bundled C deps under .deps (downloads sources, takes a few minutes)"
        );
        cmake(
            &manifest,
            &[
                "-S",
                "cmake.deps",
                "-B",
                ".deps",
                "-G",
                "Ninja",
                "-D",
                "CMAKE_BUILD_TYPE=RelWithDebInfo",
            ],
        );
        cmake(&manifest, &["--build", ".deps"]);
    }

    for libdir in ["lib", "lib64"] {
        println!(
            "cargo:rustc-link-search=native={}",
            prefix.join(libdir).display()
        );
    }

    // Static archives from `.deps`, in neovim's link order. `luv` depends
    // on `uv`, `luajit` on `m`, so ordering matters for the static linker.
    for lib in [
        "luv",
        "lpeg",
        "tree-sitter",
        "utf8proc",
        "unibilium",
        "luajit-5.1",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    // System libraries, interleaved with libuv exactly as neovim links them.
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=util");
    println!("cargo:rustc-link-lib=static=uv");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=rt");

    // neovim links with `-Wl,--export-dynamic -rdynamic`: LuaJIT FFI and
    // dlopened C modules (e.g. libnlua0) resolve symbols back into nvim.
    println!("cargo:rustc-link-arg=-Wl,--export-dynamic");

    println!("cargo:rerun-if-changed=build.rs");
}

/// Run `cmake` from the crate root, asserting it succeeds.
fn cmake(dir: &Path, args: &[&str]) {
    let status = Command::new("cmake")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("failed to spawn cmake (provided by the flake devShell)");
    assert!(status.success(), "cmake {args:?} failed: {status}");
}
