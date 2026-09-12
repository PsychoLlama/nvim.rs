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

| item                             | file                                          | why the type system cannot hold it                                                                                                 |
| -------------------------------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `Matches::with_capacity`         | `crates/nvim/src/optionstr/expand.rs`         | `cap:` the array it allocates is sized for `capacity` entries and [`Matches::push`] bounds-checks nothing.                         |
| `WinLineVars::advance_color_col` | `crates/nvim/src/drawline/state.rs`           | `field:` `color_cols` is a cursor into the window's `w_p_cc_cols`, walked until the array's negative terminator stops it.          |
| `WinLineVars::color_col_attr`    | `crates/nvim/src/drawline/state.rs`           | `field:` reads through the same `color_cols` cursor before it advances it.                                                         |
| `draw_virt_text_item`            | `crates/nvim/src/drawline/virttext.rs`        | `field:` walks the raw `char *` of each of `vt`'s chunks, which nothing in `VirtText` keeps alive.                                 |
| `marktree_clear`                 | `crates/nvim/src/marktree/mod.rs`             | `alloc:` frees every node of the tree; a `Node` or `MarkTreeIter` taken out of it dangles afterwards.                              |
| `marktree_free_subtree`          | `crates/nvim/src/marktree/mod.rs`             | `alloc:` frees a subtree the caller must already have detached, and nothing may still name it.                                     |
| `hl_msg_free`                    | `crates/nvim/src/message/history.rs`          | `alloc:` takes ownership of the message's chunks and frees them.                                                                   |
| `hl_msg_push`                    | `crates/nvim/src/message/mod.rs`              | `field:` grows the message's `items` allocation in place, so any pointer taken into it before the call dangles after.              |
| `KeyBuffer::free`                | `crates/nvim/src/getchar/buffers.rs`          | `alloc:` frees the whole block chain; nothing may hold a pointer into it.                                                          |
| `KeyBuffer::read`                | `crates/nvim/src/getchar/buffers.rs`          | `alloc:` frees each block as it is consumed.                                                                                       |
| `KeyBufferRef::free`             | `crates/nvim/src/getchar/buffers.rs`          | `alloc:` as `KeyBuffer::free`, through the global handle.                                                                          |
| `KeyBufferRef::read`             | `crates/nvim/src/getchar/buffers.rs`          | `alloc:` as `KeyBuffer::read`, through the global handle.                                                                          |
| `get_inserted`                   | `crates/nvim/src/getchar/buffers.rs`          | `alloc:` answers an `xmalloc`ed `char *` inside a `String_0` that the caller has to free.                                          |
| `free_typebuf`                   | `crates/nvim/src/getchar/typeahead.rs`        | `alloc:` frees both typeahead buffers; nothing may hold a pointer into either.                                                     |
| `del_typebuf`                    | `crates/nvim/src/getchar/typeahead.rs`        | `cap:` `offset + len` has to be within the typeahead, which is not bounds-checked.                                                 |
| `TypeAhead::delete`              | `crates/nvim/src/getchar/typeahead.rs`        | `cap:` as `del_typebuf`, which is its one caller.                                                                                  |
| `esc_leaves_insert`              | `crates/nvim/src/getchar/peek.rs`             | `cap:` pushes three bytes back into the typeahead without growing it first.                                                        |
| `read_redo`                      | `crates/nvim/src/getchar/redo.rs`             | `field:` walks a cursor into the redo buffer that a call with `init` sets up, and that a free in between leaves dangling.          |
| `copy_redo`                      | `crates/nvim/src/getchar/redo.rs`             | `field:` reads through the same cursor as `read_redo`.                                                                             |
| `paste_store`                    | `crates/nvim/src/getchar/paste.rs`            | `field:` reads `str.data` for `str.size` bytes; a `String_0` keeps nothing alive.                                                  |
| `flag_fallback`                  | `crates/nvim/src/ex_cmds/sort.rs`             | `cap:` offsets the command's argument by `at` without checking it against the string's length.                                     |
| `match_range`                    | `crates/nvim/src/ex_cmds/sort.rs`             | `cap:` `vim_regexec` reads a C string, so the byte past `line`'s last must be a NUL — which a `&mut [u8]` does not say.            |
| `number_key`                     | `crates/nvim/src/ex_cmds/sort.rs`             | `cap:` indexes `line` by `start..end` without a bounds check.                                                                      |
| `emit_line`                      | `crates/nvim/src/ex_cmds/text.rs`             | `cap:` reads the `IOSIZE` buffer up to a NUL that must be inside it.                                                               |
| `sub_set_replacement`            | `crates/nvim/src/ex_cmds/subst/parse.rs`      | `alloc:` takes over two `xmalloc`ed `char *` and frees what it displaces.                                                          |
| `check_regexp_delim`             | `crates/nvim/src/ex_cmds/subst/parse.rs`      | `cap:` indexes the ctype table by `c`, which must be a `char` value (`-128..=255`).                                                |
| `TempFile::as_cstr`              | `crates/nvim/src/ex_cmds/filter.rs`           | `field:` hands back a `&CStr` borrowed from the struct's raw `char *`, whose lifetime nothing bounds.                              |
| `typed_items`                    | `crates/nvim/src/garray.rs`                   | `cap:` reinterprets `ga_data` as `ga_len` values of a `T` the array does not record.                                               |
| `lookup_slot`                    | `crates/nvim/src/hashtab.rs`                  | `field:` compares against every key the table holds as a raw `char *`, each of which must still be NUL-terminated.                 |
| `InitVec::grow`                  | `crates/nvim/src/kvec.rs`                     | `cap:` reallocates `items`, which must be the inline array or a live allocation of exactly `capacity` elements.                    |
| `Kvec::push`                     | `crates/nvim/src/kvec.rs`                     | `cap:` as `InitVec::grow`, for a vector whose `items` may also be null.                                                            |
| `Line::byte`                     | `crates/nvim/src/winlayer.rs`                 | `cap:` indexes the line's bytes by `idx` with no bounds check.                                                                     |
| `Line::next_char`                | `crates/nvim/src/winlayer.rs`                 | `field:` steps a `StrCharInfo`, a raw cursor into the line that only its terminating NUL stops.                                    |
| `Line::ended`                    | `crates/nvim/src/winlayer.rs`                 | `field:` dereferences the same raw cursor.                                                                                         |
| `ctx_restore_funcs`              | `crates/nvim/src/context.rs`                  | `field:` reads the context's `funcs` as raw `char *`, which must still be NUL-terminated.                                          |
| `take_arg`                       | `crates/nvim/src/ui_client.rs`                | `field:` takes a value out of an `Array`, whose `items` allocation the type does not own.                                          |
| `ui_client_event_grid_resize`    | `crates/nvim/src/ui_client.rs`                | `field:` reads the decoder's `Array`, whose `items` the type does not own.                                                         |
| `ui_client_event_hl_attr_define` | `crates/nvim/src/ui_client.rs`                | `field:` as `ui_client_event_grid_resize`.                                                                                         |
| `ui_client_event_error_exit`     | `crates/nvim/src/ui_client.rs`                | `field:` as `ui_client_event_grid_resize`.                                                                                         |
| `ui_client_event_connect`        | `crates/nvim/src/ui_client.rs`                | `field:` as `ui_client_event_grid_resize`.                                                                                         |
| `ui_client_event_restart`        | `crates/nvim/src/ui_client.rs`                | `field:` as `ui_client_event_grid_resize`.                                                                                         |
| `dict_to_hlattrs`                | `crates/nvim/src/ui_client.rs`                | `field:` walks the `Dict`'s `items` allocation, which the type does not own.                                                       |
| `nvim_del_augroup_by_name`       | `crates/nvim/src/api/autocmd/group.rs`        | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `get_augroup_from_object`        | `crates/nvim/src/api/autocmd/group.rs`        | `field:` reads the raw bytes `group` points at; the api value types keep nothing alive.                                            |
| `push_spans`                     | `crates/nvim/src/api/autocmd/pattern.rs`      | `field:` reads the raw bytes `patterns`, `text` point at; the api value types keep nothing alive.                                  |
| `autocmd_dict`                   | `crates/nvim/src/api/autocmd/query.rs`        | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim__buf_stats`                | `crates/nvim/src/api/buffer/attach.rs`        | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_buf_del_mark`              | `crates/nvim/src/api/buffer/marks.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_get_mark`              | `crates/nvim/src/api/buffer/marks.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_get_var`               | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_get_keymap`            | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `mode` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_del_keymap`            | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `mode`, `lhs` point at; the api value types keep nothing alive.                                       |
| `nvim_buf_set_var`               | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_buf_del_var`               | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_set_name`              | `crates/nvim/src/api/buffer/props.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `resolve_command`                | `crates/nvim/src/api/command/cmd.rs`          | `field:` reads the raw bytes `cmd` points at; the api value types keep nothing alive.                                              |
| `collect_args`                   | `crates/nvim/src/api/command/cmd.rs`          | `field:` reads the raw bytes `given`, `args` point at; the api value types keep nothing alive.                                     |
| `nvim_del_user_command`          | `crates/nvim/src/api/command/user.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_del_user_command`      | `crates/nvim/src/api/command/user.rs`         | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_add_highlight`         | `crates/nvim/src/api/deprecated/bufhl.rs`     | `field:` reads the raw bytes `hl_group` points at; the api value types keep nothing alive.                                         |
| `nvim_exec`                      | `crates/nvim/src/api/deprecated/eval.rs`      | `field:` reads the raw bytes `src` points at; the api value types keep nothing alive.                                              |
| `nvim_command_output`            | `crates/nvim/src/api/deprecated/eval.rs`      | `field:` reads the raw bytes `command` points at; the api value types keep nothing alive.                                          |
| `nvim_get_hl_by_id`              | `crates/nvim/src/api/deprecated/highlight.rs` | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_get_hl_by_name`            | `crates/nvim/src/api/deprecated/highlight.rs` | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `buffer_get_line`                | `crates/nvim/src/api/deprecated/lines.rs`     | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `buffer_get_line_slice`          | `crates/nvim/src/api/deprecated/lines.rs`     | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_get_option_info`           | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_set_option`                | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_get_option`                | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_get_option`            | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_buf_set_option`            | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_win_get_option`            | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_win_set_option`            | `crates/nvim/src/api/deprecated/options.rs`   | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `buffer_set_var`                 | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `buffer_del_var`                 | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `window_set_var`                 | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `window_del_var`                 | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `tabpage_set_var`                | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `tabpage_del_var`                | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `vim_set_var`                    | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `vim_del_var`                    | `crates/nvim/src/api/deprecated/vars.rs`      | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `write_msg`                      | `crates/nvim/src/api/deprecated/write.rs`     | `field:` reads the raw bytes `message` points at; the api value types keep nothing alive.                                          |
| `nvim_out_write`                 | `crates/nvim/src/api/deprecated/write.rs`     | `field:` reads the raw bytes `str` points at; the api value types keep nothing alive.                                              |
| `nvim_err_write`                 | `crates/nvim/src/api/deprecated/write.rs`     | `field:` reads the raw bytes `str` points at; the api value types keep nothing alive.                                              |
| `nvim_err_writeln`               | `crates/nvim/src/api/deprecated/write.rs`     | `field:` reads the raw bytes `str` points at; the api value types keep nothing alive.                                              |
| `nvim_error_event`               | `crates/nvim/src/api/events.rs`               | `field:` reads the raw bytes `msg` points at; the api value types keep nothing alive.                                              |
| `nvim_ui_term_event`             | `crates/nvim/src/api/events.rs`               | `field:` reads the raw bytes `event`, `value` point at; the api value types keep nothing alive.                                    |
| `nvim_create_namespace`          | `crates/nvim/src/api/extmark/ns.rs`           | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_get_namespaces`            | `crates/nvim/src/api/extmark/ns.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim__ns_get`                   | `crates/nvim/src/api/extmark/ns.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `virt_text_to_array`             | `crates/nvim/src/api/extmark/query.rs`        | `field:` reads the raw bytes `vt` points at; the api value types keep nothing alive.                                               |
| `nvim_get_all_options_info`      | `crates/nvim/src/api/options.rs`              | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `set_mark`                       | `crates/nvim/src/api/private/helpers/mod.rs`  | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_tabpage_list_wins`         | `crates/nvim/src/api/tabpage.rs`              | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_tabpage_get_var`           | `crates/nvim/src/api/tabpage.rs`              | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_tabpage_set_var`           | `crates/nvim/src/api/tabpage.rs`              | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_tabpage_del_var`           | `crates/nvim/src/api/tabpage.rs`              | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_ui_set_option`             | `crates/nvim/src/api/ui/mod.rs`               | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_get_api_info`              | `crates/nvim/src/api/vim/client.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_set_client_info`           | `crates/nvim/src/api/vim/client.rs`           | `field:` reads the raw bytes `name`, `type_0` point at; the api value types keep nothing alive.                                    |
| `nvim_get_chan_info`             | `crates/nvim/src/api/vim/client.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_list_chans`                | `crates/nvim/src/api/vim/client.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_list_uis`                  | `crates/nvim/src/api/vim/client.rs`           | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_load_context`              | `crates/nvim/src/api/vim/context.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `status_named`                   | `crates/nvim/src/api/vim/echo.rs`             | `field:` reads the raw bytes `status` points at; the api value types keep nothing alive.                                           |
| `nvim_list_bufs`                 | `crates/nvim/src/api/vim/handles.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_list_wins`                 | `crates/nvim/src/api/vim/handles.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_list_tabpages`             | `crates/nvim/src/api/vim/handles.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_get_hl_id_by_name`         | `crates/nvim/src/api/vim/highlight.rs`        | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_get_color_by_name`         | `crates/nvim/src/api/vim/highlight.rs`        | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_get_color_map`             | `crates/nvim/src/api/vim/highlight.rs`        | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_feedkeys`                  | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `keys`, `mode` point at; the api value types keep nothing alive.                                      |
| `nvim_input`                     | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `keys` points at; the api value types keep nothing alive.                                             |
| `nvim_input_mouse`               | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `button`, `action`, `modifier` point at; the api value types keep nothing alive.                      |
| `nvim_replace_termcodes`         | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `str` points at; the api value types keep nothing alive.                                              |
| `nvim_get_keymap`                | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `mode` points at; the api value types keep nothing alive.                                             |
| `nvim_del_keymap`                | `crates/nvim/src/api/vim/input.rs`            | `field:` reads the raw bytes `mode`, `lhs` point at; the api value types keep nothing alive.                                       |
| `nvim__id`                       | `crates/nvim/src/api/vim/inspect.rs`          | `field:` reads the raw bytes `obj` points at; the api value types keep nothing alive.                                              |
| `nvim__id_array`                 | `crates/nvim/src/api/vim/inspect.rs`          | `field:` reads the raw bytes `arr` points at; the api value types keep nothing alive.                                              |
| `nvim__id_dict`                  | `crates/nvim/src/api/vim/inspect.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim__stats`                    | `crates/nvim/src/api/vim/inspect.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim__unpack`                   | `crates/nvim/src/api/vim/inspect.rs`          | `field:` reads the raw bytes `str` points at; the api value types keep nothing alive.                                              |
| `global_mark_name`               | `crates/nvim/src/api/vim/marks.rs`            | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `reject`                         | `crates/nvim/src/api/vim/marks.rs`            | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_del_mark`                  | `crates/nvim/src/api/vim/marks.rs`            | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_strwidth`                  | `crates/nvim/src/api/vim/runtime.rs`          | `field:` reads the raw bytes `text` points at; the api value types keep nothing alive.                                             |
| `nvim__runtime_inspect`          | `crates/nvim/src/api/vim/runtime.rs`          | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_set_current_dir`           | `crates/nvim/src/api/vim/runtime.rs`          | `field:` reads the raw bytes `dir` points at; the api value types keep nothing alive.                                              |
| `Context::of`                    | `crates/nvim/src/api/vim/statusline.rs`       | `field:` reads the raw bytes `opts`, `statuscol` point at; the api value types keep nothing alive.                                 |
| `statuscol_state`                | `crates/nvim/src/api/vim/statusline.rs`       | `field:` reads the raw bytes `statuscol` points at; the api value types keep nothing alive.                                        |
| `nvim_chan_send`                 | `crates/nvim/src/api/vim/term.rs`             | `field:` reads the raw bytes `data` points at; the api value types keep nothing alive.                                             |
| `nvim_get_current_line`          | `crates/nvim/src/api/vim/vars.rs`             | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_get_var`                   | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `key_not_found`                  | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_set_var`                   | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_del_var`                   | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_get_vvar`                  | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_set_vvar`                  | `crates/nvim/src/api/vim/vars.rs`             | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_eval`                      | `crates/nvim/src/api/vimscript/eval.rs`       | `field:` reads the raw bytes `expr` points at; the api value types keep nothing alive.                                             |
| `nvim_call_function`             | `crates/nvim/src/api/vimscript/eval.rs`       | `field:` reads the raw bytes `fn_0`, `args` point at; the api value types keep nothing alive.                                      |
| `nvim_call_dict_function`        | `crates/nvim/src/api/vimscript/eval.rs`       | `field:` reads the raw bytes `dict`, `fn_0`, `args` point at; the api value types keep nothing alive.                              |
| `nvim_command`                   | `crates/nvim/src/api/vimscript/exec.rs`       | `field:` reads the raw bytes `cmd` points at; the api value types keep nothing alive.                                              |
| `nvim_parse_expression`          | `crates/nvim/src/api/vimscript/expression.rs` | `field:` reads the raw bytes `expr`, `flags` point at; the api value types keep nothing alive.                                     |
| `parse_flags`                    | `crates/nvim/src/api/vimscript/expression.rs` | `field:` reads the raw bytes `flags` points at; the api value types keep nothing alive.                                            |
| `parse_border_item`              | `crates/nvim/src/api/win_config/border.rs`    | `field:` reads the raw bytes `item` points at; the api value types keep nothing alive.                                             |
| `cell_of`                        | `crates/nvim/src/api/win_config/border.rs`    | `field:` reads the raw bytes `string` points at; the api value types keep nothing alive.                                           |
| `parse_border_array`             | `crates/nvim/src/api/win_config/border.rs`    | `field:` reads the raw bytes `arr` points at; the api value types keep nothing alive.                                              |
| `border_array`                   | `crates/nvim/src/api/win_config/get.rs`       | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_win_get_config`            | `crates/nvim/src/api/win_config/get.rs`       | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `err_invalid_str`                | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `val` points at; the api value types keep nothing alive.                                              |
| `imatch`                         | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `s` points at; the api value types keep nothing alive.                                                |
| `parse_float_anchor`             | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `anchor` points at; the api value types keep nothing alive.                                           |
| `parse_float_relative`           | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `relative` points at; the api value types keep nothing alive.                                         |
| `parse_config_split`             | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `split` points at; the api value types keep nothing alive.                                            |
| `parse_float_bufpos`             | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `bufpos` points at; the api value types keep nothing alive.                                           |
| `parse_bordertext`               | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `bordertext` points at; the api value types keep nothing alive.                                       |
| `parse_bordertext_pos`           | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `bordertext_pos` points at; the api value types keep nothing alive.                                   |
| `smatch`                         | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `s` points at; the api value types keep nothing alive.                                                |
| `parse_win_config`               | `crates/nvim/src/api/win_config/parse.rs`     | `field:` reads the raw bytes `config` points at; the api value types keep nothing alive.                                           |
| `nvim_win_get_cursor`            | `crates/nvim/src/api/window.rs`               | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `nvim_win_set_cursor`            | `crates/nvim/src/api/window.rs`               | `field:` reads the raw bytes `pos` points at; the api value types keep nothing alive.                                              |
| `nvim_win_get_var`               | `crates/nvim/src/api/window.rs`               | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_win_set_var`               | `crates/nvim/src/api/window.rs`               | `field:` reads the raw bytes `name`, `value` point at; the api value types keep nothing alive.                                     |
| `nvim_win_del_var`               | `crates/nvim/src/api/window.rs`               | `field:` reads the raw bytes `name` points at; the api value types keep nothing alive.                                             |
| `nvim_win_get_position`          | `crates/nvim/src/api/window.rs`               | `alloc:` answers api values whose storage the caller frees.                                                                        |
| `Replacement::apply`             | `crates/nvim/src/api/buffer/text.rs`          | `field:` writes the `new_len` raw C strings the struct's `lines` array holds, which nothing owns.                                  |
| `Replacement::write_lines`       | `crates/nvim/src/api/buffer/text.rs`          | `field:` as `Replacement::apply`, whose second half it is.                                                                         |
| `run_cmd`                        | `crates/nvim/src/api/command/cmd.rs`          | `alloc:` answers the captured output as a `String_0` the caller frees.                                                             |
| `set_decor`                      | `crates/nvim/src/api/extmark/query.rs`        | `alloc:` hands the decoration's virtual text and sign strings to the mark tree, which takes them over.                             |
| `String_0::truncate`             | `crates/nvim/src/api/private/helpers/text.rs` | `cap:` cuts the length to `size` without checking it against the allocation.                                                       |
| `nvim_ui_attach`                 | `crates/nvim/src/api/ui/mod.rs`               | `field:` reads `options`' raw entries, which the `ApiDict` keeps nothing alive for.                                                |
| `ui_attach`                      | `crates/nvim/src/api/ui/mod.rs`               | `field:` as `nvim_ui_attach`, which it forwards to.                                                                                |
| `Utf8::char_at`                  | `crates/nvim/src/eval/encode/mod.rs`          | `cap:` indexes the buffer by `i` and then reads the continuation bytes the lead byte promises, neither of which is checked.        |
| `Utf8::len_at`                   | `crates/nvim/src/eval/encode/mod.rs`          | `cap:` as `Utf8::char_at`.                                                                                                         |
| `Utf8::run`                      | `crates/nvim/src/eval/encode/mod.rs`          | `cap:` takes `n` bytes from `i`, where `n` is a measured character length that may reach past the end.                             |
| `Utf8::tail`                     | `crates/nvim/src/eval/encode/mod.rs`          | `cap:` slices from `i`, which must be below the length.                                                                            |
| `json_escaped_len`               | `crates/nvim/src/eval/encode/mod.rs`          | `cap:` walks the string through `Utf8`, with the same unchecked index.                                                             |
| `parse_json_string`              | `crates/nvim/src/eval/decode/json/scan.rs`    | `cap:` `at` must index the decoder's buffer and stand on a `"`; neither is checked.                                                |
| `parse_json_number`              | `crates/nvim/src/eval/decode/json/scan.rs`    | `cap:` as `parse_json_string`, for a `-` or a digit.                                                                               |
| `Decoder::finish_value`          | `crates/nvim/src/eval/decode/json/stack.rs`   | `cap:` `at` must index the decoder's buffer, and `obj` owns the value it hands over.                                               |
| `map_to_dict`                    | `crates/nvim/src/eval/decode/msgpack.rs`      | `cap:` reads `len * 2` values out of `pairs`, whose slice length it does not consult.                                              |
| `TypVal::bit_copy`               | `crates/nvim/src/eval/typval/access.rs`       | `alloc:` duplicates the value's bits without touching its refcount, so exactly one of the two may ever be released.                |
| `DictSlot::clear`                | `crates/nvim/src/eval/typval/access.rs`       | `field:` releases through the raw slot the handle holds, which must still name a live entry.                                       |
| `tv_free`                        | `crates/nvim/src/eval/typval/value.rs`        | `alloc:` frees the typval itself, which has to be in an allocation of its own.                                                     |
| `ListRef::from_owned`            | `crates/nvim/src/eval/typval/list.rs`         | `alloc:` takes over a reference to the list the `NonNull` names — a pointer the needle does not match — and gives it back on drop. |
| `DictRef::from_owned`            | `crates/nvim/src/eval/typval/dict.rs`         | `alloc:` as `ListRef::from_owned`, for a dictionary.                                                                               |
| `dummy_ap`                       | `crates/nvim/src/eval/funcs/strings.rs`       | `ffi:` fabricates a `VaList<'static>` for the C formatter, which may only ever be read as a typval argument list.                  |
| `block_def2str`                  | `crates/nvim/src/eval/funcs/region.rs`        | `field:` reads the block's raw `textstart` cursor, which only a block-prep function leaves valid.                                  |
| `ProviderScope::leave`           | `crates/nvim/src/eval/funcs/channel.rs`       | `field:` pops the execution stack entry `enter` pushed; nothing may have touched it in between.                                    |
| `restore_funccal`                | `crates/nvim/src/eval/userfunc/funccall.rs`   | `field:` pops the `FuncCallEntry` a `save_funccal` pushed, which must still be live.                                               |
