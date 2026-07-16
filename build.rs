//! Link the transpiled neovim objects against the same native libraries
//! neovim's own `ninja` build links `bin/nvim` against.
//!
//! The transpiled Rust supplies every symbol that used to come from
//! neovim's `.c` sources; everything else (LuaJIT, libuv, tree-sitter,
//! unibilium, utf8proc, lpeg, luv) still lives in the prebuilt static
//! archives under `<neovim>/.deps/usr/lib{,64}`. We reproduce neovim's
//! link line here: the same libraries, in the same order, with the same
//! `--export-dynamic` so dlopened Lua C modules can resolve back into the
//! binary.

use std::path::PathBuf;

fn main() {
    // Locate neovim's dependency prefix (`.deps/usr`). The nix `port`
    // devShell exports NVIM_DEPS_PREFIX; otherwise fall back to the
    // `.deps` at the crate root (the neovim source tree root).
    let prefix = std::env::var("NVIM_DEPS_PREFIX")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            manifest.join(".deps/usr")
        });

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

    println!("cargo:rerun-if-env-changed=NVIM_DEPS_PREFIX");
    println!("cargo:rerun-if-changed=build.rs");
}
