use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Array, BoolVarValue, Boolean, BufUpdateCallbacks, CMD_index,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_6, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_3, Dict, Direction, ExtmarkUndoObject, FileID, Float,
    FloatAnchor, FloatRelative, GraphemeState, GridView, Integer, Intersection, KeyValuePair,
    LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkGet, MarkMove, MarkMoveRes, MarkTree,
    MotionType, Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, SpellAddType, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, UIExtension,
    VarLockStatus, VarType, VimState, VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, __compar_fn_t, __off64_t, __off_t, __time_t, alist_T, auto_event, bhdr_T,
    blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdarg_T, cmdidx_T,
    colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_16, dict_T, dictvar_S, diff_T,
    diffblock_S, disptick_T, eslist_T, eslist_elem, event_T, exarg, exarg_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_4,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_1,
    file_buffer_update_channels as C2Rust_Unnamed_2, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T, garray_T, getf_values,
    handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t,
    key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, object,
    object_data as C2Rust_Unnamed_0, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T,
    sattr_T, schar_T, scid_T, sctx_T, searchit_arg_T, size_t, smt_T, state_check_callback,
    state_execute_callback, syn_state, syn_state_sst_union as C2Rust_Unnamed_5, syn_time_T,
    synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_9, u_header_uh_alt_prev as C2Rust_Unnamed_8,
    u_header_uh_next as C2Rust_Unnamed_11, u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, utf8proc_int32_t, varnumber_T, vim_state,
    virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, yankreg_T,
    FILE, NS, QUEUE, _IO_FILE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn time(__timer: *mut time_t) -> time_t;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    static last_cursormoved_win: GlobalCell<*mut win_T>;
    static last_cursormoved: GlobalCell<pos_T>;
    static did_cursorhold: GlobalCell<bool>;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_getfile(
        n: ::core::ffi::c_int,
        lnum: linenr_T,
        options: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fileinfo(fullname: ::core::ffi::c_int, shorthelp: ::core::ffi::c_int, dont_truncate: bool);
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn buf_hide(buf: *const buf_T) -> bool;
    fn deleted_lines(lnum: linenr_T, count: linenr_T);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn ins_char(c: ::core::ffi::c_int);
    fn ins_char_bytes(buf: *mut ::core::ffi::c_char, charlen: size_t);
    fn del_chars(count: ::core::ffi::c_int, fixpos: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn open_line(
        dir: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        second_line_indent: ::core::ffi::c_int,
        did_do_comment: *mut bool,
    ) -> bool;
    fn get_leader_len(
        line: *mut ::core::ffi::c_char,
        flags: *mut *mut ::core::ffi::c_char,
        backward: bool,
        include_space: bool,
    ) -> ::core::ffi::c_int;
    static empty_string_option: GlobalCell<[::core::ffi::c_char; 0]>;
    static cb_flags: GlobalCell<::core::ffi::c_uint>;
    static p_ch: GlobalCell<OptInt>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static fdo_flags: GlobalCell<::core::ffi::c_uint>;
    static p_hls: GlobalCell<::core::ffi::c_int>;
    static jop_flags: GlobalCell<::core::ffi::c_uint>;
    static p_kp: GlobalCell<*mut ::core::ffi::c_char>;
    static p_langmap: GlobalCell<*mut ::core::ffi::c_char>;
    static p_lrm: GlobalCell<::core::ffi::c_int>;
    static p_sbo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    static p_slm: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sbr: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sc: GlobalCell<::core::ffi::c_int>;
    static p_sloc: GlobalCell<*mut ::core::ffi::c_char>;
    static p_smd: GlobalCell<::core::ffi::c_int>;
    static p_scs: GlobalCell<::core::ffi::c_int>;
    static p_sta: GlobalCell<::core::ffi::c_int>;
    static p_to: GlobalCell<::core::ffi::c_int>;
    static p_tm: GlobalCell<OptInt>;
    static p_ttm: GlobalCell<OptInt>;
    static p_ww: GlobalCell<*mut ::core::ffi::c_char>;
    static p_ws: GlobalCell<::core::ffi::c_int>;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strsave_shellescape(
        string: *const ::core::ffi::c_char,
        do_special: bool,
        do_newline: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn init_history();
    fn add_to_history(
        histype: ::core::ffi::c_int,
        new_entry: *const ::core::ffi::c_char,
        new_entrylen: size_t,
        in_map: bool,
        sep: ::core::ffi::c_int,
    );
    fn getviscol() -> ::core::ffi::c_int;
    fn coladvance_force(wcol: colnr_T) -> ::core::ffi::c_int;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn inc_cursor() -> ::core::ffi::c_int;
    fn dec_cursor() -> ::core::ffi::c_int;
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor_col(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn adjust_cursor_col();
    fn set_leftcol(leftcol: colnr_T) -> bool;
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn get_cursor_pos_len() -> colnr_T;
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor_0: bool) -> bool;
    fn win_lines_concealed(wp: *mut win_T) -> bool;
    fn ex_diffupdate(eap: *mut exarg_T);
    fn diff_set_topline(fromwin: *mut win_T, towin: *mut win_T);
    fn nv_diffgetput(put: bool, count: size_t);
    fn diff_move_to(dir: ::core::ffi::c_int, count: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn get_digraph(cmdline: bool) -> ::core::ffi::c_int;
    static diff_need_scrollbind: GlobalCell<bool>;
    fn conceal_check_cursor_line();
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn show_cursor_info_later(force: bool);
    fn showmode() -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn redraw_statuslines();
    fn win_cursorline_standout(wp: *const win_T) -> bool;
    fn edit(cmdchar: ::core::ffi::c_int, startln: bool, count: ::core::ffi::c_int) -> bool;
    fn prompt_curpos_editable() -> bool;
    fn get_literal(no_simplify: bool) -> ::core::ffi::c_int;
    fn set_last_insert(c: ::core::ffi::c_int);
    fn beginline(flags: ::core::ffi::c_int);
    fn oneright() -> ::core::ffi::c_int;
    fn oneleft() -> ::core::ffi::c_int;
    fn cursor_up_inner(wp: *mut win_T, n: linenr_T, skip_conceal: bool);
    fn cursor_up(n: linenr_T, upd_topline: bool) -> ::core::ffi::c_int;
    fn cursor_down_inner(wp: *mut win_T, n: ::core::ffi::c_int, skip_conceal: bool);
    fn cursor_down(n: ::core::ffi::c_int, upd_topline: bool) -> ::core::ffi::c_int;
    fn ins_copychar(lnum: linenr_T) -> ::core::ffi::c_int;
    static e_modifiable: [::core::ffi::c_char; 0];
    static e_noident: [::core::ffi::c_char; 0];
    fn prompt_invoke_callback();
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn set_reg_var(c: ::core::ffi::c_int);
    fn set_vcount(count: int64_t, count1: int64_t, set_prevcount: bool);
    fn do_ascii(eap: *mut exarg_T);
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn autowrite(buf: *mut buf_T, forceit: bool) -> ::core::ffi::c_int;
    fn do_exmode();
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn do_sleep(msec: int64_t, hide_cursor: bool);
    fn discard_current_exception();
    fn getcmdline(
        firstc: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn text_locked() -> bool;
    fn text_locked_msg();
    fn curbuf_locked() -> bool;
    fn getexline(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn compute_cmdrow();
    fn vim_strsave_fnameescape(
        fname: *const ::core::ffi::c_char,
        what: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn grab_file_name(
        count: ::core::ffi::c_int,
        file_lnum: *mut linenr_T,
    ) -> *mut ::core::ffi::c_char;
    fn check_timestamps(focus: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn hasAnyFolding(win: *mut win_T) -> ::core::ffi::c_int;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn foldmethodIsManual(wp: *mut win_T) -> bool;
    fn foldmethodIsMarker(wp: *mut win_T) -> bool;
    fn foldmethodIsDiff(wp: *mut win_T) -> bool;
    fn closeFold(pos: pos_T, count: ::core::ffi::c_int);
    fn closeFoldRecurse(pos: pos_T);
    fn openFold(pos: pos_T, count: ::core::ffi::c_int);
    fn openFoldRecurse(pos: pos_T);
    fn foldOpenCursor();
    fn newFoldLevel();
    fn foldCheckClose();
    fn foldManualAllowed(create: bool) -> ::core::ffi::c_int;
    fn deleteFold(
        wp: *mut win_T,
        start: linenr_T,
        end: linenr_T,
        recursive: ::core::ffi::c_int,
        had_visual: bool,
    );
    fn clearFolding(win: *mut win_T);
    fn foldUpdateAfterInsert();
    fn foldMoveTo(
        updown: bool,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn foldAdjustVisual();
    fn getDeepestNesting(wp: *mut win_T) -> ::core::ffi::c_int;
    fn stuff_empty() -> bool;
    fn readbuf1_empty() -> bool;
    fn beep_flush();
    fn ResetRedobuff();
    fn AppendToRedobuff(s: *const ::core::ffi::c_char);
    fn AppendCharToRedobuff(c: ::core::ffi::c_int);
    fn AppendNumberToRedobuff(n: ::core::ffi::c_int);
    fn stuffReadbuff(s: *const ::core::ffi::c_char);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn stuffnumReadbuff(n: ::core::ffi::c_int);
    fn start_redo(count: ::core::ffi::c_int, old_redo: bool) -> ::core::ffi::c_int;
    fn ins_char_typebuf(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
        on_key_ignore: bool,
    ) -> ::core::ffi::c_int;
    fn typebuf_typed() -> ::core::ffi::c_int;
    fn typebuf_maplen() -> ::core::ffi::c_int;
    fn gotchars_ignore();
    fn ungetchars(len: ::core::ffi::c_int);
    fn vgetc() -> ::core::ffi::c_int;
    fn safe_vgetc() -> ::core::ffi::c_int;
    fn plain_vgetc() -> ::core::ffi::c_int;
    fn vpeekc() -> ::core::ffi::c_int;
    fn char_avail() -> bool;
    fn vungetc(c: ::core::ffi::c_int);
    fn getcmdkeycmd(
        promptc: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn map_execute_lua(may_repeat: bool, discard: bool) -> bool;
    fn paste_repeat(count: ::core::ffi::c_int);
    static Rows: GlobalCell<::core::ffi::c_int>;
    static mod_mask: GlobalCell<::core::ffi::c_int>;
    static vgetc_mod_mask: GlobalCell<::core::ffi::c_int>;
    static vgetc_char: GlobalCell<::core::ffi::c_int>;
    static redraw_cmdline: GlobalCell<bool>;
    static redraw_mode: GlobalCell<bool>;
    static clear_cmdline: GlobalCell<bool>;
    static mode_displayed: GlobalCell<bool>;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static keep_msg: GlobalCell<*mut ::core::ffi::c_char>;
    static keep_msg_hl_id: GlobalCell<::core::ffi::c_int>;
    static need_fileinfo: GlobalCell<bool>;
    static msg_scroll: GlobalCell<::core::ffi::c_int>;
    static msg_didout: GlobalCell<bool>;
    static msg_didany: GlobalCell<bool>;
    static msg_nowait: GlobalCell<bool>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static msg_hist_off: GlobalCell<bool>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static emsg_on_display: GlobalCell<bool>;
    static need_wait_return: GlobalCell<bool>;
    static did_wait_return: GlobalCell<bool>;
    static quit_more: GlobalCell<bool>;
    static vgetc_busy: GlobalCell<::core::ffi::c_int>;
    static did_throw: GlobalCell<bool>;
    static may_garbage_collect: GlobalCell<bool>;
    static no_smartcase: GlobalCell<bool>;
    static need_check_timestamps: GlobalCell<bool>;
    static did_check_timestamps: GlobalCell<bool>;
    static mouse_dragging: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static redraw_tabline: GlobalCell<bool>;
    static curbuf: GlobalCell<*mut buf_T>;
    static sc_col: GlobalCell<::core::ffi::c_int>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_select: GlobalCell<bool>;
    static VIsual_select_reg: GlobalCell<::core::ffi::c_int>;
    static VIsual_select_exclu_adj: GlobalCell<bool>;
    static restart_VIsual_select: GlobalCell<::core::ffi::c_int>;
    static VIsual_reselect: GlobalCell<::core::ffi::c_int>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static resel_VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static resel_VIsual_line_count: GlobalCell<linenr_T>;
    static resel_VIsual_vcol: GlobalCell<colnr_T>;
    static did_syncbind: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    static finish_op: GlobalCell<bool>;
    static opcount: GlobalCell<::core::ffi::c_int>;
    static motion_force: GlobalCell<::core::ffi::c_int>;
    static exmode_active: GlobalCell<bool>;
    static reg_recording: GlobalCell<::core::ffi::c_int>;
    static reg_executing: GlobalCell<::core::ffi::c_int>;
    static reg_recorded: GlobalCell<::core::ffi::c_int>;
    static no_mapping: GlobalCell<::core::ffi::c_int>;
    static no_zero_mapping: GlobalCell<::core::ffi::c_int>;
    static allow_keys: GlobalCell<::core::ffi::c_int>;
    static no_u_sync: GlobalCell<::core::ffi::c_int>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static arrow_used: GlobalCell<bool>;
    static ins_at_eol: GlobalCell<bool>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static emsg_silent: GlobalCell<::core::ffi::c_int>;
    static in_assert_fails: GlobalCell<bool>;
    static typebuf_was_empty: GlobalCell<bool>;
    static ex_normal_busy: GlobalCell<::core::ffi::c_int>;
    static KeyTyped: GlobalCell<bool>;
    static KeyStuffed: GlobalCell<::core::ffi::c_int>;
    static must_redraw: GlobalCell<::core::ffi::c_int>;
    static skip_redraw: GlobalCell<bool>;
    static do_redraw: GlobalCell<bool>;
    static got_int: GlobalCell<bool>;
    static global_busy: GlobalCell<::core::ffi::c_int>;
    static g_tag_at_cursor: GlobalCell<bool>;
    static langmap_mapchar: GlobalCell<[uint8_t; 256]>;
    static km_stopsel: GlobalCell<bool>;
    static km_startsel: GlobalCell<bool>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static cmdwin_result: GlobalCell<::core::ffi::c_int>;
    static no_hlsearch: GlobalCell<bool>;
    static time_fd: GlobalCell<*mut FILE>;
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_flush();
    fn ex_help(eap: *mut exarg_T);
    static ns_hl_fast: GlobalCell<NS>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn simplify_key(
        key: ::core::ffi::c_int,
        modifiers: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn add_map(
        lhs: *mut ::core::ffi::c_char,
        rhs: *mut ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        buffer: bool,
    );
    fn langmap_adjust_mb(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn setmark(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn setpcmark();
    fn checkpcmark();
    fn get_jumplist(win: *mut win_T, count: ::core::ffi::c_int) -> *mut fmark_T;
    fn get_changelist(buf: *mut buf_T, win: *mut win_T, count: ::core::ffi::c_int) -> *mut fmark_T;
    fn mark_get(
        buf: *mut buf_T,
        win: *mut win_T,
        fmp: *mut fmark_T,
        flag: MarkGet,
        name: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn pos_to_mark(buf: *mut buf_T, fmp: *mut fmark_T, pos: pos_T) -> *mut fmark_T;
    fn mark_move_to(fm: *mut fmark_T, flags: MarkMove) -> MarkMoveRes;
    fn getnextmark(
        startpos: *mut pos_T,
        dir: ::core::ffi::c_int,
        begin_line: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn mark_mb_adjustpos(buf: *mut buf_T, lp: *mut pos_T);
    fn mb_get_class(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_iscomposing(
        c1: ::core::ffi::c_int,
        c2: ::core::ffi::c_int,
        state: *mut GraphemeState,
    ) -> bool;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn show_utf8();
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utf_find_illegal();
    fn mb_adjust_cursor();
    fn mb_check_adjust_col(win_: *mut ::core::ffi::c_void);
    fn mb_prevptr(
        line: *mut ::core::ffi::c_char,
        p: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    static utf8len_tab: [uint8_t; 256];
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_pos(pos: *const pos_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_delete_flags(lnum: linenr_T, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn goto_byte(cnt: ::core::ffi::c_int);
    fn inc(lp: *mut pos_T) -> ::core::ffi::c_int;
    static msg_grid_adj: GlobalCell<GridView>;
    fn msg_grid_validate();
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn wait_return(redraw: ::core::ffi::c_int);
    fn messaging() -> bool;
    fn msg_ext_set_trigger(trigger: *const ::core::ffi::c_char);
    fn may_clear_sb_text();
    fn show_sb_text();
    fn msg_delay(ms: uint64_t, ignoreinput: bool);
    fn do_mouse(
        oap: *mut oparg_T,
        c: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        fixindent: bool,
    ) -> bool;
    fn nv_mousescroll(cap: *mut cmdarg_T);
    fn nv_mouse(cap: *mut cmdarg_T);
    fn setmouse();
    fn sms_marker_overlap(wp: *mut win_T, extra2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn update_curswant_force();
    fn update_curswant();
    fn changed_window_setting(wp: *mut win_T);
    fn validate_botline_win(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn validate_virtcol(wp: *mut win_T);
    fn validate_cheight(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_col_off2(wp: *mut win_T) -> ::core::ffi::c_int;
    fn scroll_redraw(up: ::core::ffi::c_int, count: linenr_T);
    fn scrolldown(wp: *mut win_T, line_count: linenr_T, byfold: ::core::ffi::c_int) -> bool;
    fn scrollup(wp: *mut win_T, line_count: linenr_T, byfold: bool) -> bool;
    fn adjust_skipcol();
    fn scroll_cursor_top(
        wp: *mut win_T,
        min_scroll: ::core::ffi::c_int,
        always: ::core::ffi::c_int,
    );
    fn scroll_cursor_bot(wp: *mut win_T, min_scroll: ::core::ffi::c_int, set_topbot: bool);
    fn scroll_cursor_halfway(wp: *mut win_T, atend: bool, prefer_above: bool);
    fn cursor_correct(wp: *mut win_T);
    fn pagescroll(dir: Direction, count: ::core::ffi::c_int, half: bool) -> ::core::ffi::c_int;
    fn do_check_cursorbind();
    static showcmd_buf: GlobalCell<[::core::ffi::c_char; 41]>;
    fn get_op_type(char1: ::core::ffi::c_int, char2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn op_is_change(op: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn get_op_char(optype: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn get_extra_op_char(optype: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn swapchar(op_type: ::core::ffi::c_int, pos: *mut pos_T) -> bool;
    fn adjust_cursor_eol();
    fn do_join(
        count: size_t,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark_0: bool,
    ) -> ::core::ffi::c_int;
    fn op_addsub(oap: *mut oparg_T, Prenum1: linenr_T, g_cmd: bool);
    fn clear_oparg(oap: *mut oparg_T);
    fn cursor_pos_info(dict: *mut dict_T);
    fn do_pending_operator(cap: *mut cmdarg_T, old_col: ::core::ffi::c_int, gui_yank: bool);
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn magic_isset() -> bool;
    fn get_ve_flags(wp: *mut win_T) -> ::core::ffi::c_uint;
    fn get_showbreak_value(win: *mut win_T) -> *mut ::core::ffi::c_char;
    fn get_sidescrolloff_value(wp: *mut win_T) -> int64_t;
    fn line_breakcheck();
    fn linetabsize(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvcols(
        wp: *mut win_T,
        pos1: *mut pos_T,
        pos2: *mut pos_T,
        left: *mut colnr_T,
        right: *mut colnr_T,
    );
    fn win_get_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn plines_win(wp: *mut win_T, lnum: linenr_T, limit_winheight: bool) -> ::core::ffi::c_int;
    fn plines_m_win_fill(wp: *mut win_T, first: linenr_T, last: linenr_T) -> ::core::ffi::c_int;
    fn time_msg(mesg: *const ::core::ffi::c_char, start: *const proftime_T);
    fn time_finish();
    fn qf_view_result(split: bool);
    fn get_expr_register() -> ::core::ffi::c_int;
    fn valid_yank_reg(regname: ::core::ffi::c_int, writing: bool) -> bool;
    fn get_default_register_name() -> ::core::ffi::c_int;
    fn copy_register(name: ::core::ffi::c_int) -> *mut yankreg_T;
    fn do_record(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn do_execreg(
        regname: ::core::ffi::c_int,
        colon: ::core::ffi::c_int,
        addcr: ::core::ffi::c_int,
        silent: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn free_register(reg: *mut yankreg_T);
    fn do_put(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    );
    fn spell_add_word(
        word: *mut ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        what: SpellAddType,
        idx: ::core::ffi::c_int,
        undo: bool,
    );
    fn spell_move_to(
        wp: *mut win_T,
        dir: ::core::ffi::c_int,
        behaviour: smt_T,
        curline: bool,
        attrp: *mut hlf_T,
    ) -> size_t;
    fn state_enter(s: *mut VimState);
    fn state_handle_k_event();
    fn virtual_active(wp: *mut win_T) -> bool;
    fn get_real_state() -> ::core::ffi::c_int;
    fn may_trigger_modechanged();
    fn may_trigger_safestate(safe: bool);
    fn state_no_longer_safe(reason: *const ::core::ffi::c_char);
    fn spell_suggest(count: ::core::ffi::c_int);
    fn win_redr_status(wp: *mut win_T);
    fn draw_tabline();
    fn syn_stack_free_all(block: *mut synblock_T);
    fn do_tag(
        tag: *mut ::core::ffi::c_char,
        type_0: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
        verbose: bool,
    );
    fn terminal_check_refresh();
    fn has_format_option(x: ::core::ffi::c_int) -> bool;
    fn auto_format(trailblank: bool, prev_line: bool);
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_flush();
    fn ui_cursor_shape_no_check_conceal();
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn findsent(dir: Direction, count: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn findpar(
        pincl: *mut bool,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        what: ::core::ffi::c_int,
        both: bool,
    ) -> bool;
    fn fwd_word(count: ::core::ffi::c_int, bigword: bool, eol: bool) -> ::core::ffi::c_int;
    fn bck_word(count: ::core::ffi::c_int, bigword: bool, stop: bool) -> ::core::ffi::c_int;
    fn end_word(
        count: ::core::ffi::c_int,
        bigword: bool,
        stop: bool,
        empty: bool,
    ) -> ::core::ffi::c_int;
    fn bckend_word(count: ::core::ffi::c_int, bigword: bool, eol: bool) -> ::core::ffi::c_int;
    fn current_word(
        oap: *mut oparg_T,
        count: ::core::ffi::c_int,
        include: bool,
        bigword: bool,
    ) -> ::core::ffi::c_int;
    fn current_sent(
        oap: *mut oparg_T,
        count: ::core::ffi::c_int,
        include: bool,
    ) -> ::core::ffi::c_int;
    fn current_block(
        oap: *mut oparg_T,
        count: ::core::ffi::c_int,
        include: bool,
        what: ::core::ffi::c_int,
        other: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn current_tagblock(
        oap: *mut oparg_T,
        count_arg: ::core::ffi::c_int,
        include: bool,
    ) -> ::core::ffi::c_int;
    fn current_par(
        oap: *mut oparg_T,
        count: ::core::ffi::c_int,
        include: bool,
        type_0: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn current_quote(
        oap: *mut oparg_T,
        count: ::core::ffi::c_int,
        include: bool,
        quotechar: ::core::ffi::c_int,
    ) -> bool;
    fn ui_call_msg_showcmd(content: Array);
    fn u_save_cursor() -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn u_savesub(lnum: linenr_T) -> ::core::ffi::c_int;
    fn u_undo(count: ::core::ffi::c_int);
    fn u_redo(count: ::core::ffi::c_int);
    fn undo_time(step: ::core::ffi::c_int, sec: bool, file: bool, absolute: bool);
    fn u_clearline(buf: *mut buf_T);
    fn u_undoline();
    fn anyBufIsChanged() -> bool;
    fn curbufIsChanged() -> bool;
    fn check_can_set_curbuf_disabled() -> bool;
    fn do_window(nchar: ::core::ffi::c_int, Prenum: ::core::ffi::c_int, xchar: ::core::ffi::c_int);
    fn goto_tabpage(n: ::core::ffi::c_int);
    fn goto_tabpage_lastused() -> bool;
    fn may_make_initial_scroll_size_snapshot();
    fn may_trigger_win_scrolled_resized();
    fn win_setheight(height: ::core::ffi::c_int);
    fn set_fraction(wp: *mut win_T);
    fn reset_search_dir();
    fn searchit(
        win: *mut win_T,
        buf: *mut buf_T,
        pos: *mut pos_T,
        end_pos: *mut pos_T,
        dir: Direction,
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        count: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        pat_use: ::core::ffi::c_int,
        extra_arg: *mut searchit_arg_T,
    ) -> ::core::ffi::c_int;
    fn do_search(
        oap: *mut oparg_T,
        dirc: ::core::ffi::c_int,
        search_delim: ::core::ffi::c_int,
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        count: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        sia: *mut searchit_arg_T,
    ) -> ::core::ffi::c_int;
    fn searchc(cap: *mut cmdarg_T, t_cmd: bool) -> ::core::ffi::c_int;
    fn findmatch(oap: *mut oparg_T, initc: ::core::ffi::c_int) -> *mut pos_T;
    fn findmatchlimit(
        oap: *mut oparg_T,
        initc: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        maxtravel: int64_t,
    ) -> *mut pos_T;
    fn current_search(count: ::core::ffi::c_int, forward: bool) -> ::core::ffi::c_int;
    fn find_pattern_in_path(
        ptr: *mut ::core::ffi::c_char,
        dir: Direction,
        len: size_t,
        whole: bool,
        skip_comments: bool,
        type_0: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        action: ::core::ffi::c_int,
        start_lnum: linenr_T,
        end_lnum: linenr_T,
        forceit: bool,
        silent: bool,
    );
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
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
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kMarkChangedView: MarkMoveRes = 64;
pub const kMarkChangedCursor: MarkMoveRes = 32;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkChangedCol: MarkMoveRes = 8;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub const kMarkJumpList: MarkMove = 16;
pub const kMarkSetView: MarkMove = 8;
pub const KMarkNoContext: MarkMove = 4;
pub const kMarkContext: MarkMove = 2;
pub const kMarkBeginLine: MarkMove = 1;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
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
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_17 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_17 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_17 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_17 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_17 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_17 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_18 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_18 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_18 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_18 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_18 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_18 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_18 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_18 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_18 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_18 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_18 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_18 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_18 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_18 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_18 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_18 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_18 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_18 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_18 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptCbFlagUnnamedplus: C2Rust_Unnamed_19 = 2;
pub const kOptCbFlagUnnamed: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_20 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_20 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_20 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_20 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_20 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_20 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_20 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_20 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_20 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_20 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptJopFlagClean: C2Rust_Unnamed_21 = 4;
pub const kOptJopFlagView: C2Rust_Unnamed_21 = 2;
pub const kOptJopFlagStack: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_22 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_22 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_22 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_22 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_22 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_22 = 4;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_23 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_23 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_23 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_23 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_23 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_23 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_23 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_23 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_23 = 79;
pub const SHM_OVER: C2Rust_Unnamed_23 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_23 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_23 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_23 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_23 = 97;
pub const SHM_WRI: C2Rust_Unnamed_23 = 119;
pub const SHM_LINES: C2Rust_Unnamed_23 = 108;
pub const SHM_MOD: C2Rust_Unnamed_23 = 109;
pub const SHM_RO: C2Rust_Unnamed_23 = 114;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub const HIST_DEBUG: C2Rust_Unnamed_24 = 4;
pub const HIST_INPUT: C2Rust_Unnamed_24 = 3;
pub const HIST_EXPR: C2Rust_Unnamed_24 = 2;
pub const HIST_SEARCH: C2Rust_Unnamed_24 = 1;
pub const HIST_CMD: C2Rust_Unnamed_24 = 0;
pub const HIST_INVALID: C2Rust_Unnamed_24 = -1;
pub const HIST_DEFAULT: C2Rust_Unnamed_24 = -2;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_25 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_25 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_25 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_25 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_25 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_25 = 20;
pub const UPD_VALID: C2Rust_Unnamed_25 = 10;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_26 = 4;
pub const BL_SOL: C2Rust_Unnamed_26 = 2;
pub const BL_WHITE: C2Rust_Unnamed_26 = 1;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_27 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_27 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_27 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_27 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_27 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_27 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_28 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_28 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_28 = 0;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_29 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_29 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_29 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_29 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_29 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const VSE_BUFFER: C2Rust_Unnamed_30 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_30 = 1;
pub const VSE_NONE: C2Rust_Unnamed_30 = 0;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_31 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_31 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_31 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_31 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_31 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_31 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_31 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_31 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_31 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_31 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_31 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_31 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_31 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_31 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_31 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_31 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_31 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_31 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_31 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_31 = 1;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_32 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const CA_NO_ADJ_OP_END: C2Rust_Unnamed_33 = 2;
pub const CA_COMMAND_BUSY: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_int;
pub const REPLACE_NL_NCHAR: C2Rust_Unnamed_34 = -2;
pub const REPLACE_CR_NCHAR: C2Rust_Unnamed_34 = -1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const SHOWCMD_COLS: C2Rust_Unnamed_35 = 10;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const SHOWCMD_BUFLEN: C2Rust_Unnamed_36 = 41;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_37 = -2;
pub const MSCR_LEFT: C2Rust_Unnamed_37 = -1;
pub const MSCR_UP: C2Rust_Unnamed_37 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_37 = 0;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_38 = 4;
pub const FIND_STRING: C2Rust_Unnamed_38 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_38 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nv_cmd {
    pub cmd_char: ::core::ffi::c_int,
    pub cmd_func: nv_func_T,
    pub cmd_flags: uint16_t,
    pub cmd_arg: int16_t,
}
pub type nv_func_T = Option<unsafe extern "C" fn(*mut cmdarg_T) -> ()>;
pub const OP_NOP: C2Rust_Unnamed_40 = 0;
pub const OP_YANK: C2Rust_Unnamed_40 = 2;
pub const OP_RSHIFT: C2Rust_Unnamed_40 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_40 = 4;
pub const OP_DELETE: C2Rust_Unnamed_40 = 1;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_39 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_39 = 16;
pub const PUT_LINE: C2Rust_Unnamed_39 = 8;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_39 = 64;
pub const PUT_CURSEND: C2Rust_Unnamed_39 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_39 = 1;
pub const SEARCH_START: C2Rust_Unnamed_43 = 256;
pub const FM_FORWARD: C2Rust_Unnamed_44 = 2;
pub const RE_LAST: C2Rust_Unnamed_45 = 2;
pub const SEARCH_MSG: C2Rust_Unnamed_43 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_43 = 2;
pub const SEARCH_OPT: C2Rust_Unnamed_43 = 16;
pub const OP_CHANGE: C2Rust_Unnamed_40 = 3;
pub const OP_NR_SUB: C2Rust_Unnamed_40 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_40 = 28;
pub const OP_TILDE: C2Rust_Unnamed_40 = 7;
pub const SPELL_ADD_RARE: SpellAddType = 2;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub const OP_FOLD: C2Rust_Unnamed_40 = 19;
pub const OP_LOWER: C2Rust_Unnamed_40 = 12;
pub const OP_FORMAT: C2Rust_Unnamed_40 = 9;
pub const SEARCH_MARK: C2Rust_Unnamed_43 = 512;
pub const FM_BACKWARD: C2Rust_Unnamed_44 = 1;
pub const ACTION_GOTO: C2Rust_Unnamed_42 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_42 = 1;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_42 = 4;
pub const FIND_ANY: C2Rust_Unnamed_41 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_41 = 2;
pub const OP_UPPER: C2Rust_Unnamed_40 = 11;
pub const SEARCH_REV: C2Rust_Unnamed_43 = 1;
pub const OP_ROT13: C2Rust_Unnamed_40 = 15;
pub const DT_POP: C2Rust_Unnamed_46 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct NormalState {
    pub state: VimState,
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: oparg_T,
    pub ca: cmdarg_T,
    pub mapped_len: ::core::ffi::c_int,
    pub old_mapped_len: ::core::ffi::c_int,
    pub idx: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub old_col: ::core::ffi::c_int,
    pub old_pos: pos_T,
}
pub const OP_COLON: C2Rust_Unnamed_40 = 10;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const PUT_CURSLINE: C2Rust_Unnamed_39 = 4;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const OP_FUNCTION: C2Rust_Unnamed_40 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_40 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_40 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_40 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_40 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_40 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_40 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_40 = 20;
pub const OP_APPEND: C2Rust_Unnamed_40 = 18;
pub const OP_INSERT: C2Rust_Unnamed_40 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_40 = 16;
pub const OP_JOIN_NS: C2Rust_Unnamed_40 = 14;
pub const OP_JOIN: C2Rust_Unnamed_40 = 13;
pub const OP_INDENT: C2Rust_Unnamed_40 = 8;
pub const OP_FILTER: C2Rust_Unnamed_40 = 6;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_41 = 3;
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_42 = 5;
pub const ACTION_SPLIT: C2Rust_Unnamed_42 = 3;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_43 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_43 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_43 = 1024;
pub const SEARCH_NOOF: C2Rust_Unnamed_43 = 128;
pub const SEARCH_END: C2Rust_Unnamed_43 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_43 = 32;
pub const SEARCH_NFMSG: C2Rust_Unnamed_43 = 8;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_44 = 8;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_44 = 4;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const RE_BOTH: C2Rust_Unnamed_45 = 2;
pub const RE_SUBST: C2Rust_Unnamed_45 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_45 = 0;
pub type C2Rust_Unnamed_46 = ::core::ffi::c_uint;
pub const DT_FREE: C2Rust_Unnamed_46 = 99;
pub const DT_LTAG: C2Rust_Unnamed_46 = 11;
pub const DT_JUMP: C2Rust_Unnamed_46 = 9;
pub const DT_HELP: C2Rust_Unnamed_46 = 8;
pub const DT_SELECT: C2Rust_Unnamed_46 = 7;
pub const DT_LAST: C2Rust_Unnamed_46 = 6;
pub const DT_FIRST: C2Rust_Unnamed_46 = 5;
pub const DT_PREV: C2Rust_Unnamed_46 = 4;
pub const DT_NEXT: C2Rust_Unnamed_46 = 3;
pub const DT_TAG: C2Rust_Unnamed_46 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn lt(mut a: pos_T, mut b: pos_T) -> bool {
    if a.lnum != b.lnum {
        return a.lnum < b.lnum;
    } else if a.col != b.col {
        return a.col < b.col;
    } else {
        return a.coladd < b.coladd;
    };
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const POUND: ::core::ffi::c_int = 0xa3 as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const Ctrl_B: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const Ctrl_E: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_I: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const Ctrl_K: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const Ctrl_L: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_Q: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_S: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const Ctrl_HAT: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub const Ctrl__: ::core::ffi::c_int = 31 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0 || c == NUL;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const FO_OPEN_COMS: ::core::ffi::c_int = 'o' as ::core::ffi::c_int;
pub const CPO_DIGRAPH: ::core::ffi::c_int = 'D' as ::core::ffi::c_int;
pub const CPO_CHANGEW: ::core::ffi::c_int = '_' as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active.get()
    }
    .offset(hlf as isize);
}
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_DOWN: ::core::ffi::c_int = -25707;
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_LEFT: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_RIGHT: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('i' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_HOME: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_END: ::core::ffi::c_int =
    -('*' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F1: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HELP: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UNDO: ::core::ffi::c_int =
    -('&' as ::core::ffi::c_int + (('8' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_INS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DEL: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HOME: ::core::ffi::c_int = -26731;
pub const K_KHOME: ::core::ffi::c_int = -12619;
pub const K_END: ::core::ffi::c_int =
    -('@' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KEND: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEUP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('P' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEDOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEUP: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEDOWN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KENTER: ::core::ffi::c_int = -16715;
pub const K_PASTE_START: ::core::ffi::c_int =
    -('P' as ::core::ffi::c_int + (('S' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_SELECT: ::core::ffi::c_int =
    -(245 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static VIsual_mode_orig: GlobalCell<::core::ffi::c_int> = GlobalCell::new(NUL);
static e_changelist_is_empty: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E664: Changelist is empty\0")
});
static e_cmdline_window_already_open: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E1292: Command-line window is already open\0",
        )
    });
#[inline]
unsafe extern "C" fn normal_state_init(mut s: *mut NormalState) {
    memset(
        s as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<NormalState>(),
    );
    (*s).state.check =
        Some(normal_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
            as state_check_callback;
    (*s).state.execute = Some(
        normal_execute
            as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
    ) as state_execute_callback;
}
pub const NV_NCH: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NV_NCH_NOP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int | NV_NCH;
pub const NV_NCH_ALW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int | NV_NCH;
pub const NV_LANG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const NV_SS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const NV_SSS: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const NV_STS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const NV_RL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const NV_KEEPREG: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const NV_NCW: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
static nv_cmds: GlobalCell<[nv_cmd; 188]> = GlobalCell::new([
    nv_cmd {
        cmd_char: NUL,
        cmd_func: Some(nv_error as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_A,
        cmd_func: Some(nv_addsub as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_B,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_STS as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_C,
        cmd_func: Some(nv_esc as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_D,
        cmd_func: Some(nv_halfpage as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_E,
        cmd_func: Some(nv_scroll_line as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_F,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_STS as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_G,
        cmd_func: Some(nv_ctrlg as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_H,
        cmd_func: Some(nv_ctrlh as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_I,
        cmd_func: Some(nv_pcmark as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: NL,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_K,
        cmd_func: Some(nv_error as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_L,
        cmd_func: Some(nv_clear as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: CAR,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_N,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_STS as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_O,
        cmd_func: Some(nv_ctrlo as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_P,
        cmd_func: Some(nv_up as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_STS as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_Q,
        cmd_func: Some(nv_visual as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_R,
        cmd_func: Some(nv_redo_or_register as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_S,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_T,
        cmd_func: Some(nv_tagpop as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_U,
        cmd_func: Some(nv_halfpage as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_V,
        cmd_func: Some(nv_visual as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'V' as ::core::ffi::c_int,
        cmd_func: Some(nv_visual as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'v' as ::core::ffi::c_int,
        cmd_func: Some(nv_visual as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_W,
        cmd_func: Some(nv_window as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_X,
        cmd_func: Some(nv_addsub as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_Y,
        cmd_func: Some(nv_scroll_line as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_Z,
        cmd_func: Some(nv_suspend as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: ESC,
        cmd_func: Some(nv_esc as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_BSL,
        cmd_func: Some(nv_normal as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_RSB,
        cmd_func: Some(nv_ident as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl_HAT,
        cmd_func: Some(nv_hat as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: Ctrl__,
        cmd_func: Some(nv_error as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: ' ' as ::core::ffi::c_int,
        cmd_func: Some(nv_right as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '!' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '"' as ::core::ffi::c_int,
        cmd_func: Some(nv_regname as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_NOP | NV_KEEPREG) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '#' as ::core::ffi::c_int,
        cmd_func: Some(nv_ident as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '$' as ::core::ffi::c_int,
        cmd_func: Some(nv_dollar as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '%' as ::core::ffi::c_int,
        cmd_func: Some(nv_percent as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '&' as ::core::ffi::c_int,
        cmd_func: Some(nv_optrans as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '\'' as ::core::ffi::c_int,
        cmd_func: Some(nv_gomark as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '(' as ::core::ffi::c_int,
        cmd_func: Some(nv_brace as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: ')' as ::core::ffi::c_int,
        cmd_func: Some(nv_brace as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: '*' as ::core::ffi::c_int,
        cmd_func: Some(nv_ident as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '+' as ::core::ffi::c_int,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: ',' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '-' as ::core::ffi::c_int,
        cmd_func: Some(nv_up as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '.' as ::core::ffi::c_int,
        cmd_func: Some(nv_dot as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '/' as ::core::ffi::c_int,
        cmd_func: Some(nv_search as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '0' as ::core::ffi::c_int,
        cmd_func: Some(nv_beginline as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '1' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '2' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '3' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '4' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '5' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '6' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '7' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '8' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '9' as ::core::ffi::c_int,
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: ':' as ::core::ffi::c_int,
        cmd_func: Some(nv_colon as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: ';' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '<' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_RL as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '=' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '>' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_RL as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '?' as ::core::ffi::c_int,
        cmd_func: Some(nv_search as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: '@' as ::core::ffi::c_int,
        cmd_func: Some(nv_at as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_NOP as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'A' as ::core::ffi::c_int,
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'B' as ::core::ffi::c_int,
        cmd_func: Some(nv_bck_word as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 1 as int16_t,
    },
    nv_cmd {
        cmd_char: 'C' as ::core::ffi::c_int,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'D' as ::core::ffi::c_int,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'E' as ::core::ffi::c_int,
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'F' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_ALW | NV_LANG) as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: 'G' as ::core::ffi::c_int,
        cmd_func: Some(nv_goto as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'H' as ::core::ffi::c_int,
        cmd_func: Some(nv_scroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'I' as ::core::ffi::c_int,
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'J' as ::core::ffi::c_int,
        cmd_func: Some(nv_join as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'K' as ::core::ffi::c_int,
        cmd_func: Some(nv_ident as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'L' as ::core::ffi::c_int,
        cmd_func: Some(nv_scroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'M' as ::core::ffi::c_int,
        cmd_func: Some(nv_scroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'N' as ::core::ffi::c_int,
        cmd_func: Some(nv_next as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: SEARCH_REV as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: 'O' as ::core::ffi::c_int,
        cmd_func: Some(nv_open as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'P' as ::core::ffi::c_int,
        cmd_func: Some(nv_put as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'Q' as ::core::ffi::c_int,
        cmd_func: Some(nv_regreplay as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'R' as ::core::ffi::c_int,
        cmd_func: Some(nv_Replace as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'S' as ::core::ffi::c_int,
        cmd_func: Some(nv_subst as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'T' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_ALW | NV_LANG) as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: 'U' as ::core::ffi::c_int,
        cmd_func: Some(nv_Undo as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'W' as ::core::ffi::c_int,
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'X' as ::core::ffi::c_int,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'Y' as ::core::ffi::c_int,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'Z' as ::core::ffi::c_int,
        cmd_func: Some(nv_Zet as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_NOP | NV_NCW) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '[' as ::core::ffi::c_int,
        cmd_func: Some(nv_brackets as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: '\\' as ::core::ffi::c_int,
        cmd_func: Some(nv_error as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: ']' as ::core::ffi::c_int,
        cmd_func: Some(nv_brackets as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: '^' as ::core::ffi::c_int,
        cmd_func: Some(nv_beginline as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: (BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int) as int16_t,
    },
    nv_cmd {
        cmd_char: '_' as ::core::ffi::c_int,
        cmd_func: Some(nv_lineop as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '`' as ::core::ffi::c_int,
        cmd_func: Some(nv_gomark as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'a' as ::core::ffi::c_int,
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'b' as ::core::ffi::c_int,
        cmd_func: Some(nv_bck_word as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'c' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'd' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'e' as ::core::ffi::c_int,
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'f' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_ALW | NV_LANG) as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: 'g' as ::core::ffi::c_int,
        cmd_func: Some(nv_g_cmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'h' as ::core::ffi::c_int,
        cmd_func: Some(nv_left as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_RL as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'i' as ::core::ffi::c_int,
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'j' as ::core::ffi::c_int,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'k' as ::core::ffi::c_int,
        cmd_func: Some(nv_up as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'l' as ::core::ffi::c_int,
        cmd_func: Some(nv_right as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_RL as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'm' as ::core::ffi::c_int,
        cmd_func: Some(nv_mark as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_NOP as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'n' as ::core::ffi::c_int,
        cmd_func: Some(nv_next as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'o' as ::core::ffi::c_int,
        cmd_func: Some(nv_open as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'p' as ::core::ffi::c_int,
        cmd_func: Some(nv_put as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'q' as ::core::ffi::c_int,
        cmd_func: Some(nv_record as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'r' as ::core::ffi::c_int,
        cmd_func: Some(nv_replace as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_NOP | NV_LANG) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 's' as ::core::ffi::c_int,
        cmd_func: Some(nv_subst as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 't' as ::core::ffi::c_int,
        cmd_func: Some(nv_csearch as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_NCH_ALW | NV_LANG) as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: 'u' as ::core::ffi::c_int,
        cmd_func: Some(nv_undo as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'w' as ::core::ffi::c_int,
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'x' as ::core::ffi::c_int,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'y' as ::core::ffi::c_int,
        cmd_func: Some(nv_operator as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: 'z' as ::core::ffi::c_int,
        cmd_func: Some(nv_zet as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCH_ALW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '{' as ::core::ffi::c_int,
        cmd_func: Some(nv_findpar as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: '|' as ::core::ffi::c_int,
        cmd_func: Some(nv_pipe as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: '}' as ::core::ffi::c_int,
        cmd_func: Some(nv_findpar as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: '~' as ::core::ffi::c_int,
        cmd_func: Some(nv_tilde as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: POUND,
        cmd_func: Some(nv_ident as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mousescroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: MSCR_UP as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mousescroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: MSCR_DOWN as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mousescroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: MSCR_LEFT as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mousescroll as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: MSCR_RIGHT as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_mouse as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_ignore as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_nop as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_INS,
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_KINS as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_edit as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_BS,
        cmd_func: Some(nv_ctrlh as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_UP,
        cmd_func: Some(nv_up as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_SS as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_DOWN,
        cmd_func: Some(nv_down as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_S_DOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_SS as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_LEFT,
        cmd_func: Some(nv_left as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS | NV_RL) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_S_LEFT,
        cmd_func: Some(nv_bck_word as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SS | NV_RL) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_C_LEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_bck_word as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_RL | NV_STS) as uint16_t,
        cmd_arg: 1 as int16_t,
    },
    nv_cmd {
        cmd_char: K_RIGHT,
        cmd_func: Some(nv_right as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS | NV_RL) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_S_RIGHT,
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SS | NV_RL) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_C_RIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_wordcmd as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_RL | NV_STS) as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_PAGEUP,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_KPAGEUP,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: BACKWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_PAGEDOWN,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_KPAGEDOWN,
        cmd_func: Some(nv_page as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: FORWARD as ::core::ffi::c_int as int16_t,
    },
    nv_cmd {
        cmd_char: K_END,
        cmd_func: Some(nv_end as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_KEND,
        cmd_func: Some(nv_end as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_S_END,
        cmd_func: Some(nv_end as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_SS as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_end as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: true_0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_HOME,
        cmd_func: Some(nv_home as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_KHOME,
        cmd_func: Some(nv_home as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_S_HOME,
        cmd_func: Some(nv_home as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_SS as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_goto as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: (NV_SSS | NV_STS) as uint16_t,
        cmd_arg: false_0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_DEL,
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_abbrev as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_UNDO,
        cmd_func: Some(nv_kundo as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_HELP,
        cmd_func: Some(nv_help as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_F1,
        cmd_func: Some(nv_help as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_XF1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_help as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_NCW as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_SELECT,
        cmd_func: Some(nv_select as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: K_PASTE_START,
        cmd_func: Some(nv_paste as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_event as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: NV_KEEPREG as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_colon as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
    nv_cmd {
        cmd_char: -(253 as ::core::ffi::c_int
            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        cmd_func: Some(nv_colon as unsafe extern "C" fn(*mut cmdarg_T) -> ()),
        cmd_flags: 0 as uint16_t,
        cmd_arg: 0 as int16_t,
    },
]);
pub const NV_CMDS_SIZE: usize = ::core::mem::size_of::<[nv_cmd; 188]>()
    .wrapping_div(::core::mem::size_of::<nv_cmd>())
    .wrapping_div(
        (::core::mem::size_of::<[nv_cmd; 188]>().wrapping_rem(::core::mem::size_of::<nv_cmd>())
            == 0) as ::core::ffi::c_int as usize,
    );
static nv_cmd_idx: GlobalCell<[int16_t; 188]> = GlobalCell::new([0; 188]);
static nv_max_linear: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn nv_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut c1: ::core::ffi::c_int = (*nv_cmds.ptr())[*(s1 as *const int16_t) as usize].cmd_char;
    let mut c2: ::core::ffi::c_int = (*nv_cmds.ptr())[*(s2 as *const int16_t) as usize].cmd_char;
    if c1 < 0 as ::core::ffi::c_int {
        c1 = -c1;
    }
    if c2 < 0 as ::core::ffi::c_int {
        c2 = -c2;
    }
    return if c1 == c2 {
        0 as ::core::ffi::c_int
    } else if c1 > c2 {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn init_normal_cmds() {
    '_c2rust_label: {
        if ::core::mem::size_of::<[nv_cmd; 188]>()
            .wrapping_div(::core::mem::size_of::<nv_cmd>())
            .wrapping_div(
                (::core::mem::size_of::<[nv_cmd; 188]>()
                    .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            <= 32767 as usize
        {
        } else {
            __assert_fail(
                b"NV_CMDS_SIZE <= SHRT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                390 as ::core::ffi::c_uint,
                b"void init_normal_cmds(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut i: int16_t = 0 as int16_t;
    while (i as ::core::ffi::c_int) < NV_CMDS_SIZE as int16_t as ::core::ffi::c_int {
        (*nv_cmd_idx.ptr())[i as usize] = i;
        i += 1;
    }
    qsort(
        nv_cmd_idx.ptr() as *mut ::core::ffi::c_void,
        NV_CMDS_SIZE,
        ::core::mem::size_of::<int16_t>(),
        Some(
            nv_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    let mut i_0: int16_t = 0;
    i_0 = 0 as int16_t;
    while (i_0 as ::core::ffi::c_int) < NV_CMDS_SIZE as int16_t as ::core::ffi::c_int {
        if i_0 as ::core::ffi::c_int
            != (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i_0 as usize] as usize].cmd_char
        {
            break;
        }
        i_0 += 1;
    }
    nv_max_linear.set(i_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn find_command(mut cmdchar: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if cmdchar >= 0x100 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if cmdchar < 0 as ::core::ffi::c_int {
        cmdchar = -cmdchar;
    }
    '_c2rust_label: {
        if nv_max_linear.get()
            < ::core::mem::size_of::<[nv_cmd; 188]>()
                .wrapping_div(::core::mem::size_of::<nv_cmd>())
                .wrapping_div(
                    (::core::mem::size_of::<[nv_cmd; 188]>()
                        .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"nv_max_linear < (int)NV_CMDS_SIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                428 as ::core::ffi::c_uint,
                b"int find_command(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if cmdchar <= nv_max_linear.get() {
        return (*nv_cmd_idx.ptr())[cmdchar as usize] as ::core::ffi::c_int;
    }
    let mut bot: ::core::ffi::c_int = nv_max_linear.get() + 1 as ::core::ffi::c_int;
    let mut top: ::core::ffi::c_int = NV_CMDS_SIZE.wrapping_sub(1 as usize) as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    while bot <= top {
        let mut i: ::core::ffi::c_int = (top + bot) / 2 as ::core::ffi::c_int;
        let mut c: ::core::ffi::c_int =
            (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i as usize] as usize].cmd_char;
        if c < 0 as ::core::ffi::c_int {
            c = -c;
        }
        if cmdchar == c {
            idx = (*nv_cmd_idx.ptr())[i as usize] as ::core::ffi::c_int;
            break;
        } else if cmdchar > c {
            bot = i + 1 as ::core::ffi::c_int;
        } else {
            top = i - 1 as ::core::ffi::c_int;
        }
    }
    return idx;
}
unsafe extern "C" fn check_text_locked(mut oap: *mut oparg_T) -> bool {
    if !text_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearopbeep(oap);
    }
    text_locked_msg();
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn check_text_or_curbuf_locked(mut oap: *mut oparg_T) -> bool {
    if check_text_locked(oap) {
        return true_0 != 0;
    }
    if !curbuf_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearop(oap);
    }
    return true_0 != 0;
}
static current_oap: GlobalCell<*mut oparg_T> = GlobalCell::new(::core::ptr::null_mut::<oparg_T>());
#[no_mangle]
pub unsafe extern "C" fn op_pending() -> bool {
    return !(!(*current_oap.ptr()).is_null()
        && !finish_op.get()
        && (*current_oap.get()).prev_opcount == 0 as ::core::ffi::c_int
        && (*current_oap.get()).prev_count0 == 0 as ::core::ffi::c_int
        && (*current_oap.get()).op_type == OP_NOP as ::core::ffi::c_int
        && (*current_oap.get()).regname == NUL);
}
#[no_mangle]
pub unsafe extern "C" fn normal_enter(mut cmdwin: bool, mut noexmode: bool) {
    let mut state: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut state);
    let mut prev_oap: *mut oparg_T = current_oap.get();
    current_oap.set(&raw mut state.oa);
    state.cmdwin = cmdwin;
    state.noexmode = noexmode;
    state.toplevel = (!cmdwin || cmdwin_result.get() == 0 as ::core::ffi::c_int) && !noexmode;
    state_enter(&raw mut state.state);
    current_oap.set(prev_oap);
}
unsafe extern "C" fn normal_prepare(mut s: *mut NormalState) {
    memset(
        &raw mut (*s).ca as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    (*s).ca.oap = &raw mut (*s).oa;
    (*s).ca.opcount = opcount.get();
    let mut c: ::core::ffi::c_int = finish_op.get() as ::core::ffi::c_int;
    finish_op.set((*s).oa.op_type != OP_NOP as ::core::ffi::c_int);
    if finish_op.get() as ::core::ffi::c_int != c {
        ui_cursor_shape();
    }
    may_trigger_modechanged();
    (*s).set_prevcount = false_0 != 0;
    if !finish_op.get() && (*s).oa.regname == 0 {
        (*s).ca.opcount = 0 as ::core::ffi::c_int;
        (*s).set_prevcount = true_0 != 0;
    }
    if (*s).oa.prev_opcount > 0 as ::core::ffi::c_int
        || (*s).oa.prev_count0 > 0 as ::core::ffi::c_int
    {
        (*s).ca.opcount = (*s).oa.prev_opcount;
        (*s).ca.count0 = (*s).oa.prev_count0;
        (*s).oa.prev_opcount = 0 as ::core::ffi::c_int;
        (*s).oa.prev_count0 = 0 as ::core::ffi::c_int;
    }
    (*s).mapped_len = typebuf_maplen();
    State.set(MODE_NORMAL_BUSY as ::core::ffi::c_int);
    if (*s).toplevel as ::core::ffi::c_int != 0 && readbuf1_empty() as ::core::ffi::c_int != 0 {
        set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
    }
}
unsafe extern "C" fn normal_handle_special_visual_command(mut s: *mut NormalState) -> bool {
    if km_stopsel.get() as ::core::ffi::c_int != 0
        && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_STS != 0
        && mod_mask.get() & MOD_MASK_SHIFT == 0
    {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if km_startsel.get() {
        if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_SS != 0 {
            unshift_special(&raw mut (*s).ca);
            (*s).idx = find_command((*s).ca.cmdchar);
            if (*s).idx < 0 as ::core::ffi::c_int {
                clearopbeep(&raw mut (*s).oa);
                return true_0 != 0;
            }
        } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_SSS != 0
            && mod_mask.get() & MOD_MASK_SHIFT != 0
        {
            (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn normal_need_additional_char(mut s: *mut NormalState) -> bool {
    let mut flags: ::core::ffi::c_int =
        (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int;
    let mut pending_op: bool = (*s).oa.op_type != OP_NOP as ::core::ffi::c_int;
    let mut cmdchar: ::core::ffi::c_int = (*s).ca.cmdchar;
    return flags & NV_NCH != 0
        && (flags & NV_NCH_NOP == NV_NCH_NOP && !pending_op
            || flags & NV_NCH_ALW == NV_NCH_ALW
            || cmdchar == 'q' as ::core::ffi::c_int
                && !pending_op
                && reg_recording.get() == 0 as ::core::ffi::c_int
                && reg_executing.get() == 0 as ::core::ffi::c_int
            || (cmdchar == 'a' as ::core::ffi::c_int || cmdchar == 'i' as ::core::ffi::c_int)
                && (pending_op as ::core::ffi::c_int != 0
                    || VIsual_active.get() as ::core::ffi::c_int != 0));
}
unsafe extern "C" fn normal_need_redraw_mode_message(mut s: *mut NormalState) -> bool {
    return (p_smd.get() != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && (restart_edit.get() != 0 as ::core::ffi::c_int
            || VIsual_active.get() as ::core::ffi::c_int != 0
                && (*s).old_pos.lnum == (*curwin.get()).w_cursor.lnum
                && (*s).old_pos.col == (*curwin.get()).w_cursor.col)
        && (clear_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_cmdline.get() as ::core::ffi::c_int != 0)
        && (msg_didout.get() as ::core::ffi::c_int != 0
            || msg_didany.get() as ::core::ffi::c_int != 0 && msg_scroll.get() != 0)
        && !msg_nowait.get()
        && KeyTyped.get() as ::core::ffi::c_int != 0
        || restart_edit.get() != 0 as ::core::ffi::c_int
            && !VIsual_active.get()
            && msg_scroll.get() != 0
            && emsg_on_display.get() as ::core::ffi::c_int != 0)
        && (*s).oa.regname == 0 as ::core::ffi::c_int
        && (*s).ca.retval & CA_COMMAND_BUSY as ::core::ffi::c_int == 0
        && stuff_empty() as ::core::ffi::c_int != 0
        && typebuf_typed() != 0
        && emsg_silent.get() == 0 as ::core::ffi::c_int
        && !in_assert_fails.get()
        && !did_wait_return.get()
        && (*s).oa.op_type == OP_NOP as ::core::ffi::c_int;
}
unsafe extern "C" fn normal_redraw_mode_message(mut _s: *mut NormalState) {
    let mut save_State: ::core::ffi::c_int = State.get();
    if restart_edit.get() != 0 as ::core::ffi::c_int {
        State.set(MODE_INSERT as ::core::ffi::c_int);
    }
    if must_redraw.get() != 0 && !(*keep_msg.ptr()).is_null() && !emsg_on_display.get() {
        let mut kmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        kmsg = keep_msg.get();
        keep_msg.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        setcursor();
        update_screen();
        keep_msg.set(kmsg);
        kmsg = xstrdup(keep_msg.get());
        msg(kmsg, keep_msg_hl_id.get());
        xfree(kmsg as *mut ::core::ffi::c_void);
    }
    setcursor();
    ui_cursor_shape();
    ui_flush();
    if msg_scroll.get() != 0 || emsg_on_display.get() as ::core::ffi::c_int != 0 {
        msg_delay(1003 as uint64_t, true_0 != 0);
    }
    msg_delay(3003 as uint64_t, false_0 != 0);
    State.set(save_State);
    msg_scroll.set(false_0);
    emsg_on_display.set(false_0 != 0);
}
unsafe extern "C" fn normal_get_additional_char(mut s: *mut NormalState) {
    let mut cp: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut repl: bool = false_0 != 0;
    let mut lit: bool = false_0 != 0;
    let mut lang: bool = false;
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    did_cursorhold.set(true_0 != 0);
    if (*s).ca.cmdchar == 'g' as ::core::ffi::c_int {
        (*s).ca.nchar = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).ca.nchar >= 0 as ::core::ffi::c_int
        {
            if (*s).ca.nchar < 256 as ::core::ffi::c_int {
                (*s).ca.nchar =
                    (*langmap_mapchar.ptr())[(*s).ca.nchar as usize] as ::core::ffi::c_int;
            } else {
                (*s).ca.nchar = langmap_adjust_mb((*s).ca.nchar);
            }
        }
        (*s).need_flushbuf = (*s).need_flushbuf as ::core::ffi::c_int
            | add_to_showcmd((*s).ca.nchar) as ::core::ffi::c_int
            != 0;
        if (*s).ca.nchar == 'r' as ::core::ffi::c_int
            || (*s).ca.nchar == '\'' as ::core::ffi::c_int
            || (*s).ca.nchar == '`' as ::core::ffi::c_int
            || (*s).ca.nchar == Ctrl_BSL
        {
            cp = &raw mut (*s).ca.extra_char;
            if (*s).ca.nchar != 'r' as ::core::ffi::c_int {
                lit = true_0 != 0;
            } else {
                repl = true_0 != 0;
            }
        } else {
            cp = ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
    } else {
        if (*s).ca.cmdchar == 'r' as ::core::ffi::c_int {
            repl = true_0 != 0;
        }
        cp = &raw mut (*s).ca.nchar;
    }
    lang = repl as ::core::ffi::c_int != 0
        || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_LANG != 0;
    if !cp.is_null() {
        let mut langmap_active: bool = false_0 != 0;
        if repl {
            State.set(MODE_REPLACE as ::core::ffi::c_int);
            ui_cursor_shape_no_check_conceal();
        }
        if lang as ::core::ffi::c_int != 0 && (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt
        {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if repl {
                State.set(MODE_LREPLACE as ::core::ffi::c_int);
            } else {
                State.set(MODE_LANGMAP as ::core::ffi::c_int);
            }
            langmap_active = true_0 != 0;
        }
        *cp = plain_vgetc();
        if langmap_active {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        State.set(MODE_NORMAL_BUSY as ::core::ffi::c_int);
        (*s).need_flushbuf = (*s).need_flushbuf as ::core::ffi::c_int
            | add_to_showcmd(*cp) as ::core::ffi::c_int
            != 0;
        if !lit {
            if *cp == Ctrl_K
                && ((*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_LANG
                    != 0
                    || cp == &raw mut (*s).ca.extra_char)
                && vim_strchr(p_cpo.get(), CPO_DIGRAPH).is_null()
            {
                (*s).c = get_digraph(false_0 != 0);
                if (*s).c > 0 as ::core::ffi::c_int {
                    *cp = (*s).c;
                    del_from_showcmd(3 as ::core::ffi::c_int);
                    (*s).need_flushbuf = (*s).need_flushbuf as ::core::ffi::c_int
                        | add_to_showcmd(*cp) as ::core::ffi::c_int
                        != 0;
                }
            }
            if *p_langmap.get() as ::core::ffi::c_int != 0
                && !lang
                && (p_lrm.get() != 0
                    || (if vgetc_busy.get() != 0 {
                        (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    } else {
                        KeyTyped.get() as ::core::ffi::c_int
                    }) != 0)
                && KeyStuffed.get() == 0
                && *cp >= 0 as ::core::ffi::c_int
            {
                if *cp < 256 as ::core::ffi::c_int {
                    *cp = (*langmap_mapchar.ptr())[*cp as usize] as ::core::ffi::c_int;
                } else {
                    *cp = langmap_adjust_mb(*cp);
                }
            }
        }
        if cp == &raw mut (*s).ca.extra_char
            && (*s).ca.nchar == Ctrl_BSL
            && ((*s).ca.extra_char == Ctrl_N || (*s).ca.extra_char == Ctrl_G)
        {
            (*s).ca.cmdchar = Ctrl_BSL;
            (*s).ca.nchar = (*s).ca.extra_char;
            (*s).idx = find_command((*s).ca.cmdchar);
        } else if ((*s).ca.nchar == 'n' as ::core::ffi::c_int
            || (*s).ca.nchar == 'N' as ::core::ffi::c_int)
            && (*s).ca.cmdchar == 'g' as ::core::ffi::c_int
        {
            (*(*s).ca.oap).op_type = get_op_type(*cp, NUL);
        } else if *cp == Ctrl_BSL {
            let mut towait: ::core::ffi::c_int = if p_ttm.get() >= 0 as OptInt {
                p_ttm.get() as ::core::ffi::c_int
            } else {
                p_tm.get() as ::core::ffi::c_int
            };
            loop {
                (*s).c = vpeekc();
                if !((*s).c <= 0 as ::core::ffi::c_int && towait > 0 as ::core::ffi::c_int) {
                    break;
                }
                do_sleep(
                    (if towait > 50 as ::core::ffi::c_int {
                        50 as ::core::ffi::c_int
                    } else {
                        towait
                    }) as int64_t,
                    false_0 != 0,
                );
                towait -= 50 as ::core::ffi::c_int;
            }
            if (*s).c > 0 as ::core::ffi::c_int {
                (*s).c = plain_vgetc();
                if (*s).c != Ctrl_N && (*s).c != Ctrl_G {
                    vungetc((*s).c);
                } else {
                    (*s).ca.cmdchar = Ctrl_BSL;
                    (*s).ca.nchar = (*s).c;
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                827 as ::core::ffi::c_uint,
                                b"void normal_get_additional_char(NormalState *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            }
        }
        if lang {
            (*no_mapping.ptr()) -= 1;
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            let mut prev_code: ::core::ffi::c_int = (*s).ca.nchar;
            loop {
                (*s).c = vpeekc();
                if !((*s).c > 0 as ::core::ffi::c_int
                    && ((*s).c >= 0x100 as ::core::ffi::c_int
                        || utf8len_tab[vpeekc() as usize] as ::core::ffi::c_int
                            > 1 as ::core::ffi::c_int))
                {
                    break;
                }
                (*s).c = plain_vgetc();
                if !utf_iscomposing(prev_code, (*s).c, &raw mut state) {
                    vungetc((*s).c);
                    break;
                } else {
                    if (*s).ca.nchar_len == 0 as ::core::ffi::c_int {
                        (*s).ca.nchar_len = utf_char2bytes(
                            (*s).ca.nchar,
                            &raw mut (*s).ca.nchar_composing as *mut ::core::ffi::c_char,
                        );
                    }
                    if (*s).ca.nchar_len + utf_char2len((*s).c)
                        < ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as ::core::ffi::c_int
                    {
                        (*s).ca.nchar_len += utf_char2bytes(
                            (*s).c,
                            (&raw mut (*s).ca.nchar_composing as *mut ::core::ffi::c_char)
                                .offset((*s).ca.nchar_len as isize),
                        );
                    }
                    prev_code = (*s).c;
                }
            }
            (*s).ca.nchar_composing[(*s).ca.nchar_len as usize] = NUL as ::core::ffi::c_char;
            (*no_mapping.ptr()) += 1;
            (*no_u_sync.ptr()) += 1;
            gotchars_ignore();
            (*no_u_sync.ptr()) -= 1;
        }
    }
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
}
unsafe extern "C" fn normal_invert_horizontal(mut s: *mut NormalState) {
    match (*s).ca.cmdchar {
        108 => {
            (*s).ca.cmdchar = 'h' as ::core::ffi::c_int;
        }
        K_RIGHT => {
            (*s).ca.cmdchar = K_LEFT;
        }
        K_S_RIGHT => {
            (*s).ca.cmdchar = K_S_LEFT;
        }
        -22269 => {
            (*s).ca.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_C_LEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        104 => {
            (*s).ca.cmdchar = 'l' as ::core::ffi::c_int;
        }
        K_LEFT => {
            (*s).ca.cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*s).ca.cmdchar = K_S_RIGHT;
        }
        -22013 => {
            (*s).ca.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_C_RIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        62 => {
            (*s).ca.cmdchar = '<' as ::core::ffi::c_int;
        }
        60 => {
            (*s).ca.cmdchar = '>' as ::core::ffi::c_int;
        }
        _ => {}
    }
    (*s).idx = find_command((*s).ca.cmdchar);
}
unsafe extern "C" fn normal_get_command_count(mut s: *mut NormalState) -> bool {
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_select.get() as ::core::ffi::c_int != 0
    {
        return false_0 != 0;
    }
    while (*s).c >= '1' as ::core::ffi::c_int && (*s).c <= '9' as ::core::ffi::c_int
        || (*s).ca.count0 != 0 as ::core::ffi::c_int
            && ((*s).c == K_DEL
                || (*s).c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || (*s).c == '0' as ::core::ffi::c_int)
    {
        if (*s).c == K_DEL
            || (*s).c
                == -(253 as ::core::ffi::c_int
                    + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*s).ca.count0 /= 10 as ::core::ffi::c_int;
            del_from_showcmd(4 as ::core::ffi::c_int);
        } else if (*s).ca.count0 > 99999999 as ::core::ffi::c_int {
            (*s).ca.count0 = 999999999 as ::core::ffi::c_int;
        } else {
            (*s).ca.count0 =
                (*s).ca.count0 * 10 as ::core::ffi::c_int + ((*s).c - '0' as ::core::ffi::c_int);
        }
        if (*s).toplevel as ::core::ffi::c_int != 0 && readbuf1_empty() as ::core::ffi::c_int != 0 {
            set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
        }
        if (*s).ctrl_w {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        (*no_zero_mapping.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as ::core::ffi::c_int
        {
            if (*s).c < 256 as ::core::ffi::c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as ::core::ffi::c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_zero_mapping.ptr()) -= 1;
        if (*s).ctrl_w {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
        }
        (*s).need_flushbuf = (*s).need_flushbuf as ::core::ffi::c_int
            | add_to_showcmd((*s).c) as ::core::ffi::c_int
            != 0;
    }
    if (*s).c == Ctrl_W && !(*s).ctrl_w && (*s).oa.op_type == OP_NOP as ::core::ffi::c_int {
        (*s).ctrl_w = true_0 != 0;
        (*s).ca.opcount = (*s).ca.count0;
        (*s).ca.count0 = 0 as ::core::ffi::c_int;
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as ::core::ffi::c_int
        {
            if (*s).c < 256 as ::core::ffi::c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as ::core::ffi::c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        (*s).need_flushbuf = (*s).need_flushbuf as ::core::ffi::c_int
            | add_to_showcmd((*s).c) as ::core::ffi::c_int
            != 0;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn normal_finish_command(mut s: *mut NormalState) {
    let mut did_visual_op: bool = false_0 != 0;
    if !(*s).command_finished {
        if !finish_op.get()
            && (*s).oa.op_type == 0
            && ((*s).idx < 0 as ::core::ffi::c_int
                || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_KEEPREG
                    == 0)
        {
            clearop(&raw mut (*s).oa);
            set_reg_var(get_default_register_name());
        }
        if (*s).old_mapped_len > 0 as ::core::ffi::c_int {
            (*s).old_mapped_len = typebuf_maplen();
        }
        if (*s).ca.cmdchar
            != -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && (*s).ca.cmdchar
                != -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            did_visual_op = VIsual_active.get() as ::core::ffi::c_int != 0
                && (*s).oa.op_type != OP_NOP as ::core::ffi::c_int
                && (*s).oa.op_type != OP_COLON as ::core::ffi::c_int;
            do_pending_operator(&raw mut (*s).ca, (*s).old_col, false_0 != 0);
        }
        if normal_need_redraw_mode_message(s) {
            normal_redraw_mode_message(s);
        }
    }
    msg_nowait.set(false_0 != 0);
    if finish_op.get() as ::core::ffi::c_int != 0 || did_visual_op as ::core::ffi::c_int != 0 {
        set_reg_var(get_default_register_name());
    }
    let prev_finish_op: bool = finish_op.get();
    if (*s).oa.op_type == OP_NOP as ::core::ffi::c_int {
        finish_op.set(false_0 != 0);
        may_trigger_modechanged();
    }
    if prev_finish_op as ::core::ffi::c_int != 0
        || (*s).ca.cmdchar == 'r' as ::core::ffi::c_int
        || (*s).ca.cmdchar == 'g' as ::core::ffi::c_int
            && (*s).ca.nchar == 'r' as ::core::ffi::c_int
    {
        ui_cursor_shape();
    }
    if (*s).oa.op_type == OP_NOP as ::core::ffi::c_int
        && (*s).oa.regname == 0 as ::core::ffi::c_int
        && (*s).ca.cmdchar
            != -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        clear_showcmd();
    }
    checkpcmark();
    xfree((*s).ca.searchbuf as *mut ::core::ffi::c_void);
    mb_check_adjust_col(curwin.get() as *mut ::core::ffi::c_void);
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 && (*s).toplevel as ::core::ffi::c_int != 0 {
        validate_cursor(curwin.get());
        do_check_scrollbind(true_0 != 0);
    }
    if (*curwin.get()).w_onebuf_opt.wo_crb != 0 && (*s).toplevel as ::core::ffi::c_int != 0 {
        validate_cursor(curwin.get());
        do_check_cursorbind();
    }
    if (*s).oa.op_type == OP_NOP as ::core::ffi::c_int
        && (restart_edit.get() != 0 as ::core::ffi::c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as ::core::ffi::c_int
            || restart_VIsual_select.get() == 1 as ::core::ffi::c_int)
        && (*s).ca.retval & CA_COMMAND_BUSY as ::core::ffi::c_int == 0
        && stuff_empty() as ::core::ffi::c_int != 0
        && (*s).oa.regname == 0 as ::core::ffi::c_int
    {
        if restart_VIsual_select.get() == 1 as ::core::ffi::c_int {
            VIsual_select.set(true_0 != 0);
            VIsual_select_reg.set(0 as ::core::ffi::c_int);
            may_trigger_modechanged();
            showmode();
            restart_VIsual_select.set(0 as ::core::ffi::c_int);
        }
        if restart_edit.get() != 0 as ::core::ffi::c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as ::core::ffi::c_int
        {
            edit(restart_edit.get(), false_0 != 0, 1 as ::core::ffi::c_int);
        }
    }
    if restart_VIsual_select.get() == 2 as ::core::ffi::c_int {
        restart_VIsual_select.set(1 as ::core::ffi::c_int);
    }
    opcount.set((*s).ca.opcount);
}
unsafe extern "C" fn normal_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    (*s).command_finished = false_0 != 0;
    (*s).ctrl_w = false_0 != 0;
    (*s).old_col = (*curwin.get()).w_curswant as ::core::ffi::c_int;
    (*s).c = key;
    if *p_langmap.get() as ::core::ffi::c_int != 0
        && get_real_state() != MODE_SELECT as ::core::ffi::c_int
        && (p_lrm.get() != 0
            || (if vgetc_busy.get() != 0 {
                (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } else {
                KeyTyped.get() as ::core::ffi::c_int
            }) != 0)
        && KeyStuffed.get() == 0
        && (*s).c >= 0 as ::core::ffi::c_int
    {
        if (*s).c < 256 as ::core::ffi::c_int {
            (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as ::core::ffi::c_int;
        } else {
            (*s).c = langmap_adjust_mb((*s).c);
        }
    }
    if restart_edit.get() == 0 as ::core::ffi::c_int {
        (*s).old_mapped_len = 0 as ::core::ffi::c_int;
    } else if (*s).old_mapped_len != 0
        || VIsual_active.get() as ::core::ffi::c_int != 0
            && (*s).mapped_len == 0 as ::core::ffi::c_int
            && typebuf_maplen() > 0 as ::core::ffi::c_int
    {
        (*s).old_mapped_len = typebuf_maplen();
    }
    if (*s).c == NUL {
        (*s).c = K_ZERO;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_select.get() as ::core::ffi::c_int != 0
        && (vim_isprintc((*s).c) as ::core::ffi::c_int != 0
            || (*s).c == NL
            || (*s).c == CAR
            || (*s).c == K_KENTER)
    {
        let mut len: ::core::ffi::c_int =
            ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
        if KeyTyped.get() {
            ungetchars(len);
        }
        if restart_edit.get() != 0 as ::core::ffi::c_int {
            (*s).c = 'd' as ::core::ffi::c_int;
        } else {
            (*s).c = 'c' as ::core::ffi::c_int;
        }
        msg_nowait.set(true_0 != 0);
        (*s).old_mapped_len = 0 as ::core::ffi::c_int;
    }
    (*s).need_flushbuf = add_to_showcmd((*s).c);
    while normal_get_command_count(s) {}
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).oa.prev_opcount = (*s).ca.opcount;
        (*s).oa.prev_count0 = (*s).ca.count0;
    } else if (*s).ca.opcount != 0 as ::core::ffi::c_int {
        if (*s).ca.count0 != 0 {
            if (*s).ca.opcount >= 999999999 as ::core::ffi::c_int / (*s).ca.count0 {
                (*s).ca.count0 = 999999999 as ::core::ffi::c_int;
            } else {
                (*s).ca.count0 *= (*s).ca.opcount;
            }
        } else {
            (*s).ca.count0 = (*s).ca.opcount;
        }
    }
    (*s).ca.opcount = (*s).ca.count0;
    (*s).ca.count1 = if (*s).ca.count0 == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        (*s).ca.count0
    };
    if (*s).toplevel as ::core::ffi::c_int != 0 && readbuf1_empty() as ::core::ffi::c_int != 0 {
        set_vcount(
            (*s).ca.count0 as int64_t,
            (*s).ca.count1 as int64_t,
            (*s).set_prevcount,
        );
    }
    if (*s).ctrl_w {
        (*s).ca.nchar = (*s).c;
        (*s).ca.cmdchar = Ctrl_W;
    } else {
        (*s).ca.cmdchar = (*s).c;
    }
    (*s).idx = find_command((*s).ca.cmdchar);
    if (*s).idx < 0 as ::core::ffi::c_int {
        clearopbeep(&raw mut (*s).oa);
        (*s).command_finished = true_0 != 0;
    } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_NCW != 0
        && check_text_or_curbuf_locked(&raw mut (*s).oa) as ::core::ffi::c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else if VIsual_active.get() as ::core::ffi::c_int != 0
        && normal_handle_special_visual_command(s) as ::core::ffi::c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else {
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
            && KeyStuffed.get() == 0
            && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_RL != 0
        {
            normal_invert_horizontal(s);
        }
        if normal_need_additional_char(s) {
            normal_get_additional_char(s);
        }
        if (*s).need_flushbuf {
            ui_flush();
        }
        if (*s).ca.cmdchar
            != -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && (*s).ca.cmdchar
                != -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            did_cursorhold.set(false_0 != 0);
        }
        State.set(MODE_NORMAL as ::core::ffi::c_int);
        if (*s).ca.nchar == ESC || (*s).ca.extra_char == ESC {
            clearop(&raw mut (*s).oa);
            (*s).command_finished = true_0 != 0;
        } else {
            if (*s).ca.cmdchar
                != -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                msg_didout.set(false_0 != 0);
                msg_col.set(0 as ::core::ffi::c_int);
            }
            (*s).old_pos = (*curwin.get()).w_cursor;
            if !VIsual_active.get() && km_startsel.get() as ::core::ffi::c_int != 0 {
                if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int & NV_SS != 0
                {
                    start_selection();
                    unshift_special(&raw mut (*s).ca);
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1239 as ::core::ffi::c_uint,
                                b"int normal_execute(VimState *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as ::core::ffi::c_int
                    & NV_SSS
                    != 0
                    && mod_mask.get() & MOD_MASK_SHIFT != 0
                {
                    start_selection();
                    (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
                }
            }
            (*s).ca.arg = (*nv_cmds.ptr())[(*s).idx as usize].cmd_arg as ::core::ffi::c_int;
            (*nv_cmds.ptr())[(*s).idx as usize]
                .cmd_func
                .expect("non-null function pointer")(&raw mut (*s).ca);
        }
    }
    normal_finish_command(s);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn normal_check_stuff_buffer(mut _s: *mut NormalState) {
    if stuff_empty() {
        did_check_timestamps.set(false_0 != 0);
        if need_check_timestamps.get() {
            check_timestamps(false_0);
        }
        if need_wait_return.get() {
            wait_return(false_0);
        }
    }
}
unsafe extern "C" fn normal_check_interrupt(mut s: *mut NormalState) {
    if got_int.get() {
        if (*s).noexmode as ::core::ffi::c_int != 0
            && global_busy.get() != 0
            && !exmode_active.get()
            && (*s).previous_got_int as ::core::ffi::c_int != 0
        {
            exmode_active.set(true_0 != 0);
            State.set(MODE_NORMAL as ::core::ffi::c_int);
        } else if global_busy.get() == 0 || !exmode_active.get() {
            if !quit_more.get() {
                vgetc();
            }
            got_int.set(false_0 != 0);
        }
        (*s).previous_got_int = true_0 != 0;
    } else {
        (*s).previous_got_int = false_0 != 0;
    };
}
unsafe extern "C" fn normal_check_window_scrolled(mut _s: *mut NormalState) {
    if !finish_op.get() {
        may_trigger_win_scrolled_resized();
    }
}
unsafe extern "C" fn normal_check_cursor_moved(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_CURSORMOVED) as ::core::ffi::c_int != 0
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
    {
        apply_autocmds(
            EVENT_CURSORMOVED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set((*curwin.get()).w_cursor);
    }
}
unsafe extern "C" fn normal_check_text_changed(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_TEXTCHANGED) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_last_changedtick != buf_get_changedtick(curbuf.get())
    {
        apply_autocmds(
            EVENT_TEXTCHANGED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    }
}
unsafe extern "C" fn normal_check_buffer_modified(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_BUFMODIFIEDSET) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_changed_invalid as ::core::ffi::c_int == true_0
    {
        apply_autocmds(
            EVENT_BUFMODIFIEDSET,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_changed_invalid = false_0 != 0;
    }
}
unsafe extern "C" fn normal_check_safe_state(mut _s: *mut NormalState) {
    may_trigger_safestate(!op_pending() && restart_edit.get() == 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn normal_check_folds(mut _s: *mut NormalState) {
    foldAdjustVisual();
    if hasAnyFolding(curwin.get()) != 0 && !char_avail() {
        foldCheckClose();
        if fdo_flags.get() & kOptFdoFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            foldOpenCursor();
        }
    }
}
unsafe extern "C" fn normal_redraw(mut _s: *mut NormalState) {
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    show_cursor_info_later(false_0 != 0);
    if must_redraw.get() != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if redraw_cmdline.get() as ::core::ffi::c_int != 0
            || clear_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_mode.get() as ::core::ffi::c_int != 0
        {
            showmode();
        }
    }
    (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
    if !(*keep_msg.ptr()).is_null() {
        let p: *mut ::core::ffi::c_char = xstrdup(keep_msg.get());
        msg_hist_off.set(true_0 != 0);
        msg(p, keep_msg_hl_id.get());
        msg_hist_off.set(false_0 != 0);
        xfree(p as *mut ::core::ffi::c_void);
    }
    if need_fileinfo.get() as ::core::ffi::c_int != 0
        && !shortmess(SHM_FILEINFO as ::core::ffi::c_int)
    {
        fileinfo(false_0, true_0, false_0 != 0);
        need_fileinfo.set(false_0 != 0);
    }
    emsg_on_display.set(false_0 != 0);
    did_emsg.set(false_0);
    msg_didany.set(false_0 != 0);
    may_clear_sb_text();
    setcursor();
}
unsafe extern "C" fn normal_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    normal_check_stuff_buffer(s);
    normal_check_interrupt(s);
    if did_throw.get() as ::core::ffi::c_int != 0 && ex_normal_busy.get() == 0 {
        discard_current_exception();
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    quit_more.set(false_0 != 0);
    state_no_longer_safe(::core::ptr::null::<::core::ffi::c_char>());
    if skip_redraw.get() as ::core::ffi::c_int != 0
        || exmode_active.get() as ::core::ffi::c_int != 0
    {
        skip_redraw.set(false_0 != 0);
        setcursor();
    } else if do_redraw.get() as ::core::ffi::c_int != 0 || stuff_empty() as ::core::ffi::c_int != 0
    {
        terminal_check_refresh();
        update_topline(curwin.get());
        validate_cursor(curwin.get());
        normal_check_cursor_moved(s);
        normal_check_text_changed(s);
        normal_check_window_scrolled(s);
        normal_check_buffer_modified(s);
        normal_check_safe_state(s);
        if (*curtab.get()).tp_diff_update != 0 || (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
            (*curtab.get()).tp_diff_update = false_0;
        }
        if diff_need_scrollbind.get() {
            check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
            diff_need_scrollbind.set(false_0 != 0);
        }
        normal_check_folds(s);
        normal_redraw(s);
        do_redraw.set(false_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"first screen update\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
            time_finish();
        }
        may_make_initial_scroll_size_snapshot();
    }
    may_garbage_collect.set(!(*s).cmdwin && !(*s).noexmode);
    update_curswant();
    if exmode_active.get() {
        if (*s).noexmode {
            return 0 as ::core::ffi::c_int;
        }
        do_exmode();
        return -1 as ::core::ffi::c_int;
    }
    if (*s).cmdwin as ::core::ffi::c_int != 0 && cmdwin_result.get() != 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    normal_prepare(s);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn set_vcount_ca(mut cap: *mut cmdarg_T, mut set_prevcount: *mut bool) {
    let mut count: int64_t = (*cap).count0 as int64_t;
    if (*cap).opcount != 0 as ::core::ffi::c_int {
        count = (*cap).opcount as int64_t
            * (if count == 0 as int64_t {
                1 as int64_t
            } else {
                count
            });
    }
    set_vcount(
        count,
        if count == 0 as int64_t {
            1 as int64_t
        } else {
            count
        },
        *set_prevcount,
    );
    *set_prevcount = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn end_visual_mode() {
    VIsual_select_exclu_adj.set(false_0 != 0);
    VIsual_active.set(false_0 != 0);
    setmouse();
    mouse_dragging.set(0 as ::core::ffi::c_int);
    (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
    (*curbuf.get()).b_visual.vi_start = VIsual.get();
    (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
    (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
    (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    may_clear_cmdline();
    adjust_cursor_eol();
    may_trigger_modechanged();
}
#[no_mangle]
pub unsafe extern "C" fn reset_VIsual_and_resel() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    VIsual_reselect.set(false_0);
}
#[no_mangle]
pub unsafe extern "C" fn reset_VIsual() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        VIsual_reselect.set(false_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn restore_visual_mode() {
    if VIsual_mode_orig.get() != NUL {
        (*curbuf.get()).b_visual.vi_mode = VIsual_mode_orig.get();
        VIsual_mode_orig.set(NUL);
    }
}
unsafe extern "C" fn find_is_eval_item(
    ptr: *const ::core::ffi::c_char,
    colp: *mut ::core::ffi::c_int,
    bnp: *mut ::core::ffi::c_int,
    dir: ::core::ffi::c_int,
) -> bool {
    if *ptr as ::core::ffi::c_int == ']' as ::core::ffi::c_int
        && dir == BACKWARD as ::core::ffi::c_int
        || *ptr as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            && dir == FORWARD as ::core::ffi::c_int
    {
        *bnp += 1 as ::core::ffi::c_int;
    }
    if *bnp > 0 as ::core::ffi::c_int {
        if *ptr as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            && dir == BACKWARD as ::core::ffi::c_int
            || *ptr as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                && dir == FORWARD as ::core::ffi::c_int
        {
            *bnp -= 1 as ::core::ffi::c_int;
        }
        return true_0 != 0;
    }
    if *ptr as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if *ptr.offset(
        (if dir == BACKWARD as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as isize,
    ) as ::core::ffi::c_int
        == '>' as ::core::ffi::c_int
        && *ptr.offset(
            (if dir == BACKWARD as ::core::ffi::c_int {
                -1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as isize,
        ) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
    {
        *colp += dir;
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn find_ident_under_cursor(
    mut text: *mut *mut ::core::ffi::c_char,
    mut find_type: ::core::ffi::c_int,
    mut offset: *mut ::core::ffi::c_int,
) -> size_t {
    let mut textcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut len: size_t = find_ident_at_pos(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum,
        (*curwin.get()).w_cursor.col,
        text,
        if !offset.is_null() {
            &raw mut textcol
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_int>()
        },
        find_type,
    );
    if !offset.is_null() {
        *offset = (*curwin.get()).w_cursor.col as ::core::ffi::c_int - textcol;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn find_ident_at_pos(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut startcol: colnr_T,
    mut text: *mut *mut ::core::ffi::c_char,
    mut textcol: *mut ::core::ffi::c_int,
    mut find_type: ::core::ffi::c_int,
) -> size_t {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    let mut this_class: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_class: ::core::ffi::c_int = 0;
    let mut prevcol: ::core::ffi::c_int = 0;
    let mut bn: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ptr: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    i = if find_type & FIND_IDENT as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    while i < 2 as ::core::ffi::c_int {
        col = startcol as ::core::ffi::c_int;
        while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL {
            if find_type & FIND_EVAL as ::core::ffi::c_int != 0
                && *ptr.offset(col as isize) as ::core::ffi::c_int == ']' as ::core::ffi::c_int
            {
                break;
            }
            this_class = mb_get_class(ptr.offset(col as isize));
            if this_class != 0 as ::core::ffi::c_int
                && (i == 1 as ::core::ffi::c_int || this_class != 1 as ::core::ffi::c_int)
            {
                break;
            }
            col += utfc_ptr2len(ptr.offset(col as isize));
        }
        bn = (*ptr.offset(col as isize) as ::core::ffi::c_int == ']' as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if find_type & FIND_EVAL as ::core::ffi::c_int != 0
            && *ptr.offset(col as isize) as ::core::ffi::c_int == ']' as ::core::ffi::c_int
        {
            this_class = mb_get_class(b"a\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            this_class = mb_get_class(ptr.offset(col as isize));
        }
        while col > 0 as ::core::ffi::c_int && this_class != 0 as ::core::ffi::c_int {
            prevcol = col
                - 1 as ::core::ffi::c_int
                - utf_head_off(
                    ptr,
                    ptr.offset(col as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize)),
                );
            prev_class = mb_get_class(ptr.offset(prevcol as isize));
            if this_class != prev_class
                && (i == 0 as ::core::ffi::c_int
                    || prev_class == 0 as ::core::ffi::c_int
                    || find_type & FIND_IDENT as ::core::ffi::c_int != 0)
                && (find_type & FIND_EVAL as ::core::ffi::c_int == 0
                    || prevcol == 0 as ::core::ffi::c_int
                    || !find_is_eval_item(
                        ptr.offset(prevcol as isize),
                        &raw mut prevcol,
                        &raw mut bn,
                        BACKWARD as ::core::ffi::c_int,
                    ))
            {
                break;
            }
            col = prevcol;
        }
        this_class = if this_class < 2 as ::core::ffi::c_int {
            this_class
        } else {
            2 as ::core::ffi::c_int
        };
        if find_type & FIND_STRING as ::core::ffi::c_int == 0
            || this_class == 2 as ::core::ffi::c_int
        {
            break;
        }
        i += 1;
    }
    if *ptr.offset(col as isize) as ::core::ffi::c_int == NUL
        || i == 0 as ::core::ffi::c_int && this_class != 2 as ::core::ffi::c_int
    {
        if find_type & FIND_STRING as ::core::ffi::c_int != 0 {
            emsg(gettext(
                b"E348: No string under cursor\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(&raw const e_noident as *const ::core::ffi::c_char));
        }
        return 0 as size_t;
    }
    ptr = ptr.offset(col as isize);
    *text = ptr;
    if !textcol.is_null() {
        *textcol = col;
    }
    bn = 0 as ::core::ffi::c_int;
    startcol -= col;
    col = 0 as ::core::ffi::c_int;
    this_class = mb_get_class(ptr);
    while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL
        && ((if i == 0 as ::core::ffi::c_int {
            (mb_get_class(ptr.offset(col as isize)) == this_class) as ::core::ffi::c_int
        } else {
            (mb_get_class(ptr.offset(col as isize)) != 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int
        }) != 0
            || find_type & FIND_EVAL as ::core::ffi::c_int != 0
                && col <= startcol
                && find_is_eval_item(
                    ptr.offset(col as isize),
                    &raw mut col,
                    &raw mut bn,
                    FORWARD as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
    {
        col += utfc_ptr2len(ptr.offset(col as isize));
    }
    '_c2rust_label: {
        if col >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"col >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1748 as ::core::ffi::c_uint,
                b"size_t find_ident_at_pos(win_T *, linenr_T, colnr_T, char **, int *, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return col as size_t;
}
unsafe extern "C" fn prep_redo_cmd(mut cap: *mut cmdarg_T) {
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        NUL,
    );
    if (*cap).nchar_len > 0 as ::core::ffi::c_int {
        AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut ::core::ffi::c_char);
    } else {
        AppendCharToRedobuff((*cap).nchar);
    };
}
#[no_mangle]
pub unsafe extern "C" fn prep_redo(
    mut regname: ::core::ffi::c_int,
    mut num: ::core::ffi::c_int,
    mut cmd1: ::core::ffi::c_int,
    mut cmd2: ::core::ffi::c_int,
    mut cmd3: ::core::ffi::c_int,
    mut cmd4: ::core::ffi::c_int,
    mut cmd5: ::core::ffi::c_int,
) {
    prep_redo_num2(
        regname,
        num,
        cmd1,
        cmd2,
        0 as ::core::ffi::c_int,
        cmd3,
        cmd4,
        cmd5,
    );
}
#[no_mangle]
pub unsafe extern "C" fn prep_redo_num2(
    mut regname: ::core::ffi::c_int,
    mut num1: ::core::ffi::c_int,
    mut cmd1: ::core::ffi::c_int,
    mut cmd2: ::core::ffi::c_int,
    mut num2: ::core::ffi::c_int,
    mut cmd3: ::core::ffi::c_int,
    mut cmd4: ::core::ffi::c_int,
    mut cmd5: ::core::ffi::c_int,
) {
    ResetRedobuff();
    if regname != 0 as ::core::ffi::c_int {
        AppendCharToRedobuff('"' as ::core::ffi::c_int);
        AppendCharToRedobuff(regname);
    }
    if num1 != 0 as ::core::ffi::c_int {
        AppendNumberToRedobuff(num1);
    }
    if cmd1 != NUL {
        AppendCharToRedobuff(cmd1);
    }
    if cmd2 != NUL {
        AppendCharToRedobuff(cmd2);
    }
    if num2 != 0 as ::core::ffi::c_int {
        AppendNumberToRedobuff(num2);
    }
    if cmd3 != NUL {
        AppendCharToRedobuff(cmd3);
    }
    if cmd4 != NUL {
        AppendCharToRedobuff(cmd4);
    }
    if cmd5 != NUL {
        AppendCharToRedobuff(cmd5);
    }
}
unsafe extern "C" fn checkclearop(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as ::core::ffi::c_int {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}
unsafe extern "C" fn checkclearopq(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as ::core::ffi::c_int && !VIsual_active.get() {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn clearop(mut oap: *mut oparg_T) {
    (*oap).op_type = OP_NOP as ::core::ffi::c_int;
    (*oap).regname = 0 as ::core::ffi::c_int;
    (*oap).motion_force = NUL;
    (*oap).use_reg_one = false_0 != 0;
    motion_force.set(NUL);
}
#[no_mangle]
pub unsafe extern "C" fn clearopbeep(mut oap: *mut oparg_T) {
    clearop(oap);
    beep_flush();
}
unsafe extern "C" fn unshift_special(mut cap: *mut cmdarg_T) {
    match (*cap).cmdchar {
        K_S_RIGHT => {
            (*cap).cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*cap).cmdchar = K_LEFT;
        }
        -1277 => {
            (*cap).cmdchar = K_UP;
        }
        -1533 => {
            (*cap).cmdchar = K_DOWN;
        }
        K_S_HOME => {
            (*cap).cmdchar = K_HOME;
        }
        K_S_END => {
            (*cap).cmdchar = K_END;
        }
        _ => {}
    }
    (*cap).cmdchar = simplify_key((*cap).cmdchar, mod_mask.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn may_clear_cmdline() {
    if mode_displayed.get() {
        clear_cmdline.set(true_0 != 0);
    } else {
        clear_showcmd();
    };
}
static old_showcmd_buf: GlobalCell<[::core::ffi::c_char; 41]> = GlobalCell::new([0; 41]);
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
#[no_mangle]
pub unsafe extern "C" fn clear_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && !char_avail() {
        let mut cursor_bot: bool = lt(VIsual.get(), (*curwin.get()).w_cursor);
        let mut lines: ::core::ffi::c_int = 0;
        let mut leftcol: colnr_T = 0;
        let mut rightcol: colnr_T = 0;
        let mut top: linenr_T = 0;
        let mut bot: linenr_T = 0;
        if cursor_bot {
            top = (*VIsual.ptr()).lnum;
            bot = (*curwin.get()).w_cursor.lnum;
        } else {
            top = (*curwin.get()).w_cursor.lnum;
            bot = (*VIsual.ptr()).lnum;
        }
        hasFolding(
            curwin.get(),
            top,
            &raw mut top,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        hasFolding(
            curwin.get(),
            bot,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut bot,
        );
        lines = (bot - top + 1 as linenr_T) as ::core::ffi::c_int;
        if VIsual_mode.get() == Ctrl_V {
            let saved_sbr: *mut ::core::ffi::c_char = p_sbr.get();
            let saved_w_sbr: *mut ::core::ffi::c_char = (*curwin.get()).w_onebuf_opt.wo_sbr;
            p_sbr.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
            (*curwin.get()).w_onebuf_opt.wo_sbr =
                empty_string_option.ptr() as *mut ::core::ffi::c_char;
            getvcols(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                VIsual.ptr(),
                &raw mut leftcol,
                &raw mut rightcol,
            );
            p_sbr.set(saved_sbr);
            (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
            snprintf(
                showcmd_buf.ptr() as *mut ::core::ffi::c_char,
                SHOWCMD_BUFLEN as ::core::ffi::c_int as size_t,
                b"%ldx%ld\0".as_ptr() as *const ::core::ffi::c_char,
                lines as int64_t,
                rightcol as int64_t - leftcol as int64_t + 1 as int64_t,
            );
        } else if VIsual_mode.get() == 'V' as ::core::ffi::c_int
            || (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum
        {
            snprintf(
                showcmd_buf.ptr() as *mut ::core::ffi::c_char,
                SHOWCMD_BUFLEN as ::core::ffi::c_int as size_t,
                b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                lines as int64_t,
            );
        } else {
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut bytes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut chars: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if cursor_bot {
                s = ml_get_pos(VIsual.ptr());
                e = get_cursor_pos_ptr();
            } else {
                s = get_cursor_pos_ptr();
                e = ml_get_pos(VIsual.ptr());
            }
            while if *p_sel.get() as ::core::ffi::c_int != 'e' as ::core::ffi::c_int {
                (s <= e) as ::core::ffi::c_int
            } else {
                (s < e) as ::core::ffi::c_int
            } != 0
            {
                let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
                if l == 0 as ::core::ffi::c_int {
                    bytes += 1;
                    chars += 1;
                    break;
                } else {
                    bytes += l;
                    chars += 1;
                    s = s.offset(l as isize);
                }
            }
            if bytes == chars {
                snprintf(
                    showcmd_buf.ptr() as *mut ::core::ffi::c_char,
                    SHOWCMD_BUFLEN as ::core::ffi::c_int as size_t,
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    chars,
                );
            } else {
                snprintf(
                    showcmd_buf.ptr() as *mut ::core::ffi::c_char,
                    SHOWCMD_BUFLEN as ::core::ffi::c_int as size_t,
                    b"%d-%d\0".as_ptr() as *const ::core::ffi::c_char,
                    chars,
                    bytes,
                );
            }
        }
        let mut limit: ::core::ffi::c_int = if ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            SHOWCMD_BUFLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            SHOWCMD_COLS as ::core::ffi::c_int
        };
        (*showcmd_buf.ptr())[limit as usize] = NUL as ::core::ffi::c_char;
        showcmd_visual.set(true_0 != 0);
    } else {
        (*showcmd_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        showcmd_visual.set(false_0 != 0);
        if showcmd_is_clear.get() {
            return;
        }
    }
    display_showcmd();
}
#[no_mangle]
pub unsafe extern "C" fn add_to_showcmd(mut c: ::core::ffi::c_int) -> bool {
    static ignore: GlobalCell<[::core::ffi::c_int; 23]> = GlobalCell::new([
        -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        0 as ::core::ffi::c_int,
    ]);
    if p_sc.get() == 0 || msg_silent.get() != 0 as ::core::ffi::c_int || ex_normal_busy.get() != 0 {
        return false_0 != 0;
    }
    if showcmd_visual.get() {
        (*showcmd_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        showcmd_visual.set(false_0 != 0);
    }
    if c < 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (*ignore.ptr())[i as usize] != 0 as ::core::ffi::c_int {
            if (*ignore.ptr())[i as usize] == c {
                return false_0 != 0;
            }
            i += 1;
        }
    }
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mbyte_buf: [::core::ffi::c_char; 7] = [0; 7];
    if c <= 0x7f as ::core::ffi::c_int || !vim_isprintc(c) {
        p = transchar(c);
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            strcpy(
                p,
                b"<20>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
    } else {
        mbyte_buf[utf_char2bytes(c, &raw mut mbyte_buf as *mut ::core::ffi::c_char) as usize] =
            NUL as ::core::ffi::c_char;
        p = &raw mut mbyte_buf as *mut ::core::ffi::c_char;
    }
    let mut old_len: size_t = strlen(showcmd_buf.ptr() as *mut ::core::ffi::c_char);
    let mut extra_len: size_t = strlen(p);
    let mut limit: size_t = (if ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
        SHOWCMD_BUFLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        SHOWCMD_COLS as ::core::ffi::c_int
    }) as size_t;
    if old_len.wrapping_add(extra_len) > limit {
        let mut overflow: size_t = old_len.wrapping_add(extra_len).wrapping_sub(limit);
        memmove(
            showcmd_buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            (showcmd_buf.ptr() as *mut ::core::ffi::c_char).offset(overflow as isize)
                as *const ::core::ffi::c_void,
            old_len.wrapping_sub(overflow).wrapping_add(1 as size_t),
        );
    }
    strcat(showcmd_buf.ptr() as *mut ::core::ffi::c_char, p);
    if char_avail() {
        return false_0 != 0;
    }
    display_showcmd();
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn add_to_showcmd_c(mut c: ::core::ffi::c_int) {
    add_to_showcmd(c);
    setcursor();
}
unsafe extern "C" fn del_from_showcmd(mut len: ::core::ffi::c_int) {
    if p_sc.get() == 0 {
        return;
    }
    let mut old_len: ::core::ffi::c_int =
        strlen(showcmd_buf.ptr() as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    len = if len < old_len { len } else { old_len };
    (*showcmd_buf.ptr())[(old_len - len) as usize] = NUL as ::core::ffi::c_char;
    if !char_avail() {
        display_showcmd();
    }
}
#[no_mangle]
pub unsafe extern "C" fn push_showcmd() {
    if p_sc.get() != 0 {
        strcpy(
            old_showcmd_buf.ptr() as *mut ::core::ffi::c_char,
            showcmd_buf.ptr() as *mut ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn pop_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    strcpy(
        showcmd_buf.ptr() as *mut ::core::ffi::c_char,
        old_showcmd_buf.ptr() as *mut ::core::ffi::c_char,
    );
    display_showcmd();
}
unsafe extern "C" fn display_showcmd() {
    showcmd_is_clear
        .set((*showcmd_buf.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL);
    if *p_sloc.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
        if showcmd_is_clear.get() {
            (*curwin.get()).w_redr_status = true_0 != 0;
        } else {
            win_redr_status(curwin.get());
            setcursor();
        }
        return;
    }
    if *p_sloc.get() as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
        if showcmd_is_clear.get() {
            redraw_tabline.set(true_0 != 0);
        } else {
            draw_tabline();
            setcursor();
        }
        return;
    }
    if ui_has(kUIMessages) {
        let mut content: Array = ARRAY_DICT_INIT;
        let mut content__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 1];
        content.capacity = 1 as size_t;
        content.items = &raw mut content__items as *mut Object;
        let mut chunk: Array = ARRAY_DICT_INIT;
        let mut chunk__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 3];
        chunk.capacity = 3 as size_t;
        chunk.items = &raw mut chunk__items as *mut Object;
        if !showcmd_is_clear.get() {
            let c2rust_fresh6 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh7 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_0 {
                    string: cstr_as_string(showcmd_buf.ptr() as *mut ::core::ffi::c_char),
                },
            };
            let c2rust_fresh8 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh9 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_0 { array: chunk },
            };
        }
        ui_call_msg_showcmd(content);
        return;
    }
    if p_ch.get() == 0 as OptInt {
        return;
    }
    msg_grid_validate();
    let mut showcmd_row: ::core::ffi::c_int = Rows.get() - 1 as ::core::ffi::c_int;
    grid_line_start(msg_grid_adj.ptr(), showcmd_row);
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !showcmd_is_clear.get() {
        len = grid_line_puts(
            sc_col.get(),
            showcmd_buf.ptr() as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            *(*hl_attr_active.ptr()).offset(HLF_MSG as ::core::ffi::c_int as isize),
        );
    }
    grid_line_puts(
        sc_col.get() + len,
        (b"          \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char)
            .offset(len as isize),
        -1 as ::core::ffi::c_int,
        *(*hl_attr_active.ptr()).offset(HLF_MSG as ::core::ffi::c_int as isize),
    );
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn get_vtopline(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return plines_m_win_fill(wp, 1 as linenr_T, (*wp).w_topline) - (*wp).w_topfill;
}
#[no_mangle]
pub unsafe extern "C" fn do_check_scrollbind(mut check: bool) {
    static old_curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static old_vtopline: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
    static old_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
    static old_leftcol: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
    let mut vtopline: ::core::ffi::c_int = get_vtopline(curwin.get());
    if check as ::core::ffi::c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        if did_syncbind.get() {
            did_syncbind.set(false_0 != 0);
        } else if curwin.get() == old_curwin.get() {
            if ((*curwin.get()).w_buffer == old_buf.get()
                || (*curwin.get()).w_onebuf_opt.wo_diff != 0)
                && (vtopline as linenr_T != old_vtopline.get()
                    || (*curwin.get()).w_leftcol != old_leftcol.get())
            {
                check_scrollbind(
                    vtopline as linenr_T - old_vtopline.get(),
                    (*curwin.get()).w_leftcol as ::core::ffi::c_int
                        - old_leftcol.get() as ::core::ffi::c_int,
                );
            }
        } else if !vim_strchr(p_sbo.get(), 'j' as ::core::ffi::c_int).is_null() {
            check_scrollbind(
                vtopline as linenr_T - (*curwin.get()).w_scbind_pos as linenr_T,
                0 as ::core::ffi::c_int,
            );
        }
        (*curwin.get()).w_scbind_pos = vtopline;
    }
    old_curwin.set(curwin.get());
    old_vtopline.set(vtopline as linenr_T);
    old_buf.set((*curwin.get()).w_buffer);
    old_leftcol.set((*curwin.get()).w_leftcol);
}
#[no_mangle]
pub unsafe extern "C" fn check_scrollbind(
    mut vtopline_diff: linenr_T,
    mut leftcol_diff: ::core::ffi::c_int,
) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_VIsual_select: ::core::ffi::c_int = VIsual_select.get() as ::core::ffi::c_int;
    let mut old_VIsual_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
    let mut tgt_leftcol: colnr_T = (*curwin.get()).w_leftcol;
    let mut want_ver: bool = (*old_curwin).w_onebuf_opt.wo_diff != 0
        || !vim_strchr(p_sbo.get(), 'v' as ::core::ffi::c_int).is_null()
            && vtopline_diff != 0 as linenr_T;
    let mut want_hor: bool = !vim_strchr(p_sbo.get(), 'h' as ::core::ffi::c_int).is_null()
        && (leftcol_diff != 0 || vtopline_diff != 0 as linenr_T);
    VIsual_active.set(false);
    VIsual_select.set(VIsual_active.get());
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        if !(curwin.get() == old_curwin || (*curwin.get()).w_onebuf_opt.wo_scb == 0) {
            if want_ver {
                if (*old_curwin).w_onebuf_opt.wo_diff != 0
                    && (*curwin.get()).w_onebuf_opt.wo_diff != 0
                {
                    diff_set_topline(old_curwin, curwin.get());
                } else {
                    (*curwin.get()).w_scbind_pos += vtopline_diff as ::core::ffi::c_int;
                    let mut curr_vtopline: ::core::ffi::c_int = get_vtopline(curwin.get());
                    let mut max_vtopline: ::core::ffi::c_int = curr_vtopline
                        + (*curwin.get()).w_topfill
                        + plines_m_win_fill(
                            curwin.get(),
                            (*curwin.get()).w_topline + 1 as linenr_T,
                            (*curbuf.get()).b_ml.ml_line_count,
                        );
                    let mut new_vtopline: ::core::ffi::c_int = if (if ((*curwin.get()).w_scbind_pos
                        as linenr_T)
                        < max_vtopline as linenr_T
                    {
                        (*curwin.get()).w_scbind_pos as linenr_T
                    } else {
                        max_vtopline as linenr_T
                    }) > 1 as linenr_T
                    {
                        if ((*curwin.get()).w_scbind_pos as linenr_T) < max_vtopline as linenr_T {
                            (*curwin.get()).w_scbind_pos
                        } else {
                            max_vtopline
                        }
                    } else {
                        1 as ::core::ffi::c_int
                    };
                    let mut y: ::core::ffi::c_int = new_vtopline - curr_vtopline;
                    if y > 0 as ::core::ffi::c_int {
                        scrollup(curwin.get(), y as linenr_T, false_0 != 0);
                    } else {
                        scrolldown(curwin.get(), -(y as linenr_T), false_0);
                    }
                }
                redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
                cursor_correct(curwin.get());
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
            if want_hor {
                set_leftcol(tgt_leftcol);
            }
        }
        wp = (*wp).w_next;
    }
    VIsual_select.set(old_VIsual_select != 0);
    VIsual_active.set(old_VIsual_active != 0);
    curwin.set(old_curwin);
    curbuf.set(old_curbuf);
}
unsafe extern "C" fn nv_ignore(mut cap: *mut cmdarg_T) {
    (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
}
unsafe extern "C" fn nv_nop(mut _cap: *mut cmdarg_T) {}
unsafe extern "C" fn nv_error(mut cap: *mut cmdarg_T) {
    clearopbeep((*cap).oap);
}
unsafe extern "C" fn nv_help(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        ex_help(::core::ptr::null_mut::<exarg_T>());
    }
}
unsafe extern "C" fn nv_addsub(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
    } else if !VIsual_active.get() && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int {
        prep_redo_cmd(cap);
        (*(*cap).oap).op_type = if (*cap).cmdchar == Ctrl_A {
            OP_NR_ADD as ::core::ffi::c_int
        } else {
            OP_NR_SUB as ::core::ffi::c_int
        };
        op_addsub((*cap).oap, (*cap).count1 as linenr_T, (*cap).arg != 0);
        (*(*cap).oap).op_type = OP_NOP as ::core::ffi::c_int;
    } else if VIsual_active.get() {
        nv_operator(cap);
    } else {
        clearop((*cap).oap);
    };
}
unsafe extern "C" fn nv_page(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        if (*cap).arg == BACKWARD as ::core::ffi::c_int {
            goto_tabpage(-(*cap).count1);
        } else {
            goto_tabpage((*cap).count0);
        }
    } else {
        pagescroll((*cap).arg as Direction, (*cap).count1, false_0 != 0);
    };
}
unsafe extern "C" fn nv_gd(
    mut oap: *mut oparg_T,
    mut nchar: ::core::ffi::c_int,
    mut thisblock: ::core::ffi::c_int,
) {
    let mut len: size_t = 0;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    len = find_ident_under_cursor(
        &raw mut ptr,
        FIND_IDENT as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    if len == 0 as size_t
        || !find_decl(
            ptr,
            len,
            nchar == 'd' as ::core::ffi::c_int,
            thisblock != 0,
            SEARCH_START as ::core::ffi::c_int,
        )
    {
        clearopbeep(oap);
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
    if messaging() as ::core::ffi::c_int != 0
        && msg_silent.get() == 0
        && !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int)
    {
        clear_cmdline.set(true_0 != 0);
    }
}
unsafe extern "C" fn is_ident(
    mut line: *const ::core::ffi::c_char,
    mut offset: ::core::ffi::c_int,
) -> bool {
    let mut incomment: bool = false_0 != 0;
    let mut instring: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < offset && *line.offset(i as isize) as ::core::ffi::c_int != NUL {
        if instring != 0 as ::core::ffi::c_int {
            if prev != '\\' as ::core::ffi::c_int
                && *line.offset(i as isize) as uint8_t as ::core::ffi::c_int == instring
            {
                instring = 0 as ::core::ffi::c_int;
            }
        } else if (*line.offset(i as isize) as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            || *line.offset(i as isize) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
            && !incomment
        {
            instring = *line.offset(i as isize) as uint8_t as ::core::ffi::c_int;
        } else if incomment {
            if prev == '*' as ::core::ffi::c_int
                && *line.offset(i as isize) as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            {
                incomment = false_0 != 0;
            }
        } else if prev == '/' as ::core::ffi::c_int
            && *line.offset(i as isize) as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        {
            incomment = true_0 != 0;
        } else if prev == '/' as ::core::ffi::c_int
            && *line.offset(i as isize) as ::core::ffi::c_int == '/' as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        prev = *line.offset(i as isize) as uint8_t as ::core::ffi::c_int;
        i += 1;
    }
    return incomment as ::core::ffi::c_int == false_0 && instring == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn find_decl(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut locally: bool,
    mut thisblock: bool,
    mut flags_arg: ::core::ffi::c_int,
) -> bool {
    let mut par_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut t: bool = false;
    let mut retval: bool = true_0 != 0;
    let mut incll: bool = false;
    let mut searchflags: ::core::ffi::c_int = flags_arg;
    let mut patsize: size_t = len.wrapping_add(7 as size_t);
    let mut pat: *mut ::core::ffi::c_char = xmalloc(patsize) as *mut ::core::ffi::c_char;
    '_c2rust_label: {
        if patsize <= 2147483647 as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"patsize <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2387 as ::core::ffi::c_uint,
                b"_Bool find_decl(char *, size_t, _Bool, _Bool, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut patlen: size_t = snprintf(
        pat,
        patsize,
        if vim_iswordp(ptr) as ::core::ffi::c_int != 0 {
            b"\\V\\<%.*s\\>\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\\V%.*s\0".as_ptr() as *const ::core::ffi::c_char
        },
        len as ::core::ffi::c_int,
        ptr,
    ) as size_t;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut save_p_scs: bool = p_scs.get() != 0;
    p_ws.set(false_0);
    p_scs.set(false_0);
    if !locally
        || !findpar(
            &raw mut incll,
            BACKWARD as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            '{' as ::core::ffi::c_int,
            false_0 != 0,
        )
    {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        par_pos = (*curwin.get()).w_cursor;
    } else {
        par_pos = (*curwin.get()).w_cursor;
        while (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            && *skipwhite(get_cursor_line_ptr()) as ::core::ffi::c_int != NUL
        {
            (*curwin.get()).w_cursor.lnum -= 1;
        }
    }
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    clearpos(&raw mut found_pos);
    loop {
        t = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut (*curwin.get()).w_cursor,
            ::core::ptr::null_mut::<pos_T>(),
            FORWARD,
            pat,
            patlen,
            1 as ::core::ffi::c_int,
            searchflags,
            RE_LAST as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) != 0;
        if (*curwin.get()).w_cursor.lnum >= old_pos.lnum {
            t = false_0 != 0;
        }
        if thisblock as ::core::ffi::c_int != 0 && t as ::core::ffi::c_int != false_0 {
            let maxtravel: int64_t =
                (old_pos.lnum - (*curwin.get()).w_cursor.lnum + 1 as linenr_T) as int64_t;
            let mut pos: *const pos_T = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                '}' as ::core::ffi::c_int,
                FM_FORWARD as ::core::ffi::c_int,
                maxtravel,
            );
            if !pos.is_null() && (*pos).lnum < old_pos.lnum {
                (*curwin.get()).w_cursor = *pos;
                continue;
            }
        }
        if t as ::core::ffi::c_int == false_0 {
            if found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                t = true_0 != 0;
            }
            break;
        } else if get_leader_len(
            get_cursor_line_ptr(),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) > 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            let mut valid: bool = is_ident(
                get_cursor_line_ptr(),
                (*curwin.get()).w_cursor.col as ::core::ffi::c_int,
            );
            if !valid && found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                break;
            } else {
                if valid as ::core::ffi::c_int != 0 && !locally {
                    break;
                }
                if valid as ::core::ffi::c_int != 0 && (*curwin.get()).w_cursor.lnum >= par_pos.lnum
                {
                    if found_pos.lnum != 0 as linenr_T {
                        (*curwin.get()).w_cursor = found_pos;
                    }
                    break;
                } else {
                    if !valid {
                        clearpos(&raw mut found_pos);
                    } else {
                        found_pos = (*curwin.get()).w_cursor;
                    }
                    searchflags &= !(SEARCH_START as ::core::ffi::c_int);
                }
            }
        }
    }
    if t as ::core::ffi::c_int == false_0 {
        retval = false_0 != 0;
        (*curwin.get()).w_cursor = old_pos;
    } else {
        (*curwin.get()).w_set_curswant = true_0;
        reset_search_dir();
    }
    xfree(pat as *mut ::core::ffi::c_void);
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    p_scs.set(save_p_scs as ::core::ffi::c_int);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn nv_screengo(
    mut oap: *mut oparg_T,
    mut dir: ::core::ffi::c_int,
    mut dist: ::core::ffi::c_int,
    mut skip_conceal: bool,
) -> bool {
    let mut linelen: ::core::ffi::c_int = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
    let mut retval: bool = true_0 != 0;
    let mut atend: bool = false_0 != 0;
    let mut col_off1: ::core::ffi::c_int = 0;
    let mut col_off2: ::core::ffi::c_int = 0;
    let mut width1: ::core::ffi::c_int = 0;
    let mut width2: ::core::ffi::c_int = 0;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int;
    col_off1 = win_col_off(curwin.get());
    col_off2 = col_off1 - win_col_off2(curwin.get());
    width1 = (*curwin.get()).w_view_width - col_off1;
    width2 = (*curwin.get()).w_view_width - col_off2;
    if width2 == 0 as ::core::ffi::c_int {
        width2 = 1 as ::core::ffi::c_int;
    }
    if (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = 0;
        if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
            atend = true_0 != 0;
            validate_virtcol(curwin.get());
            if width1 <= 0 as ::core::ffi::c_int {
                (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                (*curwin.get()).w_curswant = (width1 - 1 as ::core::ffi::c_int) as colnr_T;
                if (*curwin.get()).w_virtcol > (*curwin.get()).w_curswant {
                    (*curwin.get()).w_curswant += (((*curwin.get()).w_virtcol
                        as ::core::ffi::c_int
                        - (*curwin.get()).w_curswant as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int)
                        / width2
                        + 1 as ::core::ffi::c_int)
                        * width2;
                }
            }
        } else {
            if linelen > width1 {
                n = ((linelen - width1 - 1 as ::core::ffi::c_int) / width2
                    + 1 as ::core::ffi::c_int)
                    * width2
                    + width1;
            } else {
                n = width1;
            }
            (*curwin.get()).w_curswant =
                (if (*curwin.get()).w_curswant < n - 1 as ::core::ffi::c_int {
                    (*curwin.get()).w_curswant as ::core::ffi::c_int
                } else {
                    n - 1 as ::core::ffi::c_int
                }) as colnr_T;
        }
        loop {
            let c2rust_fresh10 = dist;
            dist = dist - 1;
            if c2rust_fresh10 == 0 {
                break;
            }
            if dir == BACKWARD as ::core::ffi::c_int {
                if (*curwin.get()).w_curswant >= width1
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant -= width2;
                } else if (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_up_inner(curwin.get(), 1 as linenr_T, skip_conceal);
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                    if linelen > width1 {
                        let mut w: ::core::ffi::c_int =
                            ((linelen - width1 - 1 as ::core::ffi::c_int) / width2
                                + 1 as ::core::ffi::c_int)
                                * width2;
                        '_c2rust_label: {
                            if w <= 0 as ::core::ffi::c_int
                                || (*curwin.get()).w_curswant
                                    <= 2147483647 as ::core::ffi::c_int - w
                            {
                            } else {
                                __assert_fail(
                                    b"w <= 0 || curwin->w_curswant <= INT_MAX - w\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    2570 as ::core::ffi::c_uint,
                                    b"_Bool nv_screengo(oparg_T *, int, int, _Bool)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        (*curwin.get()).w_curswant += w;
                    }
                }
            } else {
                if linelen > width1 {
                    n = ((linelen - width1 - 1 as ::core::ffi::c_int) / width2
                        + 1 as ::core::ffi::c_int)
                        * width2
                        + width1;
                } else {
                    n = width1;
                }
                if (*curwin.get()).w_curswant as ::core::ffi::c_int + width2 < n
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant += width2;
                } else if (*curwin.get()).w_cursor.lnum
                    >= (*(*curwin.get()).w_buffer).b_ml.ml_line_count
                {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_down_inner(curwin.get(), 1 as ::core::ffi::c_int, skip_conceal);
                    (*curwin.get()).w_curswant %= width2;
                    if (*curwin.get()).w_curswant >= width1 {
                        (*curwin.get()).w_curswant -= width2;
                    }
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                }
            }
        }
    }
    if virtual_active(curwin.get()) as ::core::ffi::c_int != 0 && atend as ::core::ffi::c_int != 0 {
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    } else {
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
    }
    if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
    {
        validate_virtcol(curwin.get());
        let mut virtcol: colnr_T = (*curwin.get()).w_virtcol;
        if virtcol > width1 && *get_showbreak_value(curwin.get()) as ::core::ffi::c_int != NUL {
            virtcol -= vim_strsize(get_showbreak_value(curwin.get()));
        }
        let mut c: ::core::ffi::c_int = utf_ptr2char(get_cursor_pos_ptr());
        if dir == FORWARD as ::core::ffi::c_int
            && virtcol < (*curwin.get()).w_curswant
            && (*curwin.get()).w_curswant <= width1
            && !vim_isprintc(c)
            && c > 255 as ::core::ffi::c_int
        {
            oneright();
        }
        if virtcol > (*curwin.get()).w_curswant
            && (if (*curwin.get()).w_curswant < width1 {
                ((*curwin.get()).w_curswant > width1 / 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
            } else {
                (((*curwin.get()).w_curswant as ::core::ffi::c_int - width1) % width2
                    > width2 / 2 as ::core::ffi::c_int) as ::core::ffi::c_int
            }) != 0
        {
            (*curwin.get()).w_cursor.col -= 1;
        }
    }
    if atend {
        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
    }
    adjust_skipcol();
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn nv_scroll_line(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        scroll_redraw((*cap).arg, (*cap).count1 as linenr_T);
    }
}
unsafe extern "C" fn nv_z_get_count(
    mut cap: *mut cmdarg_T,
    mut nchar_arg: *mut ::core::ffi::c_int,
) -> bool {
    let mut nchar: ::core::ffi::c_int = *nchar_arg;
    if checkclearop((*cap).oap) {
        return false_0 != 0;
    }
    let mut n: ::core::ffi::c_int = nchar - '0' as ::core::ffi::c_int;
    loop {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as ::core::ffi::c_int
        {
            if nchar < 256 as ::core::ffi::c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as ::core::ffi::c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if nchar == K_DEL
            || nchar
                == -(253 as ::core::ffi::c_int
                    + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            n /= 10 as ::core::ffi::c_int;
        } else if ascii_isdigit(nchar) {
            if crate::src::nvim::math::vim_append_digit_int(
                &mut n,
                nchar - '0' as ::core::ffi::c_int,
            ) {
                continue;
            }
            clearopbeep((*cap).oap);
            break;
        } else if nchar == CAR {
            win_setheight(n);
            break;
        } else if nchar == 'l' as ::core::ffi::c_int
            || nchar == 'h' as ::core::ffi::c_int
            || nchar == K_LEFT
            || nchar == K_RIGHT
        {
            (*cap).count1 = if n != 0 {
                n * (*cap).count1
            } else {
                (*cap).count1
            };
            *nchar_arg = nchar;
            return true_0 != 0;
        } else {
            clearopbeep((*cap).oap);
            break;
        }
    }
    (*(*cap).oap).op_type = OP_NOP as ::core::ffi::c_int;
    return false_0 != 0;
}
unsafe extern "C" fn nv_zg_zw(
    mut cap: *mut cmdarg_T,
    mut nchar: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut undo: bool = false_0 != 0;
    if nchar == 'u' as ::core::ffi::c_int {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as ::core::ffi::c_int
        {
            if nchar < 256 as ::core::ffi::c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as ::core::ffi::c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if vim_strchr(b"gGwW\0".as_ptr() as *const ::core::ffi::c_char, nchar).is_null() {
            clearopbeep((*cap).oap);
            return OK;
        }
        undo = true_0 != 0;
    }
    if checkclearop((*cap).oap) {
        return OK;
    }
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && !get_visual_text(cap, &raw mut ptr, &raw mut len)
    {
        return FAIL;
    }
    if ptr.is_null() {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*emsg_off.ptr()) += 1;
        len = spell_move_to(
            curwin.get(),
            FORWARD as ::core::ffi::c_int,
            SMT_ALL,
            true_0 != 0,
            ::core::ptr::null_mut::<hlf_T>(),
        );
        (*emsg_off.ptr()) -= 1;
        if len != 0 as size_t && (*curwin.get()).w_cursor.col <= pos.col {
            ptr = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
        }
        (*curwin.get()).w_cursor = pos;
    }
    if ptr.is_null() && {
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        len == 0 as size_t
    } {
        return FAIL;
    }
    '_c2rust_label: {
        if len <= 2147483647 as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2754 as ::core::ffi::c_uint,
                b"int nv_zg_zw(cmdarg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    spell_add_word(
        ptr,
        len as ::core::ffi::c_int,
        (if nchar == 'w' as ::core::ffi::c_int || nchar == 'W' as ::core::ffi::c_int {
            SPELL_ADD_BAD as ::core::ffi::c_int
        } else {
            SPELL_ADD_GOOD as ::core::ffi::c_int
        }) as SpellAddType,
        if nchar == 'G' as ::core::ffi::c_int || nchar == 'W' as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            (*cap).count1
        },
        undo,
    );
    return OK;
}
unsafe extern "C" fn nv_zet(mut cap: *mut cmdarg_T) {
    let mut col: colnr_T = 0;
    let mut nchar: ::core::ffi::c_int = (*cap).nchar;
    let mut old_fdl: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_fdl as ::core::ffi::c_int;
    let mut old_fen: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    let mut siso: int64_t = get_sidescrolloff_value(curwin.get());
    if ascii_isdigit(nchar) as ::core::ffi::c_int != 0 && !nv_z_get_count(cap, &raw mut nchar) {
        return;
    }
    if (*cap).nchar != 'f' as ::core::ffi::c_int
        && (*cap).nchar != 'F' as ::core::ffi::c_int
        && !(VIsual_active.get() as ::core::ffi::c_int != 0
            && !vim_strchr(
                b"dcCoO\0".as_ptr() as *const ::core::ffi::c_char,
                (*cap).nchar,
            )
            .is_null())
        && (*cap).nchar != 'j' as ::core::ffi::c_int
        && (*cap).nchar != 'k' as ::core::ffi::c_int
        && checkclearop((*cap).oap) as ::core::ffi::c_int != 0
    {
        return;
    }
    if !vim_strchr(
        b"+\r\nt.z^-b\0".as_ptr() as *const ::core::ffi::c_char,
        nchar,
    )
    .is_null()
        && (*cap).count0 != 0
        && (*cap).count0 as linenr_T != (*curwin.get()).w_cursor.lnum
    {
        setpcmark();
        if (*cap).count0 as linenr_T > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        } else {
            (*curwin.get()).w_cursor.lnum = (*cap).count0 as linenr_T;
        }
        check_cursor_col(curwin.get());
    }
    's_906: {
        'c_53178: {
            'c_53195: {
                'c_55145: {
                    'c_55198: {
                        'c_53192: {
                            'c_55413: {
                                match nchar {
                                    43 => {
                                        if (*cap).count0 == 0 as ::core::ffi::c_int {
                                            validate_botline_win(curwin.get());
                                            (*curwin.get()).w_cursor.lnum = if (*curwin.get())
                                                .w_botline
                                                < (*curbuf.get()).b_ml.ml_line_count
                                            {
                                                (*curwin.get()).w_botline
                                            } else {
                                                (*curbuf.get()).b_ml.ml_line_count
                                            };
                                        }
                                        break 'c_55413;
                                    }
                                    NL | CAR | K_KENTER => {
                                        break 'c_55413;
                                    }
                                    116 => {
                                        break 'c_53178;
                                    }
                                    46 => {
                                        beginline(
                                            BL_WHITE as ::core::ffi::c_int
                                                | BL_FIX as ::core::ffi::c_int,
                                        );
                                    }
                                    122 => {}
                                    94 => {
                                        if (*cap).count0 != 0 as ::core::ffi::c_int {
                                            scroll_cursor_bot(
                                                curwin.get(),
                                                0 as ::core::ffi::c_int,
                                                true_0 != 0,
                                            );
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline;
                                        } else if (*curwin.get()).w_topline == 1 as linenr_T {
                                            (*curwin.get()).w_cursor.lnum =
                                                1 as ::core::ffi::c_int as linenr_T;
                                        } else {
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline - 1 as linenr_T;
                                        }
                                        break 'c_53192;
                                    }
                                    45 => {
                                        break 'c_53192;
                                    }
                                    98 => {
                                        break 'c_53195;
                                    }
                                    72 => {
                                        (*cap).count1 *=
                                            (*curwin.get()).w_view_width / 2 as ::core::ffi::c_int;
                                        break 'c_55198;
                                    }
                                    104 | K_LEFT => {
                                        break 'c_55198;
                                    }
                                    76 => {
                                        (*cap).count1 *=
                                            (*curwin.get()).w_view_width / 2 as ::core::ffi::c_int;
                                        break 'c_55145;
                                    }
                                    108 | K_RIGHT => {
                                        break 'c_55145;
                                    }
                                    115 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as ::core::ffi::c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    &raw mut col,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                );
                                            }
                                            if col as int64_t > siso {
                                                col -= siso as ::core::ffi::c_int;
                                            } else {
                                                col = 0 as ::core::ffi::c_int as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(
                                                    curwin.get(),
                                                    UPD_NOT_VALID as ::core::ffi::c_int,
                                                );
                                            }
                                        }
                                        break 's_906;
                                    }
                                    101 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as ::core::ffi::c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    &raw mut col,
                                                );
                                            }
                                            let mut n: ::core::ffi::c_int = (*curwin.get())
                                                .w_view_width
                                                - win_col_off(curwin.get());
                                            if col as int64_t + siso < n as int64_t {
                                                col = 0 as ::core::ffi::c_int as colnr_T;
                                            } else if (siso - n as int64_t)
                                                < (INT_MAX - col) as int64_t
                                            {
                                                col = (col as int64_t + siso - n as int64_t
                                                    + 1 as int64_t)
                                                    as ::core::ffi::c_int
                                                    as colnr_T;
                                            } else {
                                                col = INT_MAX as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(
                                                    curwin.get(),
                                                    UPD_NOT_VALID as ::core::ffi::c_int,
                                                );
                                            }
                                        }
                                        break 's_906;
                                    }
                                    80 | 112 => {
                                        nv_put(cap);
                                        break 's_906;
                                    }
                                    121 => {
                                        nv_operator(cap);
                                        break 's_906;
                                    }
                                    70 | 102 => {
                                        if foldManualAllowed(true_0 != 0) != 0 {
                                            (*cap).nchar = 'f' as ::core::ffi::c_int;
                                            nv_operator(cap);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                            if nchar == 'F' as ::core::ffi::c_int
                                                && (*(*cap).oap).op_type
                                                    == OP_FOLD as ::core::ffi::c_int
                                            {
                                                nv_operator(cap);
                                                finish_op.set(true_0 != 0);
                                            }
                                        } else {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    100 | 68 => {
                                        if foldManualAllowed(false_0 != 0) != 0 {
                                            if VIsual_active.get() {
                                                nv_operator(cap);
                                            } else {
                                                deleteFold(
                                                    curwin.get(),
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (nchar == 'D' as ::core::ffi::c_int)
                                                        as ::core::ffi::c_int,
                                                    false_0 != 0,
                                                );
                                            }
                                        }
                                        break 's_906;
                                    }
                                    69 => {
                                        if foldmethodIsManual(curwin.get()) {
                                            clearFolding(curwin.get());
                                            changed_window_setting(curwin.get());
                                        } else if foldmethodIsMarker(curwin.get()) {
                                            deleteFold(
                                                curwin.get(),
                                                1 as linenr_T,
                                                (*curbuf.get()).b_ml.ml_line_count,
                                                true_0,
                                                false_0 != 0,
                                            );
                                        } else {
                                            emsg(
                                                gettext(
                                                    b"E352: Cannot erase folds with current 'foldmethod'\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                            );
                                        }
                                        break 's_906;
                                    }
                                    110 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
                                        break 's_906;
                                    }
                                    78 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    105 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen =
                                            ((*curwin.get()).w_onebuf_opt.wo_fen == 0)
                                                as ::core::ffi::c_int;
                                        break 's_906;
                                    }
                                    97 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    65 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    111 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        break 's_906;
                                    }
                                    79 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        break 's_906;
                                    }
                                    99 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    67 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    118 => {
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    120 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        newFoldLevel();
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    88 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        old_fdl = -1 as ::core::ffi::c_int;
                                        break 's_906;
                                    }
                                    109 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_fdl > 0 as OptInt {
                                            (*curwin.get()).w_onebuf_opt.wo_fdl -=
                                                (*cap).count1 as OptInt;
                                            (*curwin.get()).w_onebuf_opt.wo_fdl = if (*curwin.get())
                                                .w_onebuf_opt
                                                .wo_fdl
                                                > 0 as OptInt
                                            {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                0 as OptInt
                                            };
                                        }
                                        old_fdl = -1 as ::core::ffi::c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    77 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl = 0 as OptInt;
                                        old_fdl = -1 as ::core::ffi::c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    114 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl +=
                                            (*cap).count1 as OptInt;
                                        let mut d: ::core::ffi::c_int =
                                            getDeepestNesting(curwin.get());
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            if (*curwin.get()).w_onebuf_opt.wo_fdl < d as OptInt {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                d as OptInt
                                            };
                                        break 's_906;
                                    }
                                    82 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            getDeepestNesting(curwin.get()) as OptInt;
                                        old_fdl = -1 as ::core::ffi::c_int;
                                        break 's_906;
                                    }
                                    106 | 107 => {
                                        if foldMoveTo(
                                            true_0 != 0,
                                            if nchar == 'j' as ::core::ffi::c_int {
                                                FORWARD as ::core::ffi::c_int
                                            } else {
                                                BACKWARD as ::core::ffi::c_int
                                            },
                                            (*cap).count1,
                                        ) == false_0
                                        {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    117 | 103 | 119 | 71 | 87 => {
                                        if nv_zg_zw(cap, nchar) == FAIL {
                                            return;
                                        }
                                        break 's_906;
                                    }
                                    61 => {
                                        if !checkclearop((*cap).oap) {
                                            spell_suggest((*cap).count0);
                                        }
                                        break 's_906;
                                    }
                                    _ => {
                                        clearopbeep((*cap).oap);
                                        break 's_906;
                                    }
                                }
                                scroll_cursor_halfway(curwin.get(), true_0 != 0, false_0 != 0);
                                redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
                                set_fraction(curwin.get());
                                break 's_906;
                            }
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                            break 'c_53178;
                        }
                        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                        break 'c_53195;
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                        set_leftcol(if (*cap).count1 > (*curwin.get()).w_leftcol {
                            0 as colnr_T
                        } else {
                            (*curwin.get()).w_leftcol - (*cap).count1
                        });
                    }
                    break 's_906;
                }
                if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                    set_leftcol((*curwin.get()).w_leftcol + (*cap).count1);
                }
                break 's_906;
            }
            scroll_cursor_bot(curwin.get(), 0 as ::core::ffi::c_int, true_0 != 0);
            redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
            set_fraction(curwin.get());
            break 's_906;
        }
        scroll_cursor_top(curwin.get(), 0 as ::core::ffi::c_int, true_0);
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
        set_fraction(curwin.get());
    }
    if old_fen != (*curwin.get()).w_onebuf_opt.wo_fen {
        if foldmethodIsDiff(curwin.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_onebuf_opt.wo_scb != 0
        {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if wp != curwin.get()
                    && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
                    && (*wp).w_onebuf_opt.wo_scb != 0
                {
                    (*wp).w_onebuf_opt.wo_fen = (*curwin.get()).w_onebuf_opt.wo_fen;
                    changed_window_setting(wp);
                }
                wp = (*wp).w_next;
            }
        }
        changed_window_setting(curwin.get());
    }
    if old_fdl as OptInt != (*curwin.get()).w_onebuf_opt.wo_fdl {
        newFoldLevel();
    }
}
unsafe extern "C" fn nv_regreplay(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    loop {
        let c2rust_fresh11 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh11 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg(reg_recorded.get(), false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}
unsafe extern "C" fn nv_colon(mut cap: *mut cmdarg_T) {
    let mut cmd_result: bool = false;
    let mut is_cmdkey: bool = (*cap).cmdchar
        == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    let mut is_lua: bool = (*cap).cmdchar
        == -(253 as ::core::ffi::c_int
            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    if VIsual_active.get() as ::core::ffi::c_int != 0 && !is_cmdkey && !is_lua {
        nv_operator(cap);
        return;
    }
    if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false_0 != 0;
    } else if (*cap).count0 != 0 && !is_cmdkey && !is_lua {
        stuffcharReadbuff('.' as ::core::ffi::c_int);
        if (*cap).count0 > 1 as ::core::ffi::c_int {
            stuffReadbuff(b",.+\0".as_ptr() as *const ::core::ffi::c_char);
            stuffnumReadbuff((*cap).count0 - 1 as ::core::ffi::c_int);
        }
    }
    if KeyTyped.get() {
        msg_ext_set_trigger(b"typed_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        compute_cmdrow();
    }
    if is_lua {
        cmd_result = map_execute_lua(true_0 != 0, false_0 != 0);
    } else {
        cmd_result = do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            if is_cmdkey as ::core::ffi::c_int != 0 {
                Some(
                    getcmdkeycmd
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                )
            } else {
                Some(
                    getexline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                )
            },
            NULL,
            if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int {
                DOCMD_KEEPLINE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) != 0;
    }
    msg_ext_set_trigger(b"\0".as_ptr() as *const ::core::ffi::c_char);
    if cmd_result as ::core::ffi::c_int == false_0 {
        clearop((*cap).oap);
    } else if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int
        && ((*(*cap).oap).start.lnum > (*curbuf.get()).b_ml.ml_line_count
            || (*(*cap).oap).start.col > ml_get_len((*(*cap).oap).start.lnum)
            || did_emsg.get() != 0)
    {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_ctrlg(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(!VIsual_select.get());
        may_trigger_modechanged();
        showmode();
    } else if !checkclearop((*cap).oap) {
        fileinfo((*cap).count0, false_0, true_0 != 0);
    }
}
unsafe extern "C" fn nv_ctrlh(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_select.get() as ::core::ffi::c_int != 0
    {
        (*cap).cmdchar = 'x' as ::core::ffi::c_int;
        v_visop(cap);
    } else {
        nv_left(cap);
    };
}
unsafe extern "C" fn nv_clear(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    syn_stack_free_all((*curwin.get()).w_s);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        (*(*wp).w_s).b_syn_slow = false_0 != 0;
        wp = (*wp).w_next;
    }
    redraw_later(curwin.get(), UPD_CLEAR as ::core::ffi::c_int);
}
unsafe extern "C" fn nv_ctrlo(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_select.get() as ::core::ffi::c_int != 0
    {
        VIsual_select.set(false_0 != 0);
        may_trigger_modechanged();
        showmode();
        restart_VIsual_select.set(2 as ::core::ffi::c_int);
    } else {
        (*cap).count1 = -(*cap).count1;
        nv_pcmark(cap);
    };
}
unsafe extern "C" fn nv_hat(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        buflist_getfile(
            (*cap).count0,
            0 as linenr_T,
            GETF_SETMARK as ::core::ffi::c_int | GETF_ALT as ::core::ffi::c_int,
            false_0,
        );
    }
}
unsafe extern "C" fn nv_Zet(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    match (*cap).nchar {
        90 => {
            do_cmdline_cmd(b"x\0".as_ptr() as *const ::core::ffi::c_char);
        }
        81 => {
            do_cmdline_cmd(b"q!\0".as_ptr() as *const ::core::ffi::c_char);
        }
        82 => {
            if (*cap).count0 >= 1 as ::core::ffi::c_int {
                do_cmdline_cmd(b"restart +qall!\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                do_cmdline_cmd(b"restart\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        _ => {
            clearopbeep((*cap).oap);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_nv_ident(mut c1: ::core::ffi::c_int, mut c2: ::core::ffi::c_int) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut ca: cmdarg_T = cmdarg_T {
        oap: ::core::ptr::null_mut::<oparg_T>(),
        prechar: 0,
        cmdchar: 0,
        nchar: 0,
        nchar_composing: [0; 32],
        nchar_len: 0,
        extra_char: 0,
        opcount: 0,
        count0: 0,
        count1: 0,
        arg: 0,
        retval: 0,
        searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    clear_oparg(&raw mut oa);
    memset(
        &raw mut ca as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    ca.oap = &raw mut oa;
    ca.cmdchar = c1;
    ca.nchar = c2;
    nv_ident(&raw mut ca);
}
unsafe extern "C" fn nv_K_getcmd(
    mut cap: *mut cmdarg_T,
    mut kp: *mut ::core::ffi::c_char,
    mut kp_help: bool,
    mut kp_ex: bool,
    mut ptr_arg: *mut *mut ::core::ffi::c_char,
    mut n: size_t,
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut buflen: *mut size_t,
) -> size_t {
    if kp_help {
        strcpy(
            buf,
            b"help! \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        *buflen =
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as size_t;
        return n;
    }
    if kp_ex {
        *buflen = 0 as size_t;
        *buflen = snprintf(
            buf,
            bufsize,
            b"%s \0".as_ptr() as *const ::core::ffi::c_char,
            kp,
        ) as size_t;
        if (*cap).count0 != 0 as ::core::ffi::c_int {
            *buflen = (*buflen).wrapping_add(snprintf(
                buf.offset(*buflen as isize),
                bufsize.wrapping_sub(*buflen),
                b"%ld \0".as_ptr() as *const ::core::ffi::c_char,
                (*cap).count0 as int64_t,
            ) as size_t);
        }
        return n;
    }
    let mut ptr: *mut ::core::ffi::c_char = *ptr_arg;
    while *ptr as ::core::ffi::c_int == '-' as ::core::ffi::c_int && n > 0 as size_t {
        ptr = ptr.offset(1);
        n = n.wrapping_sub(1);
    }
    if n == 0 as size_t {
        emsg(gettext(&raw const e_noident as *const ::core::ffi::c_char));
        xfree(buf as *mut ::core::ffi::c_void);
        *ptr_arg = ptr;
        return 0 as size_t;
    }
    let mut isman: bool =
        strcmp(kp, b"man\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int;
    let mut isman_s: bool =
        strcmp(kp, b"man -s\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int;
    if (*cap).count0 != 0 as ::core::ffi::c_int
        && !(isman as ::core::ffi::c_int != 0 || isman_s as ::core::ffi::c_int != 0)
    {
        *buflen = snprintf(
            buf,
            bufsize,
            b".,.+%ld\0".as_ptr() as *const ::core::ffi::c_char,
            ((*cap).count0 - 1 as ::core::ffi::c_int) as int64_t,
        ) as size_t;
    }
    do_cmdline_cmd(b"tabnew\0".as_ptr() as *const ::core::ffi::c_char);
    *buflen = (*buflen).wrapping_add(snprintf(
        buf.offset(*buflen as isize),
        bufsize.wrapping_sub(*buflen),
        b"terminal \0".as_ptr() as *const ::core::ffi::c_char,
    ) as size_t);
    if (*cap).count0 == 0 as ::core::ffi::c_int && isman_s as ::core::ffi::c_int != 0 {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"man \0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    } else {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%s \0".as_ptr() as *const ::core::ffi::c_char,
            kp,
        ) as size_t);
    }
    if (*cap).count0 != 0 as ::core::ffi::c_int
        && (isman as ::core::ffi::c_int != 0 || isman_s as ::core::ffi::c_int != 0)
    {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%ld \0".as_ptr() as *const ::core::ffi::c_char,
            (*cap).count0 as int64_t,
        ) as size_t);
    }
    *ptr_arg = ptr;
    return n;
}
unsafe extern "C" fn nv_ident(mut cap: *mut cmdarg_T) {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0 as size_t;
    let mut cmdchar: ::core::ffi::c_int = 0;
    let mut g_cmd: bool = false;
    let mut tag_cmd: bool = false_0 != 0;
    if (*cap).cmdchar == 'g' as ::core::ffi::c_int {
        cmdchar = (*cap).nchar;
        g_cmd = true_0 != 0;
    } else {
        cmdchar = (*cap).cmdchar;
        g_cmd = false_0 != 0;
    }
    if cmdchar == POUND {
        cmdchar = '#' as ::core::ffi::c_int;
    }
    let mut visual_sel: bool = false_0 != 0;
    if cmdchar == ']' as ::core::ffi::c_int
        || cmdchar == Ctrl_RSB
        || cmdchar == 'K' as ::core::ffi::c_int
    {
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && get_visual_text(cap, &raw mut ptr, &raw mut n) as ::core::ffi::c_int == false_0
        {
            return;
        }
        visual_sel = !ptr.is_null();
        if checkclearopq((*cap).oap) {
            return;
        }
    }
    let mut ident_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ptr.is_null() && {
        n = find_ident_under_cursor(
            &raw mut ptr,
            if cmdchar == '*' as ::core::ffi::c_int || cmdchar == '#' as ::core::ffi::c_int {
                FIND_IDENT as ::core::ffi::c_int | FIND_STRING as ::core::ffi::c_int
            } else {
                FIND_IDENT as ::core::ffi::c_int
            },
            &raw mut ident_offset,
        );
        n == 0 as size_t
    } {
        clearop((*cap).oap);
        return;
    }
    let mut kp: *mut ::core::ffi::c_char = if *(*curbuf.get()).b_p_kp as ::core::ffi::c_int == NUL {
        p_kp.get()
    } else {
        (*curbuf.get()).b_p_kp
    };
    let mut kp_helpbang: bool = strequal(kp, b":help!\0".as_ptr() as *const ::core::ffi::c_char);
    let mut kp_help: bool = kp_helpbang as ::core::ffi::c_int != 0
        || *kp as ::core::ffi::c_int == NUL
        || strequal(kp, b":he\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int != 0
        || strequal(kp, b":help\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
            != 0;
    if kp_help as ::core::ffi::c_int != 0
        && !kp_helpbang
        && *skipwhite(ptr) as ::core::ffi::c_int == NUL
    {
        emsg(gettext(&raw const e_noident as *const ::core::ffi::c_char));
        return;
    }
    let mut kp_ex: bool = *kp as ::core::ffi::c_int == ':' as ::core::ffi::c_int;
    let mut bufsize: size_t = n
        .wrapping_mul(2 as size_t)
        .wrapping_add(30 as size_t)
        .wrapping_add(strlen(kp));
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    let mut buflen: size_t = 0 as size_t;
    match cmdchar {
        42 | 35 => {
            setpcmark();
            (*curwin.get()).w_cursor.col = ptr.offset_from(get_cursor_line_ptr()) as colnr_T;
            if !g_cmd && vim_iswordp(ptr) as ::core::ffi::c_int != 0 {
                strcpy(
                    buf,
                    b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                buflen = ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                    as size_t;
            }
            no_smartcase.set(true_0 != 0);
        }
        75 => {
            n = nv_K_getcmd(
                cap,
                kp,
                kp_help,
                kp_ex,
                &raw mut ptr,
                n,
                buf,
                bufsize,
                &raw mut buflen,
            );
            if n == 0 as size_t {
                return;
            }
        }
        93 => {
            tag_cmd = true_0 != 0;
            strcpy(
                buf,
                b"tselect \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            buflen = ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                as size_t;
        }
        _ => {
            tag_cmd = true_0 != 0;
            if (*curbuf.get()).b_help {
                strcpy(
                    buf,
                    b"help! \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                buflen = ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                    as size_t;
            } else if g_cmd {
                strcpy(
                    buf,
                    b"tjump \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                buflen = ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                    as size_t;
            } else if (*cap).count0 == 0 as ::core::ffi::c_int {
                strcpy(
                    buf,
                    b"tag \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                buflen = ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as size_t;
            } else {
                buflen = snprintf(
                    buf,
                    bufsize,
                    b":%ldtag \0".as_ptr() as *const ::core::ffi::c_char,
                    (*cap).count0 as int64_t,
                ) as size_t;
            }
        }
    }
    if cmdchar == 'K' as ::core::ffi::c_int && kp_helpbang as ::core::ffi::c_int != 0 && !visual_sel
    {
        strcpy(
            buf,
            b"help!\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        buflen =
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as size_t;
    } else if cmdchar == 'K' as ::core::ffi::c_int && !kp_help {
        ptr = xstrnsave(ptr, n);
        if kp_ex {
            p = vim_strsave_fnameescape(ptr, VSE_NONE as ::core::ffi::c_int);
        } else {
            p = vim_strsave_shellescape(ptr, true_0 != 0, true_0 != 0);
        }
        xfree(ptr as *mut ::core::ffi::c_void);
        let mut plen: size_t = strlen(p);
        let mut newbuf: *mut ::core::ffi::c_char = xrealloc(
            buf as *mut ::core::ffi::c_void,
            buflen.wrapping_add(plen).wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        buf = newbuf;
        strcpy(buf.offset(buflen as isize), p);
        buflen = buflen.wrapping_add(plen);
        xfree(p as *mut ::core::ffi::c_void);
    } else {
        let mut aux_ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdchar == '*' as ::core::ffi::c_int {
            aux_ptr = (if magic_isset() as ::core::ffi::c_int != 0 {
                b"/.*~[^$\\\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"/^$\\\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
        } else if cmdchar == '#' as ::core::ffi::c_int {
            aux_ptr = (if magic_isset() as ::core::ffi::c_int != 0 {
                b"/?.*~[^$\\\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"/?^$\\\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
        } else if tag_cmd {
            if strcmp(
                (*curbuf.get()).b_p_ft,
                b"help\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                aux_ptr = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                aux_ptr = b"\\|\"\n[\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
        } else {
            aux_ptr =
                b"\\|\"\n*?[\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        p = buf.offset(buflen as isize);
        loop {
            let c2rust_fresh0 = n;
            n = n.wrapping_sub(1);
            if c2rust_fresh0 <= 0 as size_t {
                break;
            }
            if !vim_strchr(aux_ptr, *ptr as uint8_t as ::core::ffi::c_int).is_null() {
                let c2rust_fresh1 = p;
                p = p.offset(1);
                *c2rust_fresh1 = '\\' as ::core::ffi::c_char;
            }
            let len: size_t = (utfc_ptr2len(ptr) - 1 as ::core::ffi::c_int) as size_t;
            let mut i: size_t = 0 as size_t;
            while i < len && n > 0 as size_t {
                let c2rust_fresh2 = ptr;
                ptr = ptr.offset(1);
                let c2rust_fresh3 = p;
                p = p.offset(1);
                *c2rust_fresh3 = *c2rust_fresh2;
                i = i.wrapping_add(1);
                n = n.wrapping_sub(1);
            }
            let c2rust_fresh4 = ptr;
            ptr = ptr.offset(1);
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = *c2rust_fresh4;
        }
        *p = NUL as ::core::ffi::c_char;
        buflen = p.offset_from(buf) as size_t;
    }
    if cmdchar == '*' as ::core::ffi::c_int || cmdchar == '#' as ::core::ffi::c_int {
        if !g_cmd && vim_iswordp(mb_prevptr(get_cursor_line_ptr(), ptr)) as ::core::ffi::c_int != 0
        {
            strcpy(
                buf.offset(buflen as isize),
                b"\\>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            buflen = (buflen as ::core::ffi::c_ulong).wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                    as ::core::ffi::c_ulong,
            ) as size_t;
        }
        init_history();
        add_to_history(
            HIST_SEARCH as ::core::ffi::c_int,
            buf,
            buflen,
            true_0 != 0,
            NUL,
        );
        normal_search(
            cap,
            if cmdchar == '*' as ::core::ffi::c_int {
                '/' as ::core::ffi::c_int
            } else {
                '?' as ::core::ffi::c_int
            },
            buf,
            buflen,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
    } else {
        g_tag_at_cursor.set(true_0 != 0);
        do_cmdline_cmd(buf);
        g_tag_at_cursor.set(false_0 != 0);
        if cmdchar == 'K' as ::core::ffi::c_int && !kp_ex && !kp_help {
            restart_edit.set('i' as ::core::ffi::c_int);
            add_map(
                b"<esc>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"<Cmd>bdelete!<CR>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                MODE_TERMINAL as ::core::ffi::c_int,
                true_0 != 0,
            );
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn get_visual_text(
    mut cap: *mut cmdarg_T,
    mut pp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
) -> bool {
    if VIsual_mode.get() != 'V' as ::core::ffi::c_int {
        unadjust_for_sel();
    }
    if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
        if !cap.is_null() {
            clearopbeep((*cap).oap);
        }
        return false_0 != 0;
    }
    if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
        *pp = get_cursor_line_ptr();
        *lenp = get_cursor_line_len() as size_t;
    } else {
        if lt((*curwin.get()).w_cursor, VIsual.get()) {
            *pp = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
            *lenp = ((*VIsual.ptr()).col as size_t)
                .wrapping_sub((*curwin.get()).w_cursor.col as size_t)
                .wrapping_add(1 as size_t);
        } else {
            *pp = ml_get_pos(VIsual.ptr());
            *lenp = ((*curwin.get()).w_cursor.col as size_t)
                .wrapping_sub((*VIsual.ptr()).col as size_t)
                .wrapping_add(1 as size_t);
        }
        if **pp as ::core::ffi::c_int == NUL {
            *lenp = 0 as size_t;
        }
        if *lenp > 0 as size_t {
            *lenp = (*lenp).wrapping_add(
                (utfc_ptr2len((*pp).offset((*lenp).wrapping_sub(1 as size_t) as isize))
                    - 1 as ::core::ffi::c_int) as size_t,
            );
        }
    }
    reset_VIsual_and_resel();
    return true_0 != 0;
}
unsafe extern "C" fn nv_tagpop(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        do_tag(
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            DT_POP as ::core::ffi::c_int,
            (*cap).count1,
            false_0,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn nv_scroll(mut cap: *mut cmdarg_T) {
    let mut n: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).cmdchar == 'L' as ::core::ffi::c_int {
        validate_botline_win(curwin.get());
        (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_botline - 1 as linenr_T;
        if (*cap).count1 as linenr_T - 1 as linenr_T >= (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else if win_lines_concealed(curwin.get()) {
            n = (*cap).count1 - 1 as ::core::ffi::c_int;
            while n > 0 as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline
            {
                hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                n += decor_conceal_line(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int;
                if (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                n -= 1;
            }
        } else {
            (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                - ((*cap).count1 - 1 as ::core::ffi::c_int))
                as linenr_T;
        }
    } else {
        if (*cap).cmdchar == 'M' as ::core::ffi::c_int {
            let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            used -=
                win_get_fill(curwin.get(), (*curwin.get()).w_topline) - (*curwin.get()).w_topfill;
            validate_botline_win(curwin.get());
            let mut half: ::core::ffi::c_int = ((*curwin.get()).w_view_height
                - (*curwin.get()).w_empty_rows
                + 1 as ::core::ffi::c_int)
                / 2 as ::core::ffi::c_int;
            n = 0 as ::core::ffi::c_int;
            while ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                if n > 0 as ::core::ffi::c_int
                    && used
                        + win_get_fill(curwin.get(), (*curwin.get()).w_topline + n as linenr_T)
                            / 2 as ::core::ffi::c_int
                        >= half
                {
                    n -= 1;
                    break;
                } else {
                    used += plines_win(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        true_0 != 0,
                    );
                    if used >= half {
                        break;
                    }
                    if hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    ) {
                        n = (lnum - (*curwin.get()).w_topline) as ::core::ffi::c_int;
                    }
                    n += 1;
                }
            }
            if n > 0 as ::core::ffi::c_int && used > (*curwin.get()).w_view_height {
                n -= 1;
            }
        } else {
            n = (*cap).count1 - 1 as ::core::ffi::c_int;
            if win_lines_concealed(curwin.get()) {
                lnum = (*curwin.get()).w_topline;
                while (decor_conceal_line(
                    curwin.get(),
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0
                    || {
                        let c2rust_fresh12 = n;
                        n = n - 1;
                        c2rust_fresh12 > 0 as ::core::ffi::c_int
                    })
                    && lnum < (*curwin.get()).w_botline - 1 as linenr_T
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                    lnum += 1;
                }
                n = (lnum - (*curwin.get()).w_topline) as ::core::ffi::c_int;
            }
        }
        (*curwin.get()).w_cursor.lnum =
            if ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_topline + n as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
    }
    if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int {
        cursor_correct(curwin.get());
    }
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
}
unsafe extern "C" fn nv_right(mut cap: *mut cmdarg_T) {
    let mut n: ::core::ffi::c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = true_0;
        }
        nv_wordcmd(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut past_line: bool = VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int != 'o' as ::core::ffi::c_int;
    if virtual_active(curwin.get()) {
        past_line = false_0 != 0;
    }
    n = (*cap).count1;
    while n > 0 as ::core::ffi::c_int {
        if !past_line && oneright() == false_0
            || past_line as ::core::ffi::c_int != 0
                && *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL
        {
            if ((*cap).cmdchar == ' ' as ::core::ffi::c_int
                && !vim_strchr(p_ww.get(), 's' as ::core::ffi::c_int).is_null()
                || (*cap).cmdchar == 'l' as ::core::ffi::c_int
                    && !vim_strchr(p_ww.get(), 'l' as ::core::ffi::c_int).is_null()
                || (*cap).cmdchar == K_RIGHT
                    && !vim_strchr(p_ww.get(), '>' as ::core::ffi::c_int).is_null())
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int
                    && !(*(*cap).oap).inclusive
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
                {
                    (*(*cap).oap).inclusive = true_0 != 0;
                } else {
                    (*curwin.get()).w_cursor.lnum += 1;
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    (*curwin.get()).w_set_curswant = true_0;
                    (*(*cap).oap).inclusive = false_0 != 0;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int {
                    if n == (*cap).count1 {
                        beep_flush();
                    }
                } else if !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL) {
                    (*(*cap).oap).inclusive = true_0 != 0;
                }
                break;
            }
        } else if past_line {
            (*curwin.get()).w_set_curswant = true_0;
            if virtual_active(curwin.get()) {
                oneright();
            } else {
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_left(mut cap: *mut cmdarg_T) {
    let mut n: ::core::ffi::c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = 1 as ::core::ffi::c_int;
        }
        nv_bck_word(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    n = (*cap).count1;
    while n > 0 as ::core::ffi::c_int {
        if oneleft() == false_0 {
            if (((*cap).cmdchar == K_BS || (*cap).cmdchar == Ctrl_H)
                && !vim_strchr(p_ww.get(), 'b' as ::core::ffi::c_int).is_null()
                || (*cap).cmdchar == 'h' as ::core::ffi::c_int
                    && !vim_strchr(p_ww.get(), 'h' as ::core::ffi::c_int).is_null()
                || (*cap).cmdchar == K_LEFT
                    && !vim_strchr(p_ww.get(), '<' as ::core::ffi::c_int).is_null())
                && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            {
                (*curwin.get()).w_cursor.lnum -= 1;
                coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
                (*curwin.get()).w_set_curswant = true_0;
                if ((*(*cap).oap).op_type == OP_DELETE as ::core::ffi::c_int
                    || (*(*cap).oap).op_type == OP_CHANGE as ::core::ffi::c_int)
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
                {
                    let mut cp: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                    if *cp as ::core::ffi::c_int != NUL {
                        (*curwin.get()).w_cursor.col += utfc_ptr2len(cp);
                    }
                    (*cap).retval |= CA_NO_ADJ_OP_END as ::core::ffi::c_int;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int && n == (*cap).count1 {
                    beep_flush();
                }
                break;
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_up(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = BACKWARD as ::core::ffi::c_int;
        nv_page(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_up(
        (*cap).count1 as linenr_T,
        (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*cap).arg != 0 {
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn nv_down(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = FORWARD as ::core::ffi::c_int;
        nv_page(cap);
    } else if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0 && (*cap).cmdchar == CAR {
        qf_view_result(false_0 != 0);
    } else if cmdwin_type.get() != 0 as ::core::ffi::c_int && (*cap).cmdchar == CAR {
        cmdwin_result.set(CAR);
    } else if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
        && (*cap).cmdchar == CAR
        && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
    {
        prompt_invoke_callback();
        if restart_edit.get() == 0 as ::core::ffi::c_int {
            restart_edit.set('a' as ::core::ffi::c_int);
        }
    } else {
        (*(*cap).oap).motion_type = kMTLineWise;
        if cursor_down(
            (*cap).count1,
            (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        } else if (*cap).arg != 0 {
            beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
    };
}
unsafe extern "C" fn nv_gotofile(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = -1 as linenr_T;
    if check_text_or_curbuf_locked((*cap).oap) {
        return;
    }
    if !check_can_set_curbuf_disabled() {
        return;
    }
    let mut ptr: *mut ::core::ffi::c_char = grab_file_name((*cap).count1, &raw mut lnum);
    if !ptr.is_null() {
        if curbufIsChanged() as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int
            && !buf_hide(curbuf.get())
        {
            autowrite(curbuf.get(), false_0 != 0);
        }
        setpcmark();
        if do_ecmd(
            0 as ::core::ffi::c_int,
            ptr,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_LAST as ::core::ffi::c_int as linenr_T,
            if buf_hide(curbuf.get()) as ::core::ffi::c_int != 0 {
                ECMD_HIDE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            curwin.get(),
        ) == OK
            && (*cap).nchar == 'F' as ::core::ffi::c_int
            && lnum >= 0 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = lnum;
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
        xfree(ptr as *mut ::core::ffi::c_void);
    } else {
        clearop((*cap).oap);
    };
}
unsafe extern "C" fn nv_end(mut cap: *mut cmdarg_T) {
    if (*cap).arg != 0 || mod_mask.get() & MOD_MASK_CTRL != 0 {
        (*cap).arg = true_0;
        nv_goto(cap);
        (*cap).count1 = 1 as ::core::ffi::c_int;
    }
    nv_dollar(cap);
}
unsafe extern "C" fn nv_dollar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    if !virtual_active(curwin.get())
        || gchar_cursor() != NUL
        || (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
    }
    if cursor_down(
        (*cap).count1 - 1 as ::core::ffi::c_int,
        (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_search(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == '?' as ::core::ffi::c_int
        && (*(*cap).oap).op_type == OP_ROT13 as ::core::ffi::c_int
    {
        (*cap).cmdchar = 'g' as ::core::ffi::c_int;
        (*cap).nchar = '?' as ::core::ffi::c_int;
        nv_operator(cap);
        return;
    }
    (*cap).searchbuf = getcmdline(
        (*cap).cmdchar,
        (*cap).count1,
        0 as ::core::ffi::c_int,
        true_0 != 0,
    );
    if (*cap).searchbuf.is_null() {
        clearop(oap);
        return;
    }
    normal_search(
        cap,
        (*cap).cmdchar,
        (*cap).searchbuf,
        strlen((*cap).searchbuf),
        if (*cap).arg != 0 || !equalpos(save_cursor, (*curwin.get()).w_cursor) {
            0 as ::core::ffi::c_int
        } else {
            SEARCH_MARK as ::core::ffi::c_int
        },
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
unsafe extern "C" fn nv_next(mut cap: *mut cmdarg_T) {
    let mut old: pos_T = (*curwin.get()).w_cursor;
    let mut wrapped: ::core::ffi::c_int = false_0;
    let mut i: ::core::ffi::c_int = normal_search(
        cap,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
        SEARCH_MARK as ::core::ffi::c_int | (*cap).arg,
        &raw mut wrapped,
    );
    if i == 1 as ::core::ffi::c_int
        && wrapped == 0
        && equalpos(old, (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        (*cap).count1 += 1 as ::core::ffi::c_int;
        normal_search(
            cap,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as size_t,
            SEARCH_MARK as ::core::ffi::c_int | (*cap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        (*cap).count1 -= 1 as ::core::ffi::c_int;
    }
    if i > 0 as ::core::ffi::c_int
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as ::core::ffi::c_int)
            != win_hl_attr(curwin.get(), HLF_L as ::core::ffi::c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn normal_search(
    mut cap: *mut cmdarg_T,
    mut dir: ::core::ffi::c_int,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut opt: ::core::ffi::c_int,
    mut wrapped: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sia: searchit_arg_T = searchit_arg_T {
        sa_stop_lnum: 0,
        sa_tm: ::core::ptr::null_mut::<proftime_T>(),
        sa_timed_out: 0,
        sa_wrapped: 0,
    };
    let prev_cursor: pos_T = (*curwin.get()).w_cursor;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    memset(
        &raw mut sia as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<searchit_arg_T>(),
    );
    let mut i: ::core::ffi::c_int = do_search(
        (*cap).oap,
        dir,
        dir,
        pat,
        patlen,
        (*cap).count1,
        opt | SEARCH_OPT as ::core::ffi::c_int
            | SEARCH_ECHO as ::core::ffi::c_int
            | SEARCH_MSG as ::core::ffi::c_int,
        &raw mut sia,
    );
    if !wrapped.is_null() {
        *wrapped = sia.sa_wrapped;
    }
    if i == 0 as ::core::ffi::c_int {
        clearop((*cap).oap);
    } else {
        if i == 2 as ::core::ffi::c_int {
            (*(*cap).oap).motion_type = kMTLineWise;
        }
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
            && fdo_flags.get() & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
    }
    if !equalpos((*curwin.get()).w_cursor, prev_cursor)
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as ::core::ffi::c_int)
            != win_hl_attr(curwin.get(), HLF_L as ::core::ffi::c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as ::core::ffi::c_int);
    }
    check_cursor(curwin.get());
    return i;
}
unsafe extern "C" fn nv_csearch(mut cap: *mut cmdarg_T) {
    let mut cursor_dec: bool = false_0 != 0;
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_mode.get() == 'v' as ::core::ffi::c_int
        && VIsual_select_exclu_adj.get() as ::core::ffi::c_int != 0
    {
        unadjust_for_sel();
        cursor_dec = true_0 != 0;
    }
    let mut t_cmd: bool =
        (*cap).cmdchar == 't' as ::core::ffi::c_int || (*cap).cmdchar == 'T' as ::core::ffi::c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    if (*cap).nchar < 0 as ::core::ffi::c_int || searchc(cap, t_cmd) == false_0 {
        clearopbeep((*cap).oap);
        if cursor_dec {
            adjust_for_sel(cap);
        }
        return;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if gchar_cursor() == TAB
        && virtual_active(curwin.get()) as ::core::ffi::c_int != 0
        && (*cap).arg == FORWARD as ::core::ffi::c_int
        && (t_cmd as ::core::ffi::c_int != 0
            || (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int)
    {
        let mut scol: colnr_T = 0;
        let mut ecol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut scol,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ecol,
        );
        (*curwin.get()).w_cursor.coladd = ecol - scol;
    } else {
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    adjust_for_sel(cap);
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_bracket_block(mut cap: *mut cmdarg_T, mut old_pos: *const pos_T) {
    let mut new_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut prev_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut n: ::core::ffi::c_int = 0;
    let mut findc: ::core::ffi::c_int = 0;
    if (*cap).nchar == '*' as ::core::ffi::c_int {
        (*cap).nchar = '/' as ::core::ffi::c_int;
    }
    prev_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
    if (*cap).nchar == 'm' as ::core::ffi::c_int || (*cap).nchar == 'M' as ::core::ffi::c_int {
        if (*cap).cmdchar == '[' as ::core::ffi::c_int {
            findc = '{' as ::core::ffi::c_int;
        } else {
            findc = '}' as ::core::ffi::c_int;
        }
        n = 9999 as ::core::ffi::c_int;
    } else {
        findc = (*cap).nchar;
        n = (*cap).count1;
    }
    while n > 0 as ::core::ffi::c_int {
        pos = findmatchlimit(
            (*cap).oap,
            findc,
            if (*cap).cmdchar == '[' as ::core::ffi::c_int {
                FM_BACKWARD as ::core::ffi::c_int
            } else {
                FM_FORWARD as ::core::ffi::c_int
            },
            0 as int64_t,
        );
        if pos.is_null() {
            if new_pos.lnum == 0 as linenr_T {
                if (*cap).nchar != 'm' as ::core::ffi::c_int
                    && (*cap).nchar != 'M' as ::core::ffi::c_int
                {
                    clearopbeep((*cap).oap);
                }
            } else {
                pos = &raw mut new_pos;
            }
            break;
        } else {
            prev_pos = new_pos;
            (*curwin.get()).w_cursor = *pos;
            new_pos = *pos;
            n -= 1;
        }
    }
    (*curwin.get()).w_cursor = *old_pos;
    if (*cap).nchar == 'm' as ::core::ffi::c_int || (*cap).nchar == 'M' as ::core::ffi::c_int {
        let mut c: ::core::ffi::c_int = 0;
        let mut norm: bool = (findc == '{' as ::core::ffi::c_int) as ::core::ffi::c_int
            == ((*cap).nchar == 'm' as ::core::ffi::c_int) as ::core::ffi::c_int;
        n = (*cap).count1;
        if prev_pos.lnum != 0 as linenr_T {
            pos = &raw mut prev_pos;
            (*curwin.get()).w_cursor = prev_pos;
            if norm {
                n -= 1;
            }
        } else {
            pos = ::core::ptr::null_mut::<pos_T>();
        }
        while n > 0 as ::core::ffi::c_int {
            loop {
                if (if findc == '{' as ::core::ffi::c_int {
                    dec_cursor()
                } else {
                    inc_cursor()
                }) < 0 as ::core::ffi::c_int
                {
                    if pos.is_null() {
                        clearopbeep((*cap).oap);
                    }
                    n = 0 as ::core::ffi::c_int;
                    break;
                } else {
                    c = gchar_cursor();
                    if !(c == '{' as ::core::ffi::c_int || c == '}' as ::core::ffi::c_int) {
                        continue;
                    }
                    if c == findc && norm as ::core::ffi::c_int != 0
                        || n == 1 as ::core::ffi::c_int && !norm
                    {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                        n = 0 as ::core::ffi::c_int;
                    } else if new_pos.lnum == 0 as linenr_T {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                    } else {
                        pos = findmatchlimit(
                            (*cap).oap,
                            findc,
                            if (*cap).cmdchar == '[' as ::core::ffi::c_int {
                                FM_BACKWARD as ::core::ffi::c_int
                            } else {
                                FM_FORWARD as ::core::ffi::c_int
                            },
                            0 as int64_t,
                        );
                        if pos.is_null() {
                            n = 0 as ::core::ffi::c_int;
                        } else {
                            (*curwin.get()).w_cursor = *pos;
                        }
                    }
                    break;
                }
            }
            n -= 1;
        }
        (*curwin.get()).w_cursor = *old_pos;
        if pos.is_null() && new_pos.lnum != 0 as linenr_T {
            clearopbeep((*cap).oap);
        }
    }
    if !pos.is_null() {
        setpcmark();
        (*curwin.get()).w_cursor = *pos;
        (*curwin.get()).w_set_curswant = true_0;
        if fdo_flags.get() & kOptFdoFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        {
            foldOpenCursor();
        }
    }
}
unsafe extern "C" fn nv_brackets(mut cap: *mut cmdarg_T) {
    let mut flag: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    if (*cap).nchar == 'f' as ::core::ffi::c_int {
        nv_gotofile(cap);
    } else if !vim_strchr(
        b"iI\tdD\x04\0".as_ptr() as *const ::core::ffi::c_char,
        (*cap).nchar,
    )
    .is_null()
    {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: size_t = 0;
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if len == 0 as size_t {
            clearop((*cap).oap);
        } else {
            ptr = xmemdupz(ptr as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
            find_pattern_in_path(
                ptr,
                kDirectionNotSet,
                len,
                true_0 != 0,
                if (*cap).count0 == 0 as ::core::ffi::c_int {
                    (*(*__ctype_b_loc()).offset((*cap).nchar as isize) as ::core::ffi::c_int
                        & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0) as ::core::ffi::c_int
                } else {
                    false_0
                } != 0,
                if (*cap).nchar & 0xf as ::core::ffi::c_int
                    == 'd' as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                {
                    FIND_DEFINE as ::core::ffi::c_int
                } else {
                    FIND_ANY as ::core::ffi::c_int
                },
                (*cap).count1,
                if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as ::core::ffi::c_int
                    & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    ACTION_SHOW_ALL as ::core::ffi::c_int
                } else if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as ::core::ffi::c_int
                    & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    ACTION_SHOW as ::core::ffi::c_int
                } else {
                    ACTION_GOTO as ::core::ffi::c_int
                },
                if (*cap).cmdchar == ']' as ::core::ffi::c_int {
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T
                } else {
                    1 as linenr_T
                },
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                false_0 != 0,
                false_0 != 0,
            );
            xfree(ptr as *mut ::core::ffi::c_void);
            (*curwin.get()).w_set_curswant = true_0;
        }
    } else if (*cap).cmdchar == '[' as ::core::ffi::c_int
        && !vim_strchr(
            b"{(*/#mM\0".as_ptr() as *const ::core::ffi::c_char,
            (*cap).nchar,
        )
        .is_null()
        || (*cap).cmdchar == ']' as ::core::ffi::c_int
            && !vim_strchr(
                b"})*/#mM\0".as_ptr() as *const ::core::ffi::c_char,
                (*cap).nchar,
            )
            .is_null()
    {
        nv_bracket_block(cap, &raw mut old_pos);
    } else if (*cap).nchar == '[' as ::core::ffi::c_int || (*cap).nchar == ']' as ::core::ffi::c_int
    {
        if (*cap).nchar == (*cap).cmdchar {
            flag = '{' as ::core::ffi::c_int;
        } else {
            flag = '}' as ::core::ffi::c_int;
        }
        (*curwin.get()).w_set_curswant = true_0;
        if !findpar(
            &raw mut (*(*cap).oap).inclusive,
            (*cap).arg,
            (*cap).count1,
            flag,
            (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int
                && (*cap).arg == FORWARD as ::core::ffi::c_int
                && flag == '{' as ::core::ffi::c_int,
        ) {
            clearopbeep((*cap).oap);
        } else {
            if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int {
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            }
            if fdo_flags.get() & kOptFdoFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                && KeyTyped.get() as ::core::ffi::c_int != 0
                && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
            {
                foldOpenCursor();
            }
        }
    } else if (*cap).nchar == 'p' as ::core::ffi::c_int || (*cap).nchar == 'P' as ::core::ffi::c_int
    {
        nv_put_opt(cap, true_0 != 0);
    } else if (*cap).nchar == '\'' as ::core::ffi::c_int
        || (*cap).nchar == '`' as ::core::ffi::c_int
    {
        let mut fm: *mut fmark_T = pos_to_mark(
            curbuf.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            (*curwin.get()).w_cursor,
        );
        '_c2rust_label: {
            if !fm.is_null() {
            } else {
                __assert_fail(
                    b"fm != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4311 as ::core::ffi::c_uint,
                    b"void nv_brackets(cmdarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut prev_fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
        n = (*cap).count1;
        while n > 0 as ::core::ffi::c_int {
            prev_fm = fm;
            fm = getnextmark(
                &raw mut (*fm).mark,
                if (*cap).cmdchar == '[' as ::core::ffi::c_int {
                    BACKWARD as ::core::ffi::c_int
                } else {
                    FORWARD as ::core::ffi::c_int
                },
                ((*cap).nchar == '\'' as ::core::ffi::c_int) as ::core::ffi::c_int,
            );
            if fm.is_null() {
                break;
            }
            n -= 1;
        }
        if fm.is_null() {
            fm = prev_fm;
        }
        let mut flags: MarkMove = kMarkContext;
        flags = (flags as ::core::ffi::c_uint
            | (if (*cap).nchar == '\'' as ::core::ffi::c_int {
                kMarkBeginLine as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint) as MarkMove;
        nv_mark_move_to(cap, flags, fm);
    } else if (*cap).nchar
        >= -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && (*cap).nchar
            <= -(253 as ::core::ffi::c_int
                + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        do_mouse(
            (*cap).oap,
            (*cap).nchar,
            if (*cap).cmdchar == ']' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            },
            (*cap).count1,
            PUT_FIXINDENT as ::core::ffi::c_int != 0,
        );
    } else if (*cap).nchar == 'z' as ::core::ffi::c_int {
        if foldMoveTo(
            false_0 != 0,
            if (*cap).cmdchar == ']' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'c' as ::core::ffi::c_int {
        if diff_move_to(
            if (*cap).cmdchar == ']' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'r' as ::core::ffi::c_int
        || (*cap).nchar == 's' as ::core::ffi::c_int
        || (*cap).nchar == 'S' as ::core::ffi::c_int
    {
        setpcmark();
        n = 0 as ::core::ffi::c_int;
        while n < (*cap).count1 {
            if spell_move_to(
                curwin.get(),
                if (*cap).cmdchar == ']' as ::core::ffi::c_int {
                    FORWARD as ::core::ffi::c_int
                } else {
                    BACKWARD as ::core::ffi::c_int
                },
                (if (*cap).nchar == 's' as ::core::ffi::c_int {
                    SMT_ALL as ::core::ffi::c_int
                } else {
                    if (*cap).nchar == 'r' as ::core::ffi::c_int {
                        SMT_RARE as ::core::ffi::c_int
                    } else {
                        SMT_BAD as ::core::ffi::c_int
                    }
                }) as smt_T,
                false_0 != 0,
                ::core::ptr::null_mut::<hlf_T>(),
            ) == 0 as size_t
            {
                clearopbeep((*cap).oap);
                break;
            } else {
                (*curwin.get()).w_set_curswant = true_0;
                n += 1;
            }
        }
        if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
            && fdo_flags.get() & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_percent(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    (*(*cap).oap).inclusive = true_0 != 0;
    if (*cap).count0 != 0 {
        if (*cap).count0 > 100 as ::core::ffi::c_int {
            clearopbeep((*cap).oap);
        } else {
            (*(*cap).oap).motion_type = kMTLineWise;
            setpcmark();
            if (*curbuf.get()).b_ml.ml_line_count >= 21474836 as linenr_T {
                (*curwin.get()).w_cursor.lnum =
                    ((*curbuf.get()).b_ml.ml_line_count + 99 as linenr_T) / 100 as linenr_T
                        * (*cap).count0 as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = ((*curbuf.get()).b_ml.ml_line_count
                    * (*cap).count0 as linenr_T
                    + 99 as linenr_T)
                    / 100 as linenr_T;
            }
            (*curwin.get()).w_cursor.lnum = if (if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                (*curwin.get()).w_cursor.lnum
            } else {
                1 as linenr_T
            }) < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                    (*curwin.get()).w_cursor.lnum
                } else {
                    1 as linenr_T
                }
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
    } else {
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).use_reg_one = true_0 != 0;
        pos = findmatch((*cap).oap, NUL);
        if pos.is_null() {
            clearopbeep((*cap).oap);
        } else {
            setpcmark();
            (*curwin.get()).w_cursor = *pos;
            (*curwin.get()).w_set_curswant = true_0;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            adjust_for_sel(cap);
        }
    }
    if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        && lnum != (*curwin.get()).w_cursor.lnum
        && fdo_flags.get() & kOptFdoFlagPercent as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_brace(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if findsent((*cap).arg as Direction, (*cap).count1) == FAIL {
        clearopbeep((*cap).oap);
        return;
    }
    adjust_cursor((*cap).oap);
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_mark(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if setmark((*cap).nchar) == false_0 {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_findpar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if !findpar(
        &raw mut (*(*cap).oap).inclusive,
        (*cap).arg,
        (*cap).count1,
        NUL,
        false_0 != 0,
    ) {
        clearopbeep((*cap).oap);
        return;
    }
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_LOWER as ::core::ffi::c_int
        || VIsual_active.get() as ::core::ffi::c_int != 0
    {
        (*cap).cmdchar = 'g' as ::core::ffi::c_int;
        (*cap).nchar = 'u' as ::core::ffi::c_int;
        nv_operator(cap);
    } else {
        nv_kundo(cap);
    };
}
unsafe extern "C" fn nv_kundo(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_replace(mut cap: *mut cmdarg_T) {
    let mut had_ctrl_v: ::core::ffi::c_int = 0;
    if checkclearop((*cap).oap) {
        return;
    }
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if (*cap).nchar == Ctrl_V || (*cap).nchar == Ctrl_Q {
        had_ctrl_v = Ctrl_V;
        (*cap).nchar = get_literal(false_0 != 0);
        if (*cap).nchar > DEL {
            had_ctrl_v = NUL;
        }
    } else {
        had_ctrl_v = NUL;
    }
    if (*cap).nchar < 0 as ::core::ffi::c_int {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if got_int.get() {
            got_int.set(false_0 != 0);
        }
        if had_ctrl_v != 0 {
            if (*cap).nchar == CAR {
                (*cap).nchar = REPLACE_CR_NCHAR as ::core::ffi::c_int;
            } else if (*cap).nchar == NL {
                (*cap).nchar = REPLACE_NL_NCHAR as ::core::ffi::c_int;
            }
        }
        nv_operator(cap);
        return;
    }
    if virtual_active(curwin.get()) {
        if u_save_cursor() == false_0 {
            return;
        }
        if gchar_cursor() == NUL {
            coladvance_force(getviscol() + (*cap).count1);
            '_c2rust_label: {
                if (*cap).count1 <= 2147483647 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cap->count1 <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4553 as ::core::ffi::c_uint,
                        b"void nv_replace(cmdarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*curwin.get()).w_cursor.col -= (*cap).count1;
        } else if gchar_cursor() == TAB {
            coladvance_force(getviscol());
        }
    }
    if (get_cursor_pos_len() as size_t) < (*cap).count1 as ::core::ffi::c_uint as size_t
        || mb_charlen(get_cursor_pos_ptr()) < (*cap).count1
    {
        clearopbeep((*cap).oap);
        return;
    }
    if had_ctrl_v != Ctrl_V
        && (*cap).nchar == '\t' as ::core::ffi::c_int
        && ((*curbuf.get()).b_p_et != 0 || p_sta.get() != 0)
    {
        stuffnumReadbuff((*cap).count1);
        stuffcharReadbuff('R' as ::core::ffi::c_int);
        stuffcharReadbuff('\t' as ::core::ffi::c_int);
        stuffcharReadbuff(ESC);
        return;
    }
    if u_save_cursor() == false_0 {
        return;
    }
    if had_ctrl_v != Ctrl_V
        && ((*cap).nchar == '\r' as ::core::ffi::c_int
            || (*cap).nchar == '\n' as ::core::ffi::c_int)
    {
        del_chars((*cap).count1, false_0);
        stuffcharReadbuff('\r' as ::core::ffi::c_int);
        stuffcharReadbuff(ESC);
        invoke_edit(cap, true_0, 'r' as ::core::ffi::c_int, false_0);
    } else {
        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count1,
            NUL,
            'r' as ::core::ffi::c_int,
            NUL,
            had_ctrl_v,
            0 as ::core::ffi::c_int,
        );
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        let old_State: ::core::ffi::c_int = State.get();
        if (*cap).nchar_len > 0 as ::core::ffi::c_int {
            AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut ::core::ffi::c_char);
        } else {
            AppendCharToRedobuff((*cap).nchar);
        }
        let mut n: ::core::ffi::c_int = (*cap).count1;
        while n > 0 as ::core::ffi::c_int {
            State.set(MODE_REPLACE as ::core::ffi::c_int);
            if (*cap).nchar == Ctrl_E || (*cap).nchar == Ctrl_Y {
                let mut c: ::core::ffi::c_int = ins_copychar(
                    (*curwin.get()).w_cursor.lnum
                        + (if (*cap).nchar == Ctrl_Y {
                            -1 as linenr_T
                        } else {
                            1 as linenr_T
                        }),
                );
                if c != NUL {
                    ins_char(c);
                } else {
                    (*curwin.get()).w_cursor.col += 1;
                }
            } else if (*cap).nchar_len != 0 {
                ins_char_bytes(
                    &raw mut (*cap).nchar_composing as *mut ::core::ffi::c_char,
                    (*cap).nchar_len as size_t,
                );
            } else {
                ins_char((*cap).nchar);
            }
            State.set(old_State);
            n -= 1;
        }
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_set_curswant = true_0;
        set_last_insert((*cap).nchar);
    }
    foldUpdateAfterInsert();
}
unsafe extern "C" fn v_swap_corners(mut cmdchar: ::core::ffi::c_int) {
    let mut left: colnr_T = 0;
    let mut right: colnr_T = 0;
    if cmdchar == 'O' as ::core::ffi::c_int && VIsual_mode.get() == Ctrl_V {
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        getvcols(
            curwin.get(),
            &raw mut old_cursor,
            VIsual.ptr(),
            &raw mut left,
            &raw mut right,
        );
        (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
        coladvance(curwin.get(), left);
        VIsual.set((*curwin.get()).w_cursor);
        (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
        (*curwin.get()).w_curswant = right;
        if old_cursor.lnum >= (*VIsual.ptr()).lnum
            && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        {
            (*curwin.get()).w_curswant += 1;
        }
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
        if (*curwin.get()).w_cursor.col == old_cursor.col
            && (!virtual_active(curwin.get())
                || (*curwin.get()).w_cursor.coladd == old_cursor.coladd)
        {
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            if old_cursor.lnum <= (*VIsual.ptr()).lnum
                && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            {
                right += 1;
            }
            coladvance(curwin.get(), right);
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
            coladvance(curwin.get(), left);
            (*curwin.get()).w_curswant = left;
        }
    } else {
        let mut old_cursor_0: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = VIsual.get();
        VIsual.set(old_cursor_0);
        (*curwin.get()).w_set_curswant = true_0;
    };
}
unsafe extern "C" fn nv_Replace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'c' as ::core::ffi::c_int;
        (*cap).nchar = NUL;
        VIsual_mode_orig.set(VIsual_mode.get());
        VIsual_mode.set('V' as ::core::ffi::c_int);
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
    } else {
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(
            cap,
            false_0,
            if (*cap).arg != 0 {
                'V' as ::core::ffi::c_int
            } else {
                'R' as ::core::ffi::c_int
            },
            false_0,
        );
    };
}
unsafe extern "C" fn nv_vreplace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'r' as ::core::ffi::c_int;
        (*cap).nchar = (*cap).extra_char;
        nv_replace(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
    } else {
        if (*cap).extra_char == Ctrl_V || (*cap).extra_char == Ctrl_Q {
            (*cap).extra_char = get_literal(false_0 != 0);
        }
        if (*cap).extra_char < ' ' as ::core::ffi::c_int {
            stuffcharReadbuff(Ctrl_V);
        }
        stuffcharReadbuff((*cap).extra_char);
        stuffcharReadbuff(ESC);
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(cap, true_0, 'v' as ::core::ffi::c_int, false_0);
    };
}
unsafe extern "C" fn n_swapchar(mut cap: *mut cmdarg_T) {
    let mut did_change: bool = false_0 != 0;
    if checkclearopq((*cap).oap) {
        return;
    }
    if *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
        && vim_strchr(p_ww.get(), '~' as ::core::ffi::c_int).is_null()
    {
        clearopbeep((*cap).oap);
        return;
    }
    prep_redo_cmd(cap);
    if u_save_cursor() == false_0 {
        return;
    }
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    let mut n: ::core::ffi::c_int = (*cap).count1;
    while n > 0 as ::core::ffi::c_int {
        did_change = did_change as ::core::ffi::c_int
            | swapchar((*(*cap).oap).op_type, &raw mut (*curwin.get()).w_cursor)
                as ::core::ffi::c_int
            != 0;
        inc_cursor();
        if gchar_cursor() == NUL {
            if !(!vim_strchr(p_ww.get(), '~' as ::core::ffi::c_int).is_null()
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count)
            {
                break;
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            if n > 1 as ::core::ffi::c_int {
                if u_savesub((*curwin.get()).w_cursor.lnum) == false_0 {
                    break;
                }
                u_clearline(curbuf.get());
            }
        }
        n -= 1;
    }
    check_cursor(curwin.get());
    (*curwin.get()).w_set_curswant = true_0;
    if did_change {
        changed_lines(
            curbuf.get(),
            startpos.lnum,
            startpos.col,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        (*curbuf.get()).b_op_start = startpos;
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        if (*curbuf.get()).b_op_end.col > 0 as ::core::ffi::c_int {
            (*curbuf.get()).b_op_end.col -= 1;
        }
    }
}
unsafe extern "C" fn nv_mark_move_to(
    mut cap: *mut cmdarg_T,
    mut flags: MarkMove,
    mut fm: *mut fmark_T,
) -> MarkMoveRes {
    let mut res: MarkMoveRes = mark_move_to(fm, flags);
    if res as ::core::ffi::c_uint & kMarkMoveFailed as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        clearop((*cap).oap);
    }
    (*(*cap).oap).motion_type = (if flags as ::core::ffi::c_uint
        & kMarkBeginLine as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        kMTLineWise as ::core::ffi::c_int
    } else {
        kMTCharWise as ::core::ffi::c_int
    }) as MotionType;
    if (*cap).cmdchar == '`' as ::core::ffi::c_int {
        (*(*cap).oap).use_reg_one = true_0 != 0;
    }
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    return res;
}
unsafe extern "C" fn v_visop(mut cap: *mut cmdarg_T) {
    static trans: GlobalCell<[::core::ffi::c_char; 17]> = GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"YyDdCcxdXdAAIIrr\0")
    });
    if *(*__ctype_b_loc()).offset((*cap).cmdchar as isize) as ::core::ffi::c_int
        & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        if VIsual_mode.get() != Ctrl_V {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as ::core::ffi::c_int);
        } else if (*cap).cmdchar == 'C' as ::core::ffi::c_int
            || (*cap).cmdchar == 'D' as ::core::ffi::c_int
        {
            (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
        }
    }
    (*cap).cmdchar = *vim_strchr(trans.ptr() as *mut ::core::ffi::c_char, (*cap).cmdchar)
        .offset(1 as ::core::ffi::c_int as isize) as uint8_t
        as ::core::ffi::c_int;
    nv_operator(cap);
}
unsafe extern "C" fn nv_subst(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if (*cap).cmdchar == 'S' as ::core::ffi::c_int {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as ::core::ffi::c_int);
        }
        (*cap).cmdchar = 'c' as ::core::ffi::c_int;
        nv_operator(cap);
    } else {
        nv_optrans(cap);
    };
}
unsafe extern "C" fn nv_abbrev(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_DEL
        || (*cap).cmdchar
            == -(253 as ::core::ffi::c_int
                + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*cap).cmdchar = 'x' as ::core::ffi::c_int;
    }
    if VIsual_active.get() {
        v_visop(cap);
    } else {
        nv_optrans(cap);
    };
}
unsafe extern "C" fn nv_optrans(mut cap: *mut cmdarg_T) {
    static ar: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
        b"dl\0".as_ptr() as *const ::core::ffi::c_char,
        b"dh\0".as_ptr() as *const ::core::ffi::c_char,
        b"d$\0".as_ptr() as *const ::core::ffi::c_char,
        b"c$\0".as_ptr() as *const ::core::ffi::c_char,
        b"cl\0".as_ptr() as *const ::core::ffi::c_char,
        b"cc\0".as_ptr() as *const ::core::ffi::c_char,
        b"yy\0".as_ptr() as *const ::core::ffi::c_char,
        b":s\r\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    static str: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"xXDCsSY&\0".as_ptr() as *const ::core::ffi::c_char);
    if !checkclearopq((*cap).oap) {
        if (*cap).count0 != 0 {
            stuffnumReadbuff((*cap).count0);
        }
        stuffReadbuff(
            (*ar.ptr())[strchr(
                str.get(),
                (*cap).cmdchar as ::core::ffi::c_char as ::core::ffi::c_int,
            )
            .offset_from(str.get()) as usize] as *const ::core::ffi::c_char,
        );
    }
    (*cap).opcount = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nv_gomark(mut cap: *mut cmdarg_T) {
    let mut name: ::core::ffi::c_int = 0;
    let mut flags: MarkMove =
        (if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            kMarkSetView as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as MarkMove;
    if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int {
        flags = 0 as MarkMove;
    }
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if (*cap).cmdchar == 'g' as ::core::ffi::c_int {
        name = (*cap).extra_char;
        flags = (flags as ::core::ffi::c_uint
            | KMarkNoContext as ::core::ffi::c_int as ::core::ffi::c_uint)
            as MarkMove;
    } else {
        name = (*cap).nchar;
        flags = (flags as ::core::ffi::c_uint
            | kMarkContext as ::core::ffi::c_int as ::core::ffi::c_uint)
            as MarkMove;
    }
    flags = (flags as ::core::ffi::c_uint
        | (if (*cap).arg != 0 {
            kMarkBeginLine as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint) as MarkMove;
    flags = (flags as ::core::ffi::c_uint
        | (if (*cap).count0 != 0 {
            kMarkSetView as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint) as MarkMove;
    let mut fm: *mut fmark_T = mark_get(
        curbuf.get(),
        curwin.get(),
        ::core::ptr::null_mut::<fmark_T>(),
        kMarkAll,
        name,
    );
    move_res = nv_mark_move_to(cap, flags, fm);
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        && move_res as ::core::ffi::c_uint
            & kMarkMoveSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        && (move_res as ::core::ffi::c_uint
            & kMarkSwitchedBuf as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
            || move_res as ::core::ffi::c_uint
                & kMarkChangedCursor as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0)
        && fdo_flags.get() & kOptFdoFlagMark as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && old_KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_pcmark(mut cap: *mut cmdarg_T) {
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut flags: MarkMove =
        (if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            kMarkSetView as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as MarkMove;
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == TAB && mod_mask.get() == MOD_MASK_CTRL {
        if !goto_tabpage_lastused() {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if (*cap).cmdchar == 'g' as ::core::ffi::c_int {
        fm = get_changelist(curbuf.get(), curwin.get(), (*cap).count1);
    } else {
        fm = get_jumplist(curwin.get(), (*cap).count1);
        flags = (flags as ::core::ffi::c_uint
            | (KMarkNoContext as ::core::ffi::c_int | kMarkJumpList as ::core::ffi::c_int)
                as ::core::ffi::c_uint) as MarkMove;
    }
    if !fm.is_null() {
        move_res = nv_mark_move_to(cap, flags, fm);
    } else if (*cap).cmdchar == 'g' as ::core::ffi::c_int {
        if (*curbuf.get()).b_changelistlen == 0 as ::core::ffi::c_int {
            emsg(gettext(
                (e_changelist_is_empty.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
        } else if (*cap).count1 < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E662: At start of changelist\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(
                b"E663: At end of changelist\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    } else {
        clearopbeep((*cap).oap);
    }
    if (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        && (move_res as ::core::ffi::c_uint
            & kMarkSwitchedBuf as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
            || move_res as ::core::ffi::c_uint
                & kMarkChangedLine as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0)
        && fdo_flags.get() & kOptFdoFlagMark as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && old_KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_regname(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as ::core::ffi::c_int {
        (*cap).nchar = get_expr_register();
    }
    if (*cap).nchar != NUL && valid_yank_reg((*cap).nchar, false_0 != 0) as ::core::ffi::c_int != 0
    {
        (*(*cap).oap).regname = (*cap).nchar;
        (*cap).opcount = (*cap).count0;
        set_reg_var((*(*cap).oap).regname);
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_visual(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == Ctrl_Q {
        (*cap).cmdchar = Ctrl_V;
    }
    if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int {
        (*(*cap).oap).motion_force = (*cap).cmdchar;
        motion_force.set((*(*cap).oap).motion_force);
        finish_op.set(false_0 != 0);
        return;
    }
    VIsual_select.set((*cap).arg != 0);
    if VIsual_active.get() {
        if VIsual_mode.get() == (*cap).cmdchar {
            end_visual_mode();
        } else {
            VIsual_mode.set((*cap).cmdchar);
            showmode();
            may_trigger_modechanged();
        }
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    } else if (*cap).count0 > 0 as ::core::ffi::c_int && resel_VIsual_mode.get() != NUL {
        VIsual.set((*curwin.get()).w_cursor);
        VIsual_active.set(true_0 != 0);
        VIsual_reselect.set(true_0);
        if (*cap).arg == 0 {
            may_start_select('c' as ::core::ffi::c_int);
        }
        setmouse();
        if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
            redraw_cmdline.set(true_0 != 0);
        }
        if resel_VIsual_mode.get() != 'v' as ::core::ffi::c_int
            || resel_VIsual_line_count.get() > 1 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                + (resel_VIsual_line_count.get() * (*cap).count0 as linenr_T - 1 as linenr_T)
                    as ::core::ffi::c_int) as linenr_T;
            check_cursor(curwin.get());
        }
        VIsual_mode.set(resel_VIsual_mode.get());
        if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
            if resel_VIsual_line_count.get() <= 1 as linenr_T {
                update_curswant_force();
                '_c2rust_label: {
                    if (*cap).count0 >= -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        && (*cap).count0 <= 2147483647 as ::core::ffi::c_int
                    {
                    } else {
                        __assert_fail(
                            b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            5057 as ::core::ffi::c_uint,
                            b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                (*curwin.get()).w_curswant +=
                    resel_VIsual_vcol.get() as ::core::ffi::c_int * (*cap).count0;
                if *p_sel.get() as ::core::ffi::c_int != 'e' as ::core::ffi::c_int {
                    (*curwin.get()).w_curswant -= 1;
                }
            } else {
                (*curwin.get()).w_curswant = resel_VIsual_vcol.get();
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        if resel_VIsual_vcol.get() == MAXCOL as ::core::ffi::c_int {
            (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        } else if VIsual_mode.get() == Ctrl_V {
            let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            update_curswant_force();
            '_c2rust_label_0: {
                if (*cap).count0 >= -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && (*cap).count0 <= 2147483647 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5075 as ::core::ffi::c_uint,
                        b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*curwin.get()).w_curswant += resel_VIsual_vcol.get() as ::core::ffi::c_int
                * (*cap).count0
                - 1 as ::core::ffi::c_int;
            (*curwin.get()).w_cursor.lnum = lnum;
            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                (*curwin.get()).w_curswant += 1;
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        } else {
            (*curwin.get()).w_set_curswant = true_0;
        }
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    } else {
        if (*cap).arg == 0 {
            may_start_select('c' as ::core::ffi::c_int);
        }
        n_start_visual_mode((*cap).cmdchar);
        if VIsual_mode.get() != 'V' as ::core::ffi::c_int
            && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        {
            (*cap).count1 += 1;
        } else {
            VIsual_select_exclu_adj.set(false_0 != 0);
        }
        if (*cap).count0 > 0 as ::core::ffi::c_int && {
            (*cap).count1 -= 1;
            (*cap).count1 > 0 as ::core::ffi::c_int
        } {
            if VIsual_mode.get() == 'v' as ::core::ffi::c_int || VIsual_mode.get() == Ctrl_V {
                nv_right(cap);
            } else if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                nv_down(cap);
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn start_selection() {
    may_start_select('k' as ::core::ffi::c_int);
    n_start_visual_mode('v' as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn may_start_select(mut c: ::core::ffi::c_int) {
    VIsual_select.set(
        (c == 'o' as ::core::ffi::c_int
            || stuff_empty() as ::core::ffi::c_int != 0 && typebuf_typed() != 0)
            && !vim_strchr(p_slm.get(), c).is_null(),
    );
}
unsafe extern "C" fn n_start_visual_mode(mut c: ::core::ffi::c_int) {
    VIsual_mode.set(c);
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    if c == Ctrl_V
        && get_ve_flags(curwin.get()) & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        && gchar_cursor() == TAB
    {
        validate_virtcol(curwin.get());
        coladvance(curwin.get(), (*curwin.get()).w_virtcol);
    }
    VIsual.set((*curwin.get()).w_cursor);
    foldAdjustVisual();
    may_trigger_modechanged();
    setmouse();
    conceal_check_cursor_line();
    if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
        redraw_cmdline.set(true_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_INVERTED as ::core::ffi::c_int {
        (*curwin.get()).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*curwin.get()).w_old_visual_lnum = (*curwin.get()).w_cursor.lnum;
    }
    redraw_curbuf_later(UPD_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn nv_window(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == ':' as ::core::ffi::c_int {
        (*cap).cmdchar = ':' as ::core::ffi::c_int;
        (*cap).nchar = NUL;
        nv_colon(cap);
    } else if !checkclearop((*cap).oap) {
        do_window((*cap).nchar, (*cap).count0, NUL);
    }
}
unsafe extern "C" fn nv_suspend(mut cap: *mut cmdarg_T) {
    clearop((*cap).oap);
    if VIsual_active.get() {
        end_visual_mode();
    }
    do_cmdline_cmd(b"st\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn nv_gv_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_visual.vi_start.lnum == 0 as linenr_T
        || (*curbuf.get()).b_visual.vi_start.lnum > (*curbuf.get()).b_ml.ml_line_count
        || (*curbuf.get()).b_visual.vi_end.lnum == 0 as linenr_T
    {
        beep_flush();
        return;
    }
    let mut tpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if VIsual_active.get() {
        let mut i: ::core::ffi::c_int = VIsual_mode.get();
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curbuf.get()).b_visual.vi_mode = i;
        (*curbuf.get()).b_visual_mode_eval = i;
        i = (*curwin.get()).w_curswant as ::core::ffi::c_int;
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        (*curbuf.get()).b_visual.vi_curswant = i as colnr_T;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
        (*curbuf.get()).b_visual.vi_start = VIsual.get();
    } else {
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
    }
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    check_cursor(curwin.get());
    VIsual.set((*curwin.get()).w_cursor);
    (*curwin.get()).w_cursor = tpos;
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*cap).arg != 0 {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as ::core::ffi::c_int);
    } else {
        may_start_select('c' as ::core::ffi::c_int);
    }
    setmouse();
    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    showmode();
}
#[no_mangle]
pub unsafe extern "C" fn nv_g_home_m_cmd(mut cap: *mut cmdarg_T) {
    let mut i: ::core::ffi::c_int = 0;
    let flag: bool = (*cap).nchar == '^' as ::core::ffi::c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
    {
        let mut width1: ::core::ffi::c_int =
            (*curwin.get()).w_view_width - win_col_off(curwin.get());
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
        validate_virtcol(curwin.get());
        i = 0 as ::core::ffi::c_int;
        if (*curwin.get()).w_virtcol >= width1 && width2 > 0 as ::core::ffi::c_int {
            i = ((*curwin.get()).w_virtcol as ::core::ffi::c_int - width1) / width2 * width2
                + width1;
        }
        if (*curwin.get()).w_skipcol > 0 as ::core::ffi::c_int
            && (*curwin.get()).w_cursor.lnum == (*curwin.get()).w_topline
        {
            let mut overlap: ::core::ffi::c_int =
                sms_marker_overlap(curwin.get(), (*curwin.get()).w_view_width - width2);
            if overlap > 0 as ::core::ffi::c_int && i == (*curwin.get()).w_skipcol {
                i += overlap;
            }
        }
    } else {
        i = (*curwin.get()).w_leftcol as ::core::ffi::c_int;
    }
    if (*cap).nchar == 'm' as ::core::ffi::c_int {
        i += ((*curwin.get()).w_view_width - win_col_off(curwin.get())
            + (if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && i > 0 as ::core::ffi::c_int {
                win_col_off2(curwin.get())
            } else {
                0 as ::core::ffi::c_int
            }))
            / 2 as ::core::ffi::c_int;
    }
    coladvance(curwin.get(), i);
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite(i) as ::core::ffi::c_int != 0 && oneright() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if hasAnyFolding(curwin.get()) != 0 {
        validate_cheight(curwin.get());
        if (*curwin.get()).w_cline_folded {
            update_curswant_force();
        }
    }
    adjust_skipcol();
}
unsafe extern "C" fn nv_g_underscore_cmd(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
    if cursor_down(
        (*cap).count1 - 1 as ::core::ffi::c_int,
        (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
        return;
    }
    let mut ptr: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && *ptr.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int == NUL
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    while (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && ascii_iswhite(*ptr.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    (*curwin.get()).w_set_curswant = true_0;
    adjust_for_sel(cap);
}
unsafe extern "C" fn nv_g_dollar_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: ::core::ffi::c_int = 0;
    let mut col_off: ::core::ffi::c_int = win_col_off(curwin.get());
    let flag: bool = (*cap).nchar == K_END || (*cap).nchar == K_KEND;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = true_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
    {
        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
        if (*cap).count1 == 1 as ::core::ffi::c_int {
            let mut width1: ::core::ffi::c_int = (*curwin.get()).w_view_width - col_off;
            let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
            validate_virtcol(curwin.get());
            i = width1 - 1 as ::core::ffi::c_int;
            if (*curwin.get()).w_virtcol >= width1 {
                i += (((*curwin.get()).w_virtcol as ::core::ffi::c_int - width1) / width2
                    + 1 as ::core::ffi::c_int)
                    * width2;
            }
            coladvance(curwin.get(), i);
            update_curswant_force();
            if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            {
                if (*curwin.get()).w_virtcol > i {
                    (*curwin.get()).w_cursor.col -= 1;
                }
            }
        } else if nv_screengo(
            oap,
            FORWARD as ::core::ffi::c_int,
            (*cap).count1 - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            == false_0
        {
            clearopbeep(oap);
        }
    } else {
        if (*cap).count1 > 1 as ::core::ffi::c_int {
            cursor_down((*cap).count1 - 1 as ::core::ffi::c_int, false_0 != 0);
        }
        i = (*curwin.get()).w_leftcol as ::core::ffi::c_int + (*curwin.get()).w_view_width
            - col_off
            - 1 as ::core::ffi::c_int;
        coladvance(curwin.get(), i);
        if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            && utf_ptr2cells(get_cursor_pos_ptr()) > 1 as ::core::ffi::c_int
        {
            let mut vcol: colnr_T = 0;
            getvvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol,
            );
            if vcol
                >= (*curwin.get()).w_leftcol as ::core::ffi::c_int + (*curwin.get()).w_view_width
                    - col_off
            {
                (*curwin.get()).w_cursor.col -= 1;
            }
        }
        update_curswant_force();
    }
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite_or_nul(i) as ::core::ffi::c_int != 0 && oneleft() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
}
unsafe extern "C" fn nv_gi_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_last_insert.mark.lnum != 0 as linenr_T {
        (*curwin.get()).w_cursor = (*curbuf.get()).b_last_insert.mark;
        check_cursor_lnum(curwin.get());
        let mut i: ::core::ffi::c_int = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col > i {
            if virtual_active(curwin.get()) {
                (*curwin.get()).w_cursor.coladd +=
                    (*curwin.get()).w_cursor.col as ::core::ffi::c_int - i;
            }
            (*curwin.get()).w_cursor.col = i as colnr_T;
        }
    }
    (*cap).cmdchar = 'i' as ::core::ffi::c_int;
    nv_edit(cap);
}
unsafe extern "C" fn nv_g_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: ::core::ffi::c_int = 0;
    's_650: {
        'c_40473: {
            'c_36907: {
                match (*cap).nchar {
                    Ctrl_A | Ctrl_X => {
                        if VIsual_active.get() {
                            (*cap).arg = true_0;
                            (*cap).cmdchar = (*cap).nchar;
                            (*cap).nchar = NUL;
                            nv_addsub(cap);
                        } else {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    82 => {
                        (*cap).arg = true_0;
                        nv_Replace(cap);
                        break 's_650;
                    }
                    114 => {
                        nv_vreplace(cap);
                        break 's_650;
                    }
                    38 => {
                        do_cmdline_cmd(b"%s//~/&\0".as_ptr() as *const ::core::ffi::c_char);
                        break 's_650;
                    }
                    118 => {
                        nv_gv_cmd(cap);
                        break 's_650;
                    }
                    86 => {
                        VIsual_reselect.set(false_0);
                        break 's_650;
                    }
                    K_BS => {
                        (*cap).nchar = Ctrl_H;
                    }
                    104 | 72 | Ctrl_H => {}
                    78 | 110 => {
                        if current_search((*cap).count1, (*cap).nchar == 'n' as ::core::ffi::c_int)
                            == 0
                        {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    106 | K_DOWN => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_down(
                                (*cap).count1,
                                (*oap).op_type == OP_NOP as ::core::ffi::c_int,
                            );
                        } else {
                            i = nv_screengo(
                                oap,
                                FORWARD as ::core::ffi::c_int,
                                (*cap).count1,
                                false_0 != 0,
                            ) as ::core::ffi::c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    107 | K_UP => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_up(
                                (*cap).count1 as linenr_T,
                                (*oap).op_type == OP_NOP as ::core::ffi::c_int,
                            );
                        } else {
                            i = nv_screengo(
                                oap,
                                BACKWARD as ::core::ffi::c_int,
                                (*cap).count1,
                                false_0 != 0,
                            ) as ::core::ffi::c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    74 => {
                        nv_join(cap);
                        break 's_650;
                    }
                    94 | 48 | 109 | K_HOME | K_KHOME => {
                        nv_g_home_m_cmd(cap);
                        break 's_650;
                    }
                    77 => {
                        (*oap).motion_type = kMTCharWise;
                        (*oap).inclusive = false_0 != 0;
                        i = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                        if (*cap).count0 > 0 as ::core::ffi::c_int
                            && (*cap).count0 <= 100 as ::core::ffi::c_int
                        {
                            coladvance(curwin.get(), i * (*cap).count0 / 100 as ::core::ffi::c_int);
                        } else {
                            coladvance(curwin.get(), i / 2 as ::core::ffi::c_int);
                        }
                        (*curwin.get()).w_set_curswant = true_0;
                        break 's_650;
                    }
                    95 => {
                        nv_g_underscore_cmd(cap);
                        break 's_650;
                    }
                    36 | K_END | K_KEND => {
                        nv_g_dollar_cmd(cap);
                        break 's_650;
                    }
                    42 | 35 | POUND | Ctrl_RSB | 93 => {
                        nv_ident(cap);
                        break 's_650;
                    }
                    101 | 69 => {
                        (*oap).motion_type = kMTCharWise;
                        (*curwin.get()).w_set_curswant = true_0;
                        (*oap).inclusive = true_0 != 0;
                        if bckend_word(
                            (*cap).count1,
                            (*cap).nchar == 'E' as ::core::ffi::c_int,
                            false_0 != 0,
                        ) == false_0
                        {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    Ctrl_G => {
                        cursor_pos_info(::core::ptr::null_mut::<dict_T>());
                        break 's_650;
                    }
                    105 => {
                        nv_gi_cmd(cap);
                        break 's_650;
                    }
                    73 => {
                        beginline(0 as ::core::ffi::c_int);
                        if !checkclearopq(oap) {
                            invoke_edit(cap, false_0, 'g' as ::core::ffi::c_int, false_0);
                        }
                        break 's_650;
                    }
                    102 | 70 => {
                        nv_gotofile(cap);
                        break 's_650;
                    }
                    39 => {
                        (*cap).arg = true_0;
                        break 'c_36907;
                    }
                    96 => {
                        break 'c_36907;
                    }
                    115 => {
                        do_sleep(
                            ((*cap).count1 * 1000 as ::core::ffi::c_int) as int64_t,
                            false_0 != 0,
                        );
                        break 's_650;
                    }
                    97 => {
                        do_ascii(::core::ptr::null_mut::<exarg_T>());
                        break 's_650;
                    }
                    56 => {
                        if (*cap).count0 == 8 as ::core::ffi::c_int {
                            utf_find_illegal();
                        } else {
                            show_utf8();
                        }
                        break 's_650;
                    }
                    60 => {
                        show_sb_text();
                        break 's_650;
                    }
                    103 => {
                        (*cap).arg = false_0;
                        nv_goto(cap);
                        break 's_650;
                    }
                    113 | 119 => {
                        (*oap).cursor_start = (*curwin.get()).w_cursor;
                        break 'c_40473;
                    }
                    126 | 117 | 85 | 63 | 64 => {
                        break 'c_40473;
                    }
                    100 | 68 => {
                        nv_gd(oap, (*cap).nchar, (*cap).count0);
                        break 's_650;
                    }
                    -12285 | -12541 | -12797 | -11517 | -11773 | -12029 | -25853 | -13053
                    | -13309 | -13565 | -23037 | -23293 | -23549 | -23805 | -24061 | -24317 => {
                        mod_mask.set(MOD_MASK_CTRL);
                        do_mouse(
                            oap,
                            (*cap).nchar,
                            BACKWARD as ::core::ffi::c_int,
                            (*cap).count1,
                            false,
                        );
                        break 's_650;
                    }
                    -13821 => {
                        break 's_650;
                    }
                    112 | 80 => {
                        nv_put(cap);
                        break 's_650;
                    }
                    111 => {
                        (*oap).inclusive = false_0 != 0;
                        goto_byte((*cap).count0);
                        break 's_650;
                    }
                    81 => {
                        if !check_text_locked((*cap).oap) && !checkclearopq(oap) {
                            do_exmode();
                        }
                        break 's_650;
                    }
                    44 => {
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    59 => {
                        (*cap).count1 = -(*cap).count1;
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    116 => {
                        if !checkclearop(oap) {
                            goto_tabpage((*cap).count0);
                        }
                        break 's_650;
                    }
                    84 => {
                        if !checkclearop(oap) {
                            goto_tabpage(-(*cap).count1);
                        }
                        break 's_650;
                    }
                    TAB => {
                        if !checkclearop(oap) && !goto_tabpage_lastused() {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    43 | 45 => {
                        if !checkclearopq(oap) {
                            undo_time(
                                if (*cap).nchar == '-' as ::core::ffi::c_int {
                                    -(*cap).count1
                                } else {
                                    (*cap).count1
                                },
                                false_0 != 0,
                                false_0 != 0,
                                false_0 != 0,
                            );
                        }
                        break 's_650;
                    }
                    _ => {
                        clearopbeep(oap);
                        break 's_650;
                    }
                }
                (*cap).cmdchar =
                    (*cap).nchar + ('v' as ::core::ffi::c_int - 'h' as ::core::ffi::c_int);
                (*cap).arg = true_0;
                nv_visual(cap);
                break 's_650;
            }
            nv_gomark(cap);
            break 's_650;
        }
        nv_operator(cap);
    };
}
unsafe extern "C" fn n_opencmd(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == 'O' as ::core::ffi::c_int {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            &raw mut (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    } else {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        );
    }
    (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    if u_save(
        (*curwin.get()).w_cursor.lnum
            - (if (*cap).cmdchar == 'O' as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
        (*curwin.get()).w_cursor.lnum
            + (if (*cap).cmdchar == 'o' as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
    ) != 0
        && open_line(
            if (*cap).cmdchar == 'O' as ::core::ffi::c_int {
                BACKWARD as ::core::ffi::c_int
            } else {
                FORWARD as ::core::ffi::c_int
            },
            if has_format_option(FO_OPEN_COMS) as ::core::ffi::c_int != 0 {
                OPENLINE_DO_COM as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int
            != 0
    {
        if win_cursorline_standout(curwin.get()) {
            (*curwin.get()).w_valid &= !VALID_CROW;
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, true_0);
    }
}
unsafe extern "C" fn nv_dot(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if start_redo(
        (*cap).count0,
        restart_edit.get() != 0 as ::core::ffi::c_int && !arrow_used.get(),
    ) == false_0
    {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_redo_or_register(mut cap: *mut cmdarg_T) {
    if VIsual_select.get() as ::core::ffi::c_int != 0
        && VIsual_active.get() as ::core::ffi::c_int != 0
    {
        (*no_mapping.ptr()) += 1;
        let mut reg: ::core::ffi::c_int = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && reg >= 0 as ::core::ffi::c_int
        {
            if reg < 256 as ::core::ffi::c_int {
                reg = (*langmap_mapchar.ptr())[reg as usize] as ::core::ffi::c_int;
            } else {
                reg = langmap_adjust_mb(reg);
            }
        }
        (*no_mapping.ptr()) -= 1;
        if reg == '"' as ::core::ffi::c_int {
            reg = 0 as ::core::ffi::c_int;
        }
        VIsual_select_reg.set(
            if valid_yank_reg(reg, true_0 != 0) as ::core::ffi::c_int != 0 {
                reg
            } else {
                0 as ::core::ffi::c_int
            },
        );
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_redo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_Undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_UPPER as ::core::ffi::c_int
        || VIsual_active.get() as ::core::ffi::c_int != 0
    {
        (*cap).cmdchar = 'g' as ::core::ffi::c_int;
        (*cap).nchar = 'U' as ::core::ffi::c_int;
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undoline();
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_tilde(mut cap: *mut cmdarg_T) {
    if p_to.get() == 0
        && !VIsual_active.get()
        && (*(*cap).oap).op_type != OP_TILDE as ::core::ffi::c_int
    {
        if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 && !prompt_curpos_editable() {
            clearopbeep((*cap).oap);
            return;
        }
        n_swapchar(cap);
    } else {
        nv_operator(cap);
    };
}
unsafe extern "C" fn nv_operator(mut cap: *mut cmdarg_T) {
    let mut op_type: ::core::ffi::c_int = get_op_type((*cap).cmdchar, (*cap).nchar);
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
        && op_is_change(op_type) != 0
        && !prompt_curpos_editable()
    {
        clearopbeep((*cap).oap);
        return;
    }
    if op_type == (*(*cap).oap).op_type {
        nv_lineop(cap);
    } else if !checkclearop((*cap).oap) {
        (*(*cap).oap).start = (*curwin.get()).w_cursor;
        (*(*cap).oap).op_type = op_type;
        set_op_var(op_type);
    }
}
unsafe extern "C" fn set_op_var(mut optype: ::core::ffi::c_int) {
    if optype == OP_NOP as ::core::ffi::c_int {
        set_vim_var_string(
            VV_OP,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ptrdiff_t,
        );
    } else {
        let mut opchars: [::core::ffi::c_char; 3] = [0; 3];
        let mut opchar0: ::core::ffi::c_int = get_op_char(optype);
        '_c2rust_label: {
            if opchar0 >= 0 as ::core::ffi::c_int
                && opchar0
                    <= 127 as ::core::ffi::c_int * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"opchar0 >= 0 && opchar0 <= UCHAR_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5876 as ::core::ffi::c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        opchars[0 as ::core::ffi::c_int as usize] = opchar0 as ::core::ffi::c_char;
        let mut opchar1: ::core::ffi::c_int = get_extra_op_char(optype);
        '_c2rust_label_0: {
            if opchar1 >= 0 as ::core::ffi::c_int
                && opchar1
                    <= 127 as ::core::ffi::c_int * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"opchar1 >= 0 && opchar1 <= UCHAR_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5880 as ::core::ffi::c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        opchars[1 as ::core::ffi::c_int as usize] = opchar1 as ::core::ffi::c_char;
        opchars[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        set_vim_var_string(
            VV_OP,
            &raw mut opchars as *mut ::core::ffi::c_char,
            2 as ptrdiff_t,
        );
    };
}
unsafe extern "C" fn nv_lineop(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_down(
        (*cap).count1 - 1 as ::core::ffi::c_int,
        (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*(*cap).oap).op_type == OP_DELETE as ::core::ffi::c_int
        && (*(*cap).oap).motion_force != 'v' as ::core::ffi::c_int
        && (*(*cap).oap).motion_force != Ctrl_V
        || (*(*cap).oap).op_type == OP_LSHIFT as ::core::ffi::c_int
        || (*(*cap).oap).op_type == OP_RSHIFT as ::core::ffi::c_int
    {
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else if (*(*cap).oap).op_type != OP_YANK as ::core::ffi::c_int {
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn nv_home(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        nv_goto(cap);
    } else {
        (*cap).count0 = 1 as ::core::ffi::c_int;
        nv_pipe(cap);
    }
    ins_at_eol.set(false_0 != 0);
}
unsafe extern "C" fn nv_pipe(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline(0 as ::core::ffi::c_int);
    if (*cap).count0 > 0 as ::core::ffi::c_int {
        coladvance(curwin.get(), (*cap).count0 - 1 as ::core::ffi::c_int);
        (*curwin.get()).w_curswant = (*cap).count0 - 1 as ::core::ffi::c_int;
    } else {
        (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*curwin.get()).w_set_curswant = false_0;
}
unsafe extern "C" fn nv_bck_word(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if bck_word((*cap).count1, (*cap).arg != 0, false_0 != 0) == false_0 {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_wordcmd(mut cap: *mut cmdarg_T) {
    let mut n: ::core::ffi::c_int = 0;
    let mut word_end: bool = false;
    let mut flag: bool = false_0 != 0;
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == 'e' as ::core::ffi::c_int || (*cap).cmdchar == 'E' as ::core::ffi::c_int {
        word_end = true_0 != 0;
    } else {
        word_end = false_0 != 0;
    }
    (*(*cap).oap).inclusive = word_end;
    if !word_end && (*(*cap).oap).op_type == OP_CHANGE as ::core::ffi::c_int {
        n = gchar_cursor();
        if n != NUL && !ascii_iswhite(n) {
            if !vim_strchr(p_cpo.get(), CPO_CHANGEW).is_null() {
                (*(*cap).oap).inclusive = true_0 != 0;
                word_end = true_0 != 0;
            }
            flag = true_0 != 0;
        }
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*curwin.get()).w_set_curswant = true_0;
    if word_end {
        n = end_word((*cap).count1, (*cap).arg != 0, flag, false_0 != 0);
    } else {
        n = fwd_word(
            (*cap).count1,
            (*cap).arg != 0,
            (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int,
        );
    }
    if lt(startpos, (*curwin.get()).w_cursor) {
        adjust_cursor((*cap).oap);
    }
    if n == false_0 && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int {
        clearopbeep((*cap).oap);
    } else {
        adjust_for_sel(cap);
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        {
            foldOpenCursor();
        }
    };
}
unsafe extern "C" fn adjust_cursor(mut oap: *mut oparg_T) {
    if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && gchar_cursor() == NUL
        && (!VIsual_active.get() || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
        && !virtual_active(curwin.get())
        && get_ve_flags(curwin.get())
            & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
    {
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*oap).inclusive = true_0 != 0;
    }
}
unsafe extern "C" fn nv_beginline(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline((*cap).arg);
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
    ins_at_eol.set(false_0 != 0);
}
unsafe extern "C" fn adjust_for_sel(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).inclusive as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && gchar_cursor() != NUL
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        inc_cursor();
        (*(*cap).oap).inclusive = false_0 != 0;
        VIsual_select_exclu_adj.set(true_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn unadjust_for_sel() -> bool {
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && !equalpos(VIsual.get(), (*curwin.get()).w_cursor)
    {
        return unadjust_for_sel_inner(
            if lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0 {
                &raw mut (*curwin.get()).w_cursor
            } else {
                VIsual.ptr()
            },
        );
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn unadjust_for_sel_inner(mut pp: *mut pos_T) -> bool {
    VIsual_select_exclu_adj.set(false_0 != 0);
    if (*pp).coladd > 0 as ::core::ffi::c_int {
        (*pp).coladd -= 1;
    } else if (*pp).col > 0 as ::core::ffi::c_int {
        (*pp).col -= 1;
        mark_mb_adjustpos(curbuf.get(), pp);
        if virtual_active(curwin.get()) {
            let mut cs: colnr_T = 0;
            let mut ce: colnr_T = 0;
            getvcol(
                curwin.get(),
                pp,
                &raw mut cs,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ce,
            );
            (*pp).coladd = ce - cs;
        }
    } else if (*pp).lnum > 1 as linenr_T {
        (*pp).lnum -= 1;
        (*pp).col = ml_get_len((*pp).lnum);
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn nv_select(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as ::core::ffi::c_int);
    } else if VIsual_reselect.get() != 0 {
        (*cap).nchar = 'v' as ::core::ffi::c_int;
        (*cap).arg = true_0;
        nv_g_cmd(cap);
    }
}
unsafe extern "C" fn nv_goto(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = 0;
    if (*cap).arg != 0 {
        lnum = (*curbuf.get()).b_ml.ml_line_count;
    } else {
        lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).count0 != 0 as ::core::ffi::c_int {
        lnum = (*cap).count0 as linenr_T;
    }
    lnum = if (if lnum > 1 as linenr_T {
        lnum
    } else {
        1 as linenr_T
    }) < (*curbuf.get()).b_ml.ml_line_count
    {
        if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        }
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    (*curwin.get()).w_cursor.lnum = lnum;
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    if fdo_flags.get() & kOptFdoFlagJump as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_normal(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == Ctrl_N || (*cap).nchar == Ctrl_G {
        clearop((*cap).oap);
        if restart_edit.get() != 0 as ::core::ffi::c_int
            && mode_displayed.get() as ::core::ffi::c_int != 0
        {
            clear_cmdline.set(true_0 != 0);
        }
        restart_edit.set(0 as ::core::ffi::c_int);
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            cmdwin_result.set(Ctrl_C);
        }
        if VIsual_active.get() {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        }
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_esc(mut cap: *mut cmdarg_T) {
    let mut no_reason: bool = (*(*cap).oap).op_type == OP_NOP as ::core::ffi::c_int
        && (*cap).opcount == 0 as ::core::ffi::c_int
        && (*cap).count0 == 0 as ::core::ffi::c_int
        && (*(*cap).oap).regname == 0 as ::core::ffi::c_int;
    if (*cap).arg != 0 {
        if restart_edit.get() == 0 as ::core::ffi::c_int
            && cmdwin_type.get() == 0 as ::core::ffi::c_int
            && !VIsual_active.get()
            && no_reason as ::core::ffi::c_int != 0
        {
            if anyBufIsChanged() {
                msg(
                    gettext(
                        b"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    0 as ::core::ffi::c_int,
                );
            } else {
                msg(
                    gettext(b"Type  :qa  and press <Enter> to exit Nvim\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            }
        }
        if restart_edit.get() != 0 as ::core::ffi::c_int {
            redraw_mode.set(true_0 != 0);
        }
        restart_edit.set(0 as ::core::ffi::c_int);
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            cmdwin_result.set(
                -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
            );
            got_int.set(false_0 != 0);
            return;
        }
    } else if cmdwin_type.get() != 0 as ::core::ffi::c_int
        && ex_normal_busy.get() != 0
        && typebuf_was_empty.get() as ::core::ffi::c_int != 0
    {
        cmdwin_result.set(
            -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
        return;
    }
    if VIsual_active.get() {
        end_visual_mode();
        check_cursor_col(curwin.get());
        (*curwin.get()).w_set_curswant = true_0;
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    } else if no_reason {
        vim_beep(kOptBoFlagEsc as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    clearop((*cap).oap);
}
#[no_mangle]
pub unsafe extern "C" fn set_cursor_for_append_to_line() {
    (*curwin.get()).w_set_curswant = true_0;
    if get_ve_flags(curwin.get()) == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
        let save_State: ::core::ffi::c_int = State.get();
        State.set(MODE_INSERT as ::core::ffi::c_int);
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        State.set(save_State);
    } else {
        (*curwin.get()).w_cursor.col += strlen(get_cursor_pos_ptr()) as colnr_T;
    };
}
unsafe extern "C" fn nv_edit(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_INS
        || (*cap).cmdchar
            == -(253 as ::core::ffi::c_int
                + ((KE_KINS as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*cap).cmdchar = 'i' as ::core::ffi::c_int;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && ((*cap).cmdchar == 'A' as ::core::ffi::c_int
            || (*cap).cmdchar == 'I' as ::core::ffi::c_int)
    {
        v_visop(cap);
    } else if ((*cap).cmdchar == 'a' as ::core::ffi::c_int
        || (*cap).cmdchar == 'i' as ::core::ffi::c_int)
        && ((*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int
            || VIsual_active.get() as ::core::ffi::c_int != 0)
    {
        nv_object(cap);
    } else if (*curbuf.get()).b_p_ma == 0 && (*curbuf.get()).terminal.is_null() {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        clearop((*cap).oap);
    } else if !checkclearopq((*cap).oap) {
        match (*cap).cmdchar {
            65 => {
                set_cursor_for_append_to_line();
            }
            73 => {
                beginline(BL_WHITE as ::core::ffi::c_int);
            }
            97 => {
                if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
                    && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                        || *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL
                        || *get_cursor_pos_ptr() as ::core::ffi::c_int == TAB)
                {
                    (*curwin.get()).w_cursor.coladd += 1;
                } else if *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL {
                    inc_cursor();
                }
            }
            _ => {}
        }
        if (*curwin.get()).w_cursor.coladd != 0 && (*cap).cmdchar != 'A' as ::core::ffi::c_int {
            let mut save_State: ::core::ffi::c_int = State.get();
            State.set(MODE_INSERT as ::core::ffi::c_int);
            coladvance(curwin.get(), getviscol());
            State.set(save_State);
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, false_0);
    }
}
unsafe extern "C" fn invoke_edit(
    mut cap: *mut cmdarg_T,
    mut repl: ::core::ffi::c_int,
    mut cmd: ::core::ffi::c_int,
    mut startln: ::core::ffi::c_int,
) {
    let mut restart_edit_save: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if repl != 0 || !stuff_empty() {
        restart_edit_save = restart_edit.get();
    } else {
        restart_edit_save = 0 as ::core::ffi::c_int;
    }
    restart_edit.set(0 as ::core::ffi::c_int);
    if (*cap).cmdchar != 'O' as ::core::ffi::c_int && (*cap).cmdchar != 'o' as ::core::ffi::c_int {
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    }
    if edit(cmd, startln != 0, (*cap).count1) {
        (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
    }
    if restart_edit.get() == 0 as ::core::ffi::c_int {
        restart_edit.set(restart_edit_save);
    }
}
unsafe extern "C" fn nv_object(mut cap: *mut cmdarg_T) {
    let mut flag: bool = false;
    let mut include: bool = false;
    if (*cap).cmdchar == 'i' as ::core::ffi::c_int {
        include = false_0 != 0;
    } else {
        include = true_0 != 0;
    }
    let mut mps_save: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_mps;
    (*curbuf.get()).b_p_mps =
        b"(:),{:},[:],<:>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    match (*cap).nchar {
        119 => {
            flag = current_word((*cap).oap, (*cap).count1, include, false_0 != 0) != 0;
        }
        87 => {
            flag = current_word((*cap).oap, (*cap).count1, include, true_0 != 0) != 0;
        }
        98 | 40 | 41 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '(' as ::core::ffi::c_int,
                ')' as ::core::ffi::c_int,
            ) != 0;
        }
        66 | 123 | 125 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '{' as ::core::ffi::c_int,
                '}' as ::core::ffi::c_int,
            ) != 0;
        }
        91 | 93 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '[' as ::core::ffi::c_int,
                ']' as ::core::ffi::c_int,
            ) != 0;
        }
        60 | 62 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '<' as ::core::ffi::c_int,
                '>' as ::core::ffi::c_int,
            ) != 0;
        }
        116 => {
            (*cap).retval |= CA_NO_ADJ_OP_END as ::core::ffi::c_int;
            flag = current_tagblock((*cap).oap, (*cap).count1, include) != 0;
        }
        112 => {
            flag = current_par(
                (*cap).oap,
                (*cap).count1,
                include,
                'p' as ::core::ffi::c_int,
            ) != 0;
        }
        115 => {
            flag = current_sent((*cap).oap, (*cap).count1, include) != 0;
        }
        34 | 39 | 96 => {
            flag = current_quote((*cap).oap, (*cap).count1, include, (*cap).nchar);
        }
        _ => {
            flag = false_0 != 0;
        }
    }
    (*curbuf.get()).b_p_mps = mps_save;
    if !flag {
        clearopbeep((*cap).oap);
    }
    adjust_cursor_col();
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_record(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_FORMAT as ::core::ffi::c_int {
        (*cap).cmdchar = 'g' as ::core::ffi::c_int;
        (*cap).nchar = 'q' as ::core::ffi::c_int;
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == ':' as ::core::ffi::c_int
        || (*cap).nchar == '/' as ::core::ffi::c_int
        || (*cap).nchar == '?' as ::core::ffi::c_int
    {
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            emsg(gettext(
                (e_cmdline_window_already_open.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            return;
        }
        stuffcharReadbuff((*cap).nchar);
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_CMDWIN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    } else if reg_executing.get() == 0 as ::core::ffi::c_int && do_record((*cap).nchar) == FAIL {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_at(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as ::core::ffi::c_int {
        if get_expr_register() == NUL {
            return;
        }
    }
    loop {
        let c2rust_fresh13 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh13 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg((*cap).nchar, false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}
unsafe extern "C" fn nv_halfpage(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        pagescroll(
            (if (*cap).cmdchar == Ctrl_D {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
            (*cap).count0,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn nv_join(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    (*cap).count0 = if (*cap).count0 > 2 as ::core::ffi::c_int {
        (*cap).count0
    } else {
        2 as ::core::ffi::c_int
    };
    if (*curwin.get()).w_cursor.lnum + (*cap).count0 as linenr_T - 1 as linenr_T
        > (*curbuf.get()).b_ml.ml_line_count
    {
        if (*cap).count0 <= 2 as ::core::ffi::c_int {
            clearopbeep((*cap).oap);
            return;
        }
        (*cap).count0 = ((*curbuf.get()).b_ml.ml_line_count - (*curwin.get()).w_cursor.lnum
            + 1 as linenr_T) as ::core::ffi::c_int;
    }
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        (*cap).nchar,
    );
    do_join(
        (*cap).count0 as size_t,
        (*cap).nchar == NUL,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
}
unsafe extern "C" fn nv_put(mut cap: *mut cmdarg_T) {
    nv_put_opt(cap, false_0 != 0);
}
unsafe extern "C" fn nv_put_opt(mut cap: *mut cmdarg_T, mut fix_indent: bool) {
    let mut savereg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut empty: bool = false_0 != 0;
    let mut was_visual: bool = false_0 != 0;
    let mut dir: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let save_fen: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    if (*(*cap).oap).op_type != OP_NOP as ::core::ffi::c_int {
        if (*(*cap).oap).op_type == OP_DELETE as ::core::ffi::c_int
            && (*cap).cmdchar == 'p' as ::core::ffi::c_int
        {
            clearop((*cap).oap);
            '_c2rust_label: {
                if (*cap).opcount >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cap->opcount >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        6502 as ::core::ffi::c_uint,
                        b"void nv_put_opt(cmdarg_T *, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            nv_diffgetput(true_0 != 0, (*cap).opcount as size_t);
        } else {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 && !prompt_curpos_editable() {
        if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum {
            (*curwin.get()).w_cursor.col = (*curbuf.get()).b_prompt_start.mark.col;
            (*cap).cmdchar = 'P' as ::core::ffi::c_int;
        } else {
            clearopbeep((*cap).oap);
            return;
        }
    }
    if fix_indent {
        dir = if (*cap).cmdchar == ']' as ::core::ffi::c_int
            && (*cap).nchar == 'p' as ::core::ffi::c_int
        {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        };
        flags |= PUT_FIXINDENT as ::core::ffi::c_int;
    } else {
        dir = if (*cap).cmdchar == 'P' as ::core::ffi::c_int
            || ((*cap).cmdchar == 'g' as ::core::ffi::c_int
                || (*cap).cmdchar == 'z' as ::core::ffi::c_int)
                && (*cap).nchar == 'P' as ::core::ffi::c_int
        {
            BACKWARD as ::core::ffi::c_int
        } else {
            FORWARD as ::core::ffi::c_int
        };
    }
    prep_redo_cmd(cap);
    if (*cap).cmdchar == 'g' as ::core::ffi::c_int {
        flags |= PUT_CURSEND as ::core::ffi::c_int;
    } else if (*cap).cmdchar == 'z' as ::core::ffi::c_int {
        flags |= PUT_BLOCK_INNER as ::core::ffi::c_int;
    }
    if VIsual_active.get() {
        was_visual = true_0 != 0;
        let mut regname: ::core::ffi::c_int = (*(*cap).oap).regname;
        let mut keep_registers: bool = (*cap).cmdchar == 'P' as ::core::ffi::c_int;
        let mut clipoverwrite: bool = (regname == '+' as ::core::ffi::c_int
            || regname == '*' as ::core::ffi::c_int)
            && cb_flags.get()
                & (kOptCbFlagUnnamed as ::core::ffi::c_int
                    | kOptCbFlagUnnamedplus as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                != 0;
        if regname == 0 as ::core::ffi::c_int
            || regname == '"' as ::core::ffi::c_int
            || clipoverwrite as ::core::ffi::c_int != 0
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0
            || regname == '-' as ::core::ffi::c_int
        {
            savereg = copy_register(regname);
        }
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
        if !VIsual_active.get()
            || VIsual_mode.get() == 'V' as ::core::ffi::c_int
            || regname != '.' as ::core::ffi::c_int
        {
            (*cap).cmdchar = 'd' as ::core::ffi::c_int;
            (*cap).nchar = NUL;
            (*(*cap).oap).regname = if keep_registers as ::core::ffi::c_int != 0 {
                '_' as ::core::ffi::c_int
            } else {
                NUL
            };
            (*msg_silent.ptr()) += 1;
            nv_operator(cap);
            do_pending_operator(cap, 0 as ::core::ffi::c_int, false_0 != 0);
            empty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
            (*msg_silent.ptr()) -= 1;
            (*(*cap).oap).regname = regname;
        }
        if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
            flags |= PUT_LINE as ::core::ffi::c_int;
        } else if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
            flags |= PUT_LINE_SPLIT as ::core::ffi::c_int;
        }
        if VIsual_mode.get() == Ctrl_V && dir == FORWARD as ::core::ffi::c_int {
            flags |= PUT_LINE_FORWARD as ::core::ffi::c_int;
        }
        dir = BACKWARD as ::core::ffi::c_int;
        if VIsual_mode.get() != 'V' as ::core::ffi::c_int
            && (*curwin.get()).w_cursor.col < (*curbuf.get()).b_op_start.col
            || VIsual_mode.get() == 'V' as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_op_start.lnum
        {
            dir = FORWARD as ::core::ffi::c_int;
        }
        VIsual_active.set(true_0 != 0);
    }
    do_put((*(*cap).oap).regname, savereg, dir, (*cap).count1, flags);
    if !savereg.is_null() {
        free_register(savereg);
        xfree(savereg as *mut ::core::ffi::c_void);
    }
    if was_visual {
        if save_fen != 0 {
            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
        }
        (*curbuf.get()).b_visual.vi_start = (*curbuf.get()).b_op_start;
        (*curbuf.get()).b_visual.vi_end = (*curbuf.get()).b_op_end;
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            inc(&raw mut (*curbuf.get()).b_visual.vi_end);
        }
    }
    if empty as ::core::ffi::c_int != 0
        && *ml_get((*curbuf.get()).b_ml.ml_line_count) as ::core::ffi::c_int == NUL
    {
        ml_delete_flags(
            (*curbuf.get()).b_ml.ml_line_count,
            ML_DEL_MESSAGE as ::core::ffi::c_int,
        );
        deleted_lines(
            (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
            1 as linenr_T,
        );
        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
    }
    auto_format(false_0 != 0, true_0 != 0);
}
unsafe extern "C" fn nv_open(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_DELETE as ::core::ffi::c_int
        && (*cap).cmdchar == 'o' as ::core::ffi::c_int
    {
        clearop((*cap).oap);
        '_c2rust_label: {
            if (*cap).opcount >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"cap->opcount >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    6645 as ::core::ffi::c_uint,
                    b"void nv_open(cmdarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        nv_diffgetput(false_0 != 0, (*cap).opcount as size_t);
    } else if VIsual_active.get() {
        v_swap_corners((*cap).cmdchar);
    } else if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_prompt_start.mark.lnum
    {
        clearopbeep((*cap).oap);
    } else {
        n_opencmd(cap);
    };
}
unsafe extern "C" fn nv_paste(mut cap: *mut cmdarg_T) {
    paste_repeat((*cap).count1);
}
unsafe extern "C" fn nv_event(mut cap: *mut cmdarg_T) {
    may_garbage_collect.set(false_0 != 0);
    let mut may_restart: bool = restart_edit.get() != 0 as ::core::ffi::c_int
        || restart_VIsual_select.get() != 0 as ::core::ffi::c_int;
    state_handle_k_event();
    finish_op.set(false_0 != 0);
    if may_restart {
        (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn normal_cmd(mut oap: *mut oparg_T, mut toplevel: bool) {
    let mut s: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut s);
    s.toplevel = toplevel;
    s.oa = *oap;
    normal_prepare(&raw mut s);
    normal_execute(&raw mut s.state, safe_vgetc());
    *oap = s.oa;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
