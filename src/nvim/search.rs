use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    Direction, EvalFuncData, ExtmarkUndoObject, FileComparison, FileID, FloatAnchor, FloatRelative,
    GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType,
    MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, SearchOffset,
    SearchPattern, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal, Timestamp, TriState, UIExtension,
    VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, __off64_t, __off_t, __time_t, alist_T, auto_event, bhdr_T, blob_T, blobvar_S,
    blocknr_T, buf_T, bufstate_T, chunksize_T, cmdarg_T, cmdmod_T, colnr_T, dict_T, dictitem_T,
    dictvar_S, disptick_T, event_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    magic_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T,
    proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, searchit_arg_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, FILE, QUEUE, _IO_FILE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_leader_len(
        line: *mut ::core::ffi::c_char,
        flags: *mut *mut ::core::ffi::c_char,
        backward: bool,
        include_space: bool,
    ) -> ::core::ffi::c_int;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_def: GlobalCell<*mut ::core::ffi::c_char>;
    static p_inc: GlobalCell<*mut ::core::ffi::c_char>;
    static fdo_flags: GlobalCell<::core::ffi::c_uint>;
    static p_hls: GlobalCell<::core::ffi::c_int>;
    static p_ic: GlobalCell<::core::ffi::c_int>;
    static p_js: GlobalCell<::core::ffi::c_int>;
    static p_mat: GlobalCell<OptInt>;
    static p_msc: GlobalCell<OptInt>;
    static p_ri: GlobalCell<::core::ffi::c_int>;
    static p_so: GlobalCell<OptInt>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    static p_siso: GlobalCell<OptInt>;
    static p_scs: GlobalCell<::core::ffi::c_int>;
    static p_verbose: GlobalCell<OptInt>;
    static p_ws: GlobalCell<::core::ffi::c_int>;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn reverse_text(s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn vim_isfilec(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn add_to_history(
        histype: ::core::ffi::c_int,
        new_entry: *const ::core::ffi::c_char,
        new_entrylen: size_t,
        in_map: bool,
        sep: ::core::ffi::c_int,
    );
    fn inc_cursor() -> ::core::ffi::c_int;
    fn dec_cursor() -> ::core::ffi::c_int;
    fn check_cursor(wp: *mut win_T);
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn show_cursor_info_later(force: bool);
    fn showmode() -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    static e_interr: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_nopresub: [::core::ffi::c_char; 0];
    static e_noprevre: [::core::ffi::c_char; 0];
    static e_patnotf2: [::core::ffi::c_char; 0];
    static top_bot_msg: [::core::ffi::c_char; 0];
    static bot_top_msg: [::core::ffi::c_char; 0];
    static msg_ext_overwrite: GlobalCell<bool>;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn msg_strtrunc(
        s: *const ::core::ffi::c_char,
        force: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn msg_trunc(
        s: *mut ::core::ffi::c_char,
        force: bool,
        hl_id: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn messaging() -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_home_replace(fname: *const ::core::ffi::c_char);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_prt_line(s: *const ::core::ffi::c_char, list: bool);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_clr_eos();
    fn msg_end() -> bool;
    fn msg_check();
    fn verbose_enter();
    fn verbose_leave();
    fn give_warning(message: *const ::core::ffi::c_char, hl: bool, hist: bool);
    fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_nonnull_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn getfile(
        fnum: ::core::ffi::c_int,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        setpm: bool,
        lnum: linenr_T,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn set_no_hlsearch(flag: bool);
    fn gotocmdline(clr: bool);
    fn file_name_in_line(
        line: *mut ::core::ffi::c_char,
        col: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        rel_fname: *mut ::core::ffi::c_char,
        file_lnum: *mut linenr_T,
    ) -> *mut ::core::ffi::c_char;
    fn find_file_name_in_path(
        ptr: *mut ::core::ffi::c_char,
        len: size_t,
        options: ::core::ffi::c_int,
        count: ::core::ffi::c_long,
        rel_fname: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_fgets(buf: *mut ::core::ffi::c_char, size: ::core::ffi::c_int, fp: *mut FILE) -> bool;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn foldOpenCursor();
    fn char_avail() -> bool;
    fn is_pos_in_string(line: *const ::core::ffi::c_char, col: colnr_T) -> ::core::ffi::c_int;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static dollar_vcol: GlobalCell<colnr_T>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scrolled: GlobalCell<::core::ffi::c_int>;
    static msg_nowait: GlobalCell<bool>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static msg_hist_off: GlobalCell<bool>;
    static called_emsg: GlobalCell<::core::ffi::c_int>;
    static rc_did_emsg: GlobalCell<bool>;
    static search_match_lines: GlobalCell<linenr_T>;
    static search_match_endcol: GlobalCell<colnr_T>;
    static no_smartcase: GlobalCell<bool>;
    static curwin: GlobalCell<*mut win_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static sc_col: GlobalCell<::core::ffi::c_int>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static State: GlobalCell<::core::ffi::c_int>;
    static cmdmod: GlobalCell<cmdmod_T>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static cmd_silent: GlobalCell<bool>;
    static IObuff: GlobalCell<[::core::ffi::c_char; 1025]>;
    static KeyTyped: GlobalCell<bool>;
    static KeyStuffed: GlobalCell<::core::ffi::c_int>;
    static got_int: GlobalCell<bool>;
    static searchcmdlen: GlobalCell<::core::ffi::c_int>;
    static g_do_tagpreview: GlobalCell<::core::ffi::c_int>;
    static no_hlsearch: GlobalCell<bool>;
    fn ctrl_x_mode_not_default() -> bool;
    fn compl_status_adding() -> bool;
    fn compl_status_sol() -> bool;
    fn ins_compl_add_infercase(
        str_arg: *mut ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        icase: bool,
        fname: *mut ::core::ffi::c_char,
        dir: Direction,
        cont_s_ipos: bool,
        score: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn find_word_start(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_word_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ins_compl_interrupted() -> bool;
    fn ins_compl_len() -> ::core::ffi::c_int;
    fn ins_compl_check_keys(frequency: ::core::ffi::c_int, in_compl_func: bool);
    fn os_delay(ms: uint64_t, ignoreinput: bool);
    fn os_time() -> Timestamp;
    fn setpcmark();
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_iscomposing_first(c: ::core::ffi::c_int) -> bool;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_isupper(a: ::core::ffi::c_int) -> bool;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_strcmp_ic(
        ic: bool,
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn inc(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn incl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn decl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn setmouse();
    fn validate_cursor(wp: *mut win_T);
    fn may_start_select(c: ::core::ffi::c_int);
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn magic_isset() -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn line_breakcheck();
    fn fast_breakcheck();
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn profile_setlimit(msec: int64_t) -> proftime_T;
    fn profile_passed_limit(tm: proftime_T) -> bool;
    fn skip_regexp_ex(
        startp: *mut ::core::ffi::c_char,
        dirc: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
        newp: *mut *mut ::core::ffi::c_char,
        dropped: *mut ::core::ffi::c_int,
        magic_val: *mut magic_T,
    ) -> *mut ::core::ffi::c_char;
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
    fn ui_busy_start();
    fn ui_busy_stop();
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_flush();
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_enter(wp: *mut win_T, undo_sync: bool);
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const MAGIC_OFF: magic_T = 2;
pub const MAGIC_NONE: magic_T = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_14 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_14 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_14 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_14 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_14 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_14 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_14 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_14 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_14 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_14 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_14 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_14 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_14 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_14 = 1;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_15 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_15 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_15 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_15 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_15 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_15 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_15 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_15 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_15 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_15 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_15 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_15 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_15 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_15 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_15 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_15 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_15 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_15 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_15 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_16 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_16 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_16 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_16 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_16 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_16 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_16 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_16 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_16 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_16 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_17 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_17 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_17 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_17 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_17 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_17 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_17 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_17 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_17 = 79;
pub const SHM_OVER: C2Rust_Unnamed_17 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_17 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_17 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_17 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_17 = 97;
pub const SHM_WRI: C2Rust_Unnamed_17 = 119;
pub const SHM_LINES: C2Rust_Unnamed_17 = 108;
pub const SHM_MOD: C2Rust_Unnamed_17 = 109;
pub const SHM_RO: C2Rust_Unnamed_17 = 114;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const HIST_DEBUG: C2Rust_Unnamed_18 = 4;
pub const HIST_INPUT: C2Rust_Unnamed_18 = 3;
pub const HIST_EXPR: C2Rust_Unnamed_18 = 2;
pub const HIST_SEARCH: C2Rust_Unnamed_18 = 1;
pub const HIST_CMD: C2Rust_Unnamed_18 = 0;
pub const HIST_INVALID: C2Rust_Unnamed_18 = -1;
pub const HIST_DEFAULT: C2Rust_Unnamed_18 = -2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_19 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_19 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_19 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_19 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_19 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_19 = 20;
pub const UPD_VALID: C2Rust_Unnamed_19 = 10;
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
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_20 = 32;
pub const FNAME_REL: C2Rust_Unnamed_20 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_20 = 8;
pub const FNAME_HYP: C2Rust_Unnamed_20 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_20 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_21 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_21 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_21 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_21 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_21 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_21 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_21 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_21 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_21 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_21 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_21 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_21 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_21 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_21 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_21 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_21 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_21 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_21 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_21 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_21 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_22 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_22 = 2;
pub const FIND_ANY: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_23 = 5;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_23 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_23 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_23 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_24 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_24 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_24 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_24 = 512;
pub const SEARCH_START: C2Rust_Unnamed_24 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_24 = 128;
pub const SEARCH_END: C2Rust_Unnamed_24 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_24 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_24 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_24 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_24 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_24 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_25 = 8;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_25 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_25 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_26 = 2;
pub const RE_BOTH: C2Rust_Unnamed_26 = 2;
pub const RE_SUBST: C2Rust_Unnamed_26 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const SEARCH_STAT_DEF_TIMEOUT: C2Rust_Unnamed_27 = 40;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const SEARCH_STAT_BUF_LEN: C2Rust_Unnamed_28 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchstat_T {
    pub cur: ::core::ffi::c_int,
    pub cnt: ::core::ffi::c_int,
    pub exact_match: bool,
    pub incomplete: ::core::ffi::c_int,
    pub last_maxcount: ::core::ffi::c_int,
}
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
pub const LSIZE: C2Rust_Unnamed_29 = 512;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchedFile {
    pub fp: *mut FILE,
    pub name: *mut ::core::ffi::c_char,
    pub lnum: linenr_T,
    pub matched: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
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
unsafe extern "C" fn ltoreq(mut a: pos_T, mut b: pos_T) -> bool {
    return lt(a, b) as ::core::ffi::c_int != 0 || equalpos(a, b) as ::core::ffi::c_int != 0;
}
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const CPO_SHOWMATCH: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const CPO_LINEOFF: ::core::ffi::c_int = 'o' as ::core::ffi::c_int;
pub const CPO_MATCH: ::core::ffi::c_int = '%' as ::core::ffi::c_int;
pub const CPO_SCOLON: ::core::ffi::c_int = ';' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static e_search_hit_top_without_match_for_str: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E384: Search hit TOP without match for: %s\0",
        )
    });
static e_search_hit_bottom_without_match_for_str: GlobalCell<[::core::ffi::c_char; 46]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
            *b"E385: Search hit BOTTOM without match for: %s\0",
        )
    });
static spats: GlobalCell<[SearchPattern; 2]> = GlobalCell::new([
    SearchPattern {
        pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        patlen: 0 as size_t,
        magic: true_0 != 0,
        no_scs: false_0 != 0,
        timestamp: 0 as Timestamp,
        off: SearchOffset {
            dir: '/' as ::core::ffi::c_char,
            line: false_0 != 0,
            end: false_0 != 0,
            off: 0 as int64_t,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    SearchPattern {
        pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        patlen: 0 as size_t,
        magic: true_0 != 0,
        no_scs: false_0 != 0,
        timestamp: 0 as Timestamp,
        off: SearchOffset {
            dir: '/' as ::core::ffi::c_char,
            line: false_0 != 0,
            end: false_0 != 0,
            off: 0 as int64_t,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
]);
static last_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static lastc: GlobalCell<[uint8_t; 2]> = GlobalCell::new([NUL as uint8_t, NUL as uint8_t]);
static lastcdir: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static last_t_cmd: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static lastc_bytes: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new([0; 33]);
static lastc_bytelen: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
static saved_spats: GlobalCell<[SearchPattern; 2]> = GlobalCell::new(
    [SearchPattern {
        pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        patlen: 0,
        magic: false,
        no_scs: false,
        timestamp: 0,
        off: SearchOffset {
            dir: 0,
            line: false,
            end: false,
            off: 0,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    }; 2],
);
static saved_mr_pattern: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static saved_mr_patternlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static saved_spats_last_idx: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static saved_spats_no_hlsearch: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static mr_pattern: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static mr_patternlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
#[no_mangle]
pub unsafe extern "C" fn search_regcomp(
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut used_pat: *mut *mut ::core::ffi::c_char,
    mut pat_save: ::core::ffi::c_int,
    mut pat_use: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut regmatch: *mut regmmatch_T,
) -> ::core::ffi::c_int {
    rc_did_emsg.set(false_0 != 0);
    let mut magic: ::core::ffi::c_int = magic_isset() as ::core::ffi::c_int;
    if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
        let mut i: ::core::ffi::c_int = 0;
        if pat_use == RE_LAST as ::core::ffi::c_int {
            i = last_idx.get();
        } else {
            i = pat_use;
        }
        if (*spats.ptr())[i as usize].pat.is_null() {
            if pat_use == RE_SUBST as ::core::ffi::c_int {
                emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
            } else {
                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
            }
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        }
        pat = (*spats.ptr())[i as usize].pat;
        patlen = (*spats.ptr())[i as usize].patlen;
        magic = (*spats.ptr())[i as usize].magic as ::core::ffi::c_int;
        no_smartcase.set((*spats.ptr())[i as usize].no_scs);
    } else if options & SEARCH_HIS as ::core::ffi::c_int != 0 {
        add_to_history(
            HIST_SEARCH as ::core::ffi::c_int,
            pat,
            patlen,
            true_0 != 0,
            NUL,
        );
    }
    if !used_pat.is_null() {
        *used_pat = pat;
    }
    xfree(mr_pattern.get() as *mut ::core::ffi::c_void);
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0
        && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int == 's' as ::core::ffi::c_int
    {
        mr_pattern.set(reverse_text(pat));
    } else {
        mr_pattern.set(xstrnsave(pat, patlen));
    }
    mr_patternlen.set(patlen);
    if options & SEARCH_KEEP as ::core::ffi::c_int == 0
        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        if pat_save == RE_SEARCH as ::core::ffi::c_int || pat_save == RE_BOTH as ::core::ffi::c_int
        {
            save_re_pat(RE_SEARCH as ::core::ffi::c_int, pat, patlen, magic);
        }
        if pat_save == RE_SUBST as ::core::ffi::c_int || pat_save == RE_BOTH as ::core::ffi::c_int {
            save_re_pat(RE_SUBST as ::core::ffi::c_int, pat, patlen, magic);
        }
    }
    (*regmatch).rmm_ic = ignorecase(pat);
    (*regmatch).rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    (*regmatch).regprog = vim_regcomp(
        pat,
        if magic != 0 {
            RE_MAGIC
        } else {
            0 as ::core::ffi::c_int
        },
    );
    if (*regmatch).regprog.is_null() {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_search_pat() -> *mut ::core::ffi::c_char {
    return mr_pattern.get();
}
#[no_mangle]
pub unsafe extern "C" fn save_re_pat(
    mut idx: ::core::ffi::c_int,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut magic: ::core::ffi::c_int,
) {
    if (*spats.ptr())[idx as usize].pat == pat {
        return;
    }
    free_spat((spats.ptr() as *mut SearchPattern).offset(idx as isize));
    (*spats.ptr())[idx as usize].pat = xstrnsave(pat, patlen);
    (*spats.ptr())[idx as usize].patlen = patlen;
    (*spats.ptr())[idx as usize].magic = magic != 0;
    (*spats.ptr())[idx as usize].no_scs = no_smartcase.get();
    (*spats.ptr())[idx as usize].timestamp = os_time();
    (*spats.ptr())[idx as usize].additional_data = ::core::ptr::null_mut::<AdditionalData>();
    last_idx.set(idx);
    if p_hls.get() != 0 {
        redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    }
    set_no_hlsearch(false_0 != 0);
}
static save_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn save_search_patterns() {
    let c2rust_fresh0 = save_level.get();
    save_level.set(save_level.get() + 1);
    if c2rust_fresh0 != 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[SearchPattern; 2]>()
        .wrapping_div(::core::mem::size_of::<SearchPattern>())
        .wrapping_div(
            (::core::mem::size_of::<[SearchPattern; 2]>()
                .wrapping_rem(::core::mem::size_of::<SearchPattern>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        (*saved_spats.ptr())[i as usize] = (*spats.ptr())[i as usize];
        if !(*spats.ptr())[i as usize].pat.is_null() {
            (*saved_spats.ptr())[i as usize].pat = xstrnsave(
                (*spats.ptr())[i as usize].pat,
                (*spats.ptr())[i as usize].patlen,
            );
            (*saved_spats.ptr())[i as usize].patlen = (*spats.ptr())[i as usize].patlen;
        }
        i = i.wrapping_add(1);
    }
    if (*mr_pattern.ptr()).is_null() {
        saved_mr_pattern.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        saved_mr_patternlen.set(0 as size_t);
    } else {
        saved_mr_pattern.set(xstrnsave(mr_pattern.get(), mr_patternlen.get()));
        saved_mr_patternlen.set(mr_patternlen.get());
    }
    saved_spats_last_idx.set(last_idx.get());
    saved_spats_no_hlsearch.set(no_hlsearch.get());
}
#[no_mangle]
pub unsafe extern "C" fn restore_search_patterns() {
    (*save_level.ptr()) -= 1;
    if save_level.get() != 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[SearchPattern; 2]>()
        .wrapping_div(::core::mem::size_of::<SearchPattern>())
        .wrapping_div(
            (::core::mem::size_of::<[SearchPattern; 2]>()
                .wrapping_rem(::core::mem::size_of::<SearchPattern>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        free_spat((spats.ptr() as *mut SearchPattern).offset(i as isize));
        (*spats.ptr())[i as usize] = (*saved_spats.ptr())[i as usize];
        i = i.wrapping_add(1);
    }
    set_vv_searchforward();
    xfree(mr_pattern.get() as *mut ::core::ffi::c_void);
    mr_pattern.set(saved_mr_pattern.get());
    mr_patternlen.set(saved_mr_patternlen.get());
    last_idx.set(saved_spats_last_idx.get());
    set_no_hlsearch(saved_spats_no_hlsearch.get());
}
#[inline]
unsafe extern "C" fn free_spat(spat: *mut SearchPattern) {
    xfree((*spat).pat as *mut ::core::ffi::c_void);
    xfree((*spat).additional_data as *mut ::core::ffi::c_void);
}
static saved_last_search_spat: GlobalCell<SearchPattern> = GlobalCell::new(SearchPattern {
    pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    patlen: 0,
    magic: false,
    no_scs: false,
    timestamp: 0,
    off: SearchOffset {
        dir: 0,
        line: false,
        end: false,
        off: 0,
    },
    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
});
static did_save_last_search_spat: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static saved_last_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static saved_no_hlsearch: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static saved_search_match_endcol: GlobalCell<colnr_T> = GlobalCell::new(0);
static saved_search_match_lines: GlobalCell<linenr_T> = GlobalCell::new(0);
#[no_mangle]
pub unsafe extern "C" fn save_last_search_pattern() {
    (*did_save_last_search_spat.ptr()) += 1;
    if did_save_last_search_spat.get() != 1 as ::core::ffi::c_int {
        return;
    }
    saved_last_search_spat.set((*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]);
    if !(*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]
        .pat
        .is_null()
    {
        (*saved_last_search_spat.ptr()).pat = xstrnsave(
            (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat,
            (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen,
        );
        (*saved_last_search_spat.ptr()).patlen =
            (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen;
    }
    saved_last_idx.set(last_idx.get());
    saved_no_hlsearch.set(no_hlsearch.get());
}
#[no_mangle]
pub unsafe extern "C" fn restore_last_search_pattern() {
    (*did_save_last_search_spat.ptr()) -= 1;
    if did_save_last_search_spat.get() > 0 as ::core::ffi::c_int {
        return;
    }
    if did_save_last_search_spat.get() != 0 as ::core::ffi::c_int {
        iemsg(
            b"restore_last_search_pattern() called more often than save_last_search_pattern()\0"
                .as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    xfree((*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat as *mut ::core::ffi::c_void);
    (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize] = saved_last_search_spat.get();
    (*saved_last_search_spat.ptr()).pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*saved_last_search_spat.ptr()).patlen = 0 as size_t;
    set_vv_searchforward();
    last_idx.set(saved_last_idx.get());
    set_no_hlsearch(saved_no_hlsearch.get());
}
unsafe extern "C" fn save_incsearch_state() {
    saved_search_match_endcol.set(search_match_endcol.get());
    saved_search_match_lines.set(search_match_lines.get());
}
unsafe extern "C" fn restore_incsearch_state() {
    search_match_endcol.set(saved_search_match_endcol.get());
    search_match_lines.set(saved_search_match_lines.get());
}
#[no_mangle]
pub unsafe extern "C" fn last_search_pattern() -> *mut ::core::ffi::c_char {
    return (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat;
}
#[no_mangle]
pub unsafe extern "C" fn last_search_pattern_len() -> size_t {
    return (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen;
}
#[no_mangle]
pub unsafe extern "C" fn ignorecase(mut pat: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return ignorecase_opt(pat, p_ic.get(), p_scs.get());
}
#[no_mangle]
pub unsafe extern "C" fn ignorecase_opt(
    mut pat: *mut ::core::ffi::c_char,
    mut ic_in: ::core::ffi::c_int,
    mut scs: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ic: ::core::ffi::c_int = ic_in;
    if ic != 0
        && !no_smartcase.get()
        && scs != 0
        && !(ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && (*curbuf.get()).b_p_inf != 0)
    {
        ic = !pat_has_uppercase(pat) as ::core::ffi::c_int;
    }
    no_smartcase.set(false_0 != 0);
    return ic;
}
#[no_mangle]
pub unsafe extern "C" fn pat_has_uppercase(mut pat: *mut ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = pat;
    let mut magic_val: magic_T = MAGIC_ON;
    skip_regexp_ex(
        pat,
        NUL,
        magic_isset() as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        &raw mut magic_val,
    );
    while *p as ::core::ffi::c_int != NUL {
        let l: ::core::ffi::c_int = utfc_ptr2len(p);
        if l > 1 as ::core::ffi::c_int {
            if mb_isupper(utf_ptr2char(p)) {
                return true_0 != 0;
            }
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && magic_val as ::core::ffi::c_uint
                <= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '_' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(3 as ::core::ffi::c_int as isize);
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '%' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(3 as ::core::ffi::c_int as isize);
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else {
                p = p.offset(1 as ::core::ffi::c_int as isize);
            }
        } else if (*p as ::core::ffi::c_int == '%' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int)
            && magic_val as ::core::ffi::c_uint
                == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else {
                p = p.offset(1);
            }
        } else if mb_isupper(*p as uint8_t as ::core::ffi::c_int) {
            return true_0 != 0;
        } else {
            p = p.offset(1);
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn last_csearch() -> *const ::core::ffi::c_char {
    return lastc_bytes.ptr() as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn last_csearch_forward() -> ::core::ffi::c_int {
    return (lastcdir.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn last_csearch_until() -> ::core::ffi::c_int {
    return last_t_cmd.get() as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_last_csearch(
    mut c: ::core::ffi::c_int,
    mut s: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    *(lastc.ptr() as *mut uint8_t) = c as uint8_t;
    lastc_bytelen.set(len);
    if len != 0 {
        memcpy(
            lastc_bytes.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            len as size_t,
        );
    } else {
        memset(
            lastc_bytes.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[::core::ffi::c_char; 33]>(),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_csearch_direction(mut cdir: Direction) {
    lastcdir.set(cdir);
}
#[no_mangle]
pub unsafe extern "C" fn set_csearch_until(mut t_cmd: ::core::ffi::c_int) {
    last_t_cmd.set(t_cmd != 0);
}
#[no_mangle]
pub unsafe extern "C" fn last_search_pat() -> *mut ::core::ffi::c_char {
    return (*spats.ptr())[last_idx.get() as usize].pat;
}
#[no_mangle]
pub unsafe extern "C" fn reset_search_dir() {
    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = '/' as ::core::ffi::c_char;
    set_vv_searchforward();
}
#[no_mangle]
pub unsafe extern "C" fn set_last_search_pat(
    mut s: *const ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
    mut magic: ::core::ffi::c_int,
    mut setlast: bool,
) {
    free_spat((spats.ptr() as *mut SearchPattern).offset(idx as isize));
    if *s as ::core::ffi::c_int == NUL {
        (*spats.ptr())[idx as usize].pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*spats.ptr())[idx as usize].patlen = 0 as size_t;
    } else {
        (*spats.ptr())[idx as usize].patlen = strlen(s);
        (*spats.ptr())[idx as usize].pat = xstrnsave(s, (*spats.ptr())[idx as usize].patlen);
    }
    (*spats.ptr())[idx as usize].timestamp = os_time();
    (*spats.ptr())[idx as usize].additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*spats.ptr())[idx as usize].magic = magic != 0;
    (*spats.ptr())[idx as usize].no_scs = false_0 != 0;
    (*spats.ptr())[idx as usize].off.dir = '/' as ::core::ffi::c_char;
    set_vv_searchforward();
    (*spats.ptr())[idx as usize].off.line = false_0 != 0;
    (*spats.ptr())[idx as usize].off.end = false_0 != 0;
    (*spats.ptr())[idx as usize].off.off = 0 as int64_t;
    if setlast {
        last_idx.set(idx);
    }
    if save_level.get() != 0 {
        free_spat((saved_spats.ptr() as *mut SearchPattern).offset(idx as isize));
        (*saved_spats.ptr())[idx as usize] = (*spats.ptr())[0 as ::core::ffi::c_int as usize];
        if (*spats.ptr())[idx as usize].pat.is_null() {
            (*saved_spats.ptr())[idx as usize].pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*saved_spats.ptr())[idx as usize].patlen = 0 as size_t;
        } else {
            (*saved_spats.ptr())[idx as usize].pat = xstrnsave(
                (*spats.ptr())[idx as usize].pat,
                (*spats.ptr())[idx as usize].patlen,
            );
            (*saved_spats.ptr())[idx as usize].patlen = (*spats.ptr())[idx as usize].patlen;
        }
        saved_spats_last_idx.set(last_idx.get());
    }
    if p_hls.get() != 0 && idx == last_idx.get() && !no_hlsearch.get() {
        redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn last_pat_prog(mut regmatch: *mut regmmatch_T) {
    if (*spats.ptr())[last_idx.get() as usize].pat.is_null() {
        (*regmatch).regprog = ::core::ptr::null_mut::<regprog_T>();
        return;
    }
    (*emsg_off.ptr()) += 1;
    search_regcomp(
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        0 as size_t,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        last_idx.get(),
        SEARCH_KEEP as ::core::ffi::c_int,
        regmatch,
    );
    (*emsg_off.ptr()) -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn searchit(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut end_pos: *mut pos_T,
    mut dir: Direction,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut count: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut pat_use: ::core::ffi::c_int,
    mut extra_arg: *mut searchit_arg_T,
) -> ::core::ffi::c_int {
    let mut found: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut matchcol: colnr_T = 0;
    let mut endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut matchpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut loop_0: ::core::ffi::c_int = 0;
    let mut extra_col: ::core::ffi::c_int = 0;
    let mut start_char_len: ::core::ffi::c_int = 0;
    let mut match_ok: bool = false;
    let mut nmatched: ::core::ffi::c_int = 0;
    let mut submatch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut first_match: bool = true_0 != 0;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    let mut break_loop: bool = false_0 != 0;
    let mut stop_lnum: linenr_T = 0 as linenr_T;
    let mut tm: *mut proftime_T = ::core::ptr::null_mut::<proftime_T>();
    let mut timed_out: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    if !extra_arg.is_null() {
        stop_lnum = (*extra_arg).sa_stop_lnum;
        tm = (*extra_arg).sa_tm;
        timed_out = &raw mut (*extra_arg).sa_timed_out;
    }
    if search_regcomp(
        pat,
        patlen,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        RE_SEARCH as ::core::ffi::c_int,
        pat_use,
        options & SEARCH_HIS as ::core::ffi::c_int + SEARCH_KEEP as ::core::ffi::c_int,
        &raw mut regmatch,
    ) == FAIL
    {
        if options & SEARCH_MSG as ::core::ffi::c_int != 0 && !rc_did_emsg.get() {
            semsg(
                gettext(b"E383: Invalid search string: %s\0".as_ptr() as *const ::core::ffi::c_char),
                mr_pattern.get(),
            );
        }
        return FAIL;
    }
    let search_from_match_end: bool = !vim_strchr(p_cpo.get(), CPO_SEARCH).is_null();
    loop {
        if (*pos).col == MAXCOL as ::core::ffi::c_int {
            start_char_len = 0 as ::core::ffi::c_int;
        } else if (*pos).lnum >= 1 as linenr_T
            && (*pos).lnum <= (*buf).b_ml.ml_line_count
            && (*pos).col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int
        {
            ptr = ml_get_buf(buf, (*pos).lnum);
            if ml_get_buf_len(buf, (*pos).lnum) <= (*pos).col {
                start_char_len = 1 as ::core::ffi::c_int;
            } else {
                start_char_len = utfc_ptr2len(ptr.offset((*pos).col as isize));
            }
        } else {
            start_char_len = 1 as ::core::ffi::c_int;
        }
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            extra_col = if options & SEARCH_START as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                start_char_len
            };
        } else {
            extra_col = if options & SEARCH_START as ::core::ffi::c_int != 0 {
                start_char_len
            } else {
                0 as ::core::ffi::c_int
            };
        }
        let mut start_pos: pos_T = *pos;
        found = 0 as ::core::ffi::c_int;
        let mut at_first_line: ::core::ffi::c_int = true_0;
        if (*pos).lnum == 0 as linenr_T {
            (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
            (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
            at_first_line = false_0;
        }
        if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
            && start_pos.col == 0 as ::core::ffi::c_int
            && options & SEARCH_START as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            lnum = (*pos).lnum - 1 as linenr_T;
            at_first_line = false_0;
        } else {
            lnum = (*pos).lnum;
        }
        loop_0 = 0 as ::core::ffi::c_int;
        while loop_0 <= 1 as ::core::ffi::c_int {
            's_704: while lnum > 0 as linenr_T && lnum <= (*buf).b_ml.ml_line_count {
                if stop_lnum != 0 as linenr_T
                    && (if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                        (lnum > stop_lnum) as ::core::ffi::c_int
                    } else {
                        (lnum < stop_lnum) as ::core::ffi::c_int
                    }) != 0
                {
                    break;
                }
                if !tm.is_null() && profile_passed_limit(*tm) as ::core::ffi::c_int != 0 {
                    break;
                }
                let mut col: colnr_T =
                    if at_first_line != 0 && options & SEARCH_COL as ::core::ffi::c_int != 0 {
                        (*pos).col
                    } else {
                        0 as colnr_T
                    };
                nmatched = vim_regexec_multi(&raw mut regmatch, win, buf, lnum, col, tm, timed_out);
                if regmatch.regprog.is_null() {
                    break;
                }
                if called_emsg.get() > called_emsg_before || !timed_out.is_null() && *timed_out != 0
                {
                    break;
                }
                's_218: {
                    if nmatched > 0 as ::core::ffi::c_int {
                        matchpos = regmatch.startpos[0 as ::core::ffi::c_int as usize];
                        endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                        submatch = first_submatch(&raw mut regmatch);
                        if lnum + matchpos.lnum > (*buf).b_ml.ml_line_count {
                            ptr = b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            ptr = ml_get_buf(buf, lnum + matchpos.lnum);
                        }
                        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                            && at_first_line != 0
                        {
                            match_ok = true_0 != 0;
                            while matchpos.lnum == 0 as linenr_T
                                && (if options & SEARCH_END as ::core::ffi::c_int != 0
                                    && first_match as ::core::ffi::c_int != 0
                                {
                                    (nmatched == 1 as ::core::ffi::c_int
                                        && (endpos.col - 1 as ::core::ffi::c_int)
                                            < start_pos.col + extra_col)
                                        as ::core::ffi::c_int
                                } else {
                                    ((matchpos.col
                                        - (*ptr.offset(matchpos.col as isize) as ::core::ffi::c_int
                                            == NUL)
                                            as ::core::ffi::c_int)
                                        < start_pos.col + extra_col)
                                        as ::core::ffi::c_int
                                }) != 0
                            {
                                if search_from_match_end {
                                    if nmatched > 1 as ::core::ffi::c_int {
                                        match_ok = false_0 != 0;
                                        break;
                                    } else {
                                        matchcol = endpos.col;
                                        if matchcol == matchpos.col
                                            && *ptr.offset(matchcol as isize) as ::core::ffi::c_int
                                                != NUL
                                        {
                                            matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                        }
                                    }
                                } else {
                                    matchcol = regmatch.rmm_matchcol;
                                    if *ptr.offset(matchcol as isize) as ::core::ffi::c_int != NUL {
                                        matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                    }
                                }
                                if matchcol == 0 as ::core::ffi::c_int
                                    && options & SEARCH_START as ::core::ffi::c_int != 0
                                {
                                    break;
                                }
                                if *ptr.offset(matchcol as isize) as ::core::ffi::c_int == NUL || {
                                    nmatched = vim_regexec_multi(
                                        &raw mut regmatch,
                                        win,
                                        buf,
                                        lnum,
                                        matchcol,
                                        tm,
                                        timed_out,
                                    );
                                    nmatched == 0 as ::core::ffi::c_int
                                } {
                                    match_ok = false_0 != 0;
                                    break;
                                } else {
                                    if regmatch.regprog.is_null() {
                                        break;
                                    }
                                    matchpos = regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                    endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                    submatch = first_submatch(&raw mut regmatch);
                                    if matchpos.lnum != 0 as linenr_T {
                                        break;
                                    }
                                    ptr = ml_get_buf(buf, lnum);
                                }
                            }
                            if !match_ok {
                                break 's_218;
                            }
                        }
                        if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                            match_ok = false_0 != 0;
                            while loop_0 != 0
                                || (if options & SEARCH_END as ::core::ffi::c_int != 0 {
                                    (lnum + regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                                        < start_pos.lnum
                                        || lnum
                                            + regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                .lnum
                                            == start_pos.lnum
                                            && (regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                .col
                                                - 1 as ::core::ffi::c_int)
                                                < start_pos.col + extra_col)
                                        as ::core::ffi::c_int
                                } else {
                                    (lnum
                                        + regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                                        < start_pos.lnum
                                        || lnum
                                            + regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .lnum
                                            == start_pos.lnum
                                            && regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .col
                                                < start_pos.col + extra_col)
                                        as ::core::ffi::c_int
                                }) != 0
                            {
                                match_ok = true_0 != 0;
                                matchpos = regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                submatch = first_submatch(&raw mut regmatch);
                                if search_from_match_end {
                                    if nmatched > 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                    matchcol = endpos.col;
                                    if matchcol == matchpos.col
                                        && *ptr.offset(matchcol as isize) as ::core::ffi::c_int
                                            != NUL
                                    {
                                        matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                    }
                                } else {
                                    if matchpos.lnum > 0 as linenr_T {
                                        break;
                                    }
                                    matchcol = matchpos.col;
                                    if *ptr.offset(matchcol as isize) as ::core::ffi::c_int != NUL {
                                        matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                    }
                                }
                                if *ptr.offset(matchcol as isize) as ::core::ffi::c_int == NUL || {
                                    nmatched = vim_regexec_multi(
                                        &raw mut regmatch,
                                        win,
                                        buf,
                                        lnum + matchpos.lnum,
                                        matchcol,
                                        tm,
                                        timed_out,
                                    );
                                    nmatched == 0 as ::core::ffi::c_int
                                } {
                                    if !tm.is_null()
                                        && profile_passed_limit(*tm) as ::core::ffi::c_int != 0
                                    {
                                        match_ok = false_0 != 0;
                                    }
                                    break;
                                } else {
                                    if regmatch.regprog.is_null() {
                                        break;
                                    }
                                    ptr = ml_get_buf(buf, lnum + matchpos.lnum);
                                }
                            }
                            if !match_ok {
                                break 's_218;
                            }
                        }
                        if options & SEARCH_END as ::core::ffi::c_int != 0
                            && options & SEARCH_NOOF as ::core::ffi::c_int == 0
                            && !(matchpos.lnum == endpos.lnum && matchpos.col == endpos.col)
                        {
                            (*pos).lnum = lnum + endpos.lnum;
                            (*pos).col = endpos.col;
                            if endpos.col == 0 as ::core::ffi::c_int {
                                if (*pos).lnum > 1 as linenr_T {
                                    (*pos).lnum -= 1;
                                    (*pos).col = ml_get_buf_len(buf, (*pos).lnum);
                                }
                            } else {
                                (*pos).col -= 1;
                                if (*pos).lnum <= (*buf).b_ml.ml_line_count {
                                    ptr = ml_get_buf(buf, (*pos).lnum);
                                    (*pos).col -=
                                        utf_head_off(ptr, ptr.offset((*pos).col as isize));
                                }
                            }
                            if !end_pos.is_null() {
                                (*end_pos).lnum = lnum + matchpos.lnum;
                                (*end_pos).col = matchpos.col;
                            }
                        } else {
                            (*pos).lnum = lnum + matchpos.lnum;
                            (*pos).col = matchpos.col;
                            if !end_pos.is_null() {
                                (*end_pos).lnum = lnum + endpos.lnum;
                                (*end_pos).col = endpos.col;
                            }
                        }
                        (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
                        if !end_pos.is_null() {
                            (*end_pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        found = 1 as ::core::ffi::c_int;
                        first_match = false_0 != 0;
                        search_match_lines.set(endpos.lnum - matchpos.lnum);
                        search_match_endcol.set(endpos.col);
                        break 's_704;
                    } else {
                        line_breakcheck();
                        if got_int.get() {
                            break 's_704;
                        }
                        if options & SEARCH_PEEK as ::core::ffi::c_int != 0
                            && lnum - (*pos).lnum & 0x3f as linenr_T == 0 as linenr_T
                            && char_avail() as ::core::ffi::c_int != 0
                        {
                            break_loop = true_0 != 0;
                            break 's_704;
                        } else if loop_0 != 0 && lnum == start_pos.lnum {
                            break 's_704;
                        }
                    }
                }
                lnum = (lnum as ::core::ffi::c_int + dir as ::core::ffi::c_int) as linenr_T;
                at_first_line = false_0;
            }
            at_first_line = false_0;
            if regmatch.regprog.is_null() {
                break;
            }
            if p_ws.get() == 0
                || stop_lnum != 0 as linenr_T
                || got_int.get() as ::core::ffi::c_int != 0
                || called_emsg.get() > called_emsg_before
                || !timed_out.is_null() && *timed_out != 0
                || break_loop as ::core::ffi::c_int != 0
                || found != 0
                || loop_0 != 0
            {
                break;
            }
            lnum = if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                (*buf).b_ml.ml_line_count
            } else {
                1 as linenr_T
            };
            if !shortmess(SHM_SEARCH as ::core::ffi::c_int)
                && shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && options & SEARCH_MSG as ::core::ffi::c_int != 0
            {
                give_warning(
                    gettext(
                        if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                            &raw const top_bot_msg as *const ::core::ffi::c_char
                        } else {
                            &raw const bot_top_msg as *const ::core::ffi::c_char
                        },
                    ),
                    true_0 != 0,
                    false_0 != 0,
                );
            }
            if !extra_arg.is_null() {
                (*extra_arg).sa_wrapped = true_0;
            }
            loop_0 += 1;
        }
        if got_int.get() as ::core::ffi::c_int != 0
            || called_emsg.get() > called_emsg_before
            || !timed_out.is_null() && *timed_out != 0
            || break_loop as ::core::ffi::c_int != 0
        {
            break;
        }
        count -= 1;
        if !(count > 0 as ::core::ffi::c_int && found != 0) {
            break;
        }
    }
    vim_regfree(regmatch.regprog);
    if found == 0 {
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        } else if options & SEARCH_MSG as ::core::ffi::c_int == SEARCH_MSG as ::core::ffi::c_int {
            if p_ws.get() != 0 {
                semsg(
                    gettext(&raw const e_patnotf2 as *const ::core::ffi::c_char),
                    mr_pattern.get(),
                );
            } else if lnum == 0 as linenr_T {
                semsg(
                    gettext(
                        (e_search_hit_top_without_match_for_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    mr_pattern.get(),
                );
            } else {
                semsg(
                    gettext(
                        (e_search_hit_bottom_without_match_for_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    mr_pattern.get(),
                );
            }
        }
        return FAIL;
    }
    if (*pos).lnum > (*buf).b_ml.ml_line_count {
        (*pos).lnum = (*buf).b_ml.ml_line_count;
        (*pos).col = ml_get_buf_len(buf, (*pos).lnum);
        if (*pos).col > 0 as ::core::ffi::c_int {
            (*pos).col -= 1;
        }
    }
    return submatch + 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_search_direction(mut cdir: ::core::ffi::c_int) {
    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = cdir as ::core::ffi::c_char;
}
unsafe extern "C" fn set_vv_searchforward() {
    set_vim_var_nr(
        VV_SEARCHFORWARD,
        ((*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T,
    );
}
unsafe extern "C" fn first_submatch(mut rp: *mut regmmatch_T) -> ::core::ffi::c_int {
    let mut submatch: ::core::ffi::c_int = 0;
    submatch = 1 as ::core::ffi::c_int;
    while (*rp).startpos[submatch as usize].lnum < 0 as linenr_T {
        if submatch == 9 as ::core::ffi::c_int {
            submatch = 0 as ::core::ffi::c_int;
            break;
        } else {
            submatch += 1;
        }
    }
    return submatch;
}
#[no_mangle]
pub unsafe extern "C" fn do_search(
    mut oap: *mut oparg_T,
    mut dirc: ::core::ffi::c_int,
    mut search_delim: ::core::ffi::c_int,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut count: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut sia: *mut searchit_arg_T,
) -> ::core::ffi::c_int {
    let mut searchstr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut searchstrlen: size_t = 0;
    let mut retval: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c: int64_t = 0;
    let mut dircp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut strcopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ps: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut msgbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut msgbuflen: size_t = 0 as size_t;
    let mut has_offset: bool = false_0 != 0;
    searchcmdlen.set(0 as ::core::ffi::c_int);
    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line as ::core::ffi::c_int != 0
        && !vim_strchr(p_cpo.get(), CPO_LINEOFF).is_null()
    {
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = false_0 != 0;
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 0 as int64_t;
    }
    let mut old_off: SearchOffset = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    if dirc == 0 as ::core::ffi::c_int {
        dirc = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir as uint8_t
            as ::core::ffi::c_int;
    } else {
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = dirc as ::core::ffi::c_char;
        set_vv_searchforward();
    }
    if options & SEARCH_REV as ::core::ffi::c_int != 0 {
        dirc = if dirc == '/' as ::core::ffi::c_int {
            '?' as ::core::ffi::c_int
        } else {
            '/' as ::core::ffi::c_int
        };
    }
    if dirc == '/' as ::core::ffi::c_int {
        if hasFolding(
            curwin.get(),
            pos.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut pos.lnum,
        ) {
            pos.col = (MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as colnr_T;
        }
    } else if hasFolding(
        curwin.get(),
        pos.lnum,
        &raw mut pos.lnum,
        ::core::ptr::null_mut::<linenr_T>(),
    ) {
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if no_hlsearch.get() as ::core::ffi::c_int != 0
        && options & SEARCH_KEEP as ::core::ffi::c_int == 0
    {
        redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
        set_no_hlsearch(false_0 != 0);
    }
    '_end_do_search: {
        loop {
            let mut show_top_bot_msg: bool = false_0 != 0;
            searchstr = pat;
            searchstrlen = patlen;
            dircp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if pat.is_null()
                || *pat as ::core::ffi::c_int == NUL
                || *pat as ::core::ffi::c_int == search_delim
            {
                if (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]
                    .pat
                    .is_null()
                {
                    if (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize]
                        .pat
                        .is_null()
                    {
                        emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                        retval = 0 as ::core::ffi::c_int;
                        break '_end_do_search;
                    } else {
                        searchstr = (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize].pat;
                        searchstrlen =
                            (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize].patlen;
                    }
                } else {
                    searchstr =
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                    searchstrlen = 0 as size_t;
                }
            }
            if !pat.is_null() && *pat as ::core::ffi::c_int != NUL {
                ps = strcopy;
                p = skip_regexp_ex(
                    pat,
                    search_delim,
                    magic_isset() as ::core::ffi::c_int,
                    &raw mut strcopy,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<magic_T>(),
                );
                if strcopy != ps {
                    let mut len: size_t = strlen(strcopy);
                    (*searchcmdlen.ptr()) += patlen.wrapping_sub(len) as ::core::ffi::c_int;
                    pat = strcopy;
                    patlen = len;
                    searchstr = strcopy;
                    searchstrlen = len;
                }
                if *p as ::core::ffi::c_int == search_delim {
                    searchstrlen = p.offset_from(pat) as size_t;
                    dircp = p;
                    let c2rust_fresh1 = p;
                    p = p.offset(1);
                    *c2rust_fresh1 = NUL as ::core::ffi::c_char;
                }
                (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = false_0 != 0;
                (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end = false_0 != 0;
                (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 0 as int64_t;
                if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = true_0 != 0;
                } else if options & SEARCH_OPT as ::core::ffi::c_int != 0
                    && (*p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == 'b' as ::core::ffi::c_int)
                {
                    if *p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end = true_0 != 0;
                    }
                    p = p.offset(1);
                }
                if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                {
                    if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || ascii_isdigit(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off =
                            atol(p) as int64_t;
                    } else if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = -1 as int64_t;
                    } else {
                        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 1 as int64_t;
                    }
                    p = p.offset(1);
                    while ascii_isdigit(*p as ::core::ffi::c_int) {
                        p = p.offset(1);
                    }
                }
                (*searchcmdlen.ptr()) += p.offset_from(pat) as ::core::ffi::c_int;
                patlen = patlen.wrapping_sub(p.offset_from(pat) as size_t);
                pat = p;
            }
            let mut show_search_stats: bool = false_0 != 0;
            if options & SEARCH_ECHO as ::core::ffi::c_int != 0
                && messaging() as ::core::ffi::c_int != 0
                && msg_silent.get() == 0
                && (!cmd_silent.get() || !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int))
            {
                let mut off_buf: [::core::ffi::c_char; 40] = [0; 40];
                let mut off_len: size_t = 0 as size_t;
                msg_start();
                msg_ext_set_kind(b"search_cmd\0".as_ptr() as *const ::core::ffi::c_char);
                if !cmd_silent.get()
                    && ((*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                        as ::core::ffi::c_int
                        != 0
                        || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end
                            as ::core::ffi::c_int
                            != 0
                        || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0)
                {
                    let c2rust_fresh2 = off_len;
                    off_len = off_len.wrapping_add(1);
                    off_buf[c2rust_fresh2 as usize] = dirc as ::core::ffi::c_char;
                    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end {
                        let c2rust_fresh3 = off_len;
                        off_len = off_len.wrapping_add(1);
                        off_buf[c2rust_fresh3 as usize] = 'e' as ::core::ffi::c_char;
                    } else if !(*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line {
                        let c2rust_fresh4 = off_len;
                        off_len = off_len.wrapping_add(1);
                        off_buf[c2rust_fresh4 as usize] = 's' as ::core::ffi::c_char;
                    }
                    off_buf[off_len as usize] = NUL as ::core::ffi::c_char;
                    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0 as int64_t
                        || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                            as ::core::ffi::c_int
                            != 0
                    {
                        off_len = off_len.wrapping_add(snprintf(
                            (&raw mut off_buf as *mut ::core::ffi::c_char).offset(off_len as isize),
                            ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                .wrapping_sub(off_len),
                            b"%+ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off,
                        ) as size_t);
                    }
                }
                let mut plen: size_t = 0;
                if *searchstr as ::core::ffi::c_int == NUL {
                    p = (*spats.ptr())[0 as ::core::ffi::c_int as usize].pat;
                    plen = (*spats.ptr())[0 as ::core::ffi::c_int as usize].patlen;
                } else {
                    p = searchstr;
                    plen = searchstrlen;
                }
                let mut msgbufsize: size_t = 0;
                if !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int)
                    || cmd_silent.get() as ::core::ffi::c_int != 0
                {
                    if ui_has(kUIMessages) {
                        msgbufsize = 0 as size_t;
                    } else if msg_scrolled.get() != 0 as ::core::ffi::c_int && !cmd_silent.get() {
                        msgbufsize = ((Rows.get() - msg_row.get()) * Columns.get()
                            - 1 as ::core::ffi::c_int)
                            as size_t;
                    } else {
                        msgbufsize =
                            ((Rows.get() - msg_row.get() - 1 as ::core::ffi::c_int) * Columns.get()
                                + sc_col.get()
                                - 1 as ::core::ffi::c_int) as size_t;
                    }
                    if msgbufsize
                        < plen
                            .wrapping_add(off_len)
                            .wrapping_add(SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t)
                            .wrapping_add(3 as size_t)
                    {
                        msgbufsize = plen
                            .wrapping_add(off_len)
                            .wrapping_add(SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t)
                            .wrapping_add(3 as size_t);
                    }
                } else {
                    msgbufsize = plen.wrapping_add(off_len).wrapping_add(3 as size_t);
                }
                xfree(msgbuf as *mut ::core::ffi::c_void);
                msgbuf = xmalloc(msgbufsize) as *mut ::core::ffi::c_char;
                memset(
                    msgbuf as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    msgbufsize,
                );
                msgbuflen = msgbufsize.wrapping_sub(1 as size_t);
                *msgbuf.offset(msgbuflen as isize) = NUL as ::core::ffi::c_char;
                if !cmd_silent.get() {
                    ui_busy_start();
                    *msgbuf.offset(0 as ::core::ffi::c_int as isize) = dirc as ::core::ffi::c_char;
                    if utf_iscomposing_first(utf_ptr2char(p)) {
                        *msgbuf.offset(1 as ::core::ffi::c_int as isize) =
                            ' ' as ::core::ffi::c_char;
                        memmove(
                            msgbuf.offset(2 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            plen,
                        );
                    } else {
                        memmove(
                            msgbuf.offset(1 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            plen,
                        );
                    }
                    if off_len > 0 as size_t {
                        memmove(
                            msgbuf
                                .offset(plen as isize)
                                .offset(1 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            &raw mut off_buf as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            off_len,
                        );
                    }
                    let mut trunc: *mut ::core::ffi::c_char = msg_strtrunc(msgbuf, true_0);
                    if !trunc.is_null() {
                        xfree(msgbuf as *mut ::core::ffi::c_void);
                        msgbuf = trunc;
                        msgbuflen = strlen(msgbuf);
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_rl != 0
                        && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int
                            == 's' as ::core::ffi::c_int
                    {
                        let mut r: *mut ::core::ffi::c_char = reverse_text(msgbuf);
                        xfree(msgbuf as *mut ::core::ffi::c_void);
                        msgbuf = r;
                        msgbuflen = strlen(msgbuf);
                        while *r as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                            r = r.offset(1);
                        }
                        let mut pat_len: size_t =
                            msgbuf.offset(msgbuflen as isize).offset_from(r) as size_t;
                        memmove(
                            msgbuf as *mut ::core::ffi::c_void,
                            r as *const ::core::ffi::c_void,
                            pat_len,
                        );
                        if r.offset_from(msgbuf) as size_t >= pat_len {
                            memset(
                                r as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                pat_len,
                            );
                        } else {
                            memset(
                                msgbuf.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                r.offset_from(msgbuf) as size_t,
                            );
                        }
                    }
                    msg_outtrans(msgbuf, 0 as ::core::ffi::c_int, false_0 != 0);
                    msg_clr_eos();
                    msg_check();
                    gotocmdline(false_0 != 0);
                    ui_flush();
                    ui_busy_stop();
                    msg_nowait.set(true_0 != 0);
                }
                if !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) {
                    show_search_stats = true_0 != 0;
                }
            }
            if !(*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                && (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0
                && pos.col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            {
                if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off > 0 as int64_t {
                    c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                    while c != 0 {
                        if decl(&raw mut pos) == -1 as ::core::ffi::c_int {
                            break;
                        }
                        c -= 1;
                    }
                    if c != 0 {
                        pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                        pos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                } else {
                    c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                    while c != 0 {
                        if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                            break;
                        }
                        c += 1;
                    }
                    if c != 0 {
                        pos.lnum = (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T;
                        pos.col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                }
            }
            c = searchit(
                curwin.get(),
                curbuf.get(),
                &raw mut pos,
                ::core::ptr::null_mut::<pos_T>(),
                (if dirc == '/' as ::core::ffi::c_int {
                    FORWARD as ::core::ffi::c_int
                } else {
                    BACKWARD as ::core::ffi::c_int
                }) as Direction,
                searchstr,
                searchstrlen,
                count,
                (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end as ::core::ffi::c_int
                    * SEARCH_END as ::core::ffi::c_int
                    + (options
                        & SEARCH_KEEP as ::core::ffi::c_int
                            + SEARCH_PEEK as ::core::ffi::c_int
                            + SEARCH_HIS as ::core::ffi::c_int
                            + SEARCH_MSG as ::core::ffi::c_int
                            + SEARCH_START as ::core::ffi::c_int
                            + (if !pat.is_null()
                                && *pat as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                            {
                                0 as ::core::ffi::c_int
                            } else {
                                SEARCH_NOOF as ::core::ffi::c_int
                            })),
                RE_LAST as ::core::ffi::c_int,
                sia,
            ) as int64_t;
            if !dircp.is_null() {
                *dircp = search_delim as ::core::ffi::c_char;
            }
            if !shortmess(SHM_SEARCH as ::core::ffi::c_int)
                && !sia.is_null()
                && (*sia).sa_wrapped != 0
            {
                show_top_bot_msg = true_0 != 0;
            }
            if c == FAIL as int64_t {
                retval = 0 as ::core::ffi::c_int;
                break '_end_do_search;
            } else {
                if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end as ::core::ffi::c_int
                    != 0
                    && !oap.is_null()
                {
                    (*oap).inclusive = true_0 != 0;
                }
                retval = 1 as ::core::ffi::c_int;
                if !sia.is_null() && (*sia).sa_wrapped != 0 {
                    apply_autocmds(
                        EVENT_SEARCHWRAPPED,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        ::core::ptr::null_mut::<buf_T>(),
                    );
                }
                if options & SEARCH_NOOF as ::core::ffi::c_int == 0
                    || !pat.is_null() && *pat as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                {
                    let mut org_pos: pos_T = pos;
                    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line {
                        c = pos.lnum as int64_t
                            + (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                        if c < 1 as int64_t {
                            pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                        } else if c > (*curbuf.get()).b_ml.ml_line_count as int64_t {
                            pos.lnum = (*curbuf.get()).b_ml.ml_line_count;
                        } else {
                            pos.lnum = c as linenr_T;
                        }
                        pos.col = 0 as ::core::ffi::c_int as colnr_T;
                        retval = 2 as ::core::ffi::c_int;
                    } else if pos.col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int {
                        c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                        if c > 0 as int64_t {
                            loop {
                                let c2rust_fresh5 = c;
                                c = c - 1;
                                if c2rust_fresh5 <= 0 as int64_t {
                                    break;
                                }
                                if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                    break;
                                }
                            }
                        } else {
                            loop {
                                let c2rust_fresh6 = c;
                                c = c + 1;
                                if c2rust_fresh6 >= 0 as int64_t {
                                    break;
                                }
                                if decl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                    break;
                                }
                            }
                        }
                    }
                    if !equalpos(pos, org_pos) {
                        has_offset = true_0 != 0;
                    }
                }
                if show_search_stats {
                    cmdline_search_stat(
                        dirc,
                        &raw mut pos,
                        &raw mut (*curwin.get()).w_cursor,
                        show_top_bot_msg,
                        msgbuf,
                        msgbuflen,
                        count != 1 as ::core::ffi::c_int
                            || has_offset as ::core::ffi::c_int != 0
                            || fdo_flags.get()
                                & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint
                                == 0
                                && hasFolding(
                                    curwin.get(),
                                    (*curwin.get()).w_cursor.lnum,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    ::core::ptr::null_mut::<linenr_T>(),
                                ) as ::core::ffi::c_int
                                    != 0,
                        p_msc.get() as ::core::ffi::c_int,
                        SEARCH_STAT_DEF_TIMEOUT as ::core::ffi::c_int,
                    );
                }
                if options & SEARCH_OPT as ::core::ffi::c_int == 0
                    || pat.is_null()
                    || *pat as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                {
                    break;
                }
                pat = pat.offset(1);
                dirc = *pat as uint8_t as ::core::ffi::c_int;
                search_delim = dirc;
                if dirc != '?' as ::core::ffi::c_int && dirc != '/' as ::core::ffi::c_int {
                    retval = 0 as ::core::ffi::c_int;
                    emsg(gettext(b"E386: Expected '?' or '/'  after ';'\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    break '_end_do_search;
                } else {
                    pat = pat.offset(1);
                    patlen = patlen.wrapping_sub(1);
                }
            }
        }
        if options & SEARCH_MARK as ::core::ffi::c_int != 0 {
            setpcmark();
        }
        (*curwin.get()).w_cursor = pos;
        (*curwin.get()).w_set_curswant = true_0;
    }
    if options & SEARCH_KEEP as ::core::ffi::c_int != 0
        || (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0
    {
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off = old_off;
    }
    xfree(strcopy as *mut ::core::ffi::c_void);
    xfree(msgbuf as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn search_for_exact_line(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut dir: Direction,
    mut pat: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut start: linenr_T = 0 as linenr_T;
    let mut compl_len: ::core::ffi::c_int = ins_compl_len();
    if (*buf).b_ml.ml_line_count == 0 as linenr_T {
        return FAIL;
    }
    loop {
        (*pos).lnum = ((*pos).lnum as ::core::ffi::c_int + dir as ::core::ffi::c_int) as linenr_T;
        if (*pos).lnum < 1 as linenr_T {
            if p_ws.get() != 0 {
                (*pos).lnum = (*buf).b_ml.ml_line_count;
                if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                    give_warning(
                        gettext(&raw const top_bot_msg as *const ::core::ffi::c_char),
                        true_0 != 0,
                        false_0 != 0,
                    );
                }
            } else {
                (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                break;
            }
        } else if (*pos).lnum > (*buf).b_ml.ml_line_count {
            if p_ws.get() != 0 {
                (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                    give_warning(
                        gettext(&raw const bot_top_msg as *const ::core::ffi::c_char),
                        true_0 != 0,
                        false_0 != 0,
                    );
                }
            } else {
                (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                break;
            }
        }
        if (*pos).lnum == start {
            break;
        }
        if start == 0 as linenr_T {
            start = (*pos).lnum;
        }
        let mut ptr: *mut ::core::ffi::c_char = ml_get_buf(buf, (*pos).lnum);
        let mut p: *mut ::core::ffi::c_char = skipwhite(ptr);
        (*pos).col = p.offset_from(ptr) as colnr_T;
        if compl_status_adding() as ::core::ffi::c_int != 0 && !compl_status_sol() {
            if mb_strcmp_ic(p_ic.get() != 0, p, pat) == 0 as ::core::ffi::c_int {
                return OK;
            }
        } else if *p as ::core::ffi::c_int != NUL {
            '_c2rust_label: {
                if compl_len >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"compl_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/search.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1519 as ::core::ffi::c_uint,
                        b"int search_for_exact_line(buf_T *, pos_T *, Direction, char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if (if p_ic.get() != 0 {
                mb_strnicmp(p, pat, compl_len as size_t)
            } else {
                strncmp(p, pat, compl_len as size_t)
            }) == 0 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn searchc(mut cap: *mut cmdarg_T, mut t_cmd: bool) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = (*cap).nchar;
    let mut dir: ::core::ffi::c_int = (*cap).arg;
    let mut count: ::core::ffi::c_int = (*cap).count1;
    let mut stop: bool = true_0 != 0;
    if c != NUL {
        if KeyStuffed.get() == 0 {
            *(lastc.ptr() as *mut uint8_t) = c as uint8_t;
            set_csearch_direction(dir as Direction);
            set_csearch_until(t_cmd as ::core::ffi::c_int);
            if (*cap).nchar_len != 0 {
                lastc_bytelen.set((*cap).nchar_len);
                memcpy(
                    lastc_bytes.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    &raw mut (*cap).nchar_composing as *mut ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    (*cap).nchar_len as size_t,
                );
            } else {
                lastc_bytelen.set(utf_char2bytes(
                    c,
                    lastc_bytes.ptr() as *mut ::core::ffi::c_char,
                ));
            }
        }
    } else {
        if *(lastc.ptr() as *mut uint8_t) as ::core::ffi::c_int == NUL
            && lastc_bytelen.get() <= 1 as ::core::ffi::c_int
        {
            return FAIL;
        }
        dir = if dir != 0 {
            -(lastcdir.get() as ::core::ffi::c_int)
        } else {
            lastcdir.get() as ::core::ffi::c_int
        };
        t_cmd = last_t_cmd.get();
        c = *(lastc.ptr() as *mut uint8_t) as ::core::ffi::c_int;
        if vim_strchr(p_cpo.get(), CPO_SCOLON).is_null()
            && count == 1 as ::core::ffi::c_int
            && t_cmd as ::core::ffi::c_int != 0
        {
            stop = false_0 != 0;
        }
    }
    (*(*cap).oap).inclusive = dir != BACKWARD as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = get_cursor_line_len();
    loop {
        let c2rust_fresh7 = count;
        count = count - 1;
        if c2rust_fresh7 == 0 {
            break;
        }
        loop {
            if dir > 0 as ::core::ffi::c_int {
                col += utfc_ptr2len(p.offset(col as isize));
                if col >= len {
                    return FAIL;
                }
            } else {
                if col == 0 as ::core::ffi::c_int {
                    return FAIL;
                }
                col -= utf_head_off(
                    p,
                    p.offset(col as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize)),
                ) + 1 as ::core::ffi::c_int;
            }
            if lastc_bytelen.get() <= 1 as ::core::ffi::c_int {
                if *p.offset(col as isize) as ::core::ffi::c_int == c
                    && stop as ::core::ffi::c_int != 0
                {
                    break;
                }
            } else if strncmp(
                p.offset(col as isize),
                lastc_bytes.ptr() as *mut ::core::ffi::c_char,
                lastc_bytelen.get() as size_t,
            ) == 0 as ::core::ffi::c_int
                && stop as ::core::ffi::c_int != 0
            {
                break;
            }
            stop = true_0 != 0;
        }
    }
    if t_cmd {
        col -= dir;
        if dir < 0 as ::core::ffi::c_int {
            col += lastc_bytelen.get() - 1 as ::core::ffi::c_int;
        } else {
            col -= utf_head_off(p, p.offset(col as isize));
        }
    }
    (*curwin.get()).w_cursor.col = col as colnr_T;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn findmatch(
    mut oap: *mut oparg_T,
    mut initc: ::core::ffi::c_int,
) -> *mut pos_T {
    return findmatchlimit(oap, initc, 0 as ::core::ffi::c_int, 0 as int64_t);
}
unsafe extern "C" fn check_prevcol(
    mut linep: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut ch: ::core::ffi::c_int,
    mut prevcol: *mut ::core::ffi::c_int,
) -> bool {
    col -= 1;
    if col > 0 as ::core::ffi::c_int {
        col -= utf_head_off(linep, linep.offset(col as isize));
    }
    if !prevcol.is_null() {
        *prevcol = col;
    }
    return col >= 0 as ::core::ffi::c_int
        && *linep.offset(col as isize) as uint8_t as ::core::ffi::c_int == ch;
}
unsafe extern "C" fn find_rawstring_end(
    mut linep: *mut ::core::ffi::c_char,
    mut startpos: *mut pos_T,
    mut endpos: *mut pos_T,
) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum: linenr_T = 0;
    p = linep
        .offset((*startpos).col as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    let mut delim_len: size_t =
        (p.offset_from(linep) - (*startpos).col as isize - 1 as isize) as size_t;
    let mut delim_copy: *mut ::core::ffi::c_char = xmemdupz(
        linep
            .offset((*startpos).col as isize)
            .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        delim_len,
    ) as *mut ::core::ffi::c_char;
    let mut found: bool = false_0 != 0;
    lnum = (*startpos).lnum;
    while lnum <= (*endpos).lnum {
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        p = line.offset(
            (if lnum == (*startpos).lnum {
                (*startpos).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as isize,
        );
        while *p != 0 {
            if lnum == (*endpos).lnum && p.offset_from(line) as colnr_T >= (*endpos).col {
                break;
            }
            if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int
                && strncmp(
                    delim_copy,
                    p.offset(1 as ::core::ffi::c_int as isize),
                    delim_len,
                ) == 0 as ::core::ffi::c_int
                && *p.offset(delim_len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
            {
                found = true_0 != 0;
                break;
            } else {
                p = p.offset(1);
            }
        }
        if found {
            break;
        }
        lnum += 1;
    }
    xfree(delim_copy as *mut ::core::ffi::c_void);
    return found;
}
unsafe extern "C" fn find_mps_values(
    mut initc: *mut ::core::ffi::c_int,
    mut findc: *mut ::core::ffi::c_int,
    mut backwards: *mut bool,
    mut switchit: bool,
) {
    let mut ptr: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_mps;
    while *ptr as ::core::ffi::c_int != NUL {
        if utf_ptr2char(ptr) == *initc {
            if switchit {
                *findc = *initc;
                *initc = utf_ptr2char(
                    ptr.offset(utfc_ptr2len(ptr) as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                );
                *backwards = true_0 != 0;
            } else {
                *findc = utf_ptr2char(
                    ptr.offset(utfc_ptr2len(ptr) as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                );
                *backwards = false_0 != 0;
            }
            return;
        }
        let mut prev: *mut ::core::ffi::c_char = ptr;
        ptr = ptr.offset((utfc_ptr2len(ptr) + 1 as ::core::ffi::c_int) as isize);
        if utf_ptr2char(ptr) == *initc {
            if switchit {
                *findc = *initc;
                *initc = utf_ptr2char(prev);
                *backwards = false_0 != 0;
            } else {
                *findc = utf_ptr2char(prev);
                *backwards = true_0 != 0;
            }
            return;
        }
        ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
        if *ptr as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            ptr = ptr.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn findmatchlimit(
    mut oap: *mut oparg_T,
    mut initc: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut maxtravel: int64_t,
) -> *mut pos_T {
    static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut findc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut backwards: bool = false_0 != 0;
    let mut raw_string: bool = false_0 != 0;
    let mut inquote: bool = false_0 != 0;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hash_dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut comment_dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut traveled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ignore_cend: bool = false_0 != 0;
    let mut match_escaped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dir: ::core::ffi::c_int = 0;
    let mut comment_col: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
    let mut lispcomm: bool = false_0 != 0;
    let mut lisp: bool = (*curbuf.get()).b_p_lisp != 0;
    pos.set((*curwin.get()).w_cursor);
    (*pos.ptr()).coladd = 0 as ::core::ffi::c_int as colnr_T;
    let mut linep: *mut ::core::ffi::c_char = ml_get((*pos.ptr()).lnum);
    let mut cpo_match: bool = !vim_strchr(p_cpo.get(), CPO_MATCH).is_null();
    let mut cpo_bsl: bool = !vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null();
    if flags & FM_BACKWARD as ::core::ffi::c_int != 0 {
        dir = BACKWARD as ::core::ffi::c_int;
    } else if flags & FM_FORWARD as ::core::ffi::c_int != 0 {
        dir = FORWARD as ::core::ffi::c_int;
    } else {
        dir = 0 as ::core::ffi::c_int;
    }
    if initc == '/' as ::core::ffi::c_int
        || initc == '*' as ::core::ffi::c_int
        || initc == 'R' as ::core::ffi::c_int
    {
        comment_dir = dir;
        if initc == '/' as ::core::ffi::c_int {
            ignore_cend = true_0 != 0;
        }
        backwards = if dir == FORWARD as ::core::ffi::c_int {
            false_0
        } else {
            true_0
        } != 0;
        raw_string = initc == 'R' as ::core::ffi::c_int;
        initc = NUL;
    } else if initc != '#' as ::core::ffi::c_int && initc != NUL {
        find_mps_values(
            &raw mut initc,
            &raw mut findc,
            &raw mut backwards,
            true_0 != 0,
        );
        if dir != 0 {
            backwards = if dir == FORWARD as ::core::ffi::c_int {
                false_0
            } else {
                true_0
            } != 0;
        }
        if findc == NUL {
            return ::core::ptr::null_mut::<pos_T>();
        }
    } else {
        if initc == '#' as ::core::ffi::c_int {
            hash_dir = dir;
        } else {
            if !cpo_match {
                ptr = skipwhite(linep);
                if *ptr as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    && (*pos.ptr()).col <= ptr.offset_from(linep) as colnr_T
                {
                    ptr = skipwhite(ptr.offset(1 as ::core::ffi::c_int as isize));
                    if strncmp(
                        ptr,
                        b"if\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            ptr,
                            b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            ptr,
                            b"el\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        hash_dir = 1 as ::core::ffi::c_int;
                    }
                } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                {
                    if *linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                    {
                        comment_dir = FORWARD as ::core::ffi::c_int;
                        backwards = false_0 != 0;
                        (*pos.ptr()).col += 1;
                    } else if (*pos.ptr()).col > 0 as ::core::ffi::c_int
                        && *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                    {
                        comment_dir = BACKWARD as ::core::ffi::c_int;
                        backwards = true_0 != 0;
                        (*pos.ptr()).col -= 1;
                    }
                } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                {
                    if *linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                    {
                        comment_dir = BACKWARD as ::core::ffi::c_int;
                        backwards = true_0 != 0;
                    } else if (*pos.ptr()).col > 0 as ::core::ffi::c_int
                        && *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                    {
                        comment_dir = FORWARD as ::core::ffi::c_int;
                        backwards = false_0 != 0;
                    }
                }
            }
            if hash_dir == 0 && comment_dir == 0 {
                if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int == NUL
                    && (*pos.ptr()).col != 0
                {
                    (*pos.ptr()).col -= 1;
                }
                loop {
                    initc = utf_ptr2char(linep.offset((*pos.ptr()).col as isize));
                    if initc == NUL {
                        break;
                    }
                    find_mps_values(
                        &raw mut initc,
                        &raw mut findc,
                        &raw mut backwards,
                        false_0 != 0,
                    );
                    if findc != 0 {
                        break;
                    }
                    (*pos.ptr()).col += utfc_ptr2len(linep.offset((*pos.ptr()).col as isize));
                }
                if findc == 0 {
                    if !cpo_match
                        && *skipwhite(linep) as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    {
                        hash_dir = 1 as ::core::ffi::c_int;
                    } else {
                        return ::core::ptr::null_mut::<pos_T>();
                    }
                } else if !cpo_bsl {
                    let mut bslcnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut col: ::core::ffi::c_int = (*pos.ptr()).col as ::core::ffi::c_int;
                    while check_prevcol(linep, col, '\\' as ::core::ffi::c_int, &raw mut col) {
                        bslcnt += 1;
                    }
                    match_escaped = bslcnt & 1 as ::core::ffi::c_int;
                }
            }
        }
        if hash_dir != 0 {
            if !oap.is_null() {
                (*oap).motion_type = kMTLineWise;
            }
            if initc != '#' as ::core::ffi::c_int {
                ptr = skipwhite(skipwhite(linep).offset(1 as ::core::ffi::c_int as isize));
                if strncmp(
                    ptr,
                    b"if\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        ptr,
                        b"el\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    hash_dir = 1 as ::core::ffi::c_int;
                } else if strncmp(
                    ptr,
                    b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    hash_dir = -1 as ::core::ffi::c_int;
                } else {
                    return ::core::ptr::null_mut::<pos_T>();
                }
            }
            (*pos.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
            while !got_int.get() {
                if hash_dir > 0 as ::core::ffi::c_int {
                    if (*pos.ptr()).lnum == (*curbuf.get()).b_ml.ml_line_count {
                        break;
                    }
                } else if (*pos.ptr()).lnum == 1 as linenr_T {
                    break;
                }
                (*pos.ptr()).lnum =
                    ((*pos.ptr()).lnum as ::core::ffi::c_int + hash_dir) as linenr_T;
                linep = ml_get((*pos.ptr()).lnum);
                line_breakcheck();
                ptr = skipwhite(linep);
                if *ptr as ::core::ffi::c_int != '#' as ::core::ffi::c_int {
                    continue;
                }
                (*pos.ptr()).col = ptr.offset_from(linep) as colnr_T;
                ptr = skipwhite(ptr.offset(1 as ::core::ffi::c_int as isize));
                if hash_dir > 0 as ::core::ffi::c_int {
                    if strncmp(
                        ptr,
                        b"if\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        count += 1;
                    } else if strncmp(
                        ptr,
                        b"el\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if count == 0 as ::core::ffi::c_int {
                            return pos.ptr();
                        }
                    } else if strncmp(
                        ptr,
                        b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if count == 0 as ::core::ffi::c_int {
                            return pos.ptr();
                        }
                        count -= 1;
                    }
                } else if strncmp(
                    ptr,
                    b"if\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    if count == 0 as ::core::ffi::c_int {
                        return pos.ptr();
                    }
                    count -= 1;
                } else if initc == '#' as ::core::ffi::c_int
                    && strncmp(
                        ptr,
                        b"el\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    if count == 0 as ::core::ffi::c_int {
                        return pos.ptr();
                    }
                } else if strncmp(
                    ptr,
                    b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    count += 1;
                }
            }
            return ::core::ptr::null_mut::<pos_T>();
        }
    }
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0
        && !vim_strchr(b"()[]{}<>\0".as_ptr() as *const ::core::ffi::c_char, initc).is_null()
    {
        backwards = !backwards;
    }
    let mut do_quotes: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut at_start: ::core::ffi::c_int = 0;
    let mut start_in_quotes: TriState = kNone;
    let mut match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    clearpos(&raw mut match_pos);
    if backwards as ::core::ffi::c_int != 0 && comment_dir != 0 || lisp as ::core::ffi::c_int != 0 {
        comment_col = check_linecomment(linep);
    }
    if lisp as ::core::ffi::c_int != 0
        && comment_col != MAXCOL as ::core::ffi::c_int
        && (*pos.ptr()).col > comment_col
    {
        lispcomm = true_0 != 0;
    }
    while !got_int.get() {
        if backwards {
            if lispcomm as ::core::ffi::c_int != 0 && (*pos.ptr()).col < comment_col {
                break;
            }
            if (*pos.ptr()).col == 0 as ::core::ffi::c_int {
                if (*pos.ptr()).lnum == 1 as linenr_T {
                    break;
                }
                (*pos.ptr()).lnum -= 1;
                if maxtravel > 0 as int64_t && {
                    traveled += 1;
                    traveled as int64_t > maxtravel
                } {
                    break;
                }
                linep = ml_get((*pos.ptr()).lnum);
                (*pos.ptr()).col = ml_get_len((*pos.ptr()).lnum);
                do_quotes = -1 as ::core::ffi::c_int;
                line_breakcheck();
                if comment_dir != 0 || lisp as ::core::ffi::c_int != 0 {
                    comment_col = check_linecomment(linep);
                }
                if lisp as ::core::ffi::c_int != 0 && comment_col != MAXCOL as ::core::ffi::c_int {
                    (*pos.ptr()).col = comment_col as colnr_T;
                }
            } else {
                (*pos.ptr()).col -= 1;
                (*pos.ptr()).col -= utf_head_off(linep, linep.offset((*pos.ptr()).col as isize));
            }
        } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int == NUL
            || lisp as ::core::ffi::c_int != 0
                && comment_col != MAXCOL as ::core::ffi::c_int
                && (*pos.ptr()).col == comment_col
        {
            if (*pos.ptr()).lnum == (*curbuf.get()).b_ml.ml_line_count
                || lispcomm as ::core::ffi::c_int != 0
            {
                break;
            }
            (*pos.ptr()).lnum += 1;
            if maxtravel != 0 && {
                let c2rust_fresh8 = traveled;
                traveled = traveled + 1;
                c2rust_fresh8 as int64_t > maxtravel
            } {
                break;
            }
            linep = ml_get((*pos.ptr()).lnum);
            (*pos.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
            do_quotes = -1 as ::core::ffi::c_int;
            line_breakcheck();
            if lisp {
                comment_col = check_linecomment(linep);
            }
        } else {
            (*pos.ptr()).col += utfc_ptr2len(linep.offset((*pos.ptr()).col as isize));
        }
        if (*pos.ptr()).col == 0 as ::core::ffi::c_int
            && flags & FM_BLOCKSTOP as ::core::ffi::c_int != 0
            && (*linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '{' as ::core::ffi::c_int
                || *linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '}' as ::core::ffi::c_int)
        {
            if *linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == findc
                && count == 0 as ::core::ffi::c_int
            {
                return pos.ptr();
            }
            break;
        } else if comment_dir != 0 {
            if comment_dir == FORWARD as ::core::ffi::c_int {
                if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                    && *linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                {
                    (*pos.ptr()).col += 1;
                    return pos.ptr();
                }
            } else {
                if (*pos.ptr()).col == 0 as ::core::ffi::c_int {
                    continue;
                }
                if raw_string {
                    if *linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == 'R' as ::core::ffi::c_int
                        && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int
                        && !vim_strchr(
                            linep
                                .offset((*pos.ptr()).col as isize)
                                .offset(1 as ::core::ffi::c_int as isize),
                            '(' as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        if !find_rawstring_end(
                            linep,
                            pos.ptr(),
                            if count > 0 as ::core::ffi::c_int {
                                &raw mut match_pos
                            } else {
                                &raw mut (*curwin.get()).w_cursor
                            },
                        ) {
                            count += 1;
                            match_pos = pos.get();
                            match_pos.col -= 1;
                        }
                        linep = ml_get((*pos.ptr()).lnum);
                    }
                } else if *linep.offset(
                    ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                    && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                    && ((*pos.ptr()).col == 1 as ::core::ffi::c_int
                        || *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            != '*' as ::core::ffi::c_int)
                    && (*pos.ptr()).col < comment_col
                {
                    count += 1;
                    match_pos = pos.get();
                    match_pos.col -= 1;
                } else {
                    if !(*linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                        && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    if count > 0 as ::core::ffi::c_int {
                        pos.set(match_pos);
                    } else if (*pos.ptr()).col > 1 as ::core::ffi::c_int
                        && *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                        && (*pos.ptr()).col <= comment_col
                    {
                        (*pos.ptr()).col -= 2 as ::core::ffi::c_int;
                    } else {
                        if ignore_cend {
                            continue;
                        }
                        return ::core::ptr::null_mut::<pos_T>();
                    }
                    return pos.ptr();
                }
            }
        } else {
            if cpo_match {
                do_quotes = 0 as ::core::ffi::c_int;
            } else if do_quotes == -1 as ::core::ffi::c_int {
                at_start = do_quotes;
                ptr = linep;
                while *ptr != 0 {
                    if ptr
                        == linep
                            .offset((*pos.ptr()).col as isize)
                            .offset(backwards as ::core::ffi::c_int as isize)
                    {
                        at_start = do_quotes & 1 as ::core::ffi::c_int;
                    }
                    if *ptr as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                        && (ptr == linep
                            || *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '\'' as ::core::ffi::c_int
                            || *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '\'' as ::core::ffi::c_int)
                    {
                        do_quotes += 1;
                    }
                    if *ptr as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        ptr = ptr.offset(1);
                    }
                    ptr = ptr.offset(1);
                }
                do_quotes &= 1 as ::core::ffi::c_int;
                if do_quotes == 0 {
                    inquote = false_0 != 0;
                    if *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    {
                        do_quotes = 1 as ::core::ffi::c_int;
                        if start_in_quotes as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                            inquote = true_0 != 0;
                            start_in_quotes = kTrue;
                        } else if backwards {
                            inquote = true_0 != 0;
                        }
                    }
                    if (*pos.ptr()).lnum > 1 as linenr_T {
                        ptr = ml_get((*pos.ptr()).lnum - 1 as linenr_T);
                        if *ptr as ::core::ffi::c_int != 0
                            && *ptr
                                .offset(ml_get_len((*pos.ptr()).lnum - 1 as linenr_T) as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize))
                                as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                        {
                            do_quotes = 1 as ::core::ffi::c_int;
                            if start_in_quotes as ::core::ffi::c_int == kNone as ::core::ffi::c_int
                            {
                                inquote = at_start != 0;
                                if inquote {
                                    start_in_quotes = kTrue;
                                }
                            } else if !backwards {
                                inquote = true_0 != 0;
                            }
                        }
                        linep = ml_get((*pos.ptr()).lnum);
                    }
                }
            }
            if start_in_quotes as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                start_in_quotes = kFalse;
            }
            let c: ::core::ffi::c_int = utf_ptr2char(linep.offset((*pos.ptr()).col as isize));
            's_1456: {
                match c {
                    NUL => {
                        if (*pos.ptr()).col == 0 as ::core::ffi::c_int
                            || *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                                != '\\' as ::core::ffi::c_int
                        {
                            inquote = false_0 != 0;
                            start_in_quotes = kFalse;
                        }
                        break 's_1456;
                    }
                    34 => {
                        if do_quotes != 0 {
                            let mut col_0: ::core::ffi::c_int = 0;
                            col_0 =
                                (*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                            while col_0 >= 0 as ::core::ffi::c_int {
                                if *linep.offset(col_0 as isize) as ::core::ffi::c_int
                                    != '\\' as ::core::ffi::c_int
                                {
                                    break;
                                }
                                col_0 -= 1;
                            }
                            if (*pos.ptr()).col - 1 as ::core::ffi::c_int - col_0
                                & 1 as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                inquote = !inquote;
                                start_in_quotes = kFalse;
                            }
                        }
                        break 's_1456;
                    }
                    39 => {
                        if !cpo_match
                            && initc != '\'' as ::core::ffi::c_int
                            && findc != '\'' as ::core::ffi::c_int
                        {
                            if backwards {
                                if (*pos.ptr()).col > 1 as ::core::ffi::c_int {
                                    if *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            - 2 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int
                                        == '\'' as ::core::ffi::c_int
                                    {
                                        (*pos.ptr()).col -= 2 as ::core::ffi::c_int;
                                        break 's_1456;
                                    } else if *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            - 2 as ::core::ffi::c_int)
                                            as isize,
                                    )
                                        as ::core::ffi::c_int
                                        == '\\' as ::core::ffi::c_int
                                        && (*pos.ptr()).col > 2 as ::core::ffi::c_int
                                        && *linep.offset(
                                            ((*pos.ptr()).col as ::core::ffi::c_int
                                                - 3 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            == '\'' as ::core::ffi::c_int
                                    {
                                        (*pos.ptr()).col -= 3 as ::core::ffi::c_int;
                                        break 's_1456;
                                    }
                                }
                            } else if *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                    as isize,
                            ) != 0
                            {
                                if *linep.offset(
                                    ((*pos.ptr()).col as ::core::ffi::c_int
                                        + 1 as ::core::ffi::c_int)
                                        as isize,
                                ) as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                                    && *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            + 2 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int
                                        != 0
                                    && *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            + 3 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int
                                        == '\'' as ::core::ffi::c_int
                                {
                                    (*pos.ptr()).col += 3 as ::core::ffi::c_int;
                                    break 's_1456;
                                } else if *linep.offset(
                                    ((*pos.ptr()).col as ::core::ffi::c_int
                                        + 2 as ::core::ffi::c_int)
                                        as isize,
                                ) as ::core::ffi::c_int
                                    == '\'' as ::core::ffi::c_int
                                {
                                    (*pos.ptr()).col += 2 as ::core::ffi::c_int;
                                    break 's_1456;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                if !((*curbuf.get()).b_p_lisp != 0
                    && !vim_strchr(b"(){}[]\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                    && (*pos.ptr()).col > 1 as ::core::ffi::c_int
                    && check_prevcol(
                        linep,
                        (*pos.ptr()).col as ::core::ffi::c_int,
                        '\\' as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ) as ::core::ffi::c_int
                        != 0
                    && check_prevcol(
                        linep,
                        (*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        '#' as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ) as ::core::ffi::c_int
                        != 0)
                {
                    if (!inquote
                        || start_in_quotes as ::core::ffi::c_int == kTrue as ::core::ffi::c_int)
                        && (c == initc || c == findc)
                    {
                        let mut bslcnt_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if !cpo_bsl {
                            let mut col_1: ::core::ffi::c_int =
                                (*pos.ptr()).col as ::core::ffi::c_int;
                            while check_prevcol(
                                linep,
                                col_1,
                                '\\' as ::core::ffi::c_int,
                                &raw mut col_1,
                            ) {
                                bslcnt_0 += 1;
                            }
                        }
                        if cpo_bsl as ::core::ffi::c_int != 0
                            || bslcnt_0 & 1 as ::core::ffi::c_int == match_escaped
                        {
                            if c == initc {
                                count += 1;
                            } else {
                                if count == 0 as ::core::ffi::c_int {
                                    return pos.ptr();
                                }
                                count -= 1;
                            }
                        }
                    }
                }
            }
        }
    }
    if comment_dir == BACKWARD as ::core::ffi::c_int && count > 0 as ::core::ffi::c_int {
        pos.set(match_pos);
        return pos.ptr();
    }
    return NULL_0 as *mut pos_T;
}
#[no_mangle]
pub unsafe extern "C" fn check_linecomment(
    mut line: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = line;
    if (*curbuf.get()).b_p_lisp != 0 {
        if !vim_strchr(p, ';' as ::core::ffi::c_int).is_null() {
            let mut in_str: bool = false_0 != 0;
            loop {
                p = strpbrk(p, b"\";\0".as_ptr() as *const ::core::ffi::c_char);
                if p.is_null() {
                    break;
                }
                if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    if in_str {
                        if *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                            != '\\' as ::core::ffi::c_int
                        {
                            in_str = false_0 != 0;
                        }
                    } else if p == line
                        || p.offset_from(line) >= 2 as isize
                            && *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                                != '\\' as ::core::ffi::c_int
                            && *p.offset(-(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                                != '#' as ::core::ffi::c_int
                    {
                        in_str = true_0 != 0;
                    }
                } else if !in_str
                    && (p.offset_from(line) < 2 as isize
                        || *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                            != '\\' as ::core::ffi::c_int
                            && *p.offset(-(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                                != '#' as ::core::ffi::c_int)
                    && is_pos_in_string(line, p.offset_from(line) as colnr_T) == 0
                {
                    break;
                }
                p = p.offset(1);
            }
        } else {
            p = ::core::ptr::null::<::core::ffi::c_char>();
        }
    } else {
        loop {
            p = vim_strchr(p, '/' as ::core::ffi::c_int);
            if p.is_null() {
                break;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int
                && (p == line
                    || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '*' as ::core::ffi::c_int
                    || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '*' as ::core::ffi::c_int)
                && is_pos_in_string(line, p.offset_from(line) as colnr_T) == 0
            {
                break;
            }
            p = p.offset(1);
        }
    }
    if p.is_null() {
        return MAXCOL as ::core::ffi::c_int;
    }
    return p.offset_from(line) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn showmatch(mut c: ::core::ffi::c_int) {
    let mut lpos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut vcol: colnr_T = 0;
    let mut so: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_so >= 0 as OptInt {
        &raw mut (*curwin.get()).w_onebuf_opt.wo_so
    } else {
        p_so.ptr()
    };
    let mut siso: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_siso >= 0 as OptInt {
        &raw mut (*curwin.get()).w_onebuf_opt.wo_siso
    } else {
        p_siso.ptr()
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = (*curbuf.get()).b_p_mps;
    while *p as ::core::ffi::c_int != NUL {
        if utf_ptr2char(p) == c && (*curwin.get()).w_onebuf_opt.wo_rl ^ p_ri.get() != 0 {
            break;
        }
        p = p.offset((utfc_ptr2len(p) + 1 as ::core::ffi::c_int) as isize);
        if utf_ptr2char(p) == c && (*curwin.get()).w_onebuf_opt.wo_rl ^ p_ri.get() == 0 {
            break;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
        if *p as ::core::ffi::c_int == NUL {
            return;
        }
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    lpos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
    if lpos.is_null() {
        vim_beep(kOptBoFlagShowmatch as ::core::ffi::c_int as ::core::ffi::c_uint);
        return;
    }
    if (*lpos).lnum < (*curwin.get()).w_topline || (*lpos).lnum >= (*curwin.get()).w_botline {
        return;
    }
    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
        getvcol(
            curwin.get(),
            lpos,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
    }
    let mut col_visible: bool = (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        || vcol >= (*curwin.get()).w_leftcol
            && vcol
                < (*curwin.get()).w_leftcol as ::core::ffi::c_int + (*curwin.get()).w_view_width;
    if !col_visible {
        return;
    }
    let mut mpos: pos_T = *lpos;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut save_so: OptInt = *so;
    let mut save_siso: OptInt = *siso;
    if dollar_vcol.get() >= 0 as ::core::ffi::c_int
        && dollar_vcol.get() == (*curwin.get()).w_virtcol
    {
        dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
    }
    (*curwin.get()).w_virtcol += 1;
    let mut save_dollar_vcol: colnr_T = dollar_vcol.get();
    let mut save_state: ::core::ffi::c_int = State.get();
    State.set(MODE_SHOWMATCH as ::core::ffi::c_int);
    ui_cursor_shape();
    (*curwin.get()).w_cursor = mpos;
    *so = 0 as OptInt;
    *siso = 0 as OptInt;
    show_cursor_info_later(false_0 != 0);
    update_screen();
    setcursor();
    ui_flush();
    dollar_vcol.set(save_dollar_vcol);
    if !vim_strchr(p_cpo.get(), CPO_SHOWMATCH).is_null() {
        os_delay(
            (p_mat.get() as uint64_t)
                .wrapping_mul(100 as uint64_t)
                .wrapping_add(8 as uint64_t),
            true_0 != 0,
        );
    } else if !char_avail() {
        os_delay(
            (p_mat.get() as uint64_t)
                .wrapping_mul(100 as uint64_t)
                .wrapping_add(9 as uint64_t),
            false_0 != 0,
        );
    }
    (*curwin.get()).w_cursor = save_cursor;
    *so = save_so;
    *siso = save_siso;
    State.set(save_state);
    ui_cursor_shape();
}
#[no_mangle]
pub unsafe extern "C" fn current_search(
    mut count: ::core::ffi::c_int,
    mut forward: bool,
) -> ::core::ffi::c_int {
    let mut old_p_ws: bool = p_ws.get() != 0;
    let mut save_VIsual: pos_T = VIsual.get();
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        dec_cursor();
    }
    let skip_first_backward: bool = forward as ::core::ffi::c_int != 0
        && VIsual_active.get() as ::core::ffi::c_int != 0
        && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut orig_pos: pos_T = (*curwin.get()).w_cursor;
    if VIsual_active.get() {
        if forward {
            incl(&raw mut pos);
        } else {
            decl(&raw mut pos);
        }
    }
    let mut zero_width: ::core::ffi::c_int = is_zero_width(
        (*spats.ptr())[last_idx.get() as usize].pat,
        (*spats.ptr())[last_idx.get() as usize].patlen,
        true_0 != 0,
        &raw mut (*curwin.get()).w_cursor,
        FORWARD,
    );
    if zero_width == -1 as ::core::ffi::c_int {
        return FAIL;
    }
    let mut end_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut result: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        let mut dir: ::core::ffi::c_int = 0;
        's_71: {
            if forward {
                if i == 0 as ::core::ffi::c_int && skip_first_backward as ::core::ffi::c_int != 0 {
                    break 's_71;
                } else {
                    dir = i;
                }
            } else {
                dir = (i == 0) as ::core::ffi::c_int;
            }
            let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if dir == 0 && zero_width == 0 {
                flags = SEARCH_END as ::core::ffi::c_int;
            }
            end_pos = pos;
            if i == 0 as ::core::ffi::c_int {
                p_ws.set(false_0);
            }
            result = searchit(
                curwin.get(),
                curbuf.get(),
                &raw mut pos,
                &raw mut end_pos,
                (if dir != 0 {
                    FORWARD as ::core::ffi::c_int
                } else {
                    BACKWARD as ::core::ffi::c_int
                }) as Direction,
                (*spats.ptr())[last_idx.get() as usize].pat,
                (*spats.ptr())[last_idx.get() as usize].patlen,
                if i != 0 {
                    count
                } else {
                    1 as ::core::ffi::c_int
                },
                SEARCH_KEEP as ::core::ffi::c_int | flags,
                RE_SEARCH as ::core::ffi::c_int,
                ::core::ptr::null_mut::<searchit_arg_T>(),
            );
            p_ws.set(old_p_ws as ::core::ffi::c_int);
            if i == 1 as ::core::ffi::c_int && result == 0 {
                (*curwin.get()).w_cursor = orig_pos;
                if VIsual_active.get() {
                    VIsual.set(save_VIsual);
                }
                return FAIL;
            } else if i == 0 as ::core::ffi::c_int && result == 0 {
                if forward {
                    clearpos(&raw mut pos);
                } else {
                    pos.lnum = (*(*curwin.get()).w_buffer).b_ml.ml_line_count;
                    pos.col = ml_get_len((*(*curwin.get()).w_buffer).b_ml.ml_line_count);
                }
            }
        }
        i += 1;
    }
    let mut start_pos: pos_T = pos;
    if !VIsual_active.get() {
        VIsual.set(start_pos);
    }
    (*curwin.get()).w_cursor = end_pos;
    if lt(VIsual.get(), end_pos) as ::core::ffi::c_int != 0 && forward as ::core::ffi::c_int != 0 {
        if skip_first_backward {
            (*curwin.get()).w_cursor = pos;
        } else {
            dec_cursor();
        }
    } else if VIsual_active.get() as ::core::ffi::c_int != 0
        && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
        && forward as ::core::ffi::c_int != 0
    {
        (*curwin.get()).w_cursor = pos;
    }
    VIsual_active.set(true_0 != 0);
    VIsual_mode.set('v' as ::core::ffi::c_int);
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
        if forward as ::core::ffi::c_int != 0
            && ltoreq(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
        {
            inc_cursor();
        } else if !forward
            && ltoreq((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
        {
            inc(VIsual.ptr());
        }
    }
    if fdo_flags.get() & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    may_start_select('c' as ::core::ffi::c_int);
    setmouse();
    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    showmode();
    return OK;
}
unsafe extern "C" fn is_zero_width(
    mut pattern: *mut ::core::ffi::c_char,
    mut patternlen: size_t,
    mut move_0: bool,
    mut cur: *mut pos_T,
    mut direction: Direction,
) -> ::core::ffi::c_int {
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut result: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    let mut flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if pattern.is_null() {
        pattern = (*spats.ptr())[last_idx.get() as usize].pat;
        patternlen = (*spats.ptr())[last_idx.get() as usize].patlen;
    }
    if search_regcomp(
        pattern,
        patternlen,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        RE_SEARCH as ::core::ffi::c_int,
        RE_SEARCH as ::core::ffi::c_int,
        SEARCH_KEEP as ::core::ffi::c_int,
        &raw mut regmatch,
    ) == FAIL
    {
        return -1 as ::core::ffi::c_int;
    }
    regmatch.startpos[0 as ::core::ffi::c_int as usize].col = -1 as ::core::ffi::c_int as colnr_T;
    if move_0 {
        clearpos(&raw mut pos);
    } else {
        pos = *cur;
        flag = SEARCH_START as ::core::ffi::c_int;
    }
    if searchit(
        curwin.get(),
        curbuf.get(),
        &raw mut pos,
        ::core::ptr::null_mut::<pos_T>(),
        direction,
        pattern,
        patternlen,
        1 as ::core::ffi::c_int,
        SEARCH_KEEP as ::core::ffi::c_int + flag,
        RE_SEARCH as ::core::ffi::c_int,
        ::core::ptr::null_mut::<searchit_arg_T>(),
    ) != FAIL
    {
        let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            regmatch.startpos[0 as ::core::ffi::c_int as usize].col += 1;
            nmatched = vim_regexec_multi(
                &raw mut regmatch,
                curwin.get(),
                curbuf.get(),
                pos.lnum,
                regmatch.startpos[0 as ::core::ffi::c_int as usize].col,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if nmatched != 0 as ::core::ffi::c_int {
                break;
            }
            if !(!regmatch.regprog.is_null()
                && (if direction as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                    (regmatch.startpos[0 as ::core::ffi::c_int as usize].col < pos.col)
                        as ::core::ffi::c_int
                } else {
                    (regmatch.startpos[0 as ::core::ffi::c_int as usize].col > pos.col)
                        as ::core::ffi::c_int
                }) != 0)
            {
                break;
            }
        }
        if called_emsg.get() == called_emsg_before {
            result = (nmatched != 0 as ::core::ffi::c_int
                && regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                    == regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                && regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                    == regmatch.endpos[0 as ::core::ffi::c_int as usize].col)
                as ::core::ffi::c_int;
        }
    }
    vim_regfree(regmatch.regprog);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn linewhite(mut lnum: linenr_T) -> bool {
    let mut p: *mut ::core::ffi::c_char = skipwhite(ml_get(lnum));
    return *p as ::core::ffi::c_int == NUL;
}
unsafe extern "C" fn cmdline_search_stat(
    mut dirc: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut cursor_pos: *mut pos_T,
    mut show_top_bot_msg: bool,
    mut msgbuf: *mut ::core::ffi::c_char,
    mut msgbuflen: size_t,
    mut recompute: bool,
    mut maxcount: ::core::ffi::c_int,
    mut timeout: ::core::ffi::c_int,
) {
    let mut stat: searchstat_T = searchstat_T {
        cur: 0,
        cnt: 0,
        exact_match: false,
        incomplete: 0,
        last_maxcount: 0,
    };
    update_search_stat(
        dirc,
        pos,
        cursor_pos,
        &raw mut stat,
        recompute,
        maxcount,
        timeout,
    );
    if stat.cur <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut t: [::core::ffi::c_char; 16] = [0; 16];
    let mut len: size_t = 0;
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0
        && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int == 's' as ::core::ffi::c_int
    {
        if stat.incomplete == 1 as ::core::ffi::c_int {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[?/??]\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        } else if stat.cnt > maxcount && stat.cur > maxcount {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[>%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
                maxcount,
                maxcount,
            ) as size_t;
        } else if stat.cnt > maxcount {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[>%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
                maxcount,
                stat.cur,
            ) as size_t;
        } else {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
                stat.cnt,
                stat.cur,
            ) as size_t;
        }
    } else if stat.incomplete == 1 as ::core::ffi::c_int {
        len = vim_snprintf(
            &raw mut t as *mut ::core::ffi::c_char,
            SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
            b"[?/??]\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t;
    } else if stat.cnt > maxcount && stat.cur > maxcount {
        len = vim_snprintf(
            &raw mut t as *mut ::core::ffi::c_char,
            SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
            b"[>%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
            maxcount,
            maxcount,
        ) as size_t;
    } else if stat.cnt > maxcount {
        len = vim_snprintf(
            &raw mut t as *mut ::core::ffi::c_char,
            SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
            b"[%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
            stat.cur,
            maxcount,
        ) as size_t;
    } else {
        len = vim_snprintf(
            &raw mut t as *mut ::core::ffi::c_char,
            SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
            b"[%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
            stat.cur,
            stat.cnt,
        ) as size_t;
    }
    if show_top_bot_msg as ::core::ffi::c_int != 0
        && len.wrapping_add(2 as size_t) < SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t
    {
        memmove(
            (&raw mut t as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            &raw mut t as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            len,
        );
        t[0 as ::core::ffi::c_int as usize] = 'W' as ::core::ffi::c_char;
        t[1 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
        len = len.wrapping_add(2 as size_t);
    }
    if len > msgbuflen {
        len = msgbuflen;
    }
    memmove(
        msgbuf.offset(msgbuflen as isize).offset(-(len as isize)) as *mut ::core::ffi::c_void,
        &raw mut t as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        len,
    );
    if dirc == '?' as ::core::ffi::c_int && stat.cur == maxcount + 1 as ::core::ffi::c_int {
        stat.cur = -1 as ::core::ffi::c_int;
    }
    msg_ext_overwrite.set(true_0 != 0);
    msg_ext_set_kind(b"search_count\0".as_ptr() as *const ::core::ffi::c_char);
    give_warning(msgbuf, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn update_search_stat(
    mut dirc: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut cursor_pos: *mut pos_T,
    mut stat: *mut searchstat_T,
    mut recompute: bool,
    mut maxcount: ::core::ffi::c_int,
    mut timeout: ::core::ffi::c_int,
) {
    let mut save_ws: ::core::ffi::c_int = p_ws.get();
    let mut wraparound: bool = false_0 != 0;
    let mut p: pos_T = *pos;
    static lastpos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    });
    static cur: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static exact_match: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static incomplete: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static last_maxcount: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static chgtick: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static lastpat: GlobalCell<*mut ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
    static lastpatlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
    static lbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
    memset(
        stat as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<searchstat_T>(),
    );
    if dirc == 0 as ::core::ffi::c_int
        && !recompute
        && !((*lastpos.ptr()).lnum == 0 as linenr_T
            && (*lastpos.ptr()).col == 0 as ::core::ffi::c_int
            && (*lastpos.ptr()).coladd == 0 as ::core::ffi::c_int)
    {
        (*stat).cur = cur.get();
        (*stat).cnt = cnt.get();
        (*stat).exact_match = exact_match.get();
        (*stat).incomplete = incomplete.get();
        (*stat).last_maxcount = p_msc.get() as ::core::ffi::c_int;
        return;
    }
    last_maxcount.set(maxcount);
    wraparound = dirc == '?' as ::core::ffi::c_int
        && lt(lastpos.get(), p) as ::core::ffi::c_int != 0
        || dirc == '/' as ::core::ffi::c_int && lt(p, lastpos.get()) as ::core::ffi::c_int != 0;
    if !(chgtick.get() as varnumber_T == buf_get_changedtick(curbuf.get())
        && (!(*lastpat.ptr()).is_null()
            && strncmp(
                lastpat.get(),
                (*spats.ptr())[last_idx.get() as usize].pat,
                lastpatlen.get(),
            ) == 0 as ::core::ffi::c_int
            && lastpatlen.get() == (*spats.ptr())[last_idx.get() as usize].patlen)
        && equalpos(lastpos.get(), *cursor_pos) as ::core::ffi::c_int != 0
        && lbuf.get() == curbuf.get())
        || wraparound as ::core::ffi::c_int != 0
        || cur.get() < 0 as ::core::ffi::c_int
        || maxcount > 0 as ::core::ffi::c_int && cur.get() > maxcount
        || recompute as ::core::ffi::c_int != 0
    {
        cur.set(0 as ::core::ffi::c_int);
        cnt.set(0 as ::core::ffi::c_int);
        exact_match.set(false_0 != 0);
        incomplete.set(0 as ::core::ffi::c_int);
        clearpos(lastpos.ptr());
        lbuf.set(curbuf.get());
    }
    if equalpos(lastpos.get(), *cursor_pos) as ::core::ffi::c_int != 0
        && !wraparound
        && (if dirc == 0 as ::core::ffi::c_int || dirc == '/' as ::core::ffi::c_int {
            (cur.get() < cnt.get()) as ::core::ffi::c_int
        } else {
            (cur.get() > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
        }) != 0
    {
        (*cur.ptr()) += if dirc == 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else if dirc == '/' as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        let mut start: proftime_T = 0;
        let mut done_search: bool = false_0 != 0;
        let mut endpos: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        };
        p_ws.set(false_0);
        if timeout > 0 as ::core::ffi::c_int {
            start = profile_setlimit(timeout as int64_t);
        }
        while !got_int.get()
            && searchit(
                curwin.get(),
                curbuf.get(),
                lastpos.ptr(),
                &raw mut endpos,
                FORWARD,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as size_t,
                1 as ::core::ffi::c_int,
                SEARCH_KEEP as ::core::ffi::c_int,
                RE_LAST as ::core::ffi::c_int,
                ::core::ptr::null_mut::<searchit_arg_T>(),
            ) != FAIL
        {
            done_search = true_0 != 0;
            if timeout > 0 as ::core::ffi::c_int
                && profile_passed_limit(start) as ::core::ffi::c_int != 0
            {
                incomplete.set(1 as ::core::ffi::c_int);
                break;
            } else {
                (*cnt.ptr()) += 1;
                if ltoreq(lastpos.get(), p) {
                    cur.set(cnt.get());
                    if lt(p, endpos) {
                        exact_match.set(true_0 != 0);
                    }
                }
                fast_breakcheck();
                if !(maxcount > 0 as ::core::ffi::c_int && cnt.get() > maxcount) {
                    continue;
                }
                incomplete.set(2 as ::core::ffi::c_int);
                break;
            }
        }
        if got_int.get() {
            cur.set(-1 as ::core::ffi::c_int);
        }
        if done_search {
            xfree(lastpat.get() as *mut ::core::ffi::c_void);
            lastpat.set(xstrnsave(
                (*spats.ptr())[last_idx.get() as usize].pat,
                (*spats.ptr())[last_idx.get() as usize].patlen,
            ));
            lastpatlen.set((*spats.ptr())[last_idx.get() as usize].patlen);
            chgtick.set(buf_get_changedtick(curbuf.get()) as ::core::ffi::c_int);
            lbuf.set(curbuf.get());
            lastpos.set(p);
        }
    }
    (*stat).cur = cur.get();
    (*stat).cnt = cnt.get();
    (*stat).exact_match = exact_match.get();
    (*stat).incomplete = incomplete.get();
    (*stat).last_maxcount = last_maxcount.get();
    p_ws.set(save_ws);
}
#[no_mangle]
pub unsafe extern "C" fn f_searchcount(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut maxcount: ::core::ffi::c_int = p_msc.get() as ::core::ffi::c_int;
    let mut timeout: ::core::ffi::c_int = SEARCH_STAT_DEF_TIMEOUT as ::core::ffi::c_int;
    let mut recompute: bool = true_0 != 0;
    let mut stat: searchstat_T = searchstat_T {
        cur: 0,
        cnt: 0,
        exact_match: false,
        incomplete: 0,
        last_maxcount: 0,
    };
    tv_dict_alloc_ret(rettv);
    if shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) {
        recompute = true_0 != 0;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut error: bool = false_0 != 0;
        if tv_check_for_nonnull_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        dict = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        di = tv_dict_find(
            dict,
            b"timeout\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            timeout = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) as ::core::ffi::c_int;
            if error {
                return;
            }
        }
        di = tv_dict_find(
            dict,
            b"maxcount\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            maxcount =
                tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) as ::core::ffi::c_int;
            if error {
                return;
            }
        }
        di = tv_dict_find(
            dict,
            b"recompute\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            recompute = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) != 0;
            if error {
                return;
            }
        }
        di = tv_dict_find(
            dict,
            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            pattern = tv_get_string_chk(&raw mut (*di).di_tv) as *mut ::core::ffi::c_char;
            if pattern.is_null() {
                return;
            }
        }
        di = tv_dict_find(
            dict,
            b"pos\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"pos\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
            if tv_list_len((*di).di_tv.vval.v_list) != 3 as ::core::ffi::c_int {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"List format should be [lnum, col, off]\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                return;
            }
            let mut li: *mut listitem_T =
                tv_list_find((*di).di_tv.vval.v_list, 0 as ::core::ffi::c_int);
            if !li.is_null() {
                pos.lnum = tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error) as linenr_T;
                if error {
                    return;
                }
            }
            li = tv_list_find((*di).di_tv.vval.v_list, 1 as ::core::ffi::c_int);
            if !li.is_null() {
                pos.col = (tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as colnr_T;
                if error {
                    return;
                }
            }
            li = tv_list_find((*di).di_tv.vval.v_list, 2 as ::core::ffi::c_int);
            if !li.is_null() {
                pos.coladd = tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error) as colnr_T;
                if error {
                    return;
                }
            }
        }
    }
    save_last_search_pattern();
    save_incsearch_state();
    '_the_end: {
        if !pattern.is_null() {
            if *pattern as ::core::ffi::c_int == NUL {
                break '_the_end;
            } else {
                xfree((*spats.ptr())[last_idx.get() as usize].pat as *mut ::core::ffi::c_void);
                (*spats.ptr())[last_idx.get() as usize].patlen = strlen(pattern);
                (*spats.ptr())[last_idx.get() as usize].pat =
                    xstrnsave(pattern, (*spats.ptr())[last_idx.get() as usize].patlen);
            }
        }
        if !((*spats.ptr())[last_idx.get() as usize].pat.is_null()
            || *(*spats.ptr())[last_idx.get() as usize].pat as ::core::ffi::c_int == NUL)
        {
            update_search_stat(
                0 as ::core::ffi::c_int,
                &raw mut pos,
                &raw mut pos,
                &raw mut stat,
                recompute,
                maxcount,
                timeout,
            );
            tv_dict_add_nr(
                (*rettv).vval.v_dict,
                b"current\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                stat.cur as varnumber_T,
            );
            tv_dict_add_nr(
                (*rettv).vval.v_dict,
                b"total\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                stat.cnt as varnumber_T,
            );
            tv_dict_add_nr(
                (*rettv).vval.v_dict,
                b"exact_match\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                stat.exact_match as varnumber_T,
            );
            tv_dict_add_nr(
                (*rettv).vval.v_dict,
                b"incomplete\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                stat.incomplete as varnumber_T,
            );
            tv_dict_add_nr(
                (*rettv).vval.v_dict,
                b"maxcount\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                stat.last_maxcount as varnumber_T,
            );
        }
    }
    restore_last_search_pattern();
    restore_incsearch_state();
}
unsafe extern "C" fn get_line_and_copy(
    mut lnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    xstrlcpy(buf, line, LSIZE as ::core::ffi::c_int as size_t);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn find_pattern_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut len: size_t,
    mut whole: bool,
    mut skip_comments: bool,
    mut type_0: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut action: ::core::ffi::c_int,
    mut start_lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut forceit: bool,
    mut silent: bool,
) {
    let mut inc_opt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_files: ::core::ffi::c_int = 0;
    let mut depth: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c2rust_current_block: u64;
    let mut files: *mut SearchedFile = ::core::ptr::null_mut::<SearchedFile>();
    let mut bigger: *mut SearchedFile = ::core::ptr::null_mut::<SearchedFile>();
    let mut max_path_depth: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
    let mut match_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut new_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curr_fname: *mut ::core::ffi::c_char = (*curbuf.get()).b_fname;
    let mut prev_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut depth_displayed: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut define_matched: bool = false;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut incl_regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut def_regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut matched: bool = false_0 != 0;
    let mut did_show: bool = false_0 != 0;
    let mut found: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut already: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut startp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curwin_save: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let l_g_do_tagpreview: ::core::ffi::c_int = g_do_tagpreview.get();
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    incl_regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    def_regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut file_line: *mut ::core::ffi::c_char =
        xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
    '_fpip_end: {
        if type_0 != CHECK_PATH as ::core::ffi::c_int
            && type_0 != FIND_DEFINE as ::core::ffi::c_int
            && !compl_status_sol()
        {
            let mut patsize: size_t = len.wrapping_add(5 as size_t);
            let mut pat: *mut ::core::ffi::c_char = xmalloc(patsize) as *mut ::core::ffi::c_char;
            '_c2rust_label: {
                if len <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        2966 as ::core::ffi::c_uint,
                        b"void find_pattern_in_path(char *, Direction, size_t, _Bool, _Bool, int, int, int, linenr_T, linenr_T, _Bool, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            snprintf(
                pat,
                patsize,
                if whole as ::core::ffi::c_int != 0 {
                    b"\\<%.*s\\>\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"%.*s\0".as_ptr() as *const ::core::ffi::c_char
                },
                len as ::core::ffi::c_int,
                ptr,
            );
            regmatch.rm_ic = ignorecase(pat) != 0;
            regmatch.regprog = vim_regcomp(
                pat,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            xfree(pat as *mut ::core::ffi::c_void);
            if regmatch.regprog.is_null() {
                break '_fpip_end;
            }
        }
        inc_opt = if *(*curbuf.get()).b_p_inc as ::core::ffi::c_int == NUL {
            p_inc.get()
        } else {
            (*curbuf.get()).b_p_inc
        };
        if *inc_opt as ::core::ffi::c_int != NUL {
            incl_regmatch.regprog = vim_regcomp(
                inc_opt,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if incl_regmatch.regprog.is_null() {
                break '_fpip_end;
            } else {
                incl_regmatch.rm_ic = false_0 != 0;
            }
        }
        if type_0 == FIND_DEFINE as ::core::ffi::c_int
            && (*(*curbuf.get()).b_p_def as ::core::ffi::c_int != NUL
                || *p_def.get() as ::core::ffi::c_int != NUL)
        {
            def_regmatch.regprog = vim_regcomp(
                if *(*curbuf.get()).b_p_def as ::core::ffi::c_int == NUL {
                    p_def.get()
                } else {
                    (*curbuf.get()).b_p_def
                },
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if def_regmatch.regprog.is_null() {
                break '_fpip_end;
            } else {
                def_regmatch.rm_ic = false_0 != 0;
            }
        }
        files = xcalloc(
            max_path_depth as size_t,
            ::core::mem::size_of::<SearchedFile>(),
        ) as *mut SearchedFile;
        old_files = max_path_depth;
        depth_displayed = -1 as ::core::ffi::c_int;
        depth = depth_displayed;
        end_lnum = if end_lnum < (*curbuf.get()).b_ml.ml_line_count {
            end_lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        lnum = if start_lnum < end_lnum {
            start_lnum
        } else {
            end_lnum
        };
        line = get_line_and_copy(lnum, file_line);
        's_1511: loop {
            if !incl_regmatch.regprog.is_null()
                && vim_regexec(&raw mut incl_regmatch, line, 0 as colnr_T) as ::core::ffi::c_int
                    != 0
            {
                let mut p_fname: *mut ::core::ffi::c_char = if curr_fname == (*curbuf.get()).b_fname
                {
                    (*curbuf.get()).b_ffname
                } else {
                    curr_fname
                };
                if !strstr(inc_opt, b"\\zs\0".as_ptr() as *const ::core::ffi::c_char).is_null() {
                    new_fname = find_file_name_in_path(
                        incl_regmatch.startp[0 as ::core::ffi::c_int as usize],
                        incl_regmatch.endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(incl_regmatch.startp[0 as ::core::ffi::c_int as usize])
                            as size_t,
                        FNAME_EXP as ::core::ffi::c_int
                            | FNAME_INCL as ::core::ffi::c_int
                            | FNAME_REL as ::core::ffi::c_int,
                        1 as ::core::ffi::c_long,
                        p_fname,
                    );
                } else {
                    new_fname = file_name_in_line(
                        incl_regmatch.endp[0 as ::core::ffi::c_int as usize],
                        0 as ::core::ffi::c_int,
                        FNAME_EXP as ::core::ffi::c_int
                            | FNAME_INCL as ::core::ffi::c_int
                            | FNAME_REL as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        p_fname,
                        ::core::ptr::null_mut::<linenr_T>(),
                    );
                }
                let mut already_searched: bool = false_0 != 0;
                if !new_fname.is_null() {
                    i = 0 as ::core::ffi::c_int;
                    loop {
                        if i == depth + 1 as ::core::ffi::c_int {
                            i = old_files;
                        }
                        if i == max_path_depth {
                            break;
                        }
                        if path_full_compare(
                            new_fname,
                            (*files.offset(i as isize)).name,
                            true_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                        {
                            if type_0 != CHECK_PATH as ::core::ffi::c_int
                                && action == ACTION_SHOW_ALL as ::core::ffi::c_int
                                && (*files.offset(i as isize)).matched != 0
                            {
                                msg_putchar('\n' as ::core::ffi::c_int);
                                if !got_int.get() {
                                    msg_home_replace(new_fname);
                                    msg_puts(gettext(
                                        b" (includes previously listed match)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ));
                                    prev_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                }
                            }
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut new_fname as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL_0;
                            let _ = *ptr_;
                            already_searched = true_0 != 0;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                }
                if type_0 == CHECK_PATH as ::core::ffi::c_int
                    && (action == ACTION_SHOW_ALL as ::core::ffi::c_int
                        || new_fname.is_null() && !already_searched)
                {
                    if did_show {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    } else {
                        gotocmdline(true_0 != 0);
                        msg_puts_title(gettext(
                            b"--- Included files \0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        if action != ACTION_SHOW_ALL as ::core::ffi::c_int {
                            msg_puts_title(gettext(
                                b"not found \0".as_ptr() as *const ::core::ffi::c_char
                            ));
                        }
                        msg_puts_title(gettext(
                            b"in path ---\n\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    }
                    did_show = true_0 != 0;
                    while depth_displayed < depth && !got_int.get() {
                        depth_displayed += 1;
                        i = 0 as ::core::ffi::c_int;
                        while i < depth_displayed {
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            i += 1;
                        }
                        msg_home_replace((*files.offset(depth_displayed as isize)).name);
                        msg_puts(b" -->\n\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    if !got_int.get() {
                        i = 0 as ::core::ffi::c_int;
                        while i <= depth_displayed {
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            i += 1;
                        }
                        if !new_fname.is_null() {
                            msg_outtrans(new_fname, HLF_D as ::core::ffi::c_int, false_0 != 0);
                        } else {
                            if !strstr(inc_opt, b"\\zs\0".as_ptr() as *const ::core::ffi::c_char)
                                .is_null()
                            {
                                p = incl_regmatch.startp[0 as ::core::ffi::c_int as usize];
                                i = incl_regmatch.endp[0 as ::core::ffi::c_int as usize]
                                    .offset_from(
                                        incl_regmatch.startp[0 as ::core::ffi::c_int as usize],
                                    ) as ::core::ffi::c_int;
                            } else {
                                p = incl_regmatch.endp[0 as ::core::ffi::c_int as usize];
                                while *p as ::core::ffi::c_int != 0
                                    && !vim_isfilec(*p as uint8_t as ::core::ffi::c_int)
                                {
                                    p = p.offset(1);
                                }
                                i = 0 as ::core::ffi::c_int;
                                while vim_isfilec(
                                    *p.offset(i as isize) as uint8_t as ::core::ffi::c_int
                                ) {
                                    i += 1;
                                }
                            }
                            if i == 0 as ::core::ffi::c_int {
                                p = incl_regmatch.endp[0 as ::core::ffi::c_int as usize];
                                i = strlen(p) as ::core::ffi::c_int;
                            } else if p > line {
                                if *p.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '"' as ::core::ffi::c_int
                                    || *p.offset(-1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '<' as ::core::ffi::c_int
                                {
                                    p = p.offset(-1);
                                    i += 1;
                                }
                                if *p.offset(i as isize) as ::core::ffi::c_int
                                    == '"' as ::core::ffi::c_int
                                    || *p.offset(i as isize) as ::core::ffi::c_int
                                        == '>' as ::core::ffi::c_int
                                {
                                    i += 1;
                                }
                            }
                            let mut save_char: ::core::ffi::c_char = *p.offset(i as isize);
                            *p.offset(i as isize) = NUL as ::core::ffi::c_char;
                            msg_outtrans(p, HLF_D as ::core::ffi::c_int, false_0 != 0);
                            *p.offset(i as isize) = save_char;
                        }
                        if new_fname.is_null() && action == ACTION_SHOW_ALL as ::core::ffi::c_int {
                            if already_searched {
                                msg_puts(gettext(
                                    b"  (Already listed)\0".as_ptr() as *const ::core::ffi::c_char
                                ));
                            } else {
                                msg_puts(gettext(
                                    b"  NOT FOUND\0".as_ptr() as *const ::core::ffi::c_char
                                ));
                            }
                        }
                    }
                }
                if !new_fname.is_null() {
                    if depth + 1 as ::core::ffi::c_int == old_files {
                        bigger = xmalloc(
                            (max_path_depth as size_t)
                                .wrapping_mul(2 as size_t)
                                .wrapping_mul(::core::mem::size_of::<SearchedFile>()),
                        ) as *mut SearchedFile;
                        i = 0 as ::core::ffi::c_int;
                        while i <= depth {
                            *bigger.offset(i as isize) = *files.offset(i as isize);
                            i += 1;
                        }
                        i = depth + 1 as ::core::ffi::c_int;
                        while i < old_files + max_path_depth {
                            (*bigger.offset(i as isize)).fp = ::core::ptr::null_mut::<FILE>();
                            (*bigger.offset(i as isize)).name =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            (*bigger.offset(i as isize)).lnum = 0 as ::core::ffi::c_int as linenr_T;
                            (*bigger.offset(i as isize)).matched = false_0;
                            i += 1;
                        }
                        i = old_files;
                        while i < max_path_depth {
                            *bigger.offset((i + max_path_depth) as isize) =
                                *files.offset(i as isize);
                            i += 1;
                        }
                        old_files += max_path_depth;
                        max_path_depth *= 2 as ::core::ffi::c_int;
                        xfree(files as *mut ::core::ffi::c_void);
                        files = bigger;
                    }
                    (*files.offset((depth + 1 as ::core::ffi::c_int) as isize)).fp =
                        os_fopen(new_fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
                    if (*files.offset((depth + 1 as ::core::ffi::c_int) as isize))
                        .fp
                        .is_null()
                    {
                        xfree(new_fname as *mut ::core::ffi::c_void);
                    } else {
                        depth += 1;
                        if depth == old_files {
                            xfree(
                                (*files.offset(old_files as isize)).name
                                    as *mut ::core::ffi::c_void,
                            );
                            old_files += 1;
                        }
                        curr_fname = new_fname;
                        (*files.offset(depth as isize)).name = curr_fname;
                        (*files.offset(depth as isize)).lnum = 0 as ::core::ffi::c_int as linenr_T;
                        (*files.offset(depth as isize)).matched = false_0;
                        if action == ACTION_EXPAND as ::core::ffi::c_int
                            && !shortmess(SHM_COMPLETIONSCAN as ::core::ffi::c_int)
                            && !silent
                        {
                            msg_hist_off.set(true_0 != 0);
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                gettext(b"Scanning included file: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                new_fname,
                            );
                            msg_trunc(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                true_0 != 0,
                                HLF_R as ::core::ffi::c_int,
                            );
                        } else if p_verbose.get() >= 5 as OptInt {
                            verbose_enter();
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Searching included file %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                new_fname,
                            );
                            verbose_leave();
                        }
                    }
                }
                c2rust_current_block = 9985465603744958559;
            } else {
                p = line;
                c2rust_current_block = 2704434626355109080;
            }
            loop {
                match c2rust_current_block {
                    2704434626355109080 => {
                        define_matched = false_0 != 0;
                        if !def_regmatch.regprog.is_null()
                            && vim_regexec(&raw mut def_regmatch, line, 0 as colnr_T)
                                as ::core::ffi::c_int
                                != 0
                        {
                            p = def_regmatch.endp[0 as ::core::ffi::c_int as usize];
                            while *p as ::core::ffi::c_int != 0
                                && !vim_iswordc(*p as uint8_t as ::core::ffi::c_int)
                            {
                                p = p.offset(1);
                            }
                            define_matched = true_0 != 0;
                        }
                        if def_regmatch.regprog.is_null()
                            || define_matched as ::core::ffi::c_int != 0
                        {
                            if define_matched as ::core::ffi::c_int != 0
                                || compl_status_sol() as ::core::ffi::c_int != 0
                            {
                                startp = skipwhite(p);
                                if p_ic.get() != 0 {
                                    matched = mb_strnicmp(startp, ptr, len) == 0;
                                } else {
                                    matched = strncmp(startp, ptr, len) == 0;
                                }
                                if matched as ::core::ffi::c_int != 0
                                    && define_matched as ::core::ffi::c_int != 0
                                    && whole as ::core::ffi::c_int != 0
                                    && vim_iswordc(*startp.offset(len as isize) as uint8_t
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    matched = false_0 != 0;
                                }
                            } else if !regmatch.regprog.is_null()
                                && vim_regexec(
                                    &raw mut regmatch,
                                    line,
                                    p.offset_from(line) as colnr_T,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                matched = true_0 != 0;
                                startp = regmatch.startp[0 as ::core::ffi::c_int as usize];
                                if skip_comments {
                                    if (*line as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                                        || strncmp(
                                            skipwhite(
                                                line.offset(1 as ::core::ffi::c_int as isize),
                                            ),
                                            b"define\0".as_ptr() as *const ::core::ffi::c_char,
                                            6 as size_t,
                                        ) != 0 as ::core::ffi::c_int)
                                        && get_leader_len(
                                            line,
                                            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                                            false_0 != 0,
                                            true_0 != 0,
                                        ) != 0
                                    {
                                        matched = false_0 != 0;
                                    }
                                    p = skipwhite(line);
                                    if matched as ::core::ffi::c_int != 0
                                        || *p.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '/' as ::core::ffi::c_int
                                            && *p.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == '*' as ::core::ffi::c_int
                                        || *p.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '*' as ::core::ffi::c_int
                                    {
                                        p = line;
                                        while *p as ::core::ffi::c_int != 0 && p < startp {
                                            if matched as ::core::ffi::c_int != 0
                                                && *p.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                                && (*p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '*' as ::core::ffi::c_int
                                                    || *p.offset(1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '/' as ::core::ffi::c_int)
                                            {
                                                matched = false_0 != 0;
                                                if *p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                                {
                                                    break;
                                                }
                                                p = p.offset(1);
                                            } else if !matched
                                                && *p.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '*' as ::core::ffi::c_int
                                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                            {
                                                matched = true_0 != 0;
                                                p = p.offset(1);
                                            }
                                            p = p.offset(1);
                                        }
                                    }
                                }
                            }
                        }
                        c2rust_current_block = 9985465603744958559;
                    }
                    _ => {
                        if !matched {
                            break;
                        }
                        '_exit_matched: {
                            if action == ACTION_EXPAND as ::core::ffi::c_int {
                                let mut cont_s_ipos: bool = false_0 != 0;
                                if depth == -1 as ::core::ffi::c_int
                                    && lnum == (*curwin.get()).w_cursor.lnum
                                {
                                    break 's_1511;
                                }
                                found = true_0 != 0;
                                p = startp;
                                let mut aux: *mut ::core::ffi::c_char = p;
                                if compl_status_adding() as ::core::ffi::c_int != 0
                                    && strlen(p) as ::core::ffi::c_int >= ins_compl_len()
                                {
                                    p = p.offset(ins_compl_len() as isize);
                                    if vim_iswordp(p) {
                                        break '_exit_matched;
                                    } else {
                                        p = find_word_start(p);
                                    }
                                }
                                p = find_word_end(p);
                                i = p.offset_from(aux) as ::core::ffi::c_int;
                                if compl_status_adding() as ::core::ffi::c_int != 0
                                    && i == ins_compl_len()
                                {
                                    strncpy(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        aux,
                                        i as size_t,
                                    );
                                    if depth < 0 as ::core::ffi::c_int {
                                        if lnum >= end_lnum {
                                            break '_exit_matched;
                                        } else {
                                            lnum += 1;
                                            line = get_line_and_copy(lnum, file_line);
                                        }
                                    } else {
                                        line = file_line;
                                        if vim_fgets(
                                            line,
                                            LSIZE as ::core::ffi::c_int,
                                            (*files.offset(depth as isize)).fp,
                                        ) {
                                            break '_exit_matched;
                                        }
                                    }
                                    p = skipwhite(line);
                                    aux = p;
                                    already = aux;
                                    p = find_word_start(p);
                                    p = find_word_end(p);
                                    if p > aux {
                                        if *aux as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                                            && (*IObuff.ptr())
                                                [(i - 1 as ::core::ffi::c_int) as usize]
                                                as ::core::ffi::c_int
                                                != TAB
                                        {
                                            if (*IObuff.ptr())
                                                [(i - 1 as ::core::ffi::c_int) as usize]
                                                as ::core::ffi::c_int
                                                != ' ' as ::core::ffi::c_int
                                            {
                                                let c2rust_fresh9 = i;
                                                i = i + 1;
                                                (*IObuff.ptr())[c2rust_fresh9 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                            }
                                            if p_js.get() != 0
                                                && ((*IObuff.ptr())
                                                    [(i - 2 as ::core::ffi::c_int) as usize]
                                                    as ::core::ffi::c_int
                                                    == '.' as ::core::ffi::c_int
                                                    || (*IObuff.ptr())
                                                        [(i - 2 as ::core::ffi::c_int) as usize]
                                                        as ::core::ffi::c_int
                                                        == '?' as ::core::ffi::c_int
                                                    || (*IObuff.ptr())
                                                        [(i - 2 as ::core::ffi::c_int) as usize]
                                                        as ::core::ffi::c_int
                                                        == '!' as ::core::ffi::c_int)
                                            {
                                                let c2rust_fresh10 = i;
                                                i = i + 1;
                                                (*IObuff.ptr())[c2rust_fresh10 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                            }
                                        }
                                        if p.offset_from(aux) >= (IOSIZE - i) as isize {
                                            p = aux
                                                .offset(IOSIZE as isize)
                                                .offset(-(i as isize))
                                                .offset(-(1 as ::core::ffi::c_int as isize));
                                        }
                                        strncpy(
                                            (IObuff.ptr() as *mut ::core::ffi::c_char)
                                                .offset(i as isize),
                                            aux,
                                            p.offset_from(aux) as size_t,
                                        );
                                        i += p.offset_from(aux) as ::core::ffi::c_int;
                                        cont_s_ipos = true_0 != 0;
                                    }
                                    (*IObuff.ptr())[i as usize] = NUL as ::core::ffi::c_char;
                                    aux = IObuff.ptr() as *mut ::core::ffi::c_char;
                                    if i == ins_compl_len() {
                                        break '_exit_matched;
                                    }
                                }
                                let add_r: ::core::ffi::c_int = ins_compl_add_infercase(
                                    aux,
                                    i,
                                    p_ic.get() != 0,
                                    if curr_fname == (*curbuf.get()).b_fname {
                                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                                    } else {
                                        curr_fname
                                    },
                                    dir,
                                    cont_s_ipos,
                                    0 as ::core::ffi::c_int,
                                );
                                if add_r == OK {
                                    dir = FORWARD;
                                } else if add_r == FAIL {
                                    break 's_1511;
                                }
                            } else if action == ACTION_SHOW_ALL as ::core::ffi::c_int {
                                found = true_0 != 0;
                                if !did_show {
                                    gotocmdline(true_0 != 0);
                                }
                                if curr_fname != prev_fname {
                                    if did_show {
                                        msg_putchar('\n' as ::core::ffi::c_int);
                                    }
                                    if !got_int.get() {
                                        msg_home_replace(curr_fname);
                                    }
                                    prev_fname = curr_fname;
                                }
                                did_show = true_0 != 0;
                                if !got_int.get() {
                                    let c2rust_fresh11 = match_count;
                                    match_count = match_count + 1;
                                    show_pat_in_path(
                                        line,
                                        type_0,
                                        true_0 != 0,
                                        action,
                                        if depth == -1 as ::core::ffi::c_int {
                                            ::core::ptr::null_mut::<FILE>()
                                        } else {
                                            (*files.offset(depth as isize)).fp
                                        },
                                        if depth == -1 as ::core::ffi::c_int {
                                            &raw mut lnum
                                        } else {
                                            &raw mut (*files.offset(depth as isize)).lnum
                                        },
                                        c2rust_fresh11,
                                    );
                                }
                                i = 0 as ::core::ffi::c_int;
                                while i <= depth {
                                    (*files.offset(i as isize)).matched = true_0;
                                    i += 1;
                                }
                            } else {
                                count -= 1;
                                if count <= 0 as ::core::ffi::c_int {
                                    found = true_0 != 0;
                                    if depth == -1 as ::core::ffi::c_int
                                        && lnum == (*curwin.get()).w_cursor.lnum
                                        && l_g_do_tagpreview == 0 as ::core::ffi::c_int
                                    {
                                        emsg(gettext(b"E387: Match is on current line\0".as_ptr()
                                            as *const ::core::ffi::c_char));
                                    } else if action == ACTION_SHOW as ::core::ffi::c_int {
                                        show_pat_in_path(
                                            line,
                                            type_0,
                                            did_show,
                                            action,
                                            if depth == -1 as ::core::ffi::c_int {
                                                ::core::ptr::null_mut::<FILE>()
                                            } else {
                                                (*files.offset(depth as isize)).fp
                                            },
                                            if depth == -1 as ::core::ffi::c_int {
                                                &raw mut lnum
                                            } else {
                                                &raw mut (*files.offset(depth as isize)).lnum
                                            },
                                            1 as ::core::ffi::c_int,
                                        );
                                        did_show = true_0 != 0;
                                    } else {
                                        if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                                            curwin_save = curwin.get();
                                            prepare_tagpreview(true_0 != 0);
                                        }
                                        if action == ACTION_SPLIT as ::core::ffi::c_int {
                                            if win_split(
                                                0 as ::core::ffi::c_int,
                                                0 as ::core::ffi::c_int,
                                            ) == FAIL
                                            {
                                                break 's_1511;
                                            }
                                            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                                            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                                        }
                                        if depth == -1 as ::core::ffi::c_int {
                                            if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                                                if !win_valid(curwin_save) {
                                                    break 's_1511;
                                                }
                                                if !(getfile(
                                                    (*(*curwin_save).w_buffer).handle
                                                        as ::core::ffi::c_int,
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                    true,
                                                    lnum,
                                                    forceit,
                                                ) <= 0 as ::core::ffi::c_int)
                                                {
                                                    break 's_1511;
                                                }
                                            } else {
                                                setpcmark();
                                            }
                                            (*curwin.get()).w_cursor.lnum = lnum;
                                            check_cursor(curwin.get());
                                        } else {
                                            if !(getfile(
                                                0 as ::core::ffi::c_int,
                                                (*files.offset(depth as isize)).name,
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                true,
                                                (*files.offset(depth as isize)).lnum,
                                                forceit,
                                            ) <= 0 as ::core::ffi::c_int)
                                            {
                                                break 's_1511;
                                            }
                                            (*curwin.get()).w_cursor.lnum =
                                                (*files.offset(depth as isize)).lnum;
                                        }
                                    }
                                    if action != ACTION_SHOW as ::core::ffi::c_int {
                                        (*curwin.get()).w_cursor.col =
                                            startp.offset_from(line) as colnr_T;
                                        (*curwin.get()).w_set_curswant = true_0;
                                    }
                                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int
                                        && curwin.get() != curwin_save
                                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                                    {
                                        validate_cursor(curwin.get());
                                        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
                                        win_enter(curwin_save, true_0 != 0);
                                    }
                                    break 's_1511;
                                }
                            }
                        }
                        matched = false_0 != 0;
                        if def_regmatch.regprog.is_null()
                            && action == ACTION_EXPAND as ::core::ffi::c_int
                            && !compl_status_sol()
                            && *startp as ::core::ffi::c_int != NUL
                            && *startp.offset(utfc_ptr2len(startp) as isize) as ::core::ffi::c_int
                                != NUL
                        {
                            c2rust_current_block = 2704434626355109080;
                        } else {
                            break;
                        }
                    }
                }
            }
            line_breakcheck();
            if action == ACTION_EXPAND as ::core::ffi::c_int {
                ins_compl_check_keys(30 as ::core::ffi::c_int, false_0 != 0);
            }
            if got_int.get() as ::core::ffi::c_int != 0
                || ins_compl_interrupted() as ::core::ffi::c_int != 0
            {
                break;
            }
            while depth >= 0 as ::core::ffi::c_int && already.is_null() && {
                line = file_line;
                vim_fgets(
                    line,
                    LSIZE as ::core::ffi::c_int,
                    (*files.offset(depth as isize)).fp,
                ) as ::core::ffi::c_int
                    != 0
            } {
                fclose((*files.offset(depth as isize)).fp);
                old_files -= 1;
                (*files.offset(old_files as isize)).name = (*files.offset(depth as isize)).name;
                (*files.offset(old_files as isize)).matched =
                    (*files.offset(depth as isize)).matched;
                depth -= 1;
                curr_fname = if depth == -1 as ::core::ffi::c_int {
                    (*curbuf.get()).b_fname
                } else {
                    (*files.offset(depth as isize)).name
                };
                depth_displayed = if depth_displayed < depth {
                    depth_displayed
                } else {
                    depth
                };
            }
            if depth >= 0 as ::core::ffi::c_int {
                (*files.offset(depth as isize)).lnum += 1;
                i = strlen(line) as ::core::ffi::c_int;
                if i > 0 as ::core::ffi::c_int
                    && *line.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    i -= 1;
                    *line.offset(i as isize) = NUL as ::core::ffi::c_char;
                }
                if i > 0 as ::core::ffi::c_int
                    && *line.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '\r' as ::core::ffi::c_int
                {
                    i -= 1;
                    *line.offset(i as isize) = NUL as ::core::ffi::c_char;
                }
            } else if already.is_null() {
                lnum += 1;
                if lnum > end_lnum {
                    break;
                }
                line = get_line_and_copy(lnum, file_line);
            }
            already = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        i = 0 as ::core::ffi::c_int;
        while i <= depth {
            fclose((*files.offset(i as isize)).fp);
            xfree((*files.offset(i as isize)).name as *mut ::core::ffi::c_void);
            i += 1;
        }
        i = old_files;
        while i < max_path_depth {
            xfree((*files.offset(i as isize)).name as *mut ::core::ffi::c_void);
            i += 1;
        }
        xfree(files as *mut ::core::ffi::c_void);
        if type_0 == CHECK_PATH as ::core::ffi::c_int {
            if !did_show {
                if action != ACTION_SHOW_ALL as ::core::ffi::c_int {
                    msg(
                        gettext(b"All included files were found\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        0 as ::core::ffi::c_int,
                    );
                } else {
                    msg(
                        gettext(b"No included files\0".as_ptr() as *const ::core::ffi::c_char),
                        0 as ::core::ffi::c_int,
                    );
                }
            }
        } else if !found && action != ACTION_EXPAND as ::core::ffi::c_int && !silent {
            if got_int.get() as ::core::ffi::c_int != 0
                || ins_compl_interrupted() as ::core::ffi::c_int != 0
            {
                emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
            } else if type_0 == FIND_DEFINE as ::core::ffi::c_int {
                emsg(gettext(
                    b"E388: Couldn't find definition\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                emsg(gettext(
                    b"E389: Couldn't find pattern\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
        if action == ACTION_SHOW as ::core::ffi::c_int
            || action == ACTION_SHOW_ALL as ::core::ffi::c_int
        {
            msg_end();
        }
    }
    xfree(file_line as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    vim_regfree(incl_regmatch.regprog);
    vim_regfree(def_regmatch.regprog);
}
unsafe extern "C" fn show_pat_in_path(
    mut line: *mut ::core::ffi::c_char,
    mut type_0: ::core::ffi::c_int,
    mut did_show: bool,
    mut action: ::core::ffi::c_int,
    mut fp: *mut FILE,
    mut lnum: *mut linenr_T,
    mut count: ::core::ffi::c_int,
) {
    if did_show {
        msg_putchar('\n' as ::core::ffi::c_int);
    } else if msg_silent.get() == 0 {
        gotocmdline(true_0 != 0);
    }
    if got_int.get() {
        return;
    }
    let mut linelen: size_t = strlen(line);
    loop {
        let mut p: *mut ::core::ffi::c_char = line
            .offset(linelen as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if !fp.is_null() {
            if p >= line && *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                p = p.offset(-1);
            }
            if p >= line && *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                p = p.offset(-1);
            }
            *p.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        if action == ACTION_SHOW_ALL as ::core::ffi::c_int {
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%3d: \0".as_ptr() as *const ::core::ffi::c_char,
                count,
            );
            msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%4d\0".as_ptr() as *const ::core::ffi::c_char,
                *lnum,
            );
            msg_puts_hl(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                HLF_N as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_prt_line(line, false_0 != 0);
        if got_int.get() as ::core::ffi::c_int != 0
            || type_0 != FIND_DEFINE as ::core::ffi::c_int
            || p < line
            || *p as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
        {
            break;
        }
        if !fp.is_null() {
            if vim_fgets(line, LSIZE as ::core::ffi::c_int, fp) {
                break;
            }
            linelen = strlen(line);
            *lnum += 1;
        } else {
            *lnum += 1;
            if *lnum > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            line = ml_get(*lnum);
            linelen = ml_get_len(*lnum) as size_t;
        }
        msg_putchar('\n' as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_search_pattern(pat: *mut SearchPattern) {
    memcpy(
        pat as *mut ::core::ffi::c_void,
        (spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<SearchPattern>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_substitute_pattern(pat: *mut SearchPattern) {
    memcpy(
        pat as *mut ::core::ffi::c_void,
        (spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<SearchPattern>(),
    );
    memset(
        &raw mut (*pat).off as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<SearchOffset>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn set_search_pattern(pat: SearchPattern) {
    free_spat((spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize));
    memcpy(
        (spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        &raw const pat as *const ::core::ffi::c_void,
        ::core::mem::size_of::<SearchPattern>(),
    );
    set_vv_searchforward();
}
#[no_mangle]
pub unsafe extern "C" fn set_substitute_pattern(pat: SearchPattern) {
    free_spat((spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize));
    memcpy(
        (spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        &raw const pat as *const ::core::ffi::c_void,
        ::core::mem::size_of::<SearchPattern>(),
    );
    memset(
        &raw mut (*(spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize)).off
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<SearchOffset>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn set_last_used_pattern(is_substitute_pattern: bool) {
    last_idx.set(if is_substitute_pattern as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    });
}
#[no_mangle]
pub unsafe extern "C" fn search_was_last_used() -> bool {
    return last_idx.get() == 0 as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
