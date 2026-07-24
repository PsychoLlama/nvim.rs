# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/).

## [Unreleased]

### Changed

- Rust toolchain bumped forward by 3 years.

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

[Unreleased]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.23-95cd63254c...HEAD
[2026.07.23-95cd63254c]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.21-aa530a7...2026.07.23-95cd63254c
[2026.07.21-aa530a7]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.19-2a6342c...2026.07.21-aa530a7
[2026.07.19-2a6342c]: https://github.com/PsychoLlama/nvim.rs/commits/2026.07.19-2a6342c
