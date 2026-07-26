# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/).

## [Unreleased]

### Changed

- The embedded terminal emulator (`src/nvim/vterm`) was rewritten from its
  transpiled form into safe, idiomatic Rust, and split along the lines of the
  protocol it implements. Several latent crashes and hangs reachable from any
  program running under `:terminal` went with it.
- The terminal key-input library (`src/nvim/tui/termkey`) likewise: its key
  parsing, tables and formatting are now safe modules, and the plugin
  machinery it carried for drivers this editor never had is gone. Four
  out-of-bounds writes and reads reachable from terminal input went with it.
- Terminal descriptions (`src/nvim/tui/terminfo`) were rewritten the same way:
  the capability slots every part of the TUI indexes by are now defined once,
  the built-in terminal descriptions are data modules pinned by a checksum
  against the C original, and the parameterised-string interpreter renders its
  own conversions instead of assembling `printf` formats at runtime.
- The event loop and job control (`src/nvim/event`, `src/nvim/os/pty_proc_unix`)
  were rewritten over their libuv boundary: the queues, streams, watchers,
  sockets and child processes are safe Rust above a single set of foreign
  declarations, and the loop's intrusive lists and hand-rolled vectors are now
  owned Rust collections.

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

[Unreleased]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.26-d0c5cf2147...HEAD
[2026.07.26-d0c5cf2147]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.23-95cd63254c...2026.07.26-d0c5cf2147
[2026.07.23-95cd63254c]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.21-aa530a7...2026.07.23-95cd63254c
[2026.07.21-aa530a7]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.19-2a6342c...2026.07.21-aa530a7
[2026.07.19-2a6342c]: https://github.com/PsychoLlama/nvim.rs/commits/2026.07.19-2a6342c
