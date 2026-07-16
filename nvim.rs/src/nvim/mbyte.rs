use ::c2rust_bitfields;
extern "C" {
    pub type MsgpackRpcRequestHandler;
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
    fn tolower(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn toupper(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn iconv_close(__cd: iconv_t) -> ::core::ffi::c_int;
    fn iconv_open(
        __tocode: *const ::core::ffi::c_char,
        __fromcode: *const ::core::ffi::c_char,
    ) -> iconv_t;
    fn iconv(
        __cd: iconv_t,
        __inbuf: *mut *mut ::core::ffi::c_char,
        __inbytesleft: *mut size_t,
        __outbuf: *mut *mut ::core::ffi::c_char,
        __outbytesleft: *mut size_t,
    ) -> size_t;
    fn setlocale(
        __category: ::core::ffi::c_int,
        __locale: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn utf8proc_get_property(codepoint: utf8proc_int32_t) -> *const utf8proc_property_t;
    fn utf8proc_decompose_char(
        codepoint: utf8proc_int32_t,
        dst: *mut utf8proc_int32_t,
        bufsize: utf8proc_ssize_t,
        options: utf8proc_option_t,
        last_boundclass: *mut ::core::ffi::c_int,
    ) -> utf8proc_ssize_t;
    fn utf8proc_grapheme_break_stateful(
        codepoint1: utf8proc_int32_t,
        codepoint2: utf8proc_int32_t,
        state: *mut utf8proc_int32_t,
    ) -> utf8proc_bool;
    fn utf8proc_grapheme_break(
        codepoint1: utf8proc_int32_t,
        codepoint2: utf8proc_int32_t,
    ) -> utf8proc_bool;
    fn utf8proc_tolower(c: utf8proc_int32_t) -> utf8proc_int32_t;
    fn utf8proc_toupper(c: utf8proc_int32_t) -> utf8proc_int32_t;
    fn towlower(__wc: wint_t) -> wint_t;
    fn towupper(__wc: wint_t) -> wint_t;
    fn arabic_combine(one: ::core::ffi::c_int, two: ::core::ffi::c_int) -> bool;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_iswordc_tab(c: ::core::ffi::c_int, chartab: *const uint64_t) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    static mut p_ambw: *mut ::core::ffi::c_char;
    static mut cmp_flags: ::core::ffi::c_uint;
    static mut p_enc: *mut ::core::ffi::c_char;
    static mut p_emoji: ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_listreq: [::core::ffi::c_char; 0];
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_check_for_string_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string_buf(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn beep_flush();
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut fenc_default: *mut ::core::ffi::c_char;
    static mut IObuff: [::core::ffi::c_char; 1025];
    fn schar_from_buf(buf: *const ::core::ffi::c_char, len: size_t) -> schar_T;
    fn mark_mb_adjustpos(buf: *mut buf_T, lp: *mut pos_T);
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn changed_window_setting_all();
    fn check_chars_options() -> *const ::core::ffi::c_char;
    fn os_getenv_noalloc(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn nl_langinfo(__item: nl_item) -> *mut ::core::ffi::c_char;
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
pub type size_t = usize;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type ptrdiff_t = isize;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type utf8proc_int16_t = int16_t;
pub type utf8proc_uint16_t = uint16_t;
pub type utf8proc_int32_t = int32_t;
pub type utf8proc_ssize_t = ptrdiff_t;
pub type utf8proc_bool = bool;
pub type utf8proc_option_t = ::core::ffi::c_uint;
pub const UTF8PROC_STRIPNA: utf8proc_option_t = 16384;
pub const UTF8PROC_STRIPMARK: utf8proc_option_t = 8192;
pub const UTF8PROC_LUMP: utf8proc_option_t = 4096;
pub const UTF8PROC_CHARBOUND: utf8proc_option_t = 2048;
pub const UTF8PROC_CASEFOLD: utf8proc_option_t = 1024;
pub const UTF8PROC_STRIPCC: utf8proc_option_t = 512;
pub const UTF8PROC_NLF2LF: utf8proc_option_t = 384;
pub const UTF8PROC_NLF2PS: utf8proc_option_t = 256;
pub const UTF8PROC_NLF2LS: utf8proc_option_t = 128;
pub const UTF8PROC_REJECTNA: utf8proc_option_t = 64;
pub const UTF8PROC_IGNORE: utf8proc_option_t = 32;
pub const UTF8PROC_DECOMPOSE: utf8proc_option_t = 16;
pub const UTF8PROC_COMPOSE: utf8proc_option_t = 8;
pub const UTF8PROC_COMPAT: utf8proc_option_t = 4;
pub const UTF8PROC_STABLE: utf8proc_option_t = 2;
pub const UTF8PROC_NULLTERM: utf8proc_option_t = 1;
pub type utf8proc_propval_t = utf8proc_int16_t;
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct utf8proc_property_struct {
    pub category: utf8proc_propval_t,
    pub combining_class: utf8proc_propval_t,
    pub bidi_class: utf8proc_propval_t,
    pub decomp_type: utf8proc_propval_t,
    pub decomp_seqindex: utf8proc_uint16_t,
    pub casefold_seqindex: utf8proc_uint16_t,
    pub uppercase_seqindex: utf8proc_uint16_t,
    pub lowercase_seqindex: utf8proc_uint16_t,
    pub titlecase_seqindex: utf8proc_uint16_t,
    #[bitfield(name = "comb_index", ty = "utf8proc_uint16_t", bits = "0..=9")]
    #[bitfield(name = "comb_length", ty = "utf8proc_uint16_t", bits = "10..=14")]
    #[bitfield(name = "comb_issecond", ty = "utf8proc_uint16_t", bits = "15..=15")]
    #[bitfield(name = "bidi_mirrored", ty = "::core::ffi::c_uint", bits = "16..=16")]
    #[bitfield(name = "comp_exclusion", ty = "::core::ffi::c_uint", bits = "17..=17")]
    #[bitfield(name = "ignorable", ty = "::core::ffi::c_uint", bits = "18..=18")]
    #[bitfield(
        name = "control_boundary",
        ty = "::core::ffi::c_uint",
        bits = "19..=19"
    )]
    #[bitfield(name = "charwidth", ty = "::core::ffi::c_uint", bits = "20..=21")]
    #[bitfield(name = "ambiguous_width", ty = "::core::ffi::c_uint", bits = "22..=22")]
    #[bitfield(name = "pad", ty = "::core::ffi::c_uint", bits = "23..=23")]
    #[bitfield(name = "boundclass", ty = "::core::ffi::c_uint", bits = "24..=29")]
    #[bitfield(
        name = "indic_conjunct_break",
        ty = "::core::ffi::c_uint",
        bits = "30..=31"
    )]
    pub comb_index_comb_length_comb_issecond_bidi_mirrored_comp_exclusion_ignorable_control_boundary_charwidth_ambiguous_width_pad_boundclass_indic_conjunct_break:
        [u8; 4],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 2],
}
pub type utf8proc_property_t = utf8proc_property_struct;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const UTF8PROC_CATEGORY_CO: C2Rust_Unnamed_0 = 29;
pub const UTF8PROC_CATEGORY_CS: C2Rust_Unnamed_0 = 28;
pub const UTF8PROC_CATEGORY_CF: C2Rust_Unnamed_0 = 27;
pub const UTF8PROC_CATEGORY_CC: C2Rust_Unnamed_0 = 26;
pub const UTF8PROC_CATEGORY_ZP: C2Rust_Unnamed_0 = 25;
pub const UTF8PROC_CATEGORY_ZL: C2Rust_Unnamed_0 = 24;
pub const UTF8PROC_CATEGORY_ZS: C2Rust_Unnamed_0 = 23;
pub const UTF8PROC_CATEGORY_SO: C2Rust_Unnamed_0 = 22;
pub const UTF8PROC_CATEGORY_SK: C2Rust_Unnamed_0 = 21;
pub const UTF8PROC_CATEGORY_SC: C2Rust_Unnamed_0 = 20;
pub const UTF8PROC_CATEGORY_SM: C2Rust_Unnamed_0 = 19;
pub const UTF8PROC_CATEGORY_PO: C2Rust_Unnamed_0 = 18;
pub const UTF8PROC_CATEGORY_PF: C2Rust_Unnamed_0 = 17;
pub const UTF8PROC_CATEGORY_PI: C2Rust_Unnamed_0 = 16;
pub const UTF8PROC_CATEGORY_PE: C2Rust_Unnamed_0 = 15;
pub const UTF8PROC_CATEGORY_PS: C2Rust_Unnamed_0 = 14;
pub const UTF8PROC_CATEGORY_PD: C2Rust_Unnamed_0 = 13;
pub const UTF8PROC_CATEGORY_PC: C2Rust_Unnamed_0 = 12;
pub const UTF8PROC_CATEGORY_NO: C2Rust_Unnamed_0 = 11;
pub const UTF8PROC_CATEGORY_NL: C2Rust_Unnamed_0 = 10;
pub const UTF8PROC_CATEGORY_ND: C2Rust_Unnamed_0 = 9;
pub const UTF8PROC_CATEGORY_ME: C2Rust_Unnamed_0 = 8;
pub const UTF8PROC_CATEGORY_MC: C2Rust_Unnamed_0 = 7;
pub const UTF8PROC_CATEGORY_MN: C2Rust_Unnamed_0 = 6;
pub const UTF8PROC_CATEGORY_LO: C2Rust_Unnamed_0 = 5;
pub const UTF8PROC_CATEGORY_LM: C2Rust_Unnamed_0 = 4;
pub const UTF8PROC_CATEGORY_LT: C2Rust_Unnamed_0 = 3;
pub const UTF8PROC_CATEGORY_LL: C2Rust_Unnamed_0 = 2;
pub const UTF8PROC_CATEGORY_LU: C2Rust_Unnamed_0 = 1;
pub const UTF8PROC_CATEGORY_CN: C2Rust_Unnamed_0 = 0;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const UTF8PROC_BOUNDCLASS_E_ZWG: C2Rust_Unnamed_1 = 20;
pub const UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC: C2Rust_Unnamed_1 = 19;
pub const UTF8PROC_BOUNDCLASS_E_BASE_GAZ: C2Rust_Unnamed_1 = 18;
pub const UTF8PROC_BOUNDCLASS_GLUE_AFTER_ZWJ: C2Rust_Unnamed_1 = 17;
pub const UTF8PROC_BOUNDCLASS_E_MODIFIER: C2Rust_Unnamed_1 = 16;
pub const UTF8PROC_BOUNDCLASS_E_BASE: C2Rust_Unnamed_1 = 15;
pub const UTF8PROC_BOUNDCLASS_ZWJ: C2Rust_Unnamed_1 = 14;
pub const UTF8PROC_BOUNDCLASS_PREPEND: C2Rust_Unnamed_1 = 13;
pub const UTF8PROC_BOUNDCLASS_SPACINGMARK: C2Rust_Unnamed_1 = 12;
pub const UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR: C2Rust_Unnamed_1 = 11;
pub const UTF8PROC_BOUNDCLASS_LVT: C2Rust_Unnamed_1 = 10;
pub const UTF8PROC_BOUNDCLASS_LV: C2Rust_Unnamed_1 = 9;
pub const UTF8PROC_BOUNDCLASS_T: C2Rust_Unnamed_1 = 8;
pub const UTF8PROC_BOUNDCLASS_V: C2Rust_Unnamed_1 = 7;
pub const UTF8PROC_BOUNDCLASS_L: C2Rust_Unnamed_1 = 6;
pub const UTF8PROC_BOUNDCLASS_EXTEND: C2Rust_Unnamed_1 = 5;
pub const UTF8PROC_BOUNDCLASS_CONTROL: C2Rust_Unnamed_1 = 4;
pub const UTF8PROC_BOUNDCLASS_LF: C2Rust_Unnamed_1 = 3;
pub const UTF8PROC_BOUNDCLASS_CR: C2Rust_Unnamed_1 = 2;
pub const UTF8PROC_BOUNDCLASS_OTHER: C2Rust_Unnamed_1 = 1;
pub const UTF8PROC_BOUNDCLASS_START: C2Rust_Unnamed_1 = 0;
pub type wint_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
pub type linenr_T = int32_t;
pub type colnr_T = ::core::ffi::c_int;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_2 = 2147483647;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type proftime_T = uint64_t;
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
    pub b_wininfo: C2Rust_Unnamed_14,
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
    pub b_signcols: C2Rust_Unnamed_6,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_4,
    pub update_callbacks: C2Rust_Unnamed_3,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
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
pub struct C2Rust_Unnamed_4 {
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
    pub data: C2Rust_Unnamed_5,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
pub struct C2Rust_Unnamed_6 {
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
pub type disptick_T = uint64_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_7,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
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
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_8,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
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
    pub fc_fixvar: [C2Rust_Unnamed_9; 12],
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
pub struct C2Rust_Unnamed_9 {
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
    pub uh_next: C2Rust_Unnamed_13,
    pub uh_prev: C2Rust_Unnamed_12,
    pub uh_alt_next: C2Rust_Unnamed_11,
    pub uh_alt_prev: C2Rust_Unnamed_10,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
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
pub struct C2Rust_Unnamed_14 {
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
    pub type_0: C2Rust_Unnamed_15,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_15 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_15 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_15 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_15 = 0;
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
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type WorkingStatus = ::core::ffi::c_uint;
pub const kBroken: WorkingStatus = 2;
pub const kWorking: WorkingStatus = 1;
pub const kUnknown: WorkingStatus = 0;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptCmpFlagKeepascii: C2Rust_Unnamed_16 = 2;
pub const kOptCmpFlagInternal: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_17 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_17 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_17 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_17 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_17 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_17 = 20;
pub const UPD_VALID: C2Rust_Unnamed_17 = 10;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_18 = 6;
pub const MB_MAXBYTES: C2Rust_Unnamed_18 = 21;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const ENC_MACROMAN: C2Rust_Unnamed_19 = 2048;
pub const ENC_LATIN9: C2Rust_Unnamed_19 = 1024;
pub const ENC_LATIN1: C2Rust_Unnamed_19 = 512;
pub const ENC_2WORD: C2Rust_Unnamed_19 = 256;
pub const ENC_4BYTE: C2Rust_Unnamed_19 = 128;
pub const ENC_2BYTE: C2Rust_Unnamed_19 = 64;
pub const ENC_ENDIAN_L: C2Rust_Unnamed_19 = 32;
pub const ENC_ENDIAN_B: C2Rust_Unnamed_19 = 16;
pub const ENC_UNICODE: C2Rust_Unnamed_19 = 4;
pub const ENC_DBCS: C2Rust_Unnamed_19 = 2;
pub const ENC_8BIT: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_20 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_20 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_20 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_20 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_20 = 1;
pub const CONV_NONE: C2Rust_Unnamed_20 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharBoundsOff {
    pub begin_off: int8_t,
    pub end_off: int8_t,
}
pub type GraphemeState = utf8proc_int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub name: *const ::core::ffi::c_char,
    pub prop: ::core::ffi::c_int,
    pub codepage: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clinterval {
    pub first: ::core::ffi::c_uint,
    pub last: ::core::ffi::c_uint,
    pub cls: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cw_interval_T {
    pub first: int64_t,
    pub last: int64_t,
    pub width: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct interval {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub name: *const ::core::ffi::c_char,
    pub canon: ::core::ffi::c_int,
}
pub const CODESET: C2Rust_Unnamed_23 = 14;
pub type nl_item = ::core::ffi::c_int;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const _NL_NUM: C2Rust_Unnamed_23 = 786449;
pub const _NL_NUM_LC_IDENTIFICATION: C2Rust_Unnamed_23 = 786448;
pub const _NL_IDENTIFICATION_CODESET: C2Rust_Unnamed_23 = 786447;
pub const _NL_IDENTIFICATION_CATEGORY: C2Rust_Unnamed_23 = 786446;
pub const _NL_IDENTIFICATION_DATE: C2Rust_Unnamed_23 = 786445;
pub const _NL_IDENTIFICATION_REVISION: C2Rust_Unnamed_23 = 786444;
pub const _NL_IDENTIFICATION_ABBREVIATION: C2Rust_Unnamed_23 = 786443;
pub const _NL_IDENTIFICATION_APPLICATION: C2Rust_Unnamed_23 = 786442;
pub const _NL_IDENTIFICATION_AUDIENCE: C2Rust_Unnamed_23 = 786441;
pub const _NL_IDENTIFICATION_TERRITORY: C2Rust_Unnamed_23 = 786440;
pub const _NL_IDENTIFICATION_LANGUAGE: C2Rust_Unnamed_23 = 786439;
pub const _NL_IDENTIFICATION_FAX: C2Rust_Unnamed_23 = 786438;
pub const _NL_IDENTIFICATION_TEL: C2Rust_Unnamed_23 = 786437;
pub const _NL_IDENTIFICATION_EMAIL: C2Rust_Unnamed_23 = 786436;
pub const _NL_IDENTIFICATION_CONTACT: C2Rust_Unnamed_23 = 786435;
pub const _NL_IDENTIFICATION_ADDRESS: C2Rust_Unnamed_23 = 786434;
pub const _NL_IDENTIFICATION_SOURCE: C2Rust_Unnamed_23 = 786433;
pub const _NL_IDENTIFICATION_TITLE: C2Rust_Unnamed_23 = 786432;
pub const _NL_NUM_LC_MEASUREMENT: C2Rust_Unnamed_23 = 720898;
pub const _NL_MEASUREMENT_CODESET: C2Rust_Unnamed_23 = 720897;
pub const _NL_MEASUREMENT_MEASUREMENT: C2Rust_Unnamed_23 = 720896;
pub const _NL_NUM_LC_TELEPHONE: C2Rust_Unnamed_23 = 655365;
pub const _NL_TELEPHONE_CODESET: C2Rust_Unnamed_23 = 655364;
pub const _NL_TELEPHONE_INT_PREFIX: C2Rust_Unnamed_23 = 655363;
pub const _NL_TELEPHONE_INT_SELECT: C2Rust_Unnamed_23 = 655362;
pub const _NL_TELEPHONE_TEL_DOM_FMT: C2Rust_Unnamed_23 = 655361;
pub const _NL_TELEPHONE_TEL_INT_FMT: C2Rust_Unnamed_23 = 655360;
pub const _NL_NUM_LC_ADDRESS: C2Rust_Unnamed_23 = 589837;
pub const _NL_ADDRESS_CODESET: C2Rust_Unnamed_23 = 589836;
pub const _NL_ADDRESS_LANG_LIB: C2Rust_Unnamed_23 = 589835;
pub const _NL_ADDRESS_LANG_TERM: C2Rust_Unnamed_23 = 589834;
pub const _NL_ADDRESS_LANG_AB: C2Rust_Unnamed_23 = 589833;
pub const _NL_ADDRESS_LANG_NAME: C2Rust_Unnamed_23 = 589832;
pub const _NL_ADDRESS_COUNTRY_ISBN: C2Rust_Unnamed_23 = 589831;
pub const _NL_ADDRESS_COUNTRY_NUM: C2Rust_Unnamed_23 = 589830;
pub const _NL_ADDRESS_COUNTRY_CAR: C2Rust_Unnamed_23 = 589829;
pub const _NL_ADDRESS_COUNTRY_AB3: C2Rust_Unnamed_23 = 589828;
pub const _NL_ADDRESS_COUNTRY_AB2: C2Rust_Unnamed_23 = 589827;
pub const _NL_ADDRESS_COUNTRY_POST: C2Rust_Unnamed_23 = 589826;
pub const _NL_ADDRESS_COUNTRY_NAME: C2Rust_Unnamed_23 = 589825;
pub const _NL_ADDRESS_POSTAL_FMT: C2Rust_Unnamed_23 = 589824;
pub const _NL_NUM_LC_NAME: C2Rust_Unnamed_23 = 524295;
pub const _NL_NAME_CODESET: C2Rust_Unnamed_23 = 524294;
pub const _NL_NAME_NAME_MS: C2Rust_Unnamed_23 = 524293;
pub const _NL_NAME_NAME_MISS: C2Rust_Unnamed_23 = 524292;
pub const _NL_NAME_NAME_MRS: C2Rust_Unnamed_23 = 524291;
pub const _NL_NAME_NAME_MR: C2Rust_Unnamed_23 = 524290;
pub const _NL_NAME_NAME_GEN: C2Rust_Unnamed_23 = 524289;
pub const _NL_NAME_NAME_FMT: C2Rust_Unnamed_23 = 524288;
pub const _NL_NUM_LC_PAPER: C2Rust_Unnamed_23 = 458755;
pub const _NL_PAPER_CODESET: C2Rust_Unnamed_23 = 458754;
pub const _NL_PAPER_WIDTH: C2Rust_Unnamed_23 = 458753;
pub const _NL_PAPER_HEIGHT: C2Rust_Unnamed_23 = 458752;
pub const _NL_NUM_LC_MESSAGES: C2Rust_Unnamed_23 = 327685;
pub const _NL_MESSAGES_CODESET: C2Rust_Unnamed_23 = 327684;
pub const __NOSTR: C2Rust_Unnamed_23 = 327683;
pub const __YESSTR: C2Rust_Unnamed_23 = 327682;
pub const __NOEXPR: C2Rust_Unnamed_23 = 327681;
pub const __YESEXPR: C2Rust_Unnamed_23 = 327680;
pub const _NL_NUM_LC_NUMERIC: C2Rust_Unnamed_23 = 65542;
pub const _NL_NUMERIC_CODESET: C2Rust_Unnamed_23 = 65541;
pub const _NL_NUMERIC_THOUSANDS_SEP_WC: C2Rust_Unnamed_23 = 65540;
pub const _NL_NUMERIC_DECIMAL_POINT_WC: C2Rust_Unnamed_23 = 65539;
pub const __GROUPING: C2Rust_Unnamed_23 = 65538;
pub const THOUSEP: C2Rust_Unnamed_23 = 65537;
pub const __THOUSANDS_SEP: C2Rust_Unnamed_23 = 65537;
pub const RADIXCHAR: C2Rust_Unnamed_23 = 65536;
pub const __DECIMAL_POINT: C2Rust_Unnamed_23 = 65536;
pub const _NL_NUM_LC_MONETARY: C2Rust_Unnamed_23 = 262190;
pub const _NL_MONETARY_CODESET: C2Rust_Unnamed_23 = 262189;
pub const _NL_MONETARY_THOUSANDS_SEP_WC: C2Rust_Unnamed_23 = 262188;
pub const _NL_MONETARY_DECIMAL_POINT_WC: C2Rust_Unnamed_23 = 262187;
pub const _NL_MONETARY_CONVERSION_RATE: C2Rust_Unnamed_23 = 262186;
pub const _NL_MONETARY_DUO_VALID_TO: C2Rust_Unnamed_23 = 262185;
pub const _NL_MONETARY_DUO_VALID_FROM: C2Rust_Unnamed_23 = 262184;
pub const _NL_MONETARY_UNO_VALID_TO: C2Rust_Unnamed_23 = 262183;
pub const _NL_MONETARY_UNO_VALID_FROM: C2Rust_Unnamed_23 = 262182;
pub const _NL_MONETARY_DUO_INT_N_SIGN_POSN: C2Rust_Unnamed_23 = 262181;
pub const _NL_MONETARY_DUO_INT_P_SIGN_POSN: C2Rust_Unnamed_23 = 262180;
pub const _NL_MONETARY_DUO_N_SIGN_POSN: C2Rust_Unnamed_23 = 262179;
pub const _NL_MONETARY_DUO_P_SIGN_POSN: C2Rust_Unnamed_23 = 262178;
pub const _NL_MONETARY_DUO_INT_N_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262177;
pub const _NL_MONETARY_DUO_INT_N_CS_PRECEDES: C2Rust_Unnamed_23 = 262176;
pub const _NL_MONETARY_DUO_INT_P_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262175;
pub const _NL_MONETARY_DUO_INT_P_CS_PRECEDES: C2Rust_Unnamed_23 = 262174;
pub const _NL_MONETARY_DUO_N_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262173;
pub const _NL_MONETARY_DUO_N_CS_PRECEDES: C2Rust_Unnamed_23 = 262172;
pub const _NL_MONETARY_DUO_P_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262171;
pub const _NL_MONETARY_DUO_P_CS_PRECEDES: C2Rust_Unnamed_23 = 262170;
pub const _NL_MONETARY_DUO_FRAC_DIGITS: C2Rust_Unnamed_23 = 262169;
pub const _NL_MONETARY_DUO_INT_FRAC_DIGITS: C2Rust_Unnamed_23 = 262168;
pub const _NL_MONETARY_DUO_CURRENCY_SYMBOL: C2Rust_Unnamed_23 = 262167;
pub const _NL_MONETARY_DUO_INT_CURR_SYMBOL: C2Rust_Unnamed_23 = 262166;
pub const __INT_N_SIGN_POSN: C2Rust_Unnamed_23 = 262165;
pub const __INT_P_SIGN_POSN: C2Rust_Unnamed_23 = 262164;
pub const __INT_N_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262163;
pub const __INT_N_CS_PRECEDES: C2Rust_Unnamed_23 = 262162;
pub const __INT_P_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262161;
pub const __INT_P_CS_PRECEDES: C2Rust_Unnamed_23 = 262160;
pub const _NL_MONETARY_CRNCYSTR: C2Rust_Unnamed_23 = 262159;
pub const __N_SIGN_POSN: C2Rust_Unnamed_23 = 262158;
pub const __P_SIGN_POSN: C2Rust_Unnamed_23 = 262157;
pub const __N_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262156;
pub const __N_CS_PRECEDES: C2Rust_Unnamed_23 = 262155;
pub const __P_SEP_BY_SPACE: C2Rust_Unnamed_23 = 262154;
pub const __P_CS_PRECEDES: C2Rust_Unnamed_23 = 262153;
pub const __FRAC_DIGITS: C2Rust_Unnamed_23 = 262152;
pub const __INT_FRAC_DIGITS: C2Rust_Unnamed_23 = 262151;
pub const __NEGATIVE_SIGN: C2Rust_Unnamed_23 = 262150;
pub const __POSITIVE_SIGN: C2Rust_Unnamed_23 = 262149;
pub const __MON_GROUPING: C2Rust_Unnamed_23 = 262148;
pub const __MON_THOUSANDS_SEP: C2Rust_Unnamed_23 = 262147;
pub const __MON_DECIMAL_POINT: C2Rust_Unnamed_23 = 262146;
pub const __CURRENCY_SYMBOL: C2Rust_Unnamed_23 = 262145;
pub const __INT_CURR_SYMBOL: C2Rust_Unnamed_23 = 262144;
pub const _NL_NUM_LC_CTYPE: C2Rust_Unnamed_23 = 86;
pub const _NL_CTYPE_EXTRA_MAP_14: C2Rust_Unnamed_23 = 85;
pub const _NL_CTYPE_EXTRA_MAP_13: C2Rust_Unnamed_23 = 84;
pub const _NL_CTYPE_EXTRA_MAP_12: C2Rust_Unnamed_23 = 83;
pub const _NL_CTYPE_EXTRA_MAP_11: C2Rust_Unnamed_23 = 82;
pub const _NL_CTYPE_EXTRA_MAP_10: C2Rust_Unnamed_23 = 81;
pub const _NL_CTYPE_EXTRA_MAP_9: C2Rust_Unnamed_23 = 80;
pub const _NL_CTYPE_EXTRA_MAP_8: C2Rust_Unnamed_23 = 79;
pub const _NL_CTYPE_EXTRA_MAP_7: C2Rust_Unnamed_23 = 78;
pub const _NL_CTYPE_EXTRA_MAP_6: C2Rust_Unnamed_23 = 77;
pub const _NL_CTYPE_EXTRA_MAP_5: C2Rust_Unnamed_23 = 76;
pub const _NL_CTYPE_EXTRA_MAP_4: C2Rust_Unnamed_23 = 75;
pub const _NL_CTYPE_EXTRA_MAP_3: C2Rust_Unnamed_23 = 74;
pub const _NL_CTYPE_EXTRA_MAP_2: C2Rust_Unnamed_23 = 73;
pub const _NL_CTYPE_EXTRA_MAP_1: C2Rust_Unnamed_23 = 72;
pub const _NL_CTYPE_NONASCII_CASE: C2Rust_Unnamed_23 = 71;
pub const _NL_CTYPE_MAP_TO_NONASCII: C2Rust_Unnamed_23 = 70;
pub const _NL_CTYPE_TRANSLIT_IGNORE: C2Rust_Unnamed_23 = 69;
pub const _NL_CTYPE_TRANSLIT_IGNORE_LEN: C2Rust_Unnamed_23 = 68;
pub const _NL_CTYPE_TRANSLIT_DEFAULT_MISSING: C2Rust_Unnamed_23 = 67;
pub const _NL_CTYPE_TRANSLIT_DEFAULT_MISSING_LEN: C2Rust_Unnamed_23 = 66;
pub const _NL_CTYPE_TRANSLIT_TO_TBL: C2Rust_Unnamed_23 = 65;
pub const _NL_CTYPE_TRANSLIT_TO_IDX: C2Rust_Unnamed_23 = 64;
pub const _NL_CTYPE_TRANSLIT_FROM_TBL: C2Rust_Unnamed_23 = 63;
pub const _NL_CTYPE_TRANSLIT_FROM_IDX: C2Rust_Unnamed_23 = 62;
pub const _NL_CTYPE_TRANSLIT_TAB_SIZE: C2Rust_Unnamed_23 = 61;
pub const _NL_CTYPE_OUTDIGIT9_WC: C2Rust_Unnamed_23 = 60;
pub const _NL_CTYPE_OUTDIGIT8_WC: C2Rust_Unnamed_23 = 59;
pub const _NL_CTYPE_OUTDIGIT7_WC: C2Rust_Unnamed_23 = 58;
pub const _NL_CTYPE_OUTDIGIT6_WC: C2Rust_Unnamed_23 = 57;
pub const _NL_CTYPE_OUTDIGIT5_WC: C2Rust_Unnamed_23 = 56;
pub const _NL_CTYPE_OUTDIGIT4_WC: C2Rust_Unnamed_23 = 55;
pub const _NL_CTYPE_OUTDIGIT3_WC: C2Rust_Unnamed_23 = 54;
pub const _NL_CTYPE_OUTDIGIT2_WC: C2Rust_Unnamed_23 = 53;
pub const _NL_CTYPE_OUTDIGIT1_WC: C2Rust_Unnamed_23 = 52;
pub const _NL_CTYPE_OUTDIGIT0_WC: C2Rust_Unnamed_23 = 51;
pub const _NL_CTYPE_OUTDIGIT9_MB: C2Rust_Unnamed_23 = 50;
pub const _NL_CTYPE_OUTDIGIT8_MB: C2Rust_Unnamed_23 = 49;
pub const _NL_CTYPE_OUTDIGIT7_MB: C2Rust_Unnamed_23 = 48;
pub const _NL_CTYPE_OUTDIGIT6_MB: C2Rust_Unnamed_23 = 47;
pub const _NL_CTYPE_OUTDIGIT5_MB: C2Rust_Unnamed_23 = 46;
pub const _NL_CTYPE_OUTDIGIT4_MB: C2Rust_Unnamed_23 = 45;
pub const _NL_CTYPE_OUTDIGIT3_MB: C2Rust_Unnamed_23 = 44;
pub const _NL_CTYPE_OUTDIGIT2_MB: C2Rust_Unnamed_23 = 43;
pub const _NL_CTYPE_OUTDIGIT1_MB: C2Rust_Unnamed_23 = 42;
pub const _NL_CTYPE_OUTDIGIT0_MB: C2Rust_Unnamed_23 = 41;
pub const _NL_CTYPE_INDIGITS9_WC: C2Rust_Unnamed_23 = 40;
pub const _NL_CTYPE_INDIGITS8_WC: C2Rust_Unnamed_23 = 39;
pub const _NL_CTYPE_INDIGITS7_WC: C2Rust_Unnamed_23 = 38;
pub const _NL_CTYPE_INDIGITS6_WC: C2Rust_Unnamed_23 = 37;
pub const _NL_CTYPE_INDIGITS5_WC: C2Rust_Unnamed_23 = 36;
pub const _NL_CTYPE_INDIGITS4_WC: C2Rust_Unnamed_23 = 35;
pub const _NL_CTYPE_INDIGITS3_WC: C2Rust_Unnamed_23 = 34;
pub const _NL_CTYPE_INDIGITS2_WC: C2Rust_Unnamed_23 = 33;
pub const _NL_CTYPE_INDIGITS1_WC: C2Rust_Unnamed_23 = 32;
pub const _NL_CTYPE_INDIGITS0_WC: C2Rust_Unnamed_23 = 31;
pub const _NL_CTYPE_INDIGITS_WC_LEN: C2Rust_Unnamed_23 = 30;
pub const _NL_CTYPE_INDIGITS9_MB: C2Rust_Unnamed_23 = 29;
pub const _NL_CTYPE_INDIGITS8_MB: C2Rust_Unnamed_23 = 28;
pub const _NL_CTYPE_INDIGITS7_MB: C2Rust_Unnamed_23 = 27;
pub const _NL_CTYPE_INDIGITS6_MB: C2Rust_Unnamed_23 = 26;
pub const _NL_CTYPE_INDIGITS5_MB: C2Rust_Unnamed_23 = 25;
pub const _NL_CTYPE_INDIGITS4_MB: C2Rust_Unnamed_23 = 24;
pub const _NL_CTYPE_INDIGITS3_MB: C2Rust_Unnamed_23 = 23;
pub const _NL_CTYPE_INDIGITS2_MB: C2Rust_Unnamed_23 = 22;
pub const _NL_CTYPE_INDIGITS1_MB: C2Rust_Unnamed_23 = 21;
pub const _NL_CTYPE_INDIGITS0_MB: C2Rust_Unnamed_23 = 20;
pub const _NL_CTYPE_INDIGITS_MB_LEN: C2Rust_Unnamed_23 = 19;
pub const _NL_CTYPE_MAP_OFFSET: C2Rust_Unnamed_23 = 18;
pub const _NL_CTYPE_CLASS_OFFSET: C2Rust_Unnamed_23 = 17;
pub const _NL_CTYPE_TOLOWER32: C2Rust_Unnamed_23 = 16;
pub const _NL_CTYPE_TOUPPER32: C2Rust_Unnamed_23 = 15;
pub const _NL_CTYPE_CODESET_NAME: C2Rust_Unnamed_23 = 14;
pub const _NL_CTYPE_MB_CUR_MAX: C2Rust_Unnamed_23 = 13;
pub const _NL_CTYPE_WIDTH: C2Rust_Unnamed_23 = 12;
pub const _NL_CTYPE_MAP_NAMES: C2Rust_Unnamed_23 = 11;
pub const _NL_CTYPE_CLASS_NAMES: C2Rust_Unnamed_23 = 10;
pub const _NL_CTYPE_GAP6: C2Rust_Unnamed_23 = 9;
pub const _NL_CTYPE_GAP5: C2Rust_Unnamed_23 = 8;
pub const _NL_CTYPE_GAP4: C2Rust_Unnamed_23 = 7;
pub const _NL_CTYPE_GAP3: C2Rust_Unnamed_23 = 6;
pub const _NL_CTYPE_CLASS32: C2Rust_Unnamed_23 = 5;
pub const _NL_CTYPE_GAP2: C2Rust_Unnamed_23 = 4;
pub const _NL_CTYPE_TOLOWER: C2Rust_Unnamed_23 = 3;
pub const _NL_CTYPE_GAP1: C2Rust_Unnamed_23 = 2;
pub const _NL_CTYPE_TOUPPER: C2Rust_Unnamed_23 = 1;
pub const _NL_CTYPE_CLASS: C2Rust_Unnamed_23 = 0;
pub const _NL_NUM_LC_COLLATE: C2Rust_Unnamed_23 = 196627;
pub const _NL_COLLATE_CODESET: C2Rust_Unnamed_23 = 196626;
pub const _NL_COLLATE_COLLSEQWC: C2Rust_Unnamed_23 = 196625;
pub const _NL_COLLATE_COLLSEQMB: C2Rust_Unnamed_23 = 196624;
pub const _NL_COLLATE_SYMB_EXTRAMB: C2Rust_Unnamed_23 = 196623;
pub const _NL_COLLATE_SYMB_TABLEMB: C2Rust_Unnamed_23 = 196622;
pub const _NL_COLLATE_SYMB_HASH_SIZEMB: C2Rust_Unnamed_23 = 196621;
pub const _NL_COLLATE_INDIRECTWC: C2Rust_Unnamed_23 = 196620;
pub const _NL_COLLATE_EXTRAWC: C2Rust_Unnamed_23 = 196619;
pub const _NL_COLLATE_WEIGHTWC: C2Rust_Unnamed_23 = 196618;
pub const _NL_COLLATE_TABLEWC: C2Rust_Unnamed_23 = 196617;
pub const _NL_COLLATE_GAP3: C2Rust_Unnamed_23 = 196616;
pub const _NL_COLLATE_GAP2: C2Rust_Unnamed_23 = 196615;
pub const _NL_COLLATE_GAP1: C2Rust_Unnamed_23 = 196614;
pub const _NL_COLLATE_INDIRECTMB: C2Rust_Unnamed_23 = 196613;
pub const _NL_COLLATE_EXTRAMB: C2Rust_Unnamed_23 = 196612;
pub const _NL_COLLATE_WEIGHTMB: C2Rust_Unnamed_23 = 196611;
pub const _NL_COLLATE_TABLEMB: C2Rust_Unnamed_23 = 196610;
pub const _NL_COLLATE_RULESETS: C2Rust_Unnamed_23 = 196609;
pub const _NL_COLLATE_NRULES: C2Rust_Unnamed_23 = 196608;
pub const _NL_NUM_LC_TIME: C2Rust_Unnamed_23 = 131231;
pub const _NL_WABALTMON_12: C2Rust_Unnamed_23 = 131230;
pub const _NL_WABALTMON_11: C2Rust_Unnamed_23 = 131229;
pub const _NL_WABALTMON_10: C2Rust_Unnamed_23 = 131228;
pub const _NL_WABALTMON_9: C2Rust_Unnamed_23 = 131227;
pub const _NL_WABALTMON_8: C2Rust_Unnamed_23 = 131226;
pub const _NL_WABALTMON_7: C2Rust_Unnamed_23 = 131225;
pub const _NL_WABALTMON_6: C2Rust_Unnamed_23 = 131224;
pub const _NL_WABALTMON_5: C2Rust_Unnamed_23 = 131223;
pub const _NL_WABALTMON_4: C2Rust_Unnamed_23 = 131222;
pub const _NL_WABALTMON_3: C2Rust_Unnamed_23 = 131221;
pub const _NL_WABALTMON_2: C2Rust_Unnamed_23 = 131220;
pub const _NL_WABALTMON_1: C2Rust_Unnamed_23 = 131219;
pub const _NL_ABALTMON_12: C2Rust_Unnamed_23 = 131218;
pub const _NL_ABALTMON_11: C2Rust_Unnamed_23 = 131217;
pub const _NL_ABALTMON_10: C2Rust_Unnamed_23 = 131216;
pub const _NL_ABALTMON_9: C2Rust_Unnamed_23 = 131215;
pub const _NL_ABALTMON_8: C2Rust_Unnamed_23 = 131214;
pub const _NL_ABALTMON_7: C2Rust_Unnamed_23 = 131213;
pub const _NL_ABALTMON_6: C2Rust_Unnamed_23 = 131212;
pub const _NL_ABALTMON_5: C2Rust_Unnamed_23 = 131211;
pub const _NL_ABALTMON_4: C2Rust_Unnamed_23 = 131210;
pub const _NL_ABALTMON_3: C2Rust_Unnamed_23 = 131209;
pub const _NL_ABALTMON_2: C2Rust_Unnamed_23 = 131208;
pub const _NL_ABALTMON_1: C2Rust_Unnamed_23 = 131207;
pub const _NL_WALTMON_12: C2Rust_Unnamed_23 = 131206;
pub const _NL_WALTMON_11: C2Rust_Unnamed_23 = 131205;
pub const _NL_WALTMON_10: C2Rust_Unnamed_23 = 131204;
pub const _NL_WALTMON_9: C2Rust_Unnamed_23 = 131203;
pub const _NL_WALTMON_8: C2Rust_Unnamed_23 = 131202;
pub const _NL_WALTMON_7: C2Rust_Unnamed_23 = 131201;
pub const _NL_WALTMON_6: C2Rust_Unnamed_23 = 131200;
pub const _NL_WALTMON_5: C2Rust_Unnamed_23 = 131199;
pub const _NL_WALTMON_4: C2Rust_Unnamed_23 = 131198;
pub const _NL_WALTMON_3: C2Rust_Unnamed_23 = 131197;
pub const _NL_WALTMON_2: C2Rust_Unnamed_23 = 131196;
pub const _NL_WALTMON_1: C2Rust_Unnamed_23 = 131195;
pub const __ALTMON_12: C2Rust_Unnamed_23 = 131194;
pub const __ALTMON_11: C2Rust_Unnamed_23 = 131193;
pub const __ALTMON_10: C2Rust_Unnamed_23 = 131192;
pub const __ALTMON_9: C2Rust_Unnamed_23 = 131191;
pub const __ALTMON_8: C2Rust_Unnamed_23 = 131190;
pub const __ALTMON_7: C2Rust_Unnamed_23 = 131189;
pub const __ALTMON_6: C2Rust_Unnamed_23 = 131188;
pub const __ALTMON_5: C2Rust_Unnamed_23 = 131187;
pub const __ALTMON_4: C2Rust_Unnamed_23 = 131186;
pub const __ALTMON_3: C2Rust_Unnamed_23 = 131185;
pub const __ALTMON_2: C2Rust_Unnamed_23 = 131184;
pub const __ALTMON_1: C2Rust_Unnamed_23 = 131183;
pub const _NL_TIME_CODESET: C2Rust_Unnamed_23 = 131182;
pub const _NL_W_DATE_FMT: C2Rust_Unnamed_23 = 131181;
pub const _DATE_FMT: C2Rust_Unnamed_23 = 131180;
pub const _NL_TIME_TIMEZONE: C2Rust_Unnamed_23 = 131179;
pub const _NL_TIME_CAL_DIRECTION: C2Rust_Unnamed_23 = 131178;
pub const _NL_TIME_FIRST_WORKDAY: C2Rust_Unnamed_23 = 131177;
pub const _NL_TIME_FIRST_WEEKDAY: C2Rust_Unnamed_23 = 131176;
pub const _NL_TIME_WEEK_1STWEEK: C2Rust_Unnamed_23 = 131175;
pub const _NL_TIME_WEEK_1STDAY: C2Rust_Unnamed_23 = 131174;
pub const _NL_TIME_WEEK_NDAYS: C2Rust_Unnamed_23 = 131173;
pub const _NL_WERA_T_FMT: C2Rust_Unnamed_23 = 131172;
pub const _NL_WERA_D_T_FMT: C2Rust_Unnamed_23 = 131171;
pub const _NL_WALT_DIGITS: C2Rust_Unnamed_23 = 131170;
pub const _NL_WERA_D_FMT: C2Rust_Unnamed_23 = 131169;
pub const _NL_WERA_YEAR: C2Rust_Unnamed_23 = 131168;
pub const _NL_WT_FMT_AMPM: C2Rust_Unnamed_23 = 131167;
pub const _NL_WT_FMT: C2Rust_Unnamed_23 = 131166;
pub const _NL_WD_FMT: C2Rust_Unnamed_23 = 131165;
pub const _NL_WD_T_FMT: C2Rust_Unnamed_23 = 131164;
pub const _NL_WPM_STR: C2Rust_Unnamed_23 = 131163;
pub const _NL_WAM_STR: C2Rust_Unnamed_23 = 131162;
pub const _NL_WMON_12: C2Rust_Unnamed_23 = 131161;
pub const _NL_WMON_11: C2Rust_Unnamed_23 = 131160;
pub const _NL_WMON_10: C2Rust_Unnamed_23 = 131159;
pub const _NL_WMON_9: C2Rust_Unnamed_23 = 131158;
pub const _NL_WMON_8: C2Rust_Unnamed_23 = 131157;
pub const _NL_WMON_7: C2Rust_Unnamed_23 = 131156;
pub const _NL_WMON_6: C2Rust_Unnamed_23 = 131155;
pub const _NL_WMON_5: C2Rust_Unnamed_23 = 131154;
pub const _NL_WMON_4: C2Rust_Unnamed_23 = 131153;
pub const _NL_WMON_3: C2Rust_Unnamed_23 = 131152;
pub const _NL_WMON_2: C2Rust_Unnamed_23 = 131151;
pub const _NL_WMON_1: C2Rust_Unnamed_23 = 131150;
pub const _NL_WABMON_12: C2Rust_Unnamed_23 = 131149;
pub const _NL_WABMON_11: C2Rust_Unnamed_23 = 131148;
pub const _NL_WABMON_10: C2Rust_Unnamed_23 = 131147;
pub const _NL_WABMON_9: C2Rust_Unnamed_23 = 131146;
pub const _NL_WABMON_8: C2Rust_Unnamed_23 = 131145;
pub const _NL_WABMON_7: C2Rust_Unnamed_23 = 131144;
pub const _NL_WABMON_6: C2Rust_Unnamed_23 = 131143;
pub const _NL_WABMON_5: C2Rust_Unnamed_23 = 131142;
pub const _NL_WABMON_4: C2Rust_Unnamed_23 = 131141;
pub const _NL_WABMON_3: C2Rust_Unnamed_23 = 131140;
pub const _NL_WABMON_2: C2Rust_Unnamed_23 = 131139;
pub const _NL_WABMON_1: C2Rust_Unnamed_23 = 131138;
pub const _NL_WDAY_7: C2Rust_Unnamed_23 = 131137;
pub const _NL_WDAY_6: C2Rust_Unnamed_23 = 131136;
pub const _NL_WDAY_5: C2Rust_Unnamed_23 = 131135;
pub const _NL_WDAY_4: C2Rust_Unnamed_23 = 131134;
pub const _NL_WDAY_3: C2Rust_Unnamed_23 = 131133;
pub const _NL_WDAY_2: C2Rust_Unnamed_23 = 131132;
pub const _NL_WDAY_1: C2Rust_Unnamed_23 = 131131;
pub const _NL_WABDAY_7: C2Rust_Unnamed_23 = 131130;
pub const _NL_WABDAY_6: C2Rust_Unnamed_23 = 131129;
pub const _NL_WABDAY_5: C2Rust_Unnamed_23 = 131128;
pub const _NL_WABDAY_4: C2Rust_Unnamed_23 = 131127;
pub const _NL_WABDAY_3: C2Rust_Unnamed_23 = 131126;
pub const _NL_WABDAY_2: C2Rust_Unnamed_23 = 131125;
pub const _NL_WABDAY_1: C2Rust_Unnamed_23 = 131124;
pub const _NL_TIME_ERA_ENTRIES: C2Rust_Unnamed_23 = 131123;
pub const _NL_TIME_ERA_NUM_ENTRIES: C2Rust_Unnamed_23 = 131122;
pub const ERA_T_FMT: C2Rust_Unnamed_23 = 131121;
pub const ERA_D_T_FMT: C2Rust_Unnamed_23 = 131120;
pub const ALT_DIGITS: C2Rust_Unnamed_23 = 131119;
pub const ERA_D_FMT: C2Rust_Unnamed_23 = 131118;
pub const __ERA_YEAR: C2Rust_Unnamed_23 = 131117;
pub const ERA: C2Rust_Unnamed_23 = 131116;
pub const T_FMT_AMPM: C2Rust_Unnamed_23 = 131115;
pub const T_FMT: C2Rust_Unnamed_23 = 131114;
pub const D_FMT: C2Rust_Unnamed_23 = 131113;
pub const D_T_FMT: C2Rust_Unnamed_23 = 131112;
pub const PM_STR: C2Rust_Unnamed_23 = 131111;
pub const AM_STR: C2Rust_Unnamed_23 = 131110;
pub const MON_12: C2Rust_Unnamed_23 = 131109;
pub const MON_11: C2Rust_Unnamed_23 = 131108;
pub const MON_10: C2Rust_Unnamed_23 = 131107;
pub const MON_9: C2Rust_Unnamed_23 = 131106;
pub const MON_8: C2Rust_Unnamed_23 = 131105;
pub const MON_7: C2Rust_Unnamed_23 = 131104;
pub const MON_6: C2Rust_Unnamed_23 = 131103;
pub const MON_5: C2Rust_Unnamed_23 = 131102;
pub const MON_4: C2Rust_Unnamed_23 = 131101;
pub const MON_3: C2Rust_Unnamed_23 = 131100;
pub const MON_2: C2Rust_Unnamed_23 = 131099;
pub const MON_1: C2Rust_Unnamed_23 = 131098;
pub const ABMON_12: C2Rust_Unnamed_23 = 131097;
pub const ABMON_11: C2Rust_Unnamed_23 = 131096;
pub const ABMON_10: C2Rust_Unnamed_23 = 131095;
pub const ABMON_9: C2Rust_Unnamed_23 = 131094;
pub const ABMON_8: C2Rust_Unnamed_23 = 131093;
pub const ABMON_7: C2Rust_Unnamed_23 = 131092;
pub const ABMON_6: C2Rust_Unnamed_23 = 131091;
pub const ABMON_5: C2Rust_Unnamed_23 = 131090;
pub const ABMON_4: C2Rust_Unnamed_23 = 131089;
pub const ABMON_3: C2Rust_Unnamed_23 = 131088;
pub const ABMON_2: C2Rust_Unnamed_23 = 131087;
pub const ABMON_1: C2Rust_Unnamed_23 = 131086;
pub const DAY_7: C2Rust_Unnamed_23 = 131085;
pub const DAY_6: C2Rust_Unnamed_23 = 131084;
pub const DAY_5: C2Rust_Unnamed_23 = 131083;
pub const DAY_4: C2Rust_Unnamed_23 = 131082;
pub const DAY_3: C2Rust_Unnamed_23 = 131081;
pub const DAY_2: C2Rust_Unnamed_23 = 131080;
pub const DAY_1: C2Rust_Unnamed_23 = 131079;
pub const ABDAY_7: C2Rust_Unnamed_23 = 131078;
pub const ABDAY_6: C2Rust_Unnamed_23 = 131077;
pub const ABDAY_5: C2Rust_Unnamed_23 = 131076;
pub const ABDAY_4: C2Rust_Unnamed_23 = 131075;
pub const ABDAY_3: C2Rust_Unnamed_23 = 131074;
pub const ABDAY_2: C2Rust_Unnamed_23 = 131073;
pub const ABDAY_1: C2Rust_Unnamed_23 = 131072;
static mut corrections: [uint32_t; 7] = [0; 7];
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LC_CTYPE: ::core::ffi::c_int = __LC_CTYPE;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const DBCS_JPN: ::core::ffi::c_int = 932 as ::core::ffi::c_int;
pub const DBCS_JPNU: ::core::ffi::c_int = 9932 as ::core::ffi::c_int;
pub const DBCS_KOR: ::core::ffi::c_int = 949 as ::core::ffi::c_int;
pub const DBCS_KORU: ::core::ffi::c_int = 9949 as ::core::ffi::c_int;
pub const DBCS_CHS: ::core::ffi::c_int = 936 as ::core::ffi::c_int;
pub const DBCS_CHSU: ::core::ffi::c_int = 9936 as ::core::ffi::c_int;
pub const DBCS_CHT: ::core::ffi::c_int = 950 as ::core::ffi::c_int;
pub const DBCS_CHTU: ::core::ffi::c_int = 9950 as ::core::ffi::c_int;
pub const DBCS_DEBUG: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
static mut e_list_item_nr_is_not_list: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E1109: List item %d is not a List\0",
    )
};
static mut e_list_item_nr_does_not_contain_3_numbers: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"E1110: List item %d does not contain 3 numbers\0",
    )
};
static mut e_list_item_nr_range_invalid: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E1111: List item %d range invalid\0",
    )
};
static mut e_list_item_nr_cell_width_invalid: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E1112: List item %d cell width invalid\0",
    )
};
static mut e_overlapping_ranges_for_nr: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E1113: Overlapping ranges for 0x%lx\0",
    )
};
static mut e_only_values_of_0x80_and_higher_supported: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E1114: Only values of 0x80 and higher supported\0",
    )
};
#[no_mangle]
pub static mut utf8len_tab: [uint8_t; 256] = [
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    6 as uint8_t,
    6 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
];
#[no_mangle]
pub static mut utf8len_tab_zero: [uint8_t; 256] = [
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    6 as uint8_t,
    6 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
];
static mut enc_canon_table: [C2Rust_Unnamed_21; 59] = [
    C2Rust_Unnamed_21 {
        name: b"latin1\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_LATIN1 as ::core::ffi::c_int,
        codepage: 1252 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-3\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-6\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-7\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-9\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-10\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-11\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-13\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-14\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-15\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_LATIN9 as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-r\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-u\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_2BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_2BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_2WORD as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_2WORD as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_4BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_4BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_DEBUG,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-jp\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPNU,
    },
    C2Rust_Unnamed_21 {
        name: b"sjis\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-kr\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_KORU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-cn\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHSU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-tw\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHTU,
    },
    C2Rust_Unnamed_21 {
        name: b"big5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp437\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 437 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp737\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 737 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp775\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 775 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp850\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 850 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp852\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 852 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp855\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 855 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp857\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 857 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp860\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 860 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp861\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 861 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp862\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 862 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp863\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 863 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp865\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 865 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp866\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 866 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp869\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 869 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp874\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 874 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp932\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"cp936\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHS,
    },
    C2Rust_Unnamed_21 {
        name: b"cp949\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_KOR,
    },
    C2Rust_Unnamed_21 {
        name: b"cp950\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1250\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1250 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1251\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1251 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1253\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1253 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1254\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1254 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1255\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1255 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1256\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1256 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1257\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1257 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1258\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1258 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"macroman\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_MACROMAN as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"hp-roman8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
];
pub const IDX_LATIN_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IDX_ISO_2: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const IDX_ISO_3: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IDX_ISO_4: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const IDX_ISO_5: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IDX_ISO_6: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const IDX_ISO_7: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const IDX_ISO_8: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const IDX_ISO_9: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const IDX_ISO_10: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const IDX_ISO_11: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const IDX_ISO_13: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const IDX_ISO_14: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const IDX_ISO_15: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const IDX_UTF8: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const IDX_UCS2: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const IDX_UCS2LE: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const IDX_UTF16: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const IDX_UTF16LE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const IDX_UCS4: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const IDX_UCS4LE: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const IDX_EUC_JP: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const IDX_SJIS: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const IDX_EUC_KR: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const IDX_EUC_CN: ::core::ffi::c_int = 27 as ::core::ffi::c_int;
pub const IDX_EUC_TW: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const IDX_BIG5: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const IDX_CP932: ::core::ffi::c_int = 45 as ::core::ffi::c_int;
pub const IDX_CP936: ::core::ffi::c_int = 46 as ::core::ffi::c_int;
pub const IDX_CP949: ::core::ffi::c_int = 47 as ::core::ffi::c_int;
pub const IDX_CP950: ::core::ffi::c_int = 48 as ::core::ffi::c_int;
pub const IDX_MACROMAN: ::core::ffi::c_int = 57 as ::core::ffi::c_int;
pub const IDX_COUNT: ::core::ffi::c_int = 59 as ::core::ffi::c_int;
static mut enc_alias_table: [C2Rust_Unnamed_22; 64] = [
    C2Rust_Unnamed_22 {
        name: b"ansi\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_LATIN_1,
    },
    C2Rust_Unnamed_22 {
        name: b"iso-8859-1\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_LATIN_1,
    },
    C2Rust_Unnamed_22 {
        name: b"latin2\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_2,
    },
    C2Rust_Unnamed_22 {
        name: b"latin3\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_3,
    },
    C2Rust_Unnamed_22 {
        name: b"latin4\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_4,
    },
    C2Rust_Unnamed_22 {
        name: b"cyrillic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_5,
    },
    C2Rust_Unnamed_22 {
        name: b"arabic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_6,
    },
    C2Rust_Unnamed_22 {
        name: b"greek\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_7,
    },
    C2Rust_Unnamed_22 {
        name: b"hebrew\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_8,
    },
    C2Rust_Unnamed_22 {
        name: b"latin5\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_9,
    },
    C2Rust_Unnamed_22 {
        name: b"turkish\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_9,
    },
    C2Rust_Unnamed_22 {
        name: b"latin6\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_10,
    },
    C2Rust_Unnamed_22 {
        name: b"nordic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_10,
    },
    C2Rust_Unnamed_22 {
        name: b"thai\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_11,
    },
    C2Rust_Unnamed_22 {
        name: b"latin7\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_13,
    },
    C2Rust_Unnamed_22 {
        name: b"latin8\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_14,
    },
    C2Rust_Unnamed_22 {
        name: b"latin9\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_15,
    },
    C2Rust_Unnamed_22 {
        name: b"utf8\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF8,
    },
    C2Rust_Unnamed_22 {
        name: b"unicode\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs-2be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-16be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16LE,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs-4be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"932\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP932,
    },
    C2Rust_Unnamed_22 {
        name: b"949\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP949,
    },
    C2Rust_Unnamed_22 {
        name: b"936\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP936,
    },
    C2Rust_Unnamed_22 {
        name: b"gbk\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP936,
    },
    C2Rust_Unnamed_22 {
        name: b"950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP950,
    },
    C2Rust_Unnamed_22 {
        name: b"eucjp\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"unix-jis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"ujis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"shift-jis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_SJIS,
    },
    C2Rust_Unnamed_22 {
        name: b"pck\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_SJIS,
    },
    C2Rust_Unnamed_22 {
        name: b"euckr\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"5601\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"euccn\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"gb2312\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"euctw\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"japan\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"korea\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"prc\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"zh-cn\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"chinese\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"zh-tw\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"taiwan\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"cp950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_BIG5,
    },
    C2Rust_Unnamed_22 {
        name: b"950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_BIG5,
    },
    C2Rust_Unnamed_22 {
        name: b"mac\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_MACROMAN,
    },
    C2Rust_Unnamed_22 {
        name: b"mac-roman\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_MACROMAN,
    },
    C2Rust_Unnamed_22 {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        canon: 0 as ::core::ffi::c_int,
    },
];
unsafe extern "C" fn enc_canon_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < IDX_COUNT {
        if strcmp(name, enc_canon_table[i as usize].name) == 0 as ::core::ffi::c_int {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn enc_canon_props(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = enc_canon_search(name);
    if i >= 0 as ::core::ffi::c_int {
        return enc_canon_table[i as usize].prop;
    } else if strncmp(
        name,
        b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return ENC_DBCS as ::core::ffi::c_int;
    } else if strncmp(
        name,
        b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"iso-8859-\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        return ENC_8BIT as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bomb_size() -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*curbuf).b_p_bomb != 0 && (*curbuf).b_p_bin == 0 {
        if *(*curbuf).b_p_fenc as ::core::ffi::c_int == NUL
            || strcmp(
                (*curbuf).b_p_fenc,
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            n = 3 as ::core::ffi::c_int;
        } else if strncmp(
            (*curbuf).b_p_fenc,
            b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                (*curbuf).b_p_fenc,
                b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            n = 2 as ::core::ffi::c_int;
        } else if strncmp(
            (*curbuf).b_p_fenc,
            b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            n = 4 as ::core::ffi::c_int;
        }
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn remove_bom(mut s: *mut ::core::ffi::c_char) {
    let mut p: *mut ::core::ffi::c_char = s;
    loop {
        p = strchr(p, 0xef as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == 0xbb as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == 0xbf as ::core::ffi::c_int
        {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        } else {
            p = p.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn mb_get_class(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return mb_get_class_tab(p, &raw mut (*curbuf).b_chartab as *mut uint64_t);
}
#[no_mangle]
pub unsafe extern "C" fn mb_get_class_tab(
    mut p: *const ::core::ffi::c_char,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    if utf8len_tab[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || ascii_iswhite(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        if vim_iswordc_tab(
            *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            chartab,
        ) {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    return utf_class_tab(utf_ptr2char(p), chartab);
}
unsafe extern "C" fn prop_is_emojilike(mut prop: *const utf8proc_property_t) -> bool {
    return (*prop).boundclass() as ::core::ffi::c_int
        == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
        || (*prop).boundclass() as ::core::ffi::c_int
            == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_char2cells(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if !vim_isprintc(c) {
        '_c2rust_label: {
            if c <= 0xffff as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"c <= 0xFFFF\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    462 as ::core::ffi::c_uint,
                    b"int utf_char2cells(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return if c > 0xff as ::core::ffi::c_int {
            6 as ::core::ffi::c_int
        } else {
            4 as ::core::ffi::c_int
        };
    }
    let mut n: ::core::ffi::c_int = cw_value(c);
    if n != 0 as ::core::ffi::c_int {
        return n;
    }
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    if (*prop).charwidth() as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    }
    if *p_ambw as ::core::ffi::c_int == 'd' as ::core::ffi::c_int
        && (*prop).ambiguous_width() as ::core::ffi::c_int != 0
    {
        return 2 as ::core::ffi::c_int;
    }
    if p_emoji != 0
        && c >= 0x1f000 as ::core::ffi::c_int
        && (*prop).ambiguous_width() == 0
        && prop_is_emojilike(prop) as ::core::ffi::c_int != 0
    {
        return 2 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2cells(mut p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const uint8_t = p_in as *const uint8_t;
    if *p as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        let mut len: ::core::ffi::c_int = utf8len_tab[*p as usize] as ::core::ffi::c_int;
        let mut c: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if c <= 0 as int32_t {
            return 4 as ::core::ffi::c_int;
        }
        if c < 0x80 as int32_t {
            return char2cells(c as ::core::ffi::c_int);
        }
        let mut cells: ::core::ffi::c_int = utf_char2cells(c as ::core::ffi::c_int);
        if cells == 1 as ::core::ffi::c_int
            && p_emoji != 0
            && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t)) as ::core::ffi::c_int
                != 0
        {
            let mut c2: ::core::ffi::c_int = utf_ptr2char(p_in.offset(len as isize));
            if c2 == 0xfe0f as ::core::ffi::c_int {
                return 2 as ::core::ffi::c_int;
            }
        }
        return cells;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2CharInfo_impl(mut p: *const uint8_t, len: uintptr_t) -> int32_t {
    let corr: uint32_t = corrections[len as usize];
    let mut cur: uint8_t = 0;
    cur = *p.offset(1 as ::core::ffi::c_int as isize);
    let mut code_point: uint32_t = ((*p.offset(0 as ::core::ffi::c_int as isize) as uint32_t)
        << 6 as ::core::ffi::c_int)
        .wrapping_add(cur as uint32_t);
    if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
        as ::core::ffi::c_uint
        != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return -1 as int32_t;
    }
    if (len as uint32_t) >= 3 as uint32_t {
        cur = *p.offset(2 as ::core::ffi::c_int as isize);
        code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
        if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
            as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return -1 as int32_t;
        }
        if len as uint32_t != 3 as uint32_t {
            cur = *p.offset(3 as ::core::ffi::c_int as isize);
            code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
            if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                as ::core::ffi::c_uint
                != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                as ::core::ffi::c_long
                != 0
            {
                return -1 as int32_t;
            }
            if len as uint32_t != 4 as uint32_t {
                cur = *p.offset(4 as ::core::ffi::c_int as isize);
                code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                    as ::core::ffi::c_uint
                    != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                    as ::core::ffi::c_long
                    != 0
                {
                    return -1 as int32_t;
                }
                if len as uint32_t != 5 as uint32_t {
                    cur = *p.offset(5 as ::core::ffi::c_int as isize);
                    code_point =
                        (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                    if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                        as ::core::ffi::c_uint
                        != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        as ::core::ffi::c_long
                        != 0
                    {
                        return -1 as int32_t;
                    }
                }
            }
        }
    }
    return code_point.wrapping_add(corr) as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2cells_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if size > 0 as ::core::ffi::c_int
        && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
    {
        let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
        if len < utf8len_tab[*p as uint8_t as usize] as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if utf_ptr2len(p) == 1 as ::core::ffi::c_int || c == NUL {
            return 4 as ::core::ffi::c_int;
        }
        if c < 0x80 as ::core::ffi::c_int {
            return char2cells(c);
        }
        let mut cells: ::core::ffi::c_int = utf_char2cells(c);
        if cells == 1 as ::core::ffi::c_int
            && p_emoji != 0
            && size > len
            && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t)) as ::core::ffi::c_int
                != 0
            && utf_ptr2len_len(p.offset(len as isize), size - len)
                == utf8len_tab[*p.offset(len as isize) as uint8_t as usize] as ::core::ffi::c_int
        {
            let mut c2: ::core::ffi::c_int = utf_ptr2char(p.offset(len as isize));
            if c2 == 0xfe0f as ::core::ffi::c_int {
                return 2 as ::core::ffi::c_int;
            }
        }
        return cells;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mb_string2cells(mut str: *const ::core::ffi::c_char) -> size_t {
    let mut clen: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        clen = clen.wrapping_add(utf_ptr2cells(p) as size_t);
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return clen;
}
#[no_mangle]
pub unsafe extern "C" fn mb_string2cells_len(
    mut str: *const ::core::ffi::c_char,
    mut size: size_t,
) -> size_t {
    let mut clen: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL && p < str.offset(size as isize) {
        clen = clen.wrapping_add(utf_ptr2cells_len(
            p,
            size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
        ) as size_t);
        p = p.offset(utfc_ptr2len_len(
            p,
            size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
        ) as isize);
    }
    return clen;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    let v0: uint32_t = *p.offset(0 as ::core::ffi::c_int as isize) as uint32_t;
    if (v0 < 0x80 as uint32_t) as ::core::ffi::c_int as ::core::ffi::c_long != 0 {
        return v0 as ::core::ffi::c_int;
    }
    let len: uint8_t = utf8len_tab[v0 as usize];
    if ((len as ::core::ffi::c_int) < 2 as ::core::ffi::c_int) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    let v1: uint32_t = *p.offset(1 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v1 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        return (v0 << 6 as ::core::ffi::c_int)
            .wrapping_add(v1)
            .wrapping_sub(
                ((0xc0 as uint32_t) << 6 as ::core::ffi::c_int).wrapping_add(
                    (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                ),
            ) as ::core::ffi::c_int;
    }
    let v2: uint32_t = *p.offset(2 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v2 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
        return (v0 << 12 as ::core::ffi::c_int)
            .wrapping_add(v1 << 6 as ::core::ffi::c_int)
            .wrapping_add(v2)
            .wrapping_sub(
                ((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v3: uint32_t = *p.offset(3 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v3 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
        return (v0 << 18 as ::core::ffi::c_int)
            .wrapping_add(v1 << 12 as ::core::ffi::c_int)
            .wrapping_add(v2 << 6 as ::core::ffi::c_int)
            .wrapping_add(v3)
            .wrapping_sub(
                ((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v4: uint32_t = *p.offset(4 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v4 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
        return (v0 << 24 as ::core::ffi::c_int)
            .wrapping_add(v1 << 18 as ::core::ffi::c_int)
            .wrapping_add(v2 << 12 as ::core::ffi::c_int)
            .wrapping_add(v3 << 6 as ::core::ffi::c_int)
            .wrapping_add(v4)
            .wrapping_sub(
                ((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v5: uint32_t = *p.offset(5 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v5 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    return (v0 << 30 as ::core::ffi::c_int)
        .wrapping_add(v1 << 24 as ::core::ffi::c_int)
        .wrapping_add(v2 << 18 as ::core::ffi::c_int)
        .wrapping_add(v3 << 12 as ::core::ffi::c_int)
        .wrapping_add(v4 << 6 as ::core::ffi::c_int)
        .wrapping_add(v5)
        .wrapping_sub(
            ((0x80 as ::core::ffi::c_uint as uint32_t) << 24 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int),
        ) as ::core::ffi::c_int;
}
unsafe extern "C" fn utf_safe_read_char_adv(
    mut s: *mut *const ::core::ffi::c_char,
    mut n: *mut size_t,
) -> ::core::ffi::c_int {
    if *n == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    let mut k: uint8_t = utf8len_tab_zero[**s as uint8_t as usize];
    if k as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *n = (*n).wrapping_sub(1);
        let c2rust_fresh0 = *s;
        *s = (*s).offset(1);
        return *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
    }
    if k as size_t <= *n {
        let mut c: ::core::ffi::c_int = utf_ptr2char(*s);
        if c != **s as uint8_t as ::core::ffi::c_int
            || c == 0xc3 as ::core::ffi::c_int
                && *(*s).offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    == 0x83 as ::core::ffi::c_int
        {
            *s = (*s).offset(k as ::core::ffi::c_int as isize);
            *n = (*n).wrapping_sub(k as size_t);
            return c;
        }
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mb_ptr2char_adv(
    pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
    *pp = (*pp).offset(utfc_ptr2len(*pp) as isize);
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn mb_cptr2char_adv(
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
    *pp = (*pp).offset(utf_ptr2len(*pp) as isize);
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn utf_iscomposing_first(mut c: ::core::ffi::c_int) -> bool {
    return c >= 128 as ::core::ffi::c_int
        && !utf8proc_grapheme_break(' ' as utf8proc_int32_t, c as utf8proc_int32_t);
}
#[no_mangle]
pub unsafe extern "C" fn utf_composinglike(
    mut p1: *const ::core::ffi::c_char,
    mut p2: *const ::core::ffi::c_char,
    mut state: *mut GraphemeState,
) -> bool {
    if (*p2 as uint8_t as ::core::ffi::c_int) < 128 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut first: ::core::ffi::c_int = utf_ptr2char(p1);
    let mut second: ::core::ffi::c_int = utf_ptr2char(p2);
    if !utf8proc_grapheme_break_stateful(
        first as utf8proc_int32_t,
        second as utf8proc_int32_t,
        state as *mut utf8proc_int32_t,
    ) {
        return true_0 != 0;
    }
    return arabic_combine(first, second);
}
#[no_mangle]
pub unsafe extern "C" fn utf_iscomposing(
    mut c1: ::core::ffi::c_int,
    mut c2: ::core::ffi::c_int,
    mut state: *mut GraphemeState,
) -> bool {
    return !utf8proc_grapheme_break_stateful(
        c1 as utf8proc_int32_t,
        c2 as utf8proc_int32_t,
        state as *mut utf8proc_int32_t,
    ) || arabic_combine(c1, c2) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn utfc_ptr2schar(
    mut p: *const ::core::ffi::c_char,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    *firstc = c;
    let mut first_compose: bool = utf_iscomposing_first(c);
    let mut maxlen: size_t =
        (MAX_SCHAR_SIZE - 1 as ::core::ffi::c_int - first_compose as ::core::ffi::c_int) as size_t;
    let mut len: size_t = utfc_ptr2len_len(p, maxlen as ::core::ffi::c_int) as size_t;
    if len == 1 as size_t && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return 0 as schar_T;
    }
    return schar_from_buf_first(p, len, first_compose);
}
#[no_mangle]
pub unsafe extern "C" fn utfc_ptrlen2schar(
    mut p: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    if len == 1 as ::core::ffi::c_int
        && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        || len == 0 as ::core::ffi::c_int
    {
        *firstc = *p as uint8_t as ::core::ffi::c_int;
        return 0 as schar_T;
    }
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    *firstc = c;
    let mut first_compose: bool = utf_iscomposing_first(c);
    let mut maxlen: ::core::ffi::c_int =
        MAX_SCHAR_SIZE - 1 as ::core::ffi::c_int - first_compose as ::core::ffi::c_int;
    if len > maxlen {
        len = utfc_ptr2len_len(p, maxlen);
    }
    return schar_from_buf_first(p, len as size_t, first_compose);
}
unsafe extern "C" fn schar_from_buf_first(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut first_compose: bool,
) -> schar_T {
    if first_compose {
        let mut cbuf: [::core::ffi::c_char; 32] = [0; 32];
        cbuf[0 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
        memcpy(
            (&raw mut cbuf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            len,
        );
        return schar_from_buf(
            &raw mut cbuf as *mut ::core::ffi::c_char,
            len.wrapping_add(1 as size_t),
        );
    } else {
        return schar_from_buf(buf, len);
    };
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    if *p as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    let len: ::core::ffi::c_int = utf8len_tab[*p as usize] as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < len {
        if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
            != 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn utf_byte2len(mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return utf8len_tab[b as usize] as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut m: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = utf8len_tab[*p as uint8_t as usize] as ::core::ffi::c_int;
    if len == 1 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if len > size {
        m = size;
    } else {
        m = len;
    }
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < m {
        if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
            != 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut b0: uint8_t = *p as uint8_t;
    if b0 as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (b0 as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = utf_ptr2len(p);
    if len == 1 as ::core::ffi::c_int && b0 as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    loop {
        if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
            || !utf_composinglike(
                p.offset(prevlen as isize),
                p.offset(len as isize),
                &raw mut state,
            )
        {
            return len;
        }
        prevlen = len;
        len += utf_ptr2len(p.offset(len as isize));
    }
}
#[no_mangle]
pub unsafe extern "C" fn utfc_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if size < 1 as ::core::ffi::c_int || *p as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
        < 0x80 as ::core::ffi::c_int
        && (size == 1 as ::core::ffi::c_int
            || (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int)
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
    if len == 1 as ::core::ffi::c_int
        && *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            >= 0x80 as ::core::ffi::c_int
        || len > size
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    while len < size {
        if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            break;
        }
        let mut len_next_char: ::core::ffi::c_int =
            utf_ptr2len_len(p.offset(len as isize), size - len);
        if len_next_char > size - len {
            break;
        }
        if !utf_composinglike(
            p.offset(prevlen as isize),
            p.offset(len as isize),
            &raw mut state,
        ) {
            break;
        }
        prevlen = len;
        len += len_next_char;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    } else if c < 0x800 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    } else if c < 0x10000 as ::core::ffi::c_int {
        return 3 as ::core::ffi::c_int;
    } else if c < 0x200000 as ::core::ffi::c_int {
        return 4 as ::core::ffi::c_int;
    } else if c < 0x4000000 as ::core::ffi::c_int {
        return 5 as ::core::ffi::c_int;
    } else {
        return 6 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn utf_char2bytes(
    c: ::core::ffi::c_int,
    buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = c as ::core::ffi::c_char;
        return 1 as ::core::ffi::c_int;
    } else if c < 0x800 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xc0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 2 as ::core::ffi::c_int;
    } else if c < 0x10000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xe0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 3 as ::core::ffi::c_int;
    } else if c < 0x200000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 4 as ::core::ffi::c_int;
    } else if c < 0x4000000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf8 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(4 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 5 as ::core::ffi::c_int;
    } else {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xfc as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 30 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(4 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(5 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 6 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn utf_iscomposing_legacy(mut c: ::core::ffi::c_int) -> bool {
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    return (*prop).category as ::core::ffi::c_int == UTF8PROC_CATEGORY_MN as ::core::ffi::c_int
        || (*prop).category as ::core::ffi::c_int == UTF8PROC_CATEGORY_ME as ::core::ffi::c_int;
}
unsafe extern "C" fn intable(
    mut table: *const interval,
    mut n_items: size_t,
    mut c: ::core::ffi::c_int,
) -> bool {
    '_c2rust_label: {
        if n_items > 0 as size_t {
        } else {
            __assert_fail(
                b"n_items > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1177 as ::core::ffi::c_uint,
                b"_Bool intable(const struct interval *, size_t, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if c < (*table.offset(0 as ::core::ffi::c_int as isize)).first {
        return false_0 != 0;
    }
    '_c2rust_label_0: {
        if n_items <= (18446744073709551615 as size_t).wrapping_div(2 as size_t) {
        } else {
            __assert_fail(
                b"n_items <= SIZE_MAX / 2\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1183 as ::core::ffi::c_uint,
                b"_Bool intable(const struct interval *, size_t, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut bot: size_t = 0 as size_t;
    let mut top: size_t = n_items;
    loop {
        let mut mid: size_t = bot.wrapping_add(top) >> 1 as ::core::ffi::c_int;
        if (*table.offset(mid as isize)).last < c {
            bot = mid.wrapping_add(1 as size_t);
        } else if (*table.offset(mid as isize)).first > c {
            top = mid;
        } else {
            return true_0 != 0;
        }
        if top <= bot {
            break;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn utf_printable(mut c: ::core::ffi::c_int) -> bool {
    static mut nonprint: [interval; 9] = [
        interval {
            first: 0x70f as ::core::ffi::c_int,
            last: 0x70f as ::core::ffi::c_int,
        },
        interval {
            first: 0x180b as ::core::ffi::c_int,
            last: 0x180e as ::core::ffi::c_int,
        },
        interval {
            first: 0x200b as ::core::ffi::c_int,
            last: 0x200f as ::core::ffi::c_int,
        },
        interval {
            first: 0x202a as ::core::ffi::c_int,
            last: 0x202e as ::core::ffi::c_int,
        },
        interval {
            first: 0x2060 as ::core::ffi::c_int,
            last: 0x206f as ::core::ffi::c_int,
        },
        interval {
            first: 0xd800 as ::core::ffi::c_int,
            last: 0xdfff as ::core::ffi::c_int,
        },
        interval {
            first: 0xfeff as ::core::ffi::c_int,
            last: 0xfeff as ::core::ffi::c_int,
        },
        interval {
            first: 0xfff9 as ::core::ffi::c_int,
            last: 0xfffb as ::core::ffi::c_int,
        },
        interval {
            first: 0xfffe as ::core::ffi::c_int,
            last: 0xffff as ::core::ffi::c_int,
        },
    ];
    return !intable(
        &raw const nonprint as *const interval,
        ::core::mem::size_of::<[interval; 9]>()
            .wrapping_div(::core::mem::size_of::<interval>())
            .wrapping_div(
                (::core::mem::size_of::<[interval; 9]>()
                    .wrapping_rem(::core::mem::size_of::<interval>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        c,
    );
}
#[no_mangle]
pub unsafe extern "C" fn utf_class(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return utf_class_tab(c, &raw mut (*curbuf).b_chartab as *mut uint64_t);
}
#[no_mangle]
pub unsafe extern "C" fn utf_class_tab(
    c: ::core::ffi::c_int,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    static mut classes: [clinterval; 71] = [
        clinterval {
            first: 0x37e as ::core::ffi::c_uint,
            last: 0x37e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x387 as ::core::ffi::c_uint,
            last: 0x387 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x55a as ::core::ffi::c_uint,
            last: 0x55f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x589 as ::core::ffi::c_uint,
            last: 0x589 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5be as ::core::ffi::c_uint,
            last: 0x5be as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5c0 as ::core::ffi::c_uint,
            last: 0x5c0 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5c3 as ::core::ffi::c_uint,
            last: 0x5c3 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5f3 as ::core::ffi::c_uint,
            last: 0x5f4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x60c as ::core::ffi::c_uint,
            last: 0x60c as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x61b as ::core::ffi::c_uint,
            last: 0x61b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x61f as ::core::ffi::c_uint,
            last: 0x61f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x66a as ::core::ffi::c_uint,
            last: 0x66d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x6d4 as ::core::ffi::c_uint,
            last: 0x6d4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x700 as ::core::ffi::c_uint,
            last: 0x70d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x964 as ::core::ffi::c_uint,
            last: 0x965 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x970 as ::core::ffi::c_uint,
            last: 0x970 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xdf4 as ::core::ffi::c_uint,
            last: 0xdf4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xe4f as ::core::ffi::c_uint,
            last: 0xe4f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xe5a as ::core::ffi::c_uint,
            last: 0xe5b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf04 as ::core::ffi::c_uint,
            last: 0xf12 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf3a as ::core::ffi::c_uint,
            last: 0xf3d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf85 as ::core::ffi::c_uint,
            last: 0xf85 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x104a as ::core::ffi::c_uint,
            last: 0x104f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x10fb as ::core::ffi::c_uint,
            last: 0x10fb as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1361 as ::core::ffi::c_uint,
            last: 0x1368 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x166d as ::core::ffi::c_uint,
            last: 0x166e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1680 as ::core::ffi::c_uint,
            last: 0x1680 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x169b as ::core::ffi::c_uint,
            last: 0x169c as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x16eb as ::core::ffi::c_uint,
            last: 0x16ed as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1735 as ::core::ffi::c_uint,
            last: 0x1736 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x17d4 as ::core::ffi::c_uint,
            last: 0x17dc as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1800 as ::core::ffi::c_uint,
            last: 0x180a as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2000 as ::core::ffi::c_uint,
            last: 0x200b as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x200c as ::core::ffi::c_uint,
            last: 0x2027 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2028 as ::core::ffi::c_uint,
            last: 0x2029 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x202a as ::core::ffi::c_uint,
            last: 0x202e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x202f as ::core::ffi::c_uint,
            last: 0x202f as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2030 as ::core::ffi::c_uint,
            last: 0x205e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x205f as ::core::ffi::c_uint,
            last: 0x205f as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2060 as ::core::ffi::c_uint,
            last: 0x206f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2070 as ::core::ffi::c_uint,
            last: 0x207f as ::core::ffi::c_uint,
            cls: 0x2070 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2080 as ::core::ffi::c_uint,
            last: 0x2094 as ::core::ffi::c_uint,
            cls: 0x2080 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x20a0 as ::core::ffi::c_uint,
            last: 0x27ff as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2800 as ::core::ffi::c_uint,
            last: 0x28ff as ::core::ffi::c_uint,
            cls: 0x2800 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2900 as ::core::ffi::c_uint,
            last: 0x2998 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x29d8 as ::core::ffi::c_uint,
            last: 0x29db as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x29fc as ::core::ffi::c_uint,
            last: 0x29fd as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2e00 as ::core::ffi::c_uint,
            last: 0x2e7f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3000 as ::core::ffi::c_uint,
            last: 0x3000 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3001 as ::core::ffi::c_uint,
            last: 0x3020 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3030 as ::core::ffi::c_uint,
            last: 0x3030 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x303d as ::core::ffi::c_uint,
            last: 0x303d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3040 as ::core::ffi::c_uint,
            last: 0x309f as ::core::ffi::c_uint,
            cls: 0x3040 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x30a0 as ::core::ffi::c_uint,
            last: 0x30ff as ::core::ffi::c_uint,
            cls: 0x30a0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3300 as ::core::ffi::c_uint,
            last: 0x9fff as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xac00 as ::core::ffi::c_uint,
            last: 0xd7a3 as ::core::ffi::c_uint,
            cls: 0xac00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf900 as ::core::ffi::c_uint,
            last: 0xfaff as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xfd3e as ::core::ffi::c_uint,
            last: 0xfd3f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xfe30 as ::core::ffi::c_uint,
            last: 0xfe6b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff00 as ::core::ffi::c_uint,
            last: 0xff0f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff1a as ::core::ffi::c_uint,
            last: 0xff20 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff3b as ::core::ffi::c_uint,
            last: 0xff40 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff5b as ::core::ffi::c_uint,
            last: 0xff65 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1d000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1d24f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1d400 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1d7ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1f000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1f2ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1f300 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1f9ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x20000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2a6df as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2a700 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2b73f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2b740 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2b81f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2f800 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2fa1f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
    ];
    let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut top: ::core::ffi::c_int = ::core::mem::size_of::<[clinterval; 71]>()
        .wrapping_div(::core::mem::size_of::<clinterval>())
        .wrapping_div(
            (::core::mem::size_of::<[clinterval; 71]>()
                .wrapping_rem(::core::mem::size_of::<clinterval>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    if c < 0x100 as ::core::ffi::c_int {
        if c == ' ' as ::core::ffi::c_int
            || c == '\t' as ::core::ffi::c_int
            || c == NUL
            || c == 0xa0 as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if vim_iswordc_tab(c, chartab) {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    if prop_is_emojilike(prop) {
        return 3 as ::core::ffi::c_int;
    }
    while top >= bot {
        let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
        if classes[mid as usize].last < c as ::core::ffi::c_uint {
            bot = mid + 1 as ::core::ffi::c_int;
        } else if classes[mid as usize].first > c as ::core::ffi::c_uint {
            top = mid - 1 as ::core::ffi::c_int;
        } else {
            return classes[mid as usize].cls as ::core::ffi::c_int;
        }
    }
    return 2 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_ambiguous_width(mut p: *const ::core::ffi::c_char) -> bool {
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return false_0 != 0;
    }
    let mut info: CharInfo = utf_ptr2CharInfo(p);
    if info.value >= 0x80 as int32_t {
        let mut prop: *const utf8proc_property_t =
            utf8proc_get_property(info.value as utf8proc_int32_t);
        if (*prop).ambiguous_width() as ::core::ffi::c_int != 0
            || prop_is_emojilike(prop) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
    }
    return memcmp(
        p.offset(info.len as isize) as *const ::core::ffi::c_void,
        b"\xEF\xB8\x8F\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_fold(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 0x80 as ::core::ffi::c_int {
        return if a >= 0x41 as ::core::ffi::c_int && a <= 0x5a as ::core::ffi::c_int {
            a + 32 as ::core::ffi::c_int
        } else {
            a
        };
    }
    if a == 0xdf as ::core::ffi::c_int || a == 0x130 as ::core::ffi::c_int {
        return a;
    }
    let mut result: [utf8proc_int32_t; 1] = [0; 1];
    let mut res: utf8proc_ssize_t = utf8proc_decompose_char(
        a as utf8proc_int32_t,
        &raw mut result as *mut utf8proc_int32_t,
        1 as utf8proc_ssize_t,
        UTF8PROC_CASEFOLD,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    return if res == 1 as utf8proc_ssize_t {
        result[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    } else {
        a
    };
}
#[no_mangle]
pub unsafe extern "C" fn mb_toupper(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 128 as ::core::ffi::c_int
        && cmp_flags & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        return if a < 'a' as ::core::ffi::c_int || a > 'z' as ::core::ffi::c_int {
            a
        } else {
            a - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
    }
    if cmp_flags & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
        return towupper(a as wint_t) as ::core::ffi::c_int;
    }
    if a < 128 as ::core::ffi::c_int {
        return toupper(a);
    }
    return utf8proc_toupper(a as utf8proc_int32_t) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mb_islower(mut a: ::core::ffi::c_int) -> bool {
    return mb_toupper(a) != a;
}
#[no_mangle]
pub unsafe extern "C" fn mb_tolower(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 128 as ::core::ffi::c_int
        && cmp_flags & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        return if a < 'A' as ::core::ffi::c_int || a > 'Z' as ::core::ffi::c_int {
            a
        } else {
            a + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
    }
    if cmp_flags & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
        return towlower(a as wint_t) as ::core::ffi::c_int;
    }
    if a < 128 as ::core::ffi::c_int {
        return tolower(a);
    }
    return utf8proc_tolower(a as utf8proc_int32_t) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mb_isupper(mut a: ::core::ffi::c_int) -> bool {
    return mb_tolower(a) != a;
}
#[no_mangle]
pub unsafe extern "C" fn mb_isalpha(mut a: ::core::ffi::c_int) -> bool {
    return mb_islower(a) as ::core::ffi::c_int != 0 || mb_isupper(a) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn utf_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    mut n1: size_t,
    mut n2: size_t,
) -> ::core::ffi::c_int {
    let mut c1: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    let mut buffer: [::core::ffi::c_char; 6] = [0; 6];
    loop {
        c1 = utf_safe_read_char_adv(&raw mut s1, &raw mut n1);
        c2 = utf_safe_read_char_adv(&raw mut s2, &raw mut n2);
        if c1 <= 0 as ::core::ffi::c_int || c2 <= 0 as ::core::ffi::c_int {
            break;
        }
        if c1 == c2 {
            continue;
        }
        let mut cdiff: ::core::ffi::c_int = utf_fold(c1) - utf_fold(c2);
        if cdiff != 0 as ::core::ffi::c_int {
            return cdiff;
        }
    }
    if c1 == 0 as ::core::ffi::c_int || c2 == 0 as ::core::ffi::c_int {
        if c1 == 0 as ::core::ffi::c_int && c2 == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        return if c1 == 0 as ::core::ffi::c_int {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
    if c1 != -1 as ::core::ffi::c_int && c2 == -1 as ::core::ffi::c_int {
        n1 = utf_char2bytes(utf_fold(c1), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
        s1 = &raw mut buffer as *mut ::core::ffi::c_char;
    } else if c2 != -1 as ::core::ffi::c_int && c1 == -1 as ::core::ffi::c_int {
        n2 = utf_char2bytes(utf_fold(c2), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
        s2 = &raw mut buffer as *mut ::core::ffi::c_char;
    }
    while n1 > 0 as size_t
        && n2 > 0 as size_t
        && *s1 as ::core::ffi::c_int != NUL
        && *s2 as ::core::ffi::c_int != NUL
    {
        let mut cdiff_0: ::core::ffi::c_int =
            *s1 as uint8_t as ::core::ffi::c_int - *s2 as uint8_t as ::core::ffi::c_int;
        if cdiff_0 != 0 as ::core::ffi::c_int {
            return cdiff_0;
        }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        n1 = n1.wrapping_sub(1);
        n2 = n2.wrapping_sub(1);
    }
    if n1 > 0 as size_t && *s1 as ::core::ffi::c_int == NUL {
        n1 = 0 as size_t;
    }
    if n2 > 0 as size_t && *s2 as ::core::ffi::c_int == NUL {
        n2 = 0 as size_t;
    }
    if n1 == 0 as size_t && n2 == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    return if n1 == 0 as size_t {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn mb_utflen(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut codepoints: *mut size_t,
    mut codeunits: *mut size_t,
) {
    let mut count: size_t = 0 as size_t;
    let mut extra: size_t = 0 as size_t;
    let mut clen: size_t = 0;
    let mut i: size_t = 0 as size_t;
    while i < len {
        clen = utf_ptr2len_len(
            s.offset(i as isize),
            len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
            utf_ptr2char(s.offset(i as isize))
        } else {
            *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
        };
        count = count.wrapping_add(1);
        if c > 0xffff as ::core::ffi::c_int {
            extra = extra.wrapping_add(1);
        }
        i = i.wrapping_add(clen);
    }
    *codepoints = (*codepoints).wrapping_add(count);
    *codeunits = (*codeunits).wrapping_add(count.wrapping_add(extra));
}
#[no_mangle]
pub unsafe extern "C" fn mb_utf_index_to_bytes(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut index: size_t,
    mut use_utf16_units: bool,
) -> ssize_t {
    let mut count: size_t = 0 as size_t;
    let mut clen: size_t = 0;
    if index == 0 as size_t {
        return 0 as ssize_t;
    }
    let mut i: size_t = 0 as size_t;
    while i < len {
        clen = utf_ptr2len_len(
            s.offset(i as isize),
            len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
            utf_ptr2char(s.offset(i as isize))
        } else {
            *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
        };
        count = count.wrapping_add(1);
        if use_utf16_units as ::core::ffi::c_int != 0 && c > 0xffff as ::core::ffi::c_int {
            count = count.wrapping_add(1);
        }
        if count >= index {
            return i.wrapping_add(clen) as ssize_t;
        }
        i = i.wrapping_add(clen);
    }
    return -1 as ssize_t;
}
#[no_mangle]
pub unsafe extern "C" fn mb_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    nn: size_t,
) -> ::core::ffi::c_int {
    return utf_strnicmp(s1, s2, nn, nn);
}
#[no_mangle]
pub unsafe extern "C" fn mb_stricmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return mb_strnicmp(s1, s2, MAXCOL as ::core::ffi::c_int as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn show_utf8() {
    let mut line: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
    let mut len: ::core::ffi::c_int = utfc_ptr2len(line);
    if len == 0 as ::core::ffi::c_int {
        msg(
            b"NUL\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut rlen: size_t = 0 as size_t;
    let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < len {
        if clen == 0 as ::core::ffi::c_int {
            if i > 0 as ::core::ffi::c_int {
                strcpy(
                    (&raw mut IObuff as *mut ::core::ffi::c_char).offset(rlen as isize),
                    b"+ \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                rlen = rlen.wrapping_add(2 as size_t);
            }
            clen = utf_ptr2len(line.offset(i as isize));
        }
        '_c2rust_label: {
            if (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t > rlen {
            } else {
                __assert_fail(
                    b"IOSIZE > rlen\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1726 as ::core::ffi::c_uint,
                    b"void show_utf8(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        snprintf(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(rlen as isize),
            (IOSIZE as size_t).wrapping_sub(rlen),
            b"%02x \0".as_ptr() as *const ::core::ffi::c_char,
            if *line.offset(i as isize) as ::core::ffi::c_int == NL {
                NUL
            } else {
                *line.offset(i as isize) as uint8_t as ::core::ffi::c_int
            },
        );
        clen -= 1;
        rlen = rlen.wrapping_add(strlen(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(rlen as isize),
        ));
        if rlen > (IOSIZE - 20 as ::core::ffi::c_int) as size_t {
            break;
        }
        i += 1;
    }
    msg(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn always_break(mut bc: ::core::ffi::c_int) -> bool {
    return bc == UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int;
}
unsafe extern "C" fn always_break_two(
    mut bc1: ::core::ffi::c_int,
    mut bc2: ::core::ffi::c_int,
) -> bool {
    return bc1 != UTF8PROC_BOUNDCLASS_PREPEND as ::core::ffi::c_int
        && bc2 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
        || bc1 >= UTF8PROC_BOUNDCLASS_CR as ::core::ffi::c_int
            && bc1 <= UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int
        || bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
            && (bc1 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
                || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn utf_head_off(
    mut base_in: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*p_in as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut base: *const uint8_t = base_in as *mut uint8_t;
    let mut p: *const uint8_t = p_in as *mut uint8_t;
    let mut start: *const uint8_t = p;
    while start > base
        && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int
        && p.offset_from(start) < 6 as isize
    {
        start = start.offset(-1);
    }
    let last_len: uint8_t = utf8len_tab[*start as usize];
    let mut cur_code: int32_t = utf_ptr2CharInfo_impl(start, last_len as uintptr_t);
    if cur_code < 0 as int32_t || p.offset_from(start) >= last_len as isize {
        return 0 as ::core::ffi::c_int;
    }
    let safe_end: *const uint8_t = start.offset(last_len as ::core::ffi::c_int as isize);
    let mut cur_bc: ::core::ffi::c_int =
        (*utf8proc_get_property(cur_code as utf8proc_int32_t)).boundclass() as ::core::ffi::c_int;
    if always_break(cur_bc) as ::core::ffi::c_int != 0 || start == base {
        return p.offset_from(start) as ::core::ffi::c_int;
    }
    let mut cur_pos: *const uint8_t = start;
    let p_start: *const uint8_t = start;
    while *start.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        start = start.offset(-1);
        if (*start as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            break;
        }
        while start > base
            && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
            && cur_pos.offset_from(start) < 6 as isize
        {
            start = start.offset(-1);
        }
        let mut prev_len: ::core::ffi::c_int = utf8len_tab[*start as usize] as ::core::ffi::c_int;
        let mut prev_code: int32_t = utf_ptr2CharInfo_impl(start, prev_len as uintptr_t);
        if prev_code < 0 as int32_t || (prev_len as isize) < cur_pos.offset_from(start) {
            start = cur_pos;
            break;
        } else {
            let mut prev_bc: ::core::ffi::c_int =
                (*utf8proc_get_property(prev_code as utf8proc_int32_t)).boundclass()
                    as ::core::ffi::c_int;
            if always_break_two(prev_bc, cur_bc) as ::core::ffi::c_int != 0
                && !arabic_combine(
                    prev_code as ::core::ffi::c_int,
                    cur_code as ::core::ffi::c_int,
                )
            {
                start = cur_pos;
                break;
            } else {
                if start == base {
                    break;
                }
                cur_pos = start;
                cur_bc = prev_bc;
                cur_code = prev_code;
            }
        }
    }
    if start == p_start && last_len as isize > p.offset_from(start) {
        return p.offset_from(start) as ::core::ffi::c_int;
    }
    let mut q: *const uint8_t = start;
    while q < p {
        let mut len: ::core::ffi::c_int = utfc_ptr2len_len(
            q as *const ::core::ffi::c_char,
            safe_end.offset_from(q) as ::core::ffi::c_int,
        );
        if q.offset(len as isize) > p {
            return p.offset_from(q) as ::core::ffi::c_int;
        }
        q = q.offset(len as isize);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utfc_next_impl(mut cur: StrCharInfo) -> StrCharInfo {
    let mut prev_code: int32_t = cur.chr.value;
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    '_c2rust_label: {
        if *next as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"*next >= 0x80\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1855 as ::core::ffi::c_uint,
                b"StrCharInfo utfc_next_impl(StrCharInfo)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    loop {
        let next_len: uint8_t = utf8len_tab[*next as usize];
        let next_code: int32_t = utf_ptr2CharInfo_impl(next, next_len as uintptr_t);
        if !utf_iscomposing(
            prev_code as ::core::ffi::c_int,
            next_code as ::core::ffi::c_int,
            &raw mut state,
        ) {
            return StrCharInfo {
                ptr: next as *mut ::core::ffi::c_char,
                chr: CharInfo {
                    value: next_code,
                    len: if next_code < 0 as int32_t {
                        1 as ::core::ffi::c_int
                    } else {
                        next_len as ::core::ffi::c_int
                    },
                },
            };
        }
        prev_code = next_code;
        next = next.offset(next_len as ::core::ffi::c_int as isize);
        if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
            as ::core::ffi::c_long
            != 0
        {
            return StrCharInfo {
                ptr: next as *mut ::core::ffi::c_char,
                chr: CharInfo {
                    value: *next as int32_t,
                    len: 1 as ::core::ffi::c_int,
                },
            };
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn utf_eat_space(mut cc: ::core::ffi::c_int) -> bool {
    return cc >= 0x2000 as ::core::ffi::c_int && cc <= 0x206f as ::core::ffi::c_int
        || cc >= 0x2e00 as ::core::ffi::c_int && cc <= 0x2e7f as ::core::ffi::c_int
        || cc >= 0x3000 as ::core::ffi::c_int && cc <= 0x303f as ::core::ffi::c_int
        || cc >= 0xff01 as ::core::ffi::c_int && cc <= 0xff0f as ::core::ffi::c_int
        || cc >= 0xff1a as ::core::ffi::c_int && cc <= 0xff20 as ::core::ffi::c_int
        || cc >= 0xff3b as ::core::ffi::c_int && cc <= 0xff40 as ::core::ffi::c_int
        || cc >= 0xff5b as ::core::ffi::c_int && cc <= 0xff65 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf_allow_break_before(mut cc: ::core::ffi::c_int) -> bool {
    static mut BOL_prohibition_punct: [::core::ffi::c_int; 43] = [
        '!' as ::core::ffi::c_int,
        '%' as ::core::ffi::c_int,
        ')' as ::core::ffi::c_int,
        ',' as ::core::ffi::c_int,
        ':' as ::core::ffi::c_int,
        ';' as ::core::ffi::c_int,
        '>' as ::core::ffi::c_int,
        '?' as ::core::ffi::c_int,
        ']' as ::core::ffi::c_int,
        '}' as ::core::ffi::c_int,
        0x2019 as ::core::ffi::c_int,
        0x201d as ::core::ffi::c_int,
        0x2020 as ::core::ffi::c_int,
        0x2021 as ::core::ffi::c_int,
        0x2026 as ::core::ffi::c_int,
        0x2030 as ::core::ffi::c_int,
        0x2031 as ::core::ffi::c_int,
        0x203c as ::core::ffi::c_int,
        0x2047 as ::core::ffi::c_int,
        0x2048 as ::core::ffi::c_int,
        0x2049 as ::core::ffi::c_int,
        0x2103 as ::core::ffi::c_int,
        0x2109 as ::core::ffi::c_int,
        0x3001 as ::core::ffi::c_int,
        0x3002 as ::core::ffi::c_int,
        0x3009 as ::core::ffi::c_int,
        0x300b as ::core::ffi::c_int,
        0x300d as ::core::ffi::c_int,
        0x300f as ::core::ffi::c_int,
        0x3011 as ::core::ffi::c_int,
        0x3015 as ::core::ffi::c_int,
        0x3017 as ::core::ffi::c_int,
        0x3019 as ::core::ffi::c_int,
        0x301b as ::core::ffi::c_int,
        0xff01 as ::core::ffi::c_int,
        0xff09 as ::core::ffi::c_int,
        0xff0c as ::core::ffi::c_int,
        0xff0e as ::core::ffi::c_int,
        0xff1a as ::core::ffi::c_int,
        0xff1b as ::core::ffi::c_int,
        0xff1f as ::core::ffi::c_int,
        0xff3d as ::core::ffi::c_int,
        0xff5d as ::core::ffi::c_int,
    ];
    let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 43]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_int; 43]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while first < last {
        let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
        if cc == BOL_prohibition_punct[mid as usize] {
            return false_0 != 0;
        } else if cc > BOL_prohibition_punct[mid as usize] {
            first = mid + 1 as ::core::ffi::c_int;
        } else {
            last = mid - 1 as ::core::ffi::c_int;
        }
    }
    return cc != BOL_prohibition_punct[first as usize];
}
#[no_mangle]
pub unsafe extern "C" fn utf_allow_break_after(mut cc: ::core::ffi::c_int) -> bool {
    static mut EOL_prohibition_punct: [::core::ffi::c_int; 19] = [
        '(' as ::core::ffi::c_int,
        '<' as ::core::ffi::c_int,
        '[' as ::core::ffi::c_int,
        '`' as ::core::ffi::c_int,
        '{' as ::core::ffi::c_int,
        0x2018 as ::core::ffi::c_int,
        0x201c as ::core::ffi::c_int,
        0x3008 as ::core::ffi::c_int,
        0x300a as ::core::ffi::c_int,
        0x300c as ::core::ffi::c_int,
        0x300e as ::core::ffi::c_int,
        0x3010 as ::core::ffi::c_int,
        0x3014 as ::core::ffi::c_int,
        0x3016 as ::core::ffi::c_int,
        0x3018 as ::core::ffi::c_int,
        0x301a as ::core::ffi::c_int,
        0xff08 as ::core::ffi::c_int,
        0xff3b as ::core::ffi::c_int,
        0xff5b as ::core::ffi::c_int,
    ];
    let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 19]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_int; 19]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while first < last {
        let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
        if cc == EOL_prohibition_punct[mid as usize] {
            return false_0 != 0;
        } else if cc > EOL_prohibition_punct[mid as usize] {
            first = mid + 1 as ::core::ffi::c_int;
        } else {
            last = mid - 1 as ::core::ffi::c_int;
        }
    }
    return cc != EOL_prohibition_punct[first as usize];
}
#[no_mangle]
pub unsafe extern "C" fn utf_allow_break(
    mut cc: ::core::ffi::c_int,
    mut ncc: ::core::ffi::c_int,
) -> bool {
    if cc == ncc && (cc == 0x2014 as ::core::ffi::c_int || cc == 0x2026 as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    return utf_allow_break_after(cc) as ::core::ffi::c_int != 0
        && utf_allow_break_before(ncc) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mb_copy_char(
    fp: *mut *const ::core::ffi::c_char,
    tp: *mut *mut ::core::ffi::c_char,
) {
    let l: size_t = utfc_ptr2len(*fp) as size_t;
    memmove(
        *tp as *mut ::core::ffi::c_void,
        *fp as *const ::core::ffi::c_void,
        l,
    );
    *tp = (*tp).offset(l as isize);
    *fp = (*fp).offset(l as isize);
}
#[no_mangle]
pub unsafe extern "C" fn mb_off_next(
    mut base: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut head_off: ::core::ffi::c_int = utf_head_off(base, p);
    if head_off == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return utfc_ptr2len(p.offset(-(head_off as isize))) - head_off;
}
#[no_mangle]
pub unsafe extern "C" fn utf_cp_bounds_len(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
    mut p_len: ::core::ffi::c_int,
) -> CharBoundsOff {
    '_c2rust_label: {
        if base <= p_in && p_len > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"base <= p_in && p_len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/mbyte.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2053 as ::core::ffi::c_uint,
                b"CharBoundsOff utf_cp_bounds_len(const char *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let b: *const uint8_t = base as *mut uint8_t;
    let p: *const uint8_t = p_in as *mut uint8_t;
    if (*p as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint {
        return CharBoundsOff {
            begin_off: 0 as int8_t,
            end_off: 1 as int8_t,
        };
    }
    let max_first_off: ::core::ffi::c_int = -if (p.offset_from(b) as ::core::ffi::c_int)
        < MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    {
        p.offset_from(b) as ::core::ffi::c_int
    } else {
        MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    let mut first_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while utf_is_trail_byte(*p.offset(first_off as isize)) {
        if first_off == max_first_off {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        first_off -= 1;
    }
    let max_end_off: ::core::ffi::c_int =
        utf8len_tab[*p.offset(first_off as isize) as usize] as ::core::ffi::c_int + first_off;
    if max_end_off <= 0 as ::core::ffi::c_int || max_end_off > p_len {
        return CharBoundsOff {
            begin_off: 0 as int8_t,
            end_off: 1 as int8_t,
        };
    }
    let mut end_off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while end_off < max_end_off {
        if !utf_is_trail_byte(*p.offset(end_off as isize)) {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        end_off += 1;
    }
    return CharBoundsOff {
        begin_off: -first_off as int8_t,
        end_off: max_end_off as int8_t,
    };
}
#[no_mangle]
pub unsafe extern "C" fn utf_cp_bounds(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> CharBoundsOff {
    return utf_cp_bounds_len(base, p_in, INT_MAX);
}
#[no_mangle]
pub unsafe extern "C" fn utf_find_illegal() {
    let mut pos: pos_T = (*curwin).w_cursor;
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    if enc_canon_props((*curbuf).b_p_fenc) & ENC_8BIT as ::core::ffi::c_int != 0 {
        convert_setup(&raw mut vimconv, p_enc, (*curbuf).b_p_fenc);
    }
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    '_theend: {
        loop {
            let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            if vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                xfree(tofree as *mut ::core::ffi::c_void);
                tofree = string_convert(&raw mut vimconv, p, ::core::ptr::null_mut::<size_t>());
                if tofree.is_null() {
                    break;
                }
                p = tofree;
            }
            while *p as ::core::ffi::c_int != NUL {
                let mut len: ::core::ffi::c_int = utf_ptr2len(p);
                if *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
                    && (len == 1 as ::core::ffi::c_int || utf_char2len(utf_ptr2char(p)) != len)
                {
                    if vimconv.vc_type == CONV_NONE as ::core::ffi::c_int {
                        (*curwin).w_cursor.col += p.offset_from(get_cursor_pos_ptr()) as colnr_T;
                    } else {
                        let mut l: ::core::ffi::c_int = 0;
                        len = p.offset_from(tofree) as ::core::ffi::c_int;
                        p = get_cursor_pos_ptr();
                        while *p as ::core::ffi::c_int != NUL && {
                            let c2rust_fresh1 = len;
                            len = len - 1;
                            c2rust_fresh1 > 0 as ::core::ffi::c_int
                        } {
                            l = utf_ptr2len(p);
                            (*curwin).w_cursor.col += l;
                            p = p.offset(l as isize);
                        }
                    }
                    break '_theend;
                } else {
                    p = p.offset(len as isize);
                }
            }
            if (*curwin).w_cursor.lnum == (*curbuf).b_ml.ml_line_count {
                break;
            }
            (*curwin).w_cursor.lnum += 1;
            (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        (*curwin).w_cursor = pos;
        beep_flush();
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn utf_valid_string(
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> bool {
    let mut p: *const uint8_t = s as *mut uint8_t;
    while if end.is_null() {
        (*p as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
    } else {
        (p < end as *mut uint8_t as *const uint8_t) as ::core::ffi::c_int
    } != 0
    {
        let mut l: ::core::ffi::c_int = utf8len_tab_zero[*p as usize] as ::core::ffi::c_int;
        if l == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if !end.is_null() && p.offset(l as isize) > end as *mut uint8_t as *const uint8_t {
            return false_0 != 0;
        }
        p = p.offset(1);
        loop {
            l -= 1;
            if l <= 0 as ::core::ffi::c_int {
                break;
            }
            let c2rust_fresh12 = p;
            p = p.offset(1);
            if *c2rust_fresh12 as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                != 0x80 as ::core::ffi::c_int
            {
                return false_0 != 0;
            }
        }
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mb_adjust_cursor() {
    mark_mb_adjustpos(curbuf, &raw mut (*curwin).w_cursor);
}
#[no_mangle]
pub unsafe extern "C" fn mb_check_adjust_col(mut win_: *mut ::core::ffi::c_void) {
    let mut win: *mut win_T = win_ as *mut win_T;
    let mut oldcol: colnr_T = (*win).w_cursor.col;
    if oldcol != 0 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = ml_get_buf((*win).w_buffer, (*win).w_cursor.lnum);
        let mut len: colnr_T = strlen(p) as colnr_T;
        if len == 0 as ::core::ffi::c_int || oldcol < 0 as ::core::ffi::c_int {
            (*win).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            if oldcol > len {
                (*win).w_cursor.col =
                    (len as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            }
            (*win).w_cursor.col -= utf_head_off(p, p.offset((*win).w_cursor.col as isize));
        }
        if (*win).w_cursor.coladd == 1 as ::core::ffi::c_int
            && *p.offset((*win).w_cursor.col as isize) as ::core::ffi::c_int != TAB
            && vim_isprintc(utf_ptr2char(p.offset((*win).w_cursor.col as isize)))
                as ::core::ffi::c_int
                != 0
            && ptr2cells(p.offset((*win).w_cursor.col as isize)) > 1 as ::core::ffi::c_int
        {
            (*win).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn mb_prevptr(
    mut line: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if p > line {
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn mb_charlen(mut str: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = str;
    let mut count: ::core::ffi::c_int = 0;
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    count = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL {
        p = p.offset(utfc_ptr2len(p) as isize);
        count += 1;
    }
    return count;
}
#[no_mangle]
pub unsafe extern "C" fn mb_charlen_len(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = str;
    let mut count: ::core::ffi::c_int = 0;
    count = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL && p < str.offset(len as isize) {
        p = p.offset(utfc_ptr2len(p) as isize);
        count += 1;
    }
    return count;
}
#[no_mangle]
pub unsafe extern "C" fn mb_unescape(
    pp: *mut *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    static mut buf: [::core::ffi::c_char; 6] = [0; 6];
    let mut buf_idx: size_t = 0 as size_t;
    let mut str: *mut uint8_t = *pp as *mut uint8_t;
    let mut str_idx: size_t = 0 as size_t;
    while *str.offset(str_idx as isize) as ::core::ffi::c_int != NUL && buf_idx < 4 as size_t {
        if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL
            && *str.offset(str_idx.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == KS_SPECIAL
            && *str.offset(str_idx.wrapping_add(2 as size_t) as isize) as ::core::ffi::c_int
                == KE_FILLER
        {
            let c2rust_fresh13 = buf_idx;
            buf_idx = buf_idx.wrapping_add(1);
            buf[c2rust_fresh13 as usize] = K_SPECIAL as ::core::ffi::c_char;
            str_idx = str_idx.wrapping_add(2 as size_t);
        } else {
            if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL {
                break;
            }
            let c2rust_fresh14 = buf_idx;
            buf_idx = buf_idx.wrapping_add(1);
            buf[c2rust_fresh14 as usize] = *str.offset(str_idx as isize) as ::core::ffi::c_char;
        }
        buf[buf_idx as usize] = NUL as ::core::ffi::c_char;
        if utf_ptr2len(&raw mut buf as *mut ::core::ffi::c_char) > 1 as ::core::ffi::c_int {
            *pp = (str as *const ::core::ffi::c_char)
                .offset(str_idx as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            return &raw mut buf as *mut ::core::ffi::c_char;
        }
        if (buf[0 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int)
            < 128 as ::core::ffi::c_int
        {
            break;
        }
        str_idx = str_idx.wrapping_add(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn enc_skip(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if strncmp(
        p,
        b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return p.offset(6 as ::core::ffi::c_int as isize);
    }
    if strncmp(
        p,
        b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return p.offset(5 as ::core::ffi::c_int as isize);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn enc_canonize(
    mut enc: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if strcmp(enc, b"default\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        return xstrdup(fenc_default);
    }
    let mut r: *mut ::core::ffi::c_char =
        xmalloc(strlen(enc).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = r;
    let mut s: *mut ::core::ffi::c_char = enc;
    while *s as ::core::ffi::c_int != NUL {
        if *s as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            *c2rust_fresh15 = '-' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = (if (*s as ::core::ffi::c_int) < 'A' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
            {
                *s as ::core::ffi::c_int
            } else {
                *s as ::core::ffi::c_int + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) as ::core::ffi::c_char;
        }
        s = s.offset(1);
    }
    *p = NUL as ::core::ffi::c_char;
    p = enc_skip(r);
    if strncmp(
        p,
        b"microsoft-cp\0".as_ptr() as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p as *mut ::core::ffi::c_void,
            p.offset(10 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(10 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
    }
    if strncmp(
        p,
        b"iso8859\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p.offset(4 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        *p.offset(3 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
    }
    if strncmp(
        p,
        b"iso-8859\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
        && *p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '-' as ::core::ffi::c_int
    {
        memmove(
            p.offset(9 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(8 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(8 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        *p.offset(8 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
    }
    if strncmp(
        p,
        b"latin-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p.offset(5 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(6 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(6 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
    }
    let mut i: ::core::ffi::c_int = 0;
    if enc_canon_search(p) >= 0 as ::core::ffi::c_int {
        if p != r {
            memmove(
                r as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                strlen(p).wrapping_add(1 as size_t),
            );
        }
    } else {
        i = enc_alias_search(p);
        if i >= 0 as ::core::ffi::c_int {
            xfree(r as *mut ::core::ffi::c_void);
            r = xstrdup(enc_canon_table[i as usize].name);
        }
    }
    return r;
}
unsafe extern "C" fn enc_alias_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !enc_alias_table[i as usize].name.is_null() {
        if strcmp(name, enc_alias_table[i as usize].name) == 0 as ::core::ffi::c_int {
            return enc_alias_table[i as usize].canon;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn enc_locale() -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 50] = [0; 50];
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = nl_langinfo(CODESET as ::core::ffi::c_int);
    if s.is_null() || *s as ::core::ffi::c_int == NUL {
        s = setlocale(LC_CTYPE, ::core::ptr::null::<::core::ffi::c_char>());
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            s = os_getenv_noalloc(b"LC_ALL\0".as_ptr() as *const ::core::ffi::c_char);
            if !s.is_null() {
                s = os_getenv_noalloc(b"LC_CTYPE\0".as_ptr() as *const ::core::ffi::c_char);
                if !s.is_null() {
                    s = os_getenv_noalloc(b"LANG\0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
        }
    }
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *const ::core::ffi::c_char = vim_strchr(s, '.' as ::core::ffi::c_int);
    's_140: {
        if !p.is_null() {
            if p > s.offset(2 as ::core::ffi::c_int as isize)
                && strncasecmp(
                    p.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                    b"EUC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    3 as ::core::ffi::c_int as size_t,
                ) == 0
                && *(*__ctype_b_loc())
                    .offset(*p.offset(4 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '-' as ::core::ffi::c_int
                && *p.offset(-3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '_' as ::core::ffi::c_int
            {
                memmove(
                    &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    b"euc-\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    4 as size_t,
                );
                buf[4 as ::core::ffi::c_int as usize] =
                    (if *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                buf[5 as ::core::ffi::c_int as usize] =
                    (if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                buf[6 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                break 's_140;
            } else {
                s = p.offset(1 as ::core::ffi::c_int as isize);
            }
        }
        i = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[::core::ffi::c_char; 50]>() as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            && *s.offset(i as isize) as ::core::ffi::c_int != NUL
        {
            if *s.offset(i as isize) as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                || *s.offset(i as isize) as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            {
                buf[i as usize] = '-' as ::core::ffi::c_char;
            } else {
                if !(*s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                    >= 'A' as ::core::ffi::c_uint
                    && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                        <= 'Z' as ::core::ffi::c_uint
                    || *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                        >= 'a' as ::core::ffi::c_uint
                        && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                            <= 'z' as ::core::ffi::c_uint
                    || ascii_isdigit(*s.offset(i as isize) as uint8_t as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0)
                {
                    break;
                }
                buf[i as usize] = (if (*s.offset(i as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *s.offset(i as isize) as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
                {
                    *s.offset(i as isize) as ::core::ffi::c_int
                } else {
                    *s.offset(i as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) as ::core::ffi::c_char;
            }
            i += 1;
        }
        buf[i as usize] = NUL as ::core::ffi::c_char;
    }
    return enc_canonize(&raw mut buf as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn my_iconv_open(
    mut to: *mut ::core::ffi::c_char,
    mut from: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut tobuf: [::core::ffi::c_char; 400] = [0; 400];
    static mut iconv_working: WorkingStatus = kUnknown;
    if iconv_working as ::core::ffi::c_uint == kBroken as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        );
    }
    let mut fd: iconv_t = iconv_open(enc_skip(to), enc_skip(from));
    if fd
        != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
        && iconv_working as ::core::ffi::c_uint
            == kUnknown as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *mut ::core::ffi::c_char = &raw mut tobuf as *mut ::core::ffi::c_char;
        let mut tolen: size_t = ICONV_TESTLEN as size_t;
        iconv(
            fd,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<size_t>(),
            &raw mut p,
            &raw mut tolen,
        );
        if p.is_null() {
            iconv_working = kBroken;
            iconv_close(fd);
            fd = ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            );
        } else {
            iconv_working = kWorking;
        }
    }
    return fd;
}
pub const ICONV_TESTLEN: ::core::ffi::c_int = 400 as ::core::ffi::c_int;
unsafe extern "C" fn iconv_string(
    vcp: *const vimconv_T,
    mut str: *const ::core::ffi::c_char,
    mut slen: size_t,
    mut unconvlenp: *mut size_t,
    mut resultlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut done: size_t = 0 as size_t;
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut from: *const ::core::ffi::c_char = str;
    let mut fromlen: size_t = slen;
    loop {
        if len == 0 as size_t || *__errno_location() == ICONV_E2BIG {
            len = len
                .wrapping_add(fromlen.wrapping_mul(2 as size_t))
                .wrapping_add(40 as size_t);
            let mut p: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
            if done > 0 as size_t {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    result as *const ::core::ffi::c_void,
                    done,
                );
            }
            xfree(result as *mut ::core::ffi::c_void);
            result = p;
        }
        to = result.offset(done as isize);
        let mut tolen: size_t = len.wrapping_sub(done).wrapping_sub(2 as size_t);
        if iconv(
            (*vcp).vc_fd,
            &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
            &raw mut fromlen,
            &raw mut to,
            &raw mut tolen,
        ) != SIZE_MAX as size_t
        {
            *to = NUL as ::core::ffi::c_char;
            break;
        } else if !(*vcp).vc_fail
            && !unconvlenp.is_null()
            && (*__errno_location() == ICONV_EINVAL || *__errno_location() == EINVAL)
        {
            *to = NUL as ::core::ffi::c_char;
            *unconvlenp = fromlen;
            break;
        } else {
            if !(*vcp).vc_fail
                && (*__errno_location() == ICONV_EILSEQ
                    || *__errno_location() == EILSEQ
                    || *__errno_location() == ICONV_EINVAL
                    || *__errno_location() == EINVAL)
            {
                let c2rust_fresh10 = to;
                to = to.offset(1);
                *c2rust_fresh10 = '?' as ::core::ffi::c_char;
                if utf_ptr2cells(from) > 1 as ::core::ffi::c_int {
                    let c2rust_fresh11 = to;
                    to = to.offset(1);
                    *c2rust_fresh11 = '?' as ::core::ffi::c_char;
                }
                let mut l: ::core::ffi::c_int =
                    utfc_ptr2len_len(from, fromlen as ::core::ffi::c_int);
                from = from.offset(l as isize);
                fromlen = fromlen.wrapping_sub(l as size_t);
            } else if *__errno_location() != ICONV_E2BIG {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut result as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
                break;
            }
            done = to.offset_from(result) as size_t;
        }
    }
    if !resultlenp.is_null() && !result.is_null() {
        *resultlenp = to.offset_from(result) as size_t;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn f_iconv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let from: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char));
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let to: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char));
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    convert_setup(&raw mut vimconv, from, to);
    if vimconv.vc_type == CONV_NONE as ::core::ffi::c_int {
        (*rettv).vval.v_string = xstrdup(str);
    } else {
        (*rettv).vval.v_string = string_convert(
            &raw mut vimconv,
            str as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(from as *mut ::core::ffi::c_void);
    xfree(to as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn convert_setup(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return convert_setup_ext(vcp, from, true_0 != 0, to, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn convert_setup_ext(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut from_unicode_is_utf8: bool,
    mut to: *mut ::core::ffi::c_char,
    mut to_unicode_is_utf8: bool,
) -> ::core::ffi::c_int {
    let mut from_is_utf8: ::core::ffi::c_int = 0;
    let mut to_is_utf8: ::core::ffi::c_int = 0;
    if (*vcp).vc_type == CONV_ICONV as ::core::ffi::c_int
        && (*vcp).vc_fd
            != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
    {
        iconv_close((*vcp).vc_fd);
    }
    *vcp = vimconv_T {
        vc_type: CONV_NONE as ::core::ffi::c_int,
        vc_factor: 1 as ::core::ffi::c_int,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false_0 != 0,
    };
    if from.is_null()
        || *from as ::core::ffi::c_int == NUL
        || to.is_null()
        || *to as ::core::ffi::c_int == NUL
        || strcmp(from, to) == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    let mut from_prop: ::core::ffi::c_int = enc_canon_props(from);
    let mut to_prop: ::core::ffi::c_int = enc_canon_props(to);
    if from_unicode_is_utf8 {
        from_is_utf8 = from_prop & ENC_UNICODE as ::core::ffi::c_int;
    } else {
        from_is_utf8 = (from_prop == ENC_UNICODE as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if to_unicode_is_utf8 {
        to_is_utf8 = to_prop & ENC_UNICODE as ::core::ffi::c_int;
    } else {
        to_is_utf8 = (to_prop == ENC_UNICODE as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if from_prop & ENC_LATIN1 as ::core::ffi::c_int != 0 && to_is_utf8 != 0 {
        (*vcp).vc_type = CONV_TO_UTF8 as ::core::ffi::c_int;
        (*vcp).vc_factor = 2 as ::core::ffi::c_int;
    } else if from_prop & ENC_LATIN9 as ::core::ffi::c_int != 0 && to_is_utf8 != 0 {
        (*vcp).vc_type = CONV_9_TO_UTF8 as ::core::ffi::c_int;
        (*vcp).vc_factor = 3 as ::core::ffi::c_int;
    } else if from_is_utf8 != 0 && to_prop & ENC_LATIN1 as ::core::ffi::c_int != 0 {
        (*vcp).vc_type = CONV_TO_LATIN1 as ::core::ffi::c_int;
    } else if from_is_utf8 != 0 && to_prop & ENC_LATIN9 as ::core::ffi::c_int != 0 {
        (*vcp).vc_type = CONV_TO_LATIN9 as ::core::ffi::c_int;
    } else {
        (*vcp).vc_fd = my_iconv_open(
            (if to_is_utf8 != 0 {
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                to as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
            (if from_is_utf8 != 0 {
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                from as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
        );
        if (*vcp).vc_fd
            != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
        {
            (*vcp).vc_type = CONV_ICONV as ::core::ffi::c_int;
            (*vcp).vc_factor = 4 as ::core::ffi::c_int;
        }
    }
    if (*vcp).vc_type == CONV_NONE as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn string_convert(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    return string_convert_ext(vcp, ptr, lenp, ::core::ptr::null_mut::<size_t>());
}
#[no_mangle]
pub unsafe extern "C" fn string_convert_ext(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
    mut unconvlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut d: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut c: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    if lenp.is_null() {
        len = strlen(ptr);
    } else {
        len = *lenp;
    }
    if len == 0 as size_t {
        return xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    match (*vcp).vc_type {
        1 => {
            retval =
                xmalloc(len.wrapping_mul(2 as size_t).wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i: size_t = 0 as size_t;
            while i < len {
                c = *ptr.offset(i as isize) as uint8_t as ::core::ffi::c_int;
                if c < 0x80 as ::core::ffi::c_int {
                    let c2rust_fresh2 = d;
                    d = d.offset(1);
                    *c2rust_fresh2 = c as uint8_t;
                } else {
                    let c2rust_fresh3 = d;
                    d = d.offset(1);
                    *c2rust_fresh3 = (0xc0 as ::core::ffi::c_int
                        + (c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as uint8_t
                            as ::core::ffi::c_int) as uint8_t;
                    let c2rust_fresh4 = d;
                    d = d.offset(1);
                    *c2rust_fresh4 =
                        (0x80 as ::core::ffi::c_int + (c & 0x3f as ::core::ffi::c_int)) as uint8_t;
                }
                i = i.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        2 => {
            retval =
                xmalloc(len.wrapping_mul(3 as size_t).wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i_0: size_t = 0 as size_t;
            while i_0 < len {
                c = *ptr.offset(i_0 as isize) as uint8_t as ::core::ffi::c_int;
                match c {
                    164 => {
                        c = 0x20ac as ::core::ffi::c_int;
                    }
                    166 => {
                        c = 0x160 as ::core::ffi::c_int;
                    }
                    168 => {
                        c = 0x161 as ::core::ffi::c_int;
                    }
                    180 => {
                        c = 0x17d as ::core::ffi::c_int;
                    }
                    184 => {
                        c = 0x17e as ::core::ffi::c_int;
                    }
                    188 => {
                        c = 0x152 as ::core::ffi::c_int;
                    }
                    189 => {
                        c = 0x153 as ::core::ffi::c_int;
                    }
                    190 => {
                        c = 0x178 as ::core::ffi::c_int;
                    }
                    _ => {}
                }
                d = d.offset(utf_char2bytes(c, d as *mut ::core::ffi::c_char) as isize);
                i_0 = i_0.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        3 | 4 => {
            retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i_1: size_t = 0 as size_t;
            while i_1 < len {
                let mut l: ::core::ffi::c_int = utf_ptr2len_len(
                    ptr.offset(i_1 as isize),
                    len.wrapping_sub(i_1) as ::core::ffi::c_int,
                );
                if l == 0 as ::core::ffi::c_int {
                    let c2rust_fresh5 = d;
                    d = d.offset(1);
                    *c2rust_fresh5 = NUL as uint8_t;
                } else if l == 1 as ::core::ffi::c_int {
                    let mut l_w: uint8_t =
                        utf8len_tab_zero[*ptr.offset(i_1 as isize) as uint8_t as usize];
                    if l_w as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        xfree(retval as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    if !unconvlenp.is_null() && l_w as size_t > len.wrapping_sub(i_1) {
                        *unconvlenp = len.wrapping_sub(i_1);
                        break;
                    } else {
                        let c2rust_fresh6 = d;
                        d = d.offset(1);
                        *c2rust_fresh6 = *ptr.offset(i_1 as isize) as uint8_t;
                    }
                } else {
                    c = utf_ptr2char(ptr.offset(i_1 as isize));
                    if (*vcp).vc_type == CONV_TO_LATIN9 as ::core::ffi::c_int {
                        match c {
                            8364 => {
                                c = 0xa4 as ::core::ffi::c_int;
                            }
                            352 => {
                                c = 0xa6 as ::core::ffi::c_int;
                            }
                            353 => {
                                c = 0xa8 as ::core::ffi::c_int;
                            }
                            381 => {
                                c = 0xb4 as ::core::ffi::c_int;
                            }
                            382 => {
                                c = 0xb8 as ::core::ffi::c_int;
                            }
                            338 => {
                                c = 0xbc as ::core::ffi::c_int;
                            }
                            339 => {
                                c = 0xbd as ::core::ffi::c_int;
                            }
                            376 => {
                                c = 0xbe as ::core::ffi::c_int;
                            }
                            164 | 166 | 168 | 180 | 184 | 188 | 189 | 190 => {
                                c = 0x100 as ::core::ffi::c_int;
                            }
                            _ => {}
                        }
                    }
                    if !utf_iscomposing_legacy(c) {
                        if c < 0x100 as ::core::ffi::c_int {
                            let c2rust_fresh7 = d;
                            d = d.offset(1);
                            *c2rust_fresh7 = c as uint8_t;
                        } else if (*vcp).vc_fail {
                            xfree(retval as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        } else {
                            let c2rust_fresh8 = d;
                            d = d.offset(1);
                            *c2rust_fresh8 = 0xbf as uint8_t;
                            if utf_char2cells(c) > 1 as ::core::ffi::c_int {
                                let c2rust_fresh9 = d;
                                d = d.offset(1);
                                *c2rust_fresh9 = '?' as uint8_t;
                            }
                        }
                    }
                    i_1 = i_1.wrapping_add((l as size_t).wrapping_sub(1 as size_t));
                }
                i_1 = i_1.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        5 => {
            retval = iconv_string(vcp, ptr, len, unconvlenp, lenp) as *mut uint8_t;
        }
        _ => {}
    }
    return retval as *mut ::core::ffi::c_char;
}
static mut cw_table: *mut cw_interval_T = ::core::ptr::null_mut::<cw_interval_T>();
static mut cw_table_size: size_t = 0 as size_t;
unsafe extern "C" fn cw_value(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if cw_table.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (c as int64_t) < (*cw_table.offset(0 as ::core::ffi::c_int as isize)).first {
        return 0 as ::core::ffi::c_int;
    }
    let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut top: ::core::ffi::c_int = cw_table_size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    while top >= bot {
        let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
        if (*cw_table.offset(mid as isize)).last < c as int64_t {
            bot = mid + 1 as ::core::ffi::c_int;
        } else if (*cw_table.offset(mid as isize)).first > c as int64_t {
            top = mid - 1 as ::core::ffi::c_int;
        } else {
            return (*cw_table.offset(mid as isize)).width as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tv_nr_compare(
    mut a1: *const ::core::ffi::c_void,
    mut a2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let li1: *const listitem_T = tv_list_first(*(a1 as *mut *const list_T));
    let li2: *const listitem_T = tv_list_first(*(a2 as *mut *const list_T));
    let n1: varnumber_T = (*li1).li_tv.vval.v_number;
    let n2: varnumber_T = (*li2).li_tv.vval.v_number;
    return if n1 == n2 {
        0 as ::core::ffi::c_int
    } else if n1 > n2 {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_setcellwidths(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut ptrs: *mut *const list_T = ::core::ptr::null_mut::<*const list_T>();
    let mut item: ::core::ffi::c_int = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list
            .is_null()
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut table: *mut cw_interval_T = ::core::ptr::null_mut::<cw_interval_T>();
    let table_size: size_t = tv_list_len(l) as size_t;
    if table_size != 0 as size_t {
        ptrs = xmalloc(::core::mem::size_of::<*const list_T>().wrapping_mul(table_size))
            as *mut *const list_T;
        item = 0 as ::core::ffi::c_int;
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let li_tv: *const typval_T = &raw const (*li).li_tv;
                if (*li_tv).v_type as ::core::ffi::c_uint
                    != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*li_tv).vval.v_list.is_null()
                {
                    semsg(
                        gettext(
                            &raw const e_list_item_nr_is_not_list as *const ::core::ffi::c_char,
                        ),
                        item,
                    );
                    xfree(ptrs as *mut ::core::ffi::c_void);
                    return;
                }
                let li_l: *const list_T = (*li_tv).vval.v_list;
                *ptrs.offset(item as isize) = li_l;
                let mut lili: *const listitem_T = tv_list_first(li_l);
                let mut i: ::core::ffi::c_int = 0;
                let mut n1: varnumber_T = 0;
                i = 0 as ::core::ffi::c_int;
                while !lili.is_null() {
                    let lili_tv: *const typval_T = &raw const (*lili).li_tv;
                    if (*lili_tv).v_type as ::core::ffi::c_uint
                        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                    if i == 0 as ::core::ffi::c_int {
                        n1 = (*lili_tv).vval.v_number;
                        if n1 < 0x80 as varnumber_T {
                            emsg(gettext(
                                &raw const e_only_values_of_0x80_and_higher_supported
                                    as *const ::core::ffi::c_char,
                            ));
                            xfree(ptrs as *mut ::core::ffi::c_void);
                            return;
                        }
                    } else if i == 1 as ::core::ffi::c_int && (*lili_tv).vval.v_number < n1 {
                        semsg(
                            gettext(
                                &raw const e_list_item_nr_range_invalid
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    } else if i == 2 as ::core::ffi::c_int
                        && ((*lili_tv).vval.v_number < 1 as varnumber_T
                            || (*lili_tv).vval.v_number > 2 as varnumber_T)
                    {
                        semsg(
                            gettext(
                                &raw const e_list_item_nr_cell_width_invalid
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    }
                    lili = (*lili).li_next;
                    i += 1;
                }
                if i != 3 as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            &raw const e_list_item_nr_does_not_contain_3_numbers
                                as *const ::core::ffi::c_char,
                        ),
                        item,
                    );
                    xfree(ptrs as *mut ::core::ffi::c_void);
                    return;
                }
                item += 1;
                li = (*li).li_next;
            }
        }
        qsort(
            ptrs as *mut ::core::ffi::c_void,
            table_size,
            ::core::mem::size_of::<*const list_T>(),
            Some(
                tv_nr_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        table = xmalloc(::core::mem::size_of::<cw_interval_T>().wrapping_mul(table_size))
            as *mut cw_interval_T;
        item = 0 as ::core::ffi::c_int;
        while (item as size_t) < table_size {
            let li_l_0: *const list_T = *ptrs.offset(item as isize);
            let mut lili_0: *const listitem_T = tv_list_first(li_l_0);
            let n1_0: varnumber_T = (*lili_0).li_tv.vval.v_number;
            if item > 0 as ::core::ffi::c_int
                && n1_0 <= (*table.offset((item - 1 as ::core::ffi::c_int) as isize)).last
            {
                semsg(
                    gettext(&raw const e_overlapping_ranges_for_nr as *const ::core::ffi::c_char),
                    n1_0 as size_t,
                );
                xfree(ptrs as *mut ::core::ffi::c_void);
                xfree(table as *mut ::core::ffi::c_void);
                return;
            }
            (*table.offset(item as isize)).first = n1_0 as int64_t;
            lili_0 = (*lili_0).li_next;
            (*table.offset(item as isize)).last = (*lili_0).li_tv.vval.v_number as int64_t;
            lili_0 = (*lili_0).li_next;
            (*table.offset(item as isize)).width =
                (*lili_0).li_tv.vval.v_number as ::core::ffi::c_char;
            item += 1;
        }
        xfree(ptrs as *mut ::core::ffi::c_void);
    }
    let cw_table_save: *mut cw_interval_T = cw_table;
    let cw_table_size_save: size_t = cw_table_size;
    cw_table = table;
    cw_table_size = table_size;
    let error: *const ::core::ffi::c_char = check_chars_options();
    if !error.is_null() {
        emsg(gettext(error));
        cw_table = cw_table_save;
        cw_table_size = cw_table_size_save;
        xfree(table as *mut ::core::ffi::c_void);
        return;
    }
    xfree(cw_table_save as *mut ::core::ffi::c_void);
    changed_window_setting_all();
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcellwidths(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, cw_table_size as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < cw_table_size {
        let mut entry: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_number(entry, (*cw_table.offset(i as isize)).first);
        tv_list_append_number(entry, (*cw_table.offset(i as isize)).last);
        tv_list_append_number(entry, (*cw_table.offset(i as isize)).width as varnumber_T);
        tv_list_append_list((*rettv).vval.v_list, entry);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_charclass(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        return;
    }
    (*rettv).vval.v_number = mb_get_class(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn get_encoding_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx
        >= ::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
            .wrapping_div(::core::mem::size_of::<C2Rust_Unnamed_21>())
            .wrapping_div(
                (::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
                    .wrapping_rem(::core::mem::size_of::<C2Rust_Unnamed_21>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return enc_canon_table[idx as usize].name as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn mb_strcmp_ic(
    mut ic: bool,
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return if ic as ::core::ffi::c_int != 0 {
        mb_stricmp(s1, s2)
    } else {
        strcmp(s1, s2)
    };
}
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn utf_is_trail_byte(byte: uint8_t) -> bool {
    return (byte as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
        as ::core::ffi::c_uint
        == 0x80 as ::core::ffi::c_uint;
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int = utf8len_tab[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const __LC_CTYPE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ICONV_E2BIG: ::core::ffi::c_int = E2BIG;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
pub const ICONV_EILSEQ: ::core::ffi::c_int = EILSEQ;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const EILSEQ: ::core::ffi::c_int = 84 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    corrections = [
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (0x80 as uint32_t)
            .wrapping_add((0xc0 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
