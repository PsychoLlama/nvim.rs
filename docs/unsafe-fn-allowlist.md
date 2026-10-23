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

The file column is checked too: a row excuses its item only in the file it
names (a private helper defined in two files lists both, comma-separated), and
a row naming a file that does not define the item fails the run.

The list prunes itself. `check_unsafe_fn_allowlist` fails the run when a row
excuses nothing: its function is gone, it moved out of the file the row names,
or it now carries a raw pointer or a C ABI and the needle would skip it anyway.
A row that stops being needed has to leave in the same commit, the way a
`PERIMETER` entry does.

The phase exit is `unsafe_fns_without_raw_params` = 0 with this list as the
floor. Adding a row lowers the count, so a row is a claim a reviewer reads:
the bar is the four classes above and nothing softer.

| item                             | file                                                                             | why the type system cannot hold it                                                                                                                |
| -------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `WinLineVars::advance_color_col` | `crates/nvim/src/drawline/state.rs`                                              | `field:` `color_cols` is a cursor into the window's `w_p_cc_cols`, walked until the array's negative terminator stops it.                         |
| `WinLineVars::color_col_attr`    | `crates/nvim/src/drawline/state.rs`                                              | `field:` reads through the same `color_cols` cursor before it advances it.                                                                        |
| `draw_virt_text_item`            | `crates/nvim/src/drawline/virttext.rs`                                           | `field:` walks the raw `char *` of each of `vt`'s chunks, which nothing in `VirtText` keeps alive.                                                |
| `marktree_clear`                 | `crates/nvim/src/marktree/mod.rs`                                                | `alloc:` frees every node of the tree; a `Node` or `MarkTreeIter` taken out of it dangles afterwards.                                             |
| `marktree_free_subtree`          | `crates/nvim/src/marktree/mod.rs`                                                | `alloc:` frees a subtree the caller must already have detached, and nothing may still name it.                                                    |
| `typed_items`                    | `crates/nvim/src/garray.rs`                                                      | `cap:` reinterprets `ga_data` as `ga_len` values of a `T` the array does not record.                                                              |
| `lookup_slot`                    | `crates/nvim/src/hashtab.rs`                                                     | `field:` compares against every key the table holds as a raw `char *`, each of which must still be NUL-terminated.                                |
| `Kvec::push`                     | `crates/nvim/src/kvec.rs`                                                        | `cap:` reallocates `items`, which must be null or a live allocation of exactly `capacity` elements.                                               |
| `Line::byte`                     | `crates/nvim/src/winlayer.rs`                                                    | `cap:` indexes the line's bytes by `idx` with no bounds check.                                                                                    |
| `Line::next_char`                | `crates/nvim/src/winlayer.rs`                                                    | `field:` steps a `StrCharInfo`, a raw cursor into the line that only its terminating NUL stops.                                                   |
| `Line::ended`                    | `crates/nvim/src/winlayer.rs`                                                    | `field:` dereferences the same raw cursor.                                                                                                        |
| `take_arg`                       | `crates/nvim/src/ui_client.rs`                                                   | `field:` takes a value out of an `Array`, whose `items` allocation the type does not own.                                                         |
| `ui_client_event_grid_resize`    | `crates/nvim/src/ui_client.rs`                                                   | `field:` reads the decoder's `Array`, whose `items` the type does not own.                                                                        |
| `ui_client_event_hl_attr_define` | `crates/nvim/src/ui_client.rs`                                                   | `field:` as `ui_client_event_grid_resize`.                                                                                                        |
| `ui_client_event_error_exit`     | `crates/nvim/src/ui_client.rs`                                                   | `field:` as `ui_client_event_grid_resize`.                                                                                                        |
| `ui_client_event_connect`        | `crates/nvim/src/ui_client.rs`                                                   | `field:` as `ui_client_event_grid_resize`.                                                                                                        |
| `ui_client_event_restart`        | `crates/nvim/src/ui_client.rs`                                                   | `field:` as `ui_client_event_grid_resize`.                                                                                                        |
| `dict_to_hlattrs`                | `crates/nvim/src/ui_client.rs`                                                   | `field:` walks the `Dict`'s `items` allocation, which the type does not own.                                                                      |
| `String_0::truncate`             | `crates/nvim/src/api/private/helpers/text.rs`                                    | `cap:` cuts the length to `size` without checking it against the allocation.                                                                      |
| `nvim_ui_attach`                 | `crates/nvim/src/api/ui/mod.rs`                                                  | `field:` reads `options`' raw entries, which the `ApiDict` keeps nothing alive for.                                                               |
| `ui_attach`                      | `crates/nvim/src/api/ui/mod.rs`                                                  | `field:` as `nvim_ui_attach`, which it forwards to.                                                                                               |
| `TypVal::bit_copy`               | `crates/nvim/src/eval/typval/handle.rs`                                          | `alloc:` duplicates the value's bits without touching its refcount, so exactly one of the two may ever be released.                               |
| `ListRef::from_owned`            | `crates/nvim/src/eval/typval/handle.rs`                                          | `alloc:` takes over a reference to the list the `NonNull` names — a pointer the needle does not match — and gives it back on drop.                |
| `DictRef::from_owned`            | `crates/nvim/src/eval/typval/handle.rs`                                          | `alloc:` as `ListRef::from_owned`, for a dictionary.                                                                                              |
| `ProviderScope::leave`           | `crates/nvim/src/eval/funcs/channel.rs`                                          | `field:` pops the execution stack entry `enter` pushed; nothing may have touched it in between.                                                   |
| `utfc_next`                      | `crates/nvim/src/mbyte/utf8/mod.rs`                                              | `field:` steps a `StrCharInfo`, a raw cursor whose walk only the string's NUL ends.                                                               |
| `ClickDefs::tabline`             | `crates/nvim/src/mouse/mod.rs`                                                   | `cap:` wraps the tabline's click array, which [`ClickDefs::at`] then offsets by a column with no bounds check.                                    |
| `hlattrs2dict`                   | `crates/nvim/src/highlight/dict.rs`                                              | `cap:` fills both dicts to `HLATTRS_DICT_SIZE` entries without growing them.                                                                      |
| `dict2hlattrs`                   | `crates/nvim/src/highlight/dict.rs`                                              | `field:` reads each `Object` by its tag, which has to match the value the field actually holds.                                                   |
| `write_chunk`                    | `crates/nvim/src/mpack/mpack_core.rs`                                            | `cap:` copies `tok->length` bytes out of `tok`'s `chunk_ptr`, which nothing checks.                                                               |
| `put_byte`                       | `crates/nvim/src/msgpack_rpc/channel/envelope.rs`                                | `cap:` writes one byte at the packer's cursor without checking for room.                                                                          |
| `packer_buffer_finish`           | `crates/nvim/src/msgpack_rpc/channel/envelope.rs`                                | `alloc:` hands the packer's block to the write buffer, or frees it, and reads the addressee slice [`packer_channels`] stashed.                    |
| `Chan::unpacker`                 | `crates/nvim/src/msgpack_rpc/channel/mod.rs`                                     | `field:` hands back a `&'a mut Unpacker` over the channel's raw decoder, with a lifetime nothing bounds.                                          |
| `StlJob::run`                    | `crates/nvim/src/statusline/mod.rs`                                              | `field:` `out` must not alias `NameBuff`, which the format expander uses as scratch.                                                              |
| `name_in`                        | `crates/nvim/src/statusline/tabline.rs`                                          | `cap:` reads `name` up to a NUL that must be inside the buffer.                                                                                   |
| `fragment_bytes`                 | `crates/nvim/src/terminal/callbacks.rs, crates/nvim/src/terminal/termrequest.rs` | `ffi:` builds a slice over `frag.str` for `frag.len()` bytes on vterm's callback frame; only vterm's own contract says that is readable.          |
| `term_set_title`                 | `crates/nvim/src/terminal/callbacks.rs`                                          | `ffi:` forwards a vterm fragment to [`fragment_bytes`].                                                                                           |
| `accumulate`                     | `crates/nvim/src/terminal/termrequest.rs`                                        | `ffi:` as [`term_set_title`].                                                                                                                     |
| `set_pen_attr`                   | `crates/nvim/src/vterm/pen.rs`                                                   | `ffi:` calls through the vterm state's raw callback table, which must still be the one the session installed.                                     |
| `decode_cursor_entry`            | `crates/nvim/src/tui/cursor.rs`                                                  | `field:` reads the raw bytes of the dict's keys and values; the api value types keep nothing alive.                                               |
| `tui_mode_info_set`              | `crates/nvim/src/tui/events.rs`                                                  | `field:` walks `args` as an array of mode dictionaries, reading their raw storage.                                                                |
| `tui_ui_send`                    | `crates/nvim/src/tui/events.rs`                                                  | `field:` reads the raw bytes `content` points at.                                                                                                 |
| `tui_set_title`                  | `crates/nvim/src/tui/events.rs`                                                  | `field:` reads the raw bytes `title` points at.                                                                                                   |
| `tui_option_set`                 | `crates/nvim/src/tui/events.rs`                                                  | `field:` reads `name`'s raw bytes, and `value` by the tag the option's name implies.                                                              |
| `tui_chdir`                      | `crates/nvim/src/tui/events.rs`                                                  | `field:` reads the raw bytes `path` points at.                                                                                                    |
| `tui_screenshot`                 | `crates/nvim/src/tui/paint.rs`                                                   | `field:` as [`tui_chdir`].                                                                                                                        |
| `csi_param_value`                | `crates/nvim/src/tui/termkey/driver_csi.rs`                                      | `field:` `param` addresses the reader's buffer, and only `termkey_interpret_csi` puts it there.                                                   |
| `param_bytes`                    | `crates/nvim/src/tui/termkey/driver_csi.rs`                                      | `field:` as [`csi_param_value`], and hands back a slice whose lifetime nothing bounds.                                                            |
| `ui_comp_msg_set_pos`            | `crates/nvim/src/ui_compositor/mod.rs`                                           | `field:` reads the raw bytes `sep_char` points at.                                                                                                |
| `buckets`                        | `crates/nvim/src/map/table.rs`                                                   | `cap:` builds a slice of `h.n_buckets` entries over `h.hash`.                                                                                     |
| `buckets_mut`                    | `crates/nvim/src/map/table.rs`                                                   | `cap:` as [`buckets`], and the `&mut` may not alias another view of the table.                                                                    |
| `LuaRefArg::new`                 | `crates/nvim/src/lua/api_wrappers/mod.rs`                                        | `alloc:` takes ownership of the Lua reference and releases it on drop.                                                                            |
| `StringBuf::with_capacity`       | `crates/nvim/src/lua/executor/excmds.rs`                                         | `alloc:` hands back an allocation the caller has to [`StringBuf::free`] or `into_raw`.                                                            |
| `StringBuf::free`                | `crates/nvim/src/lua/executor/excmds.rs`                                         | `alloc:` frees the block; the buffer must not be used again.                                                                                      |
| `api_free_luaref`                | `crates/nvim/src/lua/executor/refs.rs`                                           | `alloc:` releases a reference that must be one the main state still holds.                                                                        |
| `free_reuse_blks`                | `crates/nvim/src/memory/arena.rs`                                                | `alloc:` frees the arena's reuse list; no block handed out of it may still be in use.                                                             |
| `try_to_free_memory`             | `crates/nvim/src/memory/mod.rs`                                                  | `alloc:` drops the scrollback and the memfile caches from inside an allocation.                                                                   |
| `os_fchown`                      | `crates/nvim/src/os/fs/meta.rs`                                                  | `alloc:` acts on a descriptor the process has to own — `fd` is a bare `c_int`, with none of `OwnedFd`'s guarantee behind it.                      |
| `os_set_cloexec`                 | `crates/nvim/src/os/fs/mod.rs`                                                   | `alloc:` as [`os_fchown`].                                                                                                                        |
| `os_close`                       | `crates/nvim/src/os/fs/mod.rs`                                                   | `alloc:` as [`os_fchown`], and closing another owner's descriptor is what the type cannot stop.                                                   |
| `os_dup`                         | `crates/nvim/src/os/fs/mod.rs`                                                   | `alloc:` as [`os_fchown`].                                                                                                                        |
| `os_fsync`                       | `crates/nvim/src/os/fs/mod.rs`                                                   | `alloc:` as [`os_fchown`].                                                                                                                        |
| `input_enqueue`                  | `crates/nvim/src/os/input.rs`                                                    | `field:` reads the raw bytes `keys` points at.                                                                                                    |
| `logger_gc`                      | `crates/nvim/src/lua/treesitter/logger.rs`                                       | `ffi:` a Lua `__gc` callback: it reads `logger.payload` as a `TSLuaLoggerOpts` and frees it, so a logger from anywhere else is a use-after-free.  |
| `towlower`                       | `crates/nvim/src/os/cshim.rs`                                                    | `ffi:` stands in for the libc declaration under `cfg(miri)` and keeps its `unsafe`, or every caller's block becomes `unused_unsafe` in that lane. |
| `towupper`                       | `crates/nvim/src/os/cshim.rs`                                                    | `ffi:` as [`towlower`].                                                                                                                           |
