# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/).

## [Unreleased]

### Changed

- Took the C memory layout off 239 of the editor's own struct types, leaving
  it only where something outside the editor really reads the bytes; threaded
  the regular-expression engines' match state through both engines instead of
  reaching for it as a global on every character; gave each option row a typed
  handle on the variable it keeps its value in, so a row naming a variable of
  the wrong type no longer compiles; and turned the shell, wildcard-expansion,
  file-expansion and libuv-error constants into real types instead of loose
  integers. Nothing observable changed.

## [2026.08.15-2b1aee84f3]

### Changed

- Rewrote the mark tree, the interval tree behind every extmark, sign,
  inline virtual text, conceal and decoration the editor draws, along
  with the arithmetic that moves them all as text is inserted, deleted
  or moved.
- Rewrote the terminal emulator and the terminal buffer in front of it:
  the escape-sequence parser and its state machine, every CSI and the
  whole SGR alphabet, the mode table, the UTF-8 decoder and the charset
  designators, OSC/DCS and the OSC 52 selection decoder, the mouse and
  key encoders, and the cell grid with its damage merging, scrollback
  and reflow; `:terminal` and `nvim_open_term()` on top of it, covering
  the `g:terminal_color_*` palette, the scrollback ring and
  `'scrollback'`, terminal-mode input and mouse handling, `TermRequest`
  and the reply writer; and the key reader that decodes every escape
  sequence the TUI receives.

### Fixed

- A terminal window one column wide no longer crashes the editor when the
  program inside it prints a double-width character. The terminal
  emulator wrote the second half of the glyph past the end of the row;
  it now truncates the glyph to the columns that exist.
- Setting `g:terminal_color_0` .. `_15` (or the buffer-local equivalent)
  no longer corrupts the heap. Opening a terminal freed the variable's
  own string, so reading, reassigning or unsetting it afterwards was a
  double free; the palette is now read without taking ownership.

## [2026.08.14-954ee76f86]

### Changed

- Rewrote what was left of the expression layer: the Vimscript expression
  lexer, string decoder and parser behind `nvim_parse_expression`, the
  filesystem builtins from `fnamemodify()` and `glob()` through
  `readfile()`/`writefile()` and the directory family, the
  list/dict/blob builtins (`filter()`, `map()`, `count()`, `extend()`,
  `insert()`, `remove()`, `reverse()` and friends), JSON and msgpack
  encoding, and the character-class tables and cursor-position
  arithmetic the editor asks about every character it draws or moves
  over.
- Rewrote movement and the screen shell: the viewport arithmetic behind
  every cursor move and scroll, from `'scrolloff'`/`'scrolljump'` and
  horizontal scrolling to `CTRL-D`/`CTRL-U`/`CTRL-F`/`CTRL-B` and
  `'smoothscroll'`; mouse input, covering the click counter,
  `'mousemodel'`, drag-resizing a window, the wheel and `getmousepos()`;
  the menu tree behind `:menu`/`:emenu`/`:popup`, `menu_get()` and
  `:menutranslate`; the format expander shared by `'statusline'`,
  `'tabline'`, `'winbar'`, `'statuscolumn'` and `'rulerformat'`, and the
  `%@Func@` click definitions it records; `'guicursor'`; and the
  compositor that blends floating windows and the message grid onto the
  screen.
- Rewrote windows and buffers over a shared safe layer: window splitting
  and the frame tree behind every `CTRL-W` command, `:resize` and the
  `'equalalways'`/`'winfix*'`/`'winmin*'` geometry rules, entering and
  closing windows and tab pages along with the `Win*`/`Tab*`
  autocommands they fire, and floating-window configuration:
  `relative`, `anchor`, `zindex`, borders, titles and `bufpos`; the Ex
  commands in front of all of it, from `:split`/`:vsplit`/`:new` and
  `:tabnew`/`:tabnext`/`:tabmove`/`:tabs` to `:wincmd` and the preview
  window's `:pedit`/`:psearch`/`:pbuffer`; the buffer list behind `:ls`,
  `:buffer`, `:bdelete`/`:bwipeout` and buffer-name completion, buffer
  creation, unloading and wiping with the `Buf*` autocommands around
  them, modelines, `CTRL-G` and `getbufinfo()`; the change notifications
  `nvim_buf_attach()` delivers; extmarks; the quickfix and location list
  windows behind `:copen`/`:lopen`; and the command line's own buffer,
  the text every `:` line is edited in. Window, tab page, frame and
  buffer identity now travel as typed handles instead of raw pointers.

### Fixed

- `nvim_parse_expression()` no longer rescans the whole parse stack for
  every token it reads. The check is one the original keeps to debug
  builds; 8,000 nested parentheses parse in 28 ms instead of 1,658 ms,
  and a release build no longer dies if the check ever fails.
- `getmousepos()` no longer kills the editor after a click past column
  2^31. The window column it reports is computed in the width the answer
  carries, so it reads 2147483648 instead.

## [2026.08.11-3d48beea07]

### Changed

- Replaced the eight `printf`-style message wrappers (`semsg` and its
  siblings) with macros across all 724 of their call sites, leaving
  vimscript's own `printf()` as the only format machinery that is still a
  C variadic. No message text changes.
- Rewrote version reporting, covering `:version`, `nvim -v`, the intro
  screen and the `has("nvim-…")`/`has("patch-…")` predicates. `:version`
  now names the port and the version it was built from, and reports the
  real build profile and compiler instead of the CMake-era text the
  transpiler baked in; `Build type:` and `Compilation:` are gone.
- Swept the transpiler's C idioms out of the whole tree: NUL-terminated
  byte strings became C string literals, pointer arithmetic became the
  unsigned form, the `logmsg` C variadic became a macro, and the casts
  those changes made redundant are gone. ~2,000 sites; nothing observable
  changed.
- Rewrote the runtime layer, covering how `'runtimepath'` and
  `'packpath'` are built from `$XDG_*`, `$NVIM_APPNAME` and `--clean`,
  the search behind `:runtime`, `require()` and runtime-file completion,
  `:packadd`/`:packloadall` and the `after/` ordering they splice in,
  `:source` of a file, a buffer or a range in either language, and the
  script registry `:scriptnames`, `getscriptinfo()`, `<sfile>` and
  `getstacktrace()` read.
- Rewrote the channel layer behind jobs, RPC clients and `:terminal`, the
  `assert_*()` family and `v:errors`, logging and `--startuptime`
  profiling, the clipboard provider and the `input()`/`confirm()`
  prompts, the context stack behind `ctxget()`/`ctxset()`, and the
  remaining transpiled file-system and indent code.
- Rewrote the Ex command layer, everything an `:` command does once it has
  been parsed. That covers `:substitute` and `:global` including the `c`
  confirm prompt, `\=` replacements and `'inccommand'` preview;
  `:sort`/`:uniq`, `:move`/`:copy`, `:append`/`:insert`/`:change`/`:z`,
  `:left`/`:right`/`:center`, `:read`/`:write`/`:wall`, `:edit` and the
  buffer swap behind it, and `:!`/`:filter` and the shell they run;
  `:try`/`:catch`/`:finally`/`:throw` and the exception state they share
  with `:while` and `:function`; user-defined commands, their attributes
  and their completion; `:mksession`/`:mkview`/`:mkvimrc`; the help system
  and `:helptags`; the `:debug` prompt, breakpoints and profiling ranges;
  the argument list and `:argdo`/`:bufdo`/`:windo`/`:tabdo`; digraphs;
  command-line history; and the `nvim_cmd`/`nvim_parse_cmd` API pair.
- The table that decides which `:` commands exist and how each one's range
  and arguments are parsed is generated from `ex_cmds.lua` now, as the
  original generated it at build time. The emitted table is byte-identical
  to the one it replaces.

### Fixed

- Internal consistency checks that came from the original's `assert()` no
  longer abort a release build. 237 of them across the tree are debug-only
  now, as they are in the original, and a release `nvim` carries on past
  the cases it used to die on.
- `require()` from inside a `vim.uv` thread no longer aborts. Any module
  loaded off the main thread went through the runtime search path, which
  a debug build refused to touch there.
- A number too large for the option it appears in no longer kills the
  editor. `'cinoptions'`, `'breakindentopt'`, `'comments'`, `'spellsuggest'`,
  `'rulerformat'`, `:sign place`, `:breakadd` and the `:syntax` offsets all
  read their numeric fields the same way, and anything outside the
  representable range aborted the process, even from a modeline in a
  release build. Such a number is clamped now.
- A count too large for the command it appears in no longer ends the
  editor. `:digraph a: 4294967296`, `:2147483647verbose set` and the
  `verbose` and `tab` modifiers of `nvim_cmd()` saturate now.

## [2026.08.08-0be4297933]

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
- A `vim.mpack.Packer({ ext = ... })` no longer aborts the editor when it
  encodes a table whose metatable has no handler in the ext table.
- `vim.json.encode()` called from inside a metamethod of the value it is
  encoding no longer corrupts the document its caller was building.
- `vim.diff()` with a negative `ctxlen` no longer answers a hunk header that
  contradicts the hunk body; the count is clamped at zero.

## [2026.08.06-eb75350b02]

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

[Unreleased]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.15-2b1aee84f3...HEAD
[2026.08.15-2b1aee84f3]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.14-954ee76f86...2026.08.15-2b1aee84f3
[2026.08.14-954ee76f86]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.11-3d48beea07...2026.08.14-954ee76f86
[2026.08.11-3d48beea07]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.08-0be4297933...2026.08.11-3d48beea07
[2026.08.08-0be4297933]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.06-eb75350b02...2026.08.08-0be4297933
[2026.08.06-eb75350b02]: https://github.com/PsychoLlama/nvim.rs/compare/2026.08.02-af6bcec290...2026.08.06-eb75350b02
[2026.08.02-af6bcec290]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.30-4b9dee25d3...2026.08.02-af6bcec290
[2026.07.30-4b9dee25d3]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.27-dd16441f3f...2026.07.30-4b9dee25d3
[2026.07.27-dd16441f3f]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.26-d0c5cf2147...2026.07.27-dd16441f3f
[2026.07.26-d0c5cf2147]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.23-95cd63254c...2026.07.26-d0c5cf2147
[2026.07.23-95cd63254c]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.21-aa530a7...2026.07.23-95cd63254c
[2026.07.21-aa530a7]: https://github.com/PsychoLlama/nvim.rs/compare/2026.07.19-2a6342c...2026.07.21-aa530a7
[2026.07.19-2a6342c]: https://github.com/PsychoLlama/nvim.rs/commits/2026.07.19-2a6342c
