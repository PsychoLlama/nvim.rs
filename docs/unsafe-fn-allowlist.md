# The `unsafe fn` allowlist

A quarter of the tree's `unsafe fn`s take no raw pointer at all. Their
parameters were retyped in an earlier phase, the keyword stayed, and a
`# Safety` section was written to satisfy the ratchet — "the window must be
live" on a `Win`, "`tv` must point at an initialized typval" on a `&TypVal`,
"main thread only" on editor code that is main-thread everywhere. None of
those is a contract: they name what the type already holds. The fix is to
delete the keyword, never to keep it and write the section.

`unsafe_fns_without_raw_params` counts what is left of that class, and this
file is its floor. A row here says the signature shows no raw pointer **and
the `unsafe` is still earned** — that the obligation is real and lives
somewhere the type system cannot reach.

## What a row has to say

Every row opens with one of four reason classes, and nothing else qualifies:

- **`alloc:`** — it returns raw memory, or hands ownership of an allocation
  across the call, and the caller is the one who has to free it.
- **`cap:`** — a capacity the caller must honour: a buffer of at least _n_
  bytes, an index the callee will not bounds-check, a length in elements
  rather than bytes.
- **`ffi:`** — it is an FFI callback or runs on a foreign frame: the C
  library, the `lua_State` stack, or a signal/thread context whose invariants
  no Rust type describes.
- **`field:`** — a raw-pointer _field_ reached through a borrowed struct. The
  parameter is a reference, so the needle sees nothing, but the body walks a
  pointer the struct holds. This is the string and state phases' debt: the row
  leaves when the field does, and it names the field so a reader can tell.

A `NonNull<T>` parameter is a raw pointer the needle deliberately does not
match, so it takes a row (`alloc:` or `field:`, whichever the call is doing).

## How the ratchet enforces it

`unsafe_fn_allowlist` in `scripts/ratchet.py` reads the table below. The key
is the item: `Owner::name` for a method — a bare name does not excuse a
method, or the row would quietly cover every type that grows one — and `name`
for a free function.

The list prunes itself. `check_unsafe_fn_allowlist` fails the run when a row
excuses nothing: its function is gone, or it now carries a raw pointer or a C
ABI and the needle would skip it anyway. A row that stops being needed has to
leave in the same commit, the way a `PERIMETER` entry does.

The phase exit is `unsafe_fns_without_raw_params` = 0 with this list as the
floor. Adding a row lowers the count, so a row is a claim a reviewer reads:
the bar is the four classes above and nothing softer.

| item                             | file                                   | why the type system cannot hold it                                                                                                                                            |
| -------------------------------- | -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Matches::with_capacity`         | `crates/nvim/src/optionstr/expand.rs`  | `cap:` the array it allocates is sized for `capacity` entries and [`Matches::push`] bounds-checks nothing.                                                                    |
| `WinLineVars::advance_color_col` | `crates/nvim/src/drawline/state.rs`    | `field:` `color_cols` is a cursor into the window's `w_p_cc_cols`, walked until the array's negative terminator stops it.                                                     |
| `WinLineVars::color_col_attr`    | `crates/nvim/src/drawline/state.rs`    | `field:` reads through the same `color_cols` cursor before it advances it.                                                                                                    |
| `draw_virt_text_item`            | `crates/nvim/src/drawline/virttext.rs` | `field:` walks the raw `char *` of each of `vt`'s chunks, which nothing in `VirtText` keeps alive.                                                                            |
| `as_filter`                      | `crates/nvim/src/marktree/iter.rs`     | `field:` `MetaFilter` is a `*const uint32_t` behind a type alias, so the needle sees no pointer; this dereferences it and hands back a borrow with a lifetime nothing bounds. |
| `marktree_itr_get_filter`        | `crates/nvim/src/marktree/iter.rs`     | `field:` takes the same alias-hidden `*const uint32_t` and passes it to [`as_filter`].                                                                                        |
| `marktree_itr_next_filter`       | `crates/nvim/src/marktree/iter.rs`     | `field:` as `marktree_itr_get_filter`.                                                                                                                                        |
| `marktree_itr_step_out_filter`   | `crates/nvim/src/marktree/iter.rs`     | `field:` as `marktree_itr_get_filter`.                                                                                                                                        |
| `marktree_clear`                 | `crates/nvim/src/marktree/mod.rs`      | `alloc:` frees every node of the tree; a `Node` or `MarkTreeIter` taken out of it dangles afterwards.                                                                         |
| `marktree_free_subtree`          | `crates/nvim/src/marktree/mod.rs`      | `alloc:` frees a subtree the caller must already have detached, and nothing may still name it.                                                                                |
| `hl_msg_free`                    | `crates/nvim/src/message/history.rs`   | `alloc:` takes ownership of the message's chunks and frees them.                                                                                                              |
| `hl_msg_push`                    | `crates/nvim/src/message/mod.rs`       | `field:` grows the message's `items` allocation in place, so any pointer taken into it before the call dangles after.                                                         |
| `KeyBuffer::free`                | `crates/nvim/src/getchar/buffers.rs`   | `alloc:` frees the whole block chain; nothing may hold a pointer into it.                                                                                                     |
| `KeyBuffer::read`                | `crates/nvim/src/getchar/buffers.rs`   | `alloc:` frees each block as it is consumed.                                                                                                                                  |
| `KeyBufferRef::free`             | `crates/nvim/src/getchar/buffers.rs`   | `alloc:` as `KeyBuffer::free`, through the global handle.                                                                                                                     |
| `KeyBufferRef::read`             | `crates/nvim/src/getchar/buffers.rs`   | `alloc:` as `KeyBuffer::read`, through the global handle.                                                                                                                     |
| `get_inserted`                   | `crates/nvim/src/getchar/buffers.rs`   | `alloc:` answers an `xmalloc`ed `char *` inside a `String_0` that the caller has to free.                                                                                     |
| `free_typebuf`                   | `crates/nvim/src/getchar/typeahead.rs` | `alloc:` frees both typeahead buffers; nothing may hold a pointer into either.                                                                                                |
| `del_typebuf`                    | `crates/nvim/src/getchar/typeahead.rs` | `cap:` `offset + len` has to be within the typeahead, which is not bounds-checked.                                                                                            |
| `TypeAhead::delete`              | `crates/nvim/src/getchar/typeahead.rs` | `cap:` as `del_typebuf`, which is its one caller.                                                                                                                             |
| `esc_leaves_insert`              | `crates/nvim/src/getchar/peek.rs`      | `cap:` pushes three bytes back into the typeahead without growing it first.                                                                                                   |
| `read_redo`                      | `crates/nvim/src/getchar/redo.rs`      | `field:` walks a cursor into the redo buffer that a call with `init` sets up, and that a free in between leaves dangling.                                                     |
| `copy_redo`                      | `crates/nvim/src/getchar/redo.rs`      | `field:` reads through the same cursor as `read_redo`.                                                                                                                        |
| `paste_store`                    | `crates/nvim/src/getchar/paste.rs`     | `field:` reads `str.data` for `str.size` bytes; a `String_0` keeps nothing alive.                                                                                             |
