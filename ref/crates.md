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
| cmdhist        | Command history                         |
| cmdline        | Command line state                      |
| collections    | Data structures (garray, hashtab)       |
| compositor     | Multi-grid compositing                  |
| context        | Context stack management                |
| cursor_shape   | Cursor mode and shape                   |
| decoration     | Virtual text and decorations            |
| diff           | Diff mode and output parsing            |
| digraph        | Digraph lookup                          |
| drawline       | Line rendering                          |
| drawscreen     | Screen drawing and separators           |
| edit           | Edit mode state                         |
| encoding       | Text encoding (base64, hashing)         |
| eval           | VimL expression evaluation helpers      |
| event          | Event loop and libuv integration        |
| ex_docmd       | Ex command parsing                      |
| ex_eval        | Exception handling state                |
| fileio         | File I/O utilities                      |
| fold           | Folding state and methods               |
| fuzzy          | Fuzzy matching                          |
| getchar        | Typeahead and input buffer              |
| grid           | Screen grid operations                  |
| help           | Help system utilities                   |
| highlight      | Syntax highlighting and attributes      |
| indent         | Indentation calculation                 |
| insexpand      | Insert-mode completion                  |
| keycodes       | Key code parsing                        |
| linematch      | Diff line alignment                     |
| lua            | Lua integration                         |
| mapping        | Key mappings and abbreviations          |
| mark           | Marks and positions                     |
| marktree       | B-tree for extmarks at positions        |
| math           | Math utilities                          |
| mbyte          | Multibyte and UTF-8 encoding            |
| memory         | Memory allocation wrappers              |
| memutil        | Memory and string utilities             |
| menu           | Menu system                             |
| msgpack        | MessagePack serialization               |
| normal         | Normal mode key processing and commands |
| ops            | Operator handling                       |
| option         | Option system and configuration         |
| os             | OS abstractions (env, time, fs)         |
| path           | Path manipulation                       |
| plines         | Physical line display calculations      |
| popupmenu      | Popup menu state                        |
| profile        | Profiling utilities                     |
| quickfix       | Quickfix and location lists             |
| regexp         | Regular expression engine               |
| register       | Register operations                     |
| search         | Search and substitution                 |
| spell          | Spell checking, file parsing, suggestions |
| statusline     | Status line rendering                   |
| strings        | String manipulation                     |
| tag            | Tag stack and navigation                |
| terminal       | Terminal emulator state and utilities   |
| textformat     | Text formatting options                 |
| textobject     | Text object selection and navigation    |
| tui            | Terminal UI                             |
| typval         | VimL value type handling                |
| ugrid          | Unicode grid for TUI                    |
| undo           | Undo/redo system                        |
| unpacker       | MessagePack unpacking                   |
| utf8proc       | utf8proc bindings                       |
| version        | Version checks                          |
| window         | Window state and layout                 |
