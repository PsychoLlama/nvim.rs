# Rust Crate Structure

All Rust code in `src/nvim-rs/`. Each crate handles a specific domain:

| Crate          | Purpose                                 |
| -------------- | --------------------------------------- |
| api            | API types and conversions               |
| arabic         | Arabic text shaping                     |
| arglist        | Argument list management (navigation, operations, entries) |
| ascii          | ASCII character predicates              |
| autocmd        | Autocommand events, groups, patterns, and execution |
| buffer         | Buffer state and validation             |
| buffer_updates | Buffer change tracking                  |
| bufwrite       | Buffer write operations (formats, encoding, BOM, validation) |
| change         | Buffer modification tracking and text editing |
| channel        | Channel infrastructure for RPC communication |
| charset        | Character classification and conversion |
| clipboard      | Clipboard integration (providers, selections, async operations) |
| cmdexpand      | Command-line completion and expansion   |
| cmdhist        | Command history                         |
| cmdline        | Command line mode (state, edit, history, keys, preview, UI, API, viewstate, fname) |
| collections    | Data structures (garray, hashtab, map)  |
| compositor     | Multi-grid compositing (layer stack, z-order, clipping, floating windows, blending, damage tracking) |
| context        | Context stack management                |
| cursor         | Cursor positioning and validation       |
| cursor_shape   | Cursor mode and shape                   |
| debugger       | Debugger infrastructure (DAP, breakpoints, stepping, state) |
| decoration     | Virtual text and decorations            |
| decoration_provider | Decoration provider infrastructure (callbacks, state, hl caching) |
| dict           | VimL dictionary operations and iteration |
| diff           | Diff mode (options, buffers, blocks, navigation, highlighting, scroll binding, operations, VimL functions) |
| digraph        | Digraph operations (lookup, validation, registration, input state machine, Vimscript function helpers) |
| drawline       | Line rendering (state machine, filler lines, virtual text, conceal, gutter, syntax highlighting) |
| drawscreen     | Screen update orchestration (redraw types, invalidation, scroll optimization, window refresh) |
| edit           | Edit mode (state, insert, keys, abbreviations, completion) |
| encoding       | Text encoding (base64, hashing)         |
| eval           | VimL built-in functions (math, bitwise, type, random) |
| eval_codec     | VimL value encoding/decoding (JSON, blob, escape)   |
| eval_exec      | VimL expression evaluation (eval0-eval5, operators, comparisons, error handling) |
| event          | Event loop and libuv integration        |
| extmark        | Extended marks for plugins              |
| ex_cmds        | Ex command implementations (read, write, substitute, global, sort, lines, display, shell, format) |
| ex_cmds2       | Additional Ex commands (listdo, autowrite, buffer checking, dialogs) |
| ex_docmd       | Ex command parsing                      |
| ex_eval        | Exception handling state                |
| fileio         | File I/O (encoding, read/write, backup, modeline, sync) |
| filesearch     | File search infrastructure for path, tags, cdpath      |
| fold           | Folding state and methods               |
| funcall        | VimL function call infrastructure (dispatch, lookup, argument validation) |
| fuzzy          | Fuzzy matching                          |
| getchar        | Typeahead buffer, input, stuff/redo buffers, mapping state |
| grid           | Screen grid operations (cell access, line copy/clear, allocation, memory management) |
| help           | Help system utilities                   |
| highlight      | Syntax highlighting and attributes      |
| highlight_group| Highlight group management and commands |
| indent         | Indentation calculation (getters, tabstops, helpers, checks) |
| indent_c       | C/C++/Java indentation (cindent)        |
| input          | Input handling (buffer sizes, state, ring buffer, breakcheck) |
| insexpand      | Insert-mode completion (state, modes, list, popup, sources) |
| keycodes       | Key code parsing                        |
| linematch      | Diff line alignment                     |
| list           | VimL list operations and iteration      |
| log            | Logging infrastructure (levels, filters, output destinations) |
| lua            | Lua integration (state, types, conversion, stdlib, callbacks) |
| main           | Main startup infrastructure (init, cleanup, signals, environment) |
| mapping        | Key mappings and abbreviations          |
| mark           | Marks and positions                     |
| marktree       | B-tree for extmarks at positions        |
| match          | Match highlighting (add, delete, lookup, positions, ranges, viml helpers) |
| math           | Math utilities                          |
| mbyte          | Multibyte and UTF-8 encoding            |
| memfile        | Swap file management (block headers, dirty state, page calculations) |
| memline        | Buffer memory management (B-tree, line access, modification, swap files, recovery) |
| memory         | Memory allocation wrappers              |
| memutil        | Memory and string utilities             |
| menu           | Menu system (tree structure, modes, popup, wildmenu, commands) |
| message        | Message system (history, formatting, dialogs, scrollback) |
| mouse          | Mouse event handling and selection      |
| move           | Cursor movement and scrolling           |
| msgpack        | MessagePack serialization               |
| msgpack_rpc    | MessagePack-RPC protocol for remote procedure calls |
| normal         | Normal mode (command dispatch table, motion commands, operators, visual mode) |
| ops            | Operator handling (shift, tilde, addsub, replace, join, delete, insert) |
| option         | Option system and configuration (accessors, callbacks, validation, :set parsing, scope, types, values) |
| optionstr      | Option string validation (fillchars, listchars, flags, comma-lists) |
| os             | OS abstractions (env, time, fs)         |
| path           | Path manipulation                       |
| plines         | Physical line display calculations      |
| popupmenu      | Popup menu state                        |
| profile        | Profiling utilities                     |
| quickfix       | Quickfix and location lists             |
| regexp         | Regexp pattern scanning utilities        |
| register       | Register operations                     |
| runtime        | Runtime file management and script sourcing |
| search         | Search and substitution                 |
| session        | Session persistence (:mksession, :mkview, flags, components) |
| shada          | Session data persistence (ShaDa files)  |
| sign           | Sign management (definitions, placement, queries, Ex commands, VimL functions) |
| spell          | Spell checking (word tree traversal, case folding, compound rules, edit distance scoring, soundfolding, .spl file I/O, navigation, commands) |
| state          | State management (global, mode, cursor, screen) |
| statusline     | Status/tab/winbar/ruler rendering, format parsing, click handling, statuscolumn, UI integration |
| strings        | String manipulation                     |
| syntax         | Syntax highlighting (patterns, state, clusters, groups, buffer attachment, Ex commands) |
| tag            | Tag system (stack, search, navigation, file iteration, jump orchestration, location lists) |
| terminal       | Terminal emulator state and utilities   |
| testing        | Testing framework (assertions, fixtures, runners, mocks) |
| vterm          | VTerm terminal emulation core           |
| textformat     | Text formatting options                 |
| textobject     | Text object selection and navigation    |
| tui            | Terminal UI (terminfo, input, output)   |
| typval         | VimL value type handling                |
| ugrid          | Unicode grid for TUI                    |
| ui             | UI core types and RemoteUI state        |
| ui_client      | UI client protocol (events, handlers, attachment) |
| undo           | Undo/redo system (undo tree, file I/O, :undolist, undofile(), undotree()) |
| unpacker       | MessagePack unpacking                   |
| usercmd        | User command definition, completion, execution, and parsing |
| userfunc       | User-defined functions (params, closure, funcref) |
| vars           | Variable storage and scope management (lookup, scope prefixes) |
| utf8proc       | utf8proc bindings                       |
| version        | Version checks                          |
| viewport       | Viewport and scroll management          |
| viml_parser    | VimL expression parser (tokens, literals, escapes) |
| window         | Window management (state, layout, frames, navigation, tabpages) |
| winfloat       | Floating windows (relative positioning, anchors, split, style, z-index) |
