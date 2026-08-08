# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/).

## [Unreleased]

### Changed

- Rewrote the OS layer, covering environment variables and `$VAR`/`~`
  expansion, running the shell for `:!`, `system()` and wildcard expansion,
  the locale and `:language`, and the input buffer every keystroke arrives
  in. File-system access was reorganised but not rewritten.
- Rewrote the string and multibyte layer, covering `printf()`'s whole format
  language, UTF-8 decoding, character width and character classes, case
  folding, and the `iconv`-backed encoding conversion behind `++enc` and
  `'fileencoding'`.
- Rewrote the msgpack codec and the `vim.mpack` module.
- Rewrote the JSON codec behind `vim.json`, including its float formatting,
  which is what decides the exact text `vim.json.encode()` produces for a
  number.
- Rewrote diffing, covering the vendored Myers, minimal, patience and
  histogram algorithms, `vim.diff()`, diff mode's internal and external
  paths, the highlighting down to the inline character diff, diff folds,
  `]c`/`[c` and `:diffget`/`:diffput`.
- Rewrote C indenting, covering `'cindent'`, `cindent()` and the whole
  `'cinoptions'`/`'cinkeys'`/`'cinwords'`/`'cinscopedecls'` vocabulary.
- Rewrote putting and the registers, covering `p`/`P` and their `g`, `]`,
  `[` and `z` variants, yanking into a register and appending to one,
  recording with `q` and replaying with `@`, CTRL-R in Insert mode and on
  the command line, `:registers`, and the `getreg()`/`setreg()` surface the
  clipboard provider and shada share.
- Rewrote the buffer-change primitives, covering inserting and deleting
  text, opening a new line with its indent and comment leader, and the
  notifications an edit sends to the display, the extmark tree and the
  buffer-update callbacks.
- Rewrote the operators, covering `d`, `c`, `y`, `<`/`>`, `J`/`gJ`, `r`,
  `g~`/`gu`/`gU`/`g?`, blockwise `I`/`A`, CTRL-A/CTRL-X, `!`, `=`, `g@` and
  `g CTRL-G`, the blockwise geometry they share, and the dispatcher that
  turns an operator plus a motion or a Visual selection into the region
  each one is given.
- Rewrote Insert mode, covering the key loop and everything a key can mean
  in it: backspace and delete under every `'backspace'` spelling, `<Tab>`
  under `'expandtab'`/`'softtabstop'`/`'vartabstop'`, the arrow keys and
  their shifted and CTRL- forms, CTRL-V and CTRL-K, CTRL-R, CTRL-O, CTRL-G,
  Replace and Virtual Replace mode's undo of what they overwrote, `.`
  repeating an insert, and the prompt buffer.
- Rewrote text formatting and the text objects, covering auto-wrap at
  `'textwidth'`, `gq`/`gw` and `'formatexpr'`, the `'formatoptions'` and
  `'comments'` rules that decide where a paragraph ends, every `i`/`a`
  object and the word, sentence and paragraph motions they share their
  rules with, and the indent-width arithmetic behind `'shiftwidth'`,
  `'softtabstop'` and `'vartabstop'`.

### Fixed

- `printf()` no longer exits the editor when `%S`'s field width is wider than
  the string it formats (`printf('%3S', 'éèü')`).
- `vim.mpack.encode()` no longer aborts the editor on a number over 2^53, on
  `math.huge` or on a NaN, and no longer drops the high half of a negative
  integer larger than 2^32 (which decoded back as a different number).
- `vim.json.encode()` called from inside a metamethod of the value it is
  encoding no longer corrupts the document its caller was building.

## [2026.08.06-eb75350b02]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.

### Changed

- Rewrote the input layer, covering key codes, the typeahead and its
  stuff/redo/record buffers, `:map` and friends, `maparg()`/`mapset()`,
  `nvim_set_keymap`, abbreviations, and `'langmap'`.
- Rewrote command-line completion, covering `<Tab>` and the wildmenu, every
  `EXPAND_*` source, and `getcompletion()`.
- Rewrote the command-line editor, covering the `:`, `/` and `?` key loop and
  its history, the command-line window, `'incsearch'` and `'inccommand'`,
  command-line highlighting, `input()`, the `getcmd*()` and `setcmdline()`
  functions, and the `ext_cmdline` UI events.
- Rewrote insert-mode completion, covering every `CTRL-X` source, `'complete'`
  and `'completeopt'`, `'completefunc'`/`'omnifunc'`/`'thesaurusfunc'`, the
  completion popup menu, and `complete()`, `complete_info()` and
  `CompleteDone`.
- Rewrote the Vimscript value core, covering lists, dictionaries and blobs,
  `sort()` and `uniq()`, dictionary watchers, and the type checks every
  builtin argument and return value goes through.
- Rewrote value encoding and decoding, covering msgpack, JSON, `string()`,
  `:echo`, `json_decode()` and `msgpackparse()`, and the conversions to Lua
  values and API objects.
- Rewrote the variable layer, covering `:let`, `:unlet`, `:const`, `:lockvar`,
  the `g:`/`b:`/`w:`/`t:`/`v:`/`l:`/`s:` scopes, the
  `getbufvar()`/`setwinvar()` family, here-documents, and `:redir =>`.
- Rewrote user functions, covering `:function` and `:delfunction`, argument
  defaults and `...`, funcrefs, partials, closures and lambdas, `:call`,
  `:return` and `:defer`, and autoloading.
- Rewrote autocommands, covering `:autocmd` and `:augroup`, `:doautocmd` and
  `:doautoall`, the `<buffer=N>` patterns, `++once` and `++nested`,
  `'eventignore'` and `'eventignorewin'`, `exists('#…')`, and the
  `nvim_create_autocmd` family.
- Rewrote the Lua runtime, covering `:lua`, `:luado` and `:luafile`,
  `luaeval()` and `v:lua`, `vim.schedule()` and `vim.wait()`, the luv event
  bridge, Lua-implemented user commands, `vim.regex()` and `vim.iconv()`, the
  `vim.*` table, `vim.ui_attach()`, and the tree-sitter bindings.
- Rewrote the API layer, covering every `nvim_*` function from buffers,
  windows and tabpages to extmarks and decorations, options, user commands
  and `nvim_cmd`, expression parsing, the UI attach protocol, and the
  deprecated shims.

### Fixed

- Internal consistency checks no longer abort a release build, so cases the
  original carries on past (`nvim_set_hl()` with a negative namespace id, for
  one) no longer kill the editor.
- `nvim_parse_expression()` no longer kills the editor on an unfinished
  curly-braces name such as `'a{b'`.
- A dictionary key decoded from msgpack is no longer randomly read-only or
  randomly leaked.
- Assigning to a `v:` variable through the `v:` dictionary rather than by name
  no longer replaces the variable's type.
- `nvim_win_text_height()` no longer writes past the end of the reply it
  allocates.

## [2026.08.02-af6bcec290]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.

### Changed

- Rewrote the screen pipeline, covering the cell grid, screen-line
  measurement, highlight groups, decorations and virtual text, signs,
  matches, the popup menu, syntax highlighting, and redrawing.
- Rewrote searching and navigation, covering `/` and `?`, `n`, `*` and `gd`,
  the bracket motions, tags, path expansion and `:find`, fuzzy matching, and
  the quickfix and location lists.
- Rewrote the persistence layer, covering swap files and crash recovery,
  reading and writing files with their encodings and backups, and shada.
- Rewrote spell checking, covering `:mkspell`, the `.spl` and `.sug` formats,
  `'spelllang'` and `'spellfile'`, `]s`, `z=`, and `:spelldump`.
- Rewrote the user interface, covering the built-in terminal UI and its input
  parser, `:terminal` buffers, and the remote-UI protocol on both ends.

### Fixed

- A sign text wider than two cells no longer overruns its buffer, whether it
  comes from `:sign define` or an extmark's `sign_text`.

## [2026.07.30-4b9dee25d3]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.

### Changed

- Rewrote the regular-expression engines, covering search, `:substitute`,
  syntax highlighting, and both 'regexpengine' settings.
- Rewrote the option logic, covering `:set` and its relatives, option
  validation, and the per-window and per-buffer values.
- Rewrote the Vimscript builtin functions, covering every builtin from
  `abs()` to `wordcount()`.
- Rewrote the Vimscript expression evaluator, covering expression syntax,
  `:let`, `:for`, `:echo`, `:execute`, timers, and the garbage collector.
- Rewrote normal mode, covering motions, operators, counts and registers,
  Visual and Select mode, scrolling and folding, marks and jumps, and the tag
  and identifier lookups.
- Rewrote startup, covering command-line arguments, config sourcing,
  `--embed`, `--headless`, `-l`, `--remote`, and every path out of the
  process.
- Rewrote the Ex command dispatcher, covering every `:` command, ranges,
  counts and registers, command modifiers, `nvim_parse_cmd` and `nvim_cmd`,
  and the sourcing loop behind `:source`, `:execute`, `:global`,
  autocommands and mappings.
- Generated the API dispatch layer, the option table and the Vimscript
  builtin-function table from their specs again (`just apigen`), covering
  every msgpack-RPC method, the `vim.api` Lua binding, `nvim --api-info`,
  every option's default and valid values, and every builtin function call.

### Fixed

- `:later`, `:resize`, `:tabmove`, `:tabnext`, `zH`, `zL`, `z<N>l` and a
  Visual reselection count at the number extremes wrap as documented instead
  of aborting a debug build.
- A write to freed memory in the backtracking regexp engine, on a pattern
  that visits enough distinct back edges.

## [2026.07.27-dd16441f3f]

Ongoing migration of the transpiled code toward safe, idiomatic Rust.

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

[Unreleased]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.06-eb75350b02...HEAD
[2026.08.06-eb75350b02]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.02-af6bcec290...2026.08.06-eb75350b02
[2026.08.02-af6bcec290]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.30-4b9dee25d3...2026.08.02-af6bcec290
[2026.07.30-4b9dee25d3]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.27-dd16441f3f...2026.07.30-4b9dee25d3
[2026.07.27-dd16441f3f]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.26-d0c5cf2147...2026.07.27-dd16441f3f
[2026.07.26-d0c5cf2147]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.23-95cd63254c...2026.07.26-d0c5cf2147
[2026.07.23-95cd63254c]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.21-aa530a7...2026.07.23-95cd63254c
[2026.07.21-aa530a7]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.19-2a6342c...2026.07.21-aa530a7
[2026.07.19-2a6342c]: https://github.com/PsychoLlama/nvim.rs/commits/2026.07.19-2a6342c
