use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::arglist::get_arglist_exp;
use crate::src::nvim::autocmd::{
    au_event_disable, au_event_restore, block_autocmds, unblock_autocmds,
};
use crate::src::nvim::buffer::{buflist_getfile, do_modelines, no_write_message};
use crate::src::nvim::charset::{skipdigits, skipwhite, vim_isprintc};
use crate::src::nvim::drawscreen::{redraw_curbuf_later, update_screen};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::typval::{
    callback_copy, callback_free, callback_put, tv_clear, tv_copy, tv_dict_add, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_add_tv, tv_dict_alloc, tv_dict_alloc_lock,
    tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number, tv_dict_get_string,
    tv_dict_get_tv, tv_dict_item_alloc_len, tv_dict_item_free, tv_dict_unref, tv_free,
    tv_get_number_chk, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
};
use crate::src::nvim::eval::vars::set_internal_string_var;
use crate::src::nvim::eval_1::{
    callback_call, callback_from_typval, eval_expr, set_ref_in_callback, set_ref_in_item,
};
use crate::src::nvim::ex_cmds::{append_redir, check_secure, do_shell, skip_vimgrep_pat};
use crate::src::nvim::ex_cmds2::autowrite_all;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, ex_cd, is_loclist_cmd};
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::ex_getln::get_list_range;
use crate::src::nvim::fileio::{readfile, shorten_fnames, vim_fgets, vim_tempname};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::fuzzy::fuzzy_match;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::check_help_lang;
use crate::src::nvim::highlight_group::syn_name2id;
use crate::src::nvim::main::{
    cmdline_row, cmdmod, e_au_recursive, e_buffer_is_not_loaded, e_dictreq, e_invalpat, e_invarg,
    e_invarg2, e_invrange, e_listreq, e_loclist, e_no_errors, e_nomatch, e_nomatch2, e_noprevre,
    e_notmp, e_openerrf, e_readerrf, e_string_required, e_trailing_arg,
    e_winfixbuf_cannot_go_to_buffer, empty_string_option, fdo_flags, got_int, msg_col, msg_didout,
    msg_nowait, msg_scroll, msg_scrolled, must_redraw, p_ch, p_chi, p_cpo, p_ef, p_efm, p_enc,
    p_gefm, p_gp, p_hh, p_ic, p_mef, p_menc, p_mls, p_qftf, p_rtp, p_shq, p_sp, p_swb,
    restart_edit, swb_flags, textlock, Columns, IObuff, KeyTyped, NameBuff,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{convert_setup, remove_bom, string_convert};
use crate::src::nvim::memline::{check_need_swap, ml_delete};
use crate::src::nvim::memory::{
    strequal, xcalloc, xfree, xmalloc, xmallocz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, internal_error, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_keep,
    msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl, msg_start, msg_strtrunc, semsg,
    smsg, trunc_string,
};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::{
    copy_option_part, option_set_callback_func, set_option_direct, set_option_value_give_err,
    shortmess, skip_to_option_part,
};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::env::{expand_env, os_get_pid};
use crate::src::nvim::os::fs::{
    os_dirname, os_fileinfo_link, os_fopen, os_isdir, os_open_stdin_fd, os_path_exists, os_remove,
};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, abort, abs, atoi, atol, fclose, fdopen, ferror, fgets,
    gettext, memcpy, memset, snprintf, strcat, strcmp, strcpy, strlen, strncasecmp, time,
};
use crate::src::nvim::path::{
    add_pathsep, concat_fnames, fix_fname, gen_expand_wildcards, path_fnamecmp, path_is_absolute,
    path_tail, path_try_shorten_fname, vim_isAbsName, FreeWild,
};
use crate::src::nvim::search::{do_search, last_search_pat};
use crate::src::nvim::strings::{has_non_ascii, vim_snprintf, vim_snprintf_safelen, vim_strchr};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_6,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_3, Dict, Direction, Error, ErrorType,
    EvalFuncData, ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID,
    FileInfo, Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyValuePair,
    LineGetter, ListLenSpecials, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType,
    MsgpackRpcRequestHandler, Object, ObjectType, OptIndex, OptInt, OptVal, OptValData, OptValType,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_14, String_0,
    Terminal, Timestamp, TriState, UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t,
    _IO_marker, _IO_wide_data, __off64_t, __off_t, __time_t, alist_T, auto_event, bcount_t, bhdr_T,
    bln_values, blob_T, blobvar_S, blocknr_T, bufstate_T, chunksize_T, cleanup_T, cleanup_stuff,
    cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_15,
    dict_T, dictitem_T, dictvar_S, diff_T, diffblock_S, disptick_T, dobuf_action_values, eslist_T,
    eslist_elem, event_T, exarg, exarg_T, except_T, except_type_T, extmark_undo_vec_t, fcs_chars_T,
    float_T, fmark_T, fmarkv_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T,
    garray_T, getf_values, handle_T, hash_T, hashitem_T, hashtab_T, ht_stack_S, ht_stack_T,
    iconv_t, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T, linenr_T, list_T,
    list_stack_S, list_stack_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, memfile_T, memline_T, mfdirty_T, msglist, msglist_T,
    mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed_0, oparg_T, optset_T,
    partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, queue, reg_extmatch_T,
    regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, searchit_arg_T,
    size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_5, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_10,
    u_header_uh_alt_prev as C2Rust_Unnamed_9, u_header_uh_next as C2Rust_Unnamed_12,
    u_header_uh_prev as C2Rust_Unnamed_11, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_8, uv_stat_t, uv_timespec_t, varnumber_T,
    vim_exception, vimconv_T, virt_line, visualinfo_T, winopt_T, wline_T, xfmark_T, FILE, QUEUE,
    _IO_FILE,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, check_lnums, tabline_height, win_setheight, win_setwidth,
    win_split,
};
// Phase-5a blacklist residue: this module keeps concrete local copies of
// types whose canonical form is opaque (file_buffer, window_S, ...), so
// these declarations cannot become `use` imports until the phase-8 rewrite.
// The copies are layout-identical to the canonical definitions (proven by
// the 5a parity suite); the nominal decl/decl mismatch is expected.
#[allow(clashing_extern_declarations)]
extern "C" {
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buf_valid(buf: *mut buf_T) -> bool;
    fn close_buffer(
        win: *mut win_T,
        buf: *mut buf_T,
        action: ::core::ffi::c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn buflist_findname_exp(fname: *mut ::core::ffi::c_char) -> *mut buf_T;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn setfname(
        buf: *mut buf_T,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        message: bool,
    ) -> ::core::ffi::c_int;
    fn bt_help(buf: *const buf_T) -> bool;
    fn bt_normal(buf: *const buf_T) -> bool;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn wipe_buffer(buf: *mut buf_T, aucmd: bool);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn check_cursor(wp: *mut win_T);
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn win_id2wp(id: ::core::ffi::c_int) -> *mut win_T;
    fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T;
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn can_abandon(buf: *mut buf_T, forceit: bool) -> bool;
    fn extmark_splice(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        old_row: ::core::ffi::c_int,
        old_col: colnr_T,
        old_byte: bcount_t,
        new_row: ::core::ffi::c_int,
        new_col: colnr_T,
        new_byte: bcount_t,
        undo: ExtmarkOp,
    );
    fn foldUpdateAll(win: *mut win_T);
    fn shorten_buf_fname(
        buf: *mut buf_T,
        dirname: *mut ::core::ffi::c_char,
        force: ::core::ffi::c_int,
    );
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    static firstwin: GlobalCell<*mut win_T>;
    static lastwin: GlobalCell<*mut win_T>;
    static prevwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    fn ml_open(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn get_region_bytecount(
        buf: *mut buf_T,
        start_lnum: linenr_T,
        end_lnum: linenr_T,
        start_col: colnr_T,
        end_col: colnr_T,
    ) -> bcount_t;
    fn buf_copy_options(buf: *mut buf_T, flags: ::core::ffi::c_int);
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn vim_regexec_multi(
        rmp: *mut regmmatch_T,
        win: *mut win_T,
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        tm: *mut proftime_T,
        timed_out: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn u_clearallandblockfree(buf: *mut buf_T);
    fn win_valid(win: *const win_T) -> bool;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn goto_tabpage_win(tp: *mut tabpage_T, wp: *mut win_T);
    fn win_goto(wp: *mut win_T);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed = 2147483647;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct file_buffer {
    pub handle: handle_T,
    pub b_ml: memline_T,
    pub b_next: *mut buf_T,
    pub b_prev: *mut buf_T,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: ::core::ffi::c_int,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: varnumber_T,
    pub b_last_changedtick_i: varnumber_T,
    pub b_last_changedtick_pum: varnumber_T,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: linenr_T,
    pub b_mod_bot: linenr_T,
    pub b_mod_xlines: linenr_T,
    pub b_wininfo: C2Rust_Unnamed_13,
    pub b_mod_tick_syn: disptick_T,
    pub b_mod_tick_decor: disptick_T,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [fmark_T; 26],
    pub b_visual: visualinfo_T,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: fmark_T,
    pub b_last_insert: fmark_T,
    pub b_last_change: fmark_T,
    pub b_changelist: [fmark_T; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut mapblock_T; 256],
    pub b_first_abbr: *mut mapblock_T,
    pub b_ucmds: garray_T,
    pub b_op_start: pos_T,
    pub b_op_start_orig: pos_T,
    pub b_op_end: pos_T,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    pub b_u_oldhead: *mut u_header_T,
    pub b_u_newhead: *mut u_header_T,
    pub b_u_curhead: *mut u_header_T,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: linenr_T,
    pub b_u_line_colnr: colnr_T,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub b_kmap_ga: garray_T,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [sctx_T; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut colnr_T,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut colnr_T,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: linenr_T,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut dict_T,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: fmark_T,
    pub b_s: synblock_T,
    pub b_signcols: C2Rust_Unnamed_4,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_2,
    pub update_callbacks: C2Rust_Unnamed_1,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_4 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub type WinInfo = wininfo_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wininfo_S {
    pub wi_win: *mut win_T,
    pub wi_mark: fmark_T,
    pub wi_optset: bool,
    pub wi_opt: winopt_T,
    pub wi_fold_manual: bool,
    pub wi_folds: garray_T,
    pub wi_changelistidx: ::core::ffi::c_int,
}
pub type win_T = window_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_S {
    pub handle: handle_T,
    pub w_buffer: *mut buf_T,
    pub w_s: *mut synblock_T,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    pub w_ns_set: Set_uint32_t,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    pub w_hl_needs_update: ::core::ffi::c_int,
    pub w_prev: *mut win_T,
    pub w_next: *mut win_T,
    pub w_locked: bool,
    pub w_frame: *mut frame_T,
    pub w_cursor: pos_T,
    pub w_curswant: colnr_T,
    pub w_set_curswant: ::core::ffi::c_int,
    pub w_cursorline: linenr_T,
    pub w_last_cursorline: linenr_T,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: linenr_T,
    pub w_old_cursor_fcol: colnr_T,
    pub w_old_cursor_lcol: colnr_T,
    pub w_old_visual_lnum: linenr_T,
    pub w_old_visual_col: colnr_T,
    pub w_old_curswant: colnr_T,
    pub w_last_cursor_lnum_rnu: linenr_T,
    pub w_p_lcs_chars: lcs_chars_T,
    pub w_p_fcs_chars: fcs_chars_T,
    pub w_topline: linenr_T,
    pub w_topline_was_set: ::core::ffi::c_char,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: colnr_T,
    pub w_skipcol: colnr_T,
    pub w_last_topline: linenr_T,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: colnr_T,
    pub w_last_skipcol: colnr_T,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: pos_save_T,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: ::core::ffi::c_int,
    pub w_valid_cursor: pos_T,
    pub w_valid_leftcol: colnr_T,
    pub w_valid_skipcol: colnr_T,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: linenr_T,
    pub w_viewport_last_botline: linenr_T,
    pub w_viewport_last_topfill: linenr_T,
    pub w_viewport_last_skipcol: linenr_T,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: colnr_T,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: linenr_T,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut wline_T,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: garray_T,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: linenr_T,
    pub w_redraw_bot: linenr_T,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: disptick_T,
    pub w_stl_cursor: pos_T,
    pub w_stl_virtcol: colnr_T,
    pub w_stl_topline: linenr_T,
    pub w_stl_line_count: linenr_T,
    pub w_stl_topfill: ::core::ffi::c_int,
    pub w_stl_empty: ::core::ffi::c_char,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: pos_T,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut alist_T,
    pub w_arg_idx: ::core::ffi::c_int,
    pub w_arg_idx_invalid: ::core::ffi::c_int,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: winopt_T,
    pub w_allbuf_opt: winopt_T,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut dict_T,
    pub w_pcmark: pos_T,
    pub w_prev_pcmark: pos_T,
    pub w_jumplist: [xfmark_T; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut matchitem_T,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [taggy_T; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: linenr_T,
    pub w_statuscol_line_count: linenr_T,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut qf_info_T,
    pub w_llist_ref: *mut qf_info_T,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
pub const kStlClickFuncRun: C2Rust_Unnamed_14 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_14 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_14 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_14 = 0;
pub type qf_info_T = qf_info_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qf_info_S {
    pub qf_refcount: ::core::ffi::c_int,
    pub qf_listcount: ::core::ffi::c_int,
    pub qf_curlist: ::core::ffi::c_int,
    pub qf_maxcount: ::core::ffi::c_int,
    pub qf_lists: *mut qf_list_T,
    pub qfl_type: qfltype_T,
    pub qf_bufnr: ::core::ffi::c_int,
}
pub type qfltype_T = ::core::ffi::c_uint;
pub const QFLT_INTERNAL: qfltype_T = 2;
pub const QFLT_LOCATION: qfltype_T = 1;
pub const QFLT_QUICKFIX: qfltype_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qf_list_T {
    pub qf_id: ::core::ffi::c_uint,
    pub qfl_type: qfltype_T,
    pub qf_start: *mut qfline_T,
    pub qf_last: *mut qfline_T,
    pub qf_ptr: *mut qfline_T,
    pub qf_count: ::core::ffi::c_int,
    pub qf_index: ::core::ffi::c_int,
    pub qf_nonevalid: bool,
    pub qf_has_user_data: bool,
    pub qf_title: *mut ::core::ffi::c_char,
    pub qf_ctx: *mut typval_T,
    pub qf_qftf_cb: Callback,
    pub qf_dir_stack: *mut dir_stack_T,
    pub qf_directory: *mut ::core::ffi::c_char,
    pub qf_file_stack: *mut dir_stack_T,
    pub qf_currfile: *mut ::core::ffi::c_char,
    pub qf_multiline: bool,
    pub qf_multiignore: bool,
    pub qf_multiscan: bool,
    pub qf_changedtick: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dir_stack_T {
    pub next: *mut dir_stack_T,
    pub dirname: *mut ::core::ffi::c_char,
}
pub type qfline_T = qfline_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qfline_S {
    pub qf_next: *mut qfline_T,
    pub qf_prev: *mut qfline_T,
    pub qf_lnum: linenr_T,
    pub qf_end_lnum: linenr_T,
    pub qf_fnum: ::core::ffi::c_int,
    pub qf_col: ::core::ffi::c_int,
    pub qf_end_col: ::core::ffi::c_int,
    pub qf_nr: ::core::ffi::c_int,
    pub qf_module: *mut ::core::ffi::c_char,
    pub qf_fname: *mut ::core::ffi::c_char,
    pub qf_pattern: *mut ::core::ffi::c_char,
    pub qf_text: *mut ::core::ffi::c_char,
    pub qf_viscol: ::core::ffi::c_char,
    pub qf_cleared: ::core::ffi::c_char,
    pub qf_type: ::core::ffi::c_char,
    pub qf_user_data: typval_T,
    pub qf_valid: ::core::ffi::c_char,
}
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type matchitem_T = matchitem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchitem {
    pub mit_next: *mut matchitem_T,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: regmmatch_T,
    pub mit_pos_array: *mut llpos_T,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: linenr_T,
    pub mit_botlnum: linenr_T,
    pub mit_hl: match_T,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_T {
    pub rm: regmmatch_T,
    pub buf: *mut buf_T,
    pub lnum: linenr_T,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: linenr_T,
    pub startcol: colnr_T,
    pub endcol: colnr_T,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: proftime_T,
}
pub type buf_T = file_buffer;
pub type frame_T = frame_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct frame_S {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut frame_T,
    pub fr_next: *mut frame_T,
    pub fr_prev: *mut frame_T,
    pub fr_child: *mut frame_T,
    pub fr_win: *mut win_T,
}
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_16 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_16 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_16 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_16 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_16 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_16 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_16 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_16 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_16 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_16 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_16 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_16 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_16 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_17 = 76;
pub const HLF_PRE: C2Rust_Unnamed_17 = 75;
pub const HLF_OK: C2Rust_Unnamed_17 = 74;
pub const HLF_SO: C2Rust_Unnamed_17 = 73;
pub const HLF_SE: C2Rust_Unnamed_17 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_17 = 71;
pub const HLF_TS: C2Rust_Unnamed_17 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_17 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_17 = 68;
pub const HLF_CU: C2Rust_Unnamed_17 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_17 = 66;
pub const HLF_WBR: C2Rust_Unnamed_17 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_17 = 64;
pub const HLF_MSG: C2Rust_Unnamed_17 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_17 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_17 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_17 = 60;
pub const HLF_0: C2Rust_Unnamed_17 = 59;
pub const HLF_QFL: C2Rust_Unnamed_17 = 58;
pub const HLF_MC: C2Rust_Unnamed_17 = 57;
pub const HLF_CUL: C2Rust_Unnamed_17 = 56;
pub const HLF_CUC: C2Rust_Unnamed_17 = 55;
pub const HLF_TPF: C2Rust_Unnamed_17 = 54;
pub const HLF_TPS: C2Rust_Unnamed_17 = 53;
pub const HLF_TP: C2Rust_Unnamed_17 = 52;
pub const HLF_PBR: C2Rust_Unnamed_17 = 51;
pub const HLF_PST: C2Rust_Unnamed_17 = 50;
pub const HLF_PSB: C2Rust_Unnamed_17 = 49;
pub const HLF_PSX: C2Rust_Unnamed_17 = 48;
pub const HLF_PNX: C2Rust_Unnamed_17 = 47;
pub const HLF_PSK: C2Rust_Unnamed_17 = 46;
pub const HLF_PNK: C2Rust_Unnamed_17 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_17 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_17 = 43;
pub const HLF_PSI: C2Rust_Unnamed_17 = 42;
pub const HLF_PNI: C2Rust_Unnamed_17 = 41;
pub const HLF_SPL: C2Rust_Unnamed_17 = 40;
pub const HLF_SPR: C2Rust_Unnamed_17 = 39;
pub const HLF_SPC: C2Rust_Unnamed_17 = 38;
pub const HLF_SPB: C2Rust_Unnamed_17 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_17 = 36;
pub const HLF_SC: C2Rust_Unnamed_17 = 35;
pub const HLF_TXA: C2Rust_Unnamed_17 = 34;
pub const HLF_TXD: C2Rust_Unnamed_17 = 33;
pub const HLF_DED: C2Rust_Unnamed_17 = 32;
pub const HLF_CHD: C2Rust_Unnamed_17 = 31;
pub const HLF_ADD: C2Rust_Unnamed_17 = 30;
pub const HLF_FC: C2Rust_Unnamed_17 = 29;
pub const HLF_FL: C2Rust_Unnamed_17 = 28;
pub const HLF_WM: C2Rust_Unnamed_17 = 27;
pub const HLF_W: C2Rust_Unnamed_17 = 26;
pub const HLF_VNC: C2Rust_Unnamed_17 = 25;
pub const HLF_V: C2Rust_Unnamed_17 = 24;
pub const HLF_T: C2Rust_Unnamed_17 = 23;
pub const HLF_VSP: C2Rust_Unnamed_17 = 22;
pub const HLF_C: C2Rust_Unnamed_17 = 21;
pub const HLF_SNC: C2Rust_Unnamed_17 = 20;
pub const HLF_S: C2Rust_Unnamed_17 = 19;
pub const HLF_R: C2Rust_Unnamed_17 = 18;
pub const HLF_CLF: C2Rust_Unnamed_17 = 17;
pub const HLF_CLS: C2Rust_Unnamed_17 = 16;
pub const HLF_CLN: C2Rust_Unnamed_17 = 15;
pub const HLF_LNB: C2Rust_Unnamed_17 = 14;
pub const HLF_LNA: C2Rust_Unnamed_17 = 13;
pub const HLF_N: C2Rust_Unnamed_17 = 12;
pub const HLF_CM: C2Rust_Unnamed_17 = 11;
pub const HLF_M: C2Rust_Unnamed_17 = 10;
pub const HLF_LC: C2Rust_Unnamed_17 = 9;
pub const HLF_L: C2Rust_Unnamed_17 = 8;
pub const HLF_I: C2Rust_Unnamed_17 = 7;
pub const HLF_E: C2Rust_Unnamed_17 = 6;
pub const HLF_D: C2Rust_Unnamed_17 = 5;
pub const HLF_AT: C2Rust_Unnamed_17 = 4;
pub const HLF_TERM: C2Rust_Unnamed_17 = 3;
pub const HLF_EOB: C2Rust_Unnamed_17 = 2;
pub const HLF_8: C2Rust_Unnamed_17 = 1;
pub const HLF_NONE: C2Rust_Unnamed_17 = 0;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tabpage_S {
    pub handle: handle_T,
    pub tp_next: *mut tabpage_T,
    pub tp_topframe: *mut frame_T,
    pub tp_curwin: *mut win_T,
    pub tp_prevwin: *mut win_T,
    pub tp_firstwin: *mut win_T,
    pub tp_lastwin: *mut win_T,
    pub tp_old_Rows_avail: int64_t,
    pub tp_old_Columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut diff_T,
    pub tp_diffbuf: [*mut buf_T; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut frame_T; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut dict_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
pub type tabpage_T = tabpage_S;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_18 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_18 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_18 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_18 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_18 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_18 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_18 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_18 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_18 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_18 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptSwbFlagUselast: C2Rust_Unnamed_19 = 32;
pub const kOptSwbFlagVsplit: C2Rust_Unnamed_19 = 16;
pub const kOptSwbFlagNewtab: C2Rust_Unnamed_19 = 8;
pub const kOptSwbFlagSplit: C2Rust_Unnamed_19 = 4;
pub const kOptSwbFlagUsetab: C2Rust_Unnamed_19 = 2;
pub const kOptSwbFlagUseopen: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_20 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_20 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_20 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_20 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_20 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_20 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_20 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_20 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_20 = 79;
pub const SHM_OVER: C2Rust_Unnamed_20 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_20 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_20 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_20 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_20 = 97;
pub const SHM_WRI: C2Rust_Unnamed_20 = 119;
pub const SHM_LINES: C2Rust_Unnamed_20 = 108;
pub const SHM_MOD: C2Rust_Unnamed_20 = 109;
pub const SHM_RO: C2Rust_Unnamed_20 = 114;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_21 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_21 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_21 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_21 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_21 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_21 = 20;
pub const UPD_VALID: C2Rust_Unnamed_21 = 10;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_22 = 4;
pub const BL_SOL: C2Rust_Unnamed_22 = 2;
pub const BL_WHITE: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_23 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_23 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_23 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_23 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_23 = 1;
pub const CONV_NONE: C2Rust_Unnamed_23 = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_24 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_24 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_24 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_24 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_24 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_24 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_25 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_25 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_25 = 0;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_26 = 256;
pub const READ_NOWINENTER: C2Rust_Unnamed_26 = 128;
pub const READ_FIFO: C2Rust_Unnamed_26 = 64;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_26 = 32;
pub const READ_DUMMY: C2Rust_Unnamed_26 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_26 = 8;
pub const READ_STDIN: C2Rust_Unnamed_26 = 4;
pub const READ_FILTER: C2Rust_Unnamed_26 = 2;
pub const READ_NEW: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FUZZY_MATCH_MAX_LEN: C2Rust_Unnamed_27 = 1024;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_28 = 4;
pub const BCO_ALWAYS: C2Rust_Unnamed_28 = 2;
pub const BCO_ENTER: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_29 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_29 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_29 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_29 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_29 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_29 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_29 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_30 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_30 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_30 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_30 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_30 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_30 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_30 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_30 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_30 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_30 = 512;
pub const EW_ICASE: C2Rust_Unnamed_30 = 256;
pub const EW_PATH: C2Rust_Unnamed_30 = 128;
pub const EW_EXEC: C2Rust_Unnamed_30 = 64;
pub const EW_SILENT: C2Rust_Unnamed_30 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_30 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_30 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_30 = 4;
pub const EW_FILE: C2Rust_Unnamed_30 = 2;
pub const EW_DIR: C2Rust_Unnamed_30 = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const VGR_FUZZY: C2Rust_Unnamed_31 = 4;
pub const VGR_NOJUMP: C2Rust_Unnamed_31 = 2;
pub const VGR_GLOBAL: C2Rust_Unnamed_31 = 1;
pub type qf_delq_T = qf_delq_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qf_delq_S {
    pub next: *mut qf_delq_S,
    pub qi: *mut qf_info_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qffields_T {
    pub namebuf: *mut ::core::ffi::c_char,
    pub bnr: ::core::ffi::c_int,
    pub module: *mut ::core::ffi::c_char,
    pub errmsg: *mut ::core::ffi::c_char,
    pub errmsglen: size_t,
    pub lnum: linenr_T,
    pub end_lnum: linenr_T,
    pub col: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
    pub use_viscol: bool,
    pub pattern: *mut ::core::ffi::c_char,
    pub enr: ::core::ffi::c_int,
    pub type_0: ::core::ffi::c_char,
    pub user_data: *mut typval_T,
    pub valid: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qfstate_T {
    pub linebuf: *mut ::core::ffi::c_char,
    pub linelen: size_t,
    pub growbuf: *mut ::core::ffi::c_char,
    pub growbufsiz: size_t,
    pub fd: *mut FILE,
    pub tv: *mut typval_T,
    pub p_str: *mut ::core::ffi::c_char,
    pub p_list: *mut list_T,
    pub p_li: *mut listitem_T,
    pub buf: *mut buf_T,
    pub buflnum: linenr_T,
    pub lnumlast: linenr_T,
    pub vc: vimconv_T,
}
pub const QF_FAIL: C2Rust_Unnamed_34 = 0;
pub type efm_T = efm_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct efm_S {
    pub prog: *mut regprog_T,
    pub next: *mut efm_T,
    pub addr: [::core::ffi::c_char; 14],
    pub prefix: ::core::ffi::c_char,
    pub flags: ::core::ffi::c_char,
    pub conthere: ::core::ffi::c_int,
}
pub const QF_OK: C2Rust_Unnamed_34 = 1;
pub const QF_END_OF_INPUT: C2Rust_Unnamed_34 = 2;
pub const QF_IGNORE_LINE: C2Rust_Unnamed_34 = 4;
pub const QF_MULTISCAN: C2Rust_Unnamed_34 = 5;
pub const QF_NOMEM: C2Rust_Unnamed_34 = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmtpattern {
    pub convchar: ::core::ffi::c_char,
    pub pattern: *mut ::core::ffi::c_char,
}
pub const QF_ABORT: C2Rust_Unnamed_34 = 6;
pub const WSP_ABOVE: C2Rust_Unnamed_33 = 128;
pub const WSP_NEWLOC: C2Rust_Unnamed_33 = 256;
pub const WSP_HELP: C2Rust_Unnamed_33 = 32;
pub const WSP_TOP: C2Rust_Unnamed_33 = 8;
pub const SEARCH_KEEP: C2Rust_Unnamed_32 = 1024;
pub const WSP_QUICKFIX: C2Rust_Unnamed_33 = 1024;
pub const WSP_BELOW: C2Rust_Unnamed_33 = 64;
pub const WSP_BOT: C2Rust_Unnamed_33 = 16;
pub const WSP_VERT: C2Rust_Unnamed_33 = 2;
pub const QF_WINHEIGHT: C2Rust_Unnamed_35 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vgr_args_T {
    pub tomatch: ::core::ffi::c_int,
    pub spat: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub fnames: *mut *mut ::core::ffi::c_char,
    pub fcount: ::core::ffi::c_int,
    pub regmatch: regmmatch_T,
    pub qf_title: *mut ::core::ffi::c_char,
}
pub const QF_GETLIST_QFTF: C2Rust_Unnamed_36 = 2048;
pub const QF_GETLIST_NONE: C2Rust_Unnamed_36 = 0;
pub const QF_GETLIST_QFBUFNR: C2Rust_Unnamed_36 = 1024;
pub const QF_GETLIST_FILEWINID: C2Rust_Unnamed_36 = 512;
pub const QF_GETLIST_TICK: C2Rust_Unnamed_36 = 256;
pub const QF_GETLIST_SIZE: C2Rust_Unnamed_36 = 128;
pub const QF_GETLIST_IDX: C2Rust_Unnamed_36 = 64;
pub const QF_GETLIST_ITEMS: C2Rust_Unnamed_36 = 2;
pub const QF_GETLIST_ID: C2Rust_Unnamed_36 = 32;
pub const QF_GETLIST_CONTEXT: C2Rust_Unnamed_36 = 16;
pub const QF_GETLIST_WINID: C2Rust_Unnamed_36 = 8;
pub const QF_GETLIST_NR: C2Rust_Unnamed_36 = 4;
pub const QF_GETLIST_TITLE: C2Rust_Unnamed_36 = 1;
pub const QF_GETLIST_ALL: C2Rust_Unnamed_36 = 4095;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_32 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_32 = 2048;
pub const SEARCH_MARK: C2Rust_Unnamed_32 = 512;
pub const SEARCH_START: C2Rust_Unnamed_32 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_32 = 128;
pub const SEARCH_END: C2Rust_Unnamed_32 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_32 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_32 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_32 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_32 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_32 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const WSP_NOENTER: C2Rust_Unnamed_33 = 512;
pub const WSP_HOR: C2Rust_Unnamed_33 = 4;
pub const WSP_ROOM: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INVALID_QFIDX: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const INVALID_QFBUFNR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static ql_info_actual: GlobalCell<qf_info_T> = GlobalCell::new(qf_info_T {
    qf_refcount: 0,
    qf_listcount: 0,
    qf_curlist: 0,
    qf_maxcount: 0,
    qf_lists: ::core::ptr::null_mut::<qf_list_T>(),
    qfl_type: QFLT_QUICKFIX,
    qf_bufnr: 0,
});
static ql_info: GlobalCell<*mut qf_info_T> = GlobalCell::new(::core::ptr::null_mut::<qf_info_T>());
static last_qf_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0 as ::core::ffi::c_uint);
pub const FMT_PATTERNS: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
static e_no_more_items: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E553: No more items\0".as_ptr() as *const ::core::ffi::c_char);
static e_current_quickfix_list_was_changed: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(
        b"E925: Current quickfix list was changed\0".as_ptr() as *const ::core::ffi::c_char
    );
static e_current_location_list_was_changed: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(
        b"E926: Current location list was changed\0".as_ptr() as *const ::core::ffi::c_char
    );
static qf_last_bufname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static qf_last_bufref: GlobalCell<bufref_T> = GlobalCell::new(bufref_T {
    br_buf: ::core::ptr::null_mut::<buf_T>(),
    br_fnum: 0 as ::core::ffi::c_int,
    br_buf_free_count: 0 as ::core::ffi::c_int,
});
static qfga: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
});
unsafe extern "C" fn qfga_get() -> *mut garray_T {
    static initialized: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if !initialized.get() {
        initialized.set(true_0 != 0);
        ga_init(
            qfga.ptr(),
            1 as ::core::ffi::c_int,
            256 as ::core::ffi::c_int,
        );
    }
    (*qfga.ptr()).ga_len = 0 as ::core::ffi::c_int;
    return qfga.ptr();
}
unsafe extern "C" fn qfga_clear() {
    if (*qfga.ptr()).ga_maxlen > 1000 as ::core::ffi::c_int {
        ga_clear(qfga.ptr());
    } else {
        (*qfga.ptr()).ga_len = 0 as ::core::ffi::c_int;
    };
}
static quickfix_busy: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static qf_delq_head: GlobalCell<*mut qf_delq_T> =
    GlobalCell::new(::core::ptr::null_mut::<qf_delq_T>());
unsafe extern "C" fn qf_init_process_nextline(
    mut qfl: *mut qf_list_T,
    mut fmt_first: *mut efm_T,
    mut state: *mut qfstate_T,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = qf_get_nextline(state);
    if status != QF_OK as ::core::ffi::c_int {
        return status;
    }
    status = qf_parse_line(qfl, (*state).linebuf, (*state).linelen, fmt_first, fields);
    if status != QF_OK as ::core::ffi::c_int {
        return status;
    }
    return qf_add_entry(
        qfl,
        (*qfl).qf_directory,
        if *(*fields).namebuf as ::core::ffi::c_int != 0 || !(*qfl).qf_directory.is_null() {
            (*fields).namebuf
        } else if !(*qfl).qf_currfile.is_null() && (*fields).valid as ::core::ffi::c_int != 0 {
            (*qfl).qf_currfile
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        },
        (*fields).module,
        (*fields).bnr,
        (*fields).errmsg,
        (*fields).lnum,
        (*fields).end_lnum,
        (*fields).col,
        (*fields).end_col,
        (*fields).use_viscol as ::core::ffi::c_char,
        (*fields).pattern,
        (*fields).enr,
        (*fields).type_0,
        (*fields).user_data,
        (*fields).valid as ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn qf_init(
    mut wp: *mut win_T,
    mut efile: *const ::core::ffi::c_char,
    mut errorformat: *mut ::core::ffi::c_char,
    mut newlist: ::core::ffi::c_int,
    mut qf_title: *const ::core::ffi::c_char,
    mut enc: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut qi: *mut qf_info_T = if wp.is_null() {
        ql_info.get()
    } else {
        ll_get_or_alloc_list(wp)
    };
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                390 as ::core::ffi::c_uint,
                b"int qf_init(win_T *, const char *restrict, char *restrict, int, const char *restrict, char *restrict)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return qf_init_ext(
        qi,
        (*qi).qf_curlist,
        efile,
        curbuf.get(),
        ::core::ptr::null_mut::<typval_T>(),
        errorformat,
        newlist != 0,
        0 as linenr_T,
        0 as linenr_T,
        qf_title,
        enc,
    );
}
static LINE_MAXLEN: GlobalCell<size_t> = GlobalCell::new(4096 as size_t);
static fmt_pat: GlobalCell<[fmtpattern; 14]> = GlobalCell::new([
    fmtpattern {
        convchar: 'f' as ::core::ffi::c_char,
        pattern: b".\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'b' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'n' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'l' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'e' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'c' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'k' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 't' as ::core::ffi::c_char,
        pattern: b".\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'm' as ::core::ffi::c_char,
        pattern: b".\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'r' as ::core::ffi::c_char,
        pattern: b".*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'p' as ::core::ffi::c_char,
        pattern: b"[-\t .]*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'v' as ::core::ffi::c_char,
        pattern: b"\\d\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 's' as ::core::ffi::c_char,
        pattern: b".\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    fmtpattern {
        convchar: 'o' as ::core::ffi::c_char,
        pattern: b".\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
]);
pub const FMT_PATTERN_M: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const FMT_PATTERN_R: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
unsafe extern "C" fn efmpat_to_regpat(
    mut efmpat: *const ::core::ffi::c_char,
    mut regpat: *mut ::core::ffi::c_char,
    mut efminfo: *mut efm_T,
    mut idx: ::core::ffi::c_int,
    mut round: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if (*efminfo).addr[idx as usize] != 0 {
        semsg(
            gettext(
                b"E372: Too many %%%c in format string\0".as_ptr() as *const ::core::ffi::c_char
            ),
            *efmpat as ::core::ffi::c_int,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if idx != 0
        && idx < FMT_PATTERN_R
        && !vim_strchr(
            b"DXOPQ\0".as_ptr() as *const ::core::ffi::c_char,
            (*efminfo).prefix as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        || idx == FMT_PATTERN_R
            && vim_strchr(
                b"OPQ\0".as_ptr() as *const ::core::ffi::c_char,
                (*efminfo).prefix as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
    {
        semsg(
            gettext(
                b"E373: Unexpected %%%c in format string\0".as_ptr() as *const ::core::ffi::c_char
            ),
            *efmpat as ::core::ffi::c_int,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    round += 1;
    (*efminfo).addr[idx as usize] = round as ::core::ffi::c_char;
    let c2rust_fresh16 = regpat;
    regpat = regpat.offset(1);
    *c2rust_fresh16 = '\\' as ::core::ffi::c_char;
    let c2rust_fresh17 = regpat;
    regpat = regpat.offset(1);
    *c2rust_fresh17 = '(' as ::core::ffi::c_char;
    if *efmpat as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
        && *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        if *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
            && *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '%' as ::core::ffi::c_int
        {
            strcpy(
                regpat,
                b".\\{-1,}\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            regpat = regpat.offset(7 as ::core::ffi::c_int as isize);
        } else {
            strcpy(
                regpat,
                b"\\f\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            regpat = regpat.offset(4 as ::core::ffi::c_int as isize);
        }
    } else {
        let mut srcptr: *mut ::core::ffi::c_char = (*fmt_pat.ptr())[idx as usize].pattern;
        loop {
            let c2rust_fresh18 = srcptr;
            srcptr = srcptr.offset(1);
            *regpat = *c2rust_fresh18;
            if *regpat as ::core::ffi::c_int == NUL {
                break;
            }
            regpat = regpat.offset(1);
        }
    }
    let c2rust_fresh19 = regpat;
    regpat = regpat.offset(1);
    *c2rust_fresh19 = '\\' as ::core::ffi::c_char;
    let c2rust_fresh20 = regpat;
    regpat = regpat.offset(1);
    *c2rust_fresh20 = ')' as ::core::ffi::c_char;
    return regpat;
}
unsafe extern "C" fn scanf_fmt_to_regpat(
    mut pefmp: *mut *const ::core::ffi::c_char,
    mut efm: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut regpat: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut efmp: *const ::core::ffi::c_char = *pefmp;
    if *efmp as ::core::ffi::c_int == '[' as ::core::ffi::c_int
        || *efmp as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
    {
        let c2rust_fresh9 = regpat;
        regpat = regpat.offset(1);
        let c2rust_lvalue_ptr = &raw mut *c2rust_fresh9;
        *c2rust_lvalue_ptr = *efmp;
        if *c2rust_lvalue_ptr as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
            if *efmp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '^' as ::core::ffi::c_int
            {
                efmp = efmp.offset(1);
                let c2rust_fresh10 = regpat;
                regpat = regpat.offset(1);
                *c2rust_fresh10 = *efmp;
            }
            if efmp < efm.offset(len as isize) {
                efmp = efmp.offset(1);
                let c2rust_fresh11 = regpat;
                regpat = regpat.offset(1);
                *c2rust_fresh11 = *efmp;
                while efmp < efm.offset(len as isize) && {
                    efmp = efmp.offset(1);
                    let c2rust_fresh12 = regpat;
                    regpat = regpat.offset(1);
                    let c2rust_lvalue_ptr_0 = &raw mut *c2rust_fresh12;
                    *c2rust_lvalue_ptr_0 = *efmp;
                    *c2rust_lvalue_ptr_0 as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                } {}
                if efmp == efm.offset(len as isize) {
                    emsg(gettext(b"E374: Missing ] in format string\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        } else if efmp < efm.offset(len as isize) {
            efmp = efmp.offset(1);
            let c2rust_fresh13 = regpat;
            regpat = regpat.offset(1);
            *c2rust_fresh13 = *efmp;
        }
        let c2rust_fresh14 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh14 = '\\' as ::core::ffi::c_char;
        let c2rust_fresh15 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh15 = '+' as ::core::ffi::c_char;
    } else {
        semsg(
            gettext(
                b"E375: Unsupported %%%c in format string\0".as_ptr() as *const ::core::ffi::c_char
            ),
            *efmp as ::core::ffi::c_int,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *pefmp = efmp;
    return regpat;
}
unsafe extern "C" fn efm_analyze_prefix(
    mut efmp: *const ::core::ffi::c_char,
    mut efminfo: *mut efm_T,
) -> *const ::core::ffi::c_char {
    if !vim_strchr(
        b"+-\0".as_ptr() as *const ::core::ffi::c_char,
        *efmp as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        let c2rust_fresh8 = efmp;
        efmp = efmp.offset(1);
        (*efminfo).flags = *c2rust_fresh8;
    }
    if !vim_strchr(
        b"DXAEWINCZGOPQ\0".as_ptr() as *const ::core::ffi::c_char,
        *efmp as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        (*efminfo).prefix = *efmp;
    } else {
        semsg(
            gettext(b"E376: Invalid %%%c in format string prefix\0".as_ptr()
                as *const ::core::ffi::c_char),
            *efmp as ::core::ffi::c_int,
        );
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return efmp;
}
unsafe extern "C" fn efm_to_regpat(
    mut efm: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut fmt_ptr: *mut efm_T,
    mut regpat: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = regpat;
    let c2rust_fresh2 = ptr;
    ptr = ptr.offset(1);
    *c2rust_fresh2 = '^' as ::core::ffi::c_char;
    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut efmp: *const ::core::ffi::c_char = efm;
    while efmp < efm.offset(len as isize) {
        if *efmp as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            efmp = efmp.offset(1);
            let mut idx: ::core::ffi::c_int = 0;
            idx = 0 as ::core::ffi::c_int;
            while idx < FMT_PATTERNS {
                if (*fmt_pat.ptr())[idx as usize].convchar as ::core::ffi::c_int
                    == *efmp as ::core::ffi::c_int
                {
                    break;
                }
                idx += 1;
            }
            if idx < FMT_PATTERNS {
                ptr = efmpat_to_regpat(efmp, ptr, fmt_ptr, idx, round);
                if ptr.is_null() {
                    return FAIL;
                }
                round += 1;
            } else if *efmp as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                efmp = efmp.offset(1);
                ptr = scanf_fmt_to_regpat(&raw mut efmp, efm, len, ptr);
                if ptr.is_null() {
                    return FAIL;
                }
            } else if !vim_strchr(
                b"%\\.^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                *efmp as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                let c2rust_fresh3 = ptr;
                ptr = ptr.offset(1);
                *c2rust_fresh3 = *efmp;
            } else if *efmp as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
                let c2rust_fresh4 = ptr;
                ptr = ptr.offset(1);
                *c2rust_fresh4 = '*' as ::core::ffi::c_char;
            } else if *efmp as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                (*fmt_ptr).conthere = true_0;
            } else if efmp == efm.offset(1 as ::core::ffi::c_int as isize) {
                efmp = efm_analyze_prefix(efmp, fmt_ptr);
                if efmp.is_null() {
                    return FAIL;
                }
            } else {
                semsg(
                    gettext(b"E377: Invalid %%%c in format string\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    *efmp as ::core::ffi::c_int,
                );
                return FAIL;
            }
        } else {
            if *efmp as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && efmp.offset(1 as ::core::ffi::c_int as isize) < efm.offset(len as isize)
            {
                efmp = efmp.offset(1);
            } else if !vim_strchr(
                b".*^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                *efmp as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                let c2rust_fresh5 = ptr;
                ptr = ptr.offset(1);
                *c2rust_fresh5 = '\\' as ::core::ffi::c_char;
            }
            if *efmp != 0 {
                let c2rust_fresh6 = ptr;
                ptr = ptr.offset(1);
                *c2rust_fresh6 = *efmp;
            }
        }
        efmp = efmp.offset(1);
    }
    let c2rust_fresh7 = ptr;
    ptr = ptr.offset(1);
    *c2rust_fresh7 = '$' as ::core::ffi::c_char;
    *ptr = NUL as ::core::ffi::c_char;
    return OK;
}
static fmt_start: GlobalCell<*mut efm_T> = GlobalCell::new(::core::ptr::null_mut::<efm_T>());
static qftf_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_6 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
unsafe extern "C" fn free_efm_list(mut efm_first: *mut *mut efm_T) {
    let mut efm_ptr: *mut efm_T = *efm_first;
    while !efm_ptr.is_null() {
        *efm_first = (*efm_ptr).next;
        vim_regfree((*efm_ptr).prog);
        xfree(efm_ptr as *mut ::core::ffi::c_void);
        efm_ptr = *efm_first;
    }
    fmt_start.set(::core::ptr::null_mut::<efm_T>());
}
unsafe extern "C" fn efm_regpat_bufsz(mut efm: *mut ::core::ffi::c_char) -> size_t {
    let mut sz: size_t = ((FMT_PATTERNS * 3 as ::core::ffi::c_int) as size_t)
        .wrapping_add(strlen(efm) << 2 as ::core::ffi::c_int);
    let mut i: ::core::ffi::c_int = FMT_PATTERNS - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let c2rust_fresh1 = i;
        i = i - 1;
        sz = sz.wrapping_add(strlen((*fmt_pat.ptr())[c2rust_fresh1 as usize].pattern));
    }
    sz = sz.wrapping_add(2 as size_t);
    return sz;
}
unsafe extern "C" fn efm_option_part_len(
    mut efm: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    len = 0 as ::core::ffi::c_int;
    while *efm.offset(len as isize) as ::core::ffi::c_int != NUL
        && *efm.offset(len as isize) as ::core::ffi::c_int != ',' as ::core::ffi::c_int
    {
        if *efm.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *efm.offset((len + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != NUL
        {
            len += 1;
        }
        len += 1;
    }
    return len;
}
unsafe extern "C" fn parse_efm_option(mut efm: *mut ::core::ffi::c_char) -> *mut efm_T {
    let mut fmt_first: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
    let mut fmt_last: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
    let mut sz: size_t = efm_regpat_bufsz(efm);
    let mut fmtstr: *mut ::core::ffi::c_char = xmalloc(sz) as *mut ::core::ffi::c_char;
    '_parse_efm_end: {
        '_parse_efm_error: {
            while *efm.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                let mut fmt_ptr: *mut efm_T =
                    xcalloc(1 as size_t, ::core::mem::size_of::<efm_T>()) as *mut efm_T;
                if fmt_first.is_null() {
                    fmt_first = fmt_ptr;
                } else {
                    (*fmt_last).next = fmt_ptr;
                }
                fmt_last = fmt_ptr;
                let mut len: ::core::ffi::c_int = efm_option_part_len(efm);
                if efm_to_regpat(efm, len, fmt_ptr, fmtstr) == FAIL {
                    break '_parse_efm_error;
                }
                (*fmt_ptr).prog = vim_regcomp(fmtstr, RE_MAGIC + RE_STRING);
                if (*fmt_ptr).prog.is_null() {
                    break '_parse_efm_error;
                }
                efm = skip_to_option_part(efm.offset(len as isize));
            }
            if fmt_first.is_null() {
                emsg(gettext(
                    b"E378: 'errorformat' contains no pattern\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            break '_parse_efm_end;
        }
        free_efm_list(&raw mut fmt_first);
    }
    xfree(fmtstr as *mut ::core::ffi::c_void);
    return fmt_first;
}
unsafe extern "C" fn qf_grow_linebuf(
    mut state: *mut qfstate_T,
    mut newsz: size_t,
) -> *mut ::core::ffi::c_char {
    (*state).linelen = if newsz > LINE_MAXLEN.get() {
        (*LINE_MAXLEN.ptr()).wrapping_sub(1 as size_t)
    } else {
        newsz
    };
    if (*state).growbuf.is_null() {
        (*state).growbuf =
            xmalloc((*state).linelen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        (*state).growbufsiz = (*state).linelen;
    } else if (*state).linelen > (*state).growbufsiz {
        (*state).growbuf = xrealloc(
            (*state).growbuf as *mut ::core::ffi::c_void,
            (*state).linelen.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        (*state).growbufsiz = (*state).linelen;
    }
    return (*state).growbuf;
}
unsafe extern "C" fn qf_get_next_str_line(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    let mut p_str: *mut ::core::ffi::c_char = (*state).p_str;
    if *p_str as ::core::ffi::c_int == NUL {
        return QF_END_OF_INPUT as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char = vim_strchr(p_str, '\n' as ::core::ffi::c_int);
    let mut len: size_t = if !p.is_null() {
        (p.offset_from(p_str) as size_t).wrapping_add(1 as size_t)
    } else {
        strlen(p_str)
    };
    if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
        (*state).linebuf = qf_grow_linebuf(state, len);
    } else {
        (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
        (*state).linelen = len;
    }
    memcpy(
        (*state).linebuf as *mut ::core::ffi::c_void,
        p_str as *const ::core::ffi::c_void,
        (*state).linelen,
    );
    *(*state).linebuf.offset((*state).linelen as isize) = NUL as ::core::ffi::c_char;
    p_str = p_str.offset(len as isize);
    (*state).p_str = p_str;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_get_next_list_line(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    let mut p_li: *mut listitem_T = (*state).p_li;
    while !p_li.is_null()
        && ((*p_li).li_tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*p_li).li_tv.vval.v_string.is_null())
    {
        p_li = (*p_li).li_next;
    }
    if p_li.is_null() {
        (*state).p_li = ::core::ptr::null_mut::<listitem_T>();
        return QF_END_OF_INPUT as ::core::ffi::c_int;
    }
    let mut len: size_t = strlen((*p_li).li_tv.vval.v_string);
    if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
        (*state).linebuf = qf_grow_linebuf(state, len);
    } else {
        (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
        (*state).linelen = len;
    }
    xstrlcpy(
        (*state).linebuf,
        (*p_li).li_tv.vval.v_string,
        (*state).linelen.wrapping_add(1 as size_t),
    );
    (*state).p_li = (*p_li).li_next;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_get_next_buf_line(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    if (*state).buflnum > (*state).lnumlast {
        return QF_END_OF_INPUT as ::core::ffi::c_int;
    }
    let mut p_buf: *mut ::core::ffi::c_char = ml_get_buf((*state).buf, (*state).buflnum);
    let mut len: size_t = ml_get_buf_len((*state).buf, (*state).buflnum) as size_t;
    (*state).buflnum =
        ((*state).buflnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
    if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
        (*state).linebuf = qf_grow_linebuf(state, len);
    } else {
        (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
        (*state).linelen = len;
    }
    xstrlcpy(
        (*state).linebuf,
        p_buf,
        (*state).linelen.wrapping_add(1 as size_t),
    );
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_get_next_file_line(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    loop {
        *__errno_location() = 0 as ::core::ffi::c_int;
        if fgets(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE,
            (*state).fd,
        )
        .is_null()
        {
            if *__errno_location() == EINTR {
                continue;
            }
            return QF_END_OF_INPUT as ::core::ffi::c_int;
        } else {
            let mut discard: bool = false_0 != 0;
            (*state).linelen = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
            if (*state).linelen == (IOSIZE - 1 as ::core::ffi::c_int) as size_t
                && !((*IObuff.ptr())[(*state).linelen.wrapping_sub(1 as size_t) as usize]
                    as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int)
            {
                if (*state).growbuf.is_null() {
                    (*state).growbufsiz =
                        (2 as ::core::ffi::c_int * (IOSIZE - 1 as ::core::ffi::c_int)) as size_t;
                    (*state).growbuf = xmalloc((*state).growbufsiz) as *mut ::core::ffi::c_char;
                }
                memcpy(
                    (*state).growbuf as *mut ::core::ffi::c_void,
                    IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    (IOSIZE - 1 as ::core::ffi::c_int) as size_t,
                );
                let mut growbuflen: size_t = (*state).linelen;
                *(*state).growbuf.offset(growbuflen as isize) = NUL as ::core::ffi::c_char;
                loop {
                    *__errno_location() = 0 as ::core::ffi::c_int;
                    if fgets(
                        (*state).growbuf.offset(growbuflen as isize),
                        (*state).growbufsiz.wrapping_sub(growbuflen) as ::core::ffi::c_int,
                        (*state).fd,
                    )
                    .is_null()
                    {
                        if *__errno_location() != EINTR {
                            break;
                        }
                    } else {
                        (*state).linelen = strlen((*state).growbuf.offset(growbuflen as isize));
                        growbuflen = growbuflen.wrapping_add((*state).linelen);
                        if *(*state)
                            .growbuf
                            .offset(growbuflen.wrapping_sub(1 as size_t) as isize)
                            as ::core::ffi::c_int
                            == '\n' as ::core::ffi::c_int
                        {
                            break;
                        }
                        if (*state).growbufsiz == LINE_MAXLEN.get() {
                            discard = true_0 != 0;
                            break;
                        } else {
                            (*state).growbufsiz = if (2 as size_t).wrapping_mul((*state).growbufsiz)
                                < LINE_MAXLEN.get()
                            {
                                (2 as size_t).wrapping_mul((*state).growbufsiz)
                            } else {
                                LINE_MAXLEN.get()
                            };
                            (*state).growbuf = xrealloc(
                                (*state).growbuf as *mut ::core::ffi::c_void,
                                (*state).growbufsiz,
                            )
                                as *mut ::core::ffi::c_char;
                        }
                    }
                }
                while discard {
                    *__errno_location() = 0 as ::core::ffi::c_int;
                    if fgets(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE,
                        (*state).fd,
                    )
                    .is_null()
                    {
                        if *__errno_location() != EINTR {
                            break;
                        }
                    } else if strlen(IObuff.ptr() as *mut ::core::ffi::c_char)
                        < (IOSIZE - 1 as ::core::ffi::c_int) as size_t
                        || (*IObuff.ptr())[(IOSIZE - 2 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            == '\n' as ::core::ffi::c_int
                    {
                        break;
                    }
                }
                (*state).linebuf = (*state).growbuf;
                (*state).linelen = growbuflen;
            } else {
                (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
            }
            if (*state).vc.vc_type != CONV_NONE as ::core::ffi::c_int
                && has_non_ascii((*state).linebuf) as ::core::ffi::c_int != 0
            {
                let mut line: *mut ::core::ffi::c_char = string_convert(
                    &raw mut (*state).vc,
                    (*state).linebuf,
                    &raw mut (*state).linelen,
                );
                if !line.is_null() {
                    if (*state).linelen < IOSIZE as size_t {
                        xstrlcpy(
                            (*state).linebuf,
                            line,
                            (*state).linelen.wrapping_add(1 as size_t),
                        );
                        xfree(line as *mut ::core::ffi::c_void);
                    } else {
                        xfree((*state).growbuf as *mut ::core::ffi::c_void);
                        (*state).linebuf = line;
                        (*state).growbuf = line;
                        (*state).growbufsiz = if (*state).linelen < LINE_MAXLEN.get() {
                            (*state).linelen
                        } else {
                            LINE_MAXLEN.get()
                        };
                    }
                }
            }
            return QF_OK as ::core::ffi::c_int;
        }
    }
}
unsafe extern "C" fn qf_get_nextline(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = QF_FAIL as ::core::ffi::c_int;
    if (*state).fd.is_null() {
        if !(*state).tv.is_null() {
            if (*(*state).tv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                status = qf_get_next_str_line(state);
            } else if (*(*state).tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                status = qf_get_next_list_line(state);
            }
        } else {
            status = qf_get_next_buf_line(state);
        }
    } else {
        status = qf_get_next_file_line(state);
    }
    if status != QF_OK as ::core::ffi::c_int {
        return status;
    }
    if (*state).linelen > 0 as size_t
        && *(*state)
            .linebuf
            .offset((*state).linelen.wrapping_sub(1 as size_t) as isize)
            as ::core::ffi::c_int
            == '\n' as ::core::ffi::c_int
    {
        *(*state)
            .linebuf
            .offset((*state).linelen.wrapping_sub(1 as size_t) as isize) =
            NUL as ::core::ffi::c_char;
    }
    remove_bom((*state).linebuf);
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_stack_empty(mut qi: *const qf_info_T) -> bool {
    return qi.is_null() || (*qi).qf_listcount <= 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_list_empty(mut qfl: *mut qf_list_T) -> bool {
    return qfl.is_null() || (*qfl).qf_count <= 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_list_has_valid_entries(mut qfl: *mut qf_list_T) -> bool {
    return !qf_list_empty(qfl) && !(*qfl).qf_nonevalid;
}
unsafe extern "C" fn qf_get_list(
    mut qi: *mut qf_info_T,
    mut idx: ::core::ffi::c_int,
) -> *mut qf_list_T {
    return (*qi).qf_lists.offset(idx as isize);
}
unsafe extern "C" fn qf_parse_line(
    mut qfl: *mut qf_list_T,
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_first: *mut efm_T,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    let mut fmt_ptr: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_int = 0;
    's_240: {
        loop {
            if (*fmt_start.ptr()).is_null() {
                fmt_ptr = fmt_first;
            } else {
                fmt_ptr = fmt_start.get();
                fmt_start.set(::core::ptr::null_mut::<efm_T>());
            }
            (*fields).valid = true_0 != 0;
            while !fmt_ptr.is_null() {
                idx = (*fmt_ptr).prefix as uint8_t as ::core::ffi::c_int;
                status = qf_parse_get_fields(
                    linebuf,
                    linelen,
                    fmt_ptr,
                    fields,
                    (*qfl).qf_multiline as ::core::ffi::c_int,
                    (*qfl).qf_multiscan as ::core::ffi::c_int,
                    &raw mut tail,
                );
                if status == QF_NOMEM as ::core::ffi::c_int {
                    return status;
                }
                if status == QF_OK as ::core::ffi::c_int {
                    break;
                }
                fmt_ptr = (*fmt_ptr).next;
            }
            (*qfl).qf_multiscan = false_0 != 0;
            if fmt_ptr.is_null()
                || idx == 'D' as ::core::ffi::c_int
                || idx == 'X' as ::core::ffi::c_int
            {
                if !fmt_ptr.is_null() {
                    status = qf_parse_dir_pfx(idx, fields, qfl);
                    if status != QF_OK as ::core::ffi::c_int {
                        return status;
                    }
                }
                status = qf_parse_line_nomatch(linebuf, linelen, fields);
                if status != QF_OK as ::core::ffi::c_int {
                    return status;
                }
                if fmt_ptr.is_null() {
                    (*qfl).qf_multiignore = false_0 != 0;
                    (*qfl).qf_multiline = (*qfl).qf_multiignore;
                }
                break 's_240;
            } else {
                if (*fmt_ptr).conthere != 0 {
                    fmt_start.set(fmt_ptr);
                }
                if !vim_strchr(b"AEWIN\0".as_ptr() as *const ::core::ffi::c_char, idx).is_null() {
                    (*qfl).qf_multiline = true_0 != 0;
                    (*qfl).qf_multiignore = false_0 != 0;
                    break;
                } else if !vim_strchr(b"CZ\0".as_ptr() as *const ::core::ffi::c_char, idx).is_null()
                {
                    status = qf_parse_multiline_pfx(idx, qfl, fields);
                    if status != QF_OK as ::core::ffi::c_int {
                        return status;
                    }
                    break;
                } else {
                    if vim_strchr(b"OPQ\0".as_ptr() as *const ::core::ffi::c_char, idx).is_null() {
                        break;
                    }
                    status = qf_parse_file_pfx(idx, fields, qfl, tail);
                    if status != QF_MULTISCAN as ::core::ffi::c_int {
                        break;
                    }
                    let mut s: *mut ::core::ffi::c_char = skipwhite(tail);
                    let mut new_linelen: size_t = strlen(s);
                    if new_linelen >= linelen {
                        return QF_IGNORE_LINE as ::core::ffi::c_int;
                    }
                    linebuf = s;
                    linelen = new_linelen;
                }
            }
        }
        if (*fmt_ptr).flags as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            if (*qfl).qf_multiline {
                (*qfl).qf_multiignore = true_0 != 0;
            }
            return QF_IGNORE_LINE as ::core::ffi::c_int;
        }
    }
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_alloc_fields(mut pfields: *mut qffields_T) {
    (*pfields).namebuf =
        xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    (*pfields).module =
        xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    (*pfields).errmsglen = (CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t;
    (*pfields).errmsg = xmalloc((*pfields).errmsglen) as *mut ::core::ffi::c_char;
    (*pfields).pattern =
        xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn qf_free_fields(mut pfields: *mut qffields_T) {
    xfree((*pfields).namebuf as *mut ::core::ffi::c_void);
    xfree((*pfields).module as *mut ::core::ffi::c_void);
    xfree((*pfields).errmsg as *mut ::core::ffi::c_void);
    xfree((*pfields).pattern as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn qf_setup_state(
    mut pstate: *mut qfstate_T,
    mut enc: *mut ::core::ffi::c_char,
    mut efile: *const ::core::ffi::c_char,
    mut tv: *mut typval_T,
    mut buf: *mut buf_T,
    mut lnumfirst: linenr_T,
    mut lnumlast: linenr_T,
) -> ::core::ffi::c_int {
    (*pstate).vc.vc_type = CONV_NONE as ::core::ffi::c_int;
    if !enc.is_null() && *enc as ::core::ffi::c_int != NUL {
        convert_setup(&raw mut (*pstate).vc, enc, p_enc.get());
    }
    if !efile.is_null() && {
        (*pstate).fd = if strequal(efile, b"-\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
        {
            fdopen(
                os_open_stdin_fd(),
                b"r\0".as_ptr() as *const ::core::ffi::c_char,
            )
        } else {
            os_fopen(efile, b"r\0".as_ptr() as *const ::core::ffi::c_char)
        };
        (*pstate).fd.is_null()
    } {
        semsg(
            gettext(&raw const e_openerrf as *const ::core::ffi::c_char),
            efile,
        );
        return FAIL;
    }
    if !tv.is_null() {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*pstate).p_str = (*tv).vval.v_string;
        } else if (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*pstate).p_li = tv_list_first((*tv).vval.v_list);
        }
        (*pstate).tv = tv;
    }
    (*pstate).buf = buf;
    (*pstate).buflnum = lnumfirst;
    (*pstate).lnumlast = lnumlast;
    return OK;
}
unsafe extern "C" fn qf_cleanup_state(mut pstate: *mut qfstate_T) {
    if !(*pstate).fd.is_null() {
        fclose((*pstate).fd);
    }
    xfree((*pstate).growbuf as *mut ::core::ffi::c_void);
    if (*pstate).vc.vc_type != CONV_NONE as ::core::ffi::c_int {
        convert_setup(
            &raw mut (*pstate).vc,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    }
}
unsafe extern "C" fn qf_init_ext(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut efile: *const ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut tv: *mut typval_T,
    mut errorformat: *mut ::core::ffi::c_char,
    mut newlist: bool,
    mut lnumfirst: linenr_T,
    mut lnumlast: linenr_T,
    mut qf_title: *const ::core::ffi::c_char,
    mut enc: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut qfl: *mut qf_list_T = ::core::ptr::null_mut::<qf_list_T>();
    let mut adding: bool = false;
    let mut efm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut state: qfstate_T = qfstate_T {
        linebuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        linelen: 0,
        growbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        growbufsiz: 0,
        fd: ::core::ptr::null_mut::<FILE>(),
        tv: ::core::ptr::null_mut::<typval_T>(),
        p_str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        p_list: ::core::ptr::null_mut::<list_T>(),
        p_li: ::core::ptr::null_mut::<listitem_T>(),
        buf: ::core::ptr::null_mut::<buf_T>(),
        buflnum: 0,
        lnumlast: 0,
        vc: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
    };
    let mut fields: qffields_T = qffields_T {
        namebuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        bnr: 0,
        module: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        errmsglen: 0,
        lnum: 0,
        end_lnum: 0,
        col: 0,
        end_col: 0,
        use_viscol: false,
        pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        enr: 0,
        type_0: 0,
        user_data: ::core::ptr::null_mut::<typval_T>(),
        valid: false,
    };
    let mut old_last: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    static fmt_first: GlobalCell<*mut efm_T> = GlobalCell::new(::core::ptr::null_mut::<efm_T>());
    static last_efm: GlobalCell<*mut ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
    let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        qf_last_bufname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    qf_alloc_fields(&raw mut fields);
    '_qf_init_end: {
        if qf_setup_state(&raw mut state, enc, efile, tv, buf, lnumfirst, lnumlast) != FAIL {
            qfl = ::core::ptr::null_mut::<qf_list_T>();
            adding = false_0 != 0;
            if newlist as ::core::ffi::c_int != 0 || qf_idx == (*qi).qf_listcount {
                qf_new_list(qi, qf_title);
                qf_idx = (*qi).qf_curlist;
                qfl = qf_get_list(qi, qf_idx);
            } else {
                adding = true_0 != 0;
                qfl = qf_get_list(qi, qf_idx);
                if !qf_list_empty(qfl) {
                    old_last = (*qfl).qf_last;
                }
            }
            efm = if errorformat == p_efm.get()
                && tv.is_null()
                && !buf.is_null()
                && *(*buf).b_p_efm as ::core::ffi::c_int != NUL
            {
                (*buf).b_p_efm
            } else {
                errorformat
            };
            if (*last_efm.ptr()).is_null() || strcmp(last_efm.get(), efm) != 0 as ::core::ffi::c_int
            {
                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                    last_efm.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__0);
                *ptr__0 = NULL_0;
                let _ = *ptr__0;
                free_efm_list(fmt_first.ptr());
                fmt_first.set(parse_efm_option(efm));
                if !(*fmt_first.ptr()).is_null() {
                    last_efm.set(xstrdup(efm));
                }
            }
            '_error2: {
                if !(*fmt_first.ptr()).is_null() {
                    got_int.set(false_0 != 0);
                    while !got_int.get() {
                        let mut status: ::core::ffi::c_int = qf_init_process_nextline(
                            qfl,
                            fmt_first.get(),
                            &raw mut state,
                            &raw mut fields,
                        );
                        if status == QF_END_OF_INPUT as ::core::ffi::c_int {
                            break;
                        }
                        if status == QF_FAIL as ::core::ffi::c_int {
                            break '_error2;
                        }
                        line_breakcheck();
                    }
                    if state.fd.is_null() || ferror(state.fd) == 0 {
                        if (*qfl).qf_index == 0 as ::core::ffi::c_int {
                            (*qfl).qf_ptr = (*qfl).qf_start;
                            (*qfl).qf_index = 1 as ::core::ffi::c_int;
                            (*qfl).qf_nonevalid = true_0 != 0;
                        } else {
                            (*qfl).qf_nonevalid = false_0 != 0;
                            if (*qfl).qf_ptr.is_null() {
                                (*qfl).qf_ptr = (*qfl).qf_start;
                            }
                        }
                        retval = (*qfl).qf_count;
                        break '_qf_init_end;
                    } else {
                        emsg(gettext(&raw const e_readerrf as *const ::core::ffi::c_char));
                    }
                }
            }
            if !adding {
                qf_free(qfl);
                (*qi).qf_listcount -= 1;
                if (*qi).qf_curlist > 0 as ::core::ffi::c_int {
                    (*qi).qf_curlist -= 1;
                }
            }
        }
    }
    if qf_idx == (*qi).qf_curlist {
        qf_update_buffer(qi, old_last);
    }
    qf_cleanup_state(&raw mut state);
    qf_free_fields(&raw mut fields);
    return retval;
}
unsafe extern "C" fn qf_store_title(
    mut qfl: *mut qf_list_T,
    mut title: *const ::core::ffi::c_char,
) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*qfl).qf_title as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    if title.is_null() {
        return;
    }
    let mut len: size_t = strlen(title).wrapping_add(1 as size_t);
    let mut p: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
    (*qfl).qf_title = p;
    xstrlcpy(p, title, len.wrapping_add(1 as size_t));
}
unsafe extern "C" fn qf_cmdtitle(mut cmd: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    static qftitle_str: GlobalCell<[::core::ffi::c_char; 1025]> = GlobalCell::new([0; 1025]);
    snprintf(
        qftitle_str.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b":%s\0".as_ptr() as *const ::core::ffi::c_char,
        cmd,
    );
    return qftitle_str.ptr() as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn qf_get_curlist(mut qi: *mut qf_info_T) -> *mut qf_list_T {
    return qf_get_list(qi, (*qi).qf_curlist);
}
unsafe extern "C" fn qf_pop_stack(mut qi: *mut qf_info_T, mut adjust: bool) {
    qf_free((*qi).qf_lists.offset(0 as ::core::ffi::c_int as isize));
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*qi).qf_listcount {
        *(*qi)
            .qf_lists
            .offset((i - 1 as ::core::ffi::c_int) as isize) = *(*qi).qf_lists.offset(i as isize);
        i += 1;
    }
    memset(
        (*qi)
            .qf_lists
            .offset((*qi).qf_listcount as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<qf_list_T>(),
    );
    if adjust {
        (*qi).qf_listcount -= 1;
        if (*qi).qf_curlist == 0 as ::core::ffi::c_int {
            (*qi).qf_curlist = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
        } else {
            (*qi).qf_curlist -= 1;
        }
    }
}
unsafe extern "C" fn qf_new_list(mut qi: *mut qf_info_T, mut qf_title: *const ::core::ffi::c_char) {
    while (*qi).qf_listcount > (*qi).qf_curlist + 1 as ::core::ffi::c_int {
        (*qi).qf_listcount -= 1;
        qf_free((*qi).qf_lists.offset((*qi).qf_listcount as isize));
    }
    if (*qi).qf_listcount == (*qi).qf_maxcount {
        qf_pop_stack(qi, false_0 != 0);
        (*qi).qf_curlist = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
    } else {
        let c2rust_fresh21 = (*qi).qf_listcount;
        (*qi).qf_listcount = (*qi).qf_listcount + 1;
        (*qi).qf_curlist = c2rust_fresh21;
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    memset(
        qfl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<qf_list_T>(),
    );
    qf_store_title(qfl, qf_title);
    (*qfl).qfl_type = (*qi).qfl_type;
    last_qf_id.set((*last_qf_id.ptr()).wrapping_add(1));
    (*qfl).qf_id = last_qf_id.get();
    (*qfl).qf_has_user_data = false_0 != 0;
}
unsafe extern "C" fn qf_parse_fmt_f(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut prefix: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    let mut c: ::core::ffi::c_char = *(*rmp).endp[midx as usize];
    *(*rmp).endp[midx as usize] = NUL as ::core::ffi::c_char;
    expand_env((*rmp).startp[midx as usize], (*fields).namebuf, CMDBUFFSIZE);
    *(*rmp).endp[midx as usize] = c;
    if !vim_strchr(b"OPQ\0".as_ptr() as *const ::core::ffi::c_char, prefix).is_null()
        && !os_path_exists((*fields).namebuf)
    {
        return QF_FAIL as ::core::ffi::c_int;
    }
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_b(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    let mut bnr: ::core::ffi::c_int = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
    if buflist_findnr(bnr).is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).bnr = bnr;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_n(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).enr = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_l(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).lnum = atol((*rmp).startp[midx as usize]) as linenr_T;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_e(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).end_lnum = atol((*rmp).startp[midx as usize]) as linenr_T;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_c(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_k(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).end_col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_t(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).type_0 = *(*rmp).startp[midx as usize];
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn copy_nonerror_line(
    mut linebuf: *const ::core::ffi::c_char,
    mut linelen: size_t,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if linelen >= (*fields).errmsglen {
        (*fields).errmsg = xrealloc(
            (*fields).errmsg as *mut ::core::ffi::c_void,
            linelen.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        (*fields).errmsglen = linelen.wrapping_add(1 as size_t);
    }
    xstrlcpy((*fields).errmsg, linebuf, linelen.wrapping_add(1 as size_t));
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_m(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    let mut len: size_t =
        (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
    if len >= (*fields).errmsglen {
        (*fields).errmsg = xrealloc(
            (*fields).errmsg as *mut ::core::ffi::c_void,
            len.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        (*fields).errmsglen = len.wrapping_add(1 as size_t);
    }
    xstrlcpy(
        (*fields).errmsg,
        (*rmp).startp[midx as usize],
        len.wrapping_add(1 as size_t),
    );
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_r(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    *tail = (*rmp).startp[midx as usize];
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_p(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).col = 0 as ::core::ffi::c_int;
    let mut match_ptr: *mut ::core::ffi::c_char = (*rmp).startp[midx as usize];
    while match_ptr != (*rmp).endp[midx as usize] {
        (*fields).col += 1;
        if *match_ptr as ::core::ffi::c_int == TAB {
            (*fields).col += 7 as ::core::ffi::c_int;
            (*fields).col -= (*fields).col % 8 as ::core::ffi::c_int;
        }
        match_ptr = match_ptr.offset(1);
    }
    (*fields).col += 1;
    (*fields).use_viscol = true_0 != 0;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_v(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    (*fields).col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
    (*fields).use_viscol = true_0 != 0;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_s(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    let mut len: size_t =
        (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
    len = if len < (1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as size_t {
        len
    } else {
        (1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as size_t
    };
    strcpy(
        (*fields).pattern,
        b"^\\V\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    xstrlcat(
        (*fields).pattern,
        (*rmp).startp[midx as usize],
        len.wrapping_add(4 as size_t),
    );
    *(*fields)
        .pattern
        .offset(len.wrapping_add(3 as size_t) as isize) = '\\' as ::core::ffi::c_char;
    *(*fields)
        .pattern
        .offset(len.wrapping_add(4 as size_t) as isize) = '$' as ::core::ffi::c_char;
    *(*fields)
        .pattern
        .offset(len.wrapping_add(5 as size_t) as isize) = NUL as ::core::ffi::c_char;
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_fmt_o(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
        return QF_FAIL as ::core::ffi::c_int;
    }
    let mut len: size_t =
        (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
    let mut dsize: size_t = strlen((*fields).module)
        .wrapping_add(len)
        .wrapping_add(1 as size_t);
    dsize = if dsize < 1024 as size_t {
        dsize
    } else {
        1024 as size_t
    };
    xstrlcat((*fields).module, (*rmp).startp[midx as usize], dsize);
    return QF_OK as ::core::ffi::c_int;
}
static qf_parse_fmt: GlobalCell<
    [Option<
        unsafe extern "C" fn(
            *mut regmatch_T,
            ::core::ffi::c_int,
            *mut qffields_T,
        ) -> ::core::ffi::c_int,
    >; 14],
> = GlobalCell::new([
    None,
    Some(
        qf_parse_fmt_b
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_n
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_l
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_e
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_c
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_k
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_t
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_m
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    None,
    Some(
        qf_parse_fmt_p
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_v
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_s
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
    Some(
        qf_parse_fmt_o
            as unsafe extern "C" fn(
                *mut regmatch_T,
                ::core::ffi::c_int,
                *mut qffields_T,
            ) -> ::core::ffi::c_int,
    ),
]);
unsafe extern "C" fn qf_parse_match(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_ptr: *mut efm_T,
    mut regmatch: *mut regmatch_T,
    mut fields: *mut qffields_T,
    mut qf_multiline: ::core::ffi::c_int,
    mut qf_multiscan: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_char = (*fmt_ptr).prefix;
    if (idx as ::core::ffi::c_int == 'C' as ::core::ffi::c_int
        || idx as ::core::ffi::c_int == 'Z' as ::core::ffi::c_int)
        && qf_multiline == 0
    {
        return QF_FAIL as ::core::ffi::c_int;
    }
    if !vim_strchr(
        b"EWIN\0".as_ptr() as *const ::core::ffi::c_char,
        idx as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        (*fields).type_0 = idx;
    } else {
        (*fields).type_0 = 0 as ::core::ffi::c_char;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < FMT_PATTERNS {
        let mut status: ::core::ffi::c_int = QF_OK as ::core::ffi::c_int;
        let mut midx: ::core::ffi::c_int = (*fmt_ptr).addr[i as usize] as ::core::ffi::c_int;
        if i == 0 as ::core::ffi::c_int && midx > 0 as ::core::ffi::c_int {
            status = qf_parse_fmt_f(regmatch, midx, fields, idx as ::core::ffi::c_int);
        } else if i == FMT_PATTERN_M {
            if (*fmt_ptr).flags as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                && qf_multiscan == 0
            {
                status = copy_nonerror_line(linebuf, linelen, fields);
            } else if midx > 0 as ::core::ffi::c_int {
                status = qf_parse_fmt_m(regmatch, midx, fields);
            }
        } else if i == FMT_PATTERN_R && midx > 0 as ::core::ffi::c_int {
            status = qf_parse_fmt_r(regmatch, midx, tail);
        } else if midx > 0 as ::core::ffi::c_int {
            status = (*qf_parse_fmt.ptr())[i as usize].expect("non-null function pointer")(
                regmatch, midx, fields,
            );
        }
        if status != QF_OK as ::core::ffi::c_int {
            return status;
        }
        i += 1;
    }
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_get_fields(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_ptr: *mut efm_T,
    mut fields: *mut qffields_T,
    mut qf_multiline: ::core::ffi::c_int,
    mut qf_multiscan: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if qf_multiscan != 0
        && vim_strchr(
            b"OPQ\0".as_ptr() as *const ::core::ffi::c_char,
            (*fmt_ptr).prefix as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        return QF_FAIL as ::core::ffi::c_int;
    }
    *(*fields).namebuf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    (*fields).bnr = 0 as ::core::ffi::c_int;
    *(*fields).module.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    *(*fields).pattern.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    if qf_multiscan == 0 {
        *(*fields).errmsg.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    }
    (*fields).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*fields).end_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*fields).col = 0 as ::core::ffi::c_int;
    (*fields).end_col = 0 as ::core::ffi::c_int;
    (*fields).use_viscol = false_0 != 0;
    (*fields).enr = -1 as ::core::ffi::c_int;
    (*fields).type_0 = 0 as ::core::ffi::c_char;
    *tail = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = true_0 != 0;
    regmatch.regprog = (*fmt_ptr).prog;
    let mut r: bool = vim_regexec(&raw mut regmatch, linebuf, 0 as colnr_T);
    (*fmt_ptr).prog = regmatch.regprog;
    let mut status: ::core::ffi::c_int = QF_FAIL as ::core::ffi::c_int;
    if r {
        status = qf_parse_match(
            linebuf,
            linelen,
            fmt_ptr,
            &raw mut regmatch,
            fields,
            qf_multiline,
            qf_multiscan,
            tail,
        );
    }
    return status;
}
unsafe extern "C" fn qf_parse_dir_pfx(
    mut idx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    if idx == 'D' as ::core::ffi::c_int {
        if *(*fields).namebuf as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E379: Missing or empty directory name\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*qfl).qf_directory = qf_push_dir(
            (*fields).namebuf,
            &raw mut (*qfl).qf_dir_stack,
            false_0 != 0,
        );
        if (*qfl).qf_directory.is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
    } else if idx == 'X' as ::core::ffi::c_int {
        (*qfl).qf_directory = qf_pop_dir(&raw mut (*qfl).qf_dir_stack);
    }
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_file_pfx(
    mut idx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut qfl: *mut qf_list_T,
    mut tail: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    (*fields).valid = false_0 != 0;
    if *(*fields).namebuf as ::core::ffi::c_int == NUL
        || os_path_exists((*fields).namebuf) as ::core::ffi::c_int != 0
    {
        if *(*fields).namebuf as ::core::ffi::c_int != 0 && idx == 'P' as ::core::ffi::c_int {
            (*qfl).qf_currfile = qf_push_dir(
                (*fields).namebuf,
                &raw mut (*qfl).qf_file_stack,
                true_0 != 0,
            );
        } else if idx == 'Q' as ::core::ffi::c_int {
            (*qfl).qf_currfile = qf_pop_dir(&raw mut (*qfl).qf_file_stack);
        }
        *(*fields).namebuf = NUL as ::core::ffi::c_char;
        if !tail.is_null() && *tail as ::core::ffi::c_int != 0 {
            (*qfl).qf_multiscan = true_0 != 0;
            return QF_MULTISCAN as ::core::ffi::c_int;
        }
    }
    return QF_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_parse_line_nomatch(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    *(*fields).namebuf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    (*fields).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*fields).valid = false_0 != 0;
    return copy_nonerror_line(linebuf, linelen, fields);
}
unsafe extern "C" fn qf_parse_multiline_pfx(
    mut idx: ::core::ffi::c_int,
    mut qfl: *mut qf_list_T,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    if !(*qfl).qf_multiignore {
        let mut qfprev: *mut qfline_T = (*qfl).qf_last;
        if qfprev.is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        if *(*fields).errmsg != 0 {
            let mut textlen: size_t = strlen((*qfprev).qf_text);
            let mut errlen: size_t = strlen((*fields).errmsg);
            (*qfprev).qf_text = xrealloc(
                (*qfprev).qf_text as *mut ::core::ffi::c_void,
                textlen.wrapping_add(errlen).wrapping_add(2 as size_t),
            ) as *mut ::core::ffi::c_char;
            *(*qfprev).qf_text.offset(textlen as isize) = '\n' as ::core::ffi::c_char;
            strcpy(
                (*qfprev)
                    .qf_text
                    .offset(textlen as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
                (*fields).errmsg,
            );
        }
        if (*qfprev).qf_nr == -1 as ::core::ffi::c_int {
            (*qfprev).qf_nr = (*fields).enr;
        }
        if vim_isprintc((*fields).type_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            && (*qfprev).qf_type == 0
        {
            (*qfprev).qf_type = (*fields).type_0;
        }
        if (*qfprev).qf_lnum == 0 {
            (*qfprev).qf_lnum = (*fields).lnum;
        }
        if (*qfprev).qf_end_lnum == 0 {
            (*qfprev).qf_end_lnum = (*fields).end_lnum;
        }
        if (*qfprev).qf_col == 0 {
            (*qfprev).qf_col = (*fields).col;
            (*qfprev).qf_viscol = (*fields).use_viscol as ::core::ffi::c_char;
        }
        if (*qfprev).qf_end_col == 0 {
            (*qfprev).qf_end_col = (*fields).end_col;
        }
        if (*qfprev).qf_fnum == 0 {
            (*qfprev).qf_fnum = qf_get_fnum(
                qfl,
                (*qfl).qf_directory,
                if *(*fields).namebuf as ::core::ffi::c_int != 0 || !(*qfl).qf_directory.is_null() {
                    (*fields).namebuf
                } else if !(*qfl).qf_currfile.is_null()
                    && (*fields).valid as ::core::ffi::c_int != 0
                {
                    (*qfl).qf_currfile
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                },
            );
        }
    }
    if idx == 'Z' as ::core::ffi::c_int {
        (*qfl).qf_multiignore = false_0 != 0;
        (*qfl).qf_multiline = (*qfl).qf_multiignore;
    }
    line_breakcheck();
    return QF_IGNORE_LINE as ::core::ffi::c_int;
}
unsafe extern "C" fn locstack_queue_delreq(mut qi: *mut qf_info_T) {
    let mut q: *mut qf_delq_T = xmalloc(::core::mem::size_of::<qf_delq_T>()) as *mut qf_delq_T;
    (*q).qi = qi;
    (*q).next = qf_delq_head.get() as *mut qf_delq_S;
    qf_delq_head.set(q);
}
pub unsafe extern "C" fn qf_stack_get_bufnr() -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !(*ql_info.ptr()).is_null() {
        } else {
            __assert_fail(
                b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1770 as ::core::ffi::c_uint,
                b"int qf_stack_get_bufnr(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return (*ql_info.get()).qf_bufnr;
}
unsafe extern "C" fn wipe_qf_buffer(mut qi: *mut qf_info_T) {
    if (*qi).qf_bufnr == INVALID_QFBUFNR {
        return;
    }
    let qfbuf: *mut buf_T = buflist_findnr((*qi).qf_bufnr);
    if !qfbuf.is_null() && (*qfbuf).b_nwindows == 0 as ::core::ffi::c_int {
        let mut buf_was_null: bool = false_0 != 0;
        if (*curwin.get()).w_buffer.is_null() {
            (*curwin.get()).w_buffer = curbuf.get();
            buf_was_null = true_0 != 0;
        }
        close_buffer(
            ::core::ptr::null_mut::<win_T>(),
            qfbuf,
            DOBUF_WIPE as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
        );
        (*qi).qf_bufnr = INVALID_QFBUFNR;
        if buf_was_null {
            (*curwin.get()).w_buffer = ::core::ptr::null_mut::<buf_T>();
        }
    }
}
unsafe extern "C" fn qf_free_list_stack_items(mut qi: *mut qf_info_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*qi).qf_listcount {
        qf_free(qf_get_list(qi, i));
        i += 1;
    }
}
unsafe extern "C" fn qf_free_lists(mut qi: *mut qf_info_T) {
    qf_free_list_stack_items(qi);
    xfree((*qi).qf_lists as *mut ::core::ffi::c_void);
    xfree(qi as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ll_free_all(mut pqi: *mut *mut qf_info_T) {
    let mut qi: *mut qf_info_T = *pqi;
    if qi.is_null() {
        return;
    }
    *pqi = ::core::ptr::null_mut::<qf_info_T>();
    if quickfix_busy.get() > 0 as ::core::ffi::c_int {
        locstack_queue_delreq(qi);
        return;
    }
    (*qi).qf_refcount -= 1;
    if (*qi).qf_refcount < 1 as ::core::ffi::c_int {
        wipe_qf_buffer(qi);
        qf_free_lists(qi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn qf_free_all(mut wp: *mut win_T) {
    let mut qi: *mut qf_info_T = ql_info.get();
    if !wp.is_null() {
        ll_free_all(&raw mut (*wp).w_llist);
        ll_free_all(&raw mut (*wp).w_llist_ref);
    } else if !qi.is_null() {
        qf_free_list_stack_items(qi);
    }
}
unsafe extern "C" fn incr_quickfix_busy() {
    (*quickfix_busy.ptr()) += 1;
}
unsafe extern "C" fn decr_quickfix_busy() {
    (*quickfix_busy.ptr()) -= 1;
    if quickfix_busy.get() == 0 as ::core::ffi::c_int {
        while !(*qf_delq_head.ptr()).is_null() {
            let mut q: *mut qf_delq_T = qf_delq_head.get();
            qf_delq_head.set((*q).next as *mut qf_delq_T);
            ll_free_all(&raw mut (*q).qi);
            xfree(q as *mut ::core::ffi::c_void);
        }
    }
}
unsafe extern "C" fn qf_add_entry(
    mut qfl: *mut qf_list_T,
    mut dir: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
    mut module: *mut ::core::ffi::c_char,
    mut bufnum: ::core::ffi::c_int,
    mut mesg: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut vis_col: ::core::ffi::c_char,
    mut pattern: *mut ::core::ffi::c_char,
    mut nr: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_char,
    mut user_data: *mut typval_T,
    mut valid: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut qfp: *mut qfline_T = xmalloc(::core::mem::size_of::<qfline_T>()) as *mut qfline_T;
    let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if bufnum != 0 as ::core::ffi::c_int {
        buf = buflist_findnr(bufnum);
        (*qfp).qf_fnum = bufnum;
        if !buf.is_null() {
            (*buf).b_has_qf_entry |= if (*qfl).qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                BUF_HAS_QF_ENTRY
            } else {
                BUF_HAS_LL_ENTRY
            };
        }
    } else {
        (*qfp).qf_fnum = qf_get_fnum(qfl, dir, fname);
        buf = buflist_findnr((*qfp).qf_fnum);
    }
    if !fname.is_null() {
        fullname = fix_fname(fname);
    }
    (*qfp).qf_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !buf.is_null() && !(*buf).b_ffname.is_null() && !fullname.is_null() {
        if path_fnamecmp(fullname, (*buf).b_ffname) != 0 as ::core::ffi::c_int {
            p = path_try_shorten_fname(fullname);
            if !p.is_null() {
                (*qfp).qf_fname = xstrdup(p);
            }
        }
    }
    xfree(fullname as *mut ::core::ffi::c_void);
    (*qfp).qf_text = xstrdup(mesg);
    (*qfp).qf_lnum = lnum;
    (*qfp).qf_end_lnum = end_lnum;
    (*qfp).qf_col = col;
    (*qfp).qf_end_col = end_col;
    (*qfp).qf_viscol = vis_col;
    if user_data.is_null()
        || (*user_data).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*qfp).qf_user_data.v_type = VAR_UNKNOWN;
    } else {
        tv_copy(user_data, &raw mut (*qfp).qf_user_data);
        (*qfl).qf_has_user_data = true_0 != 0;
    }
    if pattern.is_null() || *pattern as ::core::ffi::c_int == NUL {
        (*qfp).qf_pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*qfp).qf_pattern = xstrdup(pattern);
    }
    if module.is_null() || *module as ::core::ffi::c_int == NUL {
        (*qfp).qf_module = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*qfp).qf_module = xstrdup(module);
    }
    (*qfp).qf_nr = nr;
    if type_0 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        && !vim_isprintc(type_0 as ::core::ffi::c_int)
    {
        type_0 = 0 as ::core::ffi::c_char;
    }
    (*qfp).qf_type = type_0;
    (*qfp).qf_valid = valid;
    let mut lastp: *mut *mut qfline_T = &raw mut (*qfl).qf_last;
    if qf_list_empty(qfl) {
        (*qfl).qf_start = qfp;
        (*qfl).qf_ptr = qfp;
        (*qfl).qf_index = 0 as ::core::ffi::c_int;
        (*qfp).qf_prev = ::core::ptr::null_mut::<qfline_T>();
    } else {
        '_c2rust_label: {
            if !(*lastp).is_null() {
            } else {
                __assert_fail(
                    b"*lastp\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1998 as ::core::ffi::c_uint,
                    b"int qf_add_entry(qf_list_T *, char *, char *, char *, int, char *, linenr_T, linenr_T, int, int, char, char *, int, char, typval_T *, char)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*qfp).qf_prev = *lastp;
        (**lastp).qf_next = qfp;
    }
    (*qfp).qf_next = ::core::ptr::null_mut::<qfline_T>();
    (*qfp).qf_cleared = false_0 as ::core::ffi::c_char;
    *lastp = qfp;
    (*qfl).qf_count += 1;
    if (*qfl).qf_index == 0 as ::core::ffi::c_int && (*qfp).qf_valid as ::core::ffi::c_int != 0 {
        (*qfl).qf_index = (*qfl).qf_count;
        (*qfl).qf_ptr = qfp;
    }
    return QF_OK as ::core::ffi::c_int;
}
pub unsafe extern "C" fn qf_resize_stack(mut n: ::core::ffi::c_int) {
    '_c2rust_label: {
        if !(*ql_info.ptr()).is_null() {
        } else {
            __assert_fail(
                b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2018 as ::core::ffi::c_uint,
                b"void qf_resize_stack(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    qf_resize_stack_base(ql_info.get(), n);
}
#[no_mangle]
pub unsafe extern "C" fn ll_resize_stack(mut wp: *mut win_T, mut n: ::core::ffi::c_int) {
    if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
        qf_sync_llw_to_win(wp);
    } else {
        qf_sync_win_to_llw(wp);
    }
    let mut qi: *mut qf_info_T = ll_get_or_alloc_list(wp);
    qf_resize_stack_base(qi, n);
}
unsafe extern "C" fn qf_resize_stack_base(mut qi: *mut qf_info_T, mut n: ::core::ffi::c_int) {
    let mut amount_to_rm: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lsz: size_t = ::core::mem::size_of::<qf_list_T>();
    if n == (*qi).qf_maxcount {
        return;
    } else if n < (*qi).qf_maxcount && n < (*qi).qf_listcount {
        amount_to_rm = (*qi).qf_listcount - n;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < amount_to_rm {
            qf_pop_stack(qi, true_0 != 0);
            i += 1;
        }
    }
    let mut new: *mut qf_list_T = xrealloc(
        (*qi).qf_lists as *mut ::core::ffi::c_void,
        lsz.wrapping_mul(n as size_t),
    ) as *mut qf_list_T;
    if n > (*qi).qf_maxcount {
        memset(
            new.offset((*qi).qf_maxcount as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            lsz.wrapping_mul((n - (*qi).qf_maxcount) as size_t),
        );
    }
    (*qi).qf_lists = new;
    (*qi).qf_maxcount = n;
    qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
}
pub unsafe extern "C" fn qf_init_stack() {
    ql_info.set(qf_alloc_stack(
        QFLT_QUICKFIX,
        p_chi.get() as ::core::ffi::c_int,
    ));
}
unsafe extern "C" fn qf_sync_llw_to_win(mut llw: *mut win_T) {
    let mut wp: *mut win_T = qf_find_win_with_loclist((*llw).w_llist_ref);
    if !wp.is_null() {
        (*wp).w_onebuf_opt.wo_lhi = (*llw).w_onebuf_opt.wo_lhi;
    }
}
unsafe extern "C" fn qf_sync_win_to_llw(mut pwp: *mut win_T) {
    let mut llw: *mut qf_info_T = (*pwp).w_llist;
    if !llw.is_null() {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_llist_ref == llw && bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 {
                (*wp).w_onebuf_opt.wo_lhi = (*pwp).w_onebuf_opt.wo_lhi;
                return;
            }
            wp = (*wp).w_next;
        }
    }
}
unsafe extern "C" fn qf_alloc_stack(
    mut qfltype: qfltype_T,
    mut n: ::core::ffi::c_int,
) -> *mut qf_info_T {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    if qfltype as ::core::ffi::c_uint == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        qi = ql_info_actual.ptr();
    } else {
        qi = xcalloc(1 as size_t, ::core::mem::size_of::<qf_info_T>()) as *mut qf_info_T;
        (*qi).qf_refcount += 1;
    }
    (*qi).qfl_type = qfltype;
    (*qi).qf_bufnr = INVALID_QFBUFNR;
    (*qi).qf_lists = qf_alloc_list_stack(n);
    (*qi).qf_maxcount = n;
    return qi;
}
unsafe extern "C" fn qf_alloc_list_stack(mut n: ::core::ffi::c_int) -> *mut qf_list_T {
    return xcalloc(n as size_t, ::core::mem::size_of::<qf_list_T>()) as *mut qf_list_T;
}
unsafe extern "C" fn ll_get_or_alloc_list(mut wp: *mut win_T) -> *mut qf_info_T {
    if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
        return (*wp).w_llist_ref;
    }
    ll_free_all(&raw mut (*wp).w_llist_ref);
    if (*wp).w_llist.is_null() {
        (*wp).w_llist = qf_alloc_stack(
            QFLT_LOCATION,
            (*wp).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
        );
    }
    return (*wp).w_llist;
}
unsafe extern "C" fn qf_cmd_get_stack(
    mut eap: *mut exarg_T,
    mut print_emsg: bool,
) -> *mut qf_info_T {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2156 as ::core::ffi::c_uint,
                b"qf_info_T *qf_cmd_get_stack(exarg_T *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
        qi = if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
            && !(*curwin.get()).w_llist_ref.is_null()
        {
            (*curwin.get()).w_llist_ref
        } else {
            (*curwin.get()).w_llist
        };
        if qi.is_null() {
            if print_emsg {
                emsg(gettext(&raw const e_loclist as *const ::core::ffi::c_char));
            }
            return ::core::ptr::null_mut::<qf_info_T>();
        }
    }
    return qi;
}
unsafe extern "C" fn qf_cmd_get_or_alloc_stack(
    mut eap: *const exarg_T,
    mut pwinp: *mut *mut win_T,
) -> *mut qf_info_T {
    let mut qi: *mut qf_info_T = ql_info.get();
    if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
        qi = ll_get_or_alloc_list(curwin.get());
        *pwinp = curwin.get();
    }
    return qi;
}
unsafe extern "C" fn copy_loclist_entries(
    mut from_qfl: *const qf_list_T,
    mut to_qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut from_qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    i = 1 as ::core::ffi::c_int;
    from_qfp = (*from_qfl).qf_start;
    while !got_int.get() && i <= (*from_qfl).qf_count && !from_qfp.is_null() {
        if qf_add_entry(
            to_qfl,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*from_qfp).qf_module,
            0 as ::core::ffi::c_int,
            (*from_qfp).qf_text,
            (*from_qfp).qf_lnum,
            (*from_qfp).qf_end_lnum,
            (*from_qfp).qf_col,
            (*from_qfp).qf_end_col,
            (*from_qfp).qf_viscol,
            (*from_qfp).qf_pattern,
            (*from_qfp).qf_nr,
            0 as ::core::ffi::c_char,
            &raw mut (*from_qfp).qf_user_data,
            (*from_qfp).qf_valid,
        ) == QF_FAIL as ::core::ffi::c_int
        {
            return FAIL;
        }
        let prevp: *mut qfline_T = (*to_qfl).qf_last;
        (*prevp).qf_fnum = (*from_qfp).qf_fnum;
        (*prevp).qf_type = (*from_qfp).qf_type;
        if (*from_qfl).qf_ptr == from_qfp {
            (*to_qfl).qf_ptr = prevp;
        }
        i += 1;
        from_qfp = (*from_qfp).qf_next;
    }
    return OK;
}
unsafe extern "C" fn copy_loclist(
    mut from_qfl: *mut qf_list_T,
    mut to_qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    (*to_qfl).qfl_type = (*from_qfl).qfl_type;
    (*to_qfl).qf_nonevalid = (*from_qfl).qf_nonevalid;
    (*to_qfl).qf_has_user_data = (*from_qfl).qf_has_user_data;
    (*to_qfl).qf_count = 0 as ::core::ffi::c_int;
    (*to_qfl).qf_index = 0 as ::core::ffi::c_int;
    (*to_qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
    (*to_qfl).qf_last = ::core::ptr::null_mut::<qfline_T>();
    (*to_qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
    if !(*from_qfl).qf_title.is_null() {
        (*to_qfl).qf_title = xstrdup((*from_qfl).qf_title);
    } else {
        (*to_qfl).qf_title = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !(*from_qfl).qf_ctx.is_null() {
        (*to_qfl).qf_ctx =
            xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
        tv_copy((*from_qfl).qf_ctx, (*to_qfl).qf_ctx);
    } else {
        (*to_qfl).qf_ctx = ::core::ptr::null_mut::<typval_T>();
    }
    callback_copy(
        &raw mut (*to_qfl).qf_qftf_cb,
        &raw mut (*from_qfl).qf_qftf_cb,
    );
    if (*from_qfl).qf_count != 0 {
        if copy_loclist_entries(from_qfl, to_qfl) == FAIL {
            return FAIL;
        }
    }
    (*to_qfl).qf_index = (*from_qfl).qf_index;
    last_qf_id.set((*last_qf_id.ptr()).wrapping_add(1));
    (*to_qfl).qf_id = last_qf_id.get();
    (*to_qfl).qf_changedtick = 0 as ::core::ffi::c_int;
    if (*to_qfl).qf_nonevalid {
        (*to_qfl).qf_ptr = (*to_qfl).qf_start;
        (*to_qfl).qf_index = 1 as ::core::ffi::c_int;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn copy_loclist_stack(mut from: *mut win_T, mut to: *mut win_T) {
    let mut qi: *mut qf_info_T = if bt_quickfix((*from).w_buffer) as ::core::ffi::c_int != 0
        && !(*from).w_llist_ref.is_null()
    {
        (*from).w_llist_ref
    } else {
        (*from).w_llist
    };
    if qi.is_null() {
        return;
    }
    (*to).w_llist = qf_alloc_stack(
        QFLT_LOCATION,
        (*from).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
    );
    (*to).w_onebuf_opt.wo_lhi = (*(*to).w_llist).qf_maxcount as OptInt;
    (*(*to).w_llist).qf_listcount = (*qi).qf_listcount;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*qi).qf_listcount {
        (*(*to).w_llist).qf_curlist = idx;
        if copy_loclist(qf_get_list(qi, idx), qf_get_list((*to).w_llist, idx)) == FAIL {
            qf_free_all(to);
            return;
        }
        idx += 1;
    }
    (*(*to).w_llist).qf_curlist = (*qi).qf_curlist;
}
unsafe extern "C" fn qf_get_fnum(
    mut qfl: *mut qf_list_T,
    mut directory: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    if !directory.is_null() && !vim_isAbsName(fname) {
        ptr = concat_fnames(directory, fname, true_0 != 0);
        if !os_path_exists(ptr) {
            xfree(ptr as *mut ::core::ffi::c_void);
            directory = qf_guess_filepath(qfl, fname);
            if !directory.is_null() {
                ptr = concat_fnames(directory, fname, true_0 != 0);
            } else {
                ptr = xstrdup(fname);
            }
        }
        bufname = ptr;
    } else {
        bufname = fname;
    }
    if !(*qf_last_bufname.ptr()).is_null()
        && strcmp(bufname, qf_last_bufname.get()) == 0 as ::core::ffi::c_int
        && bufref_valid(qf_last_bufref.ptr()) as ::core::ffi::c_int != 0
    {
        buf = (*qf_last_bufref.ptr()).br_buf;
        xfree(ptr as *mut ::core::ffi::c_void);
    } else {
        xfree(qf_last_bufname.get() as *mut ::core::ffi::c_void);
        buf = buflist_new(
            bufname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            BLN_NOOPT as ::core::ffi::c_int,
        );
        qf_last_bufname.set(if bufname == ptr {
            bufname
        } else {
            xstrdup(bufname)
        });
        set_bufref(qf_last_bufref.ptr(), buf);
    }
    if buf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*buf).b_has_qf_entry = if (*qfl).qfl_type as ::core::ffi::c_uint
        == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };
    return (*buf).handle as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_push_dir(
    mut dirbuf: *mut ::core::ffi::c_char,
    mut stackptr: *mut *mut dir_stack_T,
    mut is_file_stack: bool,
) -> *mut ::core::ffi::c_char {
    let mut ds_ptr: *mut dir_stack_T = ::core::ptr::null_mut::<dir_stack_T>();
    let mut ds_new: *mut dir_stack_T =
        xmalloc(::core::mem::size_of::<dir_stack_T>()) as *mut dir_stack_T;
    (*ds_new).next = *stackptr;
    *stackptr = ds_new;
    if vim_isAbsName(dirbuf) as ::core::ffi::c_int != 0
        || (**stackptr).next.is_null()
        || is_file_stack as ::core::ffi::c_int != 0
    {
        (**stackptr).dirname = xstrdup(dirbuf);
    } else {
        ds_new = (**stackptr).next;
        (**stackptr).dirname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while !ds_new.is_null() {
            let mut dirname: *mut ::core::ffi::c_char =
                concat_fnames((*ds_new).dirname, dirbuf, true_0 != 0);
            if os_isdir(dirname) {
                xfree((**stackptr).dirname as *mut ::core::ffi::c_void);
                (**stackptr).dirname = dirname;
                break;
            } else {
                xfree(dirname as *mut ::core::ffi::c_void);
                ds_new = (*ds_new).next;
            }
        }
        while (**stackptr).next != ds_new {
            ds_ptr = (**stackptr).next;
            (**stackptr).next = (*(**stackptr).next).next;
            xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
            xfree(ds_ptr as *mut ::core::ffi::c_void);
        }
        if ds_new.is_null() {
            xfree((**stackptr).dirname as *mut ::core::ffi::c_void);
            (**stackptr).dirname = xstrdup(dirbuf);
        }
    }
    if !(**stackptr).dirname.is_null() {
        return (**stackptr).dirname;
    }
    ds_ptr = *stackptr;
    *stackptr = (**stackptr).next;
    xfree(ds_ptr as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn qf_pop_dir(mut stackptr: *mut *mut dir_stack_T) -> *mut ::core::ffi::c_char {
    if !(*stackptr).is_null() {
        let mut ds_ptr: *mut dir_stack_T = *stackptr;
        *stackptr = (**stackptr).next;
        xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
        xfree(ds_ptr as *mut ::core::ffi::c_void);
    }
    return if !(*stackptr).is_null() {
        (**stackptr).dirname
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
unsafe extern "C" fn qf_clean_dir_stack(mut stackptr: *mut *mut dir_stack_T) {
    let mut ds_ptr: *mut dir_stack_T = ::core::ptr::null_mut::<dir_stack_T>();
    loop {
        ds_ptr = *stackptr;
        if ds_ptr.is_null() {
            break;
        }
        *stackptr = (**stackptr).next;
        xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
        xfree(ds_ptr as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn qf_guess_filepath(
    mut qfl: *mut qf_list_T,
    mut filename: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if (*qfl).qf_dir_stack.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut ds_ptr: *mut dir_stack_T = (*(*qfl).qf_dir_stack).next;
    let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while !ds_ptr.is_null() {
        xfree(fullname as *mut ::core::ffi::c_void);
        fullname = concat_fnames((*ds_ptr).dirname, filename, true_0 != 0);
        if os_path_exists(fullname) {
            break;
        }
        ds_ptr = (*ds_ptr).next;
    }
    xfree(fullname as *mut ::core::ffi::c_void);
    while (*(*qfl).qf_dir_stack).next != ds_ptr {
        let mut ds_tmp: *mut dir_stack_T = (*(*qfl).qf_dir_stack).next;
        (*(*qfl).qf_dir_stack).next = (*(*(*qfl).qf_dir_stack).next).next;
        xfree((*ds_tmp).dirname as *mut ::core::ffi::c_void);
        xfree(ds_tmp as *mut ::core::ffi::c_void);
    }
    return if ds_ptr.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        (*ds_ptr).dirname
    };
}
unsafe extern "C" fn qflist_valid(mut wp: *mut win_T, mut qf_id: ::core::ffi::c_uint) -> bool {
    let mut qi: *mut qf_info_T = ql_info.get();
    if !wp.is_null() {
        if !win_valid(wp) {
            return false_0 != 0;
        }
        qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
            && !(*wp).w_llist_ref.is_null()
        {
            (*wp).w_llist_ref
        } else {
            (*wp).w_llist
        };
    }
    if qi.is_null() {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*qi).qf_listcount {
        if (*(*qi).qf_lists.offset(i as isize)).qf_id == qf_id {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
unsafe extern "C" fn is_qf_entry_present(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
) -> bool {
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut i: ::core::ffi::c_int = 0;
    i = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
        if qfp == qf_ptr {
            break;
        }
        i += 1;
        qfp = (*qfp).qf_next;
    }
    if i > (*qfl).qf_count {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn get_next_valid_entry(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    mut qf_index: *mut ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut idx: ::core::ffi::c_int = *qf_index;
    let mut old_qf_fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
    loop {
        if idx == (*qfl).qf_count || (*qf_ptr).qf_next.is_null() {
            return ::core::ptr::null_mut::<qfline_T>();
        }
        idx += 1;
        qf_ptr = (*qf_ptr).qf_next;
        if !(!(*qfl).qf_nonevalid && (*qf_ptr).qf_valid == 0
            || dir == FORWARD_FILE as ::core::ffi::c_int && (*qf_ptr).qf_fnum == old_qf_fnum)
        {
            break;
        }
    }
    *qf_index = idx;
    return qf_ptr;
}
unsafe extern "C" fn get_prev_valid_entry(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    mut qf_index: *mut ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut idx: ::core::ffi::c_int = *qf_index;
    let mut old_qf_fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
    loop {
        if idx == 1 as ::core::ffi::c_int || (*qf_ptr).qf_prev.is_null() {
            return ::core::ptr::null_mut::<qfline_T>();
        }
        idx -= 1;
        qf_ptr = (*qf_ptr).qf_prev;
        if !(!(*qfl).qf_nonevalid && (*qf_ptr).qf_valid == 0
            || dir == BACKWARD_FILE as ::core::ffi::c_int && (*qf_ptr).qf_fnum == old_qf_fnum)
        {
            break;
        }
    }
    *qf_index = idx;
    return qf_ptr;
}
unsafe extern "C" fn get_nth_valid_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
    let mut qf_idx: ::core::ffi::c_int = (*qfl).qf_index;
    let mut err: *const ::core::ffi::c_char = e_no_more_items.get();
    loop {
        let c2rust_fresh22 = errornr;
        errornr = errornr - 1;
        if c2rust_fresh22 == 0 {
            break;
        }
        let mut prev_qf_ptr: *mut qfline_T = qf_ptr;
        let mut prev_index: ::core::ffi::c_int = qf_idx;
        if dir == FORWARD as ::core::ffi::c_int || dir == FORWARD_FILE as ::core::ffi::c_int {
            qf_ptr = get_next_valid_entry(qfl, qf_ptr, &raw mut qf_idx, dir);
        } else {
            qf_ptr = get_prev_valid_entry(qfl, qf_ptr, &raw mut qf_idx, dir);
        }
        if qf_ptr.is_null() {
            qf_ptr = prev_qf_ptr;
            qf_idx = prev_index;
            if !err.is_null() {
                emsg(gettext(err));
                return ::core::ptr::null_mut::<qfline_T>();
            }
            break;
        } else {
            err = ::core::ptr::null::<::core::ffi::c_char>();
        }
    }
    *new_qfidx = qf_idx;
    return qf_ptr;
}
unsafe extern "C" fn get_nth_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
    let mut qf_idx: ::core::ffi::c_int = (*qfl).qf_index;
    while errornr < qf_idx && qf_idx > 1 as ::core::ffi::c_int && !(*qf_ptr).qf_prev.is_null() {
        qf_idx -= 1;
        qf_ptr = (*qf_ptr).qf_prev;
    }
    while errornr > qf_idx && qf_idx < (*qfl).qf_count && !(*qf_ptr).qf_next.is_null() {
        qf_idx += 1;
        qf_ptr = (*qf_ptr).qf_next;
    }
    *new_qfidx = qf_idx;
    return qf_ptr;
}
unsafe extern "C" fn qf_get_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
    let mut qfidx: ::core::ffi::c_int = (*qfl).qf_index;
    if dir != 0 as ::core::ffi::c_int {
        qf_ptr = get_nth_valid_entry(qfl, errornr, dir, &raw mut qfidx);
    } else if errornr != 0 as ::core::ffi::c_int {
        qf_ptr = get_nth_entry(qfl, errornr, &raw mut qfidx);
    }
    *new_qfidx = qfidx;
    return qf_ptr;
}
unsafe extern "C" fn qf_find_help_win() -> *mut win_T {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0
            && !(*wp).w_config.hide
            && (*wp).w_config.focusable as ::core::ffi::c_int != 0
        {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn win_set_loclist(mut wp: *mut win_T, mut qi: *mut qf_info_T) {
    (*wp).w_llist = qi;
    (*qi).qf_refcount += 1;
}
unsafe extern "C" fn jump_to_help_window(
    mut qi: *mut qf_info_T,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    let mut wp: *mut win_T = if (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int
        || newwin as ::core::ffi::c_int != 0
    {
        ::core::ptr::null_mut::<win_T>()
    } else {
        qf_find_help_win()
    };
    if !wp.is_null() && (*(*wp).w_buffer).b_nwindows > 0 as ::core::ffi::c_int {
        win_enter(wp, true_0 != 0);
    } else {
        let mut flags: ::core::ffi::c_int = WSP_HELP as ::core::ffi::c_int;
        if (*cmdmod.ptr()).cmod_split == 0 as ::core::ffi::c_int
            && (*curwin.get()).w_width != Columns.get()
            && (*curwin.get()).w_width < 80 as ::core::ffi::c_int
        {
            flags |= WSP_TOP as ::core::ffi::c_int;
        }
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
            && !newwin
        {
            flags |= WSP_NEWLOC as ::core::ffi::c_int;
        }
        if win_split(0 as ::core::ffi::c_int, flags) == FAIL {
            return FAIL;
        }
        *opened_window = true_0 != 0;
        if ((*curwin.get()).w_height as OptInt) < p_hh.get() {
            win_setheight(p_hh.get() as ::core::ffi::c_int);
        }
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
            && !newwin
        {
            win_set_loclist(curwin.get(), qi);
        }
    }
    restart_edit.set(0 as ::core::ffi::c_int);
    return OK;
}
unsafe extern "C" fn qf_find_win_with_loclist(mut ll: *const qf_info_T) -> *mut win_T {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_llist == ll as *mut qf_info_T && !bt_quickfix((*wp).w_buffer) {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn qf_find_win_with_normal_buf() -> *mut win_T {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if bt_normal((*wp).w_buffer) {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn qf_goto_tabwin_with_file(mut fnum: ::core::ffi::c_int) -> bool {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*(*wp).w_buffer).handle == fnum {
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                return true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
unsafe extern "C" fn qf_open_new_file_win(mut ll_ref: *mut qf_info_T) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = WSP_ABOVE as ::core::ffi::c_int;
    if !ll_ref.is_null() {
        flags |= WSP_NEWLOC as ::core::ffi::c_int;
    }
    if win_split(0 as ::core::ffi::c_int, flags) == FAIL {
        return FAIL;
    }
    p_swb.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    swb_flags.set(0 as ::core::ffi::c_uint);
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    if !ll_ref.is_null() {
        win_set_loclist(curwin.get(), ll_ref);
    }
    return OK;
}
unsafe extern "C" fn qf_goto_win_with_ll_file(
    mut use_win: *mut win_T,
    mut qf_fnum: ::core::ffi::c_int,
    mut ll_ref: *mut qf_info_T,
) {
    let mut win: *mut win_T = use_win;
    if win.is_null() {
        let mut win2: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !win2.is_null() {
            if (*(*win2).w_buffer).handle == qf_fnum {
                win = win2;
                break;
            } else {
                win2 = (*win2).w_next;
            }
        }
        if win.is_null() {
            win = curwin.get();
            while !bt_normal((*win).w_buffer) {
                if (*win).w_prev.is_null() {
                    win = lastwin.get();
                } else {
                    win = (*win).w_prev;
                }
                if win == curwin.get() {
                    break;
                }
            }
        }
    }
    win_goto(win);
    if (*win).w_llist.is_null() && !ll_ref.is_null() {
        win_set_loclist(win, ll_ref);
    }
}
unsafe extern "C" fn qf_goto_win_with_qfl_file(mut qf_fnum: ::core::ffi::c_int) {
    let mut win: *mut win_T = curwin.get();
    let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    while (*(*win).w_buffer).handle != qf_fnum {
        if (*win).w_prev.is_null() {
            win = lastwin.get();
        } else {
            win = (*win).w_prev;
        }
        if bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0 && (*win).w_llist_ref.is_null() {
            if swb_flags.get() & kOptSwbFlagUselast as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
                && win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
            {
                win = prevwin.get();
            } else if !altwin.is_null() {
                win = altwin;
            } else if !(*curwin.get()).w_prev.is_null() {
                win = (*curwin.get()).w_prev;
            } else {
                win = (*curwin.get()).w_next;
            }
            break;
        } else if altwin.is_null()
            && (*win).w_onebuf_opt.wo_pvw == 0
            && (*win).w_onebuf_opt.wo_wfb == 0
            && bt_normal((*win).w_buffer) as ::core::ffi::c_int != 0
        {
            altwin = win;
        }
    }
    win_goto(win);
}
unsafe extern "C" fn qf_jump_to_usable_window(
    mut qf_fnum: ::core::ffi::c_int,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    let mut usable_wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut usable_win: bool = false_0 != 0;
    let mut ll_ref: *mut qf_info_T = if newwin as ::core::ffi::c_int != 0 {
        ::core::ptr::null_mut::<qf_info_T>()
    } else {
        (*curwin.get()).w_llist_ref
    };
    if !ll_ref.is_null() {
        usable_wp = qf_find_win_with_loclist(ll_ref);
        if !usable_wp.is_null() {
            usable_win = true_0 != 0;
        }
    }
    if !usable_win {
        let mut win: *mut win_T = qf_find_win_with_normal_buf();
        if !win.is_null() {
            usable_win = true_0 != 0;
        }
    }
    if !usable_win
        && swb_flags.get() & kOptSwbFlagUsetab as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        usable_win = qf_goto_tabwin_with_file(qf_fnum);
    }
    if firstwin.get() == lastwin.get() && bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
        || !usable_win
        || newwin as ::core::ffi::c_int != 0
    {
        if qf_open_new_file_win(ll_ref) != OK {
            return FAIL;
        }
        *opened_window = true_0 != 0;
    } else if !(*curwin.get()).w_llist_ref.is_null() {
        qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref);
    } else {
        qf_goto_win_with_qfl_file(qf_fnum);
    }
    return OK;
}
unsafe extern "C" fn qf_jump_edit_buffer(
    mut qi: *mut qf_info_T,
    mut qf_ptr: *mut qfline_T,
    mut forceit: ::core::ffi::c_int,
    mut prev_winid: ::core::ffi::c_int,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    let mut old_changetick: ::core::ffi::c_int = (*qfl).qf_changedtick;
    let mut old_qf_curlist: ::core::ffi::c_int = (*qi).qf_curlist;
    let mut qfl_type: qfltype_T = (*qfl).qfl_type;
    let mut retval: ::core::ffi::c_int = OK;
    let mut save_qfid: ::core::ffi::c_uint = (*qfl).qf_id;
    if (*qf_ptr).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        if !can_abandon(curbuf.get(), forceit != 0) {
            no_write_message();
            return FAIL;
        }
        retval = do_ecmd(
            (*qf_ptr).qf_fnum,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            1 as linenr_T,
            ECMD_HIDE as ::core::ffi::c_int + ECMD_SET_HELP as ::core::ffi::c_int,
            if prev_winid == (*curwin.get()).handle {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        );
    } else {
        let mut fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
        if forceit == 0
            && (*curwin.get()).w_onebuf_opt.wo_wfb != 0
            && (*curbuf.get()).handle != fnum
        {
            if (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
                && !bt_quickfix((*prevwin.get()).w_buffer)
            {
                win_goto(prevwin.get());
            }
            if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                if win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == OK {
                    *opened_window = true_0 != 0;
                }
                if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                    emsg(gettext(
                        &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
                    ));
                    retval = FAIL;
                }
            }
        }
        if retval == OK {
            retval = buflist_getfile(
                fnum,
                1 as linenr_T,
                GETF_SETMARK as ::core::ffi::c_int | GETF_SWITCH as ::core::ffi::c_int,
                forceit,
            );
        }
    }
    if qfl_type as ::core::ffi::c_uint == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut wp: *mut win_T = win_id2wp(prev_winid);
        if wp.is_null() && (*curwin.get()).w_llist != qi {
            emsg(gettext(
                b"E924: Current window was closed\0".as_ptr() as *const ::core::ffi::c_char
            ));
            *opened_window = false_0 != 0;
            return QF_ABORT as ::core::ffi::c_int;
        }
    }
    if qfl_type as ::core::ffi::c_uint == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        && !qflist_valid(::core::ptr::null_mut::<win_T>(), save_qfid)
    {
        emsg(gettext(e_current_quickfix_list_was_changed.get()));
        return QF_ABORT as ::core::ffi::c_int;
    }
    if old_qf_curlist != (*qi).qf_curlist
        || old_changetick != (*qfl).qf_changedtick
        || !is_qf_entry_present(qfl, qf_ptr)
    {
        if qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_current_quickfix_list_was_changed.get()));
        } else {
            emsg(gettext(e_current_location_list_was_changed.get()));
        }
        return QF_ABORT as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn qf_jump_goto_line(
    mut qf_lnum: linenr_T,
    mut qf_col: ::core::ffi::c_int,
    mut qf_viscol: ::core::ffi::c_char,
    mut qf_pattern: *mut ::core::ffi::c_char,
) {
    if qf_pattern.is_null() {
        let mut i: linenr_T = qf_lnum;
        if i > 0 as linenr_T {
            i = if i < (*curbuf.get()).b_ml.ml_line_count {
                i
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            (*curwin.get()).w_cursor.lnum = i;
        }
        if qf_col > 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            if qf_viscol as ::core::ffi::c_int == true_0 {
                coladvance(curwin.get(), qf_col as colnr_T - 1 as colnr_T);
            } else {
                (*curwin.get()).w_cursor.col = (qf_col - 1 as ::core::ffi::c_int) as colnr_T;
            }
            (*curwin.get()).w_set_curswant = true_0;
            check_cursor(curwin.get());
        } else {
            beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
    } else {
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
        if do_search(
            ::core::ptr::null_mut::<oparg_T>(),
            '/' as ::core::ffi::c_int,
            '/' as ::core::ffi::c_int,
            qf_pattern,
            strlen(qf_pattern),
            1 as ::core::ffi::c_int,
            SEARCH_KEEP as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) == 0
        {
            (*curwin.get()).w_cursor = save_cursor;
        }
    };
}
unsafe extern "C" fn qf_jump_print_msg(
    mut qi: *mut qf_info_T,
    mut qf_index: ::core::ffi::c_int,
    mut qf_ptr: *mut qfline_T,
    mut old_curbuf: *mut buf_T,
    mut old_lnum: linenr_T,
) {
    let gap: *mut garray_T = qfga_get();
    if msg_scrolled.get() == 0 {
        update_topline(curwin.get());
        if must_redraw.get() != 0 {
            update_screen();
        }
    }
    let mut IObufflen: size_t = vim_snprintf_safelen(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"(%d of %d)%s%s: \0".as_ptr() as *const ::core::ffi::c_char),
        qf_index,
        (*qf_get_curlist(qi)).qf_count,
        if (*qf_ptr).qf_cleared as ::core::ffi::c_int != 0 {
            gettext(b" (line deleted)\0".as_ptr() as *const ::core::ffi::c_char)
                as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        qf_types((*qf_ptr).qf_type as ::core::ffi::c_int, (*qf_ptr).qf_nr),
    );
    ga_concat_len(gap, IObuff.ptr() as *mut ::core::ffi::c_char, IObufflen);
    qf_fmt_text(gap, skipwhite((*qf_ptr).qf_text));
    ga_append(gap, NUL as uint8_t);
    let mut i: linenr_T = msg_scroll.get() as linenr_T;
    if curbuf.get() == old_curbuf && (*curwin.get()).w_cursor.lnum == old_lnum {
        msg_scroll.set(true_0);
    } else if (msg_scrolled.get() == 0 as ::core::ffi::c_int
        || p_ch.get() == 0 as OptInt && msg_scrolled.get() == 1 as ::core::ffi::c_int)
        && shortmess(SHM_OVERALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        msg_scroll.set(false_0);
    }
    msg_ext_set_kind(b"quickfix\0".as_ptr() as *const ::core::ffi::c_char);
    msg_keep(
        (*gap).ga_data as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        true_0 != 0,
        false_0 != 0,
    );
    msg_scroll.set(i as ::core::ffi::c_int);
    qfga_clear();
}
unsafe extern "C" fn qf_jump_open_window(
    mut qi: *mut qf_info_T,
    mut qf_ptr: *mut qfline_T,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    let mut old_changetick: ::core::ffi::c_int = (*qfl).qf_changedtick;
    let mut old_qf_curlist: ::core::ffi::c_int = (*qi).qf_curlist;
    let mut qfl_type: qfltype_T = (*qfl).qfl_type;
    if (*qf_ptr).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (!bt_help((*curwin.get()).w_buffer)
            || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int)
    {
        if jump_to_help_window(qi, newwin, opened_window) == FAIL {
            return FAIL;
        }
    }
    if old_qf_curlist != (*qi).qf_curlist
        || old_changetick != (*qfl).qf_changedtick
        || !is_qf_entry_present(qfl, qf_ptr)
    {
        if qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_current_quickfix_list_was_changed.get()));
        } else {
            emsg(gettext(e_current_location_list_was_changed.get()));
        }
        return QF_ABORT as ::core::ffi::c_int;
    }
    if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0 && !*opened_window {
        if (*qf_ptr).qf_fnum == 0 as ::core::ffi::c_int {
            return NOTDONE;
        }
        if qf_jump_to_usable_window((*qf_ptr).qf_fnum, newwin, opened_window) == FAIL {
            return FAIL;
        }
    }
    if old_qf_curlist != (*qi).qf_curlist
        || old_changetick != (*qfl).qf_changedtick
        || !is_qf_entry_present(qfl, qf_ptr)
    {
        if qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_current_quickfix_list_was_changed.get()));
        } else {
            emsg(gettext(e_current_location_list_was_changed.get()));
        }
        return QF_ABORT as ::core::ffi::c_int;
    }
    return OK;
}
unsafe extern "C" fn qf_jump_to_buffer(
    mut qi: *mut qf_info_T,
    mut qf_index: ::core::ffi::c_int,
    mut qf_ptr: *mut qfline_T,
    mut forceit: ::core::ffi::c_int,
    mut prev_winid: ::core::ffi::c_int,
    mut opened_window: *mut bool,
    mut openfold: ::core::ffi::c_int,
    mut print_message: bool,
) -> ::core::ffi::c_int {
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut retval: ::core::ffi::c_int = OK;
    if (*qf_ptr).qf_fnum != 0 as ::core::ffi::c_int {
        retval = qf_jump_edit_buffer(qi, qf_ptr, forceit, prev_winid, opened_window);
        if retval != OK {
            return retval;
        }
    }
    if curbuf.get() == old_curbuf {
        setpcmark();
    }
    qf_jump_goto_line(
        (*qf_ptr).qf_lnum,
        (*qf_ptr).qf_col,
        (*qf_ptr).qf_viscol,
        (*qf_ptr).qf_pattern,
    );
    if fdo_flags.get() & kOptFdoFlagQuickfix as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && openfold != 0
    {
        foldOpenCursor();
    }
    if print_message {
        qf_jump_print_msg(qi, qf_index, qf_ptr, old_curbuf, old_lnum);
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn qf_jump(
    mut qi: *mut qf_info_T,
    mut dir: ::core::ffi::c_int,
    mut errornr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) {
    qf_jump_newwin(qi, dir, errornr, forceit, false_0 != 0);
}
unsafe extern "C" fn qf_jump_newwin(
    mut qi: *mut qf_info_T,
    mut dir: ::core::ffi::c_int,
    mut errornr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut newwin: bool,
) {
    let mut print_message: bool = false;
    let mut prev_winid: ::core::ffi::c_int = 0;
    let mut opened_window: bool = false;
    let mut retval: ::core::ffi::c_int = 0;
    let mut old_swb: *mut ::core::ffi::c_char = p_swb.get();
    let mut old_swb_flags: ::core::ffi::c_uint = swb_flags.get();
    let old_KeyTyped: bool = KeyTyped.get();
    if qi.is_null() {
        '_c2rust_label: {
            if !(*ql_info.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3240 as ::core::ffi::c_uint,
                    b"void qf_jump_newwin(qf_info_T *, int, int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        qi = ql_info.get();
    }
    if qf_stack_empty(qi) as ::core::ffi::c_int != 0
        || qf_list_empty(qf_get_curlist(qi)) as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_no_errors as *const ::core::ffi::c_char,
        ));
        return;
    }
    incr_quickfix_busy();
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
    let mut old_qf_ptr: *mut qfline_T = qf_ptr;
    let mut qf_index: ::core::ffi::c_int = (*qfl).qf_index;
    let mut old_qf_index: ::core::ffi::c_int = qf_index;
    qf_ptr = qf_get_entry(qfl, errornr, dir, &raw mut qf_index);
    '_theend: {
        if qf_ptr.is_null() {
            qf_ptr = old_qf_ptr;
            qf_index = old_qf_index;
        } else {
            (*qfl).qf_index = qf_index;
            (*qfl).qf_ptr = qf_ptr;
            print_message = !qf_win_pos_update(qi, old_qf_index);
            prev_winid = (*curwin.get()).handle as ::core::ffi::c_int;
            opened_window = false_0 != 0;
            retval = qf_jump_open_window(qi, qf_ptr, newwin, &raw mut opened_window);
            if retval != FAIL {
                if retval == QF_ABORT as ::core::ffi::c_int {
                    qi = ::core::ptr::null_mut::<qf_info_T>();
                    qf_ptr = ::core::ptr::null_mut::<qfline_T>();
                    break '_theend;
                } else if retval == NOTDONE {
                    break '_theend;
                } else {
                    retval = qf_jump_to_buffer(
                        qi,
                        qf_index,
                        qf_ptr,
                        forceit,
                        prev_winid,
                        &raw mut opened_window,
                        old_KeyTyped as ::core::ffi::c_int,
                        print_message,
                    );
                    if retval == QF_ABORT as ::core::ffi::c_int {
                        qi = ::core::ptr::null_mut::<qf_info_T>();
                        qf_ptr = ::core::ptr::null_mut::<qfline_T>();
                    }
                    if retval != OK {
                        if opened_window {
                            win_close(curwin.get(), true_0 != 0, false_0 != 0);
                        }
                        if !(!qf_ptr.is_null() && (*qf_ptr).qf_fnum != 0 as ::core::ffi::c_int) {
                            break '_theend;
                        }
                    } else {
                        break '_theend;
                    }
                }
            }
            qf_ptr = old_qf_ptr;
            qf_index = old_qf_index;
        }
    }
    if !qi.is_null() {
        (*qfl).qf_ptr = qf_ptr;
        (*qfl).qf_index = qf_index;
    }
    if p_swb.get() != old_swb
        && p_swb.get() == empty_string_option.ptr() as *mut ::core::ffi::c_char
    {
        p_swb.set(old_swb);
        swb_flags.set(old_swb_flags);
    }
    decr_quickfix_busy();
}
static qfFile_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static qfSep_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static qfLine_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn qf_list_entry(
    mut qfp: *mut qfline_T,
    mut qf_idx: ::core::ffi::c_int,
    mut cursel: bool,
) {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*qfp).qf_module.is_null() && *(*qfp).qf_module as ::core::ffi::c_int != NUL {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"%2d %s\0".as_ptr() as *const ::core::ffi::c_char,
            qf_idx,
            (*qfp).qf_module,
        );
    } else {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if (*qfp).qf_fnum != 0 as ::core::ffi::c_int && {
            buf = buflist_findnr((*qfp).qf_fnum);
            !buf.is_null()
        } {
            fname = if (*qfp).qf_fname.is_null() {
                (*buf).b_fname
            } else {
                (*qfp).qf_fname
            };
            if (*qfp).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                fname = path_tail(fname);
            }
        }
        if fname.is_null() {
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%2d\0".as_ptr() as *const ::core::ffi::c_char,
                qf_idx,
            );
        } else {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%2d %s\0".as_ptr() as *const ::core::ffi::c_char,
                qf_idx,
                fname,
            );
        }
    }
    let mut filter_entry: bool = true_0 != 0;
    if !(*qfp).qf_module.is_null() && *(*qfp).qf_module as ::core::ffi::c_int != NUL {
        filter_entry = filter_entry as ::core::ffi::c_int
            & message_filtered((*qfp).qf_module) as ::core::ffi::c_int
            != 0;
    }
    if filter_entry as ::core::ffi::c_int != 0 && !fname.is_null() {
        filter_entry =
            filter_entry as ::core::ffi::c_int & message_filtered(fname) as ::core::ffi::c_int != 0;
    }
    if filter_entry as ::core::ffi::c_int != 0 && !(*qfp).qf_pattern.is_null() {
        filter_entry = filter_entry as ::core::ffi::c_int
            & message_filtered((*qfp).qf_pattern) as ::core::ffi::c_int
            != 0;
    }
    if filter_entry {
        filter_entry = filter_entry as ::core::ffi::c_int
            & message_filtered((*qfp).qf_text) as ::core::ffi::c_int
            != 0;
    }
    if filter_entry {
        return;
    }
    if msg_col.get() > 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    msg_outtrans(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        if cursel as ::core::ffi::c_int != 0 {
            HLF_QFL as ::core::ffi::c_int
        } else {
            qfFile_hl_id.get()
        },
        false_0 != 0,
    );
    if (*qfp).qf_lnum != 0 as linenr_T {
        msg_puts_hl(
            b":\0".as_ptr() as *const ::core::ffi::c_char,
            qfSep_hl_id.get(),
            false_0 != 0,
        );
    }
    let mut gap: *mut garray_T = qfga_get();
    if (*qfp).qf_lnum != 0 as linenr_T {
        qf_range_text(gap, qfp);
    }
    ga_concat(
        gap,
        qf_types((*qfp).qf_type as ::core::ffi::c_int, (*qfp).qf_nr),
    );
    ga_append(gap, NUL as uint8_t);
    if *((*gap).ga_data as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL {
        msg_puts_hl(
            (*gap).ga_data as *const ::core::ffi::c_char,
            qfLine_hl_id.get(),
            false_0 != 0,
        );
    }
    msg_puts_hl(
        b":\0".as_ptr() as *const ::core::ffi::c_char,
        qfSep_hl_id.get(),
        false_0 != 0,
    );
    if !(*qfp).qf_pattern.is_null() {
        gap = qfga_get();
        qf_fmt_text(gap, (*qfp).qf_pattern);
        ga_append(gap, NUL as uint8_t);
        msg_puts((*gap).ga_data as *const ::core::ffi::c_char);
        msg_puts_hl(
            b":\0".as_ptr() as *const ::core::ffi::c_char,
            qfSep_hl_id.get(),
            false_0 != 0,
        );
    }
    msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
    gap = qfga_get();
    qf_fmt_text(
        gap,
        if !fname.is_null() || (*qfp).qf_lnum != 0 as linenr_T {
            skipwhite((*qfp).qf_text)
        } else {
            (*qfp).qf_text
        },
    );
    ga_append(gap, NUL as uint8_t);
    msg_prt_line((*gap).ga_data as *const ::core::ffi::c_char, false_0 != 0);
}
pub unsafe extern "C" fn qf_list(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut all: ::core::ffi::c_int = (*eap).forceit;
    let mut qi: *mut qf_info_T = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    if qf_stack_empty(qi) as ::core::ffi::c_int != 0
        || qf_list_empty(qf_get_curlist(qi)) as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_no_errors as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut plus: bool = false_0 != 0;
    if *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
        arg = arg.offset(1);
        plus = true_0 != 0;
    }
    let mut idx1: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut idx2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if get_list_range(&raw mut arg, &raw mut idx1, &raw mut idx2) == 0
        || *arg as ::core::ffi::c_int != NUL
    {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    let mut i: ::core::ffi::c_int = 0;
    if plus {
        i = (*qfl).qf_index;
        idx2 = i + idx1;
        idx1 = i;
    } else {
        i = (*qfl).qf_count;
        if idx1 < 0 as ::core::ffi::c_int {
            idx1 = if -idx1 > i {
                0 as ::core::ffi::c_int
            } else {
                idx1 + i + 1 as ::core::ffi::c_int
            };
        }
        if idx2 < 0 as ::core::ffi::c_int {
            idx2 = if -idx2 > i {
                0 as ::core::ffi::c_int
            } else {
                idx2 + i + 1 as ::core::ffi::c_int
            };
        }
    }
    shorten_fnames(false_0);
    qfFile_hl_id.set(syn_name2id(
        b"qfFileName\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if qfFile_hl_id.get() == 0 as ::core::ffi::c_int {
        qfFile_hl_id.set(HLF_D as ::core::ffi::c_int);
    }
    qfSep_hl_id.set(syn_name2id(
        b"qfSeparator\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if qfSep_hl_id.get() == 0 as ::core::ffi::c_int {
        qfSep_hl_id.set(HLF_D as ::core::ffi::c_int);
    }
    qfLine_hl_id.set(syn_name2id(
        b"qfLineNr\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if qfLine_hl_id.get() == 0 as ::core::ffi::c_int {
        qfLine_hl_id.set(HLF_N as ::core::ffi::c_int);
    }
    if (*qfl).qf_nonevalid {
        all = true_0;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    i = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
        if ((*qfp).qf_valid as ::core::ffi::c_int != 0 || all != 0) && idx1 <= i && i <= idx2 {
            qf_list_entry(qfp, i, i == (*qfl).qf_index);
        }
        os_breakcheck();
        i += 1;
        qfp = (*qfp).qf_next;
    }
    qfga_clear();
}
unsafe extern "C" fn qf_fmt_text(mut gap: *mut garray_T, mut text: *const ::core::ffi::c_char) {
    let mut p: *const ::core::ffi::c_char = text;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            ga_append(gap, ' ' as uint8_t);
            loop {
                p = p.offset(1);
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
                if !ascii_iswhite(*p as ::core::ffi::c_int)
                    && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                {
                    break;
                }
            }
        } else {
            let c2rust_fresh0 = p;
            p = p.offset(1);
            ga_append(gap, *c2rust_fresh0 as uint8_t);
        }
    }
}
unsafe extern "C" fn qf_range_text(mut gap: *mut garray_T, mut qfp: *const qfline_T) {
    let mut buf: String_0 = String_0 {
        data: IObuff.ptr() as *mut ::core::ffi::c_char,
        size: 0 as size_t,
    };
    buf.size = vim_snprintf_safelen(
        buf.data,
        IOSIZE as size_t,
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        (*qfp).qf_lnum,
    );
    if (*qfp).qf_end_lnum > 0 as linenr_T && (*qfp).qf_lnum != (*qfp).qf_end_lnum {
        buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
            buf.data.offset(buf.size as isize),
            (IOSIZE as size_t).wrapping_sub(buf.size),
            b"-%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*qfp).qf_end_lnum,
        ));
    }
    if (*qfp).qf_col > 0 as ::core::ffi::c_int {
        buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
            buf.data.offset(buf.size as isize),
            (IOSIZE as size_t).wrapping_sub(buf.size),
            b" col %d\0".as_ptr() as *const ::core::ffi::c_char,
            (*qfp).qf_col,
        ));
        if (*qfp).qf_end_col > 0 as ::core::ffi::c_int && (*qfp).qf_col != (*qfp).qf_end_col {
            buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
                buf.data.offset(buf.size as isize),
                (IOSIZE as size_t).wrapping_sub(buf.size),
                b"-%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*qfp).qf_end_col,
            ));
        }
    }
    ga_concat_len(gap, buf.data, buf.size);
}
unsafe extern "C" fn qf_msg(
    mut qi: *mut qf_info_T,
    mut which: ::core::ffi::c_int,
    mut lead: *mut ::core::ffi::c_char,
) {
    let mut title: *mut ::core::ffi::c_char = (*(*qi).qf_lists.offset(which as isize)).qf_title;
    let mut count: ::core::ffi::c_int = (*(*qi).qf_lists.offset(which as isize)).qf_count;
    let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
    vim_snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"%serror list %d of %d; %d errors \0".as_ptr() as *const ::core::ffi::c_char),
        lead,
        which + 1 as ::core::ffi::c_int,
        (*qi).qf_listcount,
        count,
    );
    if !title.is_null() {
        let mut len: size_t = strlen(&raw mut buf as *mut ::core::ffi::c_char);
        if len < 34 as size_t {
            memset(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                    as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                (34 as size_t).wrapping_sub(len),
            );
            buf[34 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        xstrlcat(
            &raw mut buf as *mut ::core::ffi::c_char,
            title,
            IOSIZE as size_t,
        );
    }
    trunc_string(
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut buf as *mut ::core::ffi::c_char,
        Columns.get() - 1 as ::core::ffi::c_int,
        IOSIZE,
    );
    msg(
        &raw mut buf as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn qf_age(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut count: ::core::ffi::c_int = if (*eap).addr_count != 0 as ::core::ffi::c_int {
        (*eap).line2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    loop {
        let c2rust_fresh23 = count;
        count = count - 1;
        if c2rust_fresh23 == 0 {
            break;
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_colder as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lolder as ::core::ffi::c_int
        {
            if (*qi).qf_curlist == 0 as ::core::ffi::c_int {
                emsg(gettext(
                    b"E380: At bottom of quickfix stack\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break;
            } else {
                (*qi).qf_curlist -= 1;
            }
        } else if (*qi).qf_curlist >= (*qi).qf_listcount - 1 as ::core::ffi::c_int {
            emsg(gettext(
                b"E381: At top of quickfix stack\0".as_ptr() as *const ::core::ffi::c_char
            ));
            break;
        } else {
            (*qi).qf_curlist += 1;
        }
    }
    qf_msg(
        qi,
        (*qi).qf_curlist,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
}
pub unsafe extern "C" fn qf_history(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = qf_cmd_get_stack(eap, false_0 != 0);
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        if qi.is_null() {
            emsg(gettext(&raw const e_loclist as *const ::core::ffi::c_char));
            return;
        }
        if (*eap).line2 > 0 as linenr_T && (*eap).line2 <= (*qi).qf_listcount as linenr_T {
            (*qi).qf_curlist = ((*eap).line2 - 1 as linenr_T) as ::core::ffi::c_int;
            qf_msg(
                qi,
                (*qi).qf_curlist,
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
        } else {
            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
        }
        return;
    }
    if qf_stack_empty(qi) {
        msg(
            gettext(b"No entries\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    } else {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*qi).qf_listcount {
            qf_msg(
                qi,
                i,
                (if i == (*qi).qf_curlist {
                    b"> \0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"  \0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            );
            i += 1;
        }
    };
}
unsafe extern "C" fn qf_free_items(mut qfl: *mut qf_list_T) {
    let mut stop: bool = false_0 != 0;
    while (*qfl).qf_count != 0 && !(*qfl).qf_start.is_null() {
        let mut qfp: *mut qfline_T = (*qfl).qf_start;
        let mut qfpnext: *mut qfline_T = (*qfp).qf_next;
        if !stop {
            xfree((*qfp).qf_fname as *mut ::core::ffi::c_void);
            xfree((*qfp).qf_module as *mut ::core::ffi::c_void);
            xfree((*qfp).qf_text as *mut ::core::ffi::c_void);
            xfree((*qfp).qf_pattern as *mut ::core::ffi::c_void);
            tv_clear(&raw mut (*qfp).qf_user_data);
            stop = qfp == qfpnext;
            xfree(qfp as *mut ::core::ffi::c_void);
            if stop {
                (*qfl).qf_count = 1 as ::core::ffi::c_int;
            } else {
                (*qfl).qf_start = qfpnext;
            }
        }
        (*qfl).qf_count -= 1;
    }
    (*qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
    (*qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
    (*qfl).qf_index = 0 as ::core::ffi::c_int;
    (*qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
    (*qfl).qf_last = ::core::ptr::null_mut::<qfline_T>();
    (*qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
    (*qfl).qf_nonevalid = true_0 != 0;
    qf_clean_dir_stack(&raw mut (*qfl).qf_dir_stack);
    (*qfl).qf_directory = ::core::ptr::null_mut::<::core::ffi::c_char>();
    qf_clean_dir_stack(&raw mut (*qfl).qf_file_stack);
    (*qfl).qf_currfile = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*qfl).qf_multiline = false_0 != 0;
    (*qfl).qf_multiignore = false_0 != 0;
    (*qfl).qf_multiscan = false_0 != 0;
}
unsafe extern "C" fn qf_free(mut qfl: *mut qf_list_T) {
    qf_free_items(qfl);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*qfl).qf_title as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    tv_free((*qfl).qf_ctx);
    (*qfl).qf_ctx = ::core::ptr::null_mut::<typval_T>();
    callback_free(&raw mut (*qfl).qf_qftf_cb);
    (*qfl).qf_id = 0 as ::core::ffi::c_uint;
    (*qfl).qf_changedtick = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn qf_mark_adjust(
    mut buf: *mut buf_T,
    mut wp: *mut win_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) -> bool {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3686 as ::core::ffi::c_uint,
                b"_Bool qf_mark_adjust(buf_T *, win_T *, linenr_T, linenr_T, linenr_T, linenr_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut buf_has_flag: ::core::ffi::c_int = if wp.is_null() {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };
    if (*buf).b_has_qf_entry & buf_has_flag == 0 {
        return false_0 != 0;
    }
    if !wp.is_null() {
        if (*wp).w_llist.is_null() {
            return false_0 != 0;
        }
        qi = (*wp).w_llist;
    }
    let mut i: ::core::ffi::c_int = 0;
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut found_one: bool = false_0 != 0;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*qi).qf_listcount {
        let mut qfl: *mut qf_list_T = qf_get_list(qi, idx);
        if !qf_list_empty(qfl) {
            i = 1 as ::core::ffi::c_int;
            qfp = (*qfl).qf_start;
            while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
                if (*qfp).qf_fnum == (*buf).handle {
                    found_one = true_0 != 0;
                    if (*qfp).qf_lnum >= line1 && (*qfp).qf_lnum <= line2 {
                        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                            (*qfp).qf_cleared = true_0 as ::core::ffi::c_char;
                        } else {
                            (*qfp).qf_lnum += amount;
                        }
                    } else if amount_after != 0 && (*qfp).qf_lnum > line2 {
                        (*qfp).qf_lnum += amount_after;
                    }
                }
                i += 1;
                qfp = (*qfp).qf_next;
            }
        }
        idx += 1;
    }
    return found_one;
}
unsafe extern "C" fn qf_types(
    mut c: ::core::ffi::c_int,
    mut nr: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static cc: GlobalCell<[::core::ffi::c_char; 3]> = GlobalCell::new([0; 3]);
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if c == 'W' as ::core::ffi::c_int || c == 'w' as ::core::ffi::c_int {
        p = b" warning\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if c == 'I' as ::core::ffi::c_int || c == 'i' as ::core::ffi::c_int {
        p = b" info\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if c == 'N' as ::core::ffi::c_int || c == 'n' as ::core::ffi::c_int {
        p = b" note\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if c == 'E' as ::core::ffi::c_int
        || c == 'e' as ::core::ffi::c_int
        || c == 0 as ::core::ffi::c_int && nr > 0 as ::core::ffi::c_int
    {
        p = b" error\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if c == 0 as ::core::ffi::c_int || c == 1 as ::core::ffi::c_int {
        p = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        (*cc.ptr())[0 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
        (*cc.ptr())[1 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        (*cc.ptr())[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        p = cc.ptr() as *mut ::core::ffi::c_char;
    }
    if nr <= 0 as ::core::ffi::c_int {
        return p;
    }
    static buf: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([0; 20]);
    snprintf(
        buf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
        b"%s %3d\0".as_ptr() as *const ::core::ffi::c_char,
        p,
        nr,
    );
    return buf.ptr() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn qf_view_result(mut split: bool) {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3776 as ::core::ffi::c_uint,
                b"void qf_view_result(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
        && !(*curwin.get()).w_llist_ref.is_null()
    {
        qi = if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
            && !(*curwin.get()).w_llist_ref.is_null()
        {
            (*curwin.get()).w_llist_ref
        } else {
            (*curwin.get()).w_llist
        };
    }
    if qf_list_empty(qf_get_curlist(qi)) {
        emsg(gettext(
            &raw const e_no_errors as *const ::core::ffi::c_char,
        ));
        return;
    }
    if split {
        qf_jump_newwin(
            qi,
            0 as ::core::ffi::c_int,
            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int,
            false_0,
            true_0 != 0,
        );
        do_cmdline_cmd(b"clearjumps\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    do_cmdline_cmd(
        if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
            && !(*curwin.get()).w_llist_ref.is_null()
        {
            b".ll\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b".cc\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
}
pub unsafe extern "C" fn ex_cwindow(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    let mut win: *mut win_T = qf_find_win(qi);
    if qf_stack_empty(qi) as ::core::ffi::c_int != 0
        || (*qfl).qf_nonevalid as ::core::ffi::c_int != 0
        || qf_list_empty(qfl) as ::core::ffi::c_int != 0
    {
        if !win.is_null() {
            ex_cclose(eap);
        }
    } else if win.is_null() {
        ex_copen(eap);
    }
}
pub unsafe extern "C" fn ex_cclose(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, false_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut win: *mut win_T = qf_find_win(qi);
    if !win.is_null() {
        win_close(win, false_0 != 0, false_0 != 0);
    }
}
unsafe extern "C" fn qf_goto_cwindow(
    mut qi: *const qf_info_T,
    mut resize: bool,
    mut sz: ::core::ffi::c_int,
    mut vertsplit: bool,
) -> ::core::ffi::c_int {
    let win: *mut win_T = qf_find_win(qi);
    if win.is_null() {
        return FAIL;
    }
    win_goto(win);
    if resize {
        if vertsplit {
            if sz != (*win).w_width {
                win_setwidth(sz);
            }
        } else if sz != (*win).w_height
            && (*win).w_height + (*win).w_hsep_height + (*win).w_status_height + tabline_height()
                < cmdline_row.get()
        {
            win_setheight(sz);
        }
    }
    return OK;
}
unsafe extern "C" fn qf_set_cwindow_options() {
    set_option_value_give_err(
        kOptSwapfile,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"quickfix\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(
        kOptBufhidden,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_diff = false_0;
    set_option_value_give_err(
        kOptFoldmethod,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"manual\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn qf_open_new_cwindow(
    mut qi: *mut qf_info_T,
    mut height: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut oldwin: *mut win_T = curwin.get();
    let prevtab: *const tabpage_T = curtab.get();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let qf_buf: *const buf_T = qf_find_buf(qi);
    let win: *mut win_T = curwin.get();
    if (*cmdmod.ptr()).cmod_split == 0 as ::core::ffi::c_int {
        flags = if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    flags |= WSP_NEWLOC as ::core::ffi::c_int;
    if (*qi).qfl_type as ::core::ffi::c_uint
        == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_QUICKFIX as ::core::ffi::c_int;
    }
    if win_split(height, flags) == FAIL {
        return FAIL;
    }
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    if (*qi).qfl_type as ::core::ffi::c_uint
        == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*curwin.get()).w_llist_ref = qi;
        (*qi).qf_refcount += 1;
    }
    if oldwin != curwin.get() {
        oldwin = ::core::ptr::null_mut::<win_T>();
    }
    if !qf_buf.is_null() {
        if do_ecmd(
            (*qf_buf).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_ONE as ::core::ffi::c_int as linenr_T,
            ECMD_HIDE as ::core::ffi::c_int
                + ECMD_OLDBUF as ::core::ffi::c_int
                + ECMD_NOWINENTER as ::core::ffi::c_int,
            oldwin,
        ) == FAIL
        {
            return FAIL;
        }
    } else {
        if do_ecmd(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_ONE as ::core::ffi::c_int as linenr_T,
            ECMD_HIDE as ::core::ffi::c_int + ECMD_NOWINENTER as ::core::ffi::c_int,
            oldwin,
        ) == FAIL
        {
            return FAIL;
        }
        (*qi).qf_bufnr = (*curbuf.get()).handle as ::core::ffi::c_int;
    }
    if !bt_quickfix(curbuf.get()) {
        qf_set_cwindow_options();
    }
    if curtab.get() == prevtab as *mut tabpage_T && (*curwin.get()).w_width == Columns.get() {
        win_setheight(height);
    }
    (*curwin.get()).w_onebuf_opt.wo_wfh = true_0;
    if win_valid(win) {
        prevwin.set(win);
    }
    return OK;
}
unsafe extern "C" fn qf_set_title_var(mut qfl: *mut qf_list_T) {
    if !(*qfl).qf_title.is_null() {
        set_internal_string_var(
            b"w:quickfix_title\0".as_ptr() as *const ::core::ffi::c_char,
            (*qfl).qf_title,
        );
    }
}
pub unsafe extern "C" fn ex_copen(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    incr_quickfix_busy();
    let mut height: ::core::ffi::c_int = 0;
    if (*eap).addr_count != 0 as ::core::ffi::c_int {
        height = (*eap).line2 as ::core::ffi::c_int;
    } else {
        height = QF_WINHEIGHT as ::core::ffi::c_int;
    }
    reset_VIsual_and_resel();
    let mut status: ::core::ffi::c_int = FAIL;
    if (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int {
        status = qf_goto_cwindow(
            qi,
            (*eap).addr_count != 0 as ::core::ffi::c_int,
            height,
            (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int != 0,
        );
    }
    if status == FAIL {
        if qf_open_new_cwindow(qi, height) == FAIL {
            decr_quickfix_busy();
            return;
        }
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    qf_set_title_var(qfl);
    let mut lnum: ::core::ffi::c_int = (*qfl).qf_index;
    qf_fill_buffer(
        qfl,
        curbuf.get(),
        ::core::ptr::null_mut::<qfline_T>(),
        (*curwin.get()).handle as ::core::ffi::c_int,
    );
    decr_quickfix_busy();
    (*curwin.get()).w_cursor.lnum = lnum as linenr_T;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    check_cursor(curwin.get());
    update_topline(curwin.get());
}
unsafe extern "C" fn qf_win_goto(mut win: *mut win_T, mut lnum: linenr_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    curwin.set(win);
    curbuf.set((*win).w_buffer);
    (*curwin.get()).w_cursor.lnum = lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
    update_topline(curwin.get());
    redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    (*curwin.get()).w_redr_status = true_0 != 0;
    curwin.set(old_curwin);
    curbuf.set((*curwin.get()).w_buffer);
}
pub unsafe extern "C" fn ex_cbottom(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut win: *mut win_T = qf_find_win(qi);
    if !win.is_null() && (*win).w_cursor.lnum != (*(*win).w_buffer).b_ml.ml_line_count {
        qf_win_goto(win, (*(*win).w_buffer).b_ml.ml_line_count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn qf_current_entry(mut wp: *mut win_T) -> linenr_T {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4056 as ::core::ffi::c_uint,
                b"linenr_T qf_current_entry(win_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
        qi = (*wp).w_llist_ref;
    }
    return (*qf_get_curlist(qi)).qf_index as linenr_T;
}
unsafe extern "C" fn qf_win_pos_update(
    mut qi: *mut qf_info_T,
    mut old_qf_index: ::core::ffi::c_int,
) -> bool {
    let mut qf_index: ::core::ffi::c_int = (*qf_get_curlist(qi)).qf_index;
    let mut win: *mut win_T = qf_find_win(qi);
    if !win.is_null()
        && qf_index as linenr_T <= (*(*win).w_buffer).b_ml.ml_line_count
        && old_qf_index != qf_index
    {
        (*win).w_redraw_top = (if old_qf_index < qf_index {
            old_qf_index
        } else {
            qf_index
        }) as linenr_T;
        (*win).w_redraw_bot = (if old_qf_index > qf_index {
            old_qf_index
        } else {
            qf_index
        }) as linenr_T;
        qf_win_goto(win, qf_index as linenr_T);
    }
    return !win.is_null();
}
unsafe extern "C" fn is_qf_win(
    mut win: *const win_T,
    mut qi: *const qf_info_T,
) -> ::core::ffi::c_int {
    if buf_valid((*win).w_buffer) as ::core::ffi::c_int != 0
        && bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
    {
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*win).w_llist_ref.is_null()
            || (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*win).w_llist_ref == qi as *mut qf_info_T
        {
            return true_0;
        }
    }
    return false_0;
}
unsafe extern "C" fn qf_find_win(mut qi: *const qf_info_T) -> *mut win_T {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if is_qf_win(win, qi) != 0 {
            return win;
        }
        win = (*win).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn qf_find_buf(mut qi: *mut qf_info_T) -> *mut buf_T {
    if (*qi).qf_bufnr != INVALID_QFBUFNR {
        let qfbuf: *mut buf_T = buflist_findnr((*qi).qf_bufnr);
        if !qfbuf.is_null() {
            return qfbuf;
        }
        (*qi).qf_bufnr = INVALID_QFBUFNR;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if is_qf_win(win, qi) != 0 {
                return (*win).w_buffer;
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null_mut::<buf_T>();
}
pub unsafe extern "C" fn did_set_quickfixtextfunc(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if option_set_callback_func(p_qftf.get(), qftf_cb.ptr()) == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn qf_update_win_titlevar(mut qi: *mut qf_info_T) {
    let qfl: *mut qf_list_T = qf_get_curlist(qi);
    let save_curwin: *mut win_T = curwin.get();
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if is_qf_win(win, qi) != 0 {
                curwin.set(win);
                qf_set_title_var(qfl);
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    curwin.set(save_curwin);
}
unsafe extern "C" fn qf_update_buffer(mut qi: *mut qf_info_T, mut old_last: *mut qfline_T) {
    let mut buf: *mut buf_T = qf_find_buf(qi);
    if buf.is_null() {
        return;
    }
    let mut old_line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut old_endcol: colnr_T = ml_get_buf_len(buf, old_line_count);
    let mut old_bytecount: bcount_t =
        get_region_bytecount(buf, 1 as linenr_T, old_line_count, 0 as colnr_T, old_endcol);
    let mut qf_winid_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*qi).qfl_type as ::core::ffi::c_uint
        == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*curwin.get()).w_llist == qi {
            win = curwin.get();
        } else {
            win = qf_find_win_with_loclist(qi);
            if win.is_null() {
                win = qf_find_win(qi);
            }
            if win.is_null() {
                return;
            }
        }
        qf_winid_0 = (*win).handle;
    }
    incr_quickfix_busy();
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    if old_last.is_null() {
        aucmd_prepbuf(&raw mut aco, buf);
    }
    qf_update_win_titlevar(qi);
    qf_fill_buffer(qf_get_curlist(qi), buf, old_last, qf_winid_0);
    let mut new_line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut new_endcol: colnr_T = ml_get_buf_len(buf, new_line_count);
    let mut new_byte_count: bcount_t = 0 as bcount_t;
    let mut delta: linenr_T = new_line_count - old_line_count;
    if old_last.is_null() {
        new_byte_count =
            get_region_bytecount(buf, 1 as linenr_T, new_line_count, 0 as colnr_T, new_endcol);
        extmark_splice(
            buf,
            0 as ::core::ffi::c_int,
            0 as colnr_T,
            old_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as colnr_T,
            old_bytecount,
            new_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            new_endcol,
            new_byte_count,
            kExtmarkNoUndo,
        );
        changed_lines(
            buf,
            1 as linenr_T,
            0 as colnr_T,
            if old_line_count > 0 as linenr_T {
                old_line_count + 1 as linenr_T
            } else {
                1 as linenr_T
            },
            delta,
            true_0 != 0,
        );
    } else if delta > 0 as linenr_T {
        let mut start_lnum: linenr_T = old_line_count + 1 as linenr_T;
        new_byte_count =
            get_region_bytecount(buf, start_lnum, new_line_count, 0 as colnr_T, new_endcol);
        extmark_splice(
            buf,
            old_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            old_endcol,
            0 as ::core::ffi::c_int,
            0 as colnr_T,
            0 as bcount_t,
            delta as ::core::ffi::c_int,
            new_endcol,
            new_byte_count,
            kExtmarkNoUndo,
        );
        changed_lines(
            buf,
            start_lnum,
            0 as colnr_T,
            start_lnum,
            delta,
            true_0 != 0,
        );
    }
    (*buf).b_changed = false_0;
    if old_last.is_null() {
        qf_win_pos_update(qi, 0 as ::core::ffi::c_int);
        aucmd_restbuf(&raw mut aco);
    }
    win = qf_find_win(qi);
    if !win.is_null() && old_line_count < (*win).w_botline {
        redraw_buf_later(buf, UPD_NOT_VALID as ::core::ffi::c_int);
    }
    decr_quickfix_busy();
}
unsafe extern "C" fn qf_buf_add_line(
    mut _qfl: *mut qf_list_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut qfp: *const qfline_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut qftf_str: *mut ::core::ffi::c_char,
    mut first_bufline: bool,
) -> ::core::ffi::c_int {
    let mut gap: *mut garray_T = qfga_get();
    if !qftf_str.is_null() && *qftf_str as ::core::ffi::c_int != NUL {
        ga_concat(gap, qftf_str);
    } else {
        let mut errbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if !(*qfp).qf_module.is_null() {
            ga_concat(gap, (*qfp).qf_module);
        } else if (*qfp).qf_fnum != 0 as ::core::ffi::c_int
            && {
                errbuf = buflist_findnr((*qfp).qf_fnum);
                !errbuf.is_null()
            }
            && !(*errbuf).b_fname.is_null()
        {
            if (*qfp).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                ga_concat(gap, path_tail((*errbuf).b_fname));
            } else {
                if first_bufline as ::core::ffi::c_int != 0
                    && ((*errbuf).b_sfname.is_null()
                        || path_is_absolute((*errbuf).b_sfname) as ::core::ffi::c_int != 0)
                {
                    if *dirname as ::core::ffi::c_int == NUL {
                        os_dirname(dirname, MAXPATHL as size_t);
                    }
                    shorten_buf_fname(errbuf, dirname, false_0);
                }
                ga_concat(
                    gap,
                    if (*qfp).qf_fname.is_null() {
                        (*errbuf).b_fname
                    } else {
                        (*qfp).qf_fname
                    },
                );
            }
        }
        ga_append(gap, '|' as uint8_t);
        if (*qfp).qf_lnum > 0 as linenr_T {
            qf_range_text(gap, qfp);
            ga_concat(
                gap,
                qf_types((*qfp).qf_type as ::core::ffi::c_int, (*qfp).qf_nr),
            );
        } else if !(*qfp).qf_pattern.is_null() {
            qf_fmt_text(gap, (*qfp).qf_pattern);
        }
        ga_append(gap, '|' as uint8_t);
        ga_append(gap, ' ' as uint8_t);
        qf_fmt_text(
            gap,
            if (*gap).ga_len > 3 as ::core::ffi::c_int {
                skipwhite((*qfp).qf_text)
            } else {
                (*qfp).qf_text
            },
        );
    }
    ga_append(gap, NUL as uint8_t);
    if ml_append_buf(
        buf,
        lnum,
        (*gap).ga_data as *mut ::core::ffi::c_char,
        (*gap).ga_len as colnr_T,
        false_0 != 0,
    ) == FAIL
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn call_qftf_func(
    mut qfl: *mut qf_list_T,
    mut qf_winid_0: ::core::ffi::c_int,
    mut start_idx: ::core::ffi::c_int,
    mut end_idx: ::core::ffi::c_int,
) -> *mut list_T {
    let mut cb: *mut Callback = qftf_cb.ptr();
    let mut qftf_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() {
        return ::core::ptr::null_mut::<list_T>();
    }
    recursive.set(true_0 != 0);
    if (*qfl).qf_qftf_cb.type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cb = &raw mut (*qfl).qf_qftf_cb;
    }
    if (*cb).type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut args: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let dict: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
        tv_dict_add_nr(
            dict,
            b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            ((*qfl).qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            qf_winid_0 as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*qfl).qf_id as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"start_idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            start_idx as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"end_idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            end_idx as varnumber_T,
        );
        (*dict).dv_refcount += 1;
        args[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
        args[0 as ::core::ffi::c_int as usize].vval.v_dict = dict;
        (*textlock.ptr()) += 1;
        if callback_call(
            cb,
            1 as ::core::ffi::c_int,
            &raw mut args as *mut typval_T,
            &raw mut rettv,
        ) {
            if rettv.v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                qftf_list = rettv.vval.v_list;
                tv_list_ref(qftf_list);
            }
            tv_clear(&raw mut rettv);
        }
        (*textlock.ptr()) -= 1;
        tv_dict_unref(dict);
    }
    recursive.set(false_0 != 0);
    return qftf_list;
}
unsafe extern "C" fn qf_fill_buffer(
    mut qfl: *mut qf_list_T,
    mut buf: *mut buf_T,
    mut old_last: *mut qfline_T,
    mut qf_winid_0: ::core::ffi::c_int,
) {
    let old_KeyTyped: bool = KeyTyped.get();
    if old_last.is_null() {
        if buf != curbuf.get() {
            internal_error(b"qf_fill_buffer()\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 as ::core::ffi::c_int {
            if ml_delete(1 as linenr_T) == FAIL {
                internal_error(b"qf_fill_buffer()\0".as_ptr() as *const ::core::ffi::c_char);
                return;
            }
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == curbuf.get() {
                    (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        u_clearallandblockfree(curbuf.get());
    }
    if !qfl.is_null() && !(*qfl).qf_start.is_null() {
        let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
        *(&raw mut dirname as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
        let mut lnum: linenr_T = 0;
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        if old_last.is_null() {
            qfp = (*qfl).qf_start;
            lnum = 0 as ::core::ffi::c_int as linenr_T;
        } else {
            qfp = if !(*old_last).qf_next.is_null() {
                (*old_last).qf_next
            } else {
                old_last
            };
            lnum = (*buf).b_ml.ml_line_count;
        }
        let mut qftf_list: *mut list_T = call_qftf_func(
            qfl,
            qf_winid_0,
            lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            (*qfl).qf_count,
        );
        let mut qftf_li: *mut listitem_T = tv_list_first(qftf_list);
        let mut prev_bufnr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut invalid_val: bool = false_0 != 0;
        while lnum < (*qfl).qf_count as linenr_T {
            let mut qftf_str: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !qftf_li.is_null() && !invalid_val {
                qftf_str = tv_get_string_chk(&raw mut (*qftf_li).li_tv) as *mut ::core::ffi::c_char;
                if qftf_str.is_null() {
                    invalid_val = true_0 != 0;
                }
            }
            if qf_buf_add_line(
                qfl,
                buf,
                lnum,
                qfp,
                &raw mut dirname as *mut ::core::ffi::c_char,
                qftf_str,
                prev_bufnr != (*qfp).qf_fnum,
            ) == FAIL
            {
                break;
            }
            prev_bufnr = (*qfp).qf_fnum;
            lnum += 1;
            qfp = (*qfp).qf_next;
            if qfp.is_null() {
                break;
            }
            if !qftf_li.is_null() {
                qftf_li = (*qftf_li).li_next;
            }
        }
        if old_last.is_null() {
            ml_delete(lnum + 1 as linenr_T);
        }
        qfga_clear();
    }
    check_lnums(true_0 != 0);
    if old_last.is_null() {
        (*curbuf.get()).b_ro_locked += 1;
        set_option_value_give_err(
            kOptFiletype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"qf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        (*curbuf.get()).b_p_ma = false_0;
        (*curbuf.get()).b_keep_filetype = true_0 != 0;
        apply_autocmds(
            EVENT_BUFREADPOST,
            b"quickfix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            b"quickfix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_keep_filetype = false_0 != 0;
        (*curbuf.get()).b_ro_locked -= 1;
        redraw_curbuf_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    KeyTyped.set(old_KeyTyped);
}
unsafe extern "C" fn qf_list_changed(mut qfl: *mut qf_list_T) {
    (*qfl).qf_changedtick += 1;
}
unsafe extern "C" fn qf_id2nr(
    qi: *const qf_info_T,
    qfid: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut qf_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while qf_idx < (*qi).qf_listcount {
        if (*(*qi).qf_lists.offset(qf_idx as isize)).qf_id == qfid {
            return qf_idx;
        }
        qf_idx += 1;
    }
    return INVALID_QFIDX;
}
unsafe extern "C" fn qf_restore_list(
    mut qi: *mut qf_info_T,
    mut save_qfid: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    if (*qf_get_curlist(qi)).qf_id == save_qfid {
        return OK;
    }
    let curlist: ::core::ffi::c_int = qf_id2nr(qi, save_qfid);
    if curlist < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    (*qi).qf_curlist = curlist;
    return OK;
}
unsafe extern "C" fn qf_jump_first(
    mut qi: *mut qf_info_T,
    mut save_qfid: ::core::ffi::c_uint,
    mut forceit: ::core::ffi::c_int,
) {
    if qf_restore_list(qi, save_qfid) == FAIL {
        return;
    }
    if !check_can_set_curbuf_forceit(forceit) {
        return;
    }
    if !qf_list_empty(qf_get_curlist(qi)) {
        qf_jump(
            qi,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            forceit,
        );
    }
}
pub unsafe extern "C" fn grep_internal(mut cmdidx: cmdidx_T) -> ::core::ffi::c_int {
    return ((cmdidx as ::core::ffi::c_int == CMD_grep as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_lgrep as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_grepadd as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_lgrepadd as ::core::ffi::c_int)
        && strcmp(
            b"internal\0".as_ptr() as *const ::core::ffi::c_char,
            if *(*curbuf.get()).b_p_gp as ::core::ffi::c_int == NUL {
                p_gp.get()
            } else {
                (*curbuf.get()).b_p_gp
            },
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn make_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        274 => {
            return b"make\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        248 => {
            return b"lmake\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        172 => {
            return b"grep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        239 => {
            return b"lgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        173 => {
            return b"grepadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        240 => {
            return b"lgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
unsafe extern "C" fn make_get_fullcmd(
    mut makecmd: *const ::core::ffi::c_char,
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(p_shq.get())
        .wrapping_mul(2 as size_t)
        .wrapping_add(strlen(makecmd))
        .wrapping_add(1 as size_t);
    if *p_sp.get() as ::core::ffi::c_int != NUL {
        len = len.wrapping_add(
            strlen(p_sp.get())
                .wrapping_add(strlen(fname))
                .wrapping_add(3 as size_t),
        );
    }
    let cmd: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    snprintf(
        cmd,
        len,
        b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        p_shq.get(),
        makecmd,
        p_shq.get(),
    );
    if *p_sp.get() as ::core::ffi::c_int != NUL {
        append_redir(cmd, len, p_sp.get(), fname);
    }
    if msg_col.get() == 0 as ::core::ffi::c_int {
        msg_didout.set(false_0 != 0);
    }
    msg_start();
    msg_puts(b":!\0".as_ptr() as *const ::core::ffi::c_char);
    msg_outtrans(cmd, 0 as ::core::ffi::c_int, false_0 != 0);
    return cmd;
}
pub unsafe extern "C" fn ex_make(mut eap: *mut exarg_T) {
    let mut save_qfid: ::core::ffi::c_uint = 0;
    let mut enc: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_menc as ::core::ffi::c_int != NUL {
            (*curbuf.get()).b_p_menc
        } else {
            p_menc.get()
        };
    if grep_internal((*eap).cmdidx) != 0 {
        ex_vimgrep(eap);
        return;
    }
    let au_name: *mut ::core::ffi::c_char = make_get_auname((*eap).cmdidx);
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return;
        }
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
        wp = curwin.get();
    }
    autowrite_all();
    let mut fname: *mut ::core::ffi::c_char = get_mef_name();
    if fname.is_null() {
        return;
    }
    os_remove(fname);
    let cmd: *mut ::core::ffi::c_char = make_get_fullcmd((*eap).arg, fname);
    do_shell(cmd, 0 as ::core::ffi::c_int);
    incr_quickfix_busy();
    let mut errorformat: *mut ::core::ffi::c_char = if (*eap).cmdidx as ::core::ffi::c_int
        != CMD_make as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_lmake as ::core::ffi::c_int
    {
        if *(*curbuf.get()).b_p_gefm as ::core::ffi::c_int != NUL {
            (*curbuf.get()).b_p_gefm
        } else {
            p_gefm.get()
        }
    } else {
        p_efm.get()
    };
    let mut newlist: bool = (*eap).cmdidx as ::core::ffi::c_int
        != CMD_grepadd as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrepadd as ::core::ffi::c_int;
    let mut res: ::core::ffi::c_int = qf_init(
        wp,
        fname,
        errorformat,
        newlist as ::core::ffi::c_int,
        qf_cmdtitle(*(*eap).cmdlinep),
        enc,
    );
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4655 as ::core::ffi::c_uint,
                b"void ex_make(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    '_cleanup: {
        if !wp.is_null() {
            qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null()
            {
                (*wp).w_llist_ref
            } else {
                (*wp).w_llist
            };
            if qi.is_null() {
                break '_cleanup;
            }
        }
        if res >= 0 as ::core::ffi::c_int {
            qf_list_changed(qf_get_curlist(qi));
        }
        save_qfid = (*qf_get_curlist(qi)).qf_id;
        if !au_name.is_null() {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
        }
        if res > 0 as ::core::ffi::c_int
            && (*eap).forceit == 0
            && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
        {
            qf_jump_first(qi, save_qfid, false_0);
        }
    }
    decr_quickfix_busy();
    os_remove(fname);
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(cmd as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_mef_name() -> *mut ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
    static off: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if *p_mef.get() as ::core::ffi::c_int == NUL {
        name = vim_tempname();
        if name.is_null() {
            emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
        }
        return name;
    }
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = p_mef.get();
    while *p != 0 {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '#' as ::core::ffi::c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        return xstrdup(p_mef.get());
    }
    loop {
        if start.get() == -1 as ::core::ffi::c_int {
            start.set(os_get_pid() as ::core::ffi::c_int);
        } else {
            (*off.ptr()) += 19 as ::core::ffi::c_int;
        }
        name = xmalloc(strlen(p_mef.get()).wrapping_add(30 as size_t)) as *mut ::core::ffi::c_char;
        strcpy(name, p_mef.get());
        snprintf(
            name.offset(p.offset_from(p_mef.get()) as isize),
            strlen(name),
            b"%d%d\0".as_ptr() as *const ::core::ffi::c_char,
            start.get(),
            off.get(),
        );
        strcat(name, p.offset(2 as ::core::ffi::c_int as isize));
        let mut file_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        let mut file_or_link_found: bool = os_fileinfo_link(name, &raw mut file_info);
        if !file_or_link_found {
            break;
        }
        xfree(name as *mut ::core::ffi::c_void);
    }
    return name;
}
pub unsafe extern "C" fn qf_get_size(mut eap: *mut exarg_T) -> size_t {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, false_0 != 0);
    if qi.is_null() {
        return 0 as size_t;
    }
    return (*qf_get_curlist(qi)).qf_count as size_t;
}
pub unsafe extern "C" fn qf_get_valid_size(mut eap: *mut exarg_T) -> size_t {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, false_0 != 0);
    if qi.is_null() {
        return 0 as size_t;
    }
    let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sz: size_t = 0 as size_t;
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut i: ::core::ffi::c_int = 0;
    '_c2rust_label: {
        if (*qf_get_curlist(qi)).qf_count >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"qf_get_curlist(qi)->qf_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4760 as ::core::ffi::c_uint,
                b"size_t qf_get_valid_size(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    i = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
        if (*qfp).qf_valid != 0 {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            {
                sz = sz.wrapping_add(1);
            } else if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                sz = sz.wrapping_add(1);
                prev_fnum = (*qfp).qf_fnum;
            }
        }
        i += 1;
        qfp = (*qfp).qf_next;
    }
    return sz;
}
pub unsafe extern "C" fn qf_get_cur_idx(mut eap: *mut exarg_T) -> size_t {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, false_0 != 0);
    if qi.is_null() {
        return 0 as size_t;
    }
    '_c2rust_label: {
        if (*qf_get_curlist(qi)).qf_index >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"qf_get_curlist(qi)->qf_index >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4791 as ::core::ffi::c_uint,
                b"size_t qf_get_cur_idx(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return (*qf_get_curlist(qi)).qf_index as size_t;
}
pub unsafe extern "C" fn qf_get_cur_valid_idx(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, false_0 != 0);
    if qi.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    if !qf_list_has_valid_entries(qfl) {
        return 1 as ::core::ffi::c_int;
    }
    let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eidx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut i: size_t = 0;
    '_c2rust_label: {
        if (*qfl).qf_index >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"qfl->qf_index >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4818 as ::core::ffi::c_uint,
                b"int qf_get_cur_valid_idx(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    i = 1 as size_t;
    qfp = (*qfl).qf_start;
    while i <= (*qfl).qf_index as size_t && !qfp.is_null() {
        if (*qfp).qf_valid != 0 {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
            {
                if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                    eidx += 1;
                    prev_fnum = (*qfp).qf_fnum;
                }
            } else {
                eidx += 1;
            }
        }
        i = i.wrapping_add(1);
        qfp = (*qfp).qf_next;
    }
    return if eidx != 0 as ::core::ffi::c_int {
        eidx
    } else {
        1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn qf_get_nth_valid_entry(
    mut qfl: *mut qf_list_T,
    mut n: size_t,
    mut fdo: bool,
) -> size_t {
    if !qf_list_has_valid_entries(qfl) {
        return 1 as size_t;
    }
    let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eidx: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0;
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    '_c2rust_label: {
        if (*qfl).qf_count >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"qfl->qf_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4857 as ::core::ffi::c_uint,
                b"size_t qf_get_nth_valid_entry(qf_list_T *, size_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    i = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
        if (*qfp).qf_valid != 0 {
            if fdo {
                if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                    eidx = eidx.wrapping_add(1);
                    prev_fnum = (*qfp).qf_fnum;
                }
            } else {
                eidx = eidx.wrapping_add(1);
            }
        }
        if eidx == n {
            break;
        }
        i += 1;
        qfp = (*qfp).qf_next;
    }
    return if i <= (*qfl).qf_count {
        i as size_t
    } else {
        1 as size_t
    };
}
pub unsafe extern "C" fn ex_cc(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut errornr: ::core::ffi::c_int = 0;
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        errornr = (*eap).line2 as ::core::ffi::c_int;
    } else {
        match (*eap).cmdidx as ::core::ffi::c_int {
            59 | 243 => {
                errornr = 0 as ::core::ffi::c_int;
            }
            104 | 261 | 67 | 235 => {
                errornr = 1 as ::core::ffi::c_int;
            }
            _ => {
                errornr = 32767 as ::core::ffi::c_int;
            }
        }
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
    {
        let mut n: size_t = 0;
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            '_c2rust_label: {
                if (*eap).line1 >= 0 as linenr_T {
                } else {
                    __assert_fail(
                        b"eap->line1 >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4917 as ::core::ffi::c_uint,
                        b"void ex_cc(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            n = (*eap).line1 as size_t;
        } else {
            n = 1 as size_t;
        }
        let mut valid_entry: size_t = qf_get_nth_valid_entry(
            qf_get_curlist(qi),
            n,
            (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int,
        );
        '_c2rust_label_0: {
            if valid_entry <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"valid_entry <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4924 as ::core::ffi::c_uint,
                    b"void ex_cc(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        errornr = valid_entry as ::core::ffi::c_int;
    }
    qf_jump(qi, 0 as ::core::ffi::c_int, errornr, (*eap).forceit);
}
pub unsafe extern "C" fn ex_cnext(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut errornr: ::core::ffi::c_int = 0;
    if (*eap).addr_count > 0 as ::core::ffi::c_int
        && ((*eap).cmdidx as ::core::ffi::c_int != CMD_cdo as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_ldo as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_cfdo as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_lfdo as ::core::ffi::c_int)
    {
        errornr = (*eap).line2 as ::core::ffi::c_int;
    } else {
        errornr = 1 as ::core::ffi::c_int;
    }
    let mut dir: Direction = kDirectionNotSet;
    match (*eap).cmdidx as ::core::ffi::c_int {
        101 | 259 | 44 | 211 => {
            dir = BACKWARD;
        }
        86 | 252 | 66 | 234 => {
            dir = FORWARD_FILE;
        }
        102 | 260 | 45 | 212 => {
            dir = BACKWARD_FILE;
        }
        84 | 250 | 62 | 228 | _ => {
            dir = FORWARD;
        }
    }
    qf_jump(qi, dir as ::core::ffi::c_int, errornr, (*eap).forceit);
}
unsafe extern "C" fn qf_find_first_entry_in_buf(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    idx = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && idx <= (*qfl).qf_count && !qfp.is_null() {
        if (*qfp).qf_fnum == bnr {
            break;
        }
        idx += 1;
        qfp = (*qfp).qf_next;
    }
    *errornr = idx;
    return qfp;
}
unsafe extern "C" fn qf_find_first_entry_on_line(
    mut entry: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    while !got_int.get()
        && !(*entry).qf_prev.is_null()
        && (*entry).qf_fnum == (*(*entry).qf_prev).qf_fnum
        && (*entry).qf_lnum == (*(*entry).qf_prev).qf_lnum
    {
        entry = (*entry).qf_prev;
        *errornr -= 1;
    }
    return entry;
}
unsafe extern "C" fn qf_find_last_entry_on_line(
    mut entry: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    while !got_int.get()
        && !(*entry).qf_next.is_null()
        && (*entry).qf_fnum == (*(*entry).qf_next).qf_fnum
        && (*entry).qf_lnum == (*(*entry).qf_next).qf_lnum
    {
        entry = (*entry).qf_next;
        *errornr += 1;
    }
    return entry;
}
unsafe extern "C" fn qf_entry_after_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    if linewise {
        return (*qfp).qf_lnum > (*pos).lnum;
    }
    return (*qfp).qf_lnum > (*pos).lnum
        || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col > (*pos).col;
}
unsafe extern "C" fn qf_entry_before_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    if linewise {
        return (*qfp).qf_lnum < (*pos).lnum;
    }
    return (*qfp).qf_lnum < (*pos).lnum
        || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col < (*pos).col;
}
unsafe extern "C" fn qf_entry_on_or_after_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    if linewise {
        return (*qfp).qf_lnum >= (*pos).lnum;
    }
    return (*qfp).qf_lnum > (*pos).lnum
        || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col >= (*pos).col;
}
unsafe extern "C" fn qf_entry_on_or_before_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    if linewise {
        return (*qfp).qf_lnum <= (*pos).lnum;
    }
    return (*qfp).qf_lnum < (*pos).lnum
        || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col <= (*pos).col;
}
unsafe extern "C" fn qf_find_entry_after_pos(
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut linewise: bool,
    mut qfp: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    if qf_entry_after_pos(qfp, pos, linewise) {
        return qfp;
    }
    while !(*qfp).qf_next.is_null()
        && (*(*qfp).qf_next).qf_fnum == bnr
        && qf_entry_on_or_before_pos((*qfp).qf_next, pos, linewise) as ::core::ffi::c_int != 0
    {
        qfp = (*qfp).qf_next;
        *errornr += 1;
    }
    if (*qfp).qf_next.is_null() || (*(*qfp).qf_next).qf_fnum != bnr {
        return ::core::ptr::null_mut::<qfline_T>();
    }
    qfp = (*qfp).qf_next;
    *errornr += 1;
    return qfp;
}
unsafe extern "C" fn qf_find_entry_before_pos(
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut linewise: bool,
    mut qfp: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    while !(*qfp).qf_next.is_null()
        && (*(*qfp).qf_next).qf_fnum == bnr
        && qf_entry_before_pos((*qfp).qf_next, pos, linewise) as ::core::ffi::c_int != 0
    {
        qfp = (*qfp).qf_next;
        *errornr += 1;
    }
    if qf_entry_on_or_after_pos(qfp, pos, linewise) {
        return ::core::ptr::null_mut::<qfline_T>();
    }
    if linewise {
        qfp = qf_find_first_entry_on_line(qfp, errornr);
    }
    return qfp;
}
unsafe extern "C" fn qf_find_closest_entry(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut dir: Direction,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    *errornr = 0 as ::core::ffi::c_int;
    let mut qfp: *mut qfline_T = qf_find_first_entry_in_buf(qfl, bnr, errornr);
    if qfp.is_null() {
        return ::core::ptr::null_mut::<qfline_T>();
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        qfp = qf_find_entry_after_pos(bnr, pos, linewise, qfp, errornr);
    } else {
        qfp = qf_find_entry_before_pos(bnr, pos, linewise, qfp, errornr);
    }
    return qfp;
}
unsafe extern "C" fn qf_get_nth_below_entry(
    mut entry_arg: *mut qfline_T,
    mut n: linenr_T,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) {
    let mut entry: *mut qfline_T = entry_arg;
    loop {
        let c2rust_fresh25 = n;
        n = n - 1;
        if !(c2rust_fresh25 > 0 as linenr_T && !got_int.get()) {
            break;
        }
        let mut first_errornr: ::core::ffi::c_int = *errornr;
        if linewise {
            entry = qf_find_last_entry_on_line(entry, errornr);
        }
        if (*entry).qf_next.is_null() || (*(*entry).qf_next).qf_fnum != (*entry).qf_fnum {
            if linewise {
                *errornr = first_errornr;
            }
            break;
        } else {
            entry = (*entry).qf_next;
            *errornr += 1;
        }
    }
}
unsafe extern "C" fn qf_get_nth_above_entry(
    mut entry: *mut qfline_T,
    mut n: linenr_T,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) {
    loop {
        let c2rust_fresh24 = n;
        n = n - 1;
        if !(c2rust_fresh24 > 0 as linenr_T && !got_int.get()) {
            break;
        }
        if (*entry).qf_prev.is_null() || (*(*entry).qf_prev).qf_fnum != (*entry).qf_fnum {
            break;
        }
        entry = (*entry).qf_prev;
        *errornr -= 1;
        if linewise {
            entry = qf_find_first_entry_on_line(entry, errornr);
        }
    }
}
unsafe extern "C" fn qf_find_nth_adj_entry(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut n: linenr_T,
    mut dir: Direction,
    mut linewise: bool,
) -> ::core::ffi::c_int {
    let mut errornr: ::core::ffi::c_int = 0;
    let adj_entry: *mut qfline_T =
        qf_find_closest_entry(qfl, bnr, pos, dir, linewise, &raw mut errornr);
    if adj_entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    n -= 1;
    if n > 0 as linenr_T {
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            qf_get_nth_below_entry(adj_entry, n, linewise, &raw mut errornr);
        } else {
            qf_get_nth_above_entry(adj_entry, n, linewise, &raw mut errornr);
        }
    }
    return errornr;
}
pub unsafe extern "C" fn ex_cbelow(mut eap: *mut exarg_T) {
    if (*eap).addr_count > 0 as ::core::ffi::c_int && (*eap).line2 <= 0 as linenr_T {
        emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
        return;
    }
    let mut buf_has_flag: ::core::ffi::c_int = if (*eap).cmdidx as ::core::ffi::c_int
        == CMD_cabove as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_cbelow as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_cbefore as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_cafter as ::core::ffi::c_int
    {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };
    if (*curbuf.get()).b_has_qf_entry & buf_has_flag == 0 {
        emsg(gettext(
            &raw const e_no_errors as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    qi = qf_cmd_get_stack(eap, true_0 != 0);
    if qi.is_null() {
        return;
    }
    let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
    if !qf_list_has_valid_entries(qfl) {
        emsg(gettext(
            &raw const e_no_errors as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut dir: ::core::ffi::c_int = if (*eap).cmdidx as ::core::ffi::c_int
        == CMD_cbelow as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbelow as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_cafter as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lafter as ::core::ffi::c_int
    {
        FORWARD as ::core::ffi::c_int
    } else {
        BACKWARD as ::core::ffi::c_int
    };
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    pos.col += 1;
    let errornr: ::core::ffi::c_int = qf_find_nth_adj_entry(
        qfl,
        (*curbuf.get()).handle as ::core::ffi::c_int,
        &raw mut pos,
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            (*eap).line2
        } else {
            0 as linenr_T
        },
        dir as Direction,
        (*eap).cmdidx as ::core::ffi::c_int == CMD_cbelow as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbelow as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cabove as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_labove as ::core::ffi::c_int,
    );
    if errornr > 0 as ::core::ffi::c_int {
        qf_jump(qi, 0 as ::core::ffi::c_int, errornr, false_0);
    } else {
        emsg(gettext(e_no_more_items.get()));
    };
}
unsafe extern "C" fn cfile_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        65 => {
            return b"cfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        68 => {
            return b"cgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        51 => {
            return b"caddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        233 => {
            return b"lfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        236 => {
            return b"lgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        218 => {
            return b"laddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
pub unsafe extern "C" fn ex_cfile(mut eap: *mut exarg_T) {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                5343 as ::core::ffi::c_uint,
                b"void ex_cfile(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut au_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    au_name = cfile_get_auname((*eap).cmdidx);
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return;
        }
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        set_option_direct(
            kOptErrorfile,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string((*eap).arg),
                },
            },
            0 as ::core::ffi::c_int,
            0 as scid_T,
        );
    }
    let mut enc: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_menc as ::core::ffi::c_int != NUL {
            (*curbuf.get()).b_p_menc
        } else {
            p_menc.get()
        };
    if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
        wp = curwin.get();
    }
    incr_quickfix_busy();
    let mut res: ::core::ffi::c_int = qf_init(
        wp,
        p_ef.get(),
        p_efm.get(),
        ((*eap).cmdidx as ::core::ffi::c_int != CMD_caddfile as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddfile as ::core::ffi::c_int)
            as ::core::ffi::c_int,
        qf_cmdtitle(*(*eap).cmdlinep),
        enc,
    );
    if !wp.is_null() {
        qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
            && !(*wp).w_llist_ref.is_null()
        {
            (*wp).w_llist_ref
        } else {
            (*wp).w_llist
        };
        if qi.is_null() {
            decr_quickfix_busy();
            return;
        }
    }
    if res >= 0 as ::core::ffi::c_int {
        qf_list_changed(qf_get_curlist(qi));
    }
    let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
    if !au_name.is_null() {
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    if res > 0 as ::core::ffi::c_int
        && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cfile as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfile as ::core::ffi::c_int)
        && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
    {
        qf_jump_first(qi, save_qfid, (*eap).forceit);
    }
    decr_quickfix_busy();
}
unsafe extern "C" fn vgr_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        510 => {
            return b"vimgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        267 => {
            return b"lvimgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        511 => {
            return b"vimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        268 => {
            return b"lvimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        172 => {
            return b"grep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        239 => {
            return b"lgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        173 => {
            return b"grepadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        240 => {
            return b"lgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
unsafe extern "C" fn vgr_init_regmatch(
    mut regmatch: *mut regmmatch_T,
    mut s: *mut ::core::ffi::c_char,
) {
    (*regmatch).regprog = ::core::ptr::null_mut::<regprog_T>();
    if s.is_null() || *s as ::core::ffi::c_int == NUL {
        if last_search_pat().is_null() {
            emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
            return;
        }
        (*regmatch).regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
    } else {
        (*regmatch).regprog = vim_regcomp(s, RE_MAGIC);
    }
    (*regmatch).rmm_ic = p_ic.get();
    (*regmatch).rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
}
unsafe extern "C" fn vgr_display_fname(mut fname: *mut ::core::ffi::c_char) {
    msg_start();
    let mut p: *mut ::core::ffi::c_char = msg_strtrunc(fname, true_0);
    if p.is_null() {
        msg_outtrans(fname, 0 as ::core::ffi::c_int, false_0 != 0);
    } else {
        msg_outtrans(p, 0 as ::core::ffi::c_int, false_0 != 0);
        xfree(p as *mut ::core::ffi::c_void);
    }
    msg_clr_eos();
    msg_didout.set(false_0 != 0);
    msg_nowait.set(true_0 != 0);
    msg_col.set(0 as ::core::ffi::c_int);
    ui_flush();
}
unsafe extern "C" fn vgr_load_dummy_buf(
    mut fname: *mut ::core::ffi::c_char,
    mut dirname_start: *mut ::core::ffi::c_char,
    mut dirname_now: *mut ::core::ffi::c_char,
) -> *mut buf_T {
    let mut save_ei: *mut ::core::ffi::c_char = au_event_disable(b",Filetype\0".as_ptr()
        as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char);
    let mut save_mls: OptInt = p_mls.get();
    p_mls.set(0 as OptInt);
    let mut buf: *mut buf_T = load_dummy_buffer(fname, dirname_start, dirname_now);
    p_mls.set(save_mls);
    au_event_restore(save_ei);
    return buf;
}
unsafe extern "C" fn vgr_qflist_valid(
    mut wp: *mut win_T,
    mut qi: *mut qf_info_T,
    mut qfid: ::core::ffi::c_uint,
    mut title: *mut ::core::ffi::c_char,
) -> bool {
    if !qflist_valid(wp, qfid) {
        if !wp.is_null() {
            emsg(gettext(e_current_location_list_was_changed.get()));
            return false_0 != 0;
        }
        qf_new_list(qi, title);
        return true_0 != 0;
    }
    if qf_restore_list(qi, qfid) == FAIL {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn vgr_match_buflines(
    mut qfl: *mut qf_list_T,
    mut fname: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut spat: *mut ::core::ffi::c_char,
    mut regmatch: *mut regmmatch_T,
    mut tomatch: *mut ::core::ffi::c_int,
    mut duplicate_name: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> bool {
    let mut found_match: bool = false_0 != 0;
    let mut pat_len: size_t = strlen(spat);
    pat_len = if pat_len < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t {
        pat_len
    } else {
        FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t
    };
    let mut lnum: linenr_T = 1 as linenr_T;
    while lnum <= (*buf).b_ml.ml_line_count && *tomatch > 0 as ::core::ffi::c_int {
        let mut col: colnr_T = 0 as colnr_T;
        if flags & VGR_FUZZY as ::core::ffi::c_int == 0 {
            while vim_regexec_multi(
                regmatch,
                curwin.get(),
                buf,
                lnum,
                col,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) > 0 as ::core::ffi::c_int
            {
                if qf_add_entry(
                    qfl,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    fname,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    if duplicate_name != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*buf).handle as ::core::ffi::c_int
                    },
                    ml_get_buf(
                        buf,
                        (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum + lnum,
                    ),
                    (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum + lnum,
                    (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum + lnum,
                    (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col
                        as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                    (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                    false_0 as ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_char,
                    ::core::ptr::null_mut::<typval_T>(),
                    true_0 as ::core::ffi::c_char,
                ) == QF_FAIL as ::core::ffi::c_int
                {
                    got_int.set(true_0 != 0);
                    break;
                } else {
                    found_match = true_0 != 0;
                    *tomatch -= 1;
                    if *tomatch == 0 as ::core::ffi::c_int {
                        break;
                    }
                    if flags & VGR_GLOBAL as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        || (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                    {
                        break;
                    }
                    col = ((*regmatch).endpos[0 as ::core::ffi::c_int as usize].col
                        as ::core::ffi::c_int
                        + (col == (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col)
                            as ::core::ffi::c_int) as colnr_T;
                    if col > ml_get_buf_len(buf, lnum) {
                        break;
                    }
                }
            }
        } else {
            let str: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
            let linelen: colnr_T = ml_get_buf_len(buf, lnum);
            let mut score: ::core::ffi::c_int = 0;
            let mut matches: [uint32_t; 1024] = [0; 1024];
            let sz: size_t = ::core::mem::size_of::<[uint32_t; 1024]>()
                .wrapping_div(::core::mem::size_of::<uint32_t>());
            memset(
                &raw mut matches as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[uint32_t; 1024]>(),
            );
            while fuzzy_match(
                str.offset(col as isize),
                spat,
                false_0 != 0,
                &raw mut score,
                &raw mut matches as *mut uint32_t,
                sz as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                if qf_add_entry(
                    qfl,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    fname,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    if duplicate_name != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*buf).handle as ::core::ffi::c_int
                    },
                    str,
                    lnum,
                    0 as linenr_T,
                    matches[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        + col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    false_0 as ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_char,
                    ::core::ptr::null_mut::<typval_T>(),
                    true_0 as ::core::ffi::c_char,
                ) == QF_FAIL as ::core::ffi::c_int
                {
                    got_int.set(true_0 != 0);
                    break;
                } else {
                    found_match = true_0 != 0;
                    *tomatch -= 1;
                    if *tomatch == 0 as ::core::ffi::c_int {
                        break;
                    }
                    if flags & VGR_GLOBAL as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        break;
                    }
                    col = (matches[pat_len.wrapping_sub(1 as size_t) as usize]
                        as ::core::ffi::c_int
                        + col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    if col > linelen {
                        break;
                    }
                }
            }
        }
        line_breakcheck();
        if got_int.get() {
            break;
        }
        lnum += 1;
    }
    return found_match;
}
unsafe extern "C" fn vgr_jump_to_match(
    mut qi: *mut qf_info_T,
    mut forceit: ::core::ffi::c_int,
    mut redraw_for_dummy: *mut bool,
    mut first_match_buf: *mut buf_T,
    mut target_dir: *mut ::core::ffi::c_char,
) {
    let mut buf: *mut buf_T = curbuf.get();
    qf_jump(
        qi,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        forceit,
    );
    if buf != curbuf.get() {
        *redraw_for_dummy = false_0 != 0;
    }
    if curbuf.get() == first_match_buf && !target_dir.is_null() {
        let mut ea: exarg_T = exarg {
            arg: target_dir,
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_lcd,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        ex_cd(&raw mut ea);
    }
}
unsafe extern "C" fn existing_swapfile(mut buf: *const buf_T) -> bool {
    if !(*buf).b_ml.ml_mfp.is_null() && !(*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
        let fname: *const ::core::ffi::c_char = (*(*buf).b_ml.ml_mfp).mf_fname;
        let len: size_t = strlen(fname);
        return *fname.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != 'p' as ::core::ffi::c_int
            || *fname.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                != 'w' as ::core::ffi::c_int;
    }
    return false_0 != 0;
}
unsafe extern "C" fn vgr_process_args(
    mut eap: *mut exarg_T,
    mut args: *mut vgr_args_T,
) -> ::core::ffi::c_int {
    memset(
        args as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<vgr_args_T>(),
    );
    (*args).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    (*args).qf_title = xstrdup(qf_cmdtitle(*(*eap).cmdlinep));
    (*args).tomatch = (if (*eap).addr_count > 0 as ::core::ffi::c_int {
        (*eap).line2
    } else {
        MAXLNUM as ::core::ffi::c_int as linenr_T
    }) as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char =
        skip_vimgrep_pat((*eap).arg, &raw mut (*args).spat, &raw mut (*args).flags);
    if p.is_null() {
        emsg(gettext(&raw const e_invalpat as *const ::core::ffi::c_char));
        return FAIL;
    }
    vgr_init_regmatch(&raw mut (*args).regmatch, (*args).spat);
    if (*args).regmatch.regprog.is_null() {
        return FAIL;
    }
    p = skipwhite(p);
    if *p as ::core::ffi::c_int == NUL {
        emsg(gettext(
            b"E683: File name missing or invalid pattern\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if get_arglist_exp(
        p,
        &raw mut (*args).fcount,
        &raw mut (*args).fnames,
        true_0 != 0,
    ) == FAIL
        || (*args).fcount == 0 as ::core::ffi::c_int
    {
        emsg(gettext(&raw const e_nomatch as *const ::core::ffi::c_char));
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn vgr_process_files(
    mut wp: *mut win_T,
    mut qi: *mut qf_info_T,
    mut cmd_args: *mut vgr_args_T,
    mut redraw_for_dummy: *mut bool,
    mut first_match_buf: *mut *mut buf_T,
    mut target_dir: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = FAIL;
    let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
    let mut duplicate_name: bool = false_0 != 0;
    let mut dirname_start: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut dirname_now: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(dirname_start, MAXPATHL as size_t);
    let mut seconds: time_t = 0 as time_t;
    let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_theend: {
        while fi < (*cmd_args).fcount
            && !got_int.get()
            && (*cmd_args).tomatch > 0 as ::core::ffi::c_int
        {
            let mut fname: *mut ::core::ffi::c_char =
                path_try_shorten_fname(*(*cmd_args).fnames.offset(fi as isize));
            if time(::core::ptr::null_mut::<time_t>()) > seconds {
                seconds = time(::core::ptr::null_mut::<time_t>());
                vgr_display_fname(fname);
            }
            let mut buf: *mut buf_T = buflist_findname_exp(*(*cmd_args).fnames.offset(fi as isize));
            let mut using_dummy: bool = false;
            if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
                duplicate_name = !buf.is_null();
                using_dummy = true_0 != 0;
                *redraw_for_dummy = true_0 != 0;
                buf = vgr_load_dummy_buf(fname, dirname_start, dirname_now);
            } else {
                using_dummy = false_0 != 0;
            }
            if !vgr_qflist_valid(wp, qi, save_qfid, (*cmd_args).qf_title) {
                break '_theend;
            }
            save_qfid = (*qf_get_curlist(qi)).qf_id;
            if buf.is_null() {
                if !got_int.get() {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Cannot open file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                        fname,
                    );
                }
            } else {
                let mut found_match: bool = vgr_match_buflines(
                    qf_get_curlist(qi),
                    fname,
                    buf,
                    (*cmd_args).spat,
                    &raw mut (*cmd_args).regmatch,
                    &raw mut (*cmd_args).tomatch,
                    duplicate_name as ::core::ffi::c_int,
                    (*cmd_args).flags,
                );
                if using_dummy {
                    if found_match as ::core::ffi::c_int != 0 && (*first_match_buf).is_null() {
                        *first_match_buf = buf;
                    }
                    if duplicate_name {
                        wipe_dummy_buffer(buf, dirname_start);
                        buf = ::core::ptr::null_mut::<buf_T>();
                    } else if (*cmdmod.ptr()).cmod_flags & CMOD_HIDE as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                        || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'u' as ::core::ffi::c_int
                        || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'w' as ::core::ffi::c_int
                        || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'd' as ::core::ffi::c_int
                    {
                        if !found_match {
                            wipe_dummy_buffer(buf, dirname_start);
                            buf = ::core::ptr::null_mut::<buf_T>();
                        } else if buf != *first_match_buf
                            || (*cmd_args).flags & VGR_NOJUMP as ::core::ffi::c_int != 0
                            || existing_swapfile(buf) as ::core::ffi::c_int != 0
                        {
                            unload_dummy_buffer(buf, dirname_start);
                            (*buf).b_flags &= !BF_DUMMY;
                            buf = ::core::ptr::null_mut::<buf_T>();
                        }
                    }
                    if !buf.is_null() {
                        (*buf).b_flags &= !BF_DUMMY;
                        if buf == *first_match_buf
                            && (*target_dir).is_null()
                            && strcmp(dirname_start, dirname_now) != 0 as ::core::ffi::c_int
                        {
                            *target_dir = xstrdup(dirname_now);
                        }
                        let mut aco: aco_save_T = aco_save_T {
                            use_aucmd_win_idx: 0,
                            save_curwin_handle: 0,
                            new_curwin_handle: 0,
                            save_prevwin_handle: 0,
                            new_curbuf: bufref_T {
                                br_buf: ::core::ptr::null_mut::<buf_T>(),
                                br_fnum: 0,
                                br_buf_free_count: 0,
                            },
                            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            save_VIsual_active: false,
                            save_prompt_insert: 0,
                        };
                        aucmd_prepbuf(&raw mut aco, buf);
                        apply_autocmds(
                            EVENT_FILETYPE,
                            (*buf).b_p_ft,
                            (*buf).b_fname,
                            true_0 != 0,
                            buf,
                        );
                        do_modelines(OPT_NOWIN as ::core::ffi::c_int);
                        aucmd_restbuf(&raw mut aco);
                    }
                }
            }
            fi += 1;
        }
        status = OK;
    }
    xfree(dirname_now as *mut ::core::ffi::c_void);
    xfree(dirname_start as *mut ::core::ffi::c_void);
    return status;
}
pub unsafe extern "C" fn ex_vimgrep(mut eap: *mut exarg_T) {
    let mut redraw_for_dummy: bool = false;
    let mut first_match_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut status: ::core::ffi::c_int = 0;
    let mut qfl: *mut qf_list_T = ::core::ptr::null_mut::<qf_list_T>();
    let mut save_qfid: ::core::ffi::c_uint = 0;
    if !check_can_set_curbuf_forceit((*eap).forceit) {
        return;
    }
    let mut au_name: *mut ::core::ffi::c_char = vgr_get_auname((*eap).cmdidx);
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return;
        }
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
    let mut target_dir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut args: vgr_args_T = vgr_args_T {
        tomatch: 0,
        spat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        flags: 0,
        fnames: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        fcount: 0,
        regmatch: regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        },
        qf_title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if vgr_process_args(eap, &raw mut args) != FAIL {
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_grepadd as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrepadd as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_vimgrepadd as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_lvimgrepadd as ::core::ffi::c_int
            || qf_stack_empty(qi) as ::core::ffi::c_int != 0
        {
            qf_new_list(qi, args.qf_title);
        }
        incr_quickfix_busy();
        redraw_for_dummy = false_0 != 0;
        first_match_buf = ::core::ptr::null_mut::<buf_T>();
        status = vgr_process_files(
            wp,
            qi,
            &raw mut args,
            &raw mut redraw_for_dummy,
            &raw mut first_match_buf,
            &raw mut target_dir,
        );
        if status != OK {
            FreeWild(args.fcount, args.fnames);
            decr_quickfix_busy();
        } else {
            FreeWild(args.fcount, args.fnames);
            qfl = qf_get_curlist(qi);
            (*qfl).qf_nonevalid = false_0 != 0;
            (*qfl).qf_ptr = (*qfl).qf_start;
            (*qfl).qf_index = 1 as ::core::ffi::c_int;
            qf_list_changed(qfl);
            qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
            save_qfid = (*qf_get_curlist(qi)).qf_id;
            if !au_name.is_null() {
                apply_autocmds(
                    EVENT_QUICKFIXCMDPOST,
                    au_name,
                    (*curbuf.get()).b_fname,
                    true_0 != 0,
                    curbuf.get(),
                );
            }
            if !qflist_valid(wp, save_qfid) || qf_restore_list(qi, save_qfid) == FAIL {
                decr_quickfix_busy();
            } else {
                if !qf_list_empty(qf_get_curlist(qi)) {
                    if args.flags & VGR_NOJUMP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        vgr_jump_to_match(
                            qi,
                            (*eap).forceit,
                            &raw mut redraw_for_dummy,
                            first_match_buf,
                            target_dir,
                        );
                    }
                } else {
                    semsg(
                        gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                        args.spat,
                    );
                }
                decr_quickfix_busy();
                if redraw_for_dummy {
                    foldUpdateAll(curwin.get());
                }
            }
        }
    }
    xfree(args.qf_title as *mut ::core::ffi::c_void);
    xfree(target_dir as *mut ::core::ffi::c_void);
    vim_regfree(args.regmatch.regprog);
}
unsafe extern "C" fn restore_start_dir(mut dirname_start: *mut ::core::ffi::c_char) {
    let mut dirname_now: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(dirname_now, MAXPATHL as size_t);
    if strcmp(dirname_start, dirname_now) != 0 as ::core::ffi::c_int {
        let mut ea: exarg_T = exarg {
            arg: dirname_start,
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: (if (*curwin.get()).w_localdir.is_null() {
                CMD_cd as ::core::ffi::c_int
            } else {
                CMD_lcd as ::core::ffi::c_int
            }) as cmdidx_T,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        ex_cd(&raw mut ea);
    }
    xfree(dirname_now as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn load_dummy_buffer(
    mut fname: *mut ::core::ffi::c_char,
    mut dirname_start: *mut ::core::ffi::c_char,
    mut resulting_dir: *mut ::core::ffi::c_char,
) -> *mut buf_T {
    let mut newbuf: *mut buf_T = buflist_new(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        1 as linenr_T,
        BLN_DUMMY as ::core::ffi::c_int,
    );
    if newbuf.is_null() {
        return ::core::ptr::null_mut::<buf_T>();
    }
    let mut failed: bool = true_0 != 0;
    let mut newbufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut newbufref, newbuf);
    buf_copy_options(
        newbuf,
        BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
    );
    if ml_open(newbuf) == OK {
        (*newbuf).b_locked += 1;
        let mut aco: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        aucmd_prepbuf(&raw mut aco, newbuf);
        setfname(
            curbuf.get(),
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
        );
        check_need_swap(true_0 != 0);
        (*curbuf.get()).b_flags &= !BF_DUMMY;
        let mut newbuf_to_wipe: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        newbuf_to_wipe.br_buf = ::core::ptr::null_mut::<buf_T>();
        let mut readfile_result: ::core::ffi::c_int = readfile(
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            READ_NEW as ::core::ffi::c_int | READ_DUMMY as ::core::ffi::c_int,
            false_0 != 0,
        );
        (*newbuf).b_locked -= 1;
        if readfile_result == OK && !got_int.get() && (*curbuf.get()).b_flags & BF_NEW == 0 {
            failed = false_0 != 0;
            if curbuf.get() != newbuf {
                set_bufref(&raw mut newbuf_to_wipe, newbuf);
                newbuf = curbuf.get();
            }
        }
        aucmd_restbuf(&raw mut aco);
        if !newbuf_to_wipe.br_buf.is_null()
            && bufref_valid(&raw mut newbuf_to_wipe) as ::core::ffi::c_int != 0
        {
            block_autocmds();
            wipe_dummy_buffer(
                newbuf_to_wipe.br_buf,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
            unblock_autocmds();
        }
        (*newbuf).b_flags |= BF_DUMMY;
    }
    os_dirname(resulting_dir, MAXPATHL as size_t);
    restore_start_dir(dirname_start);
    if !bufref_valid(&raw mut newbufref) {
        return ::core::ptr::null_mut::<buf_T>();
    }
    if failed {
        wipe_dummy_buffer(newbuf, dirname_start);
        return ::core::ptr::null_mut::<buf_T>();
    }
    return newbuf;
}
unsafe extern "C" fn wipe_dummy_buffer(
    mut buf: *mut buf_T,
    mut dirname_start: *mut ::core::ffi::c_char,
) {
    '_fail: {
        while (*buf).b_nwindows > 0 as ::core::ffi::c_int {
            let mut did_one: bool = false_0 != 0;
            if !(*firstwin.get()).w_next.is_null() {
                let mut wp: *mut win_T = firstwin.get();
                while !wp.is_null() {
                    if (*wp).w_buffer == buf {
                        if win_close(wp, false_0 != 0, false_0 != 0) == OK {
                            did_one = true_0 != 0;
                        }
                        break;
                    } else {
                        wp = (*wp).w_next;
                    }
                }
            }
            if !did_one {
                break '_fail;
            }
        }
        if curbuf.get() != buf && (*buf).b_nwindows == 0 as ::core::ffi::c_int {
            let mut cs: cleanup_T = cleanup_T {
                pending: 0,
                exception: ::core::ptr::null_mut::<except_T>(),
            };
            enter_cleanup(&raw mut cs);
            wipe_buffer(buf, true_0 != 0);
            leave_cleanup(&raw mut cs);
            if !dirname_start.is_null() {
                restore_start_dir(dirname_start);
            }
            return;
        }
    }
    (*buf).b_flags &= !BF_DUMMY;
}
unsafe extern "C" fn unload_dummy_buffer(
    mut buf: *mut buf_T,
    mut dirname_start: *mut ::core::ffi::c_char,
) {
    if curbuf.get() == buf {
        return;
    }
    close_buffer(
        ::core::ptr::null_mut::<win_T>(),
        buf,
        DOBUF_UNLOAD as ::core::ffi::c_int,
        false_0 != 0,
        true_0 != 0,
    );
    restore_start_dir(dirname_start);
}
unsafe extern "C" fn get_qfline_items(
    mut qfp: *mut qfline_T,
    mut list: *mut list_T,
) -> ::core::ffi::c_int {
    let mut bufnum: ::core::ffi::c_int = (*qfp).qf_fnum;
    if bufnum != 0 as ::core::ffi::c_int && buflist_findnr(bufnum).is_null() {
        bufnum = 0 as ::core::ffi::c_int;
    }
    let dict: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(list, dict);
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    buf[0 as ::core::ffi::c_int as usize] = (*qfp).qf_type;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if tv_dict_add_nr(
        dict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        bufnum as varnumber_T,
    ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*qfp).qf_lnum as varnumber_T,
        ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*qfp).qf_end_lnum as varnumber_T,
        ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"col\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            (*qfp).qf_col as varnumber_T,
        ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*qfp).qf_end_col as varnumber_T,
        ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"vcol\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*qfp).qf_viscol as varnumber_T,
        ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"nr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*qfp).qf_nr as varnumber_T,
        ) == FAIL
        || tv_dict_add_str(
            dict,
            b"module\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            if (*qfp).qf_module.is_null() {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                (*qfp).qf_module as *const ::core::ffi::c_char
            },
        ) == FAIL
        || tv_dict_add_str(
            dict,
            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            if (*qfp).qf_pattern.is_null() {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                (*qfp).qf_pattern as *const ::core::ffi::c_char
            },
        ) == FAIL
        || tv_dict_add_str(
            dict,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            if (*qfp).qf_text.is_null() {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                (*qfp).qf_text as *const ::core::ffi::c_char
            },
        ) == FAIL
        || tv_dict_add_str(
            dict,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        ) == FAIL
        || (*qfp).qf_user_data.v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_dict_add_tv(
                dict,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                &raw mut (*qfp).qf_user_data,
            ) == FAIL
        || tv_dict_add_nr(
            dict,
            b"valid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*qfp).qf_valid as varnumber_T,
        ) == FAIL
    {
        abort();
    }
    return OK;
}
unsafe extern "C" fn get_errorlist(
    mut qi_arg: *mut qf_info_T,
    mut wp: *mut win_T,
    mut qf_idx: ::core::ffi::c_int,
    mut eidx: ::core::ffi::c_int,
    mut list: *mut list_T,
) -> ::core::ffi::c_int {
    let mut qi: *mut qf_info_T = qi_arg;
    if qi.is_null() {
        qi = ql_info.get();
        if !wp.is_null() {
            qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null()
            {
                (*wp).w_llist_ref
            } else {
                (*wp).w_llist
            };
        }
        if qi.is_null() {
            return FAIL;
        }
    }
    if eidx < 0 as ::core::ffi::c_int {
        return OK;
    }
    if qf_idx == INVALID_QFIDX {
        qf_idx = (*qi).qf_curlist;
    }
    if qf_idx >= (*qi).qf_listcount {
        return FAIL;
    }
    let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
    if qf_list_empty(qfl) {
        return FAIL;
    }
    let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut i: ::core::ffi::c_int = 0;
    i = 1 as ::core::ffi::c_int;
    qfp = (*qfl).qf_start;
    while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
        if eidx > 0 as ::core::ffi::c_int {
            if eidx == i {
                return get_qfline_items(qfp, list);
            }
        } else if get_qfline_items(qfp, list) == FAIL {
            return FAIL;
        }
        i += 1;
        qfp = (*qfp).qf_next;
    }
    return OK;
}
unsafe extern "C" fn qf_get_list_from_lines(
    mut what: *mut dict_T,
    mut di: *mut dictitem_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = FAIL;
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*di).di_tv.vval.v_list.is_null()
    {
        return FAIL;
    }
    let mut errorformat: *mut ::core::ffi::c_char = p_efm.get();
    let mut efm_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    efm_di = tv_dict_find(
        what,
        b"efm\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !efm_di.is_null() {
        if (*efm_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*efm_di).di_tv.vval.v_string.is_null()
        {
            return FAIL;
        }
        errorformat = (*efm_di).di_tv.vval.v_string;
    }
    let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let qi: *mut qf_info_T = qf_alloc_stack(QFLT_INTERNAL, 1 as ::core::ffi::c_int);
    if qf_init_ext(
        qi,
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<buf_T>(),
        &raw mut (*di).di_tv,
        errorformat,
        true_0 != 0,
        0 as linenr_T,
        0 as linenr_T,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) > 0 as ::core::ffi::c_int
    {
        get_errorlist(
            qi,
            ::core::ptr::null_mut::<win_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            l,
        );
        qf_free((*qi).qf_lists.offset(0 as ::core::ffi::c_int as isize));
    }
    qf_free_lists(qi);
    tv_dict_add_list(
        retdict,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    status = OK;
    return status;
}
unsafe extern "C" fn qf_winid(mut qi: *mut qf_info_T) -> ::core::ffi::c_int {
    if qi.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut win: *mut win_T = qf_find_win(qi);
    if !win.is_null() {
        return (*win).handle as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn qf_getprop_qfbufnr(
    mut qi: *const qf_info_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut bufnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !qi.is_null() && !buflist_findnr((*qi).qf_bufnr).is_null() {
        bufnum = (*qi).qf_bufnr;
    }
    return tv_dict_add_nr(
        retdict,
        b"qfbufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        bufnum as varnumber_T,
    );
}
unsafe extern "C" fn qf_getprop_keys2flags(
    mut what: *const dict_T,
    mut loclist: bool,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = QF_GETLIST_NONE as ::core::ffi::c_int;
    if !tv_dict_find(
        what,
        b"all\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_ALL as ::core::ffi::c_int;
        if !loclist {
            flags &= !(QF_GETLIST_FILEWINID as ::core::ffi::c_int);
        }
    }
    if !tv_dict_find(
        what,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_TITLE as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"nr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_NR as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"winid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_WINID as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"context\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_CONTEXT as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"id\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_ID as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_ITEMS as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_IDX as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"size\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_SIZE as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_TICK as ::core::ffi::c_int;
    }
    if loclist as ::core::ffi::c_int != 0
        && !tv_dict_find(
            what,
            b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
    {
        flags |= QF_GETLIST_FILEWINID as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"qfbufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_QFBUFNR as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        what,
        b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as usize) as ptrdiff_t,
    )
    .is_null()
    {
        flags |= QF_GETLIST_QFTF as ::core::ffi::c_int;
    }
    return flags;
}
unsafe extern "C" fn qf_getprop_qfidx(
    mut qi: *mut qf_info_T,
    mut what: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut qf_idx: ::core::ffi::c_int = (*qi).qf_curlist;
    di = tv_dict_find(
        what,
        b"nr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*di).di_tv.vval.v_number != 0 as varnumber_T {
                qf_idx = (*di).di_tv.vval.v_number as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                if qf_idx < 0 as ::core::ffi::c_int || qf_idx >= (*qi).qf_listcount {
                    qf_idx = INVALID_QFIDX;
                }
            }
        } else if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && strequal(
                (*di).di_tv.vval.v_string,
                b"$\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
        {
            qf_idx = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
        } else {
            qf_idx = INVALID_QFIDX;
        }
    }
    di = tv_dict_find(
        what,
        b"id\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*di).di_tv.vval.v_number != 0 as varnumber_T {
                qf_idx = qf_id2nr(qi, (*di).di_tv.vval.v_number as ::core::ffi::c_uint);
            }
        } else {
            qf_idx = INVALID_QFIDX;
        }
    }
    return qf_idx;
}
unsafe extern "C" fn qf_getprop_defaults(
    mut qi: *mut qf_info_T,
    mut flags: ::core::ffi::c_int,
    mut locstack: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = OK;
    if flags & QF_GETLIST_TITLE as ::core::ffi::c_int != 0 {
        status = tv_dict_add_str(
            retdict,
            b"title\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if status == OK && flags & QF_GETLIST_ITEMS as ::core::ffi::c_int != 0 {
        let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        status = tv_dict_add_list(
            retdict,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            l,
        );
    }
    if status == OK && flags & QF_GETLIST_NR as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"nr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_WINID as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            qf_winid(qi) as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_CONTEXT as ::core::ffi::c_int != 0 {
        status = tv_dict_add_str(
            retdict,
            b"context\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if status == OK && flags & QF_GETLIST_ID as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_IDX as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_SIZE as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"size\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_TICK as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && locstack != 0 && flags & QF_GETLIST_FILEWINID as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            0 as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_QFBUFNR as ::core::ffi::c_int != 0 {
        status = qf_getprop_qfbufnr(qi, retdict);
    }
    if status == OK && flags & QF_GETLIST_QFTF as ::core::ffi::c_int != 0 {
        status = tv_dict_add_str(
            retdict,
            b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return status;
}
unsafe extern "C" fn qf_getprop_title(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    return tv_dict_add_str(
        retdict,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*qfl).qf_title,
    );
}
unsafe extern "C" fn qf_getprop_filewinid(
    mut wp: *const win_T,
    mut qi: *const qf_info_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut winid: handle_T = 0 as handle_T;
    if !wp.is_null()
        && (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null())
    {
        let mut ll_wp: *mut win_T = qf_find_win_with_loclist(qi);
        if !ll_wp.is_null() {
            winid = (*ll_wp).handle;
        }
    }
    return tv_dict_add_nr(
        retdict,
        b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        winid as varnumber_T,
    );
}
unsafe extern "C" fn qf_getprop_items(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut eidx: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    get_errorlist(qi, ::core::ptr::null_mut::<win_T>(), qf_idx, eidx, l);
    tv_dict_add_list(
        retdict,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    return OK;
}
unsafe extern "C" fn qf_getprop_ctx(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    if !(*qfl).qf_ctx.is_null() {
        let mut di: *mut dictitem_T = tv_dict_item_alloc_len(
            b"context\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        );
        tv_copy((*qfl).qf_ctx, &raw mut (*di).di_tv);
        status = tv_dict_add(retdict, di);
        if status == FAIL {
            tv_dict_item_free(di);
        }
    } else {
        status = tv_dict_add_str(
            retdict,
            b"context\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return status;
}
unsafe extern "C" fn qf_getprop_idx(
    mut qfl: *mut qf_list_T,
    mut eidx: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    if eidx == 0 as ::core::ffi::c_int {
        eidx = (*qfl).qf_index;
        if qf_list_empty(qfl) {
            eidx = 0 as ::core::ffi::c_int;
        }
    }
    return tv_dict_add_nr(
        retdict,
        b"idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        eidx as varnumber_T,
    );
}
unsafe extern "C" fn qf_getprop_qftf(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    if (*qfl).qf_qftf_cb.type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        callback_put(&raw mut (*qfl).qf_qftf_cb, &raw mut tv);
        status = tv_dict_add_tv(
            retdict,
            b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            &raw mut tv,
        );
        tv_clear(&raw mut tv);
    } else {
        status = tv_dict_add_str(
            retdict,
            b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return status;
}
unsafe extern "C" fn qf_get_properties(
    mut wp: *mut win_T,
    mut what: *mut dict_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                6512 as ::core::ffi::c_uint,
                b"int qf_get_properties(win_T *, dict_T *, dict_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut status: ::core::ffi::c_int = OK;
    let mut qf_idx: ::core::ffi::c_int = INVALID_QFIDX;
    di = tv_dict_find(
        what,
        b"lines\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        return qf_get_list_from_lines(what, di, retdict);
    }
    if !wp.is_null() {
        qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
            && !(*wp).w_llist_ref.is_null()
        {
            (*wp).w_llist_ref
        } else {
            (*wp).w_llist
        };
    }
    let flags: ::core::ffi::c_int = qf_getprop_keys2flags(what, !wp.is_null());
    if !qf_stack_empty(qi) {
        qf_idx = qf_getprop_qfidx(qi, what);
    }
    if qf_stack_empty(qi) as ::core::ffi::c_int != 0 || qf_idx == INVALID_QFIDX {
        return qf_getprop_defaults(qi, flags, !wp.is_null() as ::core::ffi::c_int, retdict);
    }
    let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
    let mut eidx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    di = tv_dict_find(
        what,
        b"idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return FAIL;
        }
        eidx = (*di).di_tv.vval.v_number as ::core::ffi::c_int;
    }
    if flags & QF_GETLIST_TITLE as ::core::ffi::c_int != 0 {
        status = qf_getprop_title(qfl, retdict);
    }
    if status == OK && flags & QF_GETLIST_NR as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"nr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (qf_idx + 1 as ::core::ffi::c_int) as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_WINID as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            qf_winid(qi) as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_ITEMS as ::core::ffi::c_int != 0 {
        status = qf_getprop_items(qi, qf_idx, eidx, retdict);
    }
    if status == OK && flags & QF_GETLIST_CONTEXT as ::core::ffi::c_int != 0 {
        status = qf_getprop_ctx(qfl, retdict);
    }
    if status == OK && flags & QF_GETLIST_ID as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*qfl).qf_id as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_IDX as ::core::ffi::c_int != 0 {
        status = qf_getprop_idx(qfl, eidx, retdict);
    }
    if status == OK && flags & QF_GETLIST_SIZE as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"size\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*qfl).qf_count as varnumber_T,
        );
    }
    if status == OK && flags & QF_GETLIST_TICK as ::core::ffi::c_int != 0 {
        status = tv_dict_add_nr(
            retdict,
            b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            (*qfl).qf_changedtick as varnumber_T,
        );
    }
    if status == OK && !wp.is_null() && flags & QF_GETLIST_FILEWINID as ::core::ffi::c_int != 0 {
        status = qf_getprop_filewinid(wp, qi, retdict);
    }
    if status == OK && flags & QF_GETLIST_QFBUFNR as ::core::ffi::c_int != 0 {
        status = qf_getprop_qfbufnr(qi, retdict);
    }
    if status == OK && flags & QF_GETLIST_QFTF as ::core::ffi::c_int != 0 {
        status = qf_getprop_qftf(qfl, retdict);
    }
    return status;
}
unsafe extern "C" fn qf_setprop_qftf(
    mut qfl: *mut qf_list_T,
    mut di: *mut dictitem_T,
) -> ::core::ffi::c_int {
    let mut cb: Callback = Callback {
        data: C2Rust_Unnamed_6 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return FAIL;
    }
    callback_free(&raw mut (*qfl).qf_qftf_cb);
    if callback_from_typval(&raw mut cb, &raw mut (*di).di_tv) {
        (*qfl).qf_qftf_cb = cb;
    }
    return OK;
}
unsafe extern "C" fn qf_add_entry_from_dict(
    mut qfl: *mut qf_list_T,
    mut d: *mut dict_T,
    mut first_entry: bool,
    mut valid_entry: *mut bool,
) -> ::core::ffi::c_int {
    static did_bufnr_emsg: GlobalCell<bool> = GlobalCell::new(false);
    if first_entry {
        did_bufnr_emsg.set(false_0 != 0);
    }
    let filename: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"filename\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    );
    let module: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"module\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    );
    let mut bufnum: ::core::ffi::c_int =
        tv_dict_get_number(d, b"bufnr\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int;
    let lnum: linenr_T =
        tv_dict_get_number(d, b"lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
    let end_lnum: linenr_T =
        tv_dict_get_number(d, b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
    let col: ::core::ffi::c_int =
        tv_dict_get_number(d, b"col\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int;
    let end_col: ::core::ffi::c_int =
        tv_dict_get_number(d, b"end_col\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int;
    let vcol: ::core::ffi::c_char =
        tv_dict_get_number(d, b"vcol\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_char;
    let nr: ::core::ffi::c_int =
        tv_dict_get_number(d, b"nr\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int;
    let type_0: *const ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"type\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let pattern: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    );
    let mut text: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"text\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    );
    if text.is_null() {
        text = xcalloc(1 as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
    }
    let mut user_data: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv_dict_get_tv(
        d,
        b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut user_data,
    );
    let mut valid: bool = true_0 != 0;
    if filename.is_null() && bufnum == 0 as ::core::ffi::c_int
        || lnum == 0 as linenr_T && pattern.is_null()
    {
        valid = false_0 != 0;
    }
    if bufnum != 0 as ::core::ffi::c_int && buflist_findnr(bufnum).is_null() {
        if !did_bufnr_emsg.get() {
            did_bufnr_emsg.set(true_0 != 0);
            semsg(
                gettext(b"E92: Buffer %d not found\0".as_ptr() as *const ::core::ffi::c_char),
                bufnum,
            );
        }
        valid = false_0 != 0;
        bufnum = 0 as ::core::ffi::c_int;
    }
    if !tv_dict_find(
        d,
        b"valid\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    )
    .is_null()
    {
        valid = tv_dict_get_bool(
            d,
            b"valid\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
        ) != 0;
    }
    let status: ::core::ffi::c_int = qf_add_entry(
        qfl,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        filename,
        module,
        bufnum,
        text,
        lnum,
        end_lnum,
        col,
        end_col,
        vcol,
        pattern,
        nr,
        (if type_0.is_null() {
            NUL
        } else {
            *type_0 as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        &raw mut user_data,
        valid as ::core::ffi::c_char,
    );
    xfree(filename as *mut ::core::ffi::c_void);
    xfree(module as *mut ::core::ffi::c_void);
    xfree(pattern as *mut ::core::ffi::c_void);
    xfree(text as *mut ::core::ffi::c_void);
    tv_clear(&raw mut user_data);
    if valid {
        *valid_entry = true_0 != 0;
    }
    return status;
}
unsafe extern "C" fn entry_is_closer_to_target(
    mut entry: *mut qfline_T,
    mut other_entry: *mut qfline_T,
    mut target_fnum: ::core::ffi::c_int,
    mut target_lnum: ::core::ffi::c_int,
    mut target_col: ::core::ffi::c_int,
) -> bool {
    if target_fnum == 0 {
        return false_0 != 0;
    }
    let mut is_target_file: bool = (*entry).qf_fnum != 0 && (*entry).qf_fnum == target_fnum;
    let mut other_is_target_file: bool =
        (*other_entry).qf_fnum != 0 && (*other_entry).qf_fnum == target_fnum;
    if !is_target_file && other_is_target_file as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    } else if is_target_file as ::core::ffi::c_int != 0 && !other_is_target_file {
        return true_0 != 0;
    }
    if target_lnum == 0 {
        return false_0 != 0;
    }
    let mut line_distance: ::core::ffi::c_int = if (*entry).qf_lnum != 0 {
        abs((*entry).qf_lnum as ::core::ffi::c_int - target_lnum)
    } else {
        INT_MAX
    };
    let mut other_line_distance: ::core::ffi::c_int = if (*other_entry).qf_lnum != 0 {
        abs((*other_entry).qf_lnum as ::core::ffi::c_int - target_lnum)
    } else {
        INT_MAX
    };
    if line_distance > other_line_distance {
        return false_0 != 0;
    } else if line_distance < other_line_distance {
        return true_0 != 0;
    }
    if target_col == 0 {
        return false_0 != 0;
    }
    let mut column_distance: ::core::ffi::c_int = if (*entry).qf_col != 0 {
        abs((*entry).qf_col - target_col)
    } else {
        INT_MAX
    };
    let mut other_column_distance: ::core::ffi::c_int = if (*other_entry).qf_col != 0 {
        abs((*other_entry).qf_col - target_col)
    } else {
        INT_MAX
    };
    if column_distance > other_column_distance {
        return false_0 != 0;
    } else if column_distance < other_column_distance {
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn qf_add_entries(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut list: *mut list_T,
    mut title: *mut ::core::ffi::c_char,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
    let mut old_last: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut retval: ::core::ffi::c_int = OK;
    let mut valid_entry: bool = false_0 != 0;
    let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*qfl).qf_ptr.is_null() {
        prev_fnum = (*(*qfl).qf_ptr).qf_fnum;
        prev_lnum = (*(*qfl).qf_ptr).qf_lnum as ::core::ffi::c_int;
        prev_col = (*(*qfl).qf_ptr).qf_col;
    }
    let mut select_first_entry: bool = false_0 != 0;
    let mut select_nearest_entry: bool = false_0 != 0;
    if action == ' ' as ::core::ffi::c_int || qf_idx == (*qi).qf_listcount {
        select_first_entry = true_0 != 0;
        qf_new_list(qi, title);
        qf_idx = (*qi).qf_curlist;
        qfl = qf_get_list(qi, qf_idx);
    } else if action == 'a' as ::core::ffi::c_int {
        if qf_list_empty(qfl) {
            select_first_entry = true_0 != 0;
        } else {
            old_last = (*qfl).qf_last;
        }
    } else if action == 'r' as ::core::ffi::c_int {
        select_first_entry = true_0 != 0;
        qf_free_items(qfl);
        qf_store_title(qfl, title);
    } else if action == 'u' as ::core::ffi::c_int {
        select_nearest_entry = true_0 != 0;
        qf_free_items(qfl);
        qf_store_title(qfl, title);
    }
    let mut entry_to_select: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
    let mut entry_to_select_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let d: *mut dict_T = (*li).li_tv.vval.v_dict;
                if !d.is_null() {
                    retval = qf_add_entry_from_dict(
                        qfl,
                        d,
                        li == tv_list_first(list) as *const listitem_T,
                        &raw mut valid_entry,
                    );
                    if retval == QF_FAIL as ::core::ffi::c_int {
                        break;
                    }
                    let mut entry: *mut qfline_T = (*qfl).qf_last;
                    if select_first_entry as ::core::ffi::c_int != 0 && entry_to_select.is_null()
                        || select_nearest_entry as ::core::ffi::c_int != 0
                            && (entry_to_select.is_null()
                                || entry_is_closer_to_target(
                                    entry,
                                    entry_to_select,
                                    prev_fnum,
                                    prev_lnum,
                                    prev_col,
                                ) as ::core::ffi::c_int
                                    != 0)
                    {
                        entry_to_select = entry;
                        entry_to_select_index = (*qfl).qf_count;
                    }
                }
            }
            li = (*li).li_next;
        }
    }
    if valid_entry {
        (*qfl).qf_nonevalid = false_0 != 0;
    } else if (*qfl).qf_index == 0 as ::core::ffi::c_int {
        (*qfl).qf_nonevalid = true_0 != 0;
    }
    if !entry_to_select.is_null() {
        (*qfl).qf_ptr = entry_to_select;
        (*qfl).qf_index = entry_to_select_index;
    }
    qf_update_buffer(qi, old_last);
    return retval;
}
unsafe extern "C" fn qf_setprop_get_qfidx(
    mut qi: *const qf_info_T,
    mut what: *const dict_T,
    mut action: ::core::ffi::c_int,
    mut newlist: *mut bool,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut qf_idx: ::core::ffi::c_int = (*qi).qf_curlist;
    di = tv_dict_find(
        what,
        b"nr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*di).di_tv.vval.v_number != 0 as varnumber_T {
                qf_idx = (*di).di_tv.vval.v_number as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            }
            if (action == ' ' as ::core::ffi::c_int || action == 'a' as ::core::ffi::c_int)
                && qf_idx == (*qi).qf_listcount
            {
                *newlist = true_0 != 0;
                qf_idx = if qf_stack_empty(qi) as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    (*qi).qf_listcount - 1 as ::core::ffi::c_int
                };
            } else if qf_idx < 0 as ::core::ffi::c_int || qf_idx >= (*qi).qf_listcount {
                return INVALID_QFIDX;
            } else if action != ' ' as ::core::ffi::c_int {
                *newlist = false_0 != 0;
            }
        } else if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && strequal(
                (*di).di_tv.vval.v_string,
                b"$\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
        {
            if !qf_stack_empty(qi) {
                qf_idx = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
            } else if *newlist {
                qf_idx = 0 as ::core::ffi::c_int;
            } else {
                return INVALID_QFIDX;
            }
        } else {
            return INVALID_QFIDX;
        }
    }
    if !*newlist && {
        di = tv_dict_find(
            what,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        !di.is_null()
    } {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return INVALID_QFIDX;
        }
        return qf_id2nr(qi, (*di).di_tv.vval.v_number as ::core::ffi::c_uint);
    }
    return qf_idx;
}
unsafe extern "C" fn qf_setprop_title(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut what: *const dict_T,
    mut di: *const dictitem_T,
) -> ::core::ffi::c_int {
    let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    xfree((*qfl).qf_title as *mut ::core::ffi::c_void);
    (*qfl).qf_title = tv_dict_get_string(
        what,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    );
    if qf_idx == (*qi).qf_curlist {
        qf_update_win_titlevar(qi);
    }
    return OK;
}
unsafe extern "C" fn qf_setprop_items(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut di: *mut dictitem_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    let mut title_save: *mut ::core::ffi::c_char =
        xstrdup((*(*qi).qf_lists.offset(qf_idx as isize)).qf_title);
    let retval: ::core::ffi::c_int = qf_add_entries(
        qi,
        qf_idx,
        (*di).di_tv.vval.v_list,
        title_save,
        if action == ' ' as ::core::ffi::c_int {
            'a' as ::core::ffi::c_int
        } else {
            action
        },
    );
    xfree(title_save as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn qf_setprop_items_from_lines(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut what: *const dict_T,
    mut di: *mut dictitem_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut errorformat: *mut ::core::ffi::c_char = p_efm.get();
    let mut efm_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut retval: ::core::ffi::c_int = FAIL;
    efm_di = tv_dict_find(
        what,
        b"efm\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !efm_di.is_null() {
        if (*efm_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*efm_di).di_tv.vval.v_string.is_null()
        {
            return FAIL;
        }
        errorformat = (*efm_di).di_tv.vval.v_string;
    }
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*di).di_tv.vval.v_list.is_null()
    {
        return FAIL;
    }
    if action == 'r' as ::core::ffi::c_int || action == 'u' as ::core::ffi::c_int {
        qf_free_items((*qi).qf_lists.offset(qf_idx as isize));
    }
    if qf_init_ext(
        qi,
        qf_idx,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<buf_T>(),
        &raw mut (*di).di_tv,
        errorformat,
        false_0 != 0,
        0 as linenr_T,
        0 as linenr_T,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) >= 0 as ::core::ffi::c_int
    {
        retval = OK;
    }
    return retval;
}
unsafe extern "C" fn qf_setprop_context(
    mut qfl: *mut qf_list_T,
    mut di: *mut dictitem_T,
) -> ::core::ffi::c_int {
    tv_free((*qfl).qf_ctx);
    let mut ctx: *mut typval_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
    tv_copy(&raw mut (*di).di_tv, ctx);
    (*qfl).qf_ctx = ctx;
    return OK;
}
unsafe extern "C" fn qf_setprop_curidx(
    mut qi: *mut qf_info_T,
    mut qfl: *mut qf_list_T,
    mut di: *const dictitem_T,
) -> ::core::ffi::c_int {
    let mut newidx: ::core::ffi::c_int = 0;
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*di).di_tv.vval.v_string.is_null()
        && strcmp(
            (*di).di_tv.vval.v_string,
            b"$\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        newidx = (*qfl).qf_count;
    } else {
        let mut denote: bool = false_0 != 0;
        newidx = tv_get_number_chk(&raw const (*di).di_tv, &raw mut denote) as ::core::ffi::c_int;
        if denote {
            return FAIL;
        }
    }
    if newidx < 1 as ::core::ffi::c_int {
        return FAIL;
    }
    newidx = if newidx < (*qfl).qf_count {
        newidx
    } else {
        (*qfl).qf_count
    };
    let old_qfidx: ::core::ffi::c_int = (*qfl).qf_index;
    let qf_ptr: *mut qfline_T = get_nth_entry(qfl, newidx, &raw mut newidx);
    if qf_ptr.is_null() {
        return FAIL;
    }
    (*qfl).qf_ptr = qf_ptr;
    (*qfl).qf_index = newidx;
    if (*(*qi).qf_lists.offset((*qi).qf_curlist as isize)).qf_id == (*qfl).qf_id {
        qf_win_pos_update(qi, old_qfidx);
    }
    return OK;
}
unsafe extern "C" fn qf_set_properties(
    mut qi: *mut qf_info_T,
    mut what: *const dict_T,
    mut action: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut newlist: bool =
        action == ' ' as ::core::ffi::c_int || qf_stack_empty(qi) as ::core::ffi::c_int != 0;
    let mut qf_idx: ::core::ffi::c_int = qf_setprop_get_qfidx(qi, what, action, &raw mut newlist);
    if qf_idx == INVALID_QFIDX {
        return FAIL;
    }
    if newlist {
        (*qi).qf_curlist = qf_idx;
        qf_new_list(qi, title);
        qf_idx = (*qi).qf_curlist;
    }
    let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut retval: ::core::ffi::c_int = FAIL;
    di = tv_dict_find(
        what,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_title(qi, qf_idx, what, di);
    }
    di = tv_dict_find(
        what,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_items(qi, qf_idx, di, action);
    }
    di = tv_dict_find(
        what,
        b"lines\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_items_from_lines(qi, qf_idx, what, di, action);
    }
    di = tv_dict_find(
        what,
        b"context\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_context(qfl, di);
    }
    di = tv_dict_find(
        what,
        b"idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_curidx(qi, qfl, di);
    }
    di = tv_dict_find(
        what,
        b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        retval = qf_setprop_qftf(qfl, di);
    }
    if newlist as ::core::ffi::c_int != 0 || retval == OK {
        qf_list_changed(qfl);
    }
    if newlist {
        qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
    }
    return retval;
}
unsafe extern "C" fn qf_free_stack(mut wp: *mut win_T, mut qi: *mut qf_info_T) {
    let mut qfwin: *mut win_T = qf_find_win(qi);
    if !qfwin.is_null() {
        if (*qi).qf_curlist < (*qi).qf_listcount {
            qf_free(qf_get_curlist(qi));
        }
        qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
    }
    if !wp.is_null()
        && (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null())
    {
        let llwin: *mut win_T = qf_find_win_with_loclist(qi);
        if !llwin.is_null() {
            wp = llwin;
        }
    }
    qf_free_all(wp);
    if wp.is_null() {
        (*qi).qf_curlist = 0 as ::core::ffi::c_int;
        (*qi).qf_listcount = 0 as ::core::ffi::c_int;
    } else if !qfwin.is_null() {
        let mut new_ll: *mut qf_info_T = qf_alloc_stack(
            QFLT_LOCATION,
            (*wp).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
        );
        (*new_ll).qf_bufnr = (*(*qfwin).w_buffer).handle as ::core::ffi::c_int;
        ll_free_all(&raw mut (*qfwin).w_llist_ref);
        (*qfwin).w_llist_ref = new_ll;
        if wp != qfwin {
            win_set_loclist(wp, new_ll);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_errorlist(
    mut wp: *mut win_T,
    mut list: *mut list_T,
    mut action: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut what: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
    if !wp.is_null() {
        qi = ll_get_or_alloc_list(wp);
    } else {
        qi = ql_info.get();
    }
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                7120 as ::core::ffi::c_uint,
                b"int set_errorlist(win_T *, list_T *, int, char *, dict_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if action == 'f' as ::core::ffi::c_int {
        qf_free_stack(wp, qi);
        return OK;
    }
    if !list.is_null() && tv_list_len(list) != 0 as ::core::ffi::c_int && !what.is_null() {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            gettext(
                b"cannot have both a list and a \"what\" argument\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    incr_quickfix_busy();
    let mut retval: ::core::ffi::c_int = OK;
    if !what.is_null() {
        retval = qf_set_properties(qi, what, action, title);
    } else {
        retval = qf_add_entries(qi, (*qi).qf_curlist, list, title, action);
        if retval == OK {
            qf_list_changed(qf_get_curlist(qi));
        }
    }
    decr_quickfix_busy();
    return retval;
}
unsafe extern "C" fn mark_quickfix_user_data(
    mut qi: *mut qf_info_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*qi).qf_maxcount && !abort_0 {
        let mut qfl: *mut qf_list_T = (*qi).qf_lists.offset(i as isize);
        if (*qfl).qf_has_user_data {
            let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
            let mut j: ::core::ffi::c_int = 0;
            j = 1 as ::core::ffi::c_int;
            qfp = (*qfl).qf_start;
            while !got_int.get() && j <= (*qfl).qf_count && !qfp.is_null() {
                let mut user_data: *mut typval_T = &raw mut (*qfp).qf_user_data;
                if !user_data.is_null()
                    && (*user_data).v_type as ::core::ffi::c_uint
                        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*user_data).v_type as ::core::ffi::c_uint
                        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*user_data).v_type as ::core::ffi::c_uint
                        != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    abort_0 = abort_0 as ::core::ffi::c_int != 0
                        || set_ref_in_item(
                            user_data,
                            copyID,
                            ::core::ptr::null_mut::<*mut ht_stack_T>(),
                            ::core::ptr::null_mut::<*mut list_stack_T>(),
                        ) as ::core::ffi::c_int
                            != 0;
                }
                j += 1;
                qfp = (*qfp).qf_next;
            }
        }
        i += 1;
    }
    return abort_0;
}
unsafe extern "C" fn mark_quickfix_ctx(
    mut qi: *mut qf_info_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*qi).qf_maxcount && !abort_0 {
        let mut ctx: *mut typval_T = (*(*qi).qf_lists.offset(i as isize)).qf_ctx;
        if !ctx.is_null()
            && (*ctx).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*ctx).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*ctx).v_type as ::core::ffi::c_uint
                != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            abort_0 = set_ref_in_item(
                ctx,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            );
        }
        let mut cb: *mut Callback = &raw mut (*(*qi).qf_lists.offset(i as isize)).qf_qftf_cb;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        i += 1;
    }
    return abort_0;
}
pub unsafe extern "C" fn set_ref_in_quickfix(mut copyID: ::core::ffi::c_int) -> bool {
    '_c2rust_label: {
        if !(*ql_info.ptr()).is_null() {
        } else {
            __assert_fail(
                b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                7196 as ::core::ffi::c_uint,
                b"_Bool set_ref_in_quickfix(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if mark_quickfix_ctx(ql_info.get(), copyID) as ::core::ffi::c_int != 0
        || mark_quickfix_user_data(ql_info.get(), copyID) as ::core::ffi::c_int != 0
        || set_ref_in_callback(
            qftf_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0
    {
        return true_0 != 0;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if !(*win).w_llist.is_null() {
                if mark_quickfix_ctx((*win).w_llist, copyID) as ::core::ffi::c_int != 0
                    || mark_quickfix_user_data((*win).w_llist, copyID) as ::core::ffi::c_int != 0
                {
                    return true_0 != 0;
                }
            }
            if bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
                && !(*win).w_llist_ref.is_null()
                && (*(*win).w_llist_ref).qf_refcount == 1 as ::core::ffi::c_int
            {
                if mark_quickfix_ctx((*win).w_llist_ref, copyID) as ::core::ffi::c_int != 0
                    || mark_quickfix_user_data((*win).w_llist_ref, copyID) as ::core::ffi::c_int
                        != 0
                {
                    return true_0 != 0;
                }
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
unsafe extern "C" fn cbuffer_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        55 => {
            return b"cbuffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        69 => {
            return b"cgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        49 => {
            return b"caddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        221 => {
            return b"lbuffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        237 => {
            return b"lgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        217 => {
            return b"laddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
unsafe extern "C" fn cbuffer_process_args(
    mut eap: *mut exarg_T,
    mut bufp: *mut *mut buf_T,
    mut line1: *mut linenr_T,
    mut line2: *mut linenr_T,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        buf = curbuf.get();
    } else if *skipwhite(skipdigits((*eap).arg)) as ::core::ffi::c_int == NUL {
        buf = buflist_findnr(atoi((*eap).arg));
    }
    if buf.is_null() {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return FAIL;
    }
    if (*buf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).line2 = (*buf).b_ml.ml_line_count;
    }
    if (*eap).line1 < 1 as linenr_T
        || (*eap).line1 > (*buf).b_ml.ml_line_count
        || (*eap).line2 < 1 as linenr_T
        || (*eap).line2 > (*buf).b_ml.ml_line_count
    {
        emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
        return FAIL;
    }
    *line1 = (*eap).line1;
    *line2 = (*eap).line2;
    *bufp = buf;
    return OK;
}
pub unsafe extern "C" fn ex_cbuffer(mut eap: *mut exarg_T) {
    let mut au_name: *mut ::core::ffi::c_char = cbuffer_get_auname((*eap).cmdidx);
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return;
        }
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut line1: linenr_T = 0;
    let mut line2: linenr_T = 0;
    if cbuffer_process_args(eap, &raw mut buf, &raw mut line1, &raw mut line2) == FAIL {
        return;
    }
    let mut qf_title: *mut ::core::ffi::c_char = qf_cmdtitle(*(*eap).cmdlinep);
    if !(*buf).b_sfname.is_null() {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"%s (%s)\0".as_ptr() as *const ::core::ffi::c_char,
            qf_title,
            (*buf).b_sfname,
        );
        qf_title = IObuff.ptr() as *mut ::core::ffi::c_char;
    }
    incr_quickfix_busy();
    let mut res: ::core::ffi::c_int = qf_init_ext(
        qi,
        (*qi).qf_curlist,
        ::core::ptr::null::<::core::ffi::c_char>(),
        buf,
        ::core::ptr::null_mut::<typval_T>(),
        p_efm.get(),
        (*eap).cmdidx as ::core::ffi::c_int != CMD_caddbuffer as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddbuffer as ::core::ffi::c_int,
        (*eap).line1,
        (*eap).line2,
        qf_title,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    if qf_stack_empty(qi) {
        decr_quickfix_busy();
        return;
    }
    if res >= 0 as ::core::ffi::c_int {
        qf_list_changed(qf_get_curlist(qi));
    }
    let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
    if !au_name.is_null() {
        let curbuf_old: *const buf_T = curbuf.get();
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        );
        if curbuf.get() != curbuf_old as *mut buf_T {
            res = 0 as ::core::ffi::c_int;
        }
    }
    if res > 0 as ::core::ffi::c_int
        && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cbuffer as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbuffer as ::core::ffi::c_int)
        && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
    {
        qf_jump_first(qi, save_qfid, (*eap).forceit);
    }
    decr_quickfix_busy();
}
unsafe extern "C" fn cexpr_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        64 => {
            return b"cexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        70 => {
            return b"cgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        50 => {
            return b"caddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        232 => {
            return b"lexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        238 => {
            return b"lgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        216 => {
            return b"laddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
unsafe extern "C" fn trigger_cexpr_autocmd(mut cmdidx: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut au_name: *mut ::core::ffi::c_char = cexpr_get_auname(cmdidx as cmdidx_T);
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn cexpr_core(
    mut eap: *const exarg_T,
    mut tv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*tv).vval.v_string.is_null()
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut au_name: *mut ::core::ffi::c_char = cexpr_get_auname((*eap).cmdidx);
        incr_quickfix_busy();
        let mut res: ::core::ffi::c_int = qf_init_ext(
            qi,
            (*qi).qf_curlist,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<buf_T>(),
            tv,
            p_efm.get(),
            (*eap).cmdidx as ::core::ffi::c_int != CMD_caddexpr as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddexpr as ::core::ffi::c_int,
            0 as linenr_T,
            0 as linenr_T,
            qf_cmdtitle(*(*eap).cmdlinep),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        if qf_stack_empty(qi) {
            decr_quickfix_busy();
            return FAIL;
        }
        if res >= 0 as ::core::ffi::c_int {
            qf_list_changed(qf_get_curlist(qi));
        }
        let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
        if !au_name.is_null() {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
        }
        if res > 0 as ::core::ffi::c_int
            && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cexpr as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lexpr as ::core::ffi::c_int)
            && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
        {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
        return OK;
    } else {
        emsg(gettext(
            b"E777: String or List expected\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    return FAIL;
}
pub unsafe extern "C" fn ex_cexpr(mut eap: *mut exarg_T) {
    if trigger_cexpr_autocmd((*eap).cmdidx as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut tv: *mut typval_T = eval_expr((*eap).arg, eap);
    if tv.is_null() {
        return;
    }
    cexpr_core(eap, tv);
    tv_free(tv);
}
unsafe extern "C" fn hgr_get_ll(mut new_ll: *mut bool) -> *mut qf_info_T {
    let mut wp: *mut win_T = if bt_help((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0 {
        curwin.get()
    } else {
        qf_find_help_win()
    };
    let mut qi: *mut qf_info_T = if wp.is_null() {
        ::core::ptr::null_mut::<qf_info_T>()
    } else {
        (*wp).w_llist
    };
    if qi.is_null() {
        qi = qf_alloc_stack(QFLT_LOCATION, 1 as ::core::ffi::c_int);
        *new_ll = true_0 != 0;
    }
    return qi;
}
unsafe extern "C" fn hgr_search_file(
    mut qfl: *mut qf_list_T,
    mut fname: *mut ::core::ffi::c_char,
    mut p_regmatch: *mut regmatch_T,
) {
    let fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        return;
    }
    let mut lnum: linenr_T = 1 as linenr_T;
    while !vim_fgets(IObuff.ptr() as *mut ::core::ffi::c_char, IOSIZE, fd) && !got_int.get() {
        let mut line: *mut ::core::ffi::c_char = IObuff.ptr() as *mut ::core::ffi::c_char;
        if vim_regexec(p_regmatch, line, 0 as colnr_T) {
            let mut l: ::core::ffi::c_int = strlen(line) as ::core::ffi::c_int;
            while l > 0 as ::core::ffi::c_int
                && *line.offset((l - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    <= ' ' as ::core::ffi::c_int
            {
                l -= 1;
                *line.offset(l as isize) = NUL as ::core::ffi::c_char;
            }
            if qf_add_entry(
                qfl,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                line,
                lnum,
                0 as linenr_T,
                (*p_regmatch).startp[0 as ::core::ffi::c_int as usize].offset_from(line)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int,
                (*p_regmatch).endp[0 as ::core::ffi::c_int as usize].offset_from(line)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int,
                false_0 as ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                1 as ::core::ffi::c_char,
                ::core::ptr::null_mut::<typval_T>(),
                true_0 as ::core::ffi::c_char,
            ) == QF_FAIL as ::core::ffi::c_int
            {
                got_int.set(true_0 != 0);
                if line != IObuff.ptr() as *mut ::core::ffi::c_char {
                    xfree(line as *mut ::core::ffi::c_void);
                }
                break;
            }
        }
        if line != IObuff.ptr() as *mut ::core::ffi::c_char {
            xfree(line as *mut ::core::ffi::c_void);
        }
        lnum += 1;
        line_breakcheck();
    }
    fclose(fd);
}
unsafe extern "C" fn hgr_search_files_in_dir(
    mut qfl: *mut qf_list_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut p_regmatch: *mut regmatch_T,
    mut lang: *const ::core::ffi::c_char,
) {
    let mut fcount: ::core::ffi::c_int = 0;
    let mut fnames: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    add_pathsep(dirname);
    strcat(
        dirname,
        b"doc/*.\\(txt\\|??x\\)\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut dirname,
        &raw mut fcount,
        &raw mut fnames,
        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
    ) == OK
        && fcount > 0 as ::core::ffi::c_int
    {
        let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while fi < fcount && !got_int.get() {
            if !(!lang.is_null()
                && strncasecmp(
                    lang as *mut ::core::ffi::c_char,
                    (*fnames.offset(fi as isize))
                        .offset(strlen(*fnames.offset(fi as isize)) as isize)
                        .offset(-(3 as ::core::ffi::c_int as isize)),
                    2 as ::core::ffi::c_int as size_t,
                ) != 0 as ::core::ffi::c_int
                && !(strncasecmp(
                    lang as *mut ::core::ffi::c_char,
                    b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    && strncasecmp(
                        b"txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        (*fnames.offset(fi as isize))
                            .offset(strlen(*fnames.offset(fi as isize)) as isize)
                            .offset(-(3 as ::core::ffi::c_int as isize)),
                        3 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int))
            {
                hgr_search_file(qfl, *fnames.offset(fi as isize), p_regmatch);
            }
            fi += 1;
        }
        FreeWild(fcount, fnames);
    }
}
unsafe extern "C" fn hgr_search_in_rtp(
    mut qfl: *mut qf_list_T,
    mut p_regmatch: *mut regmatch_T,
    mut lang: *const ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = p_rtp.get();
    while *p as ::core::ffi::c_int != NUL && !got_int.get() {
        copy_option_part(
            &raw mut p,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        hgr_search_files_in_dir(
            qfl,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            p_regmatch,
            lang,
        );
    }
}
pub unsafe extern "C" fn ex_helpgrep(mut eap: *mut exarg_T) {
    let mut qi: *mut qf_info_T = ql_info.get();
    '_c2rust_label: {
        if !qi.is_null() {
        } else {
            __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                7575 as ::core::ffi::c_uint,
                b"void ex_helpgrep(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut au_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*eap).cmdidx as ::core::ffi::c_int {
        178 => {
            au_name =
                b"helpgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        241 => {
            au_name =
                b"lhelpgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int
            != 0
    {
        if aborting() {
            return;
        }
    }
    let mut updated: bool = false_0 != 0;
    let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut new_qi: bool = false_0 != 0;
    if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
        qi = hgr_get_ll(&raw mut new_qi);
    }
    incr_quickfix_busy();
    let lang: *mut ::core::ffi::c_char = check_help_lang((*eap).arg);
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: vim_regcomp((*eap).arg, RE_MAGIC + RE_STRING),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false_0 != 0,
    };
    if !regmatch.regprog.is_null() {
        qf_new_list(qi, qf_cmdtitle(*(*eap).cmdlinep));
        let qfl: *mut qf_list_T = qf_get_curlist(qi);
        hgr_search_in_rtp(qfl, &raw mut regmatch, lang);
        vim_regfree(regmatch.regprog);
        (*qfl).qf_nonevalid = false_0 != 0;
        (*qfl).qf_ptr = (*qfl).qf_start;
        (*qfl).qf_index = 1 as ::core::ffi::c_int;
        qf_list_changed(qfl);
        updated = true_0 != 0;
    }
    if p_cpo.get() == empty_string_option.ptr() as *mut ::core::ffi::c_char {
        p_cpo.set(save_cpo);
    } else {
        if *p_cpo.get() as ::core::ffi::c_int == NUL {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(save_cpo),
                    },
                },
                0 as ::core::ffi::c_int,
            );
        }
        free_string_option(save_cpo);
    }
    if updated {
        qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
    }
    if !au_name.is_null() {
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        );
        if !new_qi
            && (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
            && qf_find_win_with_loclist(qi).is_null()
        {
            decr_quickfix_busy();
            return;
        }
    }
    if !qf_list_empty(qf_get_curlist(qi)) {
        qf_jump(
            qi,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0,
        );
    } else {
        semsg(
            gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    decr_quickfix_busy();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_lhelpgrep as ::core::ffi::c_int {
        if !bt_help((*curwin.get()).w_buffer) || (*curwin.get()).w_llist == qi {
            if new_qi {
                ll_free_all(&raw mut qi);
            }
        } else if (*curwin.get()).w_llist.is_null() && new_qi as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_llist = qi;
        }
    }
}
unsafe extern "C" fn get_qf_loc_list(
    mut is_qf: bool,
    mut wp: *mut win_T,
    mut what_arg: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if (*what_arg).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if is_qf as ::core::ffi::c_int != 0 || !wp.is_null() {
            get_errorlist(
                ::core::ptr::null_mut::<qf_info_T>(),
                wp,
                -1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*rettv).vval.v_list,
            );
        }
    } else {
        tv_dict_alloc_ret(rettv);
        if is_qf as ::core::ffi::c_int != 0 || !wp.is_null() {
            if (*what_arg).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut d: *mut dict_T = (*what_arg).vval.v_dict;
                if !d.is_null() {
                    qf_get_properties(wp, d, (*rettv).vval.v_dict);
                }
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
        }
    };
}
pub unsafe extern "C" fn f_getloclist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    get_qf_loc_list(
        false_0 != 0,
        wp,
        argvars.offset(1 as ::core::ffi::c_int as isize),
        rettv,
    );
}
pub unsafe extern "C" fn f_getqflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_qf_loc_list(
        true_0 != 0,
        ::core::ptr::null_mut::<win_T>(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
        rettv,
    );
}
unsafe extern "C" fn set_qf_ll_list(
    mut wp: *mut win_T,
    mut args: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let mut act: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut what_arg: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    static e_invact: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"E927: Invalid action: '%s'\0".as_ptr() as *const ::core::ffi::c_char);
    let mut title: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut action: ::core::ffi::c_char = ' ' as ::core::ffi::c_char;
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut what: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut list_arg: *mut typval_T = args.offset(0 as ::core::ffi::c_int as isize);
    if (*list_arg).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    } else if recursive.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_au_recursive as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut action_arg: *mut typval_T = args.offset(1 as ::core::ffi::c_int as isize);
    if (*action_arg).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*action_arg).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                &raw const e_string_required as *const ::core::ffi::c_char,
            ));
            return;
        }
        act = tv_get_string_chk(action_arg);
        if (*act as ::core::ffi::c_int == 'a' as ::core::ffi::c_int
            || *act as ::core::ffi::c_int == 'r' as ::core::ffi::c_int
            || *act as ::core::ffi::c_int == 'u' as ::core::ffi::c_int
            || *act as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            || *act as ::core::ffi::c_int == 'f' as ::core::ffi::c_int)
            && *act.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            action = *act;
        } else {
            semsg(gettext(e_invact.get()), act);
            return;
        }
        what_arg = args.offset(2 as ::core::ffi::c_int as isize);
        if (*what_arg).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*what_arg).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                title = tv_get_string_chk(what_arg);
                if title.is_null() {
                    return;
                }
            } else if (*what_arg).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*what_arg).vval.v_dict.is_null()
            {
                what = (*what_arg).vval.v_dict;
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                return;
            }
        }
    }
    if title.is_null() {
        title = if !wp.is_null() {
            b":setloclist()\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b":setqflist()\0".as_ptr() as *const ::core::ffi::c_char
        };
    }
    (*recursive.ptr()) += 1;
    let l: *mut list_T = (*list_arg).vval.v_list;
    if set_errorlist(
        wp,
        l,
        action as ::core::ffi::c_int,
        title as *mut ::core::ffi::c_char,
        what,
    ) == OK
    {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    (*recursive.ptr()) -= 1;
}
pub unsafe extern "C" fn f_setloclist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut win: *mut win_T =
        find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if !win.is_null() {
        set_qf_ll_list(win, argvars.offset(1 as ::core::ffi::c_int as isize), rettv);
    }
}
pub unsafe extern "C" fn f_setqflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_qf_ll_list(::core::ptr::null_mut::<win_T>(), argvars, rettv);
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BUF_HAS_QF_ENTRY: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BUF_HAS_LL_ENTRY: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
