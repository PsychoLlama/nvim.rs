//! Link the transpiled objects against neovim's bundled C dependencies, and
//! compile the builtin `vim.*` Lua modules to embeddable LuaJIT bytecode.
//!
//! The transpiled Rust supplies every symbol that used to come from neovim's
//! `.c` sources; everything else (LuaJIT, libuv, tree-sitter,
//! lpeg, luv) lives in the static archives neovim's `cmake.deps`
//! build produces. Those are built out-of-band by Nix (`nix/deps.nix`), which
//! hands us their install prefix via `$NVIM_DEPS_PREFIX`. We reproduce
//! neovim's link line against it: the same libraries, in the same order, with
//! the same `--export-dynamic` so dlopened Lua C modules can resolve back into
//! the binary.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Run `git` inside `repo_root`, returning its trimmed stdout on success.
/// Every failure mode — no git binary, not a repository, a command that
/// exits nonzero — collapses to `None`, because none of them are build
/// errors: a source tarball or the Nix sandbox (which builds from a filtered
/// `src` with no `.git`) is a perfectly legitimate way to build.
fn git(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Does `tag` have the shape `scripts/tag-release.sh` stamps on releases,
/// `<UTC date>-<short sha>`? Any other tag on HEAD (a personal bookmark, an
/// upstream `v0.12.4`) is not a release of *this* tree and must not be
/// reported as one.
fn is_release_tag(tag: &str) -> bool {
    let Some((date, sha)) = tag.split_once('-') else {
        return false;
    };

    let mut fields = date.split('.');
    let widths = [4, 2, 2];
    for width in widths {
        let Some(field) = fields.next() else {
            return false;
        };
        if field.len() != width || !field.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }

    fields.next().is_none() && !sha.is_empty() && sha.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The version `nvim --version` reports, resolved at build time. In order:
///
/// 1. `$NVIM_RS_VERSION`, so the release build (`just package <version>`) and
///    Nix can state the version they are producing rather than guess it.
/// 2. The CalVer release tag on HEAD, so a plain `cargo build` of a tagged
///    checkout is indistinguishable from the released artifact.
/// 3. `dev-<short sha>`, plus `-dirty` when the tree has uncommitted changes.
/// 4. `unknown`, when there is no git to ask.
fn detect_version(repo_root: &Path) -> String {
    println!("cargo:rerun-if-env-changed=NVIM_RS_VERSION");
    if let Ok(version) = std::env::var("NVIM_RS_VERSION")
        && !version.is_empty()
    {
        return version;
    }

    let Some(git_dir) = git(repo_root, &["rev-parse", "--absolute-git-dir"]) else {
        return "unknown".to_owned();
    };

    // Re-run when HEAD moves: committing, checking out, or tagging otherwise
    // leaves a stale version baked into an up-to-date binary. Watch the ref
    // HEAD points at too (a commit on the current branch only rewrites that
    // file), and `packed-refs`, where the ref lives before it is ever loose.
    // Paths that don't exist are skipped: cargo can't stat them, so it would
    // treat the build script as permanently dirty and recompile the crate on
    // every single build.
    let git_dir = PathBuf::from(git_dir);
    let head_ref = git(repo_root, &["symbolic-ref", "--quiet", "HEAD"]);
    let watched = ["HEAD".to_owned(), "packed-refs".to_owned()]
        .into_iter()
        .chain(head_ref);
    for path in watched.map(|name| git_dir.join(name)) {
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    if let Some(tag) = git(repo_root, &["describe", "--tags", "--exact-match", "HEAD"])
        && is_release_tag(&tag)
    {
        return tag;
    }

    let Some(sha) = git(repo_root, &["rev-parse", "--short", "HEAD"]) else {
        return "unknown".to_owned();
    };

    // Untracked files are excluded: they are not compiled in, so they don't
    // make the binary differ from the commit it claims to be. `git` reports
    // empty output as `None` and a clean tree prints nothing, so "some
    // output" is exactly "dirty".
    let dirty = git(
        repo_root,
        &["status", "--porcelain", "--untracked-files=no"],
    )
    .is_some();

    if dirty {
        format!("dev-{sha}-dirty")
    } else {
        format!("dev-{sha}")
    }
}

/// The compiler that built this binary, as `:version` reports it. Cargo hands
/// build scripts the exact `rustc` it is using in `$RUSTC`, so this describes
/// the real toolchain rather than whatever is first on `$PATH`.
fn rustc_version() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let Ok(output) = Command::new(rustc).arg("--version").output() else {
        return "unknown".to_owned();
    };

    let Ok(stdout) = String::from_utf8(output.stdout) else {
        return "unknown".to_owned();
    };

    // `rustc 1.94.0-nightly (0123abcde 2026-07-24)` — the leading word is
    // redundant with the label `:version` prints it under.
    let version = stdout.trim();
    version.strip_prefix("rustc ").unwrap_or(version).to_owned()
}

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
fn compile_lua_modules(manifest: &Path, repo_root: &Path, deps_prefix: &Path) {
    let script = manifest.join("src/gen/compile_lua_modules.lua");
    let outdir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("lua_modules");
    std::fs::create_dir_all(&outdir).unwrap();

    // A `_core` module upstream would have globbed but our pinned list (and
    // executor.rs's builtin_modules table) doesn't know about is a build
    // error, not a silent omission.
    let core_dir = repo_root.join("runtime/lua/vim/_core");
    for entry in std::fs::read_dir(&core_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "lua") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let modname = format!("vim._core.{stem}");
        assert!(
            EMBEDDED_LUA_MODULES.contains(&modname.as_str()),
            "{} is not in build.rs's EMBEDDED_LUA_MODULES; add it there and \
             to builtin_modules in src/lua/executor/",
            path.display(),
        );
    }

    let mut cmd = Command::new(deps_prefix.join("bin/luajit"));
    cmd.arg(&script).arg(&outdir);
    for modname in EMBEDDED_LUA_MODULES {
        let source = repo_root
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
    // runtime/ lives at the repo root, two levels above crates/nvim/.
    let repo_root = manifest
        .ancestors()
        .nth(2)
        .expect("crates/nvim has a repo root two levels up")
        .to_path_buf();

    // The prebuilt bundled C deps. Nix owns them (`nix/deps.nix`); there is
    // deliberately no in-tree fallback build. Both the package and the dev
    // shell export this, so a bare `cargo build` outside them fails here — on
    // purpose, rather than silently downloading and compiling C sources.
    let prefix = PathBuf::from(std::env::var("NVIM_DEPS_PREFIX").expect(
        "NVIM_DEPS_PREFIX must point at the built C-deps prefix; \
         build with `nix build` or from inside the flake dev shell",
    ));

    println!("cargo:rerun-if-env-changed=NVIM_DEPS_PREFIX");

    compile_lua_modules(&manifest, &repo_root, &prefix);

    for libdir in ["lib", "lib64"] {
        println!(
            "cargo:rustc-link-search=native={}",
            prefix.join(libdir).display()
        );
    }

    // Static archives from the deps prefix, in neovim's link order. `luv`
    // depends on `uv`, `luajit` on `m`, so ordering matters for the static
    // linker.
    for lib in ["luv", "lpeg", "tree-sitter", "luajit-5.1"] {
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
        ("NVIM_DEFAULT_VIMRUNTIME_DIR", repo_root.join("runtime")),
        ("NVIM_DEFAULT_LIB_DIR", prefix.join("lib/nvim")),
    ] {
        let val = std::env::var(var).unwrap_or_else(|_| default.display().to_string());
        println!("cargo:rustc-env={var}={val}");
        println!("cargo:rerun-if-env-changed={var}");
    }

    // What `:version`, `nvim -v`, the intro screen and shada's generator
    // field report about this build.
    println!(
        "cargo:rustc-env=NVIM_RS_VERSION={}",
        detect_version(&repo_root)
    );
    println!("cargo:rustc-env=NVIM_RS_RUSTC={}", rustc_version());
    println!(
        "cargo:rustc-env=NVIM_RS_PROFILE={}",
        std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned()),
    );

    println!("cargo:rerun-if-changed=build.rs");
}
