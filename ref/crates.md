# Rust Crate Structure

All Rust code in `src/nvim-rs/`. Each crate handles a specific domain:

| Crate          | Purpose                                 |
| -------------- | --------------------------------------- |
| api            | API types and conversions               |
| arabic         | Arabic text shaping                     |
| ascii          | ASCII character predicates              |
| autocmd        | Autocommand events and patterns         |
| buffer         | Buffer state and validation             |
| buffer_updates | Buffer change tracking                  |
| charset        | Character classification and conversion |
| cmdexpand      | Command-line completion and expansion   |
| cmdhist        | Command history                         |
| cmdline        | Command line mode (state, edit, history, keys, preview, UI, API, viewstate, fname) |
| collections    | Data structures (garray, hashtab)       |
| compositor     | Multi-grid compositing                  |
| context        | Context stack management                |
| cursor         | Cursor positioning and validation       |
| cursor_shape   | Cursor mode and shape                   |
| decoration     | Virtual text and decorations            |
| diff           | Diff mode and output parsing            |
| digraph        | Digraph lookup                          |
| drawline       | Line rendering                          |
| drawscreen     | Screen drawing and separators           |
| edit           | Edit mode state                         |
| encoding       | Text encoding (base64, hashing)         |
| eval           | VimL built-in functions (math, bitwise, type, random) |
| event          | Event loop and libuv integration        |
| extmark        | Extended marks for plugins              |
| ex_cmds        | Ex command implementations (read, write, substitute, global, sort, lines, display, shell, format) |
| ex_docmd       | Ex command parsing                      |
| ex_eval        | Exception handling state                |
| fileio         | File I/O (encoding, read/write, backup, modeline, sync) |
| fold           | Folding state and methods               |
| fuzzy          | Fuzzy matching                          |
| getchar        | Typeahead and input buffer              |
| grid           | Screen grid operations                  |
| help           | Help system utilities                   |
| highlight      | Syntax highlighting and attributes      |
| highlight_group| Highlight group management and commands |
| indent         | Indentation calculation                 |
| indent_c       | C/C++/Java indentation (cindent)        |
| insexpand      | Insert-mode completion                  |
| keycodes       | Key code parsing                        |
| linematch      | Diff line alignment                     |
| lua            | Lua integration                         |
| mapping        | Key mappings and abbreviations          |
| mark           | Marks and positions                     |
| marktree       | B-tree for extmarks at positions        |
| math           | Math utilities                          |
| mbyte          | Multibyte and UTF-8 encoding            |
| memline        | Buffer memory management (B-tree)       |
| memory         | Memory allocation wrappers              |
| memutil        | Memory and string utilities             |
| menu           | Menu system                             |
| message        | Message system (history, formatting, dialogs, scrollback) |
| mouse          | Mouse event handling and selection      |
| move           | Cursor movement and scrolling           |
| msgpack        | MessagePack serialization               |
| normal         | Normal mode key processing and commands |
| ops            | Operator handling (shift, tilde, addsub, replace, join, delete, insert) |
| option         | Option system and configuration         |
| os             | OS abstractions (env, time, fs)         |
| path           | Path manipulation                       |
| plines         | Physical line display calculations      |
| popupmenu      | Popup menu state                        |
| profile        | Profiling utilities                     |
| quickfix       | Quickfix and location lists             |
| regexp         | Regular expression engine               |
| register       | Register operations                     |
| runtime        | Runtime file management and script sourcing |
| search         | Search and substitution                 |
| shada          | Session data persistence (ShaDa files)  |
| sign           | Sign management and display             |
| spell          | Spell checking, file I/O, word tree, suggestions |
| statusline     | Status line rendering                   |
| strings        | String manipulation                     |
| syntax         | Syntax pattern matching and state machine |
| tag            | Tag stack and navigation                |
| terminal       | Terminal emulator state and utilities   |
| vterm          | VTerm terminal emulation core           |
| textformat     | Text formatting options                 |
| textobject     | Text object selection and navigation    |
| tui            | Terminal UI                             |
| typval         | VimL value type handling                |
| ugrid          | Unicode grid for TUI                    |
| undo           | Undo/redo system                        |
| unpacker       | MessagePack unpacking                   |
| utf8proc       | utf8proc bindings                       |
| version        | Version checks                          |
| viewport       | Viewport and scroll management          |
| window         | Window state and layout                 |
