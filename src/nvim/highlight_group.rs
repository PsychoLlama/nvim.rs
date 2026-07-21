use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
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
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn abort() -> !;
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xmemrchr(
        src: *const ::core::ffi::c_void,
        c: uint8_t,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn arena_memdupz(
        arena: *mut Arena,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t;
    fn map_put_ref_cstr_t_int(
        map: *mut Map_cstr_t_int,
        key: cstr_t,
        key_alloc: *mut *mut cstr_t,
        new_item: *mut bool,
    ) -> *mut ::core::ffi::c_int;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    static mut p_bg: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
    fn vim_strup(p: *mut ::core::ffi::c_char);
    fn vim_memcpy_up(dst: *mut ::core::ffi::c_char, src: *const ::core::ffi::c_char, n: size_t);
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cursor_mode_uses_syn_id(syn_id: ::core::ffi::c_int) -> bool;
    static mut updating_screen: bool;
    fn decor_provider_invalidate_hl();
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_highlight_group_name_invalid_char: [::core::ffi::c_char; 0];
    static e_highlight_group_name_too_long: [::core::ffi::c_char; 0];
    fn last_set_msg(script_ctx: sctx_T);
    fn do_unlet(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn get_var_value(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ga_set_growsize(gap: *mut garray_T, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    static mut Columns: ::core::ffi::c_int;
    static mut clear_cmdline: bool;
    static mut msg_col: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut t_colors: ::core::ffi::c_int;
    static mut include_none: ::core::ffi::c_int;
    static mut include_default: ::core::ffi::c_int;
    static mut include_link: ::core::ffi::c_int;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut starting: ::core::ffi::c_int;
    static mut msg_silent: ::core::ffi::c_int;
    static mut need_highlight_changed: bool;
    static mut got_int: bool;
    static mut hlf_names: [*const ::core::ffi::c_char; 0];
    static mut highlight_attr: [::core::ffi::c_int; 76];
    static mut highlight_attr_last: [::core::ffi::c_int; 76];
    static mut highlight_user: [::core::ffi::c_int; 9];
    static mut highlight_stlnc: [::core::ffi::c_int; 9];
    static mut cterm_normal_fg_color: ::core::ffi::c_int;
    static mut cterm_normal_bg_color: ::core::ffi::c_int;
    static mut normal_fg: RgbValue;
    static mut normal_bg: RgbValue;
    static mut normal_sp: RgbValue;
    fn hl_get_syn_attr(
        ns_id: ::core::ffi::c_int,
        idx: ::core::ffi::c_int,
        at_en: HlAttrs,
    ) -> ::core::ffi::c_int;
    fn ns_get_hl(
        ns_hl: *mut NS,
        hl_id: ::core::ffi::c_int,
        link: bool,
        nodefault: bool,
    ) -> ::core::ffi::c_int;
    fn hl_get_ui_attr(
        ns_id: ::core::ffi::c_int,
        idx: ::core::ffi::c_int,
        final_id: ::core::ffi::c_int,
        optional: bool,
    ) -> ::core::ffi::c_int;
    fn syn_attr2entry(attr: ::core::ffi::c_int) -> HlAttrs;
    fn hlattrs2dict(
        hl: *mut Dict,
        hl_attrs: *mut Dict,
        ae: HlAttrs,
        use_rgb: bool,
        short_keys: bool,
    );
    fn nlua_set_sctx(current: *mut sctx_T);
    fn msg_source(hl_id: ::core::ffi::c_int);
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn message_filtered(msg: *const ::core::ffi::c_char) -> bool;
    fn msg_clr_eos();
    fn msg_advance(col: ::core::ffi::c_int);
    static mut msg_grid: ScreenGrid;
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn option_was_set(opt_idx: OptIndex) -> bool;
    fn reset_option_was_set(opt_idx: OptIndex);
    fn os_delay(ms: uint64_t, ignoreinput: bool);
    static exestack: GlobalCell<garray_T>;
    fn source_runtime_vim_lua(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ui_rgb_attached() -> bool;
    fn ui_refresh();
    fn ui_default_colors_set();
    fn ui_mode_info_set();
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_hl_group_set(name: String_0, id: Integer);
}
pub type __time_t = ::core::ffi::c_long;
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
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type time_t = __time_t;
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
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
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
pub type NS = handle_T;
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
pub struct C2Rust_Unnamed_2 {
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
    pub data: C2Rust_Unnamed_3,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
pub struct C2Rust_Unnamed_4 {
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
    pub sst_union: C2Rust_Unnamed_5,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub data: C2Rust_Unnamed_6,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub fc_fixvar: [C2Rust_Unnamed_7; 12],
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
pub struct C2Rust_Unnamed_7 {
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
pub type OptionalKeys = uint64_t;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub is_set__highlight_: OptionalKeys,
    pub altfont: Boolean,
    pub blink: Boolean,
    pub bold: Boolean,
    pub conceal: Boolean,
    pub dim: Boolean,
    pub italic: Boolean,
    pub nocombine: Boolean,
    pub overline: Boolean,
    pub reverse: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub undercurl: Boolean,
    pub underdashed: Boolean,
    pub underdotted: Boolean,
    pub underdouble: Boolean,
    pub underline: Boolean,
    pub default_: Boolean,
    pub cterm: Dict,
    pub foreground: Object,
    pub fg: Object,
    pub background: Object,
    pub bg: Object,
    pub ctermfg: Object,
    pub ctermbg: Object,
    pub special: Object,
    pub sp: Object,
    pub link: HLGroupID,
    pub link_global: HLGroupID,
    pub fallback: Boolean,
    pub blend: Integer,
    pub fg_indexed: Boolean,
    pub bg_indexed: Boolean,
    pub force: Boolean,
    pub update: Boolean,
    pub url: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_highlight {
    pub is_set__get_highlight_: OptionalKeys,
    pub id: Integer,
    pub name: String_0,
    pub link: Boolean,
    pub create: Boolean,
}
pub type RgbValue = int32_t;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HL_GLOBAL: C2Rust_Unnamed_14 = 16384;
pub const HL_DEFAULT: C2Rust_Unnamed_14 = 8192;
pub const HL_FG_INDEXED: C2Rust_Unnamed_14 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_14 = 2048;
pub const HL_NOCOMBINE: C2Rust_Unnamed_14 = 1024;
pub const HL_OVERLINE: C2Rust_Unnamed_14 = 131072;
pub const HL_CONCEALED: C2Rust_Unnamed_14 = 65536;
pub const HL_BLINK: C2Rust_Unnamed_14 = 32768;
pub const HL_DIM: C2Rust_Unnamed_14 = 512;
pub const HL_ALTFONT: C2Rust_Unnamed_14 = 256;
pub const HL_STRIKETHROUGH: C2Rust_Unnamed_14 = 128;
pub const HL_STANDOUT: C2Rust_Unnamed_14 = 64;
pub const HL_UNDERDASHED: C2Rust_Unnamed_14 = 40;
pub const HL_UNDERDOTTED: C2Rust_Unnamed_14 = 32;
pub const HL_UNDERDOUBLE: C2Rust_Unnamed_14 = 24;
pub const HL_UNDERCURL: C2Rust_Unnamed_14 = 16;
pub const HL_UNDERLINE: C2Rust_Unnamed_14 = 8;
pub const HL_UNDERLINE_MASK: C2Rust_Unnamed_14 = 56;
pub const HL_ITALIC: C2Rust_Unnamed_14 = 4;
pub const HL_BOLD: C2Rust_Unnamed_14 = 2;
pub const HL_INVERSE: C2Rust_Unnamed_14 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_PRE: C2Rust_Unnamed_15 = 75;
pub const HLF_OK: C2Rust_Unnamed_15 = 74;
pub const HLF_SO: C2Rust_Unnamed_15 = 73;
pub const HLF_SE: C2Rust_Unnamed_15 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_15 = 71;
pub const HLF_TS: C2Rust_Unnamed_15 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_15 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_15 = 68;
pub const HLF_CU: C2Rust_Unnamed_15 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_15 = 66;
pub const HLF_WBR: C2Rust_Unnamed_15 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_15 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_0: C2Rust_Unnamed_15 = 59;
pub const HLF_QFL: C2Rust_Unnamed_15 = 58;
pub const HLF_MC: C2Rust_Unnamed_15 = 57;
pub const HLF_CUL: C2Rust_Unnamed_15 = 56;
pub const HLF_CUC: C2Rust_Unnamed_15 = 55;
pub const HLF_TPF: C2Rust_Unnamed_15 = 54;
pub const HLF_TPS: C2Rust_Unnamed_15 = 53;
pub const HLF_TP: C2Rust_Unnamed_15 = 52;
pub const HLF_PBR: C2Rust_Unnamed_15 = 51;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PSB: C2Rust_Unnamed_15 = 49;
pub const HLF_PSX: C2Rust_Unnamed_15 = 48;
pub const HLF_PNX: C2Rust_Unnamed_15 = 47;
pub const HLF_PSK: C2Rust_Unnamed_15 = 46;
pub const HLF_PNK: C2Rust_Unnamed_15 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_15 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_15 = 43;
pub const HLF_PSI: C2Rust_Unnamed_15 = 42;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_SPL: C2Rust_Unnamed_15 = 40;
pub const HLF_SPR: C2Rust_Unnamed_15 = 39;
pub const HLF_SPC: C2Rust_Unnamed_15 = 38;
pub const HLF_SPB: C2Rust_Unnamed_15 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_15 = 36;
pub const HLF_SC: C2Rust_Unnamed_15 = 35;
pub const HLF_TXA: C2Rust_Unnamed_15 = 34;
pub const HLF_TXD: C2Rust_Unnamed_15 = 33;
pub const HLF_DED: C2Rust_Unnamed_15 = 32;
pub const HLF_CHD: C2Rust_Unnamed_15 = 31;
pub const HLF_ADD: C2Rust_Unnamed_15 = 30;
pub const HLF_FC: C2Rust_Unnamed_15 = 29;
pub const HLF_FL: C2Rust_Unnamed_15 = 28;
pub const HLF_WM: C2Rust_Unnamed_15 = 27;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_VNC: C2Rust_Unnamed_15 = 25;
pub const HLF_V: C2Rust_Unnamed_15 = 24;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_VSP: C2Rust_Unnamed_15 = 22;
pub const HLF_C: C2Rust_Unnamed_15 = 21;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_R: C2Rust_Unnamed_15 = 18;
pub const HLF_CLF: C2Rust_Unnamed_15 = 17;
pub const HLF_CLS: C2Rust_Unnamed_15 = 16;
pub const HLF_CLN: C2Rust_Unnamed_15 = 15;
pub const HLF_LNB: C2Rust_Unnamed_15 = 14;
pub const HLF_LNA: C2Rust_Unnamed_15 = 13;
pub const HLF_N: C2Rust_Unnamed_15 = 12;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_M: C2Rust_Unnamed_15 = 10;
pub const HLF_LC: C2Rust_Unnamed_15 = 9;
pub const HLF_L: C2Rust_Unnamed_15 = 8;
pub const HLF_I: C2Rust_Unnamed_15 = 7;
pub const HLF_E: C2Rust_Unnamed_15 = 6;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_AT: C2Rust_Unnamed_15 = 4;
pub const HLF_TERM: C2Rust_Unnamed_15 = 3;
pub const HLF_EOB: C2Rust_Unnamed_15 = 2;
pub const HLF_8: C2Rust_Unnamed_15 = 1;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLATTRS_DICT_SIZE: C2Rust_Unnamed_16 = 24;
pub type cstr_t = *const ::core::ffi::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_cstr_t_int {
    pub set: Set_cstr_t,
    pub values: *mut ::core::ffi::c_int,
}
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_17 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_17 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_17 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_17 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_17 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_17 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_17 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_17 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_17 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_17 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_17 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_17 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_17 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_17 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_17 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_17 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_17 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_17 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_17 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_17 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_17 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_17 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_17 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_17 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_17 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_17 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_17 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_17 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_17 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_17 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_17 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_17 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_17 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_17 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_17 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_17 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_17 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_17 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_17 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_17 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_17 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_17 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_17 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_17 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_17 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_17 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_17 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_17 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_17 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_17 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_17 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_17 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_17 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_17 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_17 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_17 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_17 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_17 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_17 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_17 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_17 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_17 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_17 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_17 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_17 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_17 = -2;
pub type OptIndex = ::core::ffi::c_int;
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
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
pub type except_T = vim_exception;
pub type auto_event = ::core::ffi::c_uint;
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
pub type event_T = auto_event;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub type AutoPatCmd = AutoPatCmd_S;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_18 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_18 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_18 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_18 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_18 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_18 = 20;
pub const UPD_VALID: C2Rust_Unnamed_18 = 10;
pub type UIExtension = ::core::ffi::c_uint;
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
pub type etype_T = ::core::ffi::c_uint;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAX_HL_ID: C2Rust_Unnamed_20 = 20000;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct color_name_table_T {
    pub name: *mut ::core::ffi::c_char,
    pub color: RgbValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlGroup {
    pub sg_name: *mut ::core::ffi::c_char,
    pub sg_name_u: *mut ::core::ffi::c_char,
    pub sg_cleared: bool,
    pub sg_attr: ::core::ffi::c_int,
    pub sg_link: ::core::ffi::c_int,
    pub sg_deflink: ::core::ffi::c_int,
    pub sg_set: ::core::ffi::c_int,
    pub sg_deflink_sctx: sctx_T,
    pub sg_script_ctx: sctx_T,
    pub sg_cterm: ::core::ffi::c_int,
    pub sg_cterm_fg: ::core::ffi::c_int,
    pub sg_cterm_bg: ::core::ffi::c_int,
    pub sg_cterm_bold: bool,
    pub sg_gui: ::core::ffi::c_int,
    pub sg_rgb_fg: RgbValue,
    pub sg_rgb_bg: RgbValue,
    pub sg_rgb_sp: RgbValue,
    pub sg_rgb_fg_idx: ::core::ffi::c_int,
    pub sg_rgb_bg_idx: ::core::ffi::c_int,
    pub sg_rgb_sp_idx: ::core::ffi::c_int,
    pub sg_blend: ::core::ffi::c_int,
    pub sg_parent: ::core::ffi::c_int,
}
pub const kColorIdxNone: C2Rust_Unnamed_24 = -1;
pub const kColorIdxBg: C2Rust_Unnamed_24 = -4;
pub const kColorIdxFg: C2Rust_Unnamed_24 = -3;
pub const SG_LINK: C2Rust_Unnamed_23 = 8;
pub const kColorIdxHex: C2Rust_Unnamed_24 = -2;
pub const SG_GUI: C2Rust_Unnamed_23 = 4;
pub const SG_CTERM: C2Rust_Unnamed_23 = 2;
pub const DIP_OPT: C2Rust_Unnamed_22 = 16;
pub const DIP_START: C2Rust_Unnamed_22 = 8;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub dest: *mut ::core::ffi::c_int,
    pub val: RgbValue,
    pub name: Object,
}
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_22 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_22 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_22 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_22 = 32;
pub const DIP_ERR: C2Rust_Unnamed_22 = 4;
pub const DIP_DIR: C2Rust_Unnamed_22 = 2;
pub const DIP_ALL: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"_Bool hlgroup2dict(Dict *, NS, int, Arena *)\0",
    )
};
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const KEYSET_OPTIDX_highlight__bg: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__fg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__sp: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__update: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__link: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__name: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__create: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_int = Map_cstr_t_int {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_cstr_t_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut cstr_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const MAX_SYN_NAME: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
static highlight_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
#[no_mangle]
pub static highlight_arena: GlobalCell<Arena> = GlobalCell::new(ARENA_EMPTY);
#[no_mangle]
pub static highlight_unames: GlobalCell<Map_cstr_t_int> = GlobalCell::new(MAP_INIT);
static hl_name_table: GlobalCell<[*mut ::core::ffi::c_char; 18]> = GlobalCell::new([
    b"bold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"standout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"underline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"undercurl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"underdouble\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"underdotted\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"underdashed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"italic\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"reverse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"inverse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"strikethrough\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"altfont\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"dim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"blink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"overline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"nocombine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"NONE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
static hl_attr_table: GlobalCell<[::core::ffi::c_int; 18]> = GlobalCell::new([
    HL_BOLD as ::core::ffi::c_int,
    HL_STANDOUT as ::core::ffi::c_int,
    HL_UNDERLINE as ::core::ffi::c_int,
    HL_UNDERCURL as ::core::ffi::c_int,
    HL_UNDERDOUBLE as ::core::ffi::c_int,
    HL_UNDERDOTTED as ::core::ffi::c_int,
    HL_UNDERDASHED as ::core::ffi::c_int,
    HL_ITALIC as ::core::ffi::c_int,
    HL_INVERSE as ::core::ffi::c_int,
    HL_INVERSE as ::core::ffi::c_int,
    HL_STRIKETHROUGH as ::core::ffi::c_int,
    HL_ALTFONT as ::core::ffi::c_int,
    HL_DIM as ::core::ffi::c_int,
    HL_BLINK as ::core::ffi::c_int,
    HL_CONCEALED as ::core::ffi::c_int,
    HL_OVERLINE as ::core::ffi::c_int,
    HL_NOCOMBINE as ::core::ffi::c_int,
    0 as ::core::ffi::c_int,
]);
static e_highlight_group_name_not_found_str: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E411: Highlight group not found: %s\0",
        )
    });
static e_group_has_settings_highlight_link_ignored: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E414: Group has settings, highlight link ignored\0",
        )
    });
static e_unexpected_equal_sign_str: GlobalCell<[::core::ffi::c_char; 32]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
            *b"E415: Unexpected equal sign: %s\0",
        )
    });
static e_missing_equal_sign_str_2: GlobalCell<[::core::ffi::c_char; 29]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
            *b"E416: Missing equal sign: %s\0",
        )
    });
static e_missing_argument_str: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E417: Missing argument: %s\0")
});
static highlight_init_both: GlobalCell<[*const ::core::ffi::c_char; 175]> = GlobalCell::new([
    b"Cursor            guifg=bg      guibg=fg\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineNr      gui=bold      cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"PmenuMatch        gui=bold      cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"PmenuMatchSel     gui=bold      cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"PmenuSel          gui=reverse   cterm=reverse,underline blend=0\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"RedrawDebugNormal gui=reverse   cterm=reverse\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"TabLineSel        gui=bold      cterm=NONE\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"TermCursor        gui=reverse   cterm=reverse\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Underlined        gui=underline cterm=underline\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"lCursor           guifg=bg      guibg=fg\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link CursorIM         Cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link CursorLineFold   FoldColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link CursorLineSign   SignColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link DiffTextAdd      DiffText\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link EndOfBuffer      NonText\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link FloatBorder      NormalFloat\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link FloatFooter      FloatTitle\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link FloatTitle       Title\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link FoldColumn       SignColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link IncSearch        CurSearch\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link LineNrAbove      LineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link LineNrBelow      LineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link MsgSeparator     StatusLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link MsgArea          NONE\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link NormalNC         NONE\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuExtra       Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuExtraSel    PmenuSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuKind        Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuKindSel     PmenuSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuSbar        Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuBorder        Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PmenuShadow        FloatShadow\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link PmenuShadowThrough FloatShadowThrough\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link PreInsert        Added\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link ComplMatchIns    NONE\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link ComplHint        NonText\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link ComplHintMore    MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Substitute       Search\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link StatusLineTerm   StatusLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link StatusLineTermNC StatusLineNC\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link StderrMsg        ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link StdoutMsg        NONE\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link TabLine          StatusLineNC\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link TabLineFill      TabLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link VertSplit        WinSeparator\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link VisualNOS        Visual\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Whitespace       NonText\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link WildMenu         PmenuSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link WinSeparator     Normal\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Character      Constant\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Number         Constant\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Boolean        Constant\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Float          Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Conditional    Statement\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Repeat         Statement\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Label          Statement\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Keyword        Statement\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Exception      Statement\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Include        PreProc\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Define         PreProc\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Macro          PreProc\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link PreCondit      PreProc\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link StorageClass   Type\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Structure      Type\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Typedef        Type\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Tag            Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link SpecialChar    Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link SpecialComment Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Debug          Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link SpecialKey     Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link Ignore         Normal\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link LspCodeLens                 NonText\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspCodeLensSeparator        LspCodeLens\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspInlayHint                NonText\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspReferenceRead            LspReferenceText\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspReferenceText            Visual\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspReferenceWrite           LspReferenceText\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspReferenceTarget          LspReferenceText\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link LspSignatureActiveParameter Visual\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link SnippetTabstop              Visual\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link SnippetTabstopActive        SnippetTabstop\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticFloatingError    DiagnosticError\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticFloatingWarn     DiagnosticWarn\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticFloatingInfo     DiagnosticInfo\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticFloatingHint     DiagnosticHint\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticFloatingOk       DiagnosticOk\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualTextError DiagnosticError\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualTextWarn  DiagnosticWarn\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualTextInfo  DiagnosticInfo\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualTextHint  DiagnosticHint\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualTextOk    DiagnosticOk\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualLinesError DiagnosticError\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualLinesWarn  DiagnosticWarn\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualLinesInfo  DiagnosticInfo\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualLinesHint  DiagnosticHint\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticVirtualLinesOk    DiagnosticOk\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticSignError        DiagnosticError\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticSignWarn         DiagnosticWarn\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticSignInfo         DiagnosticInfo\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticSignHint         DiagnosticHint\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticSignOk           DiagnosticOk\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link DiagnosticUnnecessary      Comment\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @variable.builtin           Special\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @variable.parameter.builtin Special\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @constant         Constant\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @constant.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @module         Structure\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @module.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @label          Label\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @string             String\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @string.regexp      @string.special\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @string.escape      @string.special\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @string.special     SpecialChar\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @string.special.url Underlined\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @character         Character\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @character.special SpecialChar\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @boolean      Boolean\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @number       Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @number.float Float\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @type         Type\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @type.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @attribute         Macro\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @attribute.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @property          Identifier\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @function         Function\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @function.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @constructor Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @operator    Operator\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @keyword Keyword\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @punctuation         Delimiter\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @punctuation.special Special\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @comment Comment\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @comment.error   DiagnosticError\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @comment.warning DiagnosticWarn\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @comment.note    DiagnosticInfo\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @comment.todo    Todo\0".as_ptr() as *const ::core::ffi::c_char,
    b"@markup.strong        gui=bold          cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"@markup.italic        gui=italic        cterm=italic\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"@markup.strikethrough gui=strikethrough cterm=strikethrough\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"@markup.underline     gui=underline     cterm=underline\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @markup         Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @markup.heading Title\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @markup.link    Underlined\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @diff.plus  Added\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @diff.minus Removed\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @diff.delta Changed\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @tag         Tag\0".as_ptr() as *const ::core::ffi::c_char,
    b"default link @tag.builtin Special\0".as_ptr() as *const ::core::ffi::c_char,
    b"default @markup.heading.1.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underdouble,nocombine ctermbg=NONE ctermfg=NONE cterm=underdouble,nocombine\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"default @markup.heading.2.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underline,nocombine ctermbg=NONE ctermfg=NONE cterm=underline,nocombine\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"default link @lsp.type.class         @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.comment       @comment\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.decorator     @attribute\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.enum          @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.enumMember    @constant\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.event         @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.function      @function\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.interface     @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.keyword       @keyword\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.macro         @constant.macro\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.method        @function.method\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.modifier      @type.qualifier\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.namespace     @module\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.number        @number\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.operator      @operator\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.parameter     @variable.parameter\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.property      @property\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.regexp        @string.regexp\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.string        @string\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.struct        @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.type          @type\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.typeParameter @type.definition\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.type.variable      @variable\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"default link @lsp.mod.deprecated DiagnosticDeprecated\0".as_ptr()
        as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
static highlight_init_light: GlobalCell<[*const ::core::ffi::c_char; 71]> = GlobalCell::new([
    b"Normal guifg=NvimDarkGrey2 guibg=NvimLightGrey2 ctermfg=NONE ctermbg=NONE\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Added                guifg=NvimDarkGreen                                  ctermfg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Changed              guifg=NvimDarkCyan                                   ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ColorColumn                               guibg=NvimLightGrey4            cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Conceal              guifg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"CurSearch            guifg=NvimLightGrey1 guibg=NvimDarkYellow            ctermfg=15 ctermbg=3\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"CursorColumn                              guibg=NvimLightGrey3\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"CursorLine                                guibg=NvimLightGrey3\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiffAdd              guifg=NvimDarkGrey1  guibg=NvimLightGreen            ctermfg=15 ctermbg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiffChange           guifg=NvimDarkGrey1  guibg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiffDelete           guifg=NvimDarkRed                          gui=bold  ctermfg=1 cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiffText             guifg=NvimDarkGrey1  guibg=NvimLightCyan             ctermfg=15 ctermbg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Directory            guifg=NvimDarkCyan                                   ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ErrorMsg             guifg=NvimDarkRed                                    ctermfg=1\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"FloatShadow                               guibg=NvimLightGrey4            ctermbg=0 blend=80\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"FloatShadowThrough                        guibg=NvimLightGrey4            ctermbg=0 blend=100\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Folded               guifg=NvimDarkGrey4  guibg=NvimLightGrey1\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"LineNr               guifg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"MatchParen                                guibg=NvimLightGrey4  gui=bold  cterm=bold,underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ModeMsg              guifg=NvimDarkGreen                                  ctermfg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"MoreMsg              guifg=NvimDarkCyan                                   ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"NonText              guifg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"NormalFloat                               guibg=NvimLightGrey1\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"OkMsg                guifg=NvimDarkGreen                                  ctermfg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Pmenu                                     guibg=NvimLightGrey3            cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"PmenuThumb                                guibg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Question             guifg=NvimDarkCyan                                   ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"QuickFixLine         guifg=NvimDarkCyan                                   ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugClear                          guibg=NvimLightYellow           ctermfg=15 ctermbg=3\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugComposed                       guibg=NvimLightGreen            ctermfg=15 ctermbg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugRecompose                      guibg=NvimLightRed              ctermfg=15 ctermbg=1\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Removed              guifg=NvimDarkRed                                    ctermfg=1\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Search               guifg=NvimDarkGrey1  guibg=NvimLightYellow           ctermfg=15 ctermbg=3\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SignColumn           guifg=NvimLightGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"SpellBad             guisp=NvimDarkRed    gui=undercurl                   cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellCap             guisp=NvimDarkYellow gui=undercurl                   cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellLocal           guisp=NvimDarkGreen  gui=undercurl                   cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellRare            guisp=NvimDarkCyan   gui=undercurl                   cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"StatusLine           guifg=NvimDarkGrey2  guibg=NvimLightGrey4            cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineNC         guifg=NvimDarkGrey3  guibg=NvimLightGrey3            cterm=bold,underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Title                guifg=NvimDarkGrey2                        gui=bold  cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Visual                                    guibg=NvimLightGrey4            ctermfg=15 ctermbg=0\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WarningMsg           guifg=NvimDarkYellow                                 ctermfg=3\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WinBar               guifg=NvimDarkGrey4  guibg=NvimLightGrey1  gui=bold  cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WinBarNC             guifg=NvimDarkGrey4  guibg=NvimLightGrey1            cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Constant   guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Operator   guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"PreProc    guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Type       guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Delimiter  guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comment    guifg=NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"String     guifg=NvimDarkGreen                    ctermfg=2\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Identifier guifg=NvimDarkBlue                     ctermfg=4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Function   guifg=NvimDarkCyan                     ctermfg=6\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Statement  guifg=NvimDarkGrey2 gui=bold           cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Special    guifg=NvimDarkCyan                     ctermfg=6\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Error      guifg=NvimDarkGrey1 guibg=NvimLightRed ctermfg=15 ctermbg=1\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Todo       guifg=NvimDarkGrey2 gui=bold           cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiagnosticError          guifg=NvimDarkRed                      ctermfg=1\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticWarn           guifg=NvimDarkYellow                   ctermfg=3\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticInfo           guifg=NvimDarkCyan                     ctermfg=6\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticHint           guifg=NvimDarkBlue                     ctermfg=4\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticOk             guifg=NvimDarkGreen                    ctermfg=2\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineError guisp=NvimDarkRed    gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineWarn  guisp=NvimDarkYellow gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineInfo  guisp=NvimDarkCyan   gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineHint  guisp=NvimDarkBlue   gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineOk    guisp=NvimDarkGreen  gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticDeprecated     guisp=NvimDarkRed    gui=strikethrough cterm=strikethrough\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"@variable guifg=NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
static highlight_init_dark: GlobalCell<[*const ::core::ffi::c_char; 71]> = GlobalCell::new([
    b"Normal guifg=NvimLightGrey2 guibg=NvimDarkGrey2 ctermfg=NONE ctermbg=NONE\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Added                guifg=NvimLightGreen                                ctermfg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Changed              guifg=NvimLightCyan                                 ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ColorColumn                                guibg=NvimDarkGrey4           cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Conceal              guifg=NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"CurSearch            guifg=NvimDarkGrey1   guibg=NvimLightYellow         ctermfg=0 ctermbg=11\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"CursorColumn                               guibg=NvimDarkGrey3\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"CursorLine                                 guibg=NvimDarkGrey3\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiffAdd              guifg=NvimLightGrey1  guibg=NvimDarkGreen           ctermfg=0 ctermbg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiffChange           guifg=NvimLightGrey1  guibg=NvimDarkGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiffDelete           guifg=NvimLightRed                         gui=bold ctermfg=9 cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiffText             guifg=NvimLightGrey1  guibg=NvimDarkCyan            ctermfg=0 ctermbg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Directory            guifg=NvimLightCyan                                 ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ErrorMsg             guifg=NvimLightRed                                  ctermfg=9\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"FloatShadow                                guibg=NvimDarkGrey4           ctermbg=0 blend=80\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"FloatShadowThrough                         guibg=NvimDarkGrey4           ctermbg=0 blend=100\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Folded               guifg=NvimLightGrey4  guibg=NvimDarkGrey1\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"LineNr               guifg=NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"MatchParen                                 guibg=NvimDarkGrey4  gui=bold cterm=bold,underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"ModeMsg              guifg=NvimLightGreen                                ctermfg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"MoreMsg              guifg=NvimLightCyan                                 ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"NonText              guifg=NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"NormalFloat                                guibg=NvimDarkGrey1\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"OkMsg                guifg=NvimLightGreen                                ctermfg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Pmenu                                      guibg=NvimDarkGrey3           cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"PmenuThumb                                 guibg=NvimDarkGrey4\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Question             guifg=NvimLightCyan                                 ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"QuickFixLine         guifg=NvimLightCyan                                 ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugClear                           guibg=NvimDarkYellow          ctermfg=0 ctermbg=11\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugComposed                        guibg=NvimDarkGreen           ctermfg=0 ctermbg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"RedrawDebugRecompose                       guibg=NvimDarkRed             ctermfg=0 ctermbg=9\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Removed              guifg=NvimLightRed                                  ctermfg=9\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Search               guifg=NvimLightGrey1  guibg=NvimDarkYellow          ctermfg=0 ctermbg=11\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SignColumn           guifg=NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellBad             guisp=NvimLightRed    gui=undercurl                 cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellCap             guisp=NvimLightYellow gui=undercurl                 cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellLocal           guisp=NvimLightGreen  gui=undercurl                 cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"SpellRare            guisp=NvimLightCyan   gui=undercurl                 cterm=undercurl\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"StatusLine           guifg=NvimLightGrey2  guibg=NvimDarkGrey4           cterm=reverse\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineNC         guifg=NvimLightGrey3  guibg=NvimDarkGrey3           cterm=bold,underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Title                guifg=NvimLightGrey2                       gui=bold cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Visual                                     guibg=NvimDarkGrey4           ctermfg=0 ctermbg=15\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WarningMsg           guifg=NvimLightYellow                               ctermfg=11\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WinBar               guifg=NvimLightGrey4  guibg=NvimDarkGrey1  gui=bold cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"WinBarNC             guifg=NvimLightGrey4  guibg=NvimDarkGrey1           cterm=bold\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"Constant   guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Operator   guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"PreProc    guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Type       guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Delimiter  guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comment    guifg=NvimLightGrey4\0".as_ptr() as *const ::core::ffi::c_char,
    b"String     guifg=NvimLightGreen                   ctermfg=10\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Identifier guifg=NvimLightBlue                    ctermfg=12\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Function   guifg=NvimLightCyan                    ctermfg=14\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Statement  guifg=NvimLightGrey2 gui=bold          cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Special    guifg=NvimLightCyan                    ctermfg=14\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Error      guifg=NvimLightGrey1 guibg=NvimDarkRed ctermfg=0 ctermbg=9\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"Todo       guifg=NvimLightGrey2 gui=bold          cterm=bold\0".as_ptr()
        as *const ::core::ffi::c_char,
    b"DiagnosticError          guifg=NvimLightRed                      ctermfg=9\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticWarn           guifg=NvimLightYellow                   ctermfg=11\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticInfo           guifg=NvimLightCyan                     ctermfg=14\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticHint           guifg=NvimLightBlue                     ctermfg=12\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticOk             guifg=NvimLightGreen                    ctermfg=10\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineError guisp=NvimLightRed    gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineWarn  guisp=NvimLightYellow gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineInfo  guisp=NvimLightCyan   gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineHint  guisp=NvimLightBlue   gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticUnderlineOk    guisp=NvimLightGreen  gui=underline     cterm=underline\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"DiagnosticDeprecated     guisp=NvimLightRed    gui=strikethrough cterm=strikethrough\0"
        .as_ptr() as *const ::core::ffi::c_char,
    b"@variable guifg=NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
#[no_mangle]
pub static highlight_init_cmdline: GlobalCell<[*const ::core::ffi::c_char; 141]> =
    GlobalCell::new([
        b"NvimInternalError ctermfg=Red ctermbg=Red guifg=Red guibg=Red\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimAssignment Operator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimPlainAssignment NvimAssignment\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimAugmentedAssignment NvimAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimAssignmentWithAddition NvimAugmentedAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimAssignmentWithSubtraction NvimAugmentedAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimAssignmentWithConcatenation NvimAugmentedAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimOperator Operator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimUnaryOperator NvimOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimUnaryPlus NvimUnaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimUnaryMinus NvimUnaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimNot NvimUnaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimBinaryOperator NvimOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimComparison NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimComparisonModifier NvimComparison\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimBinaryPlus NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimBinaryMinus NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimConcat NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimConcatOrSubscript NvimConcat\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimOr NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimAnd NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimMultiplication NvimBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimDivision NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimMod NvimBinaryOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimTernary NvimOperator\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimTernaryColon NvimTernary\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimParenthesis Delimiter\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimLambda NvimParenthesis\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimNestingParenthesis NvimParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimCallingParenthesis NvimParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimSubscript NvimParenthesis\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimSubscriptBracket NvimSubscript\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimSubscriptColon NvimSubscript\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimCurly NvimSubscript\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimContainer NvimParenthesis\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimDict NvimContainer\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimList NvimContainer\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimIdentifier Identifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimIdentifierScope NvimIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimIdentifierScopeDelimiter NvimIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimIdentifierName NvimIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimIdentifierKey NvimIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimColon Delimiter\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimComma Delimiter\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimArrow Delimiter\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimRegister SpecialChar\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimNumber Number\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimFloat NvimNumber\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimNumberPrefix Type\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimOptionSigil Type\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimOptionName NvimIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimOptionScope NvimIdentifierScope\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimOptionScopeDelimiter NvimIdentifierScopeDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimEnvironmentSigil NvimOptionSigil\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimEnvironmentName NvimIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimString String\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimStringBody NvimString\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimStringQuote NvimString\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimStringSpecial SpecialChar\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimSingleQuote NvimStringQuote\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimSingleQuotedBody NvimStringBody\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimSingleQuotedQuote NvimStringSpecial\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimDoubleQuote NvimStringQuote\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimDoubleQuotedBody NvimStringBody\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimDoubleQuotedEscape NvimStringSpecial\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimFigureBrace NvimInternalError\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimSingleQuotedUnknownEscape NvimInternalError\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimSpacing Normal\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidSingleQuotedUnknownEscape NvimInternalError\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalid Error\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidAssignment NvimInvalid\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidPlainAssignment NvimInvalidAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidAugmentedAssignment NvimInvalidAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidAssignmentWithAddition NvimInvalidAugmentedAssignment\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidAssignmentWithSubtraction NvimInvalidAugmentedAssignment\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidAssignmentWithConcatenation NvimInvalidAugmentedAssignment\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidOperator NvimInvalid\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidUnaryOperator NvimInvalidOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidUnaryPlus NvimInvalidUnaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidUnaryMinus NvimInvalidUnaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidNot NvimInvalidUnaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidBinaryOperator NvimInvalidOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidComparison NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidComparisonModifier NvimInvalidComparison\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidBinaryPlus NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidBinaryMinus NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidConcat NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidConcatOrSubscript NvimInvalidConcat\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidOr NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidAnd NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidMultiplication NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDivision NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidMod NvimInvalidBinaryOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidTernary NvimInvalidOperator\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidTernaryColon NvimInvalidTernary\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDelimiter NvimInvalid\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidParenthesis NvimInvalidDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidLambda NvimInvalidParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidNestingParenthesis NvimInvalidParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidCallingParenthesis NvimInvalidParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSubscript NvimInvalidParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSubscriptBracket NvimInvalidSubscript\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSubscriptColon NvimInvalidSubscript\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidCurly NvimInvalidSubscript\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidContainer NvimInvalidParenthesis\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDict NvimInvalidContainer\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidList NvimInvalidContainer\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidValue NvimInvalid\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidIdentifier NvimInvalidValue\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidIdentifierScope NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidIdentifierScopeDelimiter NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidIdentifierName NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidIdentifierKey NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidColon NvimInvalidDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidComma NvimInvalidDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidArrow NvimInvalidDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidRegister NvimInvalidValue\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidNumber NvimInvalidValue\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidFloat NvimInvalidNumber\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidNumberPrefix NvimInvalidNumber\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidOptionSigil NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidOptionName NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidOptionScope NvimInvalidIdentifierScope\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidOptionScopeDelimiter NvimInvalidIdentifierScopeDelimiter\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidEnvironmentSigil NvimInvalidOptionSigil\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidEnvironmentName NvimInvalidIdentifier\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidString NvimInvalidValue\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimInvalidStringBody NvimStringBody\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidStringQuote NvimInvalidString\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidStringSpecial NvimStringSpecial\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSingleQuote NvimInvalidStringQuote\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSingleQuotedBody NvimInvalidStringBody\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSingleQuotedQuote NvimInvalidStringSpecial\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDoubleQuote NvimInvalidStringQuote\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDoubleQuotedBody NvimInvalidStringBody\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDoubleQuotedEscape NvimInvalidStringSpecial\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidDoubleQuotedUnknownEscape NvimInvalidValue\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidFigureBrace NvimInvalidDelimiter\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"default link NvimInvalidSpacing ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
        b"default link NvimDoubleQuotedUnknownEscape NvimInvalidValue\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ]);
#[no_mangle]
pub unsafe extern "C" fn highlight_num_groups() -> ::core::ffi::c_int {
    return (*highlight_ga.ptr()).ga_len;
}
#[no_mangle]
pub unsafe extern "C" fn highlight_group_name(
    mut id: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_name;
}
#[no_mangle]
pub unsafe extern "C" fn highlight_link_id(mut id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_link;
}
#[no_mangle]
pub unsafe extern "C" fn syn_init_cmdline_highlight(mut reset: bool, mut init: bool) {
    let mut i: size_t = 0 as size_t;
    while !(*highlight_init_cmdline.ptr())[i as usize].is_null() {
        do_highlight((*highlight_init_cmdline.ptr())[i as usize], reset, init);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn init_highlight(mut both: bool, mut reset: bool) {
    static had_both: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut p: *mut ::core::ffi::c_char =
        get_var_value(b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char);
    if !p.is_null() {
        let mut copy_p: *mut ::core::ffi::c_char = xstrdup(p);
        let mut okay: bool = load_colors(copy_p) != 0;
        xfree(copy_p as *mut ::core::ffi::c_void);
        if okay {
            return;
        }
    }
    if both {
        had_both.set(true_0 != 0);
        let pp: *const *const ::core::ffi::c_char =
            highlight_init_both.ptr() as *mut *const ::core::ffi::c_char;
        let mut i: size_t = 0 as size_t;
        while !(*pp.offset(i as isize)).is_null() {
            do_highlight(*pp.offset(i as isize), reset, true_0 != 0);
            i = i.wrapping_add(1);
        }
    } else if !had_both.get() {
        return;
    }
    let pp_0: *const *const ::core::ffi::c_char =
        if *p_bg as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            highlight_init_light.ptr() as *mut *const ::core::ffi::c_char
        } else {
            highlight_init_dark.ptr() as *mut *const ::core::ffi::c_char
        };
    let mut i_0: size_t = 0 as size_t;
    while !(*pp_0.offset(i_0 as isize)).is_null() {
        do_highlight(*pp_0.offset(i_0 as isize), reset, true_0 != 0);
        i_0 = i_0.wrapping_add(1);
    }
    syn_init_cmdline_highlight(false_0 != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn load_colors(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() {
        return OK;
    }
    recursive.set(true_0 != 0);
    let mut buflen: size_t = strlen(name).wrapping_add(12 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
    apply_autocmds(
        EVENT_COLORSCHEMEPRE,
        name,
        (*curbuf).b_fname,
        false_0 != 0,
        curbuf,
    );
    snprintf(
        buf,
        buflen,
        b"colors/%s.*\0".as_ptr() as *const ::core::ffi::c_char,
        name,
    );
    let mut retval: ::core::ffi::c_int = source_runtime_vim_lua(
        buf,
        DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int,
    );
    xfree(buf as *mut ::core::ffi::c_void);
    if retval == OK {
        apply_autocmds(
            EVENT_COLORSCHEME,
            name,
            (*curbuf).b_fname,
            false_0 != 0,
            curbuf,
        );
    }
    recursive.set(false_0 != 0);
    return retval;
}
static color_names: GlobalCell<[*mut ::core::ffi::c_char; 28]> = GlobalCell::new([
    b"Black\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkMagenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Brown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkYellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Gray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Grey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"DarkGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Blue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Green\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Cyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Red\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Magenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightMagenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"Yellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"LightYellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"White\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"NONE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
static color_numbers_16: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_88: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    32 as ::core::ffi::c_int,
    72 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    43 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    61 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    63 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    74 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    75 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    78 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_256: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    130 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    81 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    121 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    159 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    224 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    225 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    229 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_8: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
unsafe extern "C" fn lookup_color(
    idx: ::core::ffi::c_int,
    foreground: bool,
    boldp: *mut TriState,
) -> ::core::ffi::c_int {
    let mut color: ::core::ffi::c_int = (*color_numbers_16.ptr())[idx as usize];
    if color < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if t_colors == 8 as ::core::ffi::c_int {
        color = (*color_numbers_8.ptr())[idx as usize];
        if foreground {
            if color & 8 as ::core::ffi::c_int != 0 {
                *boldp = kTrue;
            } else {
                *boldp = kFalse;
            }
        }
        color &= 7 as ::core::ffi::c_int;
    } else if t_colors == 16 as ::core::ffi::c_int {
        color = (*color_numbers_8.ptr())[idx as usize];
    } else if t_colors == 88 as ::core::ffi::c_int {
        color = (*color_numbers_88.ptr())[idx as usize];
    } else if t_colors >= 256 as ::core::ffi::c_int {
        color = (*color_numbers_256.ptr())[idx as usize];
    }
    return color;
}
#[no_mangle]
pub unsafe extern "C" fn set_hl_group(
    mut id: ::core::ffi::c_int,
    mut attrs: HlAttrs,
    mut dict: *mut KeyDict_highlight,
    mut link_id: ::core::ffi::c_int,
) {
    let mut idx: ::core::ffi::c_int = id - 1 as ::core::ffi::c_int;
    let mut is_default: bool = attrs.rgb_ae_attr & HL_DEFAULT as ::core::ffi::c_int as int32_t != 0;
    if is_default as ::core::ffi::c_int != 0
        && hl_has_settings(idx, true_0 != 0) as ::core::ffi::c_int != 0
        && !(*dict).force
    {
        return;
    }
    let mut g: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    (*g).sg_cleared = false_0 != 0;
    let mut old_link: ::core::ffi::c_int = (*g).sg_link;
    if link_id > 0 as ::core::ffi::c_int {
        (*g).sg_link = link_id;
        (*g).sg_script_ctx = current_sctx;
        (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*g).sg_script_ctx);
        (*g).sg_set |= SG_LINK as ::core::ffi::c_int;
        if is_default {
            (*g).sg_deflink = link_id;
            (*g).sg_deflink_sctx = current_sctx;
            (*g).sg_deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum;
            nlua_set_sctx(&raw mut (*g).sg_deflink_sctx);
        }
    } else {
        (*g).sg_link = 0 as ::core::ffi::c_int;
    }
    let mut update: bool = (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__update
        != 0 as ::core::ffi::c_ulonglong
        && (*dict).update as ::core::ffi::c_int != 0;
    (*g).sg_gui =
        (attrs.rgb_ae_attr & !(HL_DEFAULT as ::core::ffi::c_int as int32_t)) as ::core::ffi::c_int;
    (*g).sg_rgb_fg = attrs.rgb_fg_color;
    (*g).sg_rgb_bg = attrs.rgb_bg_color;
    (*g).sg_rgb_sp = attrs.rgb_sp_color;
    let mut cattrs: [C2Rust_Unnamed_21; 4] = [
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_fg_idx,
            val: (*g).sg_rgb_fg,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fg
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).fg
            } else {
                (*dict).foreground
            },
        },
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_bg_idx,
            val: (*g).sg_rgb_bg,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bg
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).bg
            } else {
                (*dict).background
            },
        },
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_sp_idx,
            val: (*g).sg_rgb_sp,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__sp
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).sp
            } else {
                (*dict).special
            },
        },
        C2Rust_Unnamed_21 {
            dest: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            val: -1 as RgbValue,
            name: object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_0 { boolean: false },
            },
        },
    ];
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !cattrs[j as usize].dest.is_null() {
        if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if cattrs[j as usize].val < 0 as RgbValue {
                *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
            } else if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                && cattrs[j as usize].name.data.string.size != 0
            {
                name_to_color(
                    cattrs[j as usize].name.data.string.data,
                    cattrs[j as usize].dest,
                );
            } else {
                *cattrs[j as usize].dest = kColorIdxHex as ::core::ffi::c_int;
            }
        } else if !update {
            *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
        } else if old_link > 0 as ::core::ffi::c_int && cattrs[j as usize].val >= 0 as RgbValue {
            let mut linked: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((old_link - 1 as ::core::ffi::c_int) as isize);
            let mut linked_idx: ::core::ffi::c_int = if j == 0 as ::core::ffi::c_int {
                (*linked).sg_rgb_fg_idx
            } else if j == 1 as ::core::ffi::c_int {
                (*linked).sg_rgb_bg_idx
            } else {
                (*linked).sg_rgb_sp_idx
            };
            *cattrs[j as usize].dest = if linked_idx != kColorIdxNone as ::core::ffi::c_int {
                linked_idx
            } else {
                kColorIdxHex as ::core::ffi::c_int
            };
        }
        j += 1;
    }
    (*g).sg_cterm = (attrs.cterm_ae_attr & !(HL_DEFAULT as ::core::ffi::c_int as int32_t))
        as ::core::ffi::c_int;
    (*g).sg_cterm_bg = attrs.cterm_bg_color as ::core::ffi::c_int;
    (*g).sg_cterm_fg = attrs.cterm_fg_color as ::core::ffi::c_int;
    (*g).sg_cterm_bold = (*g).sg_cterm & HL_BOLD as ::core::ffi::c_int != 0;
    if attrs.hl_blend != -1 as int32_t {
        (*g).sg_blend = attrs.hl_blend as ::core::ffi::c_int;
    } else if !update {
        (*g).sg_blend = -1 as ::core::ffi::c_int;
    }
    (*g).sg_script_ctx = current_sctx;
    (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    nlua_set_sctx(&raw mut (*g).sg_script_ctx);
    (*g).sg_attr = hl_get_syn_attr(0 as ::core::ffi::c_int, id, attrs);
    if strcmp(
        (*g).sg_name_u,
        b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        cterm_normal_fg_color = (*g).sg_cterm_fg;
        cterm_normal_bg_color = (*g).sg_cterm_bg;
        let mut did_changed: bool = false_0 != 0;
        if normal_bg != (*g).sg_rgb_bg || normal_fg != (*g).sg_rgb_fg || normal_sp != (*g).sg_rgb_sp
        {
            did_changed = true_0 != 0;
        }
        normal_fg = (*g).sg_rgb_fg;
        normal_bg = (*g).sg_rgb_bg;
        normal_sp = (*g).sg_rgb_sp;
        if did_changed {
            highlight_attr_set_all();
        }
        ui_default_colors_set();
    } else if cursor_mode_uses_syn_id(id) {
        ui_mode_info_set();
    }
    if !updating_screen {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    need_highlight_changed = true_0 != 0;
}
unsafe extern "C" fn set_gui_color(
    mut idx: ::core::ffi::c_int,
    mut init: bool,
    mut arg: *const ::core::ffi::c_char,
    mut color: *mut RgbValue,
    mut color_idx: *mut ::core::ffi::c_int,
) -> bool {
    if init as ::core::ffi::c_int != 0
        && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set
            & SG_GUI as ::core::ffi::c_int
            != 0
    {
        return false_0 != 0;
    }
    if !init {
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set |=
            SG_GUI as ::core::ffi::c_int;
    }
    let mut old_color: RgbValue = *color;
    let mut old_idx: ::core::ffi::c_int = *color_idx;
    if strcmp(arg, b"NONE\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int {
        *color = name_to_color(arg, color_idx);
    } else {
        *color = -1 as ::core::ffi::c_int as RgbValue;
        *color_idx = kColorIdxNone as ::core::ffi::c_int;
    }
    return *color != old_color || *color_idx != old_idx;
}
#[no_mangle]
pub unsafe extern "C" fn do_highlight(
    mut line: *const ::core::ffi::c_char,
    forceit: bool,
    init: bool,
) {
    if !init && ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*highlight_ga.ptr()).ga_len && !got_int {
            highlight_list_one(i);
            i += 1;
        }
        return;
    }
    let mut dodefault: bool = false_0 != 0;
    let mut name_end: *const ::core::ffi::c_char = skiptowhite(line);
    let mut linep: *const ::core::ffi::c_char = skipwhite(name_end);
    if strncmp(
        line,
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        dodefault = true_0 != 0;
        line = linep;
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }
    let mut doclear: bool = false_0 != 0;
    let mut dolink: bool = false_0 != 0;
    if strncmp(
        line,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        doclear = true_0 != 0;
    } else if strncmp(
        line,
        b"link\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        dolink = true_0 != 0;
    }
    if !doclear && !dolink && ends_excmd(*linep as uint8_t as ::core::ffi::c_int) != 0 {
        let mut id: ::core::ffi::c_int =
            syn_name2id_len(line, name_end.offset_from(line) as size_t);
        if id == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    (e_highlight_group_name_not_found_str.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                line,
            );
        } else {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            highlight_list_one(id);
        }
        return;
    }
    if dolink {
        let mut from_start: *const ::core::ffi::c_char = linep;
        let mut to_id: ::core::ffi::c_int = 0;
        let mut hlgroup: *mut HlGroup = ::core::ptr::null_mut::<HlGroup>();
        let mut from_end: *const ::core::ffi::c_char = skiptowhite(from_start);
        let mut to_start: *const ::core::ffi::c_char = skipwhite(from_end);
        let mut to_end: *const ::core::ffi::c_char = skiptowhite(to_start);
        if ends_excmd(*from_start as uint8_t as ::core::ffi::c_int) != 0
            || ends_excmd(*to_start as uint8_t as ::core::ffi::c_int) != 0
        {
            semsg(
                gettext(
                    b"E412: Not enough arguments: \":highlight link %s\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                from_start,
            );
            return;
        }
        if ends_excmd(*skipwhite(to_end) as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(
                    b"E413: Too many arguments: \":highlight link %s\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                from_start,
            );
            return;
        }
        let mut from_id: ::core::ffi::c_int =
            syn_check_group(from_start, from_end.offset_from(from_start) as size_t);
        if strncmp(
            to_start,
            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_id = 0 as ::core::ffi::c_int;
        } else {
            to_id = syn_check_group(to_start, to_end.offset_from(to_start) as size_t);
        }
        if from_id > 0 as ::core::ffi::c_int {
            hlgroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((from_id - 1 as ::core::ffi::c_int) as isize);
            if dodefault as ::core::ffi::c_int != 0
                && (forceit as ::core::ffi::c_int != 0
                    || (*hlgroup).sg_deflink == 0 as ::core::ffi::c_int)
            {
                (*hlgroup).sg_deflink = to_id;
                (*hlgroup).sg_deflink_sctx = current_sctx;
                (*hlgroup).sg_deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data
                    as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
                nlua_set_sctx(&raw mut (*hlgroup).sg_deflink_sctx);
            }
        }
        if from_id > 0 as ::core::ffi::c_int
            && (!init || (*hlgroup).sg_set == 0 as ::core::ffi::c_int)
        {
            if to_id > 0 as ::core::ffi::c_int
                && !forceit
                && !init
                && hl_has_settings(from_id - 1 as ::core::ffi::c_int, dodefault)
                    as ::core::ffi::c_int
                    != 0
            {
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                    && !dodefault
                {
                    emsg(gettext(
                        (e_group_has_settings_highlight_link_ignored.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                }
            } else if (*hlgroup).sg_link != to_id
                || (*hlgroup).sg_script_ctx.sc_sid != current_sctx.sc_sid
                || (*hlgroup).sg_cleared as ::core::ffi::c_int != 0
            {
                if !init {
                    (*hlgroup).sg_set |= SG_LINK as ::core::ffi::c_int;
                }
                (*hlgroup).sg_link = to_id;
                (*hlgroup).sg_script_ctx = current_sctx;
                (*hlgroup).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
                nlua_set_sctx(&raw mut (*hlgroup).sg_script_ctx);
                (*hlgroup).sg_cleared = false_0 != 0;
                redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
                need_highlight_changed = true_0 != 0;
            }
        }
        return;
    }
    if doclear {
        line = linep;
        if ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
            do_unlet(
                b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
            restore_cterm_colors();
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < (*highlight_ga.ptr()).ga_len {
                highlight_clear(j);
                j += 1;
            }
            init_highlight(true_0 != 0, true_0 != 0);
            highlight_changed();
            redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
            return;
        }
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }
    let mut id_0: ::core::ffi::c_int = syn_check_group(line, name_end.offset_from(line) as size_t);
    if id_0 == 0 as ::core::ffi::c_int {
        return;
    }
    let mut idx: ::core::ffi::c_int = id_0 - 1 as ::core::ffi::c_int;
    if dodefault as ::core::ffi::c_int != 0
        && hl_has_settings(idx, true_0 != 0) as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut item_before: HlGroup =
        *((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    let mut is_normal_group: bool = strcmp(
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_name_u,
        b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int;
    if doclear as ::core::ffi::c_int != 0
        || forceit as ::core::ffi::c_int != 0 && init as ::core::ffi::c_int != 0
    {
        highlight_clear(idx);
        if !doclear {
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set =
                0 as ::core::ffi::c_int;
        }
    }
    let mut did_change: bool = false_0 != 0;
    let mut error: bool = false_0 != 0;
    let mut key: [::core::ffi::c_char; 64] = [0; 64];
    let mut arg: [::core::ffi::c_char; 512] = [0; 512];
    if !doclear {
        let mut arg_start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        while ends_excmd(*linep as uint8_t as ::core::ffi::c_int) == 0 {
            let mut key_start: *const ::core::ffi::c_char = linep;
            if *linep as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                semsg(
                    gettext(
                        (e_unexpected_equal_sign_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    key_start,
                );
                error = true_0 != 0;
                break;
            } else {
                while *linep as ::core::ffi::c_int != 0
                    && !ascii_iswhite(*linep as ::core::ffi::c_int)
                    && *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int
                {
                    linep = linep.offset(1);
                }
                let mut key_len: size_t = linep.offset_from(key_start) as size_t;
                if key_len
                    > ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as usize)
                {
                    emsg(gettext(
                        b"E423: Illegal argument\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    error = true_0 != 0;
                    break;
                } else {
                    vim_memcpy_up(&raw mut key as *mut ::core::ffi::c_char, key_start, key_len);
                    key[key_len as usize] = NUL as ::core::ffi::c_char;
                    linep = skipwhite(linep);
                    if strcmp(
                        &raw mut key as *mut ::core::ffi::c_char,
                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if !init
                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                .offset(idx as isize))
                            .sg_set
                                == 0 as ::core::ffi::c_int
                        {
                            if !init {
                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                    .offset(idx as isize))
                                .sg_set |=
                                    SG_CTERM as ::core::ffi::c_int + SG_GUI as ::core::ffi::c_int;
                            }
                            highlight_clear(idx);
                        }
                    } else if *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                (e_missing_equal_sign_str_2.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            key_start,
                        );
                        error = true_0 != 0;
                        break;
                    } else {
                        linep = linep.offset(1);
                        linep = skipwhite(linep);
                        if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                            linep = linep.offset(1);
                            arg_start = linep;
                            linep = strchr(linep, '\'' as ::core::ffi::c_int);
                            if linep.is_null() {
                                semsg(
                                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                    key_start,
                                );
                                error = true_0 != 0;
                                break;
                            }
                        } else {
                            arg_start = linep;
                            linep = skiptowhite(linep);
                        }
                        if linep == arg_start {
                            semsg(
                                gettext(
                                    (e_missing_argument_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                key_start,
                            );
                            error = true_0 != 0;
                            break;
                        } else {
                            let mut arg_len: size_t = linep.offset_from(arg_start) as size_t;
                            if arg_len
                                > ::core::mem::size_of::<[::core::ffi::c_char; 512]>()
                                    .wrapping_sub(1 as usize)
                            {
                                emsg(gettext(b"E423: Illegal argument\0".as_ptr()
                                    as *const ::core::ffi::c_char));
                                error = true_0 != 0;
                                break;
                            } else {
                                memcpy(
                                    &raw mut arg as *mut ::core::ffi::c_char
                                        as *mut ::core::ffi::c_void,
                                    arg_start as *const ::core::ffi::c_void,
                                    arg_len,
                                );
                                arg[arg_len as usize] = NUL as ::core::ffi::c_char;
                                if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                                    linep = linep.offset(1);
                                }
                                if strcmp(
                                    &raw mut key as *mut ::core::ffi::c_char,
                                    b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0 as ::core::ffi::c_int
                                    || strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"CTERM\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    || strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUI\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                {
                                    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut i_0: ::core::ffi::c_int = 0;
                                    while arg[off as usize] as ::core::ffi::c_int != NUL {
                                        i_0 = ::core::mem::size_of::<[::core::ffi::c_int; 18]>()
                                            .wrapping_div(
                                                ::core::mem::size_of::<::core::ffi::c_int>(),
                                            )
                                            .wrapping_div(
                                                (::core::mem::size_of::<[::core::ffi::c_int; 18]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ::core::ffi::c_int,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            )
                                            as ::core::ffi::c_int;
                                        loop {
                                            i_0 -= 1;
                                            if i_0 < 0 as ::core::ffi::c_int {
                                                break;
                                            }
                                            let mut len: ::core::ffi::c_int = strlen(
                                                (*hl_name_table.ptr())[i_0 as usize]
                                                    as *const ::core::ffi::c_char,
                                            )
                                                as ::core::ffi::c_int;
                                            if strncasecmp(
                                                (&raw mut arg as *mut ::core::ffi::c_char)
                                                    .offset(off as isize),
                                                (*hl_name_table.ptr())[i_0 as usize],
                                                len as size_t,
                                            ) != 0 as ::core::ffi::c_int
                                            {
                                                continue;
                                            }
                                            if (*hl_attr_table.ptr())[i_0 as usize]
                                                & HL_UNDERLINE_MASK as ::core::ffi::c_int
                                                != 0
                                            {
                                                attr &= !(HL_UNDERLINE_MASK as ::core::ffi::c_int);
                                            }
                                            attr |= (*hl_attr_table.ptr())[i_0 as usize];
                                            off += len;
                                            break;
                                        }
                                        if i_0 < 0 as ::core::ffi::c_int {
                                            semsg(
                                                gettext(b"E418: Illegal value: %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                            );
                                            error = true_0 != 0;
                                            break;
                                        } else if arg[off as usize] as ::core::ffi::c_int
                                            == ',' as ::core::ffi::c_int
                                        {
                                            off += 1;
                                        }
                                    }
                                    if error {
                                        break;
                                    }
                                    if *(&raw mut key as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        == 'C' as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_CTERM as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_CTERM as ::core::ffi::c_int;
                                            }
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_cterm = attr;
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_cterm_bold = false_0 != 0;
                                        }
                                    } else if *(&raw mut key as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        == 'G' as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_GUI as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_GUI as ::core::ffi::c_int;
                                            }
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_gui = attr;
                                        }
                                    }
                                } else if strcmp(
                                    &raw mut key as *mut ::core::ffi::c_char,
                                    b"FONT\0".as_ptr() as *const ::core::ffi::c_char,
                                ) != 0 as ::core::ffi::c_int
                                {
                                    if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"CTERMFG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"CTERMBG\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_CTERM as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_CTERM as ::core::ffi::c_int;
                                            }
                                            if key[5 as ::core::ffi::c_int as usize]
                                                as ::core::ffi::c_int
                                                == 'F' as ::core::ffi::c_int
                                                && (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bold
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm &= !(HL_BOLD as ::core::ffi::c_int);
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bold = false_0 != 0;
                                            }
                                            let mut color: ::core::ffi::c_int = 0;
                                            if ascii_isdigit(
                                                *(&raw mut arg as *mut ::core::ffi::c_char)
                                                    as ::core::ffi::c_int,
                                            ) {
                                                color =
                                                    atoi(&raw mut arg as *mut ::core::ffi::c_char);
                                            } else if strcasecmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"fg\0".as_ptr() as *const ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if cterm_normal_fg_color != 0 {
                                                    color = cterm_normal_fg_color
                                                        - 1 as ::core::ffi::c_int;
                                                } else {
                                                    emsg(gettext(
                                                        b"E419: FG color unknown\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ));
                                                    error = true_0 != 0;
                                                    break;
                                                }
                                            } else if strcasecmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"bg\0".as_ptr() as *const ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if cterm_normal_bg_color > 0 as ::core::ffi::c_int {
                                                    color = cterm_normal_bg_color
                                                        - 1 as ::core::ffi::c_int;
                                                } else {
                                                    emsg(gettext(
                                                        b"E420: BG color unknown\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ));
                                                    error = true_0 != 0;
                                                    break;
                                                }
                                            } else {
                                                let mut off_0: ::core::ffi::c_int =
                                                    if (*(&raw mut arg as *mut ::core::ffi::c_char)
                                                        as ::core::ffi::c_int)
                                                        < 'a' as ::core::ffi::c_int
                                                        || *(&raw mut arg
                                                            as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                            > 'z' as ::core::ffi::c_int
                                                    {
                                                        *(&raw mut arg as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                    } else {
                                                        *(&raw mut arg as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                            - ('a' as ::core::ffi::c_int
                                                                - 'A' as ::core::ffi::c_int)
                                                    };
                                                let mut i_1: ::core::ffi::c_int = 0;
                                                i_1 = ::core::mem::size_of::<
                                                    [*mut ::core::ffi::c_char; 28],
                                                >(
                                                )
                                                .wrapping_div(::core::mem::size_of::<
                                                    *mut ::core::ffi::c_char,
                                                >(
                                                ))
                                                .wrapping_div(
                                                    (::core::mem::size_of::<
                                                        [*mut ::core::ffi::c_char; 28],
                                                    >(
                                                    )
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        *mut ::core::ffi::c_char,
                                                    >(
                                                    )) == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                )
                                                    as ::core::ffi::c_int;
                                                loop {
                                                    i_1 -= 1;
                                                    if i_1 < 0 as ::core::ffi::c_int {
                                                        break;
                                                    }
                                                    if off_0
                                                        == *(*color_names.ptr())[i_1 as usize]
                                                            .offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                            as ::core::ffi::c_int
                                                        && strcasecmp(
                                                            (&raw mut arg
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    1 as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                            (*color_names.ptr())[i_1 as usize]
                                                                .offset(
                                                                    1 as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                        ) == 0 as ::core::ffi::c_int
                                                    {
                                                        break;
                                                    }
                                                }
                                                if i_1 < 0 as ::core::ffi::c_int {
                                                    semsg(
                                                        gettext(
                                                            b"E421: Color name or number not recognized: %s\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        key_start,
                                                    );
                                                    error = true_0 != 0;
                                                    break;
                                                } else {
                                                    let mut bold: TriState = kNone;
                                                    color = lookup_color(
                                                        i_1,
                                                        key[5 as ::core::ffi::c_int as usize]
                                                            as ::core::ffi::c_int
                                                            == 'F' as ::core::ffi::c_int,
                                                        &raw mut bold,
                                                    );
                                                    if bold as ::core::ffi::c_int
                                                        == kTrue as ::core::ffi::c_int
                                                    {
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm |= HL_BOLD as ::core::ffi::c_int;
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm_bold = true_0 != 0;
                                                    } else if bold as ::core::ffi::c_int
                                                        == kFalse as ::core::ffi::c_int
                                                    {
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm &=
                                                            !(HL_BOLD as ::core::ffi::c_int);
                                                    }
                                                }
                                            }
                                            if key[5 as ::core::ffi::c_int as usize]
                                                as ::core::ffi::c_int
                                                == 'F' as ::core::ffi::c_int
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_fg = color + 1 as ::core::ffi::c_int;
                                                if is_normal_group {
                                                    cterm_normal_fg_color =
                                                        color + 1 as ::core::ffi::c_int;
                                                }
                                            } else {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bg = color + 1 as ::core::ffi::c_int;
                                                if is_normal_group {
                                                    cterm_normal_bg_color =
                                                        color + 1 as ::core::ffi::c_int;
                                                    if !ui_rgb_attached() {
                                                        if color >= 0 as ::core::ffi::c_int {
                                                            let mut dark: ::core::ffi::c_int =
                                                                -1 as ::core::ffi::c_int;
                                                            if t_colors < 16 as ::core::ffi::c_int {
                                                                dark = (color
                                                                    == 0 as ::core::ffi::c_int
                                                                    || color
                                                                        == 4 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                            } else if color
                                                                < 16 as ::core::ffi::c_int
                                                            {
                                                                dark = (color
                                                                    < 7 as ::core::ffi::c_int
                                                                    || color
                                                                        == 8 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                            }
                                                            if dark != -1 as ::core::ffi::c_int
                                                                && dark
                                                                    != (*p_bg as ::core::ffi::c_int
                                                                        == 'd'
                                                                            as ::core::ffi::c_int)
                                                                        as ::core::ffi::c_int
                                                                && !option_was_set(kOptBackground)
                                                            {
                                                                set_option_value_give_err(
                                                                    kOptBackground,
                                                                    OptVal {
                                                                        type_0: kOptValTypeString,
                                                                        data: OptValData {
                                                                            string: cstr_as_string(
                                                                                if dark != 0 {
                                                                                    b"dark\0".as_ptr() as *const ::core::ffi::c_char
                                                                                } else {
                                                                                    b"light\0".as_ptr() as *const ::core::ffi::c_char
                                                                                },
                                                                            ),
                                                                        },
                                                                    },
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                                reset_option_was_set(
                                                                    kOptBackground,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUIFG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_fg,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_fg_idx,
                                        );
                                        if is_normal_group {
                                            normal_fg = (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_fg;
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUIBG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_bg,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_bg_idx,
                                        );
                                        if is_normal_group {
                                            normal_bg = (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_bg;
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUISP\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_sp,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_sp_idx,
                                        );
                                        if is_normal_group {
                                            normal_sp = (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_sp;
                                        }
                                    } else if !(strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"START\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"STOP\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int)
                                    {
                                        if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"BLEND\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            if strcmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) != 0 as ::core::ffi::c_int
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_blend = strtol(
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(
                                                    ),
                                                    10 as ::core::ffi::c_int,
                                                )
                                                    as ::core::ffi::c_int;
                                            } else {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_blend = -1 as ::core::ffi::c_int;
                                            }
                                        } else {
                                            semsg(
                                                gettext(b"E423: Illegal argument: %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                key_start,
                                            );
                                            error = true_0 != 0;
                                            break;
                                        }
                                    }
                                }
                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                    .offset(idx as isize))
                                .sg_cleared = false_0 != 0;
                                if !init
                                    || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                        .offset(idx as isize))
                                    .sg_set
                                        & SG_LINK as ::core::ffi::c_int
                                        == 0
                                {
                                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                        .offset(idx as isize))
                                    .sg_link = 0 as ::core::ffi::c_int;
                                }
                                linep = skipwhite(linep);
                            }
                        }
                    }
                }
            }
        }
    }
    let mut did_highlight_changed: bool = false_0 != 0;
    if !error && is_normal_group as ::core::ffi::c_int != 0 {
        highlight_attr_set_all();
        if !ui_has(kUILinegrid) && starting == 0 as ::core::ffi::c_int {
            ui_refresh();
        } else {
            ui_default_colors_set();
        }
        did_highlight_changed = true_0 != 0;
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    } else {
        set_hl_attr(idx);
    }
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_script_ctx =
        current_sctx;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
        .sg_script_ctx
        .sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    nlua_set_sctx(
        &raw mut (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
            .sg_script_ctx,
    );
    if (did_change as ::core::ffi::c_int != 0
        || memcmp(
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)
                as *const ::core::ffi::c_void,
            &raw mut item_before as *const ::core::ffi::c_void,
            ::core::mem::size_of::<HlGroup>(),
        ) != 0 as ::core::ffi::c_int)
        && !did_highlight_changed
    {
        if !updating_screen {
            redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
        }
        need_highlight_changed = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn restore_cterm_colors() {
    normal_fg = -1 as ::core::ffi::c_int as RgbValue;
    normal_bg = -1 as ::core::ffi::c_int as RgbValue;
    normal_sp = -1 as ::core::ffi::c_int as RgbValue;
    cterm_normal_fg_color = 0 as ::core::ffi::c_int;
    cterm_normal_bg_color = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn hl_has_settings(mut idx: ::core::ffi::c_int, mut check_link: bool) -> bool {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
        as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr
            != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_cterm_fg
                != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_cterm_bg
                != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_fg_idx
                != kColorIdxNone as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_bg_idx
                != kColorIdxNone as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_sp_idx
                != kColorIdxNone as ::core::ffi::c_int
            || check_link as ::core::ffi::c_int != 0
                && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set
                    & SG_LINK as ::core::ffi::c_int
                    != 0);
}
unsafe extern "C" fn highlight_clear(mut idx: ::core::ffi::c_int) {
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared =
        true_0 != 0;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bold =
        false_0 != 0;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_fg =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bg =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_gui =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_blend =
        -1 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_link =
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_script_ctx =
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink_sctx;
}
pub const LIST_ATTR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LIST_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LIST_INT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn highlight_list_one(id: ::core::ffi::c_int) {
    let mut sgp: *const HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((id - 1 as ::core::ffi::c_int) as isize);
    let mut didh: bool = false_0 != 0;
    if message_filtered((*sgp).sg_name) {
        return;
    }
    if (*sgp).sg_parent != 0 && (*sgp).sg_cleared as ::core::ffi::c_int != 0 {
        return;
    }
    didh = highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        (*sgp).sg_cterm,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"cterm\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_cterm_fg,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_cterm_bg,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        (*sgp).sg_gui,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"gui\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut hexbuf: [::core::ffi::c_char; 8] = [0; 8];
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_fg_idx,
            (*sgp).sg_rgb_fg as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guifg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_bg_idx,
            (*sgp).sg_rgb_bg as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guibg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_sp_idx,
            (*sgp).sg_rgb_sp as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guisp\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_blend + 1 as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"blend\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*sgp).sg_link != 0 && !got_int {
        syn_list_header(didh, 0 as ::core::ffi::c_int, id, true_0 != 0);
        didh = true_0 != 0;
        msg_puts_hl(
            b"links to\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_D as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        msg_outtrans(
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_link
                    - 1 as ::core::ffi::c_int) as isize,
            ))
            .sg_name,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    if !didh {
        highlight_list_arg(
            id,
            didh,
            LIST_STRING,
            0 as ::core::ffi::c_int,
            b"cleared\0".as_ptr() as *const ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if p_verbose > 0 as OptInt {
        last_set_msg((*sgp).sg_script_ctx);
    }
}
unsafe extern "C" fn hlgroup2dict(
    mut hl: *mut Dict,
    mut ns_id: NS,
    mut hl_id: ::core::ffi::c_int,
    mut arena: *mut Arena,
) -> bool {
    let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
    let mut ns: NS = ns_id;
    let mut link: ::core::ffi::c_int = if ns_id == 0 as ::core::ffi::c_int {
        (*sgp).sg_link
    } else {
        ns_get_hl(&raw mut ns, hl_id, true_0 != 0, (*sgp).sg_set != 0)
    };
    if link == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if ns_id == 0 as ::core::ffi::c_int
        && (*sgp).sg_cleared as ::core::ffi::c_int != 0
        && (*sgp).sg_set == 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    ns = ns_id;
    let mut attr: HlAttrs = syn_attr2entry(if ns_id == 0 as ::core::ffi::c_int {
        (*sgp).sg_attr
    } else {
        ns_get_hl(&raw mut ns, hl_id, false_0 != 0, (*sgp).sg_set != 0)
    });
    *hl = arena_dict(
        arena,
        (HLATTRS_DICT_SIZE as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
    );
    if attr.rgb_ae_attr & HL_DEFAULT as ::core::ffi::c_int as int32_t != 0 {
        let c2rust_fresh1 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(b"default\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_0 { boolean: true },
            },
        };
    }
    if link > 0 as ::core::ffi::c_int {
        '_c2rust_label: {
            if 1 as ::core::ffi::c_int <= link && link <= (*highlight_ga.ptr()).ga_len {
            } else {
                __assert_fail(
                    b"1 <= link && link <= highlight_ga.ga_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/highlight_group.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1661 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        let c2rust_fresh2 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh2 as isize) = key_value_pair {
            key: cstr_as_string(b"link\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_0 {
                    string: cstr_as_string(
                        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                            .offset((link - 1 as ::core::ffi::c_int) as isize))
                        .sg_name,
                    ),
                },
            },
        };
    }
    let mut hl_cterm: Dict = arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
    hlattrs2dict(
        hl,
        ::core::ptr::null_mut::<Dict>(),
        attr,
        true_0 != 0,
        true_0 != 0,
    );
    hlattrs2dict(hl, &raw mut hl_cterm, attr, false_0 != 0, true_0 != 0);
    if hl_cterm.size != 0 {
        let c2rust_fresh3 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"cterm\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed_0 { dict: hl_cterm },
            },
        };
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ns_get_hl_defs(
    mut ns_id: NS,
    mut opts: *mut KeyDict_get_highlight,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut link: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__link
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).link as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__name
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut create: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__create
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).create as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        id = if create as ::core::ffi::c_int != 0 {
            syn_check_group((*opts).name.data, (*opts).name.size)
        } else {
            syn_name2id_len((*opts).name.data, (*opts).name.size)
        };
        if id == 0 as ::core::ffi::c_int && !create {
            let mut attrs: Dict = ARRAY_DICT_INIT;
            return attrs;
        }
    } else if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__id
        != 0 as ::core::ffi::c_ulonglong
    {
        id = (*opts).id as ::core::ffi::c_int;
    }
    if id != -1 as ::core::ffi::c_int {
        if !(1 as ::core::ffi::c_int <= id && id <= (*highlight_ga.ptr()).ga_len) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Highlight id out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            let mut attrs_0: Dict = ARRAY_DICT_INIT;
            hlgroup2dict(
                &raw mut attrs_0,
                ns_id,
                if link as ::core::ffi::c_int != 0 {
                    id
                } else {
                    syn_get_final_id(id)
                },
                arena,
            );
            return attrs_0;
        }
    } else if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        rv = arena_dict(arena, (*highlight_ga.ptr()).ga_len as size_t);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*highlight_ga.ptr()).ga_len {
            let mut attrs_1: Dict = ARRAY_DICT_INIT;
            if hlgroup2dict(&raw mut attrs_1, ns_id, i, arena) {
                let c2rust_fresh0 = rv.size;
                rv.size = rv.size.wrapping_add(1);
                *rv.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                    key: cstr_as_string(
                        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                            ((if link as ::core::ffi::c_int != 0 {
                                i
                            } else {
                                syn_get_final_id(i)
                            }) - 1 as ::core::ffi::c_int) as isize,
                        ))
                        .sg_name,
                    ),
                    value: object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed_0 { dict: attrs_1 },
                    },
                };
            }
            i += 1;
        }
        return rv;
    }
    return ARRAY_DICT_INIT;
}
unsafe extern "C" fn highlight_list_arg(
    id: ::core::ffi::c_int,
    mut didh: bool,
    type_0: ::core::ffi::c_int,
    mut iarg: ::core::ffi::c_int,
    mut sarg: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
) -> bool {
    if got_int {
        return false_0 != 0;
    }
    if if type_0 == LIST_STRING {
        sarg.is_null() as ::core::ffi::c_int
    } else {
        (iarg == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } != 0
    {
        return didh;
    }
    let mut buf: [::core::ffi::c_char; 100] = [0; 100];
    let mut ts: *const ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    if type_0 == LIST_INT {
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            iarg - 1 as ::core::ffi::c_int,
        );
    } else if type_0 == LIST_STRING {
        ts = sarg;
    } else {
        buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (*hl_attr_table.ptr())[i as usize] != 0 as ::core::ffi::c_int {
            if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0
                && iarg & HL_UNDERLINE_MASK as ::core::ffi::c_int
                    == (*hl_attr_table.ptr())[i as usize]
                || (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK as ::core::ffi::c_int == 0
                    && iarg & (*hl_attr_table.ptr())[i as usize] != 0
            {
                if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                    xstrlcat(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        b",\0".as_ptr() as *const ::core::ffi::c_char,
                        100 as size_t,
                    );
                }
                xstrlcat(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    (*hl_name_table.ptr())[i as usize] as *const ::core::ffi::c_char,
                    100 as size_t,
                );
                if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK as ::core::ffi::c_int == 0
                {
                    iarg &= !(*hl_attr_table.ptr())[i as usize];
                }
            }
            i += 1;
        }
    }
    syn_list_header(
        didh,
        vim_strsize(ts) + strlen(name) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        id,
        false_0 != 0,
    );
    didh = true_0 != 0;
    if !got_int {
        if *name as ::core::ffi::c_int != NUL {
            msg_puts_hl(name, HLF_D as ::core::ffi::c_int, false_0 != 0);
            msg_puts_hl(
                b"=\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_D as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_outtrans(ts, 0 as ::core::ffi::c_int, false_0 != 0);
    }
    return didh;
}
#[no_mangle]
pub unsafe extern "C" fn highlight_has_attr(
    id: ::core::ffi::c_int,
    flag: ::core::ffi::c_int,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut attr: ::core::ffi::c_int = 0;
    if modec == 'g' as ::core::ffi::c_int {
        attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize))
        .sg_gui;
    } else {
        attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize))
        .sg_cterm;
    }
    if flag & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
        let mut ul: ::core::ffi::c_int = attr & HL_UNDERLINE_MASK as ::core::ffi::c_int;
        return if ul == flag {
            b"1\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    } else {
        return if attr & flag != 0 {
            b"1\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    };
}
#[no_mangle]
pub unsafe extern "C" fn highlight_color(
    id: ::core::ffi::c_int,
    what: *const ::core::ffi::c_char,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static name: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([0; 20]);
    let mut fg: bool = false_0 != 0;
    let mut sp: bool = false_0 != 0;
    let mut font: bool = false_0 != 0;
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'f' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'g' as ::core::ffi::c_int
    {
        fg = true_0 != 0;
    } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'f' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'o' as ::core::ffi::c_int
        && (if (*what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'n' as ::core::ffi::c_int
        && (if (*what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 't' as ::core::ffi::c_int
    {
        font = true_0 != 0;
    } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 's' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'p' as ::core::ffi::c_int
    {
        sp = true_0 != 0;
    } else if !((if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'b' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'g' as ::core::ffi::c_int)
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut n: ::core::ffi::c_int = 0;
    if modec == 'g' as ::core::ffi::c_int {
        if *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
            && ui_rgb_attached() as ::core::ffi::c_int != 0
        {
            if fg {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg as ::core::ffi::c_int;
            } else if sp {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp as ::core::ffi::c_int;
            } else {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg as ::core::ffi::c_int;
            }
            if n < 0 as ::core::ffi::c_int || n > 0xffffff as ::core::ffi::c_int {
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                n,
            );
            return name.ptr() as *mut ::core::ffi::c_char;
        }
        if fg {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        } else if sp {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        } else {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        }
    }
    if font as ::core::ffi::c_int != 0 || sp as ::core::ffi::c_int != 0 {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if modec == 'c' as ::core::ffi::c_int {
        if fg {
            n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_cterm_fg
                - 1 as ::core::ffi::c_int;
        } else {
            n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_cterm_bg
                - 1 as ::core::ffi::c_int;
        }
        if n < 0 as ::core::ffi::c_int {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        snprintf(
            name.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        return name.ptr() as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn syn_list_header(
    did_header: bool,
    outlen: ::core::ffi::c_int,
    id: ::core::ffi::c_int,
    mut force_newline: bool,
) -> bool {
    let mut endcol: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
    let mut newline: bool = true_0 != 0;
    let mut name_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut adjust: bool = true_0 != 0;
    if !did_header {
        if !ui_has(kUIMessages) || msg_col > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if got_int {
            return true_0 != 0;
        }
        name_col = msg_outtrans(
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_name,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_col = name_col;
        endcol = 15 as ::core::ffi::c_int;
    } else if (ui_has(kUIMessages) as ::core::ffi::c_int != 0 || msg_silent != 0) && !force_newline
    {
        msg_putchar(' ' as ::core::ffi::c_int);
        adjust = false_0 != 0;
    } else if msg_col + outlen + 1 as ::core::ffi::c_int >= Columns
        || force_newline as ::core::ffi::c_int != 0
    {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int {
            return true_0 != 0;
        }
    } else if msg_col >= endcol {
        newline = false_0 != 0;
    }
    if adjust {
        if msg_col >= endcol {
            endcol = msg_col + 1 as ::core::ffi::c_int;
        }
        msg_advance(endcol);
    }
    if !did_header {
        if endcol == Columns - 1 as ::core::ffi::c_int && endcol <= name_col {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        msg_puts_hl(
            b"xxx\0".as_ptr() as *const ::core::ffi::c_char,
            id,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    return newline;
}
unsafe extern "C" fn set_hl_attr(mut idx: ::core::ffi::c_int) {
    let mut at_en: HlAttrs = HLATTRS_INIT;
    let mut sgp: *mut HlGroup =
        ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    at_en.cterm_ae_attr = (*sgp).sg_cterm as int32_t;
    at_en.cterm_fg_color = (*sgp).sg_cterm_fg as int16_t;
    at_en.cterm_bg_color = (*sgp).sg_cterm_bg as int16_t;
    at_en.rgb_ae_attr = (*sgp).sg_gui as int32_t;
    at_en.rgb_fg_color = if (*sgp).sg_rgb_fg_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_fg
    } else {
        -1 as RgbValue
    };
    at_en.rgb_bg_color = if (*sgp).sg_rgb_bg_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_bg
    } else {
        -1 as RgbValue
    };
    at_en.rgb_sp_color = if (*sgp).sg_rgb_sp_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_sp
    } else {
        -1 as RgbValue
    };
    at_en.hl_blend = (*sgp).sg_blend as int32_t;
    (*sgp).sg_attr = hl_get_syn_attr(
        0 as ::core::ffi::c_int,
        idx + 1 as ::core::ffi::c_int,
        at_en,
    );
    if cursor_mode_uses_syn_id(idx + 1 as ::core::ffi::c_int) {
        ui_mode_info_set();
    }
}
#[no_mangle]
pub unsafe extern "C" fn syn_name2id(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '@' as ::core::ffi::c_int
    {
        return syn_check_group(name, strlen(name));
    }
    return syn_name2id_len(name, strlen(name));
}
#[no_mangle]
pub unsafe extern "C" fn syn_name2id_len(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut name_u: [::core::ffi::c_char; 201] = [0; 201];
    if len == 0 as size_t || len > MAX_SYN_NAME as size_t {
        return 0 as ::core::ffi::c_int;
    }
    vim_memcpy_up(&raw mut name_u as *mut ::core::ffi::c_char, name, len);
    name_u[len as usize] = NUL as ::core::ffi::c_char;
    return map_get_cstr_t_int(
        highlight_unames.ptr(),
        &raw mut name_u as *mut ::core::ffi::c_char as cstr_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn syn_name2attr(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut id: ::core::ffi::c_int = syn_name2id(name);
    if id != 0 as ::core::ffi::c_int {
        return syn_id2attr(id);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn highlight_exists(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return (syn_name2id(name) > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn syn_id2name(mut id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((id - 1 as ::core::ffi::c_int) as isize))
    .sg_name;
}
#[no_mangle]
pub unsafe extern "C" fn syn_check_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if len > MAX_SYN_NAME as size_t {
        emsg(gettext(
            &raw const e_highlight_group_name_too_long as *const ::core::ffi::c_char,
        ));
        return 0 as ::core::ffi::c_int;
    }
    let mut id: ::core::ffi::c_int = syn_name2id_len(name, len);
    if id == 0 as ::core::ffi::c_int {
        return syn_add_group(name, len);
    }
    return id;
}
unsafe extern "C" fn syn_add_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut i: size_t = 0 as size_t;
    while i < len {
        let mut c: ::core::ffi::c_int = *name.offset(i as isize) as uint8_t as ::core::ffi::c_int;
        if !vim_isprintc(c) {
            emsg(gettext(
                b"E669: Unprintable character in group name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        } else if !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(c) as ::core::ffi::c_int != 0)
            && c != '_' as ::core::ffi::c_int
            && c != '.' as ::core::ffi::c_int
            && c != '@' as ::core::ffi::c_int
            && c != '-' as ::core::ffi::c_int
        {
            msg_source(HLF_W as ::core::ffi::c_int);
            emsg(gettext(
                &raw const e_highlight_group_name_invalid_char as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    let mut scoped_parent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if len > 1 as size_t
        && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
    {
        let mut delim: *mut ::core::ffi::c_char =
            xmemrchr(name as *const ::core::ffi::c_void, '.' as uint8_t, len)
                as *mut ::core::ffi::c_char;
        if !delim.is_null() {
            scoped_parent = syn_check_group(name, delim.offset_from(name) as size_t);
        }
    }
    if (*highlight_ga.ptr()).ga_data.is_null() {
        (*highlight_ga.ptr()).ga_itemsize = ::core::mem::size_of::<HlGroup>() as ::core::ffi::c_int;
        ga_set_growsize(highlight_ga.ptr(), 10 as ::core::ffi::c_int);
        ga_grow(highlight_ga.ptr(), 300 as ::core::ffi::c_int);
    }
    if (*highlight_ga.ptr()).ga_len >= MAX_HL_ID as ::core::ffi::c_int {
        emsg(gettext(
            b"E849: Too many highlight and syntax groups\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return 0 as ::core::ffi::c_int;
    }
    let mut hlgp: *mut HlGroup =
        ga_append_via_ptr(highlight_ga.ptr(), ::core::mem::size_of::<HlGroup>()) as *mut HlGroup;
    memset(
        hlgp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HlGroup>(),
    );
    (*hlgp).sg_name = arena_memdupz(highlight_arena.ptr(), name, len);
    (*hlgp).sg_rgb_bg = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_fg = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_sp = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_bg_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_rgb_fg_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_rgb_sp_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_blend = -1 as ::core::ffi::c_int;
    (*hlgp).sg_name_u = arena_memdupz(highlight_arena.ptr(), name, len);
    (*hlgp).sg_parent = scoped_parent;
    (*hlgp).sg_cleared = true_0 != 0;
    vim_strup((*hlgp).sg_name_u);
    let mut id: ::core::ffi::c_int = (*highlight_ga.ptr()).ga_len;
    map_put_cstr_t_int(highlight_unames.ptr(), (*hlgp).sg_name_u as cstr_t, id);
    return id;
}
#[no_mangle]
pub unsafe extern "C" fn syn_id2attr(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut optional: bool = false_0 != 0;
    return syn_ns_id2attr(-1 as ::core::ffi::c_int, hl_id, &raw mut optional);
}
#[no_mangle]
pub unsafe extern "C" fn syn_ns_id2attr(
    mut ns_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut optional: *mut bool,
) -> ::core::ffi::c_int {
    if syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id) {
        *optional = false_0 != 0;
    }
    let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
    let mut attr: ::core::ffi::c_int =
        ns_get_hl(&raw mut ns_id, hl_id, false_0 != 0, (*sgp).sg_set != 0);
    if attr >= 0 as ::core::ffi::c_int
        || *optional as ::core::ffi::c_int != 0 && ns_id > 0 as ::core::ffi::c_int
    {
        return attr;
    }
    return (*sgp).sg_attr;
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_final_id(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ns_id: ::core::ffi::c_int = (*curwin).w_ns_hl_active;
    syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id);
    return hl_id;
}
#[no_mangle]
pub unsafe extern "C" fn syn_ns_get_final_id(
    mut ns_id: *mut ::core::ffi::c_int,
    mut hl_idp: *mut ::core::ffi::c_int,
) -> bool {
    let mut hl_id: ::core::ffi::c_int = *hl_idp;
    let mut used: bool = false_0 != 0;
    if hl_id > (*highlight_ga.ptr()).ga_len || hl_id < 1 as ::core::ffi::c_int {
        *hl_idp = 0 as ::core::ffi::c_int;
        return false_0 != 0;
    }
    let mut count: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
        let mut check: ::core::ffi::c_int =
            ns_get_hl(ns_id as *mut NS, hl_id, true_0 != 0, (*sgp).sg_set != 0);
        if check == 0 as ::core::ffi::c_int {
            *hl_idp = hl_id;
            return true_0 != 0;
        } else if check > 0 as ::core::ffi::c_int {
            used = true_0 != 0;
            hl_id = check;
        } else if (*sgp).sg_link > 0 as ::core::ffi::c_int
            && (*sgp).sg_link <= (*highlight_ga.ptr()).ga_len
        {
            hl_id = (*sgp).sg_link;
        } else {
            if !((*sgp).sg_cleared as ::core::ffi::c_int != 0
                && (*sgp).sg_parent > 0 as ::core::ffi::c_int)
            {
                break;
            }
            hl_id = (*sgp).sg_parent;
        }
    }
    *hl_idp = hl_id;
    return used;
}
#[no_mangle]
pub unsafe extern "C" fn highlight_attr_set_all() {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*highlight_ga.ptr()).ga_len {
        let mut sgp: *mut HlGroup =
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
        if (*sgp).sg_rgb_bg_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_bg = normal_fg;
        } else if (*sgp).sg_rgb_bg_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_bg = normal_bg;
        }
        if (*sgp).sg_rgb_fg_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_fg = normal_fg;
        } else if (*sgp).sg_rgb_fg_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_fg = normal_bg;
        }
        if (*sgp).sg_rgb_sp_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_sp = normal_fg;
        } else if (*sgp).sg_rgb_sp_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_sp = normal_bg;
        }
        set_hl_attr(idx);
        idx += 1;
    }
}
unsafe extern "C" fn combine_stl_hlt(
    mut id: ::core::ffi::c_int,
    mut id_S: ::core::ffi::c_int,
    mut id_alt: ::core::ffi::c_int,
    mut hlcnt: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
    mut hlf: ::core::ffi::c_int,
    mut table: *mut ::core::ffi::c_int,
) {
    let hlt: *mut HlGroup = (*highlight_ga.ptr()).ga_data as *mut HlGroup;
    if id_alt == 0 as ::core::ffi::c_int {
        memset(
            hlt.offset((hlcnt + i) as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HlGroup>(),
        );
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm = highlight_attr[hlf as usize];
        (*hlt.offset((hlcnt + i) as isize)).sg_gui = highlight_attr[hlf as usize];
    } else {
        memmove(
            hlt.offset((hlcnt + i) as isize) as *mut ::core::ffi::c_void,
            hlt.offset((id_alt - 1 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<HlGroup>(),
        );
    }
    (*hlt.offset((hlcnt + i) as isize)).sg_link = 0 as ::core::ffi::c_int;
    (*hlt.offset((hlcnt + i) as isize)).sg_cterm ^=
        (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm
            ^ (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm;
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm_fg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm_bg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg;
    }
    (*hlt.offset((hlcnt + i) as isize)).sg_gui ^=
        (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_gui
            ^ (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_gui;
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_fg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_bg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_sp =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp;
    }
    (*highlight_ga.ptr()).ga_len = hlcnt + i + 1 as ::core::ffi::c_int;
    set_hl_attr(hlcnt + i);
    *table.offset(i as isize) = syn_id2attr(hlcnt + i + 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn highlight_changed() {
    let mut userhl: [::core::ffi::c_char; 30] = [0; 30];
    let mut id_S: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut id_SNC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    need_highlight_changed = false_0 != 0;
    highlight_attr[HLF_NONE as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    let mut hlf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while hlf < HLF_COUNT as ::core::ffi::c_int {
        let mut id: ::core::ffi::c_int = syn_check_group(
            *(&raw mut hlf_names as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            strlen(*(&raw mut hlf_names as *mut *const ::core::ffi::c_char).offset(hlf as isize)),
        );
        if id == 0 as ::core::ffi::c_int {
            abort();
        }
        let mut ns_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut final_id: ::core::ffi::c_int = id;
        syn_ns_get_final_id(&raw mut ns_id, &raw mut final_id);
        if hlf == HLF_SNC as ::core::ffi::c_int {
            id_SNC = final_id;
        } else if hlf == HLF_S as ::core::ffi::c_int {
            id_S = final_id;
        }
        highlight_attr[hlf as usize] = hl_get_ui_attr(
            ns_id,
            hlf,
            final_id,
            hlf == HLF_INACTIVE as ::core::ffi::c_int,
        );
        if highlight_attr[hlf as usize] != highlight_attr_last[hlf as usize] {
            if hlf == HLF_MSG as ::core::ffi::c_int {
                clear_cmdline = true_0 != 0;
                let mut attrs: HlAttrs = syn_attr2entry(highlight_attr[hlf as usize]);
                msg_grid.blending = attrs.hl_blend > -1 as int32_t;
            }
            ui_call_hl_group_set(
                cstr_as_string(
                    *(&raw mut hlf_names as *mut *const ::core::ffi::c_char).offset(hlf as isize),
                ),
                highlight_attr[hlf as usize] as Integer,
            );
            highlight_attr_last[hlf as usize] = highlight_attr[hlf as usize];
        }
        hlf += 1;
    }
    ga_grow(highlight_ga.ptr(), 10 as ::core::ffi::c_int);
    let mut hlcnt: ::core::ffi::c_int = (*highlight_ga.ptr()).ga_len;
    if id_S == -1 as ::core::ffi::c_int {
        memset(
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((hlcnt + 9 as ::core::ffi::c_int) as isize)
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HlGroup>(),
        );
        id_S = hlcnt + 10 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 9 as ::core::ffi::c_int {
        snprintf(
            &raw mut userhl as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"User%d\0".as_ptr() as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
        );
        let mut id_0: ::core::ffi::c_int = syn_name2id(&raw mut userhl as *mut ::core::ffi::c_char);
        if id_0 == 0 as ::core::ffi::c_int {
            highlight_user[i as usize] = 0 as ::core::ffi::c_int;
            highlight_stlnc[i as usize] = 0 as ::core::ffi::c_int;
        } else {
            highlight_user[i as usize] = syn_id2attr(id_0);
            combine_stl_hlt(
                id_0,
                id_S,
                id_SNC,
                hlcnt,
                i,
                HLF_SNC as ::core::ffi::c_int,
                &raw mut highlight_stlnc as *mut ::core::ffi::c_int,
            );
        }
        i += 1;
    }
    (*highlight_ga.ptr()).ga_len = hlcnt;
    decor_provider_invalidate_hl();
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_highlight_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_link = 2 as ::core::ffi::c_int;
    include_default = 1 as ::core::ffi::c_int;
    if *arg as ::core::ffi::c_int == NUL {
        return;
    }
    let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    include_default = 0 as ::core::ffi::c_int;
    if strncmp(
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        arg,
        p.offset_from(arg) as ::core::ffi::c_uint as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = skipwhite(p);
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        p = skiptowhite(arg);
    }
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    include_link = 0 as ::core::ffi::c_int;
    if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'i' as ::core::ffi::c_int
        && *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'N' as ::core::ffi::c_int
    {
        highlight_list();
    }
    if strncmp(
        b"link\0".as_ptr() as *const ::core::ffi::c_char,
        arg,
        p.offset_from(arg) as ::core::ffi::c_uint as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            b"clear\0".as_ptr() as *const ::core::ffi::c_char,
            arg,
            p.offset_from(arg) as ::core::ffi::c_uint as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_pattern = skipwhite(p);
        p = skiptowhite((*xp).xp_pattern);
        if *p as ::core::ffi::c_int != NUL {
            (*xp).xp_pattern = skipwhite(p);
            p = skiptowhite((*xp).xp_pattern);
        }
    }
    if *p as ::core::ffi::c_int != NUL {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn highlight_list() {
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        highlight_list_two(i, HLF_D as ::core::ffi::c_int);
    }
    let mut i_0: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        highlight_list_two(99 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn highlight_list_two(mut cnt: ::core::ffi::c_int, mut id: ::core::ffi::c_int) {
    msg_puts_hl(
        (b"N \x08I \x08!  \x08\0".as_ptr() as *const ::core::ffi::c_char)
            .offset((cnt / 11 as ::core::ffi::c_int) as isize),
        id,
        false_0 != 0,
    );
    msg_clr_eos();
    ui_flush();
    os_delay(
        if cnt == 99 as ::core::ffi::c_int {
            40 as uint64_t
        } else {
            (cnt as uint64_t).wrapping_mul(50 as uint64_t)
        },
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_highlight_name(
    xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return get_highlight_name_ext(xp, idx, true_0 != 0) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn get_highlight_name_ext(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut skip_cleared: bool,
) -> *const ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if skip_cleared as ::core::ffi::c_int != 0
        && idx < (*highlight_ga.ptr()).ga_len
        && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
            as ::core::ffi::c_int
            != 0
    {
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if idx == (*highlight_ga.ptr()).ga_len && include_none != 0 as ::core::ffi::c_int {
        return b"none\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx == (*highlight_ga.ptr()).ga_len + include_none
        && include_default != 0 as ::core::ffi::c_int
    {
        return b"default\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx == (*highlight_ga.ptr()).ga_len + include_none + include_default
        && include_link != 0 as ::core::ffi::c_int
    {
        return b"link\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx
        == (*highlight_ga.ptr()).ga_len + include_none + include_default + 1 as ::core::ffi::c_int
        && include_link != 0 as ::core::ffi::c_int
    {
        return b"clear\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx >= (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_name;
}
#[no_mangle]
pub static color_name_table: GlobalCell<[color_name_table_T; 708]> = GlobalCell::new([
    color_name_table_T {
        name: b"AliceBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"AntiqueWhite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xeb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd7 as RgbValue,
    },
    color_name_table_T {
        name: b"AntiqueWhite1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xef as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdb as RgbValue,
    },
    color_name_table_T {
        name: b"AntiqueWhite2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcc as RgbValue,
    },
    color_name_table_T {
        name: b"AntiqueWhite3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb0 as RgbValue,
    },
    color_name_table_T {
        name: b"AntiqueWhite4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x83 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x78 as RgbValue,
    },
    color_name_table_T {
        name: b"Aqua\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Aquamarine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd4 as RgbValue,
    },
    color_name_table_T {
        name: b"Aquamarine1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd4 as RgbValue,
    },
    color_name_table_T {
        name: b"Aquamarine2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x76 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc6 as RgbValue,
    },
    color_name_table_T {
        name: b"Aquamarine3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xaa as RgbValue,
    },
    color_name_table_T {
        name: b"Aquamarine4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x45 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x74 as RgbValue,
    },
    color_name_table_T {
        name: b"Azure\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Azure1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Azure2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Azure3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Azure4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x83 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Beige\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdc as RgbValue,
    },
    color_name_table_T {
        name: b"Bisque\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc4 as RgbValue,
    },
    color_name_table_T {
        name: b"Bisque1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc4 as RgbValue,
    },
    color_name_table_T {
        name: b"Bisque2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb7 as RgbValue,
    },
    color_name_table_T {
        name: b"Bisque3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9e as RgbValue,
    },
    color_name_table_T {
        name: b"Bisque4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6b as RgbValue,
    },
    color_name_table_T {
        name: b"Black\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"BlanchedAlmond\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xeb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Blue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Blue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Blue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Blue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Blue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"BlueViolet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2b as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe2 as RgbValue,
    },
    color_name_table_T {
        name: b"Brown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2a as RgbValue,
    },
    color_name_table_T {
        name: b"Brown1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x40 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x40 as RgbValue,
    },
    color_name_table_T {
        name: b"Brown2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3b as RgbValue,
    },
    color_name_table_T {
        name: b"Brown3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x33 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x33 as RgbValue,
    },
    color_name_table_T {
        name: b"Brown4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x23 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x23 as RgbValue,
    },
    color_name_table_T {
        name: b"BurlyWood\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xde as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x87 as RgbValue,
    },
    color_name_table_T {
        name: b"Burlywood1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9b as RgbValue,
    },
    color_name_table_T {
        name: b"Burlywood2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x91 as RgbValue,
    },
    color_name_table_T {
        name: b"Burlywood3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xaa as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7d as RgbValue,
    },
    color_name_table_T {
        name: b"Burlywood4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x73 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x55 as RgbValue,
    },
    color_name_table_T {
        name: b"CadetBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa0 as RgbValue,
    },
    color_name_table_T {
        name: b"CadetBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x98 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"CadetBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8e as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"CadetBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"CadetBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x53 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"ChartReuse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Chartreuse1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Chartreuse2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x76 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Chartreuse3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Chartreuse4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x45 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Chocolate\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1e as RgbValue,
    },
    color_name_table_T {
        name: b"Chocolate1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x24 as RgbValue,
    },
    color_name_table_T {
        name: b"Chocolate2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x76 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x21 as RgbValue,
    },
    color_name_table_T {
        name: b"Chocolate3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1d as RgbValue,
    },
    color_name_table_T {
        name: b"Chocolate4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x13 as RgbValue,
    },
    color_name_table_T {
        name: b"Coral\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x50 as RgbValue,
    },
    color_name_table_T {
        name: b"Coral1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x72 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x56 as RgbValue,
    },
    color_name_table_T {
        name: b"Coral2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x50 as RgbValue,
    },
    color_name_table_T {
        name: b"Coral3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x45 as RgbValue,
    },
    color_name_table_T {
        name: b"Coral4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2f as RgbValue,
    },
    color_name_table_T {
        name: b"CornFlowerBlue\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x64 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x95 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xed as RgbValue,
    },
    color_name_table_T {
        name: b"Cornsilk\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdc as RgbValue,
    },
    color_name_table_T {
        name: b"Cornsilk1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdc as RgbValue,
    },
    color_name_table_T {
        name: b"Cornsilk2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Cornsilk3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb1 as RgbValue,
    },
    color_name_table_T {
        name: b"Cornsilk4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x88 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x78 as RgbValue,
    },
    color_name_table_T {
        name: b"Crimson\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdc as RgbValue) << 16 as ::core::ffi::c_int
            | (0x14 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3c as RgbValue,
    },
    color_name_table_T {
        name: b"Cyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Cyan1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Cyan2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Cyan3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Cyan4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGoldenrod\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGoldenrod1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGoldenrod2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xad as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGoldenrod3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x95 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGoldenrod4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x65 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa9 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x64 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa9 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkKhaki\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkMagenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOliveGreen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x55 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2f as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOliveGreen1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xca as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOliveGreen2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xbc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x68 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOliveGreen3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xa2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5a as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOliveGreen4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x6e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3d as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrange\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8c as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrange1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7f as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrange2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x76 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrange3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrange4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrchid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x99 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x32 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcc as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrchid1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbf as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrchid2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3a as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrchid3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x32 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"DarkOrchid4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x68 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x22 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSalmon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x96 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSeaGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbc as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8f as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSeaGreen1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc1 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSeaGreen2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSeaGreen3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9b as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSeaGreen4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x48 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4f as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGray1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x97 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGray2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x8d as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGray3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x79 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGray4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x52 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DarkSlateGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4f as RgbValue,
    },
    color_name_table_T {
        name: b"DarkTurquoise\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xce as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd1 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkViolet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x94 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd3 as RgbValue,
    },
    color_name_table_T {
        name: b"DarkYellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbb as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepPink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x14 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x93 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepPink1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x14 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x93 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepPink2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x12 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x89 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepPink3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x10 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x76 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepPink4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa as RgbValue) << 8 as ::core::ffi::c_int
            | 0x50 as RgbValue,
    },
    color_name_table_T {
        name: b"DeepSkyBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DeepSkyBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DeepSkyBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"DeepSkyBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9a as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"DeepSkyBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x68 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"DimGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"DimGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"DodgerBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x90 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DodgerBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x90 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"DodgerBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"DodgerBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x18 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x74 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"DodgerBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x10 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Firebrick\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x22 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x22 as RgbValue,
    },
    color_name_table_T {
        name: b"Firebrick1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x30 as RgbValue,
    },
    color_name_table_T {
        name: b"Firebrick2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2c as RgbValue,
    },
    color_name_table_T {
        name: b"Firebrick3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x26 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x26 as RgbValue,
    },
    color_name_table_T {
        name: b"Firebrick4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1a as RgbValue,
    },
    color_name_table_T {
        name: b"FloralWhite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"ForestGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x22 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x22 as RgbValue,
    },
    color_name_table_T {
        name: b"Fuchsia\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Gainsboro\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdc as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdc as RgbValue,
    },
    color_name_table_T {
        name: b"GhostWhite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Gold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gold1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gold2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gold3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xad as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gold4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x75 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Goldenrod\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xda as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x20 as RgbValue,
    },
    color_name_table_T {
        name: b"Goldenrod1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x25 as RgbValue,
    },
    color_name_table_T {
        name: b"Goldenrod2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x22 as RgbValue,
    },
    color_name_table_T {
        name: b"Goldenrod3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1d as RgbValue,
    },
    color_name_table_T {
        name: b"Goldenrod4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x14 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray10\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1a as RgbValue,
    },
    color_name_table_T {
        name: b"Gray100\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Gray11\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1c as RgbValue,
    },
    color_name_table_T {
        name: b"Gray12\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1f as RgbValue,
    },
    color_name_table_T {
        name: b"Gray13\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x21 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x21 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x21 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray14\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x24 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x24 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x24 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray15\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x26 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x26 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x26 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray16\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x29 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x29 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x29 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray17\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2b as RgbValue,
    },
    color_name_table_T {
        name: b"Gray18\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2e as RgbValue,
    },
    color_name_table_T {
        name: b"Gray19\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x30 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x30 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray20\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x33 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x33 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x33 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray21\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x36 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x36 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x36 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray22\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x38 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x38 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x38 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray23\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3b as RgbValue,
    },
    color_name_table_T {
        name: b"Gray24\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3d as RgbValue,
    },
    color_name_table_T {
        name: b"Gray25\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x40 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x40 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x40 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray26\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x42 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x42 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x42 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray27\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x45 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x45 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray28\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x47 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x47 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray29\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4a as RgbValue,
    },
    color_name_table_T {
        name: b"Gray3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray30\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4d as RgbValue,
    },
    color_name_table_T {
        name: b"Gray31\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4f as RgbValue,
    },
    color_name_table_T {
        name: b"Gray32\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x52 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x52 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray33\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x54 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x54 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x54 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray34\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x57 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x57 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x57 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray35\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x59 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x59 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x59 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray36\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5c as RgbValue,
    },
    color_name_table_T {
        name: b"Gray37\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5e as RgbValue,
    },
    color_name_table_T {
        name: b"Gray38\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x61 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x61 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x61 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray39\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x63 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x63 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa as RgbValue,
    },
    color_name_table_T {
        name: b"Gray40\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x66 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray41\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray42\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6b as RgbValue,
    },
    color_name_table_T {
        name: b"Gray43\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6e as RgbValue,
    },
    color_name_table_T {
        name: b"Gray44\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x70 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray45\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x73 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x73 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x73 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray46\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x75 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x75 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x75 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray47\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x78 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x78 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x78 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray48\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"Gray49\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7d as RgbValue,
    },
    color_name_table_T {
        name: b"Gray5\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd as RgbValue,
    },
    color_name_table_T {
        name: b"Gray50\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7f as RgbValue,
    },
    color_name_table_T {
        name: b"Gray51\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x82 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray52\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x85 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x85 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x85 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray53\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x87 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x87 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x87 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray54\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8a as RgbValue,
    },
    color_name_table_T {
        name: b"Gray55\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8c as RgbValue,
    },
    color_name_table_T {
        name: b"Gray56\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8f as RgbValue,
    },
    color_name_table_T {
        name: b"Gray57\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x91 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x91 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x91 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray58\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x94 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x94 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x94 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray59\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x96 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x96 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x96 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray6\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf as RgbValue,
    },
    color_name_table_T {
        name: b"Gray60\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x99 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x99 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x99 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray61\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9c as RgbValue,
    },
    color_name_table_T {
        name: b"Gray62\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9e as RgbValue,
    },
    color_name_table_T {
        name: b"Gray63\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa1 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray64\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa3 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray65\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa6 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray66\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa8 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray67\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xab as RgbValue) << 16 as ::core::ffi::c_int
            | (0xab as RgbValue) << 8 as ::core::ffi::c_int
            | 0xab as RgbValue,
    },
    color_name_table_T {
        name: b"Gray68\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xad as RgbValue) << 16 as ::core::ffi::c_int
            | (0xad as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"Gray69\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray7\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x12 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x12 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x12 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray70\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb3 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray71\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb5 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray72\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb8 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray73\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xba as RgbValue) << 16 as ::core::ffi::c_int
            | (0xba as RgbValue) << 8 as ::core::ffi::c_int
            | 0xba as RgbValue,
    },
    color_name_table_T {
        name: b"Gray74\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbd as RgbValue,
    },
    color_name_table_T {
        name: b"Gray75\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbf as RgbValue,
    },
    color_name_table_T {
        name: b"Gray76\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc2 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray77\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc4 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray78\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc7 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray79\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc9 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x14 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x14 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x14 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray80\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcc as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcc as RgbValue,
    },
    color_name_table_T {
        name: b"Gray81\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcf as RgbValue,
    },
    color_name_table_T {
        name: b"Gray82\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd1 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray83\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd4 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray84\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd6 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray85\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd9 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray86\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdb as RgbValue,
    },
    color_name_table_T {
        name: b"Gray87\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xde as RgbValue) << 16 as ::core::ffi::c_int
            | (0xde as RgbValue) << 8 as ::core::ffi::c_int
            | 0xde as RgbValue,
    },
    color_name_table_T {
        name: b"Gray88\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray89\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe3 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray9\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x17 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x17 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x17 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray90\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe5 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray91\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe8 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray92\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xeb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xeb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xeb as RgbValue,
    },
    color_name_table_T {
        name: b"Gray93\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xed as RgbValue) << 16 as ::core::ffi::c_int
            | (0xed as RgbValue) << 8 as ::core::ffi::c_int
            | 0xed as RgbValue,
    },
    color_name_table_T {
        name: b"Gray94\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray95\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf2 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray96\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf5 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray97\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf7 as RgbValue,
    },
    color_name_table_T {
        name: b"Gray98\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"Gray99\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfc as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfc as RgbValue,
    },
    color_name_table_T {
        name: b"Green\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Green1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Green2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Green3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Green4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"GreenYellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xad as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2f as RgbValue,
    },
    color_name_table_T {
        name: b"Grey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey10\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1a as RgbValue,
    },
    color_name_table_T {
        name: b"Grey100\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Grey11\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1c as RgbValue,
    },
    color_name_table_T {
        name: b"Grey12\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x1f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1f as RgbValue,
    },
    color_name_table_T {
        name: b"Grey13\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x21 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x21 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x21 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey14\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x24 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x24 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x24 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey15\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x26 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x26 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x26 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey16\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x29 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x29 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x29 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey17\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2b as RgbValue,
    },
    color_name_table_T {
        name: b"Grey18\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2e as RgbValue,
    },
    color_name_table_T {
        name: b"Grey19\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x30 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x30 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey20\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x33 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x33 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x33 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey21\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x36 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x36 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x36 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey22\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x38 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x38 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x38 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey23\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3b as RgbValue,
    },
    color_name_table_T {
        name: b"Grey24\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3d as RgbValue,
    },
    color_name_table_T {
        name: b"Grey25\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x40 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x40 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x40 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey26\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x42 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x42 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x42 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey27\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x45 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x45 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey28\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x47 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x47 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey29\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4a as RgbValue,
    },
    color_name_table_T {
        name: b"Grey3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey30\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4d as RgbValue,
    },
    color_name_table_T {
        name: b"Grey31\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4f as RgbValue,
    },
    color_name_table_T {
        name: b"Grey32\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x52 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x52 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey33\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x54 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x54 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x54 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey34\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x57 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x57 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x57 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey35\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x59 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x59 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x59 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey36\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5c as RgbValue,
    },
    color_name_table_T {
        name: b"Grey37\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5e as RgbValue,
    },
    color_name_table_T {
        name: b"Grey38\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x61 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x61 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x61 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey39\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x63 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x63 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa as RgbValue,
    },
    color_name_table_T {
        name: b"Grey40\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x66 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey41\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey42\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6b as RgbValue,
    },
    color_name_table_T {
        name: b"Grey43\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6e as RgbValue,
    },
    color_name_table_T {
        name: b"Grey44\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x70 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey45\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x73 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x73 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x73 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey46\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x75 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x75 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x75 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey47\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x78 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x78 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x78 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey48\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"Grey49\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7d as RgbValue,
    },
    color_name_table_T {
        name: b"Grey5\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd as RgbValue,
    },
    color_name_table_T {
        name: b"Grey50\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7f as RgbValue,
    },
    color_name_table_T {
        name: b"Grey51\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x82 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey52\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x85 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x85 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x85 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey53\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x87 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x87 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x87 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey54\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8a as RgbValue,
    },
    color_name_table_T {
        name: b"Grey55\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8c as RgbValue,
    },
    color_name_table_T {
        name: b"Grey56\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8f as RgbValue,
    },
    color_name_table_T {
        name: b"Grey57\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x91 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x91 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x91 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey58\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x94 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x94 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x94 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey59\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x96 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x96 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x96 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey6\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf as RgbValue,
    },
    color_name_table_T {
        name: b"Grey60\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x99 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x99 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x99 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey61\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9c as RgbValue,
    },
    color_name_table_T {
        name: b"Grey62\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9e as RgbValue,
    },
    color_name_table_T {
        name: b"Grey63\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa1 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey64\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa3 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey65\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa6 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey66\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa8 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey67\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xab as RgbValue) << 16 as ::core::ffi::c_int
            | (0xab as RgbValue) << 8 as ::core::ffi::c_int
            | 0xab as RgbValue,
    },
    color_name_table_T {
        name: b"Grey68\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xad as RgbValue) << 16 as ::core::ffi::c_int
            | (0xad as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"Grey69\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb0 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey7\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x12 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x12 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x12 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey70\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb3 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey71\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb5 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey72\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb8 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey73\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xba as RgbValue) << 16 as ::core::ffi::c_int
            | (0xba as RgbValue) << 8 as ::core::ffi::c_int
            | 0xba as RgbValue,
    },
    color_name_table_T {
        name: b"Grey74\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbd as RgbValue,
    },
    color_name_table_T {
        name: b"Grey75\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbf as RgbValue,
    },
    color_name_table_T {
        name: b"Grey76\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc2 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey77\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc4 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey78\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc7 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey79\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc9 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x14 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x14 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x14 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey80\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcc as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcc as RgbValue,
    },
    color_name_table_T {
        name: b"Grey81\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcf as RgbValue,
    },
    color_name_table_T {
        name: b"Grey82\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd1 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey83\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd4 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey84\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd6 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey85\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd9 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey86\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdb as RgbValue,
    },
    color_name_table_T {
        name: b"Grey87\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xde as RgbValue) << 16 as ::core::ffi::c_int
            | (0xde as RgbValue) << 8 as ::core::ffi::c_int
            | 0xde as RgbValue,
    },
    color_name_table_T {
        name: b"Grey88\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey89\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe3 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey9\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x17 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x17 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x17 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey90\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe5 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey91\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe8 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey92\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xeb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xeb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xeb as RgbValue,
    },
    color_name_table_T {
        name: b"Grey93\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xed as RgbValue) << 16 as ::core::ffi::c_int
            | (0xed as RgbValue) << 8 as ::core::ffi::c_int
            | 0xed as RgbValue,
    },
    color_name_table_T {
        name: b"Grey94\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey95\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf2 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey96\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf5 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey97\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf7 as RgbValue,
    },
    color_name_table_T {
        name: b"Grey98\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"Grey99\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfc as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfc as RgbValue,
    },
    color_name_table_T {
        name: b"Honeydew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Honeydew1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Honeydew2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"Honeydew3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc1 as RgbValue,
    },
    color_name_table_T {
        name: b"Honeydew4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x83 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x83 as RgbValue,
    },
    color_name_table_T {
        name: b"HotPink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"HotPink1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"HotPink2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6a as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa7 as RgbValue,
    },
    color_name_table_T {
        name: b"HotPink3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x60 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"HotPink4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x62 as RgbValue,
    },
    color_name_table_T {
        name: b"IndianRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5c as RgbValue,
    },
    color_name_table_T {
        name: b"IndianRed1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6a as RgbValue,
    },
    color_name_table_T {
        name: b"IndianRed2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x63 as RgbValue,
    },
    color_name_table_T {
        name: b"IndianRed3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x55 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x55 as RgbValue,
    },
    color_name_table_T {
        name: b"IndianRed4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3a as RgbValue,
    },
    color_name_table_T {
        name: b"Indigo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4b as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"Ivory\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Ivory1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Ivory2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"Ivory3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc1 as RgbValue,
    },
    color_name_table_T {
        name: b"Ivory4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x83 as RgbValue,
    },
    color_name_table_T {
        name: b"Khaki\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8c as RgbValue,
    },
    color_name_table_T {
        name: b"Khaki1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8f as RgbValue,
    },
    color_name_table_T {
        name: b"Khaki2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x85 as RgbValue,
    },
    color_name_table_T {
        name: b"Khaki3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x73 as RgbValue,
    },
    color_name_table_T {
        name: b"Khaki4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4e as RgbValue,
    },
    color_name_table_T {
        name: b"Lavender\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"LavenderBlush\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf5 as RgbValue,
    },
    color_name_table_T {
        name: b"LavenderBlush1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf5 as RgbValue,
    },
    color_name_table_T {
        name: b"LavenderBlush2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe5 as RgbValue,
    },
    color_name_table_T {
        name: b"LavenderBlush3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc5 as RgbValue,
    },
    color_name_table_T {
        name: b"LavenderBlush4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x83 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x86 as RgbValue,
    },
    color_name_table_T {
        name: b"LawnGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfc as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"LemonChiffon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LemonChiffon1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LemonChiffon2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbf as RgbValue,
    },
    color_name_table_T {
        name: b"LemonChiffon3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa5 as RgbValue,
    },
    color_name_table_T {
        name: b"LemonChiffon4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x89 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"LightBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xad as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe6 as RgbValue,
    },
    color_name_table_T {
        name: b"LightBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xef as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"LightBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9a as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LightBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x68 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x83 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"LightCoral\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"LightCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightCyan1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightCyan2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"LightCyan3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LightCyan4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrod\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrod1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xec as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrod2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdc as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrod3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbe as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrod4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x81 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4c as RgbValue,
    },
    color_name_table_T {
        name: b"LightGoldenrodYellow\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd2 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd3 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x90 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"LightGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd3 as RgbValue,
    },
    color_name_table_T {
        name: b"LightMagenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightPink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc1 as RgbValue,
    },
    color_name_table_T {
        name: b"LightPink1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xae as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb9 as RgbValue,
    },
    color_name_table_T {
        name: b"LightPink2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"LightPink3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x95 as RgbValue,
    },
    color_name_table_T {
        name: b"LightPink4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x65 as RgbValue,
    },
    color_name_table_T {
        name: b"LightRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbb as RgbValue,
    },
    color_name_table_T {
        name: b"LightSalmon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"LightSalmon1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"LightSalmon2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x95 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x72 as RgbValue,
    },
    color_name_table_T {
        name: b"LightSalmon3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x81 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x62 as RgbValue,
    },
    color_name_table_T {
        name: b"LightSalmon4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x57 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x42 as RgbValue,
    },
    color_name_table_T {
        name: b"LightSeaGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x20 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xaa as RgbValue,
    },
    color_name_table_T {
        name: b"LightSkyBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x87 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xce as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"LightSkyBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightSkyBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"LightSkyBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8d as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LightSkyBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x60 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"LightSlateBlue\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x84 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightSlateGray\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x77 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x88 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x99 as RgbValue,
    },
    color_name_table_T {
        name: b"LightSlateGrey\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x77 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x88 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x99 as RgbValue,
    },
    color_name_table_T {
        name: b"LightSteelBlue\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xde as RgbValue,
    },
    color_name_table_T {
        name: b"LightSteelBlue1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xca as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"LightSteelBlue2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xbc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"LightSteelBlue3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xa2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"LightSteelBlue4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x6e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"LightYellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"LightYellow1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe0 as RgbValue,
    },
    color_name_table_T {
        name: b"LightYellow2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd1 as RgbValue,
    },
    color_name_table_T {
        name: b"LightYellow3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"LightYellow4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7a as RgbValue,
    },
    color_name_table_T {
        name: b"Lime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"LimeGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x32 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x32 as RgbValue,
    },
    color_name_table_T {
        name: b"Linen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe6 as RgbValue,
    },
    color_name_table_T {
        name: b"Magenta\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Magenta1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Magenta2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Magenta3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Magenta4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Maroon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Maroon1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x34 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb3 as RgbValue,
    },
    color_name_table_T {
        name: b"Maroon2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa7 as RgbValue,
    },
    color_name_table_T {
        name: b"Maroon3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x29 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"Maroon4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x62 as RgbValue,
    },
    color_name_table_T {
        name: b"MediumAquamarine\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xaa as RgbValue,
    },
    color_name_table_T {
        name: b"MediumBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"MediumOrchid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xba as RgbValue) << 16 as ::core::ffi::c_int
            | (0x55 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd3 as RgbValue,
    },
    color_name_table_T {
        name: b"MediumOrchid1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"MediumOrchid2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd1 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5f as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"MediumOrchid3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"MediumOrchid4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x37 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"MediumPurple\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x93 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdb as RgbValue,
    },
    color_name_table_T {
        name: b"MediumPurple1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xab as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"MediumPurple2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x79 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"MediumPurple3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x89 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x68 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"MediumPurple4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"MediumSeaGreen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x3c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x71 as RgbValue,
    },
    color_name_table_T {
        name: b"MediumSlateBlue\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x7b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x68 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"MediumSpringGreen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9a as RgbValue,
    },
    color_name_table_T {
        name: b"MediumTurquoise\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x48 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcc as RgbValue,
    },
    color_name_table_T {
        name: b"MediumVioletRed\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xc7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x15 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x85 as RgbValue,
    },
    color_name_table_T {
        name: b"MidnightBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x19 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x19 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x70 as RgbValue,
    },
    color_name_table_T {
        name: b"MintCream\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"MistyRose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe1 as RgbValue,
    },
    color_name_table_T {
        name: b"MistyRose1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe1 as RgbValue,
    },
    color_name_table_T {
        name: b"MistyRose2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd2 as RgbValue,
    },
    color_name_table_T {
        name: b"MistyRose3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb5 as RgbValue,
    },
    color_name_table_T {
        name: b"MistyRose4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7d as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7b as RgbValue,
    },
    color_name_table_T {
        name: b"Moccasin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb5 as RgbValue,
    },
    color_name_table_T {
        name: b"NavajoWhite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xde as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"NavajoWhite1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xde as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"NavajoWhite2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa1 as RgbValue,
    },
    color_name_table_T {
        name: b"NavajoWhite3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"NavajoWhite4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x79 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5e as RgbValue,
    },
    color_name_table_T {
        name: b"Navy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"NavyBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x73 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x73 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x73 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGray1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGray2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x14 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x16 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1b as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGray3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x33 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGray4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x58 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x55 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x23 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGrey1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGrey2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x14 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x16 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x1b as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGrey3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x33 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkGrey4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x58 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkMagenta\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x47 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x45 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x59 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimDarkYellow\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x6b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x53 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xdb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightCyan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf7 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGray1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf8 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGray2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xea as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGray3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xc4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGray4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x9b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa4 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGreen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xb3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc0 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGrey1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf8 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGrey2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xe0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xea as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGrey3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xc4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightGrey4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x9b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xa4 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightMagenta\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xca as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb9 as RgbValue,
    },
    color_name_table_T {
        name: b"NvimLightYellow\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xfc as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x94 as RgbValue,
    },
    color_name_table_T {
        name: b"OldLace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe6 as RgbValue,
    },
    color_name_table_T {
        name: b"Olive\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OliveDrab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x23 as RgbValue,
    },
    color_name_table_T {
        name: b"OliveDrab1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3e as RgbValue,
    },
    color_name_table_T {
        name: b"OliveDrab2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb3 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3a as RgbValue,
    },
    color_name_table_T {
        name: b"OliveDrab3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9a as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x32 as RgbValue,
    },
    color_name_table_T {
        name: b"OliveDrab4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x22 as RgbValue,
    },
    color_name_table_T {
        name: b"Orange\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Orange1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Orange2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9a as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Orange3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x85 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Orange4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5a as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OrangeRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OrangeRed1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OrangeRed2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x40 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OrangeRed3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x37 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"OrangeRed4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x25 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Orchid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xda as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd6 as RgbValue,
    },
    color_name_table_T {
        name: b"Orchid1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x83 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"Orchid2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7a as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe9 as RgbValue,
    },
    color_name_table_T {
        name: b"Orchid3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc9 as RgbValue,
    },
    color_name_table_T {
        name: b"Orchid4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x89 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGoldenrod\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xaa as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x98 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfb as RgbValue) << 8 as ::core::ffi::c_int
            | 0x98 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGreen1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9a as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9a as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGreen2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x90 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGreen3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7c as RgbValue,
    },
    color_name_table_T {
        name: b"PaleGreen4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x54 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x54 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleTurquoise\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xaf as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"PaleTurquoise1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xbb as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"PaleTurquoise2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xae as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"PaleTurquoise3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x96 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"PaleTurquoise4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"PaleVioletRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdb as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x93 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleVioletRed1\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xab as RgbValue,
    },
    color_name_table_T {
        name: b"PaleVioletRed2\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x79 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9f as RgbValue,
    },
    color_name_table_T {
        name: b"PaleVioletRed3\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x68 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x89 as RgbValue,
    },
    color_name_table_T {
        name: b"PaleVioletRed4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x5d as RgbValue,
    },
    color_name_table_T {
        name: b"PapayaWhip\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xef as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd5 as RgbValue,
    },
    color_name_table_T {
        name: b"PeachPuff\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xda as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb9 as RgbValue,
    },
    color_name_table_T {
        name: b"PeachPuff1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xda as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb9 as RgbValue,
    },
    color_name_table_T {
        name: b"PeachPuff2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xad as RgbValue,
    },
    color_name_table_T {
        name: b"PeachPuff3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xaf as RgbValue) << 8 as ::core::ffi::c_int
            | 0x95 as RgbValue,
    },
    color_name_table_T {
        name: b"PeachPuff4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x77 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x65 as RgbValue,
    },
    color_name_table_T {
        name: b"Peru\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x85 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3f as RgbValue,
    },
    color_name_table_T {
        name: b"Pink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcb as RgbValue,
    },
    color_name_table_T {
        name: b"Pink1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc5 as RgbValue,
    },
    color_name_table_T {
        name: b"Pink2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb8 as RgbValue,
    },
    color_name_table_T {
        name: b"Pink3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x91 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9e as RgbValue,
    },
    color_name_table_T {
        name: b"Pink4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x6c as RgbValue,
    },
    color_name_table_T {
        name: b"Plum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xdd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xdd as RgbValue,
    },
    color_name_table_T {
        name: b"Plum1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbb as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Plum2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xae as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Plum3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x96 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Plum4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x66 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"PowderBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe6 as RgbValue,
    },
    color_name_table_T {
        name: b"Purple\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"Purple1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Purple2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x91 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x2c as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Purple3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7d as RgbValue) << 16 as ::core::ffi::c_int
            | (0x26 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Purple4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x55 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x1a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"RebeccaPurple\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x66 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x33 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x99 as RgbValue,
    },
    color_name_table_T {
        name: b"Red\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Red1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Red2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Red3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Red4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"RosyBrown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbc as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8f as RgbValue,
    },
    color_name_table_T {
        name: b"RosyBrown1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc1 as RgbValue,
    },
    color_name_table_T {
        name: b"RosyBrown2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"RosyBrown3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9b as RgbValue,
    },
    color_name_table_T {
        name: b"RosyBrown4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"RoyalBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x41 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x69 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe1 as RgbValue,
    },
    color_name_table_T {
        name: b"RoyalBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x48 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x76 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"RoyalBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x43 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6e as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"RoyalBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x3a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5f as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"RoyalBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x27 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x40 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"SaddleBrown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x45 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x13 as RgbValue,
    },
    color_name_table_T {
        name: b"Salmon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xfa as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x72 as RgbValue,
    },
    color_name_table_T {
        name: b"Salmon1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x69 as RgbValue,
    },
    color_name_table_T {
        name: b"Salmon2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x62 as RgbValue,
    },
    color_name_table_T {
        name: b"Salmon3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x54 as RgbValue,
    },
    color_name_table_T {
        name: b"Salmon4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x39 as RgbValue,
    },
    color_name_table_T {
        name: b"SandyBrown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf4 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x60 as RgbValue,
    },
    color_name_table_T {
        name: b"SeaGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x57 as RgbValue,
    },
    color_name_table_T {
        name: b"SeaGreen1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x54 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x9f as RgbValue,
    },
    color_name_table_T {
        name: b"SeaGreen2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4e as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x94 as RgbValue,
    },
    color_name_table_T {
        name: b"SeaGreen3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x43 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"SeaGreen4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x2e as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x57 as RgbValue,
    },
    color_name_table_T {
        name: b"SeaShell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Seashell1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Seashell2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xde as RgbValue,
    },
    color_name_table_T {
        name: b"Seashell3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbf as RgbValue,
    },
    color_name_table_T {
        name: b"Seashell4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x82 as RgbValue,
    },
    color_name_table_T {
        name: b"Sienna\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x52 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2d as RgbValue,
    },
    color_name_table_T {
        name: b"Sienna1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x47 as RgbValue,
    },
    color_name_table_T {
        name: b"Sienna2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x79 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x42 as RgbValue,
    },
    color_name_table_T {
        name: b"Sienna3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x68 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x39 as RgbValue,
    },
    color_name_table_T {
        name: b"Sienna4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x47 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x26 as RgbValue,
    },
    color_name_table_T {
        name: b"Silver\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc0 as RgbValue,
    },
    color_name_table_T {
        name: b"SkyBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x87 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xce as RgbValue) << 8 as ::core::ffi::c_int
            | 0xeb as RgbValue,
    },
    color_name_table_T {
        name: b"SkyBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x87 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xce as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"SkyBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7e as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"SkyBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"SkyBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x70 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"SlateBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5a as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"SlateBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x83 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x6f as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"SlateBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x7a as RgbValue) << 16 as ::core::ffi::c_int
            | (0x67 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"SlateBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x69 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x59 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"SlateBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x47 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x70 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGray1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xc6 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGray2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb9 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd3 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGray3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9f as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb6 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGray4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x6c as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"SlateGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x70 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"Snow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"Snow1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xfa as RgbValue) << 8 as ::core::ffi::c_int
            | 0xfa as RgbValue,
    },
    color_name_table_T {
        name: b"Snow2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xe9 as RgbValue,
    },
    color_name_table_T {
        name: b"Snow3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc9 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xc9 as RgbValue,
    },
    color_name_table_T {
        name: b"Snow4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x89 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x89 as RgbValue,
    },
    color_name_table_T {
        name: b"SpringGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7f as RgbValue,
    },
    color_name_table_T {
        name: b"SpringGreen1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0x7f as RgbValue,
    },
    color_name_table_T {
        name: b"SpringGreen2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0x76 as RgbValue,
    },
    color_name_table_T {
        name: b"SpringGreen3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x66 as RgbValue,
    },
    color_name_table_T {
        name: b"SpringGreen4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x45 as RgbValue,
    },
    color_name_table_T {
        name: b"SteelBlue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x46 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb4 as RgbValue,
    },
    color_name_table_T {
        name: b"SteelBlue1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x63 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"SteelBlue2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x5c as RgbValue) << 16 as ::core::ffi::c_int
            | (0xac as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"SteelBlue3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x4f as RgbValue) << 16 as ::core::ffi::c_int
            | (0x94 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"SteelBlue4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x36 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x64 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Tan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd2 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb4 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8c as RgbValue,
    },
    color_name_table_T {
        name: b"Tan1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xa5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x4f as RgbValue,
    },
    color_name_table_T {
        name: b"Tan2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x9a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x49 as RgbValue,
    },
    color_name_table_T {
        name: b"Tan3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x85 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x3f as RgbValue,
    },
    color_name_table_T {
        name: b"Tan4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x2b as RgbValue,
    },
    color_name_table_T {
        name: b"Teal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"Thistle\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd8 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbf as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd8 as RgbValue,
    },
    color_name_table_T {
        name: b"Thistle1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe1 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Thistle2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd2 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Thistle3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xb5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Thistle4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7b as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Tomato\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x47 as RgbValue,
    },
    color_name_table_T {
        name: b"Tomato1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x63 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x47 as RgbValue,
    },
    color_name_table_T {
        name: b"Tomato2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x5c as RgbValue) << 8 as ::core::ffi::c_int
            | 0x42 as RgbValue,
    },
    color_name_table_T {
        name: b"Tomato3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x4f as RgbValue) << 8 as ::core::ffi::c_int
            | 0x39 as RgbValue,
    },
    color_name_table_T {
        name: b"Tomato4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x36 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x26 as RgbValue,
    },
    color_name_table_T {
        name: b"Turquoise\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x40 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xd0 as RgbValue,
    },
    color_name_table_T {
        name: b"Turquoise1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"Turquoise2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"Turquoise3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xc5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xcd as RgbValue,
    },
    color_name_table_T {
        name: b"Turquoise4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x86 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8b as RgbValue,
    },
    color_name_table_T {
        name: b"Violet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x82 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xee as RgbValue,
    },
    color_name_table_T {
        name: b"VioletRed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xd0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x20 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x90 as RgbValue,
    },
    color_name_table_T {
        name: b"VioletRed1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x96 as RgbValue,
    },
    color_name_table_T {
        name: b"VioletRed2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0x3a as RgbValue) << 8 as ::core::ffi::c_int
            | 0x8c as RgbValue,
    },
    color_name_table_T {
        name: b"VioletRed3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0x32 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x78 as RgbValue,
    },
    color_name_table_T {
        name: b"VioletRed4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x22 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x52 as RgbValue,
    },
    color_name_table_T {
        name: b"WebGray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"WebGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"WebGrey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x80 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"WebMaroon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"WebPurple\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x80 as RgbValue) << 16 as ::core::ffi::c_int
            | (0 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x80 as RgbValue,
    },
    color_name_table_T {
        name: b"Wheat\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xde as RgbValue) << 8 as ::core::ffi::c_int
            | 0xb3 as RgbValue,
    },
    color_name_table_T {
        name: b"Wheat1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xe7 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xba as RgbValue,
    },
    color_name_table_T {
        name: b"Wheat2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xd8 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xae as RgbValue,
    },
    color_name_table_T {
        name: b"Wheat3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xba as RgbValue) << 8 as ::core::ffi::c_int
            | 0x96 as RgbValue,
    },
    color_name_table_T {
        name: b"Wheat4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x7e as RgbValue) << 8 as ::core::ffi::c_int
            | 0x66 as RgbValue,
    },
    color_name_table_T {
        name: b"White\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0xff as RgbValue,
    },
    color_name_table_T {
        name: b"WhiteSmoke\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xf5 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xf5 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf5 as RgbValue,
    },
    color_name_table_T {
        name: b"X11Gray\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbe as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbe as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbe as RgbValue,
    },
    color_name_table_T {
        name: b"X11Green\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"X11Grey\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xbe as RgbValue) << 16 as ::core::ffi::c_int
            | (0xbe as RgbValue) << 8 as ::core::ffi::c_int
            | 0xbe as RgbValue,
    },
    color_name_table_T {
        name: b"X11Maroon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xb0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x30 as RgbValue) << 8 as ::core::ffi::c_int
            | 0x60 as RgbValue,
    },
    color_name_table_T {
        name: b"X11Purple\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xa0 as RgbValue) << 16 as ::core::ffi::c_int
            | (0x20 as RgbValue) << 8 as ::core::ffi::c_int
            | 0xf0 as RgbValue,
    },
    color_name_table_T {
        name: b"Yellow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Yellow1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xff as RgbValue) << 16 as ::core::ffi::c_int
            | (0xff as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Yellow2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xee as RgbValue) << 16 as ::core::ffi::c_int
            | (0xee as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Yellow3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0xcd as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"Yellow4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x8b as RgbValue) << 16 as ::core::ffi::c_int
            | (0x8b as RgbValue) << 8 as ::core::ffi::c_int
            | 0 as RgbValue,
    },
    color_name_table_T {
        name: b"YellowGreen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        color: (0x9a as RgbValue) << 16 as ::core::ffi::c_int
            | (0xcd as RgbValue) << 8 as ::core::ffi::c_int
            | 0x32 as RgbValue,
    },
    color_name_table_T {
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        color: 0 as RgbValue,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn name_to_color(
    mut name: *const ::core::ffi::c_char,
    mut idx: *mut ::core::ffi::c_int,
) -> RgbValue {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '#' as ::core::ffi::c_int
        && *(*__ctype_b_loc()).offset(*name.offset(1 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(2 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(3 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(4 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(5 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(6 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *name.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *idx = kColorIdxHex as ::core::ffi::c_int;
        return strtol(
            name.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            16 as ::core::ffi::c_int,
        ) as RgbValue;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"bg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0
        || strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"background\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0
    {
        *idx = kColorIdxBg as ::core::ffi::c_int;
        return normal_bg;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"fg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0
        || strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"foreground\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0
    {
        *idx = kColorIdxFg as ::core::ffi::c_int;
        return normal_fg;
    }
    let mut lo: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hi: ::core::ffi::c_int = ::core::mem::size_of::<[color_name_table_T; 708]>()
        .wrapping_div(::core::mem::size_of::<color_name_table_T>())
        .wrapping_div(
            (::core::mem::size_of::<[color_name_table_T; 708]>()
                .wrapping_rem(::core::mem::size_of::<color_name_table_T>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while lo < hi {
        let mut m: ::core::ffi::c_int = (lo + hi) / 2 as ::core::ffi::c_int;
        let mut cmp: ::core::ffi::c_int = strcasecmp(
            name as *mut ::core::ffi::c_char,
            (*color_name_table.ptr())[m as usize].name,
        );
        if cmp < 0 as ::core::ffi::c_int {
            hi = m;
        } else if cmp > 0 as ::core::ffi::c_int {
            lo = m + 1 as ::core::ffi::c_int;
        } else {
            *idx = m;
            return (*color_name_table.ptr())[m as usize].color;
        }
    }
    *idx = kColorIdxNone as ::core::ffi::c_int;
    return -1 as RgbValue;
}
#[no_mangle]
pub unsafe extern "C" fn coloridx_to_name(
    mut idx: ::core::ffi::c_int,
    mut val: ::core::ffi::c_int,
    mut hexbuf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if idx >= 0 as ::core::ffi::c_int {
        return (*color_name_table.ptr())[idx as usize].name;
    }
    match idx {
        -1 => return ::core::ptr::null::<::core::ffi::c_char>(),
        -3 => return b"fg\0".as_ptr() as *const ::core::ffi::c_char,
        -4 => return b"bg\0".as_ptr() as *const ::core::ffi::c_char,
        -2 => {
            snprintf(
                hexbuf as *mut ::core::ffi::c_char,
                (7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                val,
            );
            return hexbuf as *const ::core::ffi::c_char;
        }
        _ => {
            abort();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn name_to_ctermcolor(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = if (*name as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
        || *name as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
    {
        *name as ::core::ffi::c_int
    } else {
        *name as ::core::ffi::c_int - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    };
    i = ::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
        .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
                .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if off
            == *(*color_names.ptr())[i as usize].offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            && strcasecmp(
                name.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                (*color_names.ptr())[i as usize].offset(1 as ::core::ffi::c_int as isize),
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
    }
    if i < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    let mut bold: TriState = kNone;
    return lookup_color(i, false_0 != 0, &raw mut bold);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
