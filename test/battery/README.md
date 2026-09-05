# The differential battery

Thirty-two stored-baseline oracles plus a paired startup probe. Each row drives
the binary this tree builds through a fixed corpus, scrubs everything that names
_where_ or _when_ the run happened, and diffs the result against a baseline
committed here. A row says `IDENTICAL` or `DIFFERS`; the last line is
`BATTERY_EXIT=0` when every row agreed.

```
just battery              # all 33, ~15 min, logs under target/battery-logs
just battery my-label     # same, naming the log set
test/battery/keyverify.sh # one row, on its own
```

These are **behaviour** gates, not correctness gates: a baseline records what
nvim did at the commit it was cut from, not what it should do. A `DIFFERS` means
the behaviour moved — which is a bug report or a deliberate change, and you have
to say which.

## The rows

Each row is `<name>verify.sh` (build + diff), `<name>sweep.sh` (sandbox,
environment, the scrubs only the shell can see), `<name>sweep.lua` (the corpus)
and `<name>base/` (the baseline artifacts). Four rows deviate: `sess` and `undo`
use `<row>gold.{sh,lua}`, `ex` uses `ex-run.sh` + `exprobe.lua`/`parseprobe.lua`, and
`decode` has no sweep at all — `decodeverify.sh` runs the five
`decodecorpus-*` files directly.

| row      | what it watches                                                                               |
| -------- | --------------------------------------------------------------------------------------------- |
| `key`    | `getchar.rs`, `mapping.rs`, `keycodes.rs` — typeahead and the `:map` family                   |
| `scr`    | the screen pipeline: `drawscreen`, `drawline`, decorations, highlight, syntax, the popup menu |
| `eval`   | the value substrate — typval, lists/dicts, `userfunc`, `vars`, encode/decode                  |
| `au`     | `autocmd.rs` + `api/autocmd.rs`, as an event-ORDER log                                        |
| `vars`   | `v:` variables and the variable scopes                                                        |
| `op`     | `ops.rs`, `register.rs`, `change.rs`, `textobject.rs`, `edit.rs`                              |
| `fmt`    | `indent_c.rs`, `textformat.rs`, the indent options                                            |
| `diff`   | `diff.rs` and the vendored `xdiff/`                                                           |
| `utf`    | `mbyte.rs` and the UTF-8/latin1 surface                                                       |
| `cmd`    | the Ex layer through `nvim_parse_cmd()`/`nvim_cmd()`                                          |
| `rt`     | `runtime/` — `:runtime`, the rtp, packages, `:scriptnames`, `estack`                          |
| `sess`   | `:mksession`/`:mkview` byte goldens                                                           |
| `ex`     | the rehomed Ex probe: parse + excmd                                                           |
| `fs`     | the eval filesystem family (slowest of the Lua rows, ~30 s)                                   |
| `stl`    | `statusline.rs` — `build_stl_str_hl`, click definitions, ruler, tabline                       |
| `mouse`  | `mouse.rs` — `do_mouse`, `jump_to_mouse`, the wheel, `getmousepos()`                          |
| `menu`   | `menu.rs` — `:menu`, `menu_get`/`menu_info`, `:emenu`, `:popup`                               |
| `win`    | `window.rs` + `winfloat.rs` — every `wincmd`, the float config matrix, tabpages               |
| `buf`    | `buffer.rs` — `:ls`, addressing, the delete/unload lifecycle, `getbufinfo()`                  |
| `vt`     | `vterm/` — the escape parser, the pen, modes, mouse encoders, damage merging                  |
| `term`   | `terminal.rs` and its submodules, including a real pty                                        |
| `mark`   | `marktree.rs` and `marktree/` — the tree's SHAPE, via `nvim__buf_debug_extmarks`              |
| `re`     | `regexp.rs` and `regexp/` — both engines side by side under `'regexpengine'`                  |
| `opt`    | the option TABLE, one row per option for all 374, and `do_set`'s alphabet                     |
| `spell`  | `spellfile.rs` + `spellsuggest.rs` — 30 `:mkspell` cases, hashed `.spl`/`.sug`                |
| `pers`   | `memline`, `memfile`, `shada`, `fileio`, `bufwrite` — swap and ShaDa bytes                    |
| `undo`   | the undofile golden: 20 cases, 20 `.un~`, decoded field by field                              |
| `fold`   | `fold/` — the fold TREE under edits, with a per-case `'foldlevel'` scan                       |
| `jmark`  | `mark/` — named marks, the jumplist, the changelist                                           |
| `nav`    | `file_search.rs`, `path.rs`, `quickfix.rs`, `search.rs`, `tag.rs`                             |
| `decode` | json, msgpack, `vim.json`, `vim.mpack` and the msgpack-RPC transport                          |
| `ins`    | `insexpand/` — the CTRL-X alphabet, one line per KEYSTROKE                                    |
| probe    | `startprobe.py`: ~200 nvim _processes_ — argv, `$XDG_*`, `-l`/`-S`/`--api-info`, the log file |

Every `*verify.sh` and `*sweep.sh` carries a long header explaining what its row
paid for. Read it before touching the row; several of them record a hazard that
cost a whole slice to find.

## Re-cutting a baseline

Only when the behaviour change is **intended and reviewed**. Each `*verify.sh`
header gives the exact regeneration command; they all follow the same shape:

```sh
just build                              # never regenerate from a mutant binary
test/battery/keysweep.sh target/debug/nvim runtime test/battery/keybase base
git -C . rev-parse HEAD > test/battery/keybase/COMMIT
```

Three rules:

1. **A baseline change never shares a commit with the change it gates.** Land the
   behaviour change, watch the row go red, then re-cut in a follow-up commit whose
   body says why every changed line is expected. A gate that moves in the same
   commit as the code gates nothing.
2. **Build first.** A mutation harness leaves the binary built from its last
   mutant; a baseline cut from that compares mutant against mutant forever after.
3. **Read the diff line by line.** "The row is green again" is not review.

## Paths and formatting

Nothing here holds an absolute path: `$HERE` is the script's own directory and
`$REPO` (overridable) is its grandparent. The work directories are short and in
`/tmp` on purpose — nvim elides a message wider than the 80-column headless
screen, and a long fixture path is what turns an error text into `...<tail>`.

For the same reason LuaJIT elides a chunk name past ~60 characters, so every
`<SCRIPT>` mask has a second, basename-anchored form
(`'%.%.%.[^%s\'"]-<name>%.lua'` in Lua, `s#\.\.\.[^ "]*/<name>\.lua#` in sed).
`rtsweep` additionally masks `$HERE` **before** `$VIM`: now that the harness
lives inside the checkout, the checkout is a prefix of the script path.

The corpora are frozen text. `test/battery/` is in `.styluaignore` — stylua would
reflow the `.lua` sweeps and the baselines record `<SCRIPT> line N`. Edits inside
a `.lua` must be **line-neutral** (chain a `gsub` onto the same physical line)
for the same reason. `.gitignore` carries a `!test/battery/**/*.un~` negation so
the `*~` rule does not swallow the undo row's goldens.
