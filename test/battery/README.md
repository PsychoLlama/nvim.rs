# The differential battery

Thirty-two baselined oracles plus a paired startup probe. Each row drives the
binary this tree builds through a fixed corpus, scrubs everything that names
_where_ or _when_ the run happened, and diffs the result against the same
corpus run through the binary of a **pinned reference commit**. A row says
`IDENTICAL` or `DIFFERS`; the last line is `BATTERY_EXIT=0` when every row
agreed.

Nothing generated is committed. The baselines used to be 276 files and 53 MiB
of `*base/` in git; they are now **cut on demand** and cached under `target/`.

```
just battery              # all 33, ~7 min warm, logs under target/battery-logs
just battery my-label     # same, naming the log set
test/battery/keyverify.sh # one row, on its own — same cache, same pin
```

These are **behaviour** gates, not correctness gates: a baseline records what
nvim did at the commit `BASE` names, not what it should do. A `DIFFERS` means
the behaviour moved — which is a bug report or a deliberate change, and you have
to say which.

## Where a baseline comes from

`BASE` holds one line: the commit the baselines speak for. Everything else
follows from it.

1. `refbin.sh` materialises that commit **once per checkout** — a detached
   `git worktree` under `target/battery/ref/<sha>/src`, built with its own
   `CARGO_TARGET_DIR` so it never races the main tree's `target/`. That
   directory is named `target/` on purpose: the reference binary's path has to
   end in `/target/debug/nvim`, because fourteen sweeps mask the binary out of
   their artifacts with `%S*/target/debug/nvim`, and a binary anywhere else
   would survive the mask and make every one of those rows differ on nothing
   but its own path.
2. `baseline.sh <row>` prints `target/battery/base/<sha>/<row>base`, cutting it
   on a miss by re-entering `<row>verify.sh --cut <nvim> <dir>`. The cut runs
   **the current tree's** sweep script and corpus — only the _binary_ comes
   from `BASE`. Scripts and corpora are versioned with the tree, and a row must
   always run the corpus it was just edited to run.
3. The head run then diffs against that directory exactly as it used to diff
   against a committed one. `$<ROW>_BASELINE` still overrides.

The cache key is the sha, so **a stale cache is impossible**: bump `BASE` and
every row misses and re-cuts; revert it and the old cut is still there.
`rm -rf target/battery` is always safe. A cut is stamped only on success
(`<row>base/.cut`), so an interrupted run re-cuts rather than comparing against
half-written artifacts.

Cost: the reference build, ~1 min 15 s, once. A cold `just battery` runs every
row twice — once for the cut, once for head — so it costs about twice a warm
one.

## The rows

Each row is `<name>verify.sh` (build + diff), `<name>sweep.sh` (sandbox,
environment, the scrubs only the shell can see), `<name>sweep.lua` (the corpus)
and a `<name>base/` cut into the cache. Four rows deviate: `sess` and `undo`
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

## Bumping BASE

Only when the behaviour change is **intended and reviewed**. There is no
re-cutting in place any more: a deliberate re-cut is one line of `BASE`.

```sh
git rev-parse --short HEAD > test/battery/BASE   # then commit that alone
just battery                                     # every row re-cuts and agrees
```

Three rules:

1. **A `BASE` bump is its own commit, and its body says what moved.** Land the
   behaviour change, watch the row go red, then bump in a follow-up commit that
   names every row that moved and why each changed line is expected. A gate
   that moves in the same commit as the code gates nothing.
2. **Bump to a commit whose binary you trust.** `BASE` is built from a clean
   worktree of that commit, so a mutation harness cannot leave a mutant behind
   — but a commit that was never green is still a mutant baseline.
3. **Read the diff line by line.** "The row is green again" is not review. The
   old cut is still on disk at `target/battery/base/<old-sha>/`, and
   `diff -r` against the new one is the review.

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

The corpora are frozen text. `test/battery/` is in `.styluaignore` — stylua
would reflow the `.lua` sweeps and the baselines record `<SCRIPT> line N`. Edits
inside a `.lua` must be **line-neutral** (chain a `gsub` onto the same physical
line) for the same reason.
