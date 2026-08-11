# nvim.rs

A rewrite of Neovim [v0.12.4](https://github.com/neovim/neovim/releases/tag/v0.12.4) to Rust.

## Purpose

I've been a vim/neovim zealot for the past decade. It's my home. But the nvim tradition of modding and some emacs envy leaves a giant wishlist unlikely to hit mainline any time soon.

I'd yeet some AI at it, but a codebase dating back to the 80s isn't exactly the safest playground. Segfaults don't spark joy. It needs guardrails. It needs Rust.

This repo is an attempt to port neovim's `v0.12.4` codebase to safe, idiomatic Rust while keeping user-facing behavior identical. Once it's my preferred brand of "maintainable" I'll fork it to pursue new absurdities.

## Roadmap

- [x] Mechanical Rust port using `c2rust` (unsafe, non-idiomatic, C-style).
- [x] Build refactoring safety nets (test ports, ASAN, Miri, ABI snapshots, static analysis, ratchets, ...)
- [x] Port simple CMake dependencies to Rust.
- [x] Quarantine shared mutable data behind runtime checks.
- [x] Replace mechanical ports of generated data with `build.rs` equivalents.
- [ ] Incrementally refactor feature-level scopes to idiomatic Rust.
- [ ] Migrate cross-cutting concerns to idiomatic Rust.
- [ ] Replace runtime-checked cells with static `&mut` where possible.
- [ ] Break codebase into smaller crates.

## Caveats

- Rust was bootstrapped by a custom fork of [c2rust](https://github.com/PsychoLlama/c2rust). Unidiomatic and `unsafe`.
- **Linux only.** Transpilation hard-coded many assumptions about the platform.
- **Refactored by AI.** Don't expect human quality code.

## Usage

**Download**

See [the GitHub latest release](https://github.com/PsychoLlama/nvim.rs/releases/latest) for a prebuilt binary (again, only Linux).

**Nix**

```bash
nix run 'github:PsychoLlama/nvim.rs#'
```

## License

[Inherited from Neovim](https://github.com/neovim/neovim/blob/v0.12.4/README.md?plain=1#L94-L99):

> Neovim contributions since [b17d96](https://github.com/neovim/neovim/commit/b17d9691a24099c9210289f16afb1a498a89d803) are licensed under the Apache 2.0 license, except for contributions copied from Vim (identified by the `vim-patch` token). See [LICENSE.txt](./LICENSE.txt) for details.
