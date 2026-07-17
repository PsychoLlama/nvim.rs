extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn parse_virt_text(chunks: Array, err: *mut Error, width: *mut ::core::ffi::c_int) -> VirtText;
    fn api_free_object(value: Object);
    fn api_clear_error(value: *mut Error);
    fn buf_updates_send_changes(
        buf: *mut buf_T,
        firstline: linenr_T,
        num_added: int64_t,
        num_removed: int64_t,
    );
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    static mut p_fcl: *mut ::core::ffi::c_char;
    static mut p_sel: *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn transstr(s: *const ::core::ffi::c_char, untab: bool) -> *mut ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn check_cursor_col(win: *mut win_T);
    fn clear_virttext(text: *mut VirtText);
    fn next_virt_text_chunk(
        vt: VirtText,
        pos: *mut size_t,
        attr: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static mut diff_context: ::core::ffi::c_int;
    static mut need_diff_redraw: bool;
    fn diff_infold(wp: *mut win_T, lnum: linenr_T) -> bool;
    fn diff_lnum_win(lnum: linenr_T, wp: *mut win_T) -> linenr_T;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn redraw_win_range_later(wp: *mut win_T, first: linenr_T, last: linenr_T);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    static e_modifiable: [::core::ffi::c_char; 0];
    fn eval_foldexpr(wp: *mut win_T, cp: *mut ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn eval_foldtext(wp: *mut win_T) -> Object;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn tv_get_lnum(tv: *const typval_T) -> linenr_T;
    fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn put_eol(fd: *mut FILE) -> ::core::ffi::c_int;
    fn put_line(fd: *mut FILE, s: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn extmark_splice_cols(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        old_col: colnr_T,
        new_col: colnr_T,
        undo: ExtmarkOp,
    );
    static mut disable_fold_update: ::core::ffi::c_int;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static mut emsg_off: ::core::ffi::c_int;
    static mut did_emsg: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut State: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut got_int: bool;
    fn get_sw_value(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn get_indent_buf(buf: *mut buf_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn setpcmark();
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_adjust_cursor();
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_replace_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
        noalloc: bool,
    ) -> ::core::ffi::c_int;
    fn changed_window_setting(wp: *mut win_T);
    fn skip_comment(
        line: *mut ::core::ffi::c_char,
        process: bool,
        include_space: bool,
        is_comment: *mut bool,
    ) -> *mut ::core::ffi::c_char;
    fn line_breakcheck();
    fn plines_win_nofold(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn linewhite(lnum: linenr_T) -> bool;
    fn syn_get_foldlevel(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type time_t = __time_t;
pub type ptrdiff_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type ApiDispatchWrapper =
    Option<unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub boolean: Boolean,
    pub integer: Integer,
    pub floating: Float,
    pub string: String_0,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
}
pub type KeyValuePair = key_value_pair;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Float = ::core::ffi::c_double;
pub type Integer = int64_t;
pub type Boolean = bool;
pub type ObjectType = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type OptInt = int64_t;
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
    pub b_wininfo: C2Rust_Unnamed_12,
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
    pub b_signcols: C2Rust_Unnamed_3,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_1,
    pub update_callbacks: C2Rust_Unnamed_0,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint32_t_uint32_t {
    pub set: Set_uint32_t,
    pub values: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint32_t {
    pub h: MapHash,
    pub keys: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MapHash {
    pub n_buckets: uint32_t,
    pub size: uint32_t,
    pub n_occupied: uint32_t,
    pub upper_bound: uint32_t,
    pub n_keys: uint32_t,
    pub keys_capacity: uint32_t,
    pub hash: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTree {
    pub root: *mut MTNode,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub n_nodes: size_t,
    pub id2node: [Map_uint64_t_ptr_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_ptr_t {
    pub set: Set_uint64_t,
    pub values: *mut ptr_t,
}
pub type ptr_t = *mut ::core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
pub type MTNode = mtnode_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_s {
    pub n: int32_t,
    pub level: int16_t,
    pub p_idx: int16_t,
    pub intersect: Intersection,
    pub parent: *mut MTNode,
    pub key: [MTKey; 19],
    pub s: [mtnode_inner_s; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_inner_s {
    pub i_ptr: [*mut MTNode; 20],
    pub i_meta: [[uint32_t; 5]; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: uint16_t,
    pub decor_data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorInlineData {
    pub hl: DecorHighlightInline,
    pub ext: DecorExt,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: C2Rust_Unnamed_2,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub virt_text: VirtText,
    pub virt_lines: VirtLines,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtTextChunk {
    pub text: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type DecorPriority = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Intersection {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
    pub init_array: [uint64_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synblock_T {
    pub b_keywtab: hashtab_T,
    pub b_keywtab_ic: hashtab_T,
    pub b_syn_error: bool,
    pub b_syn_slow: bool,
    pub b_syn_ic: ::core::ffi::c_int,
    pub b_syn_foldlevel: ::core::ffi::c_int,
    pub b_syn_spell: ::core::ffi::c_int,
    pub b_syn_patterns: garray_T,
    pub b_syn_clusters: garray_T,
    pub b_spell_cluster_id: ::core::ffi::c_int,
    pub b_nospell_cluster_id: ::core::ffi::c_int,
    pub b_syn_containedin: ::core::ffi::c_int,
    pub b_syn_sync_flags: ::core::ffi::c_int,
    pub b_syn_sync_id: int16_t,
    pub b_syn_sync_minlines: linenr_T,
    pub b_syn_sync_maxlines: linenr_T,
    pub b_syn_sync_linebreaks: linenr_T,
    pub b_syn_linecont_pat: *mut ::core::ffi::c_char,
    pub b_syn_linecont_prog: *mut regprog_T,
    pub b_syn_linecont_time: syn_time_T,
    pub b_syn_linecont_ic: ::core::ffi::c_int,
    pub b_syn_topgrp: ::core::ffi::c_int,
    pub b_syn_conceal: ::core::ffi::c_int,
    pub b_syn_folditems: ::core::ffi::c_int,
    pub b_sst_array: *mut synstate_T,
    pub b_sst_len: ::core::ffi::c_int,
    pub b_sst_first: *mut synstate_T,
    pub b_sst_firstfree: *mut synstate_T,
    pub b_sst_freecount: ::core::ffi::c_int,
    pub b_sst_check_lnum: linenr_T,
    pub b_sst_lasttick: disptick_T,
    pub b_langp: garray_T,
    pub b_spell_ismw: [bool; 256],
    pub b_spell_ismw_mb: *mut ::core::ffi::c_char,
    pub b_p_spc: *mut ::core::ffi::c_char,
    pub b_cap_prog: *mut regprog_T,
    pub b_p_spf: *mut ::core::ffi::c_char,
    pub b_p_spl: *mut ::core::ffi::c_char,
    pub b_p_spo: *mut ::core::ffi::c_char,
    pub b_p_spo_flags: ::core::ffi::c_uint,
    pub b_cjk: ::core::ffi::c_int,
    pub b_syn_chartab: [uint8_t; 32],
    pub b_syn_isk: *mut ::core::ffi::c_char,
}
pub type regprog_T = regprog;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type disptick_T = uint64_t;
pub type linenr_T = int32_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_4,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub sst_stack: [bufstate_T; 7],
    pub sst_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufstate_T {
    pub bs_idx: ::core::ffi::c_int,
    pub bs_flags: ::core::ffi::c_int,
    pub bs_seqnr: ::core::ffi::c_int,
    pub bs_cchar: ::core::ffi::c_int,
    pub bs_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg_extmatch_T {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_time_T {
    pub total: proftime_T,
    pub slowest: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmark_T {
    pub mark: pos_T,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: fmarkv_T,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmarkv_T {
    pub topline_offset: linenr_T,
    pub skipcol: colnr_T,
}
pub type colnr_T = ::core::ffi::c_int;
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_5,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
}
pub type partial_T = partial_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct partial_S {
    pub pt_refcount: ::core::ffi::c_int,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
pub type QUEUE = queue;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLockStatus,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: ::core::ffi::c_int,
    pub bv_lock: VarLockStatus,
}
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: ::core::ffi::c_int,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLockStatus,
    pub lua_table_ref: LuaRef,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type SpecialVarValue = ::core::ffi::c_uint;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type BoolVarValue = ::core::ffi::c_uint;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type varnumber_T = int64_t;
pub type VarType = ::core::ffi::c_uint;
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
pub type ufunc_T = ufunc_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: ::core::ffi::c_int,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [C2Rust_Unnamed_6; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: ::core::ffi::c_int,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScopeDictDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_6 {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
pub type scid_T = ::core::ffi::c_int;
pub type u_header_T = u_header;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_header {
    pub uh_next: C2Rust_Unnamed_11,
    pub uh_prev: C2Rust_Unnamed_10,
    pub uh_alt_next: C2Rust_Unnamed_9,
    pub uh_alt_prev: C2Rust_Unnamed_8,
    pub uh_seq: ::core::ffi::c_int,
    pub uh_walk: ::core::ffi::c_int,
    pub uh_entry: *mut u_entry_T,
    pub uh_getbot_entry: *mut u_entry_T,
    pub uh_cursor: pos_T,
    pub uh_cursor_vcol: colnr_T,
    pub uh_flags: ::core::ffi::c_int,
    pub uh_namedm: [fmark_T; 26],
    pub uh_extmark: extmark_undo_vec_t,
    pub uh_visual: visualinfo_T,
    pub uh_time: time_t,
    pub uh_save_nr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visualinfo_T {
    pub vi_start: pos_T,
    pub vi_end: pos_T,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extmark_undo_vec_t {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExtmarkUndoObject,
}
pub type ExtmarkUndoObject = undo_object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct undo_object {
    pub type_0: UndoObjectType,
    pub data: C2Rust_Unnamed_7,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub splice: ExtmarkSplice,
    pub move_0: ExtmarkMove,
    pub savepos: ExtmarkSavePos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSavePos {
    pub mark: uint64_t,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub invalidated: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkMove {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub extent_row: ::core::ffi::c_int,
    pub extent_col: ::core::ffi::c_int,
    pub new_row: ::core::ffi::c_int,
    pub new_col: ::core::ffi::c_int,
    pub start_byte: bcount_t,
    pub extent_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type bcount_t = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSplice {
    pub start_row: ::core::ffi::c_int,
    pub start_col: colnr_T,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub new_row: ::core::ffi::c_int,
    pub new_col: colnr_T,
    pub start_byte: bcount_t,
    pub old_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type UndoObjectType = ::core::ffi::c_uint;
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub type u_entry_T = u_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_entry {
    pub ue_next: *mut u_entry_T,
    pub ue_top: linenr_T,
    pub ue_bot: linenr_T,
    pub ue_lcount: linenr_T,
    pub ue_array: *mut *mut ::core::ffi::c_char,
    pub ue_size: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
pub type mapblock_T = mapblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mapblock {
    pub m_next: *mut mapblock_T,
    pub m_alt: *mut mapblock_T,
    pub m_keys: *mut ::core::ffi::c_char,
    pub m_str: *mut ::core::ffi::c_char,
    pub m_orig_str: *mut ::core::ffi::c_char,
    pub m_luaref: LuaRef,
    pub m_keylen: ::core::ffi::c_int,
    pub m_mode: ::core::ffi::c_int,
    pub m_simplified: ::core::ffi::c_int,
    pub m_noremap: ::core::ffi::c_int,
    pub m_silent: ::core::ffi::c_char,
    pub m_nowait: ::core::ffi::c_char,
    pub m_expr: ::core::ffi::c_char,
    pub m_script_ctx: sctx_T,
    pub m_desc: *mut ::core::ffi::c_char,
    pub m_replace_keycodes: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_12 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winopt_T {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: *mut ::core::ffi::c_char,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: *mut ::core::ffi::c_char,
    pub wo_eiw: *mut ::core::ffi::c_char,
    pub wo_fdc_save: *mut ::core::ffi::c_char,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: *mut ::core::ffi::c_char,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: *mut ::core::ffi::c_char,
    pub wo_fdm_save: *mut ::core::ffi::c_char,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: *mut ::core::ffi::c_char,
    pub wo_fdt: *mut ::core::ffi::c_char,
    pub wo_fmr: *mut ::core::ffi::c_char,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: *mut ::core::ffi::c_char,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: *mut ::core::ffi::c_char,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: *mut ::core::ffi::c_char,
    pub wo_cc: *mut ::core::ffi::c_char,
    pub wo_sbr: *mut ::core::ffi::c_char,
    pub wo_stc: *mut ::core::ffi::c_char,
    pub wo_stl: *mut ::core::ffi::c_char,
    pub wo_wbr: *mut ::core::ffi::c_char,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: *mut ::core::ffi::c_char,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: *mut ::core::ffi::c_char,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: *mut ::core::ffi::c_char,
    pub wo_lcs: *mut ::core::ffi::c_char,
    pub wo_fcs: *mut ::core::ffi::c_char,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [sctx_T; 51],
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickDefinition {
    pub type_0: C2Rust_Unnamed_13,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub type qf_info_T = qf_info_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinConfig {
    pub window: Window,
    pub bufpos: lpos_T,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub row: ::core::ffi::c_double,
    pub col: ::core::ffi::c_double,
    pub anchor: FloatAnchor,
    pub relative: FloatRelative,
    pub external: bool,
    pub focusable: bool,
    pub mouse: bool,
    pub split: WinSplit,
    pub zindex: ::core::ffi::c_int,
    pub style: WinStyle,
    pub border: bool,
    pub shadow: bool,
    pub border_chars: [[::core::ffi::c_char; 32]; 8],
    pub border_hl_ids: [::core::ffi::c_int; 8],
    pub border_attr: [::core::ffi::c_int; 8],
    pub title: bool,
    pub title_pos: AlignTextPos,
    pub title_chunks: VirtText,
    pub title_width: ::core::ffi::c_int,
    pub footer: bool,
    pub footer_pos: AlignTextPos,
    pub footer_chunks: VirtText,
    pub footer_width: ::core::ffi::c_int,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub _cmdline_offset: ::core::ffi::c_int,
}
pub type AlignTextPos = ::core::ffi::c_uint;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub type WinStyle = ::core::ffi::c_uint;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub type WinSplit = ::core::ffi::c_uint;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub type FloatRelative = ::core::ffi::c_uint;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type FloatAnchor = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenGrid {
    pub handle: handle_T,
    pub chars: *mut schar_T,
    pub attrs: *mut sattr_T,
    pub vcols: *mut colnr_T,
    pub line_offset: *mut size_t,
    pub dirty_col: *mut ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: ::core::ffi::c_int,
    pub comp_row: ::core::ffi::c_int,
    pub comp_col: ::core::ffi::c_int,
    pub comp_width: ::core::ffi::c_int,
    pub comp_height: ::core::ffi::c_int,
    pub comp_index: size_t,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridView {
    pub target: *mut ScreenGrid,
    pub row_offset: ::core::ffi::c_int,
    pub col_offset: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct taggy_T {
    pub tagname: *mut ::core::ffi::c_char,
    pub fmark: fmark_T,
    pub cur_match: ::core::ffi::c_int,
    pub cur_fnum: ::core::ffi::c_int,
    pub user_data: *mut ::core::ffi::c_char,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmmatch_T {
    pub regprog: *mut regprog_T,
    pub startpos: [lpos_T; 10],
    pub endpos: [lpos_T; 10],
    pub rmm_matchcol: colnr_T,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct llpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct xfmark_T {
    pub fmark: fmark_T,
    pub fname: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wline_T {
    pub wl_lnum: linenr_T,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: linenr_T,
    pub wl_lastlnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_save_T {
    pub w_topline_save: ::core::ffi::c_int,
    pub w_topline_corr: ::core::ffi::c_int,
    pub w_cursor_save: pos_T,
    pub w_cursor_corr: pos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fcs_chars_T {
    pub stl: schar_T,
    pub stlnc: schar_T,
    pub wbr: schar_T,
    pub horiz: schar_T,
    pub horizup: schar_T,
    pub horizdown: schar_T,
    pub vert: schar_T,
    pub vertleft: schar_T,
    pub vertright: schar_T,
    pub verthoriz: schar_T,
    pub fold: schar_T,
    pub foldopen: schar_T,
    pub foldclosed: schar_T,
    pub foldsep: schar_T,
    pub foldinner: schar_T,
    pub diff: schar_T,
    pub msgsep: schar_T,
    pub eob: schar_T,
    pub lastline: schar_T,
    pub trunc: schar_T,
    pub truncrl: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lcs_chars_T {
    pub eol: schar_T,
    pub ext: schar_T,
    pub prec: schar_T,
    pub nbsp: schar_T,
    pub space: schar_T,
    pub tab1: schar_T,
    pub tab2: schar_T,
    pub tab3: schar_T,
    pub leadtab1: schar_T,
    pub leadtab2: schar_T,
    pub leadtab3: schar_T,
    pub lead: schar_T,
    pub trail: schar_T,
    pub multispace: *mut schar_T,
    pub leadmultispace: *mut schar_T,
    pub conceal: schar_T,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memline_T {
    pub ml_line_count: linenr_T,
    pub ml_mfp: *mut memfile_T,
    pub ml_stack: *mut infoptr_T,
    pub ml_stack_top: ::core::ffi::c_int,
    pub ml_stack_size: ::core::ffi::c_int,
    pub ml_flags: ::core::ffi::c_int,
    pub ml_line_textlen: colnr_T,
    pub ml_line_lnum: linenr_T,
    pub ml_line_ptr: *mut ::core::ffi::c_char,
    pub ml_line_offset: size_t,
    pub ml_line_offset_ff: ::core::ffi::c_int,
    pub ml_locked: *mut bhdr_T,
    pub ml_locked_low: linenr_T,
    pub ml_locked_high: linenr_T,
    pub ml_locked_lineadd: ::core::ffi::c_int,
    pub ml_chunksize: *mut chunksize_T,
    pub ml_numchunks: ::core::ffi::c_int,
    pub ml_usedchunks: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chunksize_T {
    pub mlcs_numlines: ::core::ffi::c_int,
    pub mlcs_totalsize: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bhdr_T {
    pub bh_bnum: blocknr_T,
    pub bh_data: *mut ::core::ffi::c_void,
    pub bh_page_count: ::core::ffi::c_uint,
    pub bh_flags: ::core::ffi::c_uint,
}
pub type blocknr_T = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct infoptr_T {
    pub ip_bnum: blocknr_T,
    pub ip_low: linenr_T,
    pub ip_high: linenr_T,
    pub ip_index: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memfile_T {
    pub mf_fname: *mut ::core::ffi::c_char,
    pub mf_ffname: *mut ::core::ffi::c_char,
    pub mf_fd: ::core::ffi::c_int,
    pub mf_flags: ::core::ffi::c_int,
    pub mf_reopen: bool,
    pub mf_free_first: *mut bhdr_T,
    pub mf_hash: Map_int64_t_ptr_t,
    pub mf_trans: Map_int64_t_int64_t,
    pub mf_blocknr_max: blocknr_T,
    pub mf_blocknr_min: blocknr_T,
    pub mf_neg_count: blocknr_T,
    pub mf_infile_count: blocknr_T,
    pub mf_page_size: ::core::ffi::c_uint,
    pub mf_dirty: mfdirty_T,
}
pub type mfdirty_T = ::core::ffi::c_uint;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_int64_t {
    pub set: Set_int64_t,
    pub values: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int64_t {
    pub h: MapHash,
    pub keys: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_ptr_t {
    pub set: Set_int64_t,
    pub values: *mut ptr_t,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_15 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_15 = 3;
pub const BACKWARD: C2Rust_Unnamed_15 = -1;
pub const FORWARD: C2Rust_Unnamed_15 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_15 = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const kWinOptWrap: C2Rust_Unnamed_16 = 50;
pub const kWinOptWinhighlight: C2Rust_Unnamed_16 = 49;
pub const kWinOptWinfixwidth: C2Rust_Unnamed_16 = 48;
pub const kWinOptWinfixheight: C2Rust_Unnamed_16 = 47;
pub const kWinOptWinfixbuf: C2Rust_Unnamed_16 = 46;
pub const kWinOptWinblend: C2Rust_Unnamed_16 = 45;
pub const kWinOptWinbar: C2Rust_Unnamed_16 = 44;
pub const kWinOptVirtualedit: C2Rust_Unnamed_16 = 43;
pub const kWinOptStatusline: C2Rust_Unnamed_16 = 42;
pub const kWinOptStatuscolumn: C2Rust_Unnamed_16 = 41;
pub const kWinOptSpell: C2Rust_Unnamed_16 = 40;
pub const kWinOptSmoothscroll: C2Rust_Unnamed_16 = 39;
pub const kWinOptSigncolumn: C2Rust_Unnamed_16 = 38;
pub const kWinOptSidescrolloff: C2Rust_Unnamed_16 = 37;
pub const kWinOptShowbreak: C2Rust_Unnamed_16 = 36;
pub const kWinOptScrolloff: C2Rust_Unnamed_16 = 35;
pub const kWinOptScrollbind: C2Rust_Unnamed_16 = 34;
pub const kWinOptScroll: C2Rust_Unnamed_16 = 33;
pub const kWinOptRightleftcmd: C2Rust_Unnamed_16 = 32;
pub const kWinOptRightleft: C2Rust_Unnamed_16 = 31;
pub const kWinOptRelativenumber: C2Rust_Unnamed_16 = 30;
pub const kWinOptPreviewwindow: C2Rust_Unnamed_16 = 29;
pub const kWinOptNumberwidth: C2Rust_Unnamed_16 = 28;
pub const kWinOptNumber: C2Rust_Unnamed_16 = 27;
pub const kWinOptListchars: C2Rust_Unnamed_16 = 26;
pub const kWinOptList: C2Rust_Unnamed_16 = 25;
pub const kWinOptLinebreak: C2Rust_Unnamed_16 = 24;
pub const kWinOptLhistory: C2Rust_Unnamed_16 = 23;
pub const kWinOptFoldtext: C2Rust_Unnamed_16 = 22;
pub const kWinOptFoldnestmax: C2Rust_Unnamed_16 = 21;
pub const kWinOptFoldminlines: C2Rust_Unnamed_16 = 20;
pub const kWinOptFoldmethod: C2Rust_Unnamed_16 = 19;
pub const kWinOptFoldmarker: C2Rust_Unnamed_16 = 18;
pub const kWinOptFoldlevel: C2Rust_Unnamed_16 = 17;
pub const kWinOptFoldignore: C2Rust_Unnamed_16 = 16;
pub const kWinOptFoldexpr: C2Rust_Unnamed_16 = 15;
pub const kWinOptFoldenable: C2Rust_Unnamed_16 = 14;
pub const kWinOptFoldcolumn: C2Rust_Unnamed_16 = 13;
pub const kWinOptFillchars: C2Rust_Unnamed_16 = 12;
pub const kWinOptEventignorewin: C2Rust_Unnamed_16 = 11;
pub const kWinOptDiff: C2Rust_Unnamed_16 = 10;
pub const kWinOptCursorlineopt: C2Rust_Unnamed_16 = 9;
pub const kWinOptCursorline: C2Rust_Unnamed_16 = 8;
pub const kWinOptCursorcolumn: C2Rust_Unnamed_16 = 7;
pub const kWinOptCursorbind: C2Rust_Unnamed_16 = 6;
pub const kWinOptConceallevel: C2Rust_Unnamed_16 = 5;
pub const kWinOptConcealcursor: C2Rust_Unnamed_16 = 4;
pub const kWinOptColorcolumn: C2Rust_Unnamed_16 = 3;
pub const kWinOptBreakindentopt: C2Rust_Unnamed_16 = 2;
pub const kWinOptBreakindent: C2Rust_Unnamed_16 = 1;
pub const kWinOptArabic: C2Rust_Unnamed_16 = 0;
pub const kWinOptInvalid: C2Rust_Unnamed_16 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
}
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const FOLD_TEXT_LEN: C2Rust_Unnamed_17 = 51;
pub type ExtmarkOp = ::core::ffi::c_uint;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffblock_S {
    pub df_next: *mut diff_T,
    pub df_lnum: [linenr_T; 8],
    pub df_count: [linenr_T; 8],
    pub is_linematched: bool,
    pub has_changes: bool,
    pub df_changes: garray_T,
}
pub type diff_T = diffblock_S;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_18 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_18 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_18 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_18 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_18 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_18 = 20;
pub const UPD_VALID: C2Rust_Unnamed_18 = 10;
pub type VimVarIndex = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fold_T {
    pub fd_top: linenr_T,
    pub fd_len: linenr_T,
    pub fd_nested: garray_T,
    pub fd_flags: ::core::ffi::c_char,
    pub fd_small: TriState,
}
pub const FD_CLOSED: C2Rust_Unnamed_20 = 1;
pub const FD_LEVEL: C2Rust_Unnamed_20 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fline_T {
    pub wp: *mut win_T,
    pub lnum: linenr_T,
    pub off: linenr_T,
    pub lnum_save: linenr_T,
    pub lvl: ::core::ffi::c_int,
    pub lvl_next: ::core::ffi::c_int,
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub had_end: ::core::ffi::c_int,
}
pub const MODE_INSERT: C2Rust_Unnamed_19 = 16;
pub type LevelGetter = Option<unsafe extern "C" fn(*mut fline_T) -> ()>;
pub const FD_OPEN: C2Rust_Unnamed_20 = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_19 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_19 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_19 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_19 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_19 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_19 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_19 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_19 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_19 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_19 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_19 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_19 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_19 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_19 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_19 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_19 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_19 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_19 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: VirtText = VirtText {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<VirtTextChunk>(),
};
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
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const VIRTTEXT_EMPTY: VirtText = KV_INITIAL_VALUE;
pub const MAX_LEVEL: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
static mut fold_changed: bool = false;
static mut e_nofold: *const ::core::ffi::c_char =
    b"E490: No fold found\0".as_ptr() as *const ::core::ffi::c_char;
static mut invalid_top: linenr_T = 0 as linenr_T;
static mut invalid_bot: linenr_T = 0 as linenr_T;
static mut prev_lnum: linenr_T = 0 as linenr_T;
static mut prev_lnum_lvl: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const DONE_NOTHING: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DONE_ACTION: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DONE_FOLD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut foldstartmarkerlen: size_t = 0;
static mut foldendmarker: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
static mut foldendmarkerlen: size_t = 0;
#[no_mangle]
pub unsafe extern "C" fn copyFoldingState(mut wp_from: *mut win_T, mut wp_to: *mut win_T) {
    (*wp_to).w_fold_manual = (*wp_from).w_fold_manual;
    (*wp_to).w_foldinvalid = (*wp_from).w_foldinvalid;
    cloneFoldGrowArray(&raw mut (*wp_from).w_folds, &raw mut (*wp_to).w_folds);
}
#[no_mangle]
pub unsafe extern "C" fn hasAnyFolding(mut win: *mut win_T) -> ::core::ffi::c_int {
    return ((*(*win).w_buffer).terminal.is_null()
        && (*win).w_onebuf_opt.wo_fen != 0
        && (!foldmethodIsManual(win) || !((*win).w_folds.ga_len <= 0 as ::core::ffi::c_int)))
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hasFolding(
    mut win: *mut win_T,
    mut lnum: linenr_T,
    mut firstp: *mut linenr_T,
    mut lastp: *mut linenr_T,
) -> bool {
    return hasFoldingWin(
        win,
        lnum,
        firstp,
        lastp,
        true_0 != 0,
        ::core::ptr::null_mut::<foldinfo_T>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn hasFoldingWin(
    win: *mut win_T,
    lnum: linenr_T,
    firstp: *mut linenr_T,
    lastp: *mut linenr_T,
    cache: bool,
    infop: *mut foldinfo_T,
) -> bool {
    checkupdate(win);
    if hasAnyFolding(win) == 0 {
        if !infop.is_null() {
            (*infop).fi_level = 0 as ::core::ffi::c_int;
        }
        return false_0 != 0;
    }
    let mut had_folded: bool = false_0 != 0;
    let mut first: linenr_T = 0 as linenr_T;
    let mut last: linenr_T = 0 as linenr_T;
    if cache {
        let x: ::core::ffi::c_int = find_wl_entry(win, lnum);
        if x >= 0 as ::core::ffi::c_int {
            first = (*(*win).w_lines.offset(x as isize)).wl_lnum;
            last = (*(*win).w_lines.offset(x as isize)).wl_foldend;
            had_folded = (*(*win).w_lines.offset(x as isize)).wl_folded;
        }
    }
    let mut lnum_rel: linenr_T = lnum;
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut low_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut maybe_small: bool = false_0 != 0;
    let mut use_level: bool = false_0 != 0;
    if first == 0 as linenr_T {
        let mut gap: *mut garray_T = &raw mut (*win).w_folds;
        while foldFind(gap, lnum_rel, &raw mut fp) {
            if lnum_rel == (*fp).fd_top && low_level == 0 as ::core::ffi::c_int {
                low_level = level + 1 as ::core::ffi::c_int;
            }
            first += (*fp).fd_top;
            last += (*fp).fd_top;
            had_folded = check_closed(
                win,
                fp,
                &raw mut use_level,
                level,
                &raw mut maybe_small,
                lnum - lnum_rel,
            );
            if had_folded {
                last = (last as ::core::ffi::c_int
                    + ((*fp).fd_len - 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                break;
            } else {
                gap = &raw mut (*fp).fd_nested;
                lnum_rel -= (*fp).fd_top;
                level += 1;
            }
        }
    }
    if !had_folded {
        if !infop.is_null() {
            (*infop).fi_level = level;
            (*infop).fi_lnum = lnum - lnum_rel;
            (*infop).fi_low_level = if low_level == 0 as ::core::ffi::c_int {
                level
            } else {
                low_level
            };
        }
        return false_0 != 0;
    }
    last = if last < (*(*win).w_buffer).b_ml.ml_line_count {
        last
    } else {
        (*(*win).w_buffer).b_ml.ml_line_count
    };
    if !lastp.is_null() {
        *lastp = last;
    }
    if !firstp.is_null() {
        *firstp = first;
    }
    if !infop.is_null() {
        (*infop).fi_level = level + 1 as ::core::ffi::c_int;
        (*infop).fi_lnum = first;
        (*infop).fi_low_level = if low_level == 0 as ::core::ffi::c_int {
            level + 1 as ::core::ffi::c_int
        } else {
            low_level
        };
    }
    return true_0 != 0;
}
unsafe extern "C" fn foldLevel(mut lnum: linenr_T) -> ::core::ffi::c_int {
    if invalid_top == 0 as linenr_T {
        checkupdate(curwin);
    } else if lnum == prev_lnum && prev_lnum_lvl >= 0 as ::core::ffi::c_int {
        return prev_lnum_lvl;
    } else if lnum >= invalid_top && lnum <= invalid_bot {
        return -1 as ::core::ffi::c_int;
    }
    if hasAnyFolding(curwin) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return foldLevelWin(curwin, lnum);
}
#[no_mangle]
pub unsafe extern "C" fn lineFolded(win: *mut win_T, lnum: linenr_T) -> bool {
    return fold_info(win, lnum).fi_lines != 0 as linenr_T;
}
#[no_mangle]
pub unsafe extern "C" fn fold_info(mut win: *mut win_T, mut lnum: linenr_T) -> foldinfo_T {
    let mut info: foldinfo_T = foldinfo_T {
        fi_lnum: 0,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    let mut last: linenr_T = 0;
    if hasFoldingWin(
        win,
        lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut last,
        false_0 != 0,
        &raw mut info,
    ) {
        info.fi_lines = last - lnum + 1 as linenr_T;
    } else {
        info.fi_lines = 0 as ::core::ffi::c_int as linenr_T;
    }
    return info;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsManual(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != NUL
        && *(*wp)
            .w_onebuf_opt
            .wo_fdm
            .offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'u' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsIndent(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'i' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsExpr(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != NUL
        && *(*wp)
            .w_onebuf_opt
            .wo_fdm
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'x' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsMarker(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != NUL
        && *(*wp)
            .w_onebuf_opt
            .wo_fdm
            .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'r' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsSyntax(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldmethodIsDiff(mut wp: *mut win_T) -> bool {
    return *(*wp)
        .w_onebuf_opt
        .wo_fdm
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'd' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn closeFold(mut pos: pos_T, mut count: ::core::ffi::c_int) {
    setFoldRepeat(pos, count, false_0);
}
#[no_mangle]
pub unsafe extern "C" fn closeFoldRecurse(mut pos: pos_T) {
    setManualFold(
        pos,
        false_0 != 0,
        true_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn opFoldRange(
    mut firstpos: pos_T,
    mut lastpos: pos_T,
    mut opening: ::core::ffi::c_int,
    mut recurse: ::core::ffi::c_int,
    mut had_visual: bool,
) {
    let mut done: ::core::ffi::c_int = DONE_NOTHING;
    let mut first: linenr_T = firstpos.lnum;
    let mut last: linenr_T = lastpos.lnum;
    let mut lnum_next: linenr_T = 0;
    let mut lnum: linenr_T = first;
    while lnum <= last {
        let mut temp: pos_T = pos_T {
            lnum: lnum,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        };
        lnum_next = lnum;
        if opening != 0 && recurse == 0 {
            hasFolding(
                curwin,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut lnum_next,
            );
        }
        setManualFold(temp, opening != 0, recurse != 0, &raw mut done);
        if opening == 0 && recurse == 0 {
            hasFolding(
                curwin,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut lnum_next,
            );
        }
        lnum = lnum_next + 1 as linenr_T;
    }
    if done == DONE_NOTHING {
        emsg(gettext(e_nofold));
    }
    if had_visual {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn openFold(mut pos: pos_T, mut count: ::core::ffi::c_int) {
    setFoldRepeat(pos, count, true_0);
}
#[no_mangle]
pub unsafe extern "C" fn openFoldRecurse(mut pos: pos_T) {
    setManualFold(
        pos,
        true_0 != 0,
        true_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn foldOpenCursor() {
    checkupdate(curwin);
    if hasAnyFolding(curwin) != 0 {
        loop {
            let mut done: ::core::ffi::c_int = DONE_NOTHING;
            setManualFold((*curwin).w_cursor, true_0 != 0, false_0 != 0, &raw mut done);
            if done & DONE_ACTION == 0 {
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn newFoldLevel() {
    newFoldLevelWin(curwin);
    if foldmethodIsDiff(curwin) as ::core::ffi::c_int != 0 && (*curwin).w_onebuf_opt.wo_scb != 0 {
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if wp != curwin
                && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
                && (*wp).w_onebuf_opt.wo_scb != 0
            {
                (*wp).w_onebuf_opt.wo_fdl = (*curwin).w_onebuf_opt.wo_fdl;
                newFoldLevelWin(wp);
            }
            wp = (*wp).w_next;
        }
    }
}
unsafe extern "C" fn newFoldLevelWin(mut wp: *mut win_T) {
    checkupdate(wp);
    if (*wp).w_fold_manual {
        let mut fp: *mut fold_T = (*wp).w_folds.ga_data as *mut fold_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*wp).w_folds.ga_len {
            (*fp.offset(i as isize)).fd_flags =
                FD_LEVEL as ::core::ffi::c_int as ::core::ffi::c_char;
            i += 1;
        }
        (*wp).w_fold_manual = false_0 != 0;
    }
    changed_window_setting(wp);
}
#[no_mangle]
pub unsafe extern "C" fn foldCheckClose() {
    if *p_fcl as ::core::ffi::c_int == NUL {
        return;
    }
    checkupdate(curwin);
    if checkCloseRec(
        &raw mut (*curwin).w_folds,
        (*curwin).w_cursor.lnum,
        (*curwin).w_onebuf_opt.wo_fdl as ::core::ffi::c_int,
    ) {
        changed_window_setting(curwin);
    }
}
unsafe extern "C" fn checkCloseRec(
    mut gap: *mut garray_T,
    mut lnum: linenr_T,
    mut level: ::core::ffi::c_int,
) -> bool {
    let mut retval: bool = false_0 != 0;
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        if (*fp.offset(i as isize)).fd_flags as ::core::ffi::c_int == FD_OPEN as ::core::ffi::c_int
        {
            if level <= 0 as ::core::ffi::c_int
                && (lnum < (*fp.offset(i as isize)).fd_top
                    || lnum >= (*fp.offset(i as isize)).fd_top + (*fp.offset(i as isize)).fd_len)
            {
                (*fp.offset(i as isize)).fd_flags =
                    FD_LEVEL as ::core::ffi::c_int as ::core::ffi::c_char;
                retval = true_0 != 0;
            } else {
                retval = retval as ::core::ffi::c_int
                    | checkCloseRec(
                        &raw mut (*fp.offset(i as isize)).fd_nested,
                        lnum - (*fp.offset(i as isize)).fd_top,
                        level - 1 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                    != 0;
            }
        }
        i += 1;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn foldManualAllowed(mut create: bool) -> ::core::ffi::c_int {
    if foldmethodIsManual(curwin) as ::core::ffi::c_int != 0
        || foldmethodIsMarker(curwin) as ::core::ffi::c_int != 0
    {
        return true_0;
    }
    if create {
        emsg(gettext(
            b"E350: Cannot create fold with current 'foldmethod'\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    } else {
        emsg(gettext(
            b"E351: Cannot delete fold with current 'foldmethod'\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    }
    return false_0;
}
#[no_mangle]
pub unsafe extern "C" fn foldCreate(mut wp: *mut win_T, mut start: pos_T, mut end: pos_T) {
    let mut use_level: bool = false_0 != 0;
    let mut closed: bool = false_0 != 0;
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start_rel: pos_T = start;
    let mut end_rel: pos_T = end;
    if start.lnum > end.lnum {
        end = start_rel;
        start = end_rel;
        start_rel = start;
        end_rel = end;
    }
    if foldmethodIsMarker(wp) {
        foldCreateMarkers(wp, start, end);
        return;
    }
    checkupdate(wp);
    let mut i: ::core::ffi::c_int = 0;
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    if (*gap).ga_len == 0 as ::core::ffi::c_int {
        i = 0 as ::core::ffi::c_int;
    } else {
        let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
        while foldFind(gap, start_rel.lnum, &raw mut fp) {
            if (*fp).fd_top + (*fp).fd_len <= end_rel.lnum {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            start_rel.lnum -= (*fp).fd_top;
            end_rel.lnum -= (*fp).fd_top;
            if use_level as ::core::ffi::c_int != 0
                || (*fp).fd_flags as ::core::ffi::c_int == FD_LEVEL as ::core::ffi::c_int
            {
                use_level = true_0 != 0;
                if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
                    closed = true_0 != 0;
                }
            } else if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int {
                closed = true_0 != 0;
            }
            level += 1;
        }
        if (*gap).ga_len == 0 as ::core::ffi::c_int {
            i = 0 as ::core::ffi::c_int;
        } else {
            i = fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
        }
    }
    ga_grow(gap, 1 as ::core::ffi::c_int);
    let mut fp_0: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(i as isize);
    let mut fold_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut fold_ga,
        ::core::mem::size_of::<fold_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut cont: ::core::ffi::c_int = 0;
    cont = 0 as ::core::ffi::c_int;
    while i + cont < (*gap).ga_len {
        if (*fp_0.offset(cont as isize)).fd_top > end_rel.lnum {
            break;
        }
        cont += 1;
    }
    if cont > 0 as ::core::ffi::c_int {
        ga_grow(&raw mut fold_ga, cont);
        start_rel.lnum = if start_rel.lnum < (*fp_0).fd_top {
            start_rel.lnum
        } else {
            (*fp_0).fd_top
        };
        end_rel.lnum = if end_rel.lnum
            > (*fp_0.offset((cont - 1 as ::core::ffi::c_int) as isize)).fd_top
                + (*fp_0.offset((cont - 1 as ::core::ffi::c_int) as isize)).fd_len
                - 1 as linenr_T
        {
            end_rel.lnum
        } else {
            (*fp_0.offset((cont - 1 as ::core::ffi::c_int) as isize)).fd_top
                + (*fp_0.offset((cont - 1 as ::core::ffi::c_int) as isize)).fd_len
                - 1 as linenr_T
        };
        memmove(
            fold_ga.ga_data,
            fp_0 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<fold_T>().wrapping_mul(cont as size_t),
        );
        fold_ga.ga_len += cont;
        i += cont;
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < cont {
            (*(fold_ga.ga_data as *mut fold_T).offset(j as isize)).fd_top -= start_rel.lnum;
            j += 1;
        }
    }
    if i < (*gap).ga_len {
        memmove(
            fp_0.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            ((*gap).ga_data as *mut fold_T).offset(i as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<fold_T>().wrapping_mul(((*gap).ga_len - i) as size_t),
        );
    }
    (*gap).ga_len = (*gap).ga_len + 1 as ::core::ffi::c_int - cont;
    (*fp_0).fd_nested = fold_ga;
    (*fp_0).fd_top = start_rel.lnum;
    (*fp_0).fd_len = end_rel.lnum - start_rel.lnum + 1 as linenr_T;
    if use_level as ::core::ffi::c_int != 0
        && !closed
        && (level as OptInt) < (*wp).w_onebuf_opt.wo_fdl
    {
        closeFold(start, 1 as ::core::ffi::c_int);
    }
    if !use_level {
        (*wp).w_fold_manual = true_0 != 0;
    }
    (*fp_0).fd_flags = FD_CLOSED as ::core::ffi::c_int as ::core::ffi::c_char;
    (*fp_0).fd_small = kNone;
    changed_window_setting(wp);
}
#[no_mangle]
pub unsafe extern "C" fn deleteFold(
    wp: *mut win_T,
    start: linenr_T,
    end: linenr_T,
    recursive: ::core::ffi::c_int,
    had_visual: bool,
) {
    let mut found_fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut found_off: linenr_T = 0 as linenr_T;
    let mut maybe_small: bool = false_0 != 0;
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lnum: linenr_T = start;
    let mut did_one: bool = false_0 != 0;
    let mut first_lnum: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
    let mut last_lnum: linenr_T = 0 as linenr_T;
    checkupdate(wp);
    while lnum <= end {
        let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
        let mut found_ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
        let mut lnum_off: linenr_T = 0 as linenr_T;
        let mut use_level: bool = false_0 != 0;
        loop {
            let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
            if !foldFind(gap, lnum - lnum_off, &raw mut fp) {
                break;
            }
            found_ga = gap;
            found_fp = fp;
            found_off = lnum_off;
            if check_closed(
                wp,
                fp,
                &raw mut use_level,
                level,
                &raw mut maybe_small,
                lnum_off,
            ) {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            lnum_off += (*fp).fd_top;
            level += 1;
        }
        if found_ga.is_null() {
            lnum += 1;
        } else {
            lnum = (*found_fp).fd_top + (*found_fp).fd_len + found_off;
            if foldmethodIsManual(wp) {
                deleteFoldEntry(
                    wp,
                    found_ga,
                    found_fp.offset_from((*found_ga).ga_data as *mut fold_T) as ::core::ffi::c_int,
                    recursive != 0,
                );
            } else {
                first_lnum = if first_lnum < (*found_fp).fd_top + found_off {
                    first_lnum
                } else {
                    (*found_fp).fd_top + found_off
                };
                last_lnum = if last_lnum > lnum { last_lnum } else { lnum };
                if !did_one {
                    parseMarker(wp);
                }
                deleteFoldMarkers(wp, found_fp, recursive != 0, found_off);
            }
            did_one = true_0 != 0;
            changed_window_setting(wp);
        }
    }
    if !did_one {
        emsg(gettext(e_nofold));
        if had_visual {
            redraw_buf_later((*wp).w_buffer, UPD_INVERTED as ::core::ffi::c_int);
        }
    } else {
        check_cursor_col(wp);
    }
    if last_lnum > 0 as linenr_T {
        changed_lines(
            (*wp).w_buffer,
            first_lnum,
            0 as colnr_T,
            last_lnum,
            0 as linenr_T,
            false_0 != 0,
        );
        let mut num_changed: int64_t = (last_lnum - first_lnum) as int64_t;
        buf_updates_send_changes((*wp).w_buffer, first_lnum, num_changed, num_changed);
    }
}
#[no_mangle]
pub unsafe extern "C" fn clearFolding(mut win: *mut win_T) {
    deleteFoldRecurse((*win).w_buffer, &raw mut (*win).w_folds);
    (*win).w_foldinvalid = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn foldUpdate(mut wp: *mut win_T, mut top: linenr_T, mut bot: linenr_T) {
    if disable_fold_update != 0
        || State & MODE_INSERT as ::core::ffi::c_int != 0 && !foldmethodIsIndent(wp)
    {
        return;
    }
    if need_diff_redraw {
        return;
    }
    if (*wp).w_folds.ga_len > 0 as ::core::ffi::c_int {
        let mut maybe_small_start: linenr_T = if top < bot { top } else { bot };
        let mut maybe_small_end: linenr_T = if top > bot { top } else { bot };
        let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
        foldFind(&raw mut (*wp).w_folds, maybe_small_start, &raw mut fp);
        while fp < ((*wp).w_folds.ga_data as *mut fold_T).offset((*wp).w_folds.ga_len as isize)
            && (*fp).fd_top <= maybe_small_end
        {
            (*fp).fd_small = kNone;
            fp = fp.offset(1);
        }
    }
    if foldmethodIsIndent(wp) as ::core::ffi::c_int != 0
        || foldmethodIsExpr(wp) as ::core::ffi::c_int != 0
        || foldmethodIsMarker(wp) as ::core::ffi::c_int != 0
        || foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
        || foldmethodIsSyntax(wp) as ::core::ffi::c_int != 0
    {
        let mut save_got_int: ::core::ffi::c_int = got_int as ::core::ffi::c_int;
        got_int = false_0 != 0;
        foldUpdateIEMS(wp, top, bot);
        got_int = got_int as ::core::ffi::c_int | save_got_int != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn foldUpdateAfterInsert() {
    if foldmethodIsManual(curwin) as ::core::ffi::c_int != 0
        || foldmethodIsSyntax(curwin) as ::core::ffi::c_int != 0
        || foldmethodIsExpr(curwin) as ::core::ffi::c_int != 0
    {
        return;
    }
    foldUpdateAll(curwin);
    foldOpenCursor();
}
#[no_mangle]
pub unsafe extern "C" fn foldUpdateAll(mut win: *mut win_T) {
    (*win).w_foldinvalid = true_0 != 0;
    redraw_later(win, UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn foldMoveTo(
    updown: bool,
    dir: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    checkupdate(curwin);
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < count {
        let mut lnum_off: linenr_T = 0 as linenr_T;
        let mut gap: *mut garray_T = &raw mut (*curwin).w_folds;
        if (*gap).ga_len == 0 as ::core::ffi::c_int {
            break;
        }
        let mut use_level: bool = false_0 != 0;
        let mut maybe_small: bool = false_0 != 0;
        let mut lnum_found: linenr_T = (*curwin).w_cursor.lnum;
        let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut last: bool = false_0 != 0;
        loop {
            if !foldFind(gap, (*curwin).w_cursor.lnum - lnum_off, &raw mut fp) {
                if !updown || (*gap).ga_len == 0 as ::core::ffi::c_int {
                    break;
                }
                if dir == FORWARD as ::core::ffi::c_int {
                    if fp.offset_from((*gap).ga_data as *mut fold_T) >= (*gap).ga_len as isize {
                        break;
                    }
                    fp = fp.offset(-1);
                } else if fp == (*gap).ga_data as *mut fold_T {
                    break;
                }
                last = true_0 != 0;
            }
            if !last {
                if check_closed(
                    curwin,
                    fp,
                    &raw mut use_level,
                    level,
                    &raw mut maybe_small,
                    lnum_off,
                ) {
                    last = true_0 != 0;
                }
                if last as ::core::ffi::c_int != 0 && !updown {
                    break;
                }
            }
            if updown {
                if dir == FORWARD as ::core::ffi::c_int {
                    if fp
                        .offset(1 as ::core::ffi::c_int as isize)
                        .offset_from((*gap).ga_data as *mut fold_T)
                        < (*gap).ga_len as isize
                    {
                        let mut lnum: linenr_T =
                            (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top + lnum_off;
                        if lnum > (*curwin).w_cursor.lnum {
                            lnum_found = lnum;
                        }
                    }
                } else if fp > (*gap).ga_data as *mut fold_T {
                    let mut lnum_0: linenr_T = (*fp.offset(-1 as ::core::ffi::c_int as isize))
                        .fd_top
                        + lnum_off
                        + (*fp.offset(-1 as ::core::ffi::c_int as isize)).fd_len
                        - 1 as linenr_T;
                    if lnum_0 < (*curwin).w_cursor.lnum {
                        lnum_found = lnum_0;
                    }
                }
            } else if dir == FORWARD as ::core::ffi::c_int {
                let mut lnum_1: linenr_T = (*fp).fd_top + lnum_off + (*fp).fd_len - 1 as linenr_T;
                if lnum_1 > (*curwin).w_cursor.lnum {
                    lnum_found = lnum_1;
                }
            } else {
                let mut lnum_2: linenr_T = (*fp).fd_top + lnum_off;
                if lnum_2 < (*curwin).w_cursor.lnum {
                    lnum_found = lnum_2;
                }
            }
            if last {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            lnum_off += (*fp).fd_top;
            level += 1;
        }
        if lnum_found == (*curwin).w_cursor.lnum {
            break;
        }
        if retval == FAIL {
            setpcmark();
        }
        (*curwin).w_cursor.lnum = lnum_found;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        retval = OK;
        n += 1;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn foldInitWin(mut new_win: *mut win_T) {
    ga_init(
        &raw mut (*new_win).w_folds,
        ::core::mem::size_of::<fold_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_wl_entry(
    mut win: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*win).w_lines_valid {
        if (*(*win).w_lines.offset(i as isize)).wl_valid {
            if lnum < (*(*win).w_lines.offset(i as isize)).wl_lnum {
                return -1 as ::core::ffi::c_int;
            }
            if lnum <= (*(*win).w_lines.offset(i as isize)).wl_foldend {
                return i;
            }
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn foldAdjustVisual() {
    if !VIsual_active || hasAnyFolding(curwin) == 0 {
        return;
    }
    let mut start: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut end: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if ltoreq(VIsual, (*curwin).w_cursor) {
        start = &raw mut VIsual;
        end = &raw mut (*curwin).w_cursor;
    } else {
        start = &raw mut (*curwin).w_cursor;
        end = &raw mut VIsual;
    }
    if hasFolding(
        curwin,
        (*start).lnum,
        &raw mut (*start).lnum,
        ::core::ptr::null_mut::<linenr_T>(),
    ) {
        (*start).col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if !hasFolding(
        curwin,
        (*end).lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut (*end).lnum,
    ) {
        return;
    }
    (*end).col = ml_get_len((*end).lnum);
    if (*end).col > 0 as ::core::ffi::c_int
        && *p_sel as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
    {
        (*end).col -= 1;
    }
    mb_adjust_cursor();
}
#[no_mangle]
pub unsafe extern "C" fn foldAdjustCursor(mut wp: *mut win_T) {
    hasFolding(
        wp,
        (*wp).w_cursor.lnum,
        &raw mut (*wp).w_cursor.lnum,
        ::core::ptr::null_mut::<linenr_T>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn cloneFoldGrowArray(mut from: *mut garray_T, mut to: *mut garray_T) {
    ga_init(to, (*from).ga_itemsize, (*from).ga_growsize);
    if (*from).ga_len <= 0 as ::core::ffi::c_int {
        return;
    }
    ga_grow(to, (*from).ga_len);
    let mut from_p: *mut fold_T = (*from).ga_data as *mut fold_T;
    let mut to_p: *mut fold_T = (*to).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*from).ga_len {
        (*to_p).fd_top = (*from_p).fd_top;
        (*to_p).fd_len = (*from_p).fd_len;
        (*to_p).fd_flags = (*from_p).fd_flags;
        (*to_p).fd_small = (*from_p).fd_small;
        cloneFoldGrowArray(&raw mut (*from_p).fd_nested, &raw mut (*to_p).fd_nested);
        (*to).ga_len += 1;
        from_p = from_p.offset(1);
        to_p = to_p.offset(1);
        i += 1;
    }
}
unsafe extern "C" fn foldFind(
    mut gap: *const garray_T,
    mut lnum: linenr_T,
    mut fpp: *mut *mut fold_T,
) -> bool {
    if (*gap).ga_len == 0 as ::core::ffi::c_int {
        *fpp = ::core::ptr::null_mut::<fold_T>();
        return false_0 != 0;
    }
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut low: linenr_T = 0 as linenr_T;
    let mut high: linenr_T = (*gap).ga_len as linenr_T - 1 as linenr_T;
    while low <= high {
        let mut i: linenr_T = (low + high) / 2 as linenr_T;
        if (*fp.offset(i as isize)).fd_top > lnum {
            high = i - 1 as linenr_T;
        } else if (*fp.offset(i as isize)).fd_top + (*fp.offset(i as isize)).fd_len <= lnum {
            low = i + 1 as linenr_T;
        } else {
            *fpp = fp.offset(i as isize);
            return true_0 != 0;
        }
    }
    *fpp = fp.offset(low as isize);
    return false_0 != 0;
}
unsafe extern "C" fn foldLevelWin(mut wp: *mut win_T, mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut lnum_rel: linenr_T = lnum;
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    while foldFind(gap, lnum_rel, &raw mut fp) {
        gap = &raw mut (*fp).fd_nested;
        lnum_rel -= (*fp).fd_top;
        level += 1;
    }
    return level;
}
unsafe extern "C" fn checkupdate(mut wp: *mut win_T) {
    if !(*wp).w_foldinvalid {
        return;
    }
    foldUpdate(wp, 1 as linenr_T, MAXLNUM as ::core::ffi::c_int as linenr_T);
    (*wp).w_foldinvalid = false_0 != 0;
}
unsafe extern "C" fn setFoldRepeat(
    mut pos: pos_T,
    mut count: ::core::ffi::c_int,
    mut do_open: ::core::ffi::c_int,
) {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < count {
        let mut done: ::core::ffi::c_int = DONE_NOTHING;
        setManualFold(pos, do_open != 0, false_0 != 0, &raw mut done);
        if done & DONE_ACTION == 0 {
            if n == 0 as ::core::ffi::c_int && done & DONE_FOLD == 0 {
                emsg(gettext(e_nofold));
            }
            break;
        } else {
            n += 1;
        }
    }
}
unsafe extern "C" fn setManualFold(
    mut pos: pos_T,
    mut opening: bool,
    mut recurse: bool,
    mut donep: *mut ::core::ffi::c_int,
) -> linenr_T {
    if foldmethodIsDiff(curwin) as ::core::ffi::c_int != 0 && (*curwin).w_onebuf_opt.wo_scb != 0 {
        let mut dlnum: linenr_T = 0;
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if wp != curwin
                && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
                && (*wp).w_onebuf_opt.wo_scb != 0
            {
                dlnum = diff_lnum_win((*curwin).w_cursor.lnum, wp);
                if dlnum != 0 as linenr_T {
                    setManualFoldWin(
                        wp,
                        dlnum,
                        opening,
                        recurse,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    );
                }
            }
            wp = (*wp).w_next;
        }
    }
    return setManualFoldWin(curwin, pos.lnum, opening, recurse, donep);
}
unsafe extern "C" fn setManualFoldWin(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut opening: bool,
    mut recurse: bool,
    mut donep: *mut ::core::ffi::c_int,
) -> linenr_T {
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut fp2: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut found: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut use_level: bool = false_0 != 0;
    let mut found_fold: bool = false_0 != 0;
    let mut next: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
    let mut off: linenr_T = 0 as linenr_T;
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    checkupdate(wp);
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    loop {
        if !foldFind(gap, lnum, &raw mut fp) {
            if !fp.is_null() && fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
            {
                next = (*fp).fd_top + off;
            }
            break;
        } else {
            found_fold = true_0 != 0;
            if fp.offset(1 as ::core::ffi::c_int as isize)
                < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
            {
                next = (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top + off;
            }
            if use_level as ::core::ffi::c_int != 0
                || (*fp).fd_flags as ::core::ffi::c_int == FD_LEVEL as ::core::ffi::c_int
            {
                use_level = true_0 != 0;
                (*fp).fd_flags = (if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
                    FD_CLOSED as ::core::ffi::c_int
                } else {
                    FD_OPEN as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                fp2 = (*fp).fd_nested.ga_data as *mut fold_T;
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < (*fp).fd_nested.ga_len {
                    (*fp2.offset(j as isize)).fd_flags =
                        FD_LEVEL as ::core::ffi::c_int as ::core::ffi::c_char;
                    j += 1;
                }
            }
            if !opening && recurse as ::core::ffi::c_int != 0 {
                if (*fp).fd_flags as ::core::ffi::c_int != FD_CLOSED as ::core::ffi::c_int {
                    done |= DONE_ACTION;
                    (*fp).fd_flags = FD_CLOSED as ::core::ffi::c_int as ::core::ffi::c_char;
                }
            } else if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int {
                if opening {
                    (*fp).fd_flags = FD_OPEN as ::core::ffi::c_int as ::core::ffi::c_char;
                    done |= DONE_ACTION;
                    if recurse {
                        foldOpenNested(fp);
                    }
                }
                break;
            }
            found = fp;
            gap = &raw mut (*fp).fd_nested;
            lnum -= (*fp).fd_top;
            off += (*fp).fd_top;
            level += 1;
        }
    }
    if found_fold {
        if !opening && !found.is_null() {
            (*found).fd_flags = FD_CLOSED as ::core::ffi::c_int as ::core::ffi::c_char;
            done |= DONE_ACTION;
        }
        (*wp).w_fold_manual = true_0 != 0;
        if done & DONE_ACTION != 0 {
            changed_window_setting(wp);
        }
        done |= DONE_FOLD;
    } else if donep.is_null() && wp == curwin {
        emsg(gettext(e_nofold));
    }
    if !donep.is_null() {
        *donep |= done;
    }
    return next;
}
unsafe extern "C" fn foldOpenNested(mut fpr: *mut fold_T) {
    let mut fp: *mut fold_T = (*fpr).fd_nested.ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*fpr).fd_nested.ga_len {
        foldOpenNested(fp.offset(i as isize));
        (*fp.offset(i as isize)).fd_flags = FD_OPEN as ::core::ffi::c_int as ::core::ffi::c_char;
        i += 1;
    }
}
unsafe extern "C" fn deleteFoldEntry(
    wp: *mut win_T,
    gap: *mut garray_T,
    idx: ::core::ffi::c_int,
    recursive: bool,
) {
    let mut fp: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(idx as isize);
    if recursive as ::core::ffi::c_int != 0 || (*fp).fd_nested.ga_len <= 0 as ::core::ffi::c_int {
        deleteFoldRecurse((*wp).w_buffer, &raw mut (*fp).fd_nested);
        (*gap).ga_len -= 1;
        if idx < (*gap).ga_len {
            memmove(
                fp as *mut ::core::ffi::c_void,
                fp.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<fold_T>().wrapping_mul(((*gap).ga_len - idx) as size_t),
            );
        }
    } else {
        let mut moved: ::core::ffi::c_int = (*fp).fd_nested.ga_len;
        ga_grow(gap, moved - 1 as ::core::ffi::c_int);
        fp = ((*gap).ga_data as *mut fold_T).offset(idx as isize);
        let mut nfp: *mut fold_T = (*fp).fd_nested.ga_data as *mut fold_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < moved {
            (*nfp.offset(i as isize)).fd_top += (*fp).fd_top;
            if (*fp).fd_flags as ::core::ffi::c_int == FD_LEVEL as ::core::ffi::c_int {
                (*nfp.offset(i as isize)).fd_flags =
                    FD_LEVEL as ::core::ffi::c_int as ::core::ffi::c_char;
            }
            if (*fp).fd_small as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                (*nfp.offset(i as isize)).fd_small = kNone;
            }
            i += 1;
        }
        if (idx + 1 as ::core::ffi::c_int) < (*gap).ga_len {
            memmove(
                fp.offset(moved as isize) as *mut ::core::ffi::c_void,
                fp.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<fold_T>()
                    .wrapping_mul(((*gap).ga_len - (idx + 1 as ::core::ffi::c_int)) as size_t),
            );
        }
        memmove(
            fp as *mut ::core::ffi::c_void,
            nfp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<fold_T>().wrapping_mul(moved as size_t),
        );
        xfree(nfp as *mut ::core::ffi::c_void);
        (*gap).ga_len += moved - 1 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn deleteFoldRecurse(mut bp: *mut buf_T, mut gap: *mut garray_T) {
    let mut _gap: *mut garray_T = gap;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut fold_T = ((*_gap).ga_data as *mut fold_T).offset(i as isize);
            deleteFoldRecurse(bp, &raw mut (*_item).fd_nested);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn foldMarkAdjust(
    mut wp: *mut win_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if amount == MAXLNUM as ::core::ffi::c_int as linenr_T
        && line2 >= line1
        && line2 - line1 >= -amount_after
    {
        line2 = line1 - amount_after - 1 as linenr_T;
    }
    if line2 < line1 {
        line2 = line1;
    }
    if State & MODE_INSERT as ::core::ffi::c_int != 0
        && amount == 1 as linenr_T
        && line2 == MAXLNUM as ::core::ffi::c_int as linenr_T
    {
        line1 -= 1;
    }
    foldMarkAdjustRecurse(
        wp,
        &raw mut (*wp).w_folds,
        line1,
        line2,
        amount,
        amount_after,
    );
}
unsafe extern "C" fn foldMarkAdjustRecurse(
    mut wp: *mut win_T,
    mut gap: *mut garray_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if (*gap).ga_len == 0 as ::core::ffi::c_int {
        return;
    }
    let mut top: linenr_T = if State & MODE_INSERT as ::core::ffi::c_int != 0
        && amount == 1 as linenr_T
        && line2 == MAXLNUM as ::core::ffi::c_int as linenr_T
    {
        line1 + 1 as linenr_T
    } else {
        line1
    };
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    foldFind(gap, line1, &raw mut fp);
    let mut i: ::core::ffi::c_int =
        fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut last: linenr_T = (*fp).fd_top + (*fp).fd_len - 1 as linenr_T;
        if last >= line1 {
            if (*fp).fd_top > line2 {
                if amount_after == 0 as linenr_T {
                    break;
                }
                (*fp).fd_top += amount_after;
            } else if (*fp).fd_top >= top && last <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    deleteFoldEntry(wp, gap, i, true_0 != 0);
                    i -= 1;
                    fp = fp.offset(-1);
                } else {
                    (*fp).fd_top += amount;
                }
            } else if (*fp).fd_top < top {
                foldMarkAdjustRecurse(
                    wp,
                    &raw mut (*fp).fd_nested,
                    line1 - (*fp).fd_top,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after,
                );
                if last <= line2 {
                    if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                        (*fp).fd_len = line1 - (*fp).fd_top;
                    } else {
                        (*fp).fd_len += amount;
                    }
                } else {
                    (*fp).fd_len += amount_after;
                }
            } else if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                foldMarkAdjustRecurse(
                    wp,
                    &raw mut (*fp).fd_nested,
                    0 as linenr_T,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after + ((*fp).fd_top - top),
                );
                (*fp).fd_len = ((*fp).fd_len as ::core::ffi::c_int
                    - (line2 - (*fp).fd_top + 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                (*fp).fd_top = line1;
            } else {
                foldMarkAdjustRecurse(
                    wp,
                    &raw mut (*fp).fd_nested,
                    0 as linenr_T,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after - amount,
                );
                (*fp).fd_len += amount_after - amount;
                (*fp).fd_top += amount;
            }
        }
        i += 1;
        fp = fp.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn getDeepestNesting(mut wp: *mut win_T) -> ::core::ffi::c_int {
    checkupdate(wp);
    return getDeepestNestingRecurse(&raw mut (*wp).w_folds);
}
unsafe extern "C" fn getDeepestNestingRecurse(mut gap: *mut garray_T) -> ::core::ffi::c_int {
    let mut maxlevel: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut level: ::core::ffi::c_int =
            getDeepestNestingRecurse(&raw mut (*fp.offset(i as isize)).fd_nested)
                + 1 as ::core::ffi::c_int;
        maxlevel = if maxlevel > level { maxlevel } else { level };
        i += 1;
    }
    return maxlevel;
}
unsafe extern "C" fn check_closed(
    wp: *mut win_T,
    fp: *mut fold_T,
    use_levelp: *mut bool,
    level: ::core::ffi::c_int,
    maybe_smallp: *mut bool,
    lnum_off: linenr_T,
) -> bool {
    let mut closed: bool = false_0 != 0;
    if *use_levelp as ::core::ffi::c_int != 0
        || (*fp).fd_flags as ::core::ffi::c_int == FD_LEVEL as ::core::ffi::c_int
    {
        *use_levelp = true_0 != 0;
        if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
            closed = true_0 != 0;
        }
    } else if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int {
        closed = true_0 != 0;
    }
    if (*fp).fd_small as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        *maybe_smallp = true_0 != 0;
    }
    if closed {
        if *maybe_smallp {
            (*fp).fd_small = kNone;
        }
        checkSmall(wp, fp, lnum_off);
        if (*fp).fd_small as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
            closed = false_0 != 0;
        }
    }
    return closed;
}
unsafe extern "C" fn checkSmall(wp: *mut win_T, fp: *mut fold_T, lnum_off: linenr_T) {
    if (*fp).fd_small as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
        return;
    }
    setSmallMaybe(&raw mut (*fp).fd_nested);
    if (*fp).fd_len as OptInt > (*wp).w_onebuf_opt.wo_fml {
        (*fp).fd_small = kFalse;
    } else {
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (n as linenr_T) < (*fp).fd_len {
            count += plines_win_nofold(wp, (*fp).fd_top + lnum_off + n as linenr_T);
            if count as OptInt > (*wp).w_onebuf_opt.wo_fml {
                (*fp).fd_small = kFalse;
                return;
            }
            n += 1;
        }
        (*fp).fd_small = kTrue;
    };
}
unsafe extern "C" fn setSmallMaybe(mut gap: *mut garray_T) {
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        (*fp.offset(i as isize)).fd_small = kNone;
        i += 1;
    }
}
unsafe extern "C" fn foldCreateMarkers(mut wp: *mut win_T, mut start: pos_T, mut end: pos_T) {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if (*buf).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        return;
    }
    parseMarker(wp);
    foldAddMarker(buf, start, (*wp).w_onebuf_opt.wo_fmr, foldstartmarkerlen);
    foldAddMarker(buf, end, foldendmarker, foldendmarkerlen);
    changed_lines(
        buf,
        start.lnum,
        0 as colnr_T,
        end.lnum,
        0 as linenr_T,
        false_0 != 0,
    );
    let mut num_changed: int64_t = (1 as linenr_T + end.lnum - start.lnum) as int64_t;
    buf_updates_send_changes(buf, start.lnum, num_changed, num_changed);
}
unsafe extern "C" fn foldAddMarker(
    mut buf: *mut buf_T,
    mut pos: pos_T,
    mut marker: *const ::core::ffi::c_char,
    mut markerlen: size_t,
) {
    let mut cms: *mut ::core::ffi::c_char = (*buf).b_p_cms;
    let mut p: *mut ::core::ffi::c_char = strstr(
        (*buf).b_p_cms,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut line_is_comment: bool = false_0 != 0;
    let mut lnum: linenr_T = pos.lnum;
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
    let mut line_len: size_t = ml_get_buf_len(buf, lnum) as size_t;
    let mut added: size_t = 0 as size_t;
    if u_save(lnum - 1 as linenr_T, lnum + 1 as linenr_T) != OK {
        return;
    }
    skip_comment(line, false_0 != 0, false_0 != 0, &raw mut line_is_comment);
    let mut newline: *mut ::core::ffi::c_char = xmalloc(
        line_len
            .wrapping_add(markerlen)
            .wrapping_add(strlen(cms))
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    strcpy(newline, line);
    if p.is_null() || line_is_comment as ::core::ffi::c_int != 0 {
        xmemcpyz(
            newline.offset(line_len as isize) as *mut ::core::ffi::c_void,
            marker as *const ::core::ffi::c_void,
            markerlen,
        );
        added = markerlen;
    } else {
        strcpy(newline.offset(line_len as isize), cms);
        memcpy(
            newline
                .offset(line_len as isize)
                .offset(p.offset_from(cms) as isize) as *mut ::core::ffi::c_void,
            marker as *const ::core::ffi::c_void,
            markerlen,
        );
        strcpy(
            newline
                .offset(line_len as isize)
                .offset(p.offset_from(cms) as isize)
                .offset(markerlen as isize),
            p.offset(2 as ::core::ffi::c_int as isize),
        );
        added = markerlen
            .wrapping_add(strlen(cms))
            .wrapping_sub(2 as size_t);
    }
    ml_replace_buf(buf, lnum, newline, false_0 != 0, false_0 != 0);
    if added != 0 {
        extmark_splice_cols(
            buf,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            line_len as colnr_T,
            0 as colnr_T,
            added as colnr_T,
            kExtmarkUndo,
        );
    }
}
unsafe extern "C" fn deleteFoldMarkers(
    mut wp: *mut win_T,
    mut fp: *mut fold_T,
    mut recursive: bool,
    mut lnum_off: linenr_T,
) {
    if recursive {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*fp).fd_nested.ga_len {
            deleteFoldMarkers(
                wp,
                ((*fp).fd_nested.ga_data as *mut fold_T).offset(i as isize),
                true_0 != 0,
                lnum_off + (*fp).fd_top,
            );
            i += 1;
        }
    }
    foldDelMarker(
        (*wp).w_buffer,
        (*fp).fd_top + lnum_off,
        (*wp).w_onebuf_opt.wo_fmr,
        foldstartmarkerlen,
    );
    foldDelMarker(
        (*wp).w_buffer,
        (*fp).fd_top + lnum_off + (*fp).fd_len - 1 as linenr_T,
        foldendmarker,
        foldendmarkerlen,
    );
}
unsafe extern "C" fn foldDelMarker(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut marker: *mut ::core::ffi::c_char,
    mut markerlen: size_t,
) {
    if lnum > (*buf).b_ml.ml_line_count {
        return;
    }
    let mut cms: *mut ::core::ffi::c_char = (*buf).b_p_cms;
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
    let mut p: *mut ::core::ffi::c_char = line;
    while *p as ::core::ffi::c_int != NUL {
        if strncmp(p, marker, markerlen) != 0 as ::core::ffi::c_int {
            p = p.offset(1);
        } else {
            let mut len: size_t = markerlen;
            if ascii_isdigit(*p.offset(len as isize) as ::core::ffi::c_int) {
                len = len.wrapping_add(1);
            }
            if *cms as ::core::ffi::c_int != NUL {
                let mut cms2: *mut ::core::ffi::c_char =
                    strstr(cms, b"%s\0".as_ptr() as *const ::core::ffi::c_char);
                if !cms2.is_null()
                    && p.offset_from(line) >= cms2.offset_from(cms)
                    && strncmp(
                        p.offset(-(cms2.offset_from(cms) as isize)),
                        cms,
                        cms2.offset_from(cms) as size_t,
                    ) == 0 as ::core::ffi::c_int
                    && strncmp(
                        p.offset(len as isize),
                        cms2.offset(2 as ::core::ffi::c_int as isize),
                        strlen(cms2.offset(2 as ::core::ffi::c_int as isize)),
                    ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(-(cms2.offset_from(cms) as isize));
                    len = len.wrapping_add(strlen(cms).wrapping_sub(2 as size_t));
                }
            }
            if u_save(lnum - 1 as linenr_T, lnum + 1 as linenr_T) == OK {
                let mut newline: *mut ::core::ffi::c_char = xmalloc(
                    (ml_get_buf_len(buf, lnum) as size_t)
                        .wrapping_sub(len)
                        .wrapping_add(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                '_c2rust_label: {
                    if p >= line {
                    } else {
                        __assert_fail(
                            b"p >= line\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/fold.c\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            1670 as ::core::ffi::c_uint,
                            b"void foldDelMarker(buf_T *, linenr_T, char *, size_t)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                memcpy(
                    newline as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    p.offset_from(line) as size_t,
                );
                strcpy(
                    newline.offset(p.offset_from(line) as isize),
                    p.offset(len as isize),
                );
                ml_replace_buf(buf, lnum, newline, false_0 != 0, false_0 != 0);
                extmark_splice_cols(
                    buf,
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    p.offset_from(line) as colnr_T,
                    len as colnr_T,
                    0 as colnr_T,
                    kExtmarkUndo,
                );
            }
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_foldtext(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut lnume: linenr_T,
    mut foldinfo: foldinfo_T,
    mut buf: *mut ::core::ffi::c_char,
    mut vt: *mut VirtText,
) -> *mut ::core::ffi::c_char {
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut got_fdt_error: bool = false_0 != 0;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg;
    static mut last_wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    static mut last_lnum: linenr_T = 0 as linenr_T;
    if last_wp.is_null() || last_wp != wp || last_lnum > lnum || last_lnum == 0 as linenr_T {
        got_fdt_error = false_0 != 0;
    }
    if !got_fdt_error {
        did_emsg = false_0;
    }
    if *(*wp).w_onebuf_opt.wo_fdt as ::core::ffi::c_int != NUL {
        let mut dashes: [::core::ffi::c_char; 22] = [0; 22];
        set_vim_var_nr(VV_FOLDSTART, lnum as varnumber_T);
        set_vim_var_nr(VV_FOLDEND, lnume as varnumber_T);
        let mut level: ::core::ffi::c_int = if foldinfo.fi_level
            < ::core::mem::size_of::<[::core::ffi::c_char; 22]>() as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
        {
            foldinfo.fi_level
        } else {
            ::core::mem::size_of::<[::core::ffi::c_char; 22]>() as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
        };
        memset(
            &raw mut dashes as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '-' as ::core::ffi::c_int,
            level as size_t,
        );
        dashes[level as usize] = NUL as ::core::ffi::c_char;
        set_vim_var_string(
            VV_FOLDDASHES,
            &raw mut dashes as *mut ::core::ffi::c_char,
            level as ptrdiff_t,
        );
        set_vim_var_nr(VV_FOLDLEVEL, level as varnumber_T);
        if !got_fdt_error {
            let save_curwin: *mut win_T = curwin;
            let saved_sctx: sctx_T = current_sctx;
            curwin = wp;
            curbuf = (*wp).w_buffer;
            current_sctx =
                (*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldtext as ::core::ffi::c_int as usize];
            emsg_off += 1;
            let mut obj: Object = eval_foldtext(wp);
            if obj.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut err: Error = Error {
                    type_0: kErrorTypeNone,
                    msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
                *vt = parse_virt_text(
                    obj.data.array,
                    &raw mut err,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                );
                if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    *buf = NUL as ::core::ffi::c_char;
                    text = buf;
                }
                api_clear_error(&raw mut err);
            } else if obj.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                text = obj.data.string.data;
                obj = object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
            }
            api_free_object(obj);
            emsg_off -= 1;
            if text.is_null() || did_emsg != 0 {
                got_fdt_error = true_0 != 0;
            }
            curwin = save_curwin;
            curbuf = (*curwin).w_buffer;
            current_sctx = saved_sctx;
        }
        last_lnum = lnum;
        last_wp = wp;
        set_vim_var_string(
            VV_FOLDDASHES,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        if did_emsg == 0 && save_did_emsg != 0 {
            did_emsg = save_did_emsg;
        }
        if !text.is_null() {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = text;
            while *p as ::core::ffi::c_int != NUL {
                let mut len: ::core::ffi::c_int = utfc_ptr2len(p);
                if len > 1 as ::core::ffi::c_int {
                    if !vim_isprintc(utf_ptr2char(p)) {
                        break;
                    }
                    p = p.offset((len - 1 as ::core::ffi::c_int) as isize);
                } else if *p as ::core::ffi::c_int == TAB {
                    *p = ' ' as ::core::ffi::c_char;
                } else if ptr2cells(p) > 1 as ::core::ffi::c_int {
                    break;
                }
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int != NUL {
                p = transstr(text, true_0 != 0);
                xfree(text as *mut ::core::ffi::c_void);
                text = p;
            }
        }
    }
    if text.is_null() {
        let mut count: ::core::ffi::c_int =
            lnume as ::core::ffi::c_int - lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        vim_snprintf(
            buf,
            FOLD_TEXT_LEN as ::core::ffi::c_int as size_t,
            ngettext(
                b"+--%3d line folded\0".as_ptr() as *const ::core::ffi::c_char,
                b"+--%3d lines folded \0".as_ptr() as *const ::core::ffi::c_char,
                count as ::core::ffi::c_ulong,
            ),
            count,
        );
        text = buf;
    }
    return text;
}
unsafe extern "C" fn foldtext_cleanup(mut str: *mut ::core::ffi::c_char) {
    let mut cms_start: *mut ::core::ffi::c_char = skipwhite((*curbuf).b_p_cms);
    let mut cms_slen: size_t = strlen(cms_start);
    while cms_slen > 0 as size_t
        && ascii_iswhite(
            *cms_start.offset(cms_slen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
    {
        cms_slen = cms_slen.wrapping_sub(1);
    }
    let mut cms_end: *mut ::core::ffi::c_char =
        strstr(cms_start, b"%s\0".as_ptr() as *const ::core::ffi::c_char);
    let mut cms_elen: size_t = 0 as size_t;
    if !cms_end.is_null() {
        cms_elen = cms_slen.wrapping_sub(cms_end.offset_from(cms_start) as size_t);
        cms_slen = cms_end.offset_from(cms_start) as size_t;
        while cms_slen > 0 as size_t
            && ascii_iswhite(
                *cms_start.offset(cms_slen.wrapping_sub(1 as size_t) as isize)
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        {
            cms_slen = cms_slen.wrapping_sub(1);
        }
        let mut s: *mut ::core::ffi::c_char =
            skipwhite(cms_end.offset(2 as ::core::ffi::c_int as isize));
        cms_elen = cms_elen.wrapping_sub(s.offset_from(cms_end) as size_t);
        cms_end = s;
    }
    parseMarker(curwin);
    let mut did1: bool = false_0 != 0;
    let mut did2: bool = false_0 != 0;
    let mut s_0: *mut ::core::ffi::c_char = str;
    while *s_0 as ::core::ffi::c_int != NUL {
        let mut len: size_t = 0 as size_t;
        if strncmp(s_0, (*curwin).w_onebuf_opt.wo_fmr, foldstartmarkerlen)
            == 0 as ::core::ffi::c_int
        {
            len = foldstartmarkerlen;
        } else if strncmp(s_0, foldendmarker, foldendmarkerlen) == 0 as ::core::ffi::c_int {
            len = foldendmarkerlen;
        }
        if len > 0 as size_t {
            if ascii_isdigit(*s_0.offset(len as isize) as ::core::ffi::c_int) {
                len = len.wrapping_add(1);
            }
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = s_0;
            while p > str
                && ascii_iswhite(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(-1);
            }
            if p >= str.offset(cms_slen as isize)
                && strncmp(p.offset(-(cms_slen as isize)), cms_start, cms_slen)
                    == 0 as ::core::ffi::c_int
            {
                len = len.wrapping_add((s_0.offset_from(p) as size_t).wrapping_add(cms_slen));
                s_0 = p.offset(-(cms_slen as isize));
            }
        } else if !cms_end.is_null() {
            if !did1
                && cms_slen > 0 as size_t
                && strncmp(s_0, cms_start, cms_slen) == 0 as ::core::ffi::c_int
            {
                len = cms_slen;
                did1 = true_0 != 0;
            } else if !did2
                && cms_elen > 0 as size_t
                && strncmp(s_0, cms_end, cms_elen) == 0 as ::core::ffi::c_int
            {
                len = cms_elen;
                did2 = true_0 != 0;
            }
        }
        if len != 0 as size_t {
            while ascii_iswhite(*s_0.offset(len as isize) as ::core::ffi::c_int) {
                len = len.wrapping_add(1);
            }
            memmove(
                s_0 as *mut ::core::ffi::c_void,
                s_0.offset(len as isize) as *const ::core::ffi::c_void,
                strlen(s_0.offset(len as isize)).wrapping_add(1 as size_t),
            );
        } else {
            s_0 = s_0.offset(utfc_ptr2len(s_0) as isize);
        }
    }
}
unsafe extern "C" fn foldUpdateIEMS(wp: *mut win_T, mut top: linenr_T, mut bot: linenr_T) {
    if invalid_top != 0 as linenr_T {
        return;
    }
    if (*wp).w_foldinvalid {
        top = 1 as ::core::ffi::c_int as linenr_T;
        bot = (*(*wp).w_buffer).b_ml.ml_line_count;
        (*wp).w_foldinvalid = false_0 != 0;
        setSmallMaybe(&raw mut (*wp).w_folds);
    }
    if foldmethodIsDiff(wp) {
        if top > diff_context as linenr_T {
            top = (top as ::core::ffi::c_int - diff_context) as linenr_T;
        } else {
            top = 1 as ::core::ffi::c_int as linenr_T;
        }
        bot = (bot as ::core::ffi::c_int + diff_context) as linenr_T;
    }
    top = if top < (*(*wp).w_buffer).b_ml.ml_line_count {
        top
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    let mut fline: fline_T = fline_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        lnum: 0,
        off: 0,
        lnum_save: 0,
        lvl: 0,
        lvl_next: 0,
        start: 0,
        end: 0,
        had_end: 0,
    };
    fold_changed = false_0 != 0;
    fline.wp = wp;
    fline.off = 0 as ::core::ffi::c_int as linenr_T;
    fline.lvl = 0 as ::core::ffi::c_int;
    fline.lvl_next = -1 as ::core::ffi::c_int;
    fline.start = 0 as ::core::ffi::c_int;
    fline.end = MAX_LEVEL + 1 as ::core::ffi::c_int;
    fline.had_end = MAX_LEVEL + 1 as ::core::ffi::c_int;
    invalid_top = top;
    invalid_bot = bot;
    let mut getlevel: LevelGetter = None;
    if foldmethodIsMarker(wp) {
        getlevel = Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ()) as LevelGetter;
        parseMarker(wp);
        if top > 1 as linenr_T {
            let level: ::core::ffi::c_int = foldLevelWin(wp, top - 1 as linenr_T);
            fline.lnum = top - 1 as linenr_T;
            fline.lvl = level;
            getlevel.expect("non-null function pointer")(&raw mut fline);
            if fline.lvl > level {
                fline.lvl = level - (fline.lvl - fline.lvl_next);
            } else {
                fline.lvl = fline.lvl_next;
            }
        }
        fline.lnum = top;
        getlevel.expect("non-null function pointer")(&raw mut fline);
    } else {
        fline.lnum = top;
        if foldmethodIsExpr(wp) {
            getlevel =
                Some(foldlevelExpr as unsafe extern "C" fn(*mut fline_T) -> ()) as LevelGetter;
            if top > 1 as linenr_T {
                fline.lnum -= 1;
            }
        } else if foldmethodIsSyntax(wp) {
            getlevel =
                Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ()) as LevelGetter;
        } else if foldmethodIsDiff(wp) {
            getlevel =
                Some(foldlevelDiff as unsafe extern "C" fn(*mut fline_T) -> ()) as LevelGetter;
        } else {
            getlevel =
                Some(foldlevelIndent as unsafe extern "C" fn(*mut fline_T) -> ()) as LevelGetter;
            if top > 1 as linenr_T {
                fline.lnum -= 1;
            }
        }
        fline.lvl = -1 as ::core::ffi::c_int;
        while !got_int {
            fline.lvl_next = -1 as ::core::ffi::c_int;
            getlevel.expect("non-null function pointer")(&raw mut fline);
            if fline.lvl >= 0 as ::core::ffi::c_int {
                break;
            }
            fline.lnum -= 1;
        }
    }
    if Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ()) == getlevel {
        let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
        let mut fpn: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
        let mut current_fdl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fold_start_lnum: linenr_T = 0 as linenr_T;
        let mut lnum_rel: linenr_T = fline.lnum;
        while current_fdl < fline.lvl {
            if !foldFind(gap, lnum_rel, &raw mut fpn) {
                break;
            }
            current_fdl += 1;
            fold_start_lnum += (*fpn).fd_top;
            gap = &raw mut (*fpn).fd_nested;
            lnum_rel -= (*fpn).fd_top;
        }
        if !fpn.is_null() && current_fdl == fline.lvl {
            let mut fold_end_lnum: linenr_T = fold_start_lnum + (*fpn).fd_len;
            bot = if bot > fold_end_lnum {
                bot
            } else {
                fold_end_lnum
            };
        }
    }
    let mut start: linenr_T = fline.lnum;
    let mut end: linenr_T = bot;
    if start > end && end < (*(*wp).w_buffer).b_ml.ml_line_count {
        end = start;
    }
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    while !got_int {
        if fline.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            break;
        }
        if fline.lnum > end {
            if getlevel != Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ())
                && getlevel != Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ())
                && getlevel != Some(foldlevelExpr as unsafe extern "C" fn(*mut fline_T) -> ())
            {
                break;
            }
            if start <= end
                && foldFind(&raw mut (*wp).w_folds, end, &raw mut fp) as ::core::ffi::c_int != 0
                && (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > end
                || fline.lvl == 0 as ::core::ffi::c_int
                    && foldFind(&raw mut (*wp).w_folds, fline.lnum, &raw mut fp)
                        as ::core::ffi::c_int
                        != 0
                    && (*fp).fd_top < fline.lnum
            {
                end = (*fp).fd_top + (*fp).fd_len - 1 as linenr_T;
            } else {
                if !(getlevel == Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ())
                    && foldLevelWin(wp, fline.lnum) != fline.lvl)
                {
                    break;
                }
                end = fline.lnum;
            }
        }
        if fline.lvl > 0 as ::core::ffi::c_int {
            invalid_top = fline.lnum;
            invalid_bot = end;
            end = foldUpdateIEMSRecurse(
                &raw mut (*wp).w_folds,
                1 as ::core::ffi::c_int,
                start,
                &raw mut fline,
                getlevel,
                end,
                FD_LEVEL as ::core::ffi::c_int as ::core::ffi::c_char,
            );
            start = fline.lnum;
        } else {
            if fline.lnum == (*(*wp).w_buffer).b_ml.ml_line_count {
                break;
            }
            fline.lnum += 1;
            fline.lvl = fline.lvl_next;
            getlevel.expect("non-null function pointer")(&raw mut fline);
        }
    }
    foldRemove(wp, &raw mut (*wp).w_folds, start, end);
    if fold_changed as ::core::ffi::c_int != 0 && (*wp).w_onebuf_opt.wo_fen != 0 {
        changed_window_setting(wp);
    }
    if end != bot {
        redraw_win_range_later(wp, top, end);
    }
    invalid_top = 0 as ::core::ffi::c_int as linenr_T;
}
unsafe extern "C" fn foldUpdateIEMSRecurse(
    gap: *mut garray_T,
    level: ::core::ffi::c_int,
    startlnum: linenr_T,
    flp: *mut fline_T,
    mut getlevel: LevelGetter,
    mut bot: linenr_T,
    topflags: ::core::ffi::c_char,
) -> linenr_T {
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    if getlevel == Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ())
        && (*flp).start <= (*flp).lvl - level
        && (*flp).lvl > 0 as ::core::ffi::c_int
    {
        foldFind(gap, startlnum - 1 as linenr_T, &raw mut fp);
        if !fp.is_null()
            && (fp >= ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
                || (*fp).fd_top >= startlnum)
        {
            fp = ::core::ptr::null_mut::<fold_T>();
        }
    }
    let mut fp2: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut lvl: ::core::ffi::c_int = level;
    let mut startlnum2: linenr_T = startlnum;
    let firstlnum: linenr_T = (*flp).lnum;
    let mut finish: bool = false_0 != 0;
    let linecount: linenr_T = (*(*(*flp).wp).w_buffer).b_ml.ml_line_count - (*flp).off;
    (*flp).lnum_save = (*flp).lnum;
    while !got_int {
        line_breakcheck();
        lvl = if (*flp).lvl < 20 as ::core::ffi::c_int {
            (*flp).lvl
        } else {
            20 as ::core::ffi::c_int
        };
        if (*flp).lnum > firstlnum && (level > lvl - (*flp).start || level >= (*flp).had_end) {
            lvl = 0 as ::core::ffi::c_int;
        }
        if (*flp).lnum > bot && !finish && !fp.is_null() {
            if getlevel != Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ())
                && getlevel != Some(foldlevelExpr as unsafe extern "C" fn(*mut fline_T) -> ())
                && getlevel != Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ())
            {
                break;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            fp2 = fp;
            if lvl >= level {
                let mut ll: ::core::ffi::c_int =
                    (*flp).lnum as ::core::ffi::c_int - (*fp).fd_top as ::core::ffi::c_int;
                while foldFind(&raw mut (*fp2).fd_nested, ll as linenr_T, &raw mut fp2) {
                    i += 1;
                    ll -= (*fp2).fd_top as ::core::ffi::c_int;
                }
            }
            if lvl < level + i {
                foldFind(
                    &raw mut (*fp).fd_nested,
                    (*flp).lnum - (*fp).fd_top,
                    &raw mut fp2,
                );
                if !fp2.is_null() {
                    bot = (*fp2).fd_top + (*fp2).fd_len - 1 as linenr_T + (*fp).fd_top;
                }
            } else {
                if !((*fp).fd_top + (*fp).fd_len <= (*flp).lnum && lvl >= level) {
                    break;
                }
                finish = true_0 != 0;
            }
        }
        if fp.is_null()
            && (lvl != level
                || (*flp).lnum_save >= bot
                || (*flp).start != 0 as ::core::ffi::c_int
                || (*flp).had_end <= MAX_LEVEL
                || (*flp).lnum == linecount)
        {
            while !got_int {
                let mut concat: ::core::ffi::c_int =
                    if (*flp).start != 0 as ::core::ffi::c_int || (*flp).had_end <= MAX_LEVEL {
                        0 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                if (*gap).ga_len > 0 as ::core::ffi::c_int
                    && (foldFind(gap, startlnum, &raw mut fp) as ::core::ffi::c_int != 0
                        || fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
                            && (*fp).fd_top <= firstlnum
                        || foldFind(gap, firstlnum - concat as linenr_T, &raw mut fp)
                            as ::core::ffi::c_int
                            != 0
                        || fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
                            && (lvl < level && (*fp).fd_top < (*flp).lnum
                                || lvl >= level && (*fp).fd_top <= (*flp).lnum_save))
                {
                    if (*fp).fd_top + (*fp).fd_len + concat as linenr_T > firstlnum {
                        if (*fp).fd_top != firstlnum {
                            if (*fp).fd_top >= startlnum {
                                if (*fp).fd_top > firstlnum {
                                    foldMarkAdjustRecurse(
                                        (*flp).wp,
                                        &raw mut (*fp).fd_nested,
                                        0 as linenr_T,
                                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                                        (*fp).fd_top - firstlnum,
                                        0 as linenr_T,
                                    );
                                } else {
                                    foldMarkAdjustRecurse(
                                        (*flp).wp,
                                        &raw mut (*fp).fd_nested,
                                        0 as linenr_T,
                                        firstlnum - (*fp).fd_top - 1 as linenr_T,
                                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                                        (*fp).fd_top - firstlnum,
                                    );
                                }
                                (*fp).fd_len += (*fp).fd_top - firstlnum;
                                (*fp).fd_top = firstlnum;
                                (*fp).fd_small = kNone;
                                fold_changed = true_0 != 0;
                            } else if (*flp).start != 0 as ::core::ffi::c_int && lvl == level
                                || firstlnum != startlnum
                            {
                                let mut breakstart: linenr_T = 0;
                                let mut breakend: linenr_T = 0;
                                if firstlnum != startlnum {
                                    breakstart = startlnum;
                                    breakend = firstlnum;
                                } else {
                                    breakstart = (*flp).lnum;
                                    breakend = (*flp).lnum;
                                }
                                foldRemove(
                                    (*flp).wp,
                                    &raw mut (*fp).fd_nested,
                                    breakstart - (*fp).fd_top,
                                    breakend - (*fp).fd_top,
                                );
                                let mut i_0: ::core::ffi::c_int = fp
                                    .offset_from((*gap).ga_data as *mut fold_T)
                                    as ::core::ffi::c_int;
                                foldSplit(
                                    (*(*flp).wp).w_buffer,
                                    gap,
                                    i_0,
                                    breakstart,
                                    breakend - 1 as linenr_T,
                                );
                                fp = ((*gap).ga_data as *mut fold_T)
                                    .offset(i_0 as isize)
                                    .offset(1 as ::core::ffi::c_int as isize);
                                if getlevel
                                    == Some(
                                        foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> (),
                                    )
                                    || getlevel
                                        == Some(
                                            foldlevelExpr
                                                as unsafe extern "C" fn(*mut fline_T) -> (),
                                        )
                                    || getlevel
                                        == Some(
                                            foldlevelSyntax
                                                as unsafe extern "C" fn(*mut fline_T) -> (),
                                        )
                                {
                                    finish = true_0 != 0;
                                }
                            }
                        }
                        if (*fp).fd_top == startlnum && concat != 0 {
                            let mut i_1: ::core::ffi::c_int =
                                fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
                            if i_1 != 0 as ::core::ffi::c_int {
                                fp2 = fp.offset(-(1 as ::core::ffi::c_int as isize));
                                if (*fp2).fd_top + (*fp2).fd_len == (*fp).fd_top {
                                    foldMerge((*flp).wp, fp2, gap, fp);
                                    fp = fp2;
                                }
                            }
                        }
                        break;
                    } else if (*fp).fd_top >= startlnum {
                        deleteFoldEntry(
                            (*flp).wp,
                            gap,
                            fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int,
                            true_0 != 0,
                        );
                    } else {
                        (*fp).fd_len = startlnum - (*fp).fd_top;
                        foldMarkAdjustRecurse(
                            (*flp).wp,
                            &raw mut (*fp).fd_nested,
                            (*fp).fd_len,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            0 as linenr_T,
                        );
                        fold_changed = true_0 != 0;
                    }
                } else {
                    let mut i_2: ::core::ffi::c_int = 0;
                    if (*gap).ga_len == 0 as ::core::ffi::c_int {
                        i_2 = 0 as ::core::ffi::c_int;
                    } else {
                        i_2 = fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
                    }
                    foldInsert(gap, i_2);
                    fp = ((*gap).ga_data as *mut fold_T).offset(i_2 as isize);
                    (*fp).fd_top = firstlnum;
                    (*fp).fd_len = bot - firstlnum + 1 as linenr_T;
                    if topflags as ::core::ffi::c_int == FD_OPEN as ::core::ffi::c_int {
                        (*(*flp).wp).w_fold_manual = true_0 != 0;
                        (*fp).fd_flags = FD_OPEN as ::core::ffi::c_int as ::core::ffi::c_char;
                    } else if i_2 <= 0 as ::core::ffi::c_int {
                        (*fp).fd_flags = topflags;
                        if topflags as ::core::ffi::c_int != FD_LEVEL as ::core::ffi::c_int {
                            (*(*flp).wp).w_fold_manual = true_0 != 0;
                        }
                    } else {
                        (*fp).fd_flags = (*fp.offset(-(1 as ::core::ffi::c_int as isize))).fd_flags;
                    }
                    (*fp).fd_small = kNone;
                    if getlevel == Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ())
                        || getlevel
                            == Some(foldlevelExpr as unsafe extern "C" fn(*mut fline_T) -> ())
                        || getlevel
                            == Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ())
                    {
                        finish = true_0 != 0;
                    }
                    fold_changed = true_0 != 0;
                    break;
                }
            }
        }
        if lvl < level || (*flp).lnum > linecount {
            break;
        }
        if lvl > level && !fp.is_null() {
            bot = if bot > (*flp).lnum { bot } else { (*flp).lnum };
            (*flp).lnum = (*flp).lnum_save - (*fp).fd_top;
            (*flp).off += (*fp).fd_top;
            let mut i_3: ::core::ffi::c_int =
                fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
            bot = foldUpdateIEMSRecurse(
                &raw mut (*fp).fd_nested,
                level + 1 as ::core::ffi::c_int,
                startlnum2 - (*fp).fd_top,
                flp,
                getlevel,
                bot - (*fp).fd_top,
                (*fp).fd_flags,
            );
            fp = ((*gap).ga_data as *mut fold_T).offset(i_3 as isize);
            (*flp).lnum += (*fp).fd_top;
            (*flp).lnum_save += (*fp).fd_top;
            (*flp).off -= (*fp).fd_top;
            bot += (*fp).fd_top;
            startlnum2 = (*flp).lnum;
        } else {
            (*flp).lnum = (*flp).lnum_save;
            let mut ll_0: ::core::ffi::c_int =
                (*flp).lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            while !got_int {
                prev_lnum = (*flp).lnum;
                prev_lnum_lvl = (*flp).lvl;
                (*flp).lnum += 1;
                if (*flp).lnum > linecount {
                    break;
                }
                (*flp).lvl = (*flp).lvl_next;
                getlevel.expect("non-null function pointer")(flp);
                if (*flp).lvl >= 0 as ::core::ffi::c_int || (*flp).had_end <= MAX_LEVEL {
                    break;
                }
            }
            prev_lnum = 0 as ::core::ffi::c_int as linenr_T;
            if (*flp).lnum > linecount {
                break;
            }
            (*flp).lnum_save = (*flp).lnum;
            (*flp).lnum = ll_0 as linenr_T;
        }
    }
    if fp.is_null() {
        return bot;
    }
    if (*fp).fd_len < (*flp).lnum - (*fp).fd_top {
        (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
        (*fp).fd_small = kNone;
        fold_changed = true_0 != 0;
    } else if (*fp).fd_top + (*fp).fd_len > linecount {
        (*fp).fd_len = linecount - (*fp).fd_top + 1 as linenr_T;
    }
    foldRemove(
        (*flp).wp,
        &raw mut (*fp).fd_nested,
        startlnum2 - (*fp).fd_top,
        (*flp).lnum - 1 as linenr_T - (*fp).fd_top,
    );
    if lvl < level {
        if (*fp).fd_len != (*flp).lnum - (*fp).fd_top {
            if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > bot {
                if getlevel == Some(foldlevelMarker as unsafe extern "C" fn(*mut fline_T) -> ())
                    || getlevel == Some(foldlevelExpr as unsafe extern "C" fn(*mut fline_T) -> ())
                    || getlevel == Some(foldlevelSyntax as unsafe extern "C" fn(*mut fline_T) -> ())
                {
                    bot = (*fp).fd_top + (*fp).fd_len - 1 as linenr_T;
                    (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
                } else {
                    let mut i_4: ::core::ffi::c_int =
                        fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int;
                    foldSplit((*(*flp).wp).w_buffer, gap, i_4, (*flp).lnum, bot);
                    fp = ((*gap).ga_data as *mut fold_T).offset(i_4 as isize);
                }
            } else {
                (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
            }
            fold_changed = true_0 != 0;
        }
    }
    loop {
        fp2 = fp.offset(1 as ::core::ffi::c_int as isize);
        if fp2 >= ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
            || (*fp2).fd_top > (*flp).lnum
        {
            break;
        }
        if (*fp2).fd_top + (*fp2).fd_len > (*flp).lnum {
            if (*fp2).fd_top < (*flp).lnum {
                foldMarkAdjustRecurse(
                    (*flp).wp,
                    &raw mut (*fp2).fd_nested,
                    0 as linenr_T,
                    (*flp).lnum - (*fp2).fd_top - 1 as linenr_T,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    (*fp2).fd_top - (*flp).lnum,
                );
                (*fp2).fd_len -= (*flp).lnum - (*fp2).fd_top;
                (*fp2).fd_top = (*flp).lnum;
                fold_changed = true_0 != 0;
            }
            if lvl >= level {
                foldMerge((*flp).wp, fp, gap, fp2);
            }
            break;
        } else {
            fold_changed = true_0 != 0;
            deleteFoldEntry(
                (*flp).wp,
                gap,
                fp2.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int,
                true_0 != 0,
            );
        }
    }
    bot = if bot > (*flp).lnum - 1 as linenr_T {
        bot
    } else {
        (*flp).lnum - 1 as linenr_T
    };
    return bot;
}
unsafe extern "C" fn foldInsert(mut gap: *mut garray_T, mut i: ::core::ffi::c_int) {
    ga_grow(gap, 1 as ::core::ffi::c_int);
    let mut fp: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(i as isize);
    if (*gap).ga_len > 0 as ::core::ffi::c_int && i < (*gap).ga_len {
        memmove(
            fp.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            fp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<fold_T>().wrapping_mul(((*gap).ga_len - i) as size_t),
        );
    }
    (*gap).ga_len += 1;
    ga_init(
        &raw mut (*fp).fd_nested,
        ::core::mem::size_of::<fold_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn foldSplit(
    mut _buf: *mut buf_T,
    gap: *mut garray_T,
    i: ::core::ffi::c_int,
    top: linenr_T,
    bot: linenr_T,
) {
    let mut fp2: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    foldInsert(gap, i + 1 as ::core::ffi::c_int);
    let fp: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(i as isize);
    (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top = bot + 1 as linenr_T;
    '_c2rust_label: {
        if (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top > bot {
        } else {
            __assert_fail(
                b"fp[1].fd_top > bot\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/fold.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2553 as ::core::ffi::c_uint,
                b"void foldSplit(buf_T *, garray_T *const, const int, const linenr_T, const linenr_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_len =
        (*fp).fd_len - ((*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top - (*fp).fd_top);
    (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_flags = (*fp).fd_flags;
    (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_small = kNone;
    (*fp).fd_small = kNone;
    let gap1: *mut garray_T = &raw mut (*fp).fd_nested;
    let gap2: *mut garray_T = &raw mut (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_nested;
    foldFind(gap1, bot + 1 as linenr_T - (*fp).fd_top, &raw mut fp2);
    if !fp2.is_null() {
        let len: ::core::ffi::c_int = ((*gap1).ga_data as *mut fold_T)
            .offset((*gap1).ga_len as isize)
            .offset_from(fp2) as ::core::ffi::c_int;
        if len > 0 as ::core::ffi::c_int {
            ga_grow(gap2, len);
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while idx < len {
                *((*gap2).ga_data as *mut fold_T).offset(idx as isize) = *fp2.offset(idx as isize);
                (*((*gap2).ga_data as *mut fold_T).offset(idx as isize)).fd_top -=
                    (*fp.offset(1 as ::core::ffi::c_int as isize)).fd_top - (*fp).fd_top;
                idx += 1;
            }
            (*gap2).ga_len = len;
            (*gap1).ga_len -= len;
        }
    }
    (*fp).fd_len = top - (*fp).fd_top;
    fold_changed = true_0 != 0;
}
unsafe extern "C" fn foldRemove(
    wp: *mut win_T,
    mut gap: *mut garray_T,
    mut top: linenr_T,
    mut bot: linenr_T,
) {
    if bot < top {
        return;
    }
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    while (*gap).ga_len > 0 as ::core::ffi::c_int {
        if foldFind(gap, top, &raw mut fp) as ::core::ffi::c_int != 0 && (*fp).fd_top < top {
            foldRemove(
                wp,
                &raw mut (*fp).fd_nested,
                top - (*fp).fd_top,
                bot - (*fp).fd_top,
            );
            if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > bot {
                foldSplit(
                    (*wp).w_buffer,
                    gap,
                    fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int,
                    top,
                    bot,
                );
            } else {
                (*fp).fd_len = top - (*fp).fd_top;
            }
            fold_changed = true_0 != 0;
        } else {
            if (*gap).ga_data.is_null()
                || fp >= ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
                || (*fp).fd_top > bot
            {
                break;
            }
            if (*fp).fd_top < top {
                continue;
            }
            fold_changed = true_0 != 0;
            if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > bot {
                foldMarkAdjustRecurse(
                    wp,
                    &raw mut (*fp).fd_nested,
                    0 as linenr_T,
                    bot - (*fp).fd_top,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    (*fp).fd_top - bot - 1 as linenr_T,
                );
                (*fp).fd_len = ((*fp).fd_len as ::core::ffi::c_int
                    - (bot - (*fp).fd_top + 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                (*fp).fd_top = bot + 1 as linenr_T;
                break;
            } else {
                deleteFoldEntry(
                    wp,
                    gap,
                    fp.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int,
                    true_0 != 0,
                );
            }
        }
    }
}
unsafe extern "C" fn foldReverseOrder(
    mut gap: *mut garray_T,
    start_arg: linenr_T,
    end_arg: linenr_T,
) {
    let mut start: linenr_T = start_arg;
    let mut end: linenr_T = end_arg;
    while start < end {
        let mut left: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(start as isize);
        let mut right: *mut fold_T = ((*gap).ga_data as *mut fold_T).offset(end as isize);
        let mut tmp: fold_T = *left;
        *left = *right;
        *right = tmp;
        start += 1;
        end -= 1;
    }
}
unsafe extern "C" fn truncate_fold(wp: *mut win_T, mut fp: *mut fold_T, mut end: linenr_T) {
    end = (end as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
    foldRemove(
        wp,
        &raw mut (*fp).fd_nested,
        end - (*fp).fd_top,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
    );
    (*fp).fd_len = end - (*fp).fd_top;
}
#[no_mangle]
pub unsafe extern "C" fn foldMoveRange(
    wp: *mut win_T,
    mut gap: *mut garray_T,
    line1: linenr_T,
    line2: linenr_T,
    dest: linenr_T,
) {
    let mut fp: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let range_len: linenr_T = line2 - line1 + 1 as linenr_T;
    let move_len: linenr_T = dest - line2;
    let at_start: bool = foldFind(gap, line1 - 1 as linenr_T, &raw mut fp);
    if at_start {
        if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > dest {
            foldMoveRange(
                wp,
                &raw mut (*fp).fd_nested,
                line1 - (*fp).fd_top,
                line2 - (*fp).fd_top,
                dest - (*fp).fd_top,
            );
            return;
        } else if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > line2 {
            foldMarkAdjustRecurse(
                wp,
                &raw mut (*fp).fd_nested,
                line1 - (*fp).fd_top,
                line2 - (*fp).fd_top,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                -range_len,
            );
            (*fp).fd_len -= range_len;
        } else {
            truncate_fold(wp, fp, line1 - 1 as linenr_T);
        }
        fp = fp.offset(1 as ::core::ffi::c_int as isize);
    }
    if !((*gap).ga_len > 0 as ::core::ffi::c_int
        && fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize))
        || (*fp).fd_top > dest
    {
        return;
    } else if (*fp).fd_top > line2 {
        while (*gap).ga_len > 0 as ::core::ffi::c_int
            && fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
            && (*fp).fd_top + (*fp).fd_len - 1 as linenr_T <= dest
        {
            (*fp).fd_top -= range_len;
            fp = fp.offset(1);
        }
        if (*gap).ga_len > 0 as ::core::ffi::c_int
            && fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
            && (*fp).fd_top <= dest
        {
            truncate_fold(wp, fp, dest);
            (*fp).fd_top -= range_len;
        }
        return;
    } else if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > dest {
        foldMarkAdjustRecurse(
            wp,
            &raw mut (*fp).fd_nested,
            line2 + 1 as linenr_T - (*fp).fd_top,
            dest - (*fp).fd_top,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -move_len,
        );
        (*fp).fd_len -= move_len;
        (*fp).fd_top += move_len;
        return;
    }
    let mut move_start: size_t = fp.offset_from((*gap).ga_data as *mut fold_T) as size_t;
    let mut move_end: size_t = 0 as size_t;
    let mut dest_index: size_t = 0 as size_t;
    while (*gap).ga_len > 0 as ::core::ffi::c_int
        && fp < ((*gap).ga_data as *mut fold_T).offset((*gap).ga_len as isize)
        && (*fp).fd_top <= dest
    {
        if (*fp).fd_top <= line2 {
            if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > line2 {
                truncate_fold(wp, fp, line2);
            }
            (*fp).fd_top += move_len;
        } else {
            if move_end == 0 as size_t {
                move_end = fp.offset_from((*gap).ga_data as *mut fold_T) as size_t;
            }
            if (*fp).fd_top + (*fp).fd_len - 1 as linenr_T > dest {
                truncate_fold(wp, fp, dest);
            }
            (*fp).fd_top -= range_len;
        }
        fp = fp.offset(1);
    }
    dest_index = fp.offset_from((*gap).ga_data as *mut fold_T) as size_t;
    if move_end == 0 as size_t {
        return;
    }
    foldReverseOrder(
        gap,
        move_start as linenr_T,
        dest_index.wrapping_sub(1 as size_t) as linenr_T,
    );
    foldReverseOrder(
        gap,
        move_start as linenr_T,
        move_start
            .wrapping_add(dest_index)
            .wrapping_sub(move_end)
            .wrapping_sub(1 as size_t) as linenr_T,
    );
    foldReverseOrder(
        gap,
        move_start.wrapping_add(dest_index).wrapping_sub(move_end) as linenr_T,
        dest_index.wrapping_sub(1 as size_t) as linenr_T,
    );
}
unsafe extern "C" fn foldMerge(
    wp: *mut win_T,
    mut fp1: *mut fold_T,
    mut gap: *mut garray_T,
    mut fp2: *mut fold_T,
) {
    let mut fp3: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut fp4: *mut fold_T = ::core::ptr::null_mut::<fold_T>();
    let mut gap1: *mut garray_T = &raw mut (*fp1).fd_nested;
    let mut gap2: *mut garray_T = &raw mut (*fp2).fd_nested;
    if foldFind(gap1, (*fp1).fd_len - 1 as linenr_T, &raw mut fp3) as ::core::ffi::c_int != 0
        && foldFind(gap2, 0 as linenr_T, &raw mut fp4) as ::core::ffi::c_int != 0
    {
        foldMerge(wp, fp3, gap2, fp4);
    }
    if !((*gap2).ga_len <= 0 as ::core::ffi::c_int) {
        ga_grow(gap1, (*gap2).ga_len);
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*gap2).ga_len {
            *((*gap1).ga_data as *mut fold_T).offset((*gap1).ga_len as isize) =
                *((*gap2).ga_data as *mut fold_T).offset(idx as isize);
            (*((*gap1).ga_data as *mut fold_T).offset((*gap1).ga_len as isize)).fd_top +=
                (*fp1).fd_len;
            (*gap1).ga_len += 1;
            idx += 1;
        }
        (*gap2).ga_len = 0 as ::core::ffi::c_int;
    }
    (*fp1).fd_len += (*fp2).fd_len;
    deleteFoldEntry(
        wp,
        gap,
        fp2.offset_from((*gap).ga_data as *mut fold_T) as ::core::ffi::c_int,
        true_0 != 0,
    );
    fold_changed = true_0 != 0;
}
unsafe extern "C" fn foldlevelIndent(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    let mut buf: *mut buf_T = (*(*flp).wp).w_buffer;
    let mut s: *mut ::core::ffi::c_char = skipwhite(ml_get_buf(buf, lnum));
    if *s as ::core::ffi::c_int == NUL
        || !vim_strchr(
            (*(*flp).wp).w_onebuf_opt.wo_fdi,
            *s as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        (*flp).lvl = if lnum == 1 as linenr_T || lnum == (*buf).b_ml.ml_line_count {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        (*flp).lvl = get_indent_buf(buf, lnum) / get_sw_value(buf);
    }
    (*flp).lvl = if (*flp).lvl
        < (if 0 as OptInt > (*(*flp).wp).w_onebuf_opt.wo_fdn {
            0 as OptInt
        } else {
            (*(*flp).wp).w_onebuf_opt.wo_fdn
        }) as ::core::ffi::c_int
    {
        (*flp).lvl
    } else {
        (if 0 as OptInt > (*(*flp).wp).w_onebuf_opt.wo_fdn {
            0 as OptInt
        } else {
            (*(*flp).wp).w_onebuf_opt.wo_fdn
        }) as ::core::ffi::c_int
    };
}
unsafe extern "C" fn foldlevelDiff(mut flp: *mut fline_T) {
    (*flp).lvl = if diff_infold((*flp).wp, (*flp).lnum + (*flp).off) as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn foldlevelExpr(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    let mut win: *mut win_T = curwin;
    curwin = (*flp).wp;
    curbuf = (*(*flp).wp).w_buffer;
    set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
    (*flp).start = 0 as ::core::ffi::c_int;
    (*flp).had_end = (*flp).end;
    (*flp).end = MAX_LEVEL + 1 as ::core::ffi::c_int;
    if lnum <= 1 as linenr_T {
        (*flp).lvl = 0 as ::core::ffi::c_int;
    }
    let save_keytyped: bool = KeyTyped;
    let mut c: ::core::ffi::c_int = 0;
    let n: ::core::ffi::c_int = eval_foldexpr((*flp).wp, &raw mut c);
    KeyTyped = save_keytyped;
    match c {
        97 => {
            if (*flp).lvl >= 0 as ::core::ffi::c_int {
                (*flp).lvl += n;
                (*flp).lvl_next = (*flp).lvl;
            }
            (*flp).start = n;
        }
        115 => {
            if (*flp).lvl >= 0 as ::core::ffi::c_int {
                if n > (*flp).lvl {
                    (*flp).lvl_next = 0 as ::core::ffi::c_int;
                } else {
                    (*flp).lvl_next = (*flp).lvl - n;
                }
                (*flp).end = (*flp).lvl_next + 1 as ::core::ffi::c_int;
            }
        }
        62 => {
            (*flp).lvl = n;
            (*flp).lvl_next = n;
            (*flp).start = 1 as ::core::ffi::c_int;
        }
        60 => {
            (*flp).lvl_next = if (*flp).lvl < n - 1 as ::core::ffi::c_int {
                (*flp).lvl
            } else {
                n - 1 as ::core::ffi::c_int
            };
            (*flp).end = n;
        }
        61 => {
            (*flp).lvl_next = (*flp).lvl;
        }
        _ => {
            if n < 0 as ::core::ffi::c_int {
                (*flp).lvl_next = (*flp).lvl;
            } else {
                (*flp).lvl_next = n;
            }
            (*flp).lvl = n;
        }
    }
    if (*flp).lvl < 0 as ::core::ffi::c_int {
        if lnum <= 1 as linenr_T {
            (*flp).lvl = 0 as ::core::ffi::c_int;
            (*flp).lvl_next = 0 as ::core::ffi::c_int;
        }
        if lnum == (*curbuf).b_ml.ml_line_count {
            (*flp).lvl_next = 0 as ::core::ffi::c_int;
        }
    }
    curwin = win;
    curbuf = (*curwin).w_buffer;
}
unsafe extern "C" fn parseMarker(mut wp: *mut win_T) {
    foldendmarker = vim_strchr((*wp).w_onebuf_opt.wo_fmr, ',' as ::core::ffi::c_int);
    let c2rust_fresh0 = foldendmarker;
    foldendmarker = foldendmarker.offset(1);
    foldstartmarkerlen = c2rust_fresh0.offset_from((*wp).w_onebuf_opt.wo_fmr) as size_t;
    foldendmarkerlen = strlen(foldendmarker);
}
unsafe extern "C" fn foldlevelMarker(mut flp: *mut fline_T) {
    let mut start_lvl: ::core::ffi::c_int = (*flp).lvl;
    let mut startmarker: *mut ::core::ffi::c_char = (*(*flp).wp).w_onebuf_opt.wo_fmr;
    let mut cstart: ::core::ffi::c_char = *startmarker;
    startmarker = startmarker.offset(1);
    let mut cend: ::core::ffi::c_char = *foldendmarker;
    (*flp).start = 0 as ::core::ffi::c_int;
    (*flp).lvl_next = (*flp).lvl;
    let mut s: *mut ::core::ffi::c_char =
        ml_get_buf((*(*flp).wp).w_buffer, (*flp).lnum + (*flp).off);
    while *s != 0 {
        if *s as ::core::ffi::c_int == cstart as ::core::ffi::c_int
            && strncmp(
                s.offset(1 as ::core::ffi::c_int as isize),
                startmarker,
                foldstartmarkerlen.wrapping_sub(1 as size_t),
            ) == 0 as ::core::ffi::c_int
        {
            s = s.offset(foldstartmarkerlen as isize);
            if ascii_isdigit(*s as ::core::ffi::c_int) {
                let mut n: ::core::ffi::c_int = atoi(s);
                if n > 0 as ::core::ffi::c_int {
                    (*flp).lvl = n;
                    (*flp).lvl_next = n;
                    (*flp).start = if n - start_lvl > 1 as ::core::ffi::c_int {
                        n - start_lvl
                    } else {
                        1 as ::core::ffi::c_int
                    };
                }
            } else {
                (*flp).lvl += 1;
                (*flp).lvl_next += 1;
                (*flp).start += 1;
            }
        } else if *s as ::core::ffi::c_int == cend as ::core::ffi::c_int
            && strncmp(
                s.offset(1 as ::core::ffi::c_int as isize),
                foldendmarker.offset(1 as ::core::ffi::c_int as isize),
                foldendmarkerlen.wrapping_sub(1 as size_t),
            ) == 0 as ::core::ffi::c_int
        {
            s = s.offset(foldendmarkerlen as isize);
            if ascii_isdigit(*s as ::core::ffi::c_int) {
                let mut n_0: ::core::ffi::c_int = atoi(s);
                if n_0 > 0 as ::core::ffi::c_int {
                    (*flp).lvl = n_0;
                    (*flp).lvl_next = n_0 - 1 as ::core::ffi::c_int;
                    (*flp).lvl_next = if (*flp).lvl_next < start_lvl {
                        (*flp).lvl_next
                    } else {
                        start_lvl
                    };
                }
            } else {
                (*flp).lvl_next -= 1;
            }
        } else {
            s = s.offset(utfc_ptr2len(s) as isize);
        }
    }
    (*flp).lvl_next = if (*flp).lvl_next > 0 as ::core::ffi::c_int {
        (*flp).lvl_next
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn foldlevelSyntax(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    (*flp).lvl = syn_get_foldlevel((*flp).wp, lnum);
    (*flp).start = 0 as ::core::ffi::c_int;
    if lnum < (*(*(*flp).wp).w_buffer).b_ml.ml_line_count {
        let mut n: ::core::ffi::c_int = syn_get_foldlevel((*flp).wp, lnum + 1 as linenr_T);
        if n > (*flp).lvl {
            (*flp).start = n - (*flp).lvl;
            (*flp).lvl = n;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn put_folds(mut fd: *mut FILE, mut wp: *mut win_T) -> ::core::ffi::c_int {
    if foldmethodIsManual(wp) {
        if put_line(
            fd,
            b"silent! normal! zE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == FAIL
            || put_folds_recurse(fd, &raw mut (*wp).w_folds, 0 as linenr_T) == FAIL
            || put_line(
                fd,
                b"let &fdl = &fdl\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
    }
    if (*wp).w_fold_manual {
        return put_foldopen_recurse(fd, wp, &raw mut (*wp).w_folds, 0 as linenr_T);
    }
    return OK;
}
unsafe extern "C" fn put_folds_recurse(
    mut fd: *mut FILE,
    mut gap: *mut garray_T,
    mut off: linenr_T,
) -> ::core::ffi::c_int {
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        if put_folds_recurse(fd, &raw mut (*fp).fd_nested, off + (*fp).fd_top) == FAIL {
            return FAIL;
        }
        if fprintf(
            fd,
            b"sil! %ld,%ldfold\0".as_ptr() as *const ::core::ffi::c_char,
            (*fp).fd_top as int64_t + off as int64_t,
            ((*fp).fd_top + off + (*fp).fd_len - 1 as linenr_T) as int64_t,
        ) < 0 as ::core::ffi::c_int
            || put_eol(fd) == FAIL
        {
            return FAIL;
        }
        fp = fp.offset(1);
        i += 1;
    }
    return OK;
}
unsafe extern "C" fn put_foldopen_recurse(
    mut fd: *mut FILE,
    mut wp: *mut win_T,
    mut gap: *mut garray_T,
    mut off: linenr_T,
) -> ::core::ffi::c_int {
    let mut fp: *mut fold_T = (*gap).ga_data as *mut fold_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        if (*fp).fd_flags as ::core::ffi::c_int != FD_LEVEL as ::core::ffi::c_int {
            if !((*fp).fd_nested.ga_len <= 0 as ::core::ffi::c_int) {
                if fprintf(
                    fd,
                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                    (*fp).fd_top as int64_t + off as int64_t,
                ) < 0 as ::core::ffi::c_int
                    || put_eol(fd) == FAIL
                    || put_line(
                        fd,
                        b"sil! normal! zo\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == FAIL
                {
                    return FAIL;
                }
                if put_foldopen_recurse(fd, wp, &raw mut (*fp).fd_nested, off + (*fp).fd_top)
                    == FAIL
                {
                    return FAIL;
                }
                if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int {
                    if put_fold_open_close(fd, fp, off) == FAIL {
                        return FAIL;
                    }
                }
            } else {
                let mut level: ::core::ffi::c_int = foldLevelWin(wp, off + (*fp).fd_top);
                if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int
                    && (*wp).w_onebuf_opt.wo_fdl >= level as OptInt
                    || (*fp).fd_flags as ::core::ffi::c_int != FD_CLOSED as ::core::ffi::c_int
                        && (*wp).w_onebuf_opt.wo_fdl < level as OptInt
                {
                    if put_fold_open_close(fd, fp, off) == FAIL {
                        return FAIL;
                    }
                }
            }
        }
        fp = fp.offset(1);
        i += 1;
    }
    return OK;
}
unsafe extern "C" fn put_fold_open_close(
    mut fd: *mut FILE,
    mut fp: *mut fold_T,
    mut off: linenr_T,
) -> ::core::ffi::c_int {
    if fprintf(
        fd,
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        (*fp).fd_top + off,
    ) < 0 as ::core::ffi::c_int
        || put_eol(fd) == FAIL
        || fprintf(
            fd,
            b"sil! normal! z%c\0".as_ptr() as *const ::core::ffi::c_char,
            if (*fp).fd_flags as ::core::ffi::c_int == FD_CLOSED as ::core::ffi::c_int {
                'c' as ::core::ffi::c_int
            } else {
                'o' as ::core::ffi::c_int
            },
        ) < 0 as ::core::ffi::c_int
        || put_eol(fd) == FAIL
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn foldclosed_both(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut end: bool,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf).b_ml.ml_line_count {
        let mut first: linenr_T = 0;
        let mut last: linenr_T = 0;
        if hasFoldingWin(
            curwin,
            lnum,
            &raw mut first,
            &raw mut last,
            false_0 != 0,
            ::core::ptr::null_mut::<foldinfo_T>(),
        ) {
            (*rettv).vval.v_number = (if end as ::core::ffi::c_int != 0 {
                last
            } else {
                first
            }) as varnumber_T;
            return;
        }
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_foldclosed(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    foldclosed_both(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_foldclosedend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    foldclosed_both(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_foldlevel(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf).b_ml.ml_line_count {
        (*rettv).vval.v_number = foldLevel(lnum) as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_foldtext(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut foldstart: linenr_T = get_vim_var_nr(VV_FOLDSTART) as linenr_T;
    let mut foldend: linenr_T = get_vim_var_nr(VV_FOLDEND) as linenr_T;
    let mut dashes: *mut ::core::ffi::c_char = get_vim_var_str(VV_FOLDDASHES);
    if foldstart > 0 as linenr_T && foldend <= (*curbuf).b_ml.ml_line_count {
        let mut lnum: linenr_T = 0;
        lnum = foldstart;
        while lnum < foldend {
            if !linewhite(lnum) {
                break;
            }
            lnum += 1;
        }
        let mut s: *mut ::core::ffi::c_char = skipwhite(ml_get(lnum));
        if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int
            && (*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int)
        {
            s = skipwhite(s.offset(2 as ::core::ffi::c_int as isize));
            if *skipwhite(s) as ::core::ffi::c_int == NUL && (lnum + 1 as linenr_T) < foldend {
                s = skipwhite(ml_get(lnum + 1 as linenr_T));
                if *s as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    s = skipwhite(s.offset(1 as ::core::ffi::c_int as isize));
                }
            }
        }
        let mut count: ::core::ffi::c_int = foldend as ::core::ffi::c_int
            - foldstart as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        let mut txt: *mut ::core::ffi::c_char = ngettext(
            b"+-%s%3d line: \0".as_ptr() as *const ::core::ffi::c_char,
            b"+-%s%3d lines: \0".as_ptr() as *const ::core::ffi::c_char,
            count as ::core::ffi::c_ulong,
        );
        let mut len: size_t = strlen(txt)
            .wrapping_add(strlen(dashes))
            .wrapping_add(20 as size_t)
            .wrapping_add(strlen(s));
        let mut r: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        snprintf(r, len, txt, dashes, count);
        len = strlen(r);
        strcat(r, s);
        foldtext_cleanup(r.offset(len as isize));
        (*rettv).vval.v_string = r;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_foldtextresult(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 51] = [0; 51];
    static mut entered: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if entered {
        return;
    }
    entered = true_0 != 0;
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    lnum = if lnum > 0 as linenr_T {
        lnum
    } else {
        0 as linenr_T
    };
    let mut info: foldinfo_T = fold_info(curwin, lnum);
    if info.fi_lines > 0 as linenr_T {
        let mut vt: VirtText = VIRTTEXT_EMPTY;
        let mut text: *mut ::core::ffi::c_char = get_foldtext(
            curwin,
            lnum,
            lnum + info.fi_lines - 1 as linenr_T,
            info,
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut vt,
        );
        if text == &raw mut buf as *mut ::core::ffi::c_char {
            text = xstrdup(text);
        }
        if vt.size > 0 as size_t {
            '_c2rust_label: {
                if *text as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"*text == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/fold.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        3284 as ::core::ffi::c_uint,
                        b"void f_foldtextresult(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut i: size_t = 0 as size_t;
            while i < vt.size {
                let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut new_text: *mut ::core::ffi::c_char =
                    next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
                if new_text.is_null() {
                    break;
                }
                new_text = concat_str(text, new_text);
                xfree(text as *mut ::core::ffi::c_void);
                text = new_text;
            }
        }
        clear_virttext(&raw mut vt);
        (*rettv).vval.v_string = text;
    }
    entered = false_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
