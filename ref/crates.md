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
| bufwrite       | Buffer write operations (formats, encoding, BOM, validation) |
| change         | Buffer modification tracking and text editing |
| channel        | Channel infrastructure for RPC communication |
| charset        | Character classification and conversion |
| cmdexpand      | Command-line completion and expansion   |
| cmdhist        | Command history                         |
| cmdline        | Command line mode (state, edit, history, keys, preview, UI, API, viewstate, fname) |
| collections    | Data structures (garray, hashtab)       |
| compositor     | Multi-grid compositing                  |
| context        | Context stack management                |
| cursor         | Cursor positioning and validation       |
| cursor_shape   | Cursor mode and shape                   |
| debugger       | Debugger infrastructure (DAP, breakpoints, stepping, state) |
| decoration     | Virtual text and decorations            |
| dict           | VimL dictionary operations and iteration |
| diff           | Diff mode and output parsing            |
| digraph        | Digraph lookup                          |
| drawline       | Line rendering                          |
| drawscreen     | Screen drawing and separators           |
| edit           | Edit mode (state, insert, keys, abbreviations, completion) |
| encoding       | Text encoding (base64, hashing)         |
| eval           | VimL built-in functions (math, bitwise, type, random) |
| eval_codec     | VimL value encoding/decoding (JSON, blob, escape)   |
| eval_exec      | VimL expression evaluation (operators, comparisons) |
| event          | Event loop and libuv integration        |
| extmark        | Extended marks for plugins              |
| ex_cmds        | Ex command implementations (read, write, substitute, global, sort, lines, display, shell, format) |
| ex_docmd       | Ex command parsing                      |
| ex_eval        | Exception handling state                |
| fileio         | File I/O (encoding, read/write, backup, modeline, sync) |
| filesearch     | File search infrastructure for path, tags, cdpath      |
| fold           | Folding state and methods               |
| funcall        | VimL function call infrastructure       |
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
| list           | VimL list operations and iteration      |
| lua            | Lua integration (state, types, conversion, stdlib, callbacks) |
| mapping        | Key mappings and abbreviations          |
| mark           | Marks and positions                     |
| marktree       | B-tree for extmarks at positions        |
| match          | Match highlighting (IDs, priorities, positions) |
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
| msgpack_rpc    | MessagePack-RPC protocol for remote procedure calls |
| normal         | Normal mode key processing and commands |
| ops            | Operator handling (shift, tilde, addsub, replace, join, delete, insert) |
| option         | Option system and configuration         |
| os             | OS abstractions (env, time, fs)         |
| path           | Path manipulation                       |
| plines         | Physical line display calculations      |
| popupmenu      | Popup menu state                        |
| profile        | Profiling utilities                     |
| quickfix       | Quickfix and location lists             |
| regexp         | Regular expression engine (BT/NFA compilation, execution, substitution, special matching) |
| register       | Register operations                     |
| runtime        | Runtime file management and script sourcing |
| search         | Search and substitution                 |
| shada          | Session data persistence (ShaDa files)  |
| sign           | Sign management and display             |
| spell          | Spell checking (word tree traversal, edit distance, suggestion scoring, .spl file I/O) |
| statusline     | Status/tab/winbar rendering, format parsing, click handling |
| strings        | String manipulation                     |
| syntax         | Syntax highlighting (patterns, state, clusters, groups, buffer attachment, Ex commands) |
| tag            | Tag stack and navigation                |
| terminal       | Terminal emulator state and utilities   |
| vterm          | VTerm terminal emulation core           |
| textformat     | Text formatting options                 |
| textobject     | Text object selection and navigation    |
| tui            | Terminal UI (terminfo, input, output)   |
| typval         | VimL value type handling                |
| ugrid          | Unicode grid for TUI                    |
| ui             | UI core types and RemoteUI state        |
| undo           | Undo/redo system                        |
| unpacker       | MessagePack unpacking                   |
| usercmd        | User command definition, completion, execution, and parsing |
| userfunc       | User-defined functions (params, closure, funcref) |
| vars           | Variable storage and scope management   |
| utf8proc       | utf8proc bindings                       |
| version        | Version checks                          |
| viewport       | Viewport and scroll management          |
| window         | Window management (state, layout, frames, navigation, tabpages) |
