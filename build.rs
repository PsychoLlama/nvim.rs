//! Link the transpiled objects against neovim's bundled C dependencies.
//!
//! The transpiled Rust supplies every symbol that used to come from neovim's
//! `.c` sources; everything else (LuaJIT, libuv, tree-sitter, unibilium,
//! utf8proc, lpeg, luv) lives in the static archives neovim's `cmake.deps`
//! build produces. Those are built out-of-band by Nix (`nix/deps.nix`), which
//! hands us their install prefix via `$NVIM_DEPS_PREFIX`. We reproduce
//! neovim's link line against it: the same libraries, in the same order, with
//! the same `--export-dynamic` so dlopened Lua C modules can resolve back into
//! the binary.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // The prebuilt bundled C deps. Nix owns them (`nix/deps.nix`); there is
    // deliberately no in-tree fallback build. Both the package and the dev
    // shell export this, so a bare `cargo build` outside them fails here — on
    // purpose, rather than silently downloading and compiling C sources.
    let prefix = PathBuf::from(std::env::var("NVIM_DEPS_PREFIX").expect(
        "NVIM_DEPS_PREFIX must point at the built C-deps prefix; \
         build with `nix build` or from inside the flake dev shell",
    ));

    println!("cargo:rerun-if-env-changed=NVIM_DEPS_PREFIX");

    for libdir in ["lib", "lib64"] {
        println!(
            "cargo:rustc-link-search=native={}",
            prefix.join(libdir).display()
        );
    }

    // Static archives from the deps prefix, in neovim's link order. `luv`
    // depends on `uv`, `luajit` on `m`, so ordering matters for the static
    // linker.
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

    // Bake the compiled-in default paths (neovim's generated `pathdef.c`) so
    // the dev binary finds its runtime and bundled tree-sitter parsers with no
    // env vars. Both are `os_isdir`-guarded at resolution time, so an installed
    // binary falls through to the exe-relative layout unless the baked dir
    // exists. Override each var for a prod build to point at the install prefix.
    for (var, default) in [
        ("NVIM_DEFAULT_VIMRUNTIME_DIR", manifest.join("runtime")),
        ("NVIM_DEFAULT_LIB_DIR", prefix.join("lib/nvim")),
    ] {
        let val = std::env::var(var).unwrap_or_else(|_| default.display().to_string());
        println!("cargo:rustc-env={var}={val}");
        println!("cargo:rerun-if-env-changed={var}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
