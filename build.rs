//! Link the transpiled objects against neovim's bundled C dependencies, and
//! compile the builtin `vim.*` Lua modules to embeddable LuaJIT bytecode.
//!
//! The transpiled Rust supplies every symbol that used to come from neovim's
//! `.c` sources; everything else (LuaJIT, libuv, tree-sitter, unibilium,
//! utf8proc, lpeg, luv) lives in the static archives neovim's `cmake.deps`
//! build produces. Those are built out-of-band by Nix (`nix/deps.nix`), which
//! hands us their install prefix via `$NVIM_DEPS_PREFIX`. We reproduce
//! neovim's link line against it: the same libraries, in the same order, with
//! the same `--export-dynamic` so dlopened Lua C modules can resolve back into
//! the binary.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The `vim.*` modules embedded in the binary as LuaJIT bytecode, in the
/// order of executor.rs's `builtin_modules` table. Upstream CMake globbed
/// `_core/*.lua` for the tail of this list; we pin it and verify the glob
/// below so a new `_core` module can't silently ship un-embedded.
const EMBEDDED_LUA_MODULES: &[&str] = &[
    "vim._init_packages",
    "vim.inspect",
    "vim.filetype",
    "vim.fs",
    "vim.F",
    "vim.keymap",
    "vim.loader",
    "vim.text",
    "vim._core.defaults",
    "vim._core.editor",
    "vim._core.ex_cmd",
    "vim._core.exrc",
    "vim._core.help",
    "vim._core.log",
    "vim._core.options",
    "vim._core.server",
    "vim._core.shared",
    "vim._core.stringbuffer",
    "vim._core.system",
    "vim._core.ui2",
    "vim._core.util",
];

/// Compile `runtime/lua/vim/*` to bytecode in `$OUT_DIR/lua_modules/`, where
/// executor.rs `include_bytes!`s it. This replaces the upstream CMake +
/// `gen_char_blob.lua` step whose output c2rust transpiled as array
/// literals: `runtime/lua` is the single source of truth again.
fn compile_lua_modules(manifest: &Path, deps_prefix: &Path) {
    let script = manifest.join("src/gen/compile_lua_modules.lua");
    let outdir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("lua_modules");
    std::fs::create_dir_all(&outdir).unwrap();

    // A `_core` module upstream would have globbed but our pinned list (and
    // executor.rs's builtin_modules table) doesn't know about is a build
    // error, not a silent omission.
    let core_dir = manifest.join("runtime/lua/vim/_core");
    for entry in std::fs::read_dir(&core_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(true, |ext| ext != "lua") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let modname = format!("vim._core.{stem}");
        assert!(
            EMBEDDED_LUA_MODULES.contains(&modname.as_str()),
            "{} is not in build.rs's EMBEDDED_LUA_MODULES; add it there and \
             to builtin_modules in src/nvim/lua/executor.rs",
            path.display(),
        );
    }

    let mut cmd = Command::new(deps_prefix.join("bin/luajit"));
    cmd.arg(&script).arg(&outdir);
    for modname in EMBEDDED_LUA_MODULES {
        let source = manifest
            .join("runtime/lua")
            .join(modname.replace('.', "/"))
            .with_extension("lua");
        println!("cargo:rerun-if-changed={}", source.display());
        cmd.arg(source).arg(modname);
    }
    println!("cargo:rerun-if-changed={}", script.display());

    let status = cmd.status().expect("failed to run the deps-prefix luajit");
    assert!(status.success(), "compile_lua_modules.lua failed: {status}");
}

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

    compile_lua_modules(&manifest, &prefix);

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
