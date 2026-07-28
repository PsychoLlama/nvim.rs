# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/).

## [Unreleased]

Ongoing migration of the transpiled code toward safe, idiomatic Rust, and of
the transpiled tables back onto generated code.
Behavior-preserving: same features, formats, and RPC surface.

### Changed

- Rewrote the regular-expression engines, covering every pattern the editor
  matches: search, `:substitute`, syntax highlighting, and both
  'regexpengine' settings.
- Generated the API dispatch layer from the API signatures again
  (`just apigen`), covering every msgpack-RPC method, the `vim.api` Lua
  binding, and `nvim --api-info`.
- Generated the option table from `options.lua` again (`just apigen`),
  covering every option's default, valid values, and `:set`.
- Generated the Vimscript builtin-function table from `eval.lua` again
  (`just apigen`), covering every builtin function call.

## [2026.07.27-dd16441f3f]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.
Behavior-preserving except where noted under Fixed.

### Changed

- Rewrote the terminal stack (`vterm`, `termkey`, `terminfo`), covering
  `:terminal`, terminal key input, and TUI rendering.
- Rewrote the event loop, job control, msgpack-rpc, and channels, covering
  jobs, pty processes, RPC clients, and UI attachments.
- Rewrote the Vimscript expression parser, covering expression evaluation,
  `nvim_parse_expression`, and cmdline highlighting.
- Rewrote the extmark store and the core hash containers, covering extmarks,
  signs, decorations, and virtual text.
- Rewrote character classification, display translation, cursor motion, and
  indenting, covering unprintable rendering, `'vartabstop'`, and cursor
  movement.
- Rewrote undo, the argument list, folds, and marks, covering undo files
  (format unchanged), `:args`, folding, marks, the jumplist, and shada.
- Rewrote the buffer, window, and compound-assignment builtins, covering the
  `*bufline*()` and `win_*()` function families and `:let x += y`.

### Fixed

- Several crashes and hangs reachable from any program running under
  `:terminal`.
- Four out-of-bounds reads and writes reachable from terminal key input.
- A leak and an aliasing fault in cmdline highlighting of figure braces.
- `:set vartabstop` no longer clears a buffer's tabstops when it rejects the
  new value.
- `:let n += 1` at the largest representable number wraps as documented
  instead of aborting a debug build.

## [2026.07.26-d0c5cf2147]

### Changed

- Rust toolchain bumped forward by 3 years.
- Migrate fully off unstable language features.

### Fixed

- Patched several UB gaps identified by new aggressive analysis tools.

## [2026.07.23-95cd63254c]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.
Behavior-preserving: same features, formats, and RPC surface.

### Changed

- Cleared the build-warning noise floor and made the warning count a ratchet,
  then enforced `-D warnings` in CI.
- Extracted the ~215k lines of embedded LuaJIT bytecode out of `executor.rs`;
  the `vim.*` core modules are now compiled from `runtime/lua` at build time.
- Unified the duplicated c2rust type graph into a single canonical
  `src/nvim/types/` tree, so every logical type has exactly one definition.
- Replaced the per-module `extern "C"` re-declarations with real imports, so
  cross-module calls are checked by the compiler instead of the linker.
- Cut the unit suite loose from the frozen `v0.12.4` C tree: FFI definitions
  are generated from the crate, pure-logic specs moved to Rust tests, and the
  upstream header scaffolding is gone.
- Rewrote `sha256`, `cmdhist`, `digraph`, and `profile` as safe idiomatic Rust.

### Removed

- De-exported the internal-only symbol surface and deleted the dead transpiled
  code that pruning exposed.
- Dropped `unibilium` and `utf8proc` from the dependency build, porting the
  used subset of each into the tree.

## [2026.07.21-aa530a7]

Safety foundation: make undefined behavior observable, then structurally shrink
its two worst sources (manual heap ownership, aliased mutable globals).

### Added

- AddressSanitizer test coverage, an ABI ledger of the exported-symbol surface,
  and ratchet safety metrics (`unsafe`/`static mut`/`#[no_mangle]`/file size).

### Changed

- Unified Rust and the legacy `xmalloc` family onto a single global allocator,
  so ownership can cross the FFI boundary without copying.
- Gave the shared memory primitives (`garray`, `hashtab`, the `memory.rs` and
  `strings.rs` helpers) safe cores behind C-ABI shims.
- Replaced editor-state `static mut` with `GlobalCell`, a checked cell that
  detects reentrancy aliasing in debug builds.
- Rewrote `math`, `base64`, `arabic`, and `clipboard` as fully safe modules.

## [2026.07.19-2a6342c]

First tagged release (unstable). The c2rust transpile of Neovim from C to Rust,
made to build and pass the functional, old, and unit suites, with the CalVer
release pipeline in place. The starting point: ~1.21M lines of mostly `unsafe`
Rust with no user-visible change from upstream.

[Unreleased]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.27-dd16441f3f...HEAD
[2026.07.27-dd16441f3f]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.26-d0c5cf2147...2026.07.27-dd16441f3f
[2026.07.26-d0c5cf2147]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.23-95cd63254c...2026.07.26-d0c5cf2147
[2026.07.23-95cd63254c]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.21-aa530a7...2026.07.23-95cd63254c
[2026.07.21-aa530a7]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.19-2a6342c...2026.07.21-aa530a7
[2026.07.19-2a6342c]: https://github.com/PsychoLlama/nvim.rs/commits/2026.07.19-2a6342c
