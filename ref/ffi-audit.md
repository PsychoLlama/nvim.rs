# FFI Signature-Mismatch Audit (2026-06-08)

Workspace-wide audit triggered after the `tv_list_equal`/`tv_dict_equal`/`tv_blob_equal`
bug (commit 2fadb04b26): a Rust fn exported `-> bool` was re-declared in another
crate's `extern "C"` block as `-> c_int`. On x86-64, `bool` only sets the low byte
(`al`) of the return register; reading the full 32-bit `eax` picks up garbage in the
upper bytes, so `!= 0` checks spuriously succeed → crashes, wrong control flow, hangs.

ALL mismatches are Rust-to-Rust (one crate's extern block redeclaring another crate's
export). The C side (`src/nvim/*.c/*.h`) and the cbindgen-generated header are CLEAN.

## Class A — return: def `bool`, decl `c_int` (HIGH — exact bug class). 59 symbols / 136 sites.
Fix: change each listed decl from `-> c_int` to `-> bool`, and fix the call sites
(drop `!= 0`, adjust any `as` casts). Functions implicated in known clusters:
`text_locked`/`curbuf_locked` → Cluster D textlock/cmdwin hangs; `stuff_empty` →
feedkeys hangs; `aborting`/`should_abort`/`aborted_in_try` → exception control flow.

| Symbol | True def (-> bool) | Wrong `-> c_int` decls |
|---|---|---|
| aborting | ex_eval/src/lib.rs:308 | eval/funcs/misc.rs:2117; window/enter.rs:70; window/close/win_close.rs:92; funcall/dispatch.rs:193; eval_exec/errors.rs:470; eval_exec/lval.rs:215; eval_exec/eval.rs:293; eval_exec/eval_top.rs:133; insexpand/userfunc.rs:27; ex_cmds/lib.rs:507; ex_cmds/write.rs:862; ex_cmds/substitute.rs:151; ex_cmds/edit.rs:249; ex_cmds/shell.rs:1082; ex_cmds/buffer.rs:373; bufwrite/write.rs:149; bufwrite/autocmd.rs:81; buffer/lifecycle.rs:1380; buffer/close.rs:108 |
| aborted_in_try | ex_eval/src/lib.rs:298 | userfunc/lookup.rs:78 |
| augroup_exists | autocmd/src/group.rs:624 | fileio/operations.rs:878 |
| autocmd_supported | autocmd/src/lib.rs:975 | eval/funcs/misc.rs:2129 |
| bufIsChanged | undo/src/lib.rs:1102 | window/close/validation.rs:60; arglist/ffi.rs:390; ex_cmds/lib.rs:535; ex_cmds/write.rs:619; ex_docmd/cmd_impl.rs:816 |
| buf_hide | buffer/src/lib.rs:1088 | window/close/validation.rs:63 |
| buf_valid | buffer/src/lib.rs:1074 | cmdline/state.rs:1742 |
| check_abbr | mapping/src/eval.rs:305 | edit/helpers.rs:58; cmdline/edit.rs:67 |
| check_changed | ex_cmds2/src/autowrite_impl.rs:276 | arglist/ffi.rs:317 |
| curbufIsChanged | undo/src/lib.rs:1132 | change/recording.rs:33; bufwrite/autocmd.rs:79 |
| curbuf_locked | buffer/src/misc.rs:230 | ex_docmd/execute.rs:354; ex_docmd/do_one_cmd.rs:150 |
| inindent | indent/src/getters.rs:142 | edit/tab.rs:51; edit/state_machine.rs:169; edit/backspace.rs:79; ops/pending.rs:194; buffer/lifecycle.rs:1746 |
| is_aucmd_win | autocmd/src/lib.rs:461 | window/equalize.rs:94; window/list.rs:59; window/lib.rs:310; window/wrappers.rs:372; window/close/win_close.rs:39; window/close/orchestrate.rs:36; window/close/validation.rs:67; arglist/ffi.rs:402; ex_docmd/cmd_impl.rs:814; buffer/lifecycle.rs:1522 |
| match_file_list | fileio/src/pattern.rs:398 | bufwrite/write.rs:198 |
| menu_is_separator | menu/src/classify.rs:88 | cmdexpand/wildmenu.rs:226; cmdline/wildmenu.rs:169 |
| message_filtered | message/src/misc.rs:685 | userfunc/listing.rs:82 |
| messaging | message/src/misc.rs:556 | ex_cmds/lib.rs:505; ex_cmds/substitute.rs:165 |
| mf_need_trans | memfile/src/lib.rs:673 | memline/swap.rs:2163 |
| msg_add_fileformat | fileio/src/operations.rs:634 | fileio/readfile.rs:299 |
| nvim_win_get_config_external | window/src/win_struct.rs:3089 | window/events.rs:72; window/close/win_close.rs:32 |
| os_env_exists | os/src/env.rs:106 | tui/lib.rs:280 |
| os_fileinfo | os/src/fs.rs:1687 | fileio/rename.rs:31 |
| os_fileinfo_id_equal | os/src/fs.rs:1588 | fileio/rename.rs:32 |
| os_fileinfo_link | os/src/fs.rs:1726 | fileio/rename.rs:36 |
| os_isdir | os/src/fs.rs:73 | ex_cmds/lib.rs:543; ex_cmds/write.rs:730; filesearch/lib.rs:95; bufwrite/backup.rs:122 |
| os_path_exists | os/src/fs.rs:53 | memline/recovery.rs:81; ex_cmds/lib.rs:541; ex_cmds/write.rs:729; filesearch/lib.rs:96; fileio/rename.rs:37 |
| otherfile | buffer/src/lib.rs:1110 | ex_cmds/lib.rs:551; ex_cmds/write.rs:826; ex_docmd/commands.rs:257 |
| pum_drawn | popupmenu/src/lib.rs:251 | option/callbacks/mod.rs:81 |
| pum_visible | popupmenu/src/lib.rs:239 | move/lib.rs:787; insexpand/info.rs:95; insexpand/ctrl_x.rs:153; insexpand/ui.rs:14; insexpand/entry.rs:15; insexpand/viml.rs:12; insexpand/pum.rs:174 |
| qf_mark_adjust | quickfix/src/navigate.rs:2357 | mark/lib.rs:73 |
| rem_backslash | charset/src/lib.rs:1070 | cmdline/wildmenu.rs:168; arglist/ffi.rs:212 |
| rs_buf_hide | buffer/src/lib.rs:521 | arglist/ffi.rs:311 |
| rs_curbuf_reusable | buffer/src/lib.rs:598 | arglist/ffi.rs:351 |
| rs_eval_isnamec | eval/src/lib.rs:146 | userfunc/excmd.rs:164 |
| rs_eval_isnamec1 | eval/src/lib.rs:158 | userfunc/excmd.rs:165 |
| rs_is_luafunc | eval/src/lib.rs:339 | funcall/dispatch.rs:155 |
| rs_otherfile | buffer/src/lib.rs:862 | arglist/ffi.rs:315 |
| script_autoload | runtime/src/path.rs:741 | funcall/dispatch.rs:198 |
| set_indent | indent/src/set_indent.rs:77 | ex_cmds/lib.rs:470 |
| should_abort | ex_eval/src/lib.rs:315 | ex_cmds/lib.rs:603; ex_cmds/edit.rs:136; bufwrite/write.rs:326 |
| strequal | memutil/src/lib.rs:139 | popupmenu/item.rs:53 |
| stuff_empty | getchar/src/lib.rs:62 | cmdline/state.rs:1104; ex_docmd/commands.rs:2424; ex_docmd/cmd_impl.rs:935 |
| text_locked | ex_docmd/src/lib.rs:335 | ex_cmds/lib.rs:519; ex_cmds/write.rs:424 |
| tv2bool | typval/src/lib.rs:3893 | eval_exec/eval.rs:304 |
| vim_fgets | fileio/src/binary_io.rs:208 | search/path_search.rs:105; insexpand/dict.rs:392 |
| vim_isIDc | charset/src/lib.rs:649 | regexp/lib.rs:3310; indent_c/lib.rs:185 |
| vim_isfilec | charset/src/lib.rs:594 | regexp/lib.rs:3311; search/path_search.rs:101; filesearch/lib.rs:173 |
| vim_isprintc | charset/src/lib.rs:855 | regexp/lib.rs:3312; cmdline/state.rs:1725; message/format.rs:440; mark/lib.rs:71; ex_cmds/lib.rs:768; sign/text.rs:36; syntax/opt_parse.rs:47; option/callbacks/behavior.rs:212 |
| vim_iswordc | charset/src/lib.rs:713 | search/path_search.rs:99; indent_c/lib.rs:184 |
| vim_iswordc_buf | charset/src/lib.rs:695 | regexp/lib.rs:1572 |
| vim_iswordp | charset/src/lib.rs:783 | search/path_search.rs:100 |

### Class A, void-decl variant (LOW — bool return discarded, but signature still wrong)
`do_mouse`, `garbage_collect`, `set_leftcol`, `prepare_tagpreview`, `rs_marktree_itr_get`,
`rs_marktree_itr_next`, `win_check_ns_hl`, `os_setenv_append_path` — decl `()` for a `-> bool` def.

## arena_alloc (HIGH — missing param + wrong return)
Def: `(arena: *mut Arena, size: usize, align: bool) -> *mut c_void` @ memory/src/lib.rs:182.
Wrong decls `(arena, size) -> *mut c_char` (missing `align`, wrong return) at:
api/lib.rs:1159; highlight/lib.rs:6296; mapping/query.rs:99; message/keys.rs:171;
runtime/searchpath.rs:128; unpacker/lib.rs:41.

## Residual param mismatch in the already-"fixed" three (MEDIUM)
eval_exec/src/operators.rs:126 and :130 still declare the `ic` param as `c_int`, but
true defs use `ic: bool` (typval/src/lib.rs:3808, :6219). `pattern_match` at
operators.rs:140 has the same `bool`-param-as-`c_int` issue.

## Class D — param: def `bool`, decl `c_int` (MEDIUM, ~40 sites)
Examples: del_char, del_bytes, changed_lines, get_leader_len, gotocmdline,
set_no_hlsearch, getdigits, modname, autowrite, can_abandon. Re-derive full list with
the audit method below before fixing.

## Class B/E (MEDIUM, lower priority)
B: def `c_int`, decl `bool` (~30 symbols / 154 sites) — only low byte read, safe for 0/1.
E: def `c_int` param, decl `bool` param (~130 sites).

## Audit method (to re-derive / verify)
- `rg -n '#\[no_mangle\]|#\[export_name' src/nvim-rs -A3` → true signatures.
- For each exported bool fn `foo`: `rg -n '\bfoo\b' src/nvim-rs` and inspect `extern "C"` decls.
- Width mismatches (type-alias / opaque-pointer) are benign false positives; ignore.
- Param-COUNT raw hits are mostly parser artifacts (fn-ptr/generic `>` splits) — verify per-site; only arena_alloc was real.
