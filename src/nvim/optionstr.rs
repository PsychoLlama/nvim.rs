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
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn parse_winborder(
        fconfig: *mut WinConfig,
        border_opt: *mut ::core::ffi::c_char,
        err: *mut Error,
    ) -> bool;
    static mut opt_bkc_values: [*const ::core::ffi::c_char; 6];
    static mut opt_bh_values: [*const ::core::ffi::c_char; 6];
    static mut opt_bt_values: [*const ::core::ffi::c_char; 9];
    static mut opt_cot_values: [*const ::core::ffi::c_char; 12];
    static mut opt_dip_algorithm_values: [*const ::core::ffi::c_char; 5];
    static mut opt_dip_inline_values: [*const ::core::ffi::c_char; 5];
    static mut opt_ff_values: [*const ::core::ffi::c_char; 4];
    static mut opt_ssop_values: [*const ::core::ffi::c_char; 19];
    static mut opt_scl_values: [*const ::core::ffi::c_char; 23];
    static mut opt_spo_values: [*const ::core::ffi::c_char; 3];
    static mut opt_tc_values: [*const ::core::ffi::c_char; 6];
    static mut opt_ve_values: [*const ::core::ffi::c_char; 7];
    fn check_ei(ei: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn get_event_name_no_group(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
        win: bool,
    ) -> *mut ::core::ffi::c_char;
    static mut empty_string_option: [::core::ffi::c_char; 0];
    static mut p_bs: *mut ::core::ffi::c_char;
    static mut p_bg: *mut ::core::ffi::c_char;
    static mut p_bkc: *mut ::core::ffi::c_char;
    static mut bkc_flags: ::core::ffi::c_uint;
    static mut p_bex: *mut ::core::ffi::c_char;
    static mut breakat_flags: [::core::ffi::c_char; 256];
    static mut p_breakat: *mut ::core::ffi::c_char;
    static mut p_enc: *mut ::core::ffi::c_char;
    static mut p_cia: *mut ::core::ffi::c_char;
    static mut cia_flags: ::core::ffi::c_uint;
    static mut p_cot: *mut ::core::ffi::c_char;
    static mut cot_flags: ::core::ffi::c_uint;
    static mut p_pumborder: *mut ::core::ffi::c_char;
    static mut p_ei: *mut ::core::ffi::c_char;
    static mut p_fenc: *mut ::core::ffi::c_char;
    static mut p_fcs: *mut ::core::ffi::c_char;
    static mut p_hlg: *mut ::core::ffi::c_char;
    static mut p_isk: *mut ::core::ffi::c_char;
    static mut p_km: *mut ::core::ffi::c_char;
    static mut p_lcs: *mut ::core::ffi::c_char;
    static mut p_mousescroll: *mut ::core::ffi::c_char;
    static mut p_mousescroll_vert: OptInt;
    static mut p_mousescroll_hor: OptInt;
    static mut p_pm: *mut ::core::ffi::c_char;
    static mut p_ruf: *mut ::core::ffi::c_char;
    static mut ssop_flags: ::core::ffi::c_uint;
    static mut spo_flags: ::core::ffi::c_uint;
    static mut p_tc: *mut ::core::ffi::c_char;
    static mut tc_flags: ::core::ffi::c_uint;
    static mut p_shada: *mut ::core::ffi::c_char;
    static mut p_ve: *mut ::core::ffi::c_char;
    static mut ve_flags: ::core::ffi::c_uint;
    static mut p_vfile: *mut ::core::ffi::c_char;
    static mut p_winborder: *mut ::core::ffi::c_char;
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
    fn init_chartab() -> ::core::ffi::c_int;
    fn buf_init_chartab(buf: *mut buf_T, global: bool) -> ::core::ffi::c_int;
    fn check_isopt(var: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn transchar_byte(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn hexhex2nr(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ExpandGeneric(
        pat: *const ::core::ffi::c_char,
        xp: *mut expand_T,
        regmatch: *mut regmatch_T,
        matches: *mut *mut *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
        func: CompleteListItemGetter,
        escaped: bool,
    );
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn parse_shape_opt(what: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn diffanchors_changed(buflocal: bool) -> ::core::ffi::c_int;
    fn diffopt_changed() -> ::core::ffi::c_int;
    fn keymap_init() -> *mut ::core::ffi::c_char;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg: [::core::ffi::c_char; 0];
    static e_modifiable: [::core::ffi::c_char; 0];
    static e_unsupportedoption: [::core::ffi::c_char; 0];
    static e_leadtab_requires_tab: [::core::ffi::c_char; 0];
    static e_invalid_format_string_single_percent_s: [::core::ffi::c_char; 0];
    fn comp_col();
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn redrawWinline(wp: *mut win_T, lnum: linenr_T);
    fn status_redraw_buf(buf: *mut buf_T);
    fn get_scriptlocal_funcname(funcname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn do_unlet(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn get_var_value(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn check_opt_wim() -> ::core::ffi::c_int;
    fn foldmethodIsIndent(wp: *mut win_T) -> bool;
    fn foldmethodIsExpr(wp: *mut win_T) -> bool;
    fn foldmethodIsMarker(wp: *mut win_T) -> bool;
    fn foldmethodIsDiff(wp: *mut win_T) -> bool;
    fn newFoldLevel();
    fn foldUpdateAll(win: *mut win_T);
    static mut didset_vim: bool;
    static mut didset_vimruntime: bool;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut firstbuf: *mut buf_T;
    static mut ru_wid: ::core::ffi::c_int;
    static mut secure: ::core::ffi::c_int;
    static mut VIsual_active: bool;
    static mut cmdpreview: bool;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut km_stopsel: bool;
    static mut km_startsel: bool;
    static mut stl_syntax: ::core::ffi::c_int;
    fn schar_from_str(str: *const ::core::ffi::c_char) -> schar_T;
    fn schar_from_char(c: ::core::ffi::c_int) -> schar_T;
    fn init_highlight(both: bool, reset: bool);
    fn get_highlight_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn parse_cino(buf: *mut buf_T);
    fn tabstop_set(var: *mut ::core::ffi::c_char, array: *mut *mut colnr_T) -> bool;
    fn briopt_check(briopt: *mut ::core::ffi::c_char, wp: *mut win_T) -> bool;
    fn set_cpt_callbacks(args: *mut optset_T) -> ::core::ffi::c_int;
    fn os_time() -> Timestamp;
    fn free_fmark(fm: fmark_T);
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2schar(p: *const ::core::ffi::c_char, firstc: *mut ::core::ffi::c_int) -> schar_T;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn enc_canonize(enc: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_encoding_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn ml_setflags(buf: *mut buf_T);
    fn validate_virtcol(wp: *mut win_T);
    fn msg_grid_validate();
    fn messagesopt_changed() -> ::core::ffi::c_int;
    fn verbose_stop();
    fn verbose_open() -> ::core::ffi::c_int;
    fn api_clear_error(value: *mut Error);
    fn get_option_default(opt_idx: OptIndex, opt_flags: ::core::ffi::c_int) -> OptVal;
    fn did_set_title();
    fn redraw_titles();
    fn valid_name(val: *const ::core::ffi::c_char, allowed: *const ::core::ffi::c_char) -> bool;
    fn parse_winhl_opt(winhl: *const ::core::ffi::c_char, wp: *mut win_T) -> bool;
    fn get_option(opt_idx: OptIndex) -> *mut vimoption_T;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn get_option_varp_scope_from(
        opt_idx: OptIndex,
        opt_flags: ::core::ffi::c_int,
        buf: *mut buf_T,
        win: *mut win_T,
    ) -> *mut ::core::ffi::c_void;
    fn set_iminsert_global(buf: *mut buf_T);
    fn set_imsearch_global(buf: *mut buf_T);
    fn fill_culopt_flags(val: *mut ::core::ffi::c_char, wp: *mut win_T) -> ::core::ffi::c_int;
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn skip_to_option_part(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn vim_unsetenv_ext(var: *const ::core::ffi::c_char);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn get_shada_parameter(type_0: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn spell_reload();
    fn valid_spelllang(val: *const ::core::ffi::c_char) -> bool;
    fn valid_spellfile(val: *const ::core::ffi::c_char) -> bool;
    fn did_set_spell_option() -> *const ::core::ffi::c_char;
    fn compile_cap_prog(synblock: *mut synblock_T) -> *const ::core::ffi::c_char;
    fn spell_check_sps() -> ::core::ffi::c_int;
    fn spell_check_msm() -> ::core::ffi::c_int;
    fn global_stl_height() -> ::core::ffi::c_int;
    fn check_colorcolumn(
        cc: *mut ::core::ffi::c_char,
        wp: *mut win_T,
    ) -> *const ::core::ffi::c_char;
    fn win_config_float(wp: *mut win_T, fconfig: WinConfig);
    fn terminal_notify_theme(term: *mut Terminal, dark: bool);
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
    pub b_wininfo: C2Rust_Unnamed_10,
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
    pub b_signcols: C2Rust_Unnamed_2,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_0,
    pub update_callbacks: C2Rust_Unnamed,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
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
pub struct C2Rust_Unnamed_0 {
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
    pub data: C2Rust_Unnamed_1,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub sst_union: C2Rust_Unnamed_3,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
    pub data: C2Rust_Unnamed_4,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
    pub fc_fixvar: [C2Rust_Unnamed_5; 12],
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
pub struct C2Rust_Unnamed_5 {
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
    pub uh_next: C2Rust_Unnamed_9,
    pub uh_prev: C2Rust_Unnamed_8,
    pub uh_alt_next: C2Rust_Unnamed_7,
    pub uh_alt_prev: C2Rust_Unnamed_6,
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
pub union C2Rust_Unnamed_6 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
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
pub struct C2Rust_Unnamed_10 {
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
    pub type_0: C2Rust_Unnamed_11,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_11 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_13 = 250;
pub const kZIndexMessages: C2Rust_Unnamed_13 = 200;
pub const kZIndexPopupMenu: C2Rust_Unnamed_13 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_13 = 50;
pub const kZIndexDefaultGrid: C2Rust_Unnamed_13 = 0;
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
pub type CompleteListItemGetter =
    Option<unsafe extern "C" fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kOptFlagColon: C2Rust_Unnamed_14 = 33554432;
pub const kOptFlagFunc: C2Rust_Unnamed_14 = 16777216;
pub const kOptFlagMLE: C2Rust_Unnamed_14 = 8388608;
pub const kOptFlagHLOnly: C2Rust_Unnamed_14 = 4194304;
pub const kOptFlagNDname: C2Rust_Unnamed_14 = 2097152;
pub const kOptFlagCurswant: C2Rust_Unnamed_14 = 1048576;
pub const kOptFlagPriMkrc: C2Rust_Unnamed_14 = 524288;
pub const kOptFlagInsecure: C2Rust_Unnamed_14 = 262144;
pub const kOptFlagNFname: C2Rust_Unnamed_14 = 131072;
pub const kOptFlagNoGlob: C2Rust_Unnamed_14 = 65536;
pub const kOptFlagGettext: C2Rust_Unnamed_14 = 32768;
pub const kOptFlagSecure: C2Rust_Unnamed_14 = 16384;
pub const kOptFlagFlagList: C2Rust_Unnamed_14 = 8192;
pub const kOptFlagNoDup: C2Rust_Unnamed_14 = 4096;
pub const kOptFlagOneComma: C2Rust_Unnamed_14 = 3072;
pub const kOptFlagComma: C2Rust_Unnamed_14 = 1024;
pub const kOptFlagRedrClear: C2Rust_Unnamed_14 = 896;
pub const kOptFlagRedrAll: C2Rust_Unnamed_14 = 768;
pub const kOptFlagRedrBuf: C2Rust_Unnamed_14 = 512;
pub const kOptFlagRedrWin: C2Rust_Unnamed_14 = 256;
pub const kOptFlagRedrStat: C2Rust_Unnamed_14 = 128;
pub const kOptFlagRedrTabl: C2Rust_Unnamed_14 = 64;
pub const kOptFlagUIOption: C2Rust_Unnamed_14 = 32;
pub const kOptFlagNoMkrc: C2Rust_Unnamed_14 = 16;
pub const kOptFlagWasSet: C2Rust_Unnamed_14 = 8;
pub const kOptFlagNoDefault: C2Rust_Unnamed_14 = 4;
pub const kOptFlagNoDefExp: C2Rust_Unnamed_14 = 2;
pub const kOptFlagExpand: C2Rust_Unnamed_14 = 1;
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub type OptScopeFlags = uint8_t;
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
pub struct optset_T {
    pub os_varp: *mut ::core::ffi::c_void,
    pub os_idx: OptIndex,
    pub os_flags: ::core::ffi::c_int,
    pub os_oldval: OptValData,
    pub os_newval: OptValData,
    pub os_value_checked: bool,
    pub os_value_changed: bool,
    pub os_restore_chartab: bool,
    pub os_errbuf: *mut ::core::ffi::c_char,
    pub os_errbuflen: size_t,
    pub os_win: *mut ::core::ffi::c_void,
    pub os_buf: *mut ::core::ffi::c_void,
}
pub type opt_did_set_cb_T =
    Option<unsafe extern "C" fn(*mut optset_T) -> *const ::core::ffi::c_char>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct optexpand_T {
    pub oe_varp: *mut ::core::ffi::c_char,
    pub oe_idx: OptIndex,
    pub oe_opt_value: *mut ::core::ffi::c_char,
    pub oe_append: bool,
    pub oe_include_orig_val: bool,
    pub oe_regmatch: *mut regmatch_T,
    pub oe_xp: *mut expand_T,
    pub oe_set_arg: *mut ::core::ffi::c_char,
}
pub type opt_expand_cb_T = Option<
    unsafe extern "C" fn(
        *mut optexpand_T,
        *mut ::core::ffi::c_int,
        *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimoption_T {
    pub fullname: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
    pub flags: uint32_t,
    pub type_0: OptValType,
    pub scope_flags: OptScopeFlags,
    pub var: *mut ::core::ffi::c_void,
    pub flags_var: *mut ::core::ffi::c_uint,
    pub scope_idx: [ssize_t; 3],
    pub immutable: bool,
    pub values: *mut *const ::core::ffi::c_char,
    pub values_len: size_t,
    pub opt_did_set_cb: opt_did_set_cb_T,
    pub opt_expand_cb: opt_expand_cb_T,
    pub def_val: OptVal,
    pub script_ctx: sctx_T,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const STL_CLICK_FUNC: C2Rust_Unnamed_15 = 64;
pub const STL_TABCLOSENR: C2Rust_Unnamed_15 = 88;
pub const STL_TABPAGENR: C2Rust_Unnamed_15 = 84;
pub const STL_HIGHLIGHT_COMB: C2Rust_Unnamed_15 = 36;
pub const STL_HIGHLIGHT: C2Rust_Unnamed_15 = 35;
pub const STL_USER_HL: C2Rust_Unnamed_15 = 42;
pub const STL_TRUNCMARK: C2Rust_Unnamed_15 = 60;
pub const STL_SEPARATE: C2Rust_Unnamed_15 = 61;
pub const STL_VIM_EXPR: C2Rust_Unnamed_15 = 123;
pub const STL_SIGNCOL: C2Rust_Unnamed_15 = 115;
pub const STL_FOLDCOL: C2Rust_Unnamed_15 = 67;
pub const STL_SHOWCMD: C2Rust_Unnamed_15 = 83;
pub const STL_PAGENUM: C2Rust_Unnamed_15 = 78;
pub const STL_ARGLISTSTAT: C2Rust_Unnamed_15 = 97;
pub const STL_ALTPERCENT: C2Rust_Unnamed_15 = 80;
pub const STL_PERCENTAGE: C2Rust_Unnamed_15 = 112;
pub const STL_QUICKFIX: C2Rust_Unnamed_15 = 113;
pub const STL_MODIFIED_ALT: C2Rust_Unnamed_15 = 77;
pub const STL_MODIFIED: C2Rust_Unnamed_15 = 109;
pub const STL_PREVIEWFLAG_ALT: C2Rust_Unnamed_15 = 87;
pub const STL_PREVIEWFLAG: C2Rust_Unnamed_15 = 119;
pub const STL_FILETYPE_ALT: C2Rust_Unnamed_15 = 89;
pub const STL_FILETYPE: C2Rust_Unnamed_15 = 121;
pub const STL_HELPFLAG_ALT: C2Rust_Unnamed_15 = 72;
pub const STL_HELPFLAG: C2Rust_Unnamed_15 = 104;
pub const STL_ROFLAG_ALT: C2Rust_Unnamed_15 = 82;
pub const STL_ROFLAG: C2Rust_Unnamed_15 = 114;
pub const STL_BYTEVAL_X: C2Rust_Unnamed_15 = 66;
pub const STL_BYTEVAL: C2Rust_Unnamed_15 = 98;
pub const STL_OFFSET_X: C2Rust_Unnamed_15 = 79;
pub const STL_OFFSET: C2Rust_Unnamed_15 = 111;
pub const STL_KEYMAP: C2Rust_Unnamed_15 = 107;
pub const STL_BUFNO: C2Rust_Unnamed_15 = 110;
pub const STL_NUMLINES: C2Rust_Unnamed_15 = 76;
pub const STL_LINE: C2Rust_Unnamed_15 = 108;
pub const STL_VIRTCOL_ALT: C2Rust_Unnamed_15 = 86;
pub const STL_VIRTCOL: C2Rust_Unnamed_15 = 118;
pub const STL_COLUMN: C2Rust_Unnamed_15 = 99;
pub const STL_FILENAME: C2Rust_Unnamed_15 = 116;
pub const STL_FULLPATH: C2Rust_Unnamed_15 = 70;
pub const STL_FILEPATH: C2Rust_Unnamed_15 = 102;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptBkcFlagBreakhardlink: C2Rust_Unnamed_16 = 16;
pub const kOptBkcFlagBreaksymlink: C2Rust_Unnamed_16 = 8;
pub const kOptBkcFlagNo: C2Rust_Unnamed_16 = 4;
pub const kOptBkcFlagAuto: C2Rust_Unnamed_16 = 2;
pub const kOptBkcFlagYes: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptSsopFlagSkiprtp: C2Rust_Unnamed_17 = 131072;
pub const kOptSsopFlagTerminal: C2Rust_Unnamed_17 = 65536;
pub const kOptSsopFlagTabpages: C2Rust_Unnamed_17 = 32768;
pub const kOptSsopFlagCursor: C2Rust_Unnamed_17 = 16384;
pub const kOptSsopFlagFolds: C2Rust_Unnamed_17 = 8192;
pub const kOptSsopFlagCurdir: C2Rust_Unnamed_17 = 4096;
pub const kOptSsopFlagSesdir: C2Rust_Unnamed_17 = 2048;
pub const kOptSsopFlagUnix: C2Rust_Unnamed_17 = 1024;
pub const kOptSsopFlagSlash: C2Rust_Unnamed_17 = 512;
pub const kOptSsopFlagGlobals: C2Rust_Unnamed_17 = 256;
pub const kOptSsopFlagBlank: C2Rust_Unnamed_17 = 128;
pub const kOptSsopFlagHelp: C2Rust_Unnamed_17 = 64;
pub const kOptSsopFlagOptions: C2Rust_Unnamed_17 = 32;
pub const kOptSsopFlagLocaloptions: C2Rust_Unnamed_17 = 16;
pub const kOptSsopFlagWinsize: C2Rust_Unnamed_17 = 8;
pub const kOptSsopFlagResize: C2Rust_Unnamed_17 = 4;
pub const kOptSsopFlagWinpos: C2Rust_Unnamed_17 = 2;
pub const kOptSsopFlagBuffers: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_18 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_18 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_18 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_18 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_18 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_18 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_18 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_18 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_18 = 79;
pub const SHM_OVER: C2Rust_Unnamed_18 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_18 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_18 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_18 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_18 = 97;
pub const SHM_WRI: C2Rust_Unnamed_18 = 119;
pub const SHM_LINES: C2Rust_Unnamed_18 = 108;
pub const SHM_MOD: C2Rust_Unnamed_18 = 109;
pub const SHM_RO: C2Rust_Unnamed_18 = 114;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_19 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_19 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_19 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_19 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_19 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_19 = 20;
pub const UPD_VALID: C2Rust_Unnamed_19 = 10;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const CPT_COUNT: C2Rust_Unnamed_20 = 4;
pub const CPT_INFO: C2Rust_Unnamed_20 = 3;
pub const CPT_MENU: C2Rust_Unnamed_20 = 2;
pub const CPT_KIND: C2Rust_Unnamed_20 = 1;
pub const CPT_ABBR: C2Rust_Unnamed_20 = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_21 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_21 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_21 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_21 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_21 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_21 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_21 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_21 = 1;
pub type CharsOption = ::core::ffi::c_uint;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chars_tab {
    pub cp: *mut schar_T,
    pub name: String_0,
    pub def: *const ::core::ffi::c_char,
    pub fallback: *const ::core::ffi::c_char,
}
pub const LSIZE: C2Rust_Unnamed_22 = 512;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 74] = unsafe {
    ::core::mem::transmute::<[u8; 74], [::core::ffi::c_char; 74]>(
        *b"int opt_strings_flags(const char *, const char **, unsigned int *, _Bool)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const B_IMODE_USE_INSERT: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const HIGHLIGHT_INIT: [::core::ffi::c_char; 779] = unsafe {
    ::core::mem::transmute::<
        [u8; 779],
        [::core::ffi::c_char; 779],
    >(
        *b"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert\0",
    )
};
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FO_ALL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"tcro/q2vlb1mMBn,aw]jp\0")
};
pub const CPO_VI: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_\0",
    )
};
pub const WW_ALL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"bshl<>[]~\0") };
pub const MOUSE_ALL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"anvichr\0") };
pub const MOUSESCROLL_VERT_DFLT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const MOUSESCROLL_HOR_DFLT: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const COCU_ALL: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"nvic\0") };
pub const COM_ALL: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"nbsmexflrO\0") };
pub const SCL_NO: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SHAPE_CURSOR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut e_illegal_character_after_chr: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E535: Illegal character after <%c>\0",
    )
};
static mut e_comma_required: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E536: Comma required\0")
};
static mut e_unclosed_expression_sequence: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E540: Unclosed expression sequence\0",
    )
};
static mut e_unbalanced_groups: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E542: Unbalanced groups\0")
};
static mut e_backupext_and_patchmode_are_equal: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E589: 'backupext' and 'patchmode' are equal\0",
    )
};
static mut e_showbreak_contains_unprintable_or_wide_character: [::core::ffi::c_char; 57] = unsafe {
    ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
        *b"E595: 'showbreak' contains unprintable or wide character\0",
    )
};
static mut e_wrong_number_of_characters_for_field_str: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"E1511: Wrong number of characters for field \"%s\"\0",
    )
};
static mut e_wrong_character_width_for_field_str: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E1512: Wrong character width for field \"%s\"\0",
    )
};
static mut SHM_ALL: [::core::ffi::c_char; 23] = [
    SHM_RO as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_MOD as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_LINES as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_WRI as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_ABBREVIATIONS as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_WRITE as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_TRUNC as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_TRUNCALL as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_OVER as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_OVERALL as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_SEARCH as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_ATTENTION as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_INTRO as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_COMPLETIONMENU as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_COMPLETIONSCAN as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_RECORDING as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_FILEINFO as ::core::ffi::c_int as ::core::ffi::c_char,
    SHM_SEARCHCOUNT as ::core::ffi::c_int as ::core::ffi::c_char,
    'n' as ::core::ffi::c_char,
    'f' as ::core::ffi::c_char,
    'x' as ::core::ffi::c_char,
    'i' as ::core::ffi::c_char,
    0 as ::core::ffi::c_char,
];
#[no_mangle]
pub unsafe extern "C" fn didset_string_options() {
    check_str_opt(
        kOptCasemap,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptBackupcopy,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptBelloff,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptCompleteopt,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptSessionoptions,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptViewoptions,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptFoldopen,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptDisplay,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptJumpoptions,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptRedrawdebug,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptTagcase,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptTermpastefilter,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptVirtualedit,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptSwitchbuf,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptTabclose,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptWildoptions,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    check_str_opt(
        kOptClipboard,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn illegal_char(
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext(b"E539: Illegal character <%s>\0".as_ptr() as *const ::core::ffi::c_char),
        transchar(c),
    );
    return errbuf;
}
unsafe extern "C" fn illegal_char_after_chr(
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext(&raw const e_illegal_character_after_chr as *const ::core::ffi::c_char),
        c,
    );
    return errbuf;
}
#[no_mangle]
pub unsafe extern "C" fn check_buf_options(mut buf: *mut buf_T) {
    check_string_option(&raw mut (*buf).b_p_bh);
    check_string_option(&raw mut (*buf).b_p_bt);
    check_string_option(&raw mut (*buf).b_p_fenc);
    check_string_option(&raw mut (*buf).b_p_ff);
    check_string_option(&raw mut (*buf).b_p_def);
    check_string_option(&raw mut (*buf).b_p_inc);
    check_string_option(&raw mut (*buf).b_p_inex);
    check_string_option(&raw mut (*buf).b_p_inde);
    check_string_option(&raw mut (*buf).b_p_indk);
    check_string_option(&raw mut (*buf).b_p_fp);
    check_string_option(&raw mut (*buf).b_p_fex);
    check_string_option(&raw mut (*buf).b_p_kp);
    check_string_option(&raw mut (*buf).b_p_mps);
    check_string_option(&raw mut (*buf).b_p_fo);
    check_string_option(&raw mut (*buf).b_p_flp);
    check_string_option(&raw mut (*buf).b_p_isk);
    check_string_option(&raw mut (*buf).b_p_com);
    check_string_option(&raw mut (*buf).b_p_cms);
    check_string_option(&raw mut (*buf).b_p_nf);
    check_string_option(&raw mut (*buf).b_p_qe);
    check_string_option(&raw mut (*buf).b_p_syn);
    check_string_option(&raw mut (*buf).b_s.b_syn_isk);
    check_string_option(&raw mut (*buf).b_s.b_p_spc);
    check_string_option(&raw mut (*buf).b_s.b_p_spf);
    check_string_option(&raw mut (*buf).b_s.b_p_spl);
    check_string_option(&raw mut (*buf).b_s.b_p_spo);
    check_string_option(&raw mut (*buf).b_p_sua);
    check_string_option(&raw mut (*buf).b_p_cink);
    check_string_option(&raw mut (*buf).b_p_cino);
    parse_cino(buf);
    check_string_option(&raw mut (*buf).b_p_lop);
    check_string_option(&raw mut (*buf).b_p_ft);
    check_string_option(&raw mut (*buf).b_p_cinw);
    check_string_option(&raw mut (*buf).b_p_cinsd);
    check_string_option(&raw mut (*buf).b_p_cot);
    check_string_option(&raw mut (*buf).b_p_cpt);
    check_string_option(&raw mut (*buf).b_p_cfu);
    check_string_option(&raw mut (*buf).b_p_ofu);
    check_string_option(&raw mut (*buf).b_p_keymap);
    check_string_option(&raw mut (*buf).b_p_gefm);
    check_string_option(&raw mut (*buf).b_p_gp);
    check_string_option(&raw mut (*buf).b_p_mp);
    check_string_option(&raw mut (*buf).b_p_efm);
    check_string_option(&raw mut (*buf).b_p_ep);
    check_string_option(&raw mut (*buf).b_p_path);
    check_string_option(&raw mut (*buf).b_p_tags);
    check_string_option(&raw mut (*buf).b_p_ffu);
    check_string_option(&raw mut (*buf).b_p_tfu);
    check_string_option(&raw mut (*buf).b_p_tc);
    check_string_option(&raw mut (*buf).b_p_dict);
    check_string_option(&raw mut (*buf).b_p_dia);
    check_string_option(&raw mut (*buf).b_p_tsr);
    check_string_option(&raw mut (*buf).b_p_tsrfu);
    check_string_option(&raw mut (*buf).b_p_lw);
    check_string_option(&raw mut (*buf).b_p_bkc);
    check_string_option(&raw mut (*buf).b_p_menc);
    check_string_option(&raw mut (*buf).b_p_vsts);
    check_string_option(&raw mut (*buf).b_p_vts);
}
#[no_mangle]
pub unsafe extern "C" fn free_string_option(mut p: *mut ::core::ffi::c_char) {
    if p != &raw mut empty_string_option as *mut ::core::ffi::c_char {
        xfree(p as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn clear_string_option(mut pp: *mut *mut ::core::ffi::c_char) {
    if *pp != &raw mut empty_string_option as *mut ::core::ffi::c_char {
        xfree(*pp as *mut ::core::ffi::c_void);
    }
    *pp = &raw mut empty_string_option as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn check_string_option(mut pp: *mut *mut ::core::ffi::c_char) {
    if (*pp).is_null() {
        *pp = &raw mut empty_string_option as *mut ::core::ffi::c_char;
    }
}
unsafe extern "C" fn valid_filetype(mut val: *const ::core::ffi::c_char) -> bool {
    return valid_name(val, b".-_\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn check_signcolumn(
    mut scl: *mut ::core::ffi::c_char,
    mut wp: *mut win_T,
) -> ::core::ffi::c_int {
    let mut val: *mut ::core::ffi::c_char =
        &raw mut empty_string_option as *mut ::core::ffi::c_char;
    if !scl.is_null() {
        val = scl;
    } else if !wp.is_null() {
        val = (*wp).w_onebuf_opt.wo_scl;
    }
    if *val as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    if opt_strings_flags(
        val,
        &raw mut opt_scl_values as *mut *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_uint>(),
        false_0 != 0,
    ) == OK
    {
        if wp.is_null() {
            return OK;
        }
        if strncmp(
            val,
            b"no\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0
        {
            (*wp).w_maxscwidth = SCL_NO;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(
            val,
            b"nu\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0
            && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
        {
            (*wp).w_maxscwidth = SCL_NUM;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(
            val,
            b"yes:\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0
        {
            (*wp).w_maxscwidth = *val.offset(4 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                - '0' as ::core::ffi::c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if *val as ::core::ffi::c_int == 'y' as ::core::ffi::c_int {
            (*wp).w_maxscwidth = 1 as ::core::ffi::c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(
            val,
            b"auto:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0
        {
            (*wp).w_minscwidth = 0 as ::core::ffi::c_int;
            (*wp).w_maxscwidth = *val.offset(5 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                - '0' as ::core::ffi::c_int;
        } else {
            (*wp).w_minscwidth = 0 as ::core::ffi::c_int;
            (*wp).w_maxscwidth = 1 as ::core::ffi::c_int;
        }
    } else {
        if strncmp(
            val,
            b"auto:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) != 0 as ::core::ffi::c_int
            || strlen(val) != 8 as size_t
            || !ascii_isdigit(*val.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            || *val.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '-' as ::core::ffi::c_int
            || !ascii_isdigit(*val.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            return FAIL;
        }
        let mut min: ::core::ffi::c_int = *val.offset(5 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            - '0' as ::core::ffi::c_int;
        let mut max: ::core::ffi::c_int = *val.offset(7 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            - '0' as ::core::ffi::c_int;
        if min < 1 as ::core::ffi::c_int
            || max < 2 as ::core::ffi::c_int
            || min > 8 as ::core::ffi::c_int
            || min >= max
        {
            return FAIL;
        }
        if wp.is_null() {
            return OK;
        }
        (*wp).w_minscwidth = min;
        (*wp).w_maxscwidth = max;
    }
    let mut scwidth: ::core::ffi::c_int = if (*wp).w_minscwidth <= 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else if (*wp).w_maxscwidth < (*wp).w_scwidth {
        (*wp).w_maxscwidth
    } else {
        (*wp).w_scwidth
    };
    (*wp).w_scwidth = if (*wp).w_minscwidth > scwidth {
        (*wp).w_minscwidth
    } else {
        scwidth
    };
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_stl_option(
    mut s: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut groupdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    static mut errbuf: [::core::ffi::c_char; 80] = [0; 80];
    while *s != 0 {
        while *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int != '%' as ::core::ffi::c_int
        {
            s = s.offset(1);
        }
        if *s == 0 {
            break;
        }
        s = s.offset(1);
        if *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == STL_TRUNCMARK as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == STL_SEPARATE as ::core::ffi::c_int
        {
            s = s.offset(1);
        } else if *s as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
            s = s.offset(1);
            groupdepth -= 1;
            if groupdepth < 0 as ::core::ffi::c_int {
                break;
            }
        } else {
            if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                s = s.offset(1);
            }
            while ascii_isdigit(*s as ::core::ffi::c_int) {
                s = s.offset(1);
            }
            if *s as ::core::ffi::c_int == STL_USER_HL as ::core::ffi::c_int {
                continue;
            }
            if *s as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                s = s.offset(1);
                while *s as ::core::ffi::c_int != 0
                    && ascii_isdigit(*s as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    s = s.offset(1);
                }
            }
            if *s as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                groupdepth += 1;
            } else {
                let mut c2rust_lvalue: [::core::ffi::c_char; 45] = [
                    STL_FILEPATH as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_FULLPATH as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_FILENAME as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_COLUMN as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_VIRTCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_VIRTCOL_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_LINE as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_NUMLINES as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_BUFNO as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_KEYMAP as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_OFFSET as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_OFFSET_X as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_BYTEVAL as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_BYTEVAL_X as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_ROFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_ROFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_HELPFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_HELPFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_FILETYPE as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_FILETYPE_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_PREVIEWFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_PREVIEWFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_MODIFIED as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_MODIFIED_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_PERCENTAGE as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_ALTPERCENT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_ARGLISTSTAT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_PAGENUM as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_SHOWCMD as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_VIM_EXPR as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_SEPARATE as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_TRUNCMARK as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_USER_HL as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_HIGHLIGHT as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_HIGHLIGHT_COMB as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_TABPAGENR as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_TABCLOSENR as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_CLICK_FUNC as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_TABPAGENR as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_TABCLOSENR as ::core::ffi::c_int as ::core::ffi::c_char,
                    STL_CLICK_FUNC as ::core::ffi::c_int as ::core::ffi::c_char,
                    0 as ::core::ffi::c_char,
                ];
                if vim_strchr(
                    &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                    *s as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                {
                    return illegal_char(
                        &raw mut errbuf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 80]>(),
                        *s as uint8_t as ::core::ffi::c_int,
                    );
                }
                if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                    s = s.offset(1);
                    let mut reevaluate: bool =
                        *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int;
                    if reevaluate as ::core::ffi::c_int != 0 && {
                        s = s.offset(1);
                        *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                    } {
                        return illegal_char(
                            &raw mut errbuf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 80]>(),
                            '}' as ::core::ffi::c_int,
                        );
                    }
                    while (*s as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                        || reevaluate as ::core::ffi::c_int != 0
                            && *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '%' as ::core::ffi::c_int)
                        && *s as ::core::ffi::c_int != 0
                    {
                        s = s.offset(1);
                    }
                    if *s as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                        return &raw const e_unclosed_expression_sequence
                            as *const ::core::ffi::c_char;
                    }
                }
            }
        }
    }
    if groupdepth != 0 as ::core::ffi::c_int {
        return &raw const e_unbalanced_groups as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn check_illegal_path_names(
    mut val: *mut ::core::ffi::c_char,
    mut flags: uint32_t,
) -> bool {
    return flags & kOptFlagNFname as ::core::ffi::c_int as uint32_t != 0
        && !strpbrk(
            val,
            (if secure != 0 {
                b"/\\*?[|;&<>\r\n\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"/\\*?[<>\r\n\0".as_ptr() as *const ::core::ffi::c_char
            }),
        )
        .is_null()
        || flags & kOptFlagNDname as ::core::ffi::c_int as uint32_t != 0
            && !strpbrk(
                val,
                b"*?[|;&<>\r\n\0".as_ptr() as *const ::core::ffi::c_char,
            )
            .is_null();
}
unsafe extern "C" fn did_set_opt_flags(
    mut val: *mut ::core::ffi::c_char,
    mut values: *mut *const ::core::ffi::c_char,
    mut flagp: *mut ::core::ffi::c_uint,
    mut list: bool,
) -> *const ::core::ffi::c_char {
    if opt_strings_flags(val, values, flagp, list) != OK {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn opt_values(
    mut idx: OptIndex,
    mut values_len: *mut size_t,
) -> *mut *const ::core::ffi::c_char {
    let mut idx1: OptIndex = (if idx as ::core::ffi::c_int == kOptViewoptions as ::core::ffi::c_int
    {
        kOptSessionoptions as ::core::ffi::c_int
    } else if idx as ::core::ffi::c_int == kOptFileformats as ::core::ffi::c_int {
        kOptFileformat as ::core::ffi::c_int
    } else {
        idx as ::core::ffi::c_int
    }) as OptIndex;
    let mut opt: *mut vimoption_T = get_option(idx1);
    if !values_len.is_null() {
        *values_len = (*opt).values_len;
    }
    return (*opt).values;
}
unsafe extern "C" fn check_str_opt(
    mut idx: OptIndex,
    mut varp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut opt: *mut vimoption_T = get_option(idx);
    if varp.is_null() {
        varp = (*opt).var as *mut *mut ::core::ffi::c_char;
    }
    let mut list: bool = (*opt).flags
        & (kOptFlagComma as ::core::ffi::c_int | kOptFlagOneComma as ::core::ffi::c_int)
            as uint32_t
        != 0;
    let mut values: *mut *const ::core::ffi::c_char =
        opt_values(idx, ::core::ptr::null_mut::<size_t>());
    return opt_strings_flags(*varp, values, (*opt).flags_var, list);
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_str_generic(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut values_len: size_t = 0;
    let mut values: *mut *const ::core::ffi::c_char =
        opt_values((*args).oe_idx, &raw mut values_len);
    return expand_set_opt_string(args, values, values_len, numMatches, matches);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_str_generic(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    return if check_str_opt(
        (*args).os_idx,
        (*args).os_varp as *mut *mut ::core::ffi::c_char,
    ) != OK
    {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
unsafe extern "C" fn did_set_option_listflag(
    mut val: *mut ::core::ffi::c_char,
    mut flags: *mut ::core::ffi::c_char,
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
) -> *const ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = val;
    while *s != 0 {
        if vim_strchr(flags, *s as uint8_t as ::core::ffi::c_int).is_null() {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as ::core::ffi::c_int);
        }
        s = s.offset(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn expand_set_opt_string(
    mut args: *mut optexpand_T,
    mut values: *mut *const ::core::ffi::c_char,
    mut numValues: size_t,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut regmatch: *mut regmatch_T = (*args).oe_regmatch;
    let mut include_orig_val: bool = (*args).oe_include_orig_val;
    let mut option_val: *mut ::core::ffi::c_char = (*args).oe_opt_value;
    *matches = xmalloc(
        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
            .wrapping_mul(numValues.wrapping_add(1 as size_t)),
    ) as *mut *mut ::core::ffi::c_char;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if include_orig_val as ::core::ffi::c_int != 0 && *option_val as ::core::ffi::c_int != NUL {
        let c2rust_fresh0 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh0 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut val: *mut *const ::core::ffi::c_char = values;
    while !(*val).is_null() {
        's_27: {
            if **val as ::core::ffi::c_int != NUL {
                if include_orig_val as ::core::ffi::c_int != 0
                    && *option_val as ::core::ffi::c_int != NUL
                {
                    if strcmp(*val, option_val) == 0 as ::core::ffi::c_int {
                        break 's_27;
                    }
                }
                if vim_regexec(regmatch, *val, 0 as colnr_T) {
                    let c2rust_fresh1 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh1 as isize);
                    *c2rust_lvalue_ptr_0 = xstrdup(*val);
                }
            }
        }
        val = val.offset(1);
    }
    if count == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void = matches as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}
static mut set_opt_callback_orig_option: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
static mut set_opt_callback_func: Option<
    unsafe extern "C" fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char,
> = None;
unsafe extern "C" fn expand_set_opt_callback(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        if !set_opt_callback_orig_option.is_null() {
            return set_opt_callback_orig_option;
        } else {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    }
    return set_opt_callback_func.expect("non-null function pointer")(
        xp,
        idx - 1 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn expand_set_opt_generic(
    mut args: *mut optexpand_T,
    mut func: CompleteListItemGetter,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    set_opt_callback_orig_option = if (*args).oe_include_orig_val as ::core::ffi::c_int != 0 {
        (*args).oe_opt_value
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    set_opt_callback_func = func as Option<
        unsafe extern "C" fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char,
    >;
    ExpandGeneric(
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        (*args).oe_xp,
        (*args).oe_regmatch,
        matches,
        numMatches,
        Some(
            expand_set_opt_callback
                as unsafe extern "C" fn(
                    *mut expand_T,
                    ::core::ffi::c_int,
                ) -> *mut ::core::ffi::c_char,
        ),
        false_0 != 0,
    );
    set_opt_callback_orig_option = ::core::ptr::null_mut::<::core::ffi::c_char>();
    set_opt_callback_func = None;
    return OK;
}
unsafe extern "C" fn expand_set_opt_listflag(
    mut args: *mut optexpand_T,
    mut flags: *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut option_val: *mut ::core::ffi::c_char = (*args).oe_opt_value;
    let mut cmdline_val: *mut ::core::ffi::c_char = (*args).oe_set_arg;
    let mut append: bool = (*args).oe_append;
    let mut include_orig_val: bool = (*args).oe_include_orig_val as ::core::ffi::c_int != 0
        && *option_val as ::core::ffi::c_int != NUL;
    let mut num_flags: size_t = strlen(flags);
    *matches = xmalloc(
        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
            .wrapping_mul(num_flags.wrapping_add(1 as size_t)),
    ) as *mut *mut ::core::ffi::c_char;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if include_orig_val {
        let c2rust_fresh7 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh7 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut flag: *mut ::core::ffi::c_char = flags;
    while *flag as ::core::ffi::c_int != NUL {
        if !(append as ::core::ffi::c_int != 0
            && !vim_strchr(option_val, *flag as ::core::ffi::c_int).is_null())
        {
            if vim_strchr(cmdline_val, *flag as ::core::ffi::c_int).is_null() {
                if !(include_orig_val as ::core::ffi::c_int != 0
                    && *option_val.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == NUL
                    && *flag as ::core::ffi::c_int
                        == *option_val.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                {
                    let c2rust_fresh8 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh8 as isize);
                    *c2rust_lvalue_ptr_0 = xmemdupz(flag as *const ::core::ffi::c_void, 1 as size_t)
                        as *mut ::core::ffi::c_char;
                }
            }
        }
        flag = flag.offset(1);
    }
    if count == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void = matches as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_ambiwidth(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    return check_chars_options();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_emoji(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if check_str_opt(
        kOptAmbiwidth,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return check_chars_options();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_background(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if *(*args)
        .os_oldval
        .string
        .data
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == *p_bg as ::core::ffi::c_int
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut dark: ::core::ffi::c_int =
        (*p_bg as ::core::ffi::c_int == 'd' as ::core::ffi::c_int) as ::core::ffi::c_int;
    init_highlight(false_0 != 0, false_0 != 0);
    if dark != (*p_bg as ::core::ffi::c_int == 'd' as ::core::ffi::c_int) as ::core::ffi::c_int
        && !get_var_value(b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        do_unlet(
            b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
        free_string_option(p_bg);
        p_bg = xstrdup(if dark != 0 {
            b"dark\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"light\0".as_ptr() as *const ::core::ffi::c_char
        });
        check_string_option(&raw mut p_bg);
        init_highlight(false_0 != 0, false_0 != 0);
    }
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        if !(*buf).terminal.is_null() {
            terminal_notify_theme((*buf).terminal, dark != 0);
        }
        buf = (*buf).b_next;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_backspace(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if ascii_isdigit(*p_bs as ::core::ffi::c_int) {
        if *p_bs as ::core::ffi::c_int != '2' as ::core::ffi::c_int {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return did_set_str_generic(args);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_backupcopy(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const ::core::ffi::c_char = (*args).os_oldval.string.data;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    let mut bkc: *mut ::core::ffi::c_char = p_bkc;
    let mut flags: *mut ::core::ffi::c_uint = &raw mut bkc_flags;
    if opt_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        bkc = (*buf).b_p_bkc;
        flags = &raw mut (*buf).b_bkc_flags;
    } else if opt_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
        (*buf).b_bkc_flags = 0 as ::core::ffi::c_uint;
    }
    if opt_flags & OPT_LOCAL as ::core::ffi::c_int != 0 && *bkc as ::core::ffi::c_int == NUL {
        *flags = 0 as ::core::ffi::c_uint;
    } else {
        if opt_strings_flags(
            bkc,
            &raw mut opt_bkc_values as *mut *const ::core::ffi::c_char,
            flags,
            true_0 != 0,
        ) != OK
        {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if (*flags & kOptBkcFlagAuto as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
            + (*flags & kOptBkcFlagYes as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
            + (*flags & kOptBkcFlagNo as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        {
            opt_strings_flags(
                oldval,
                &raw mut opt_bkc_values as *mut *const ::core::ffi::c_char,
                flags,
                true_0 != 0,
            );
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_backupext_or_patchmode(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if strcmp(
        (if *p_bex as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            p_bex.offset(1 as ::core::ffi::c_int as isize)
        } else {
            p_bex
        }),
        (if *p_pm as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            p_pm.offset(1 as ::core::ffi::c_int as isize)
        } else {
            p_pm
        }),
    ) == 0 as ::core::ffi::c_int
    {
        return &raw const e_backupext_and_patchmode_are_equal as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_breakat(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        breakat_flags[i as usize] = false_0 as ::core::ffi::c_char;
        i += 1;
    }
    if !p_breakat.is_null() {
        let mut p: *mut ::core::ffi::c_char = p_breakat;
        while *p != 0 {
            breakat_flags[*p as uint8_t as usize] = true_0 as ::core::ffi::c_char;
            p = p.offset(1);
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_breakindentopt(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if briopt_check(
        *varp,
        (if varp == &raw mut (*win).w_onebuf_opt.wo_briopt {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        }),
    ) as ::core::ffi::c_int
        == FAIL
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    if varp == &raw mut (*win).w_onebuf_opt.wo_briopt && (*win).w_briopt_list != 0 {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_bufhidden(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    return did_set_opt_flags(
        (*buf).b_p_bh,
        &raw mut opt_bh_values as *mut *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_uint>(),
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_buftype(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if !(*buf).terminal.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 't' as ::core::ffi::c_int
        || (*buf).terminal.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int
        || opt_strings_flags(
            (*buf).b_p_bt,
            &raw mut opt_bt_values as *mut *const ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_uint>(),
            false_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    if *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'p' as ::core::ffi::c_int
    {
        set_option_direct(
            kOptComments,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
            SID_NONE,
        );
        let mut next_prompt: pos_T = pos_T {
            lnum: (*buf).b_ml.ml_line_count,
            col: (*buf).b_prompt_start.mark.col,
            coladd: 0 as colnr_T,
        };
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_prompt_start;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = next_prompt;
        (*fmarkp__).fnum = 0 as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
    if (*win).w_status_height != 0 || global_stl_height() != 0 {
        (*win).w_redr_status = true_0 != 0;
        redraw_later(win, UPD_VALID as ::core::ffi::c_int);
    }
    (*buf).b_help = *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'h' as ::core::ffi::c_int;
    redraw_titles();
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn did_set_global_chars_option(
    mut win: *mut win_T,
    mut val: *mut ::core::ffi::c_char,
    mut what: CharsOption,
    mut opt_flags: ::core::ffi::c_int,
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut local_ptr: *mut *mut ::core::ffi::c_char =
        if what as ::core::ffi::c_uint == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint {
            &raw mut (*win).w_onebuf_opt.wo_lcs
        } else {
            &raw mut (*win).w_onebuf_opt.wo_fcs
        };
    errmsg = set_chars_option(
        win,
        val,
        what,
        **local_ptr as ::core::ffi::c_int == NUL
            || opt_flags & OPT_GLOBAL as ::core::ffi::c_int == 0,
        errbuf,
        errbuflen,
    );
    if !errmsg.is_null() {
        return errmsg;
    }
    if opt_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
        clear_string_option(local_ptr);
    }
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            let mut opt: *mut ::core::ffi::c_char = if what as ::core::ffi::c_uint
                == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*wp).w_onebuf_opt.wo_lcs
            } else {
                (*wp).w_onebuf_opt.wo_fcs
            };
            if *opt as ::core::ffi::c_int == NUL {
                set_chars_option(wp, opt, what, true_0 != 0, errbuf, errbuflen);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_chars_option(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if varp == &raw mut p_lcs {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kListchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut p_fcs {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kFillchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_lcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kListchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_fcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kFillchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_chars_option(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).oe_varp as *mut *mut ::core::ffi::c_char;
    let mut is_lcs: bool = varp == &raw mut p_lcs || varp == &raw mut (*curwin).w_onebuf_opt.wo_lcs;
    return expand_set_opt_generic(
        args,
        if is_lcs as ::core::ffi::c_int != 0 {
            Some(
                get_listchars_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            )
        } else {
            Some(
                get_fillchars_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            )
        },
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_cinoptions(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    parse_cino(buf);
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_colorcolumn(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return check_colorcolumn(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_cc {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_comments(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut errmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = *varp;
    while *s != 0 {
        while *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if vim_strchr(COM_ALL.as_ptr(), *s as uint8_t as ::core::ffi::c_int).is_null()
                && !ascii_isdigit(*s as ::core::ffi::c_int)
                && *s as ::core::ffi::c_int != '-' as ::core::ffi::c_int
            {
                errmsg = illegal_char(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    *s as uint8_t as ::core::ffi::c_int,
                );
                break;
            } else {
                s = s.offset(1);
            }
        }
        let c2rust_fresh4 = s;
        s = s.offset(1);
        if *c2rust_fresh4 as ::core::ffi::c_int == NUL {
            errmsg = b"E524: Missing colon\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == NUL
        {
            errmsg = b"E525: Zero length string\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        if !errmsg.is_null() {
            break;
        }
        while *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
        {
            if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                s = s.offset(1);
            }
            s = s.offset(1);
        }
        s = skip_to_option_part(s);
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_commentstring(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if **varp as ::core::ffi::c_int != NUL
        && strstr(*varp, b"%s\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return b"E537: 'commentstring' must be empty or contain %s\0".as_ptr()
            as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_complete(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut buffer: [::core::ffi::c_char; 512] = [0; 512];
    let mut char_before: uint8_t = NUL as uint8_t;
    let mut p: *mut ::core::ffi::c_char = *varp;
    while *p != 0 {
        memset(
            &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            LSIZE as ::core::ffi::c_int as size_t,
        );
        let mut buf_ptr: *mut ::core::ffi::c_char = &raw mut buffer as *mut ::core::ffi::c_char;
        let mut escape: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != 0
            && (*p as ::core::ffi::c_int != ',' as ::core::ffi::c_int || escape != 0)
            && buf_ptr
                < (&raw mut buffer as *mut ::core::ffi::c_char)
                    .offset(LSIZE as ::core::ffi::c_int as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
        {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int
            {
                escape = 1 as ::core::ffi::c_int;
                p = p.offset(1);
            } else {
                escape = 0 as ::core::ffi::c_int;
                let c2rust_fresh5 = buf_ptr;
                buf_ptr = buf_ptr.offset(1);
                *c2rust_fresh5 = *p;
            }
            p = p.offset(1);
        }
        *buf_ptr = NUL as ::core::ffi::c_char;
        if vim_strchr(
            b".wbuksid]tUfFo\0".as_ptr() as *const ::core::ffi::c_char,
            *(&raw mut buffer as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return illegal_char(
                (*args).os_errbuf,
                (*args).os_errbuflen,
                *(&raw mut buffer as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
            );
        }
        if vim_strchr(
            b"ksF\0".as_ptr() as *const ::core::ffi::c_char,
            *(&raw mut buffer as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
            && *(&raw mut buffer as *mut ::core::ffi::c_char)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != NUL
            && *(&raw mut buffer as *mut ::core::ffi::c_char)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '^' as ::core::ffi::c_int
        {
            char_before = *(&raw mut buffer as *mut ::core::ffi::c_char) as uint8_t;
        } else {
            let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            t = vim_strchr(
                &raw mut buffer as *mut ::core::ffi::c_char,
                '^' as ::core::ffi::c_int,
            );
            if !t.is_null() {
                let c2rust_fresh6 = t;
                t = t.offset(1);
                *c2rust_fresh6 = NUL as ::core::ffi::c_char;
                if *t == 0 {
                    char_before = '^' as uint8_t;
                } else {
                    while *t != 0 {
                        if !ascii_isdigit(*t as ::core::ffi::c_int) {
                            char_before = '^' as uint8_t;
                            break;
                        } else {
                            t = t.offset(1);
                        }
                    }
                }
            }
        }
        if char_before as ::core::ffi::c_int != NUL {
            if !(*args).os_errbuf.is_null() {
                return illegal_char_after_chr(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    char_before as ::core::ffi::c_int,
                );
            }
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
    }
    if set_cpt_callbacks(args) != OK {
        return illegal_char_after_chr(
            (*args).os_errbuf,
            (*args).os_errbuflen,
            'F' as ::core::ffi::c_int,
        );
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_completeitemalign(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = p_cia;
    let mut new_cia_flags: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut seen: [bool; 3] = [false_0 != 0, false_0 != 0, false_0 != 0];
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: [::core::ffi::c_char; 10] = [0; 10];
    while *p != 0 {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if count >= 3 as ::core::ffi::c_int {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if strequal(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if seen[CPT_ABBR as ::core::ffi::c_int as usize] {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_add(CPT_ABBR as ::core::ffi::c_int as ::core::ffi::c_uint);
            seen[CPT_ABBR as ::core::ffi::c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"kind\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if seen[CPT_KIND as ::core::ffi::c_int as usize] {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_add(CPT_KIND as ::core::ffi::c_int as ::core::ffi::c_uint);
            seen[CPT_KIND as ::core::ffi::c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"menu\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if seen[CPT_MENU as ::core::ffi::c_int as usize] {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_add(CPT_MENU as ::core::ffi::c_int as ::core::ffi::c_uint);
            seen[CPT_MENU as ::core::ffi::c_int as usize] = true_0 != 0;
            count += 1;
        } else {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    }
    if new_cia_flags == 0 as ::core::ffi::c_uint || count != 3 as ::core::ffi::c_int {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    cia_flags = new_cia_flags;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_completeopt(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut cot: *mut ::core::ffi::c_char = p_cot;
    let mut flags: *mut ::core::ffi::c_uint = &raw mut cot_flags;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        cot = (*buf).b_p_cot;
        flags = &raw mut (*buf).b_cot_flags;
    } else if (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
        (*buf).b_cot_flags = 0 as ::core::ffi::c_uint;
    }
    if opt_strings_flags(
        cot,
        &raw mut opt_cot_values as *mut *const ::core::ffi::c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_concealcursor(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        COCU_ALL.as_ptr() as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_concealcursor(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        COCU_ALL.as_ptr() as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_cpoptions(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        CPO_VI.as_ptr() as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_cpoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        CPO_VI.as_ptr() as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_cursorlineopt(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if **varp as ::core::ffi::c_int == NUL || fill_culopt_flags(*varp, win) != OK {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_diffanchors(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if diffanchors_changed((*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0) == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_diffopt(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    return if diffopt_changed() == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_diffopt(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut xp: *mut expand_T = (*args).oe_xp;
    if (*xp).xp_pattern > (*args).oe_set_arg
        && *(*xp).xp_pattern.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        let algo_len: size_t = strlen(b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg)
            >= algo_len as ::core::ffi::c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(algo_len as isize)),
                b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
                algo_len,
            ) == 0 as ::core::ffi::c_int
        {
            return expand_set_opt_string(
                args,
                &raw mut opt_dip_algorithm_values as *mut *const ::core::ffi::c_char,
                ::core::mem::size_of::<[*const ::core::ffi::c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const ::core::ffi::c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        let inline_len: size_t = strlen(b"inline:\0".as_ptr() as *const ::core::ffi::c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg)
            >= inline_len as ::core::ffi::c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(inline_len as isize)),
                b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
                inline_len,
            ) == 0 as ::core::ffi::c_int
        {
            return expand_set_opt_string(
                args,
                &raw mut opt_dip_inline_values as *mut *const ::core::ffi::c_char,
                ::core::mem::size_of::<[*const ::core::ffi::c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const ::core::ffi::c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        return FAIL;
    }
    return expand_set_str_generic(args, numMatches, matches);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_display(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    init_chartab();
    msg_grid_validate();
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_encoding(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    let mut gvarp: *mut *mut ::core::ffi::c_char = get_option_varp_scope_from(
        (*args).os_idx,
        OPT_GLOBAL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<win_T>(),
    ) as *mut *mut ::core::ffi::c_char;
    if gvarp == &raw mut p_fenc {
        if (*buf).b_p_ma == 0 && opt_flags != OPT_GLOBAL as ::core::ffi::c_int {
            return &raw const e_modifiable as *const ::core::ffi::c_char;
        }
        if !vim_strchr(*varp, ',' as ::core::ffi::c_int).is_null() {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        redraw_titles();
        ml_setflags(buf);
    }
    let mut p: *mut ::core::ffi::c_char = enc_canonize(*varp);
    xfree(*varp as *mut ::core::ffi::c_void);
    *varp = p;
    if varp == &raw mut p_enc {
        if strcmp(p_enc, b"utf-8\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as ::core::ffi::c_int
        {
            return &raw const e_unsupportedoption as *const ::core::ffi::c_char;
        }
        spell_reload();
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_encoding(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_generic(
        args,
        Some(
            get_encoding_name
                as unsafe extern "C" fn(
                    *mut expand_T,
                    ::core::ffi::c_int,
                ) -> *mut ::core::ffi::c_char,
        ),
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_eventignore(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if check_ei(*varp) == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
static mut expand_eiw: bool = false_0 != 0;
unsafe extern "C" fn get_eventignore_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut subtract: bool = *(*xp).xp_pattern as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
    if !subtract && idx == 0 as ::core::ffi::c_int {
        return b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut name: *mut ::core::ffi::c_char = get_event_name_no_group(
        xp,
        idx - 1 as ::core::ffi::c_int + subtract as ::core::ffi::c_int,
        expand_eiw,
    );
    if name.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    snprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        if subtract as ::core::ffi::c_int != 0 {
            b"-\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        name,
    );
    return &raw mut IObuff as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_eventignore(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    expand_eiw = (*args).oe_varp != &raw mut p_ei as *mut ::core::ffi::c_char;
    return expand_set_opt_generic(
        args,
        Some(
            get_eventignore_name
                as unsafe extern "C" fn(
                    *mut expand_T,
                    ::core::ffi::c_int,
                ) -> *mut ::core::ffi::c_char,
        ),
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_fileformat(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const ::core::ffi::c_char = (*args).os_oldval.string.data;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    if (*buf).b_p_ma == 0 && opt_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
        return &raw const e_modifiable as *const ::core::ffi::c_char;
    }
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    redraw_titles();
    ml_setflags(buf);
    if get_fileformat(buf) == EOL_MAC || *oldval as ::core::ffi::c_int == 'm' as ::core::ffi::c_int
    {
        redraw_buf_later(buf, UPD_NOT_VALID as ::core::ffi::c_int);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn get_fileformat_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx
        >= ::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return opt_ff_values[idx as usize] as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_filetype_or_syntax(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    (*args).os_value_changed =
        strcmp((*args).os_oldval.string.data, *varp) != 0 as ::core::ffi::c_int;
    (*args).os_value_checked = true_0 != 0;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_foldexpr(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    did_set_optexpr(args);
    if foldmethodIsExpr(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_foldignore(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if foldmethodIsIndent(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_foldmarker(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = vim_strchr(*varp, ',' as ::core::ffi::c_int);
    if p.is_null() {
        return &raw const e_comma_required as *const ::core::ffi::c_char;
    }
    if p == *varp || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    if foldmethodIsMarker(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_foldmethod(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    foldUpdateAll(win);
    if foldmethodIsDiff(win) {
        newFoldLevel();
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_formatoptions(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        FO_ALL.as_ptr() as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_formatoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        FO_ALL.as_ptr() as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_guicursor(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = parse_shape_opt(SHAPE_CURSOR);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active {
        redrawWinline(curwin, (*curwin).w_cursor.lnum);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_helpfile(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if didset_vim {
        vim_unsetenv_ext(b"VIM\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if didset_vimruntime {
        vim_unsetenv_ext(b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_helplang(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = p_hlg;
    while *s as ::core::ffi::c_int != NUL {
        if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || (*s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ',' as ::core::ffi::c_int
                || *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        s = s.offset(3 as ::core::ffi::c_int as isize);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_highlight(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if strcmp(*varp, HIGHLIGHT_INIT.as_ptr()) != 0 as ::core::ffi::c_int {
        return &raw const e_unsupportedoption as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_iconstring(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    return did_set_titleiconstring(args, STL_IN_ICON);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_inccommand(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if cmdpreview {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return did_set_str_generic(args);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_iskeyword(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if varp == &raw mut p_isk {
        if check_isopt(*varp) == FAIL {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    } else {
        return did_set_isopt(args);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_isopt(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if buf_init_chartab(buf, true_0 != 0) == FAIL {
        (*args).os_restore_chartab = true_0 != 0;
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_keymap(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    let mut secure_save: ::core::ffi::c_int = secure;
    secure = 0 as ::core::ffi::c_int;
    let mut errmsg: *const ::core::ffi::c_char = keymap_init();
    secure = secure_save;
    (*args).os_value_checked = true_0 != 0;
    if errmsg.is_null() {
        if *(*buf).b_p_keymap as ::core::ffi::c_int != NUL {
            (*buf).b_p_iminsert = B_IMODE_LMAP as OptInt;
            if (*buf).b_p_imsearch != B_IMODE_USE_INSERT as OptInt {
                (*buf).b_p_imsearch = B_IMODE_LMAP as OptInt;
            }
        } else {
            if (*buf).b_p_iminsert == B_IMODE_LMAP as OptInt {
                (*buf).b_p_iminsert = B_IMODE_NONE as OptInt;
            }
            if (*buf).b_p_imsearch == B_IMODE_LMAP as OptInt {
                (*buf).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
            }
        }
        if opt_flags & OPT_LOCAL as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            set_iminsert_global(buf);
            set_imsearch_global(buf);
        }
        status_redraw_buf(buf);
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_keymodel(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    km_stopsel = !vim_strchr(p_km, 'o' as ::core::ffi::c_int).is_null();
    km_startsel = !vim_strchr(p_km, 'a' as ::core::ffi::c_int).is_null();
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_lispoptions(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if **varp as ::core::ffi::c_int != NUL
        && strcmp(*varp, b"expr:0\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as ::core::ffi::c_int
        && strcmp(*varp, b"expr:1\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as ::core::ffi::c_int
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_matchpairs(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = *varp;
    while *p as ::core::ffi::c_int != NUL {
        let mut x2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut x3: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        p = p.offset(utfc_ptr2len(p) as isize);
        if *p as ::core::ffi::c_int != NUL {
            let c2rust_fresh9 = p;
            p = p.offset(1);
            x2 = *c2rust_fresh9 as ::core::ffi::c_uchar as ::core::ffi::c_int;
        }
        if *p as ::core::ffi::c_int != NUL {
            x3 = utf_ptr2char(p);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if x2 != ':' as ::core::ffi::c_int
            || x3 == -1 as ::core::ffi::c_int
            || *p as ::core::ffi::c_int != NUL
                && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
        {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_messagesopt(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if messagesopt_changed() == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_mkspellmem(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if spell_check_msm() != OK {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_mouse(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        MOUSE_ALL.as_ptr() as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_mouse(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        MOUSE_ALL.as_ptr() as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_mousescroll(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut vertical: OptInt = -1 as OptInt;
    let mut horizontal: OptInt = -1 as OptInt;
    let mut string: *mut ::core::ffi::c_char = p_mousescroll;
    loop {
        let mut end: *mut ::core::ffi::c_char = vim_strchr(string, ',' as ::core::ffi::c_int);
        let mut length: size_t = if !end.is_null() {
            end.offset_from(string) as size_t
        } else {
            strlen(string)
        };
        if length <= 4 as size_t {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        let mut direction: *mut OptInt = ::core::ptr::null_mut::<OptInt>();
        if memcmp(
            string as *const ::core::ffi::c_void,
            b"ver:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            direction = &raw mut vertical;
        } else if memcmp(
            string as *const ::core::ffi::c_void,
            b"hor:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            direction = &raw mut horizontal;
        } else {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if *direction != -1 as OptInt {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        let mut i: size_t = 4 as size_t;
        while i < length {
            if !ascii_isdigit(*string.offset(i as isize) as ::core::ffi::c_int) {
                return b"E5080: Digit expected\0".as_ptr() as *const ::core::ffi::c_char;
            }
            i = i.wrapping_add(1);
        }
        string = string.offset(4 as ::core::ffi::c_int as isize);
        *direction =
            getdigits_int(&raw mut string, false_0 != 0, -1 as ::core::ffi::c_int) as OptInt;
        if *direction == -1 as OptInt {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        if end.is_null() {
            break;
        }
        string = end.offset(1 as ::core::ffi::c_int as isize);
    }
    p_mousescroll_vert = if vertical == -1 as OptInt {
        MOUSESCROLL_VERT_DFLT as OptInt
    } else {
        vertical
    };
    p_mousescroll_hor = if horizontal == -1 as OptInt {
        MOUSESCROLL_HOR_DFLT as OptInt
    } else {
        horizontal
    };
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_optexpr(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut name: *mut ::core::ffi::c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_rulerformat(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    return did_set_statustabline_rulerformat(args, true_0 != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_selection(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_sessionoptions(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if ssop_flags & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && ssop_flags & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        let mut oldval: *const ::core::ffi::c_char = (*args).os_oldval.string.data;
        opt_strings_flags(
            oldval,
            &raw mut opt_ssop_values as *mut *const ::core::ffi::c_char,
            &raw mut ssop_flags,
            true_0 != 0,
        );
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_shada(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errbuf: *mut ::core::ffi::c_char = (*args).os_errbuf;
    let mut errbuflen: size_t = (*args).os_errbuflen;
    let mut s: *mut ::core::ffi::c_char = p_shada;
    while *s != 0 {
        if vim_strchr(
            b"!\"%'/:<@cfhnrs\0".as_ptr() as *const ::core::ffi::c_char,
            *s as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as ::core::ffi::c_int);
        }
        if *s as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            break;
        }
        if *s as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
            loop {
                s = s.offset(1);
                if !(*s as ::core::ffi::c_int != 0
                    && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int)
                {
                    break;
                }
            }
        } else if *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as ::core::ffi::c_int) {
                    break;
                }
            }
        } else if *s as ::core::ffi::c_int == '!' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
        {
            s = s.offset(1);
        } else {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as ::core::ffi::c_int) {
                    break;
                }
            }
            if !ascii_isdigit(*s.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int)
            {
                if !errbuf.is_null() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        gettext(b"E526: Missing number after <%s>\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        transchar_byte(*s.offset(-(1 as ::core::ffi::c_int as isize)) as uint8_t
                            as ::core::ffi::c_int),
                    );
                    return errbuf;
                } else {
                    return b"\0".as_ptr() as *const ::core::ffi::c_char;
                }
            }
        }
        if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            s = s.offset(1);
        } else if *s != 0 {
            if !errbuf.is_null() {
                return b"E527: Missing comma\0".as_ptr() as *const ::core::ffi::c_char;
            } else {
                return b"\0".as_ptr() as *const ::core::ffi::c_char;
            }
        }
    }
    if *p_shada as ::core::ffi::c_int != 0
        && get_shada_parameter('\'' as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
    {
        return b"E528: Must specify a ' value\0".as_ptr() as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_shellpipe_redir(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut seen: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*args).os_newval.string.data;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                return &raw const e_invalid_format_string_single_percent_s
                    as *const ::core::ffi::c_char;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '%' as ::core::ffi::c_int
            {
                p = p.offset(1);
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
            {
                if seen {
                    return &raw const e_invalid_format_string_single_percent_s
                        as *const ::core::ffi::c_char;
                }
                seen = true_0 != 0;
                p = p.offset(1);
            } else {
                return &raw const e_invalid_format_string_single_percent_s
                    as *const ::core::ffi::c_char;
            }
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_shortmess(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        &raw mut SHM_ALL as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_shortmess(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        &raw mut SHM_ALL as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_showbreak(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = *varp;
    while *s != 0 {
        if ptr2cells(s) != 1 as ::core::ffi::c_int {
            return &raw const e_showbreak_contains_unprintable_or_wide_character
                as *const ::core::ffi::c_char;
        }
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_showcmdloc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = did_set_str_generic(args);
    if errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_signcolumn(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut oldval: *const ::core::ffi::c_char = (*args).os_oldval.string.data;
    if check_signcolumn(
        *varp,
        (if varp == &raw mut (*win).w_onebuf_opt.wo_scl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        }),
    ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    if *oldval as ::core::ffi::c_int == 'n' as ::core::ffi::c_int
        && *oldval.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'u' as ::core::ffi::c_int
        || (*win).w_minscwidth == SCL_NUM
    {
        (*win).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spellcapcheck(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    return compile_cap_prog((*win).w_s);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spellfile(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if !valid_spellfile(*varp) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return did_set_spell_option();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spelllang(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if !valid_spelllang(*varp) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return did_set_spell_option();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spelloptions(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    let mut val: *const ::core::ffi::c_char = (*args).os_newval.string.data;
    if opt_flags & OPT_LOCAL as ::core::ffi::c_int == 0
        && opt_strings_flags(
            val,
            &raw mut opt_spo_values as *mut *const ::core::ffi::c_char,
            &raw mut spo_flags,
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    if opt_flags & OPT_GLOBAL as ::core::ffi::c_int == 0
        && opt_strings_flags(
            val,
            &raw mut opt_spo_values as *mut *const ::core::ffi::c_char,
            &raw mut (*(*win).w_s).b_p_spo_flags,
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spellsuggest(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if spell_check_sps() != OK {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_statuscolumn(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_statusline(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn did_set_statustabline_rulerformat(
    mut args: *mut optset_T,
    mut rulerformat: bool,
    mut statuscolumn: bool,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if rulerformat {
        ru_wid = 0 as ::core::ffi::c_int;
    } else if statuscolumn {
        (*win).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
    }
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = *varp;
    let mut is_stl: bool =
        (*args).os_idx as ::core::ffi::c_int == kOptStatusline as ::core::ffi::c_int;
    if is_stl as ::core::ffi::c_int != 0
        && ((*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int != 0
            || (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int == 0)
        && *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        xfree(*varp as *mut ::core::ffi::c_void);
        *varp = xstrdup(
            get_option_default((*args).os_idx, (*args).os_flags)
                .data
                .string
                .data,
        );
        s = *varp;
    }
    if is_stl as ::core::ffi::c_int != 0
        && !win.is_null()
        && (*win).w_floating as ::core::ffi::c_int != 0
    {
        win_config_float(win, (*win).w_config);
    }
    if rulerformat as ::core::ffi::c_int != 0
        && *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int
    {
        s = s.offset(1);
        if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            s = s.offset(1);
        }
        let mut wid: ::core::ffi::c_int =
            getdigits_int(&raw mut s, true_0 != 0, 0 as ::core::ffi::c_int);
        if wid != 0 && *s as ::core::ffi::c_int == '(' as ::core::ffi::c_int && {
            errmsg = check_stl_option(p_ruf);
            errmsg.is_null()
        } {
            ru_wid = wid;
        } else if *(*varp).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '!' as ::core::ffi::c_int
        {
            errmsg = check_stl_option(p_ruf);
        }
    } else if rulerformat as ::core::ffi::c_int != 0
        || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '%' as ::core::ffi::c_int
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '!' as ::core::ffi::c_int
    {
        errmsg = check_stl_option(s);
    }
    if rulerformat as ::core::ffi::c_int != 0 && errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_tabline(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_tagcase(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut opt_flags: ::core::ffi::c_int = (*args).os_flags;
    let mut flags: *mut ::core::ffi::c_uint = ::core::ptr::null_mut::<::core::ffi::c_uint>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if opt_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        p = (*buf).b_p_tc;
        flags = &raw mut (*buf).b_tc_flags;
    } else {
        p = p_tc;
        flags = &raw mut tc_flags;
    }
    if opt_flags & OPT_LOCAL as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int == NUL {
        *flags = 0 as ::core::ffi::c_uint;
    } else if opt_strings_flags(
        p,
        &raw mut opt_tc_values as *mut *const ::core::ffi::c_char,
        flags,
        false_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn did_set_titleiconstring(
    mut args: *mut optset_T,
    mut flagval: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if !vim_strchr(*varp, '%' as ::core::ffi::c_int).is_null() && check_stl_option(*varp).is_null()
    {
        stl_syntax |= flagval;
    } else {
        stl_syntax &= !flagval;
    }
    did_set_title();
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_titlestring(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    return did_set_titleiconstring(args, STL_IN_TITLE);
}
#[no_mangle]
pub unsafe extern "C" fn did_set_varsofttabstop(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if *(*varp).offset(0 as ::core::ffi::c_int as isize) == 0
        || *(*varp).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '0' as ::core::ffi::c_int
            && *(*varp).offset(1 as ::core::ffi::c_int as isize) == 0
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_p_vsts_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut cp: *mut ::core::ffi::c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as ::core::ffi::c_int) {
            if !(*cp as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                && cp > *varp
                && *cp.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int)
            {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vsts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vsts_array) {
        xfree(oldarray as *mut ::core::ffi::c_void);
    } else {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_vartabstop(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if *(*varp).offset(0 as ::core::ffi::c_int as isize) == 0
        || *(*varp).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '0' as ::core::ffi::c_int
            && *(*varp).offset(1 as ::core::ffi::c_int as isize) == 0
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_p_vts_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut cp: *mut ::core::ffi::c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as ::core::ffi::c_int) {
            if !(*cp as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                && cp > *varp
                && *cp.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int)
            {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vts_array) {
        xfree(oldarray as *mut ::core::ffi::c_void);
        if foldmethodIsIndent(win) {
            foldUpdateAll(win);
        }
    } else {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_verbosefile(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    verbose_stop();
    if *p_vfile as ::core::ffi::c_int != NUL && verbose_open() == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_virtualedit(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut ve: *mut ::core::ffi::c_char = p_ve;
    let mut flags: *mut ::core::ffi::c_uint = &raw mut ve_flags;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        ve = (*win).w_onebuf_opt.wo_ve;
        flags = &raw mut (*win).w_onebuf_opt.wo_ve_flags;
    }
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 && *ve as ::core::ffi::c_int == NUL {
        *flags = 0 as ::core::ffi::c_uint;
    } else if opt_strings_flags(
        ve,
        &raw mut opt_ve_values as *mut *const ::core::ffi::c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    } else if strcmp(ve, (*args).os_oldval.string.data) != 0 as ::core::ffi::c_int {
        validate_virtcol(win);
        coladvance(win, (*win).w_virtcol);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_whichwrap(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    return did_set_option_listflag(
        *varp,
        b"bshl<>[]~,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_whichwrap(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_listflag(
        args,
        WW_ALL.as_ptr() as *mut ::core::ffi::c_char,
        numMatches,
        matches,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_wildmode(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if check_opt_wim() == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_winbar(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn parse_border_opt(mut border_opt: *mut ::core::ffi::c_char) -> bool {
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: bool = true_0 != 0;
    if !parse_winborder(&raw mut fconfig, border_opt, &raw mut err) {
        result = false_0 != 0;
    }
    api_clear_error(&raw mut err);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_winborder(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if !parse_border_opt(p_winborder) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_pumborder(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    if !parse_border_opt(p_pumborder) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn did_set_winhighlight(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    if !parse_winhl_opt(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_winhl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_set_winhighlight(
    mut args: *mut optexpand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return expand_set_opt_generic(
        args,
        Some(
            get_highlight_name
                as unsafe extern "C" fn(
                    *mut expand_T,
                    ::core::ffi::c_int,
                ) -> *mut ::core::ffi::c_char,
        ),
        numMatches,
        matches,
    );
}
unsafe extern "C" fn opt_strings_flags(
    mut val: *const ::core::ffi::c_char,
    mut values: *mut *const ::core::ffi::c_char,
    mut flagp: *mut ::core::ffi::c_uint,
    mut list: bool,
) -> ::core::ffi::c_int {
    let mut new_flags: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut iter_one: bool = *val as ::core::ffi::c_int == NUL && !list;
    while *val as ::core::ffi::c_int != 0 || iter_one as ::core::ffi::c_int != 0 {
        let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        loop {
            if (*values.offset(i as isize)).is_null() {
                return FAIL;
            }
            let mut len: size_t = strlen(*values.offset(i as isize));
            if strncmp(*values.offset(i as isize), val, len) == 0 as ::core::ffi::c_int
                && (list as ::core::ffi::c_int != 0
                    && *val.offset(len as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                    || *val.offset(len as isize) as ::core::ffi::c_int == NUL)
            {
                val = val.offset(len.wrapping_add(
                    (*val.offset(len as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                        as ::core::ffi::c_int as size_t,
                ) as isize);
                '_c2rust_label: {
                    if (i as usize)
                        < ::core::mem::size_of::<::core::ffi::c_uint>().wrapping_mul(8 as usize)
                    {
                    } else {
                        __assert_fail(
                            b"i < sizeof(new_flags) * 8\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/optionstr.c\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            2192 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                new_flags |= (1 as ::core::ffi::c_uint) << i;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if iter_one {
            break;
        }
    }
    if !flagp.is_null() {
        *flagp = new_flags;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_ff_value(mut p: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return opt_strings_flags(
        p,
        &raw mut opt_ff_values as *mut *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_uint>(),
        false_0 != 0,
    );
}
static mut e_conflicts_with_value_of_listchars: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E834: Conflicts with value of 'listchars'\0",
    )
};
static mut e_conflicts_with_value_of_fillchars: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E835: Conflicts with value of 'fillchars'\0",
    )
};
unsafe extern "C" fn get_encoded_char_adv(mut p: *mut *const ::core::ffi::c_char) -> schar_T {
    let mut s: *const ::core::ffi::c_char = *p;
    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\\' as ::core::ffi::c_int
        && (*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'x' as ::core::ffi::c_int
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'u' as ::core::ffi::c_int
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'U' as ::core::ffi::c_int)
    {
        let mut num: int64_t = 0 as int64_t;
        let mut bytes: ::core::ffi::c_int = if *s.offset(1 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 'x' as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'u' as ::core::ffi::c_int
        {
            2 as ::core::ffi::c_int
        } else {
            4 as ::core::ffi::c_int
        };
        while bytes > 0 as ::core::ffi::c_int {
            *p = (*p).offset(2 as ::core::ffi::c_int as isize);
            let mut n: ::core::ffi::c_int = hexhex2nr(*p);
            if n < 0 as ::core::ffi::c_int {
                return 0 as schar_T;
            }
            num = num * 256 as int64_t + n as int64_t;
            bytes -= 1;
        }
        *p = (*p).offset(2 as ::core::ffi::c_int as isize);
        return if char2cells(num as ::core::ffi::c_int) > 1 as ::core::ffi::c_int {
            0 as schar_T
        } else {
            schar_from_char(num as ::core::ffi::c_int)
        };
    }
    let mut clen: ::core::ffi::c_int = utfc_ptr2len(s);
    let mut firstc: ::core::ffi::c_int = 0;
    let mut c: schar_T = utfc_ptr2schar(s, &raw mut firstc);
    *p = (*p).offset(clen as isize);
    return if clen == 1 as ::core::ffi::c_int && firstc > 127 as ::core::ffi::c_int
        || char2cells(firstc) > 1 as ::core::ffi::c_int
    {
        0 as schar_T
    } else {
        c
    };
}
static mut fcs_chars: fcs_chars_T = fcs_chars_T {
    stl: 0,
    stlnc: 0,
    wbr: 0,
    horiz: 0,
    horizup: 0,
    horizdown: 0,
    vert: 0,
    vertleft: 0,
    vertright: 0,
    verthoriz: 0,
    fold: 0,
    foldopen: 0,
    foldclosed: 0,
    foldsep: 0,
    foldinner: 0,
    diff: 0,
    msgsep: 0,
    eob: 0,
    lastline: 0,
    trunc: 0,
    truncrl: 0,
};
static mut fcs_tab: [chars_tab; 21] = [chars_tab {
    cp: ::core::ptr::null_mut::<schar_T>(),
    name: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    def: ::core::ptr::null::<::core::ffi::c_char>(),
    fallback: ::core::ptr::null::<::core::ffi::c_char>(),
}; 21];
static mut lcs_chars: lcs_chars_T = lcs_chars_T {
    eol: 0,
    ext: 0,
    prec: 0,
    nbsp: 0,
    space: 0,
    tab1: 0,
    tab2: 0,
    tab3: 0,
    leadtab1: 0,
    leadtab2: 0,
    leadtab3: 0,
    lead: 0,
    trail: 0,
    multispace: ::core::ptr::null_mut::<schar_T>(),
    leadmultispace: ::core::ptr::null_mut::<schar_T>(),
    conceal: 0,
};
static mut lcs_tab: [chars_tab; 12] = [chars_tab {
    cp: ::core::ptr::null_mut::<schar_T>(),
    name: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    def: ::core::ptr::null::<::core::ffi::c_char>(),
    fallback: ::core::ptr::null::<::core::ffi::c_char>(),
}; 12];
unsafe extern "C" fn field_value_err(
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut field: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    vim_snprintf(errbuf, errbuflen, gettext(fmt), field);
    return errbuf;
}
#[no_mangle]
pub unsafe extern "C" fn set_chars_option(
    mut wp: *mut win_T,
    mut value: *const ::core::ffi::c_char,
    mut what: CharsOption,
    mut apply: bool,
    mut errbuf: *mut ::core::ffi::c_char,
    mut errbuflen: size_t,
) -> *const ::core::ffi::c_char {
    let mut last_multispace: *const ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>();
    let mut last_lmultispace: *const ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>();
    let mut multispace_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lead_multispace_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tab: *const chars_tab = ::core::ptr::null::<chars_tab>();
    let mut entries: ::core::ffi::c_int = 0;
    if what as ::core::ffi::c_uint == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint {
        tab = &raw const lcs_tab as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 12]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 12]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int;
        if *(*wp)
            .w_onebuf_opt
            .wo_lcs
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == NUL
        {
            value = p_lcs;
        }
    } else {
        tab = &raw const fcs_tab as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 21]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 21]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int;
        if *(*wp)
            .w_onebuf_opt
            .wo_fcs
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == NUL
        {
            value = p_fcs;
        }
    }
    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while round
        <= (if apply as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })
    {
        let mut has_tab: bool = false_0 != 0;
        let mut has_leadtab: bool = false_0 != 0;
        if round > 0 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < entries {
                if !(*tab.offset(i as isize)).cp.is_null() {
                    *(*tab.offset(i as isize)).cp = schar_from_str(
                        if !(*tab.offset(i as isize)).def.is_null()
                            && ptr2cells((*tab.offset(i as isize)).def) == 1 as ::core::ffi::c_int
                        {
                            (*tab.offset(i as isize)).def
                        } else {
                            (*tab.offset(i as isize)).fallback
                        },
                    );
                }
                i += 1;
            }
            if what as ::core::ffi::c_uint
                == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lcs_chars.tab1 = NUL as schar_T;
                lcs_chars.tab3 = NUL as schar_T;
                lcs_chars.leadtab1 = NUL as schar_T;
                lcs_chars.leadtab3 = NUL as schar_T;
                if multispace_len > 0 as ::core::ffi::c_int {
                    lcs_chars.multispace = xmalloc(
                        (multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *lcs_chars.multispace.offset(multispace_len as isize) = NUL as schar_T;
                } else {
                    lcs_chars.multispace = ::core::ptr::null_mut::<schar_T>();
                }
                if lead_multispace_len > 0 as ::core::ffi::c_int {
                    lcs_chars.leadmultispace = xmalloc(
                        (lead_multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *lcs_chars
                        .leadmultispace
                        .offset(lead_multispace_len as isize) = NUL as schar_T;
                } else {
                    lcs_chars.leadmultispace = ::core::ptr::null_mut::<schar_T>();
                }
            }
        }
        let mut p: *const ::core::ffi::c_char = value;
        while *p != 0 {
            let mut i_0: ::core::ffi::c_int = 0;
            i_0 = 0 as ::core::ffi::c_int;
            while i_0 < entries {
                if !(strncmp(
                    p,
                    (*tab.offset(i_0 as isize)).name.data,
                    (*tab.offset(i_0 as isize)).name.size,
                ) == 0 as ::core::ffi::c_int
                    && *p.offset((*tab.offset(i_0 as isize)).name.size as isize)
                        as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int)
                {
                    i_0 += 1;
                } else {
                    let mut s: *const ::core::ffi::c_char = p
                        .offset((*tab.offset(i_0 as isize)).name.size as isize)
                        .offset(1 as ::core::ffi::c_int as isize);
                    if what as ::core::ffi::c_uint
                        == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"multispace\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        if round == 0 as ::core::ffi::c_int {
                            last_multispace = p;
                            multispace_len = 0 as ::core::ffi::c_int;
                            while *s as ::core::ffi::c_int != NUL
                                && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            {
                                let mut c1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        &raw const e_wrong_character_width_for_field_str
                                            as *const ::core::ffi::c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                multispace_len += 1;
                            }
                            if multispace_len == 0 as ::core::ffi::c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    &raw const e_wrong_number_of_characters_for_field_str
                                        as *const ::core::ffi::c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while *s as ::core::ffi::c_int != NUL
                                && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            {
                                let mut c1_0: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_multispace {
                                    let c2rust_fresh2 = multispace_pos;
                                    multispace_pos = multispace_pos + 1;
                                    *lcs_chars.multispace.offset(c2rust_fresh2 as isize) = c1_0;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else if what as ::core::ffi::c_uint
                        == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"leadmultispace\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        if round == 0 as ::core::ffi::c_int {
                            last_lmultispace = p;
                            lead_multispace_len = 0 as ::core::ffi::c_int;
                            while *s as ::core::ffi::c_int != NUL
                                && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            {
                                let mut c1_1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1_1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        &raw const e_wrong_character_width_for_field_str
                                            as *const ::core::ffi::c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                lead_multispace_len += 1;
                            }
                            if lead_multispace_len == 0 as ::core::ffi::c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    &raw const e_wrong_number_of_characters_for_field_str
                                        as *const ::core::ffi::c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while *s as ::core::ffi::c_int != NUL
                                && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            {
                                let mut c1_2: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_lmultispace {
                                    let c2rust_fresh3 = multispace_pos_0;
                                    multispace_pos_0 = multispace_pos_0 + 1;
                                    *lcs_chars.leadmultispace.offset(c2rust_fresh3 as isize) = c1_2;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else {
                        if *s as ::core::ffi::c_int == NUL {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                &raw const e_wrong_number_of_characters_for_field_str
                                    as *const ::core::ffi::c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c1_3: schar_T = get_encoded_char_adv(&raw mut s);
                        if c1_3 == 0 as schar_T {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                &raw const e_wrong_character_width_for_field_str
                                    as *const ::core::ffi::c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c2: schar_T = 0 as schar_T;
                        let mut c3: schar_T = 0 as schar_T;
                        if (*tab.offset(i_0 as isize)).cp == &raw mut lcs_chars.tab2
                            || (*tab.offset(i_0 as isize)).cp == &raw mut lcs_chars.leadtab2
                        {
                            if *s as ::core::ffi::c_int == NUL {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    &raw const e_wrong_number_of_characters_for_field_str
                                        as *const ::core::ffi::c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            c2 = get_encoded_char_adv(&raw mut s);
                            if c2 == 0 as schar_T {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    &raw const e_wrong_character_width_for_field_str
                                        as *const ::core::ffi::c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            if !(*s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                                || *s as ::core::ffi::c_int == NUL)
                            {
                                c3 = get_encoded_char_adv(&raw mut s);
                                if c3 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        &raw const e_wrong_character_width_for_field_str
                                            as *const ::core::ffi::c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                            }
                            if (*tab.offset(i_0 as isize)).cp == &raw mut lcs_chars.tab2 {
                                has_tab = true_0 != 0;
                            } else {
                                has_leadtab = true_0 != 0;
                            }
                        }
                        if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                            || *s as ::core::ffi::c_int == NUL
                        {
                            if round > 0 as ::core::ffi::c_int {
                                if (*tab.offset(i_0 as isize)).cp == &raw mut lcs_chars.tab2 {
                                    lcs_chars.tab1 = c1_3;
                                    lcs_chars.tab2 = c2;
                                    lcs_chars.tab3 = c3;
                                } else if (*tab.offset(i_0 as isize)).cp
                                    == &raw mut lcs_chars.leadtab2
                                {
                                    lcs_chars.leadtab1 = c1_3;
                                    lcs_chars.leadtab2 = c2;
                                    lcs_chars.leadtab3 = c3;
                                } else if !(*tab.offset(i_0 as isize)).cp.is_null() {
                                    *(*tab.offset(i_0 as isize)).cp = c1_3;
                                }
                            }
                            p = s;
                            break;
                        } else {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                &raw const e_wrong_number_of_characters_for_field_str
                                    as *const ::core::ffi::c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                    }
                }
            }
            if i_0 == entries {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
        if what as ::core::ffi::c_uint == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint
            && has_leadtab as ::core::ffi::c_int != 0
            && !has_tab
        {
            return &raw const e_leadtab_requires_tab as *const ::core::ffi::c_char;
        }
        round += 1;
    }
    if apply {
        if what as ::core::ffi::c_uint == kListchars as ::core::ffi::c_int as ::core::ffi::c_uint {
            xfree((*wp).w_p_lcs_chars.multispace as *mut ::core::ffi::c_void);
            xfree((*wp).w_p_lcs_chars.leadmultispace as *mut ::core::ffi::c_void);
            (*wp).w_p_lcs_chars = lcs_chars;
        } else {
            (*wp).w_p_fcs_chars = fcs_chars;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn get_fillchars_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 21]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 21]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return fcs_tab[idx as usize].name.data;
}
#[no_mangle]
pub unsafe extern "C" fn get_listchars_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 12]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 12]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return lcs_tab[idx as usize].name.data;
}
#[no_mangle]
pub unsafe extern "C" fn check_chars_options() -> *const ::core::ffi::c_char {
    if !set_chars_option(
        curwin,
        p_lcs,
        kListchars,
        false_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return &raw const e_conflicts_with_value_of_listchars as *const ::core::ffi::c_char;
    }
    if !set_chars_option(
        curwin,
        p_fcs,
        kFillchars,
        false_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return &raw const e_conflicts_with_value_of_fillchars as *const ::core::ffi::c_char;
    }
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_lcs,
                kListchars,
                true_0 != 0,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return &raw const e_conflicts_with_value_of_listchars
                    as *const ::core::ffi::c_char;
            }
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_fcs,
                kFillchars,
                true_0 != 0,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return &raw const e_conflicts_with_value_of_fillchars
                    as *const ::core::ffi::c_char;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    fcs_tab = [
        chars_tab {
            cp: &raw mut fcs_chars.stl,
            name: String_0 {
                data: b"stl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.stlnc,
            name: String_0 {
                data: b"stlnc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.wbr,
            name: String_0 {
                data: b"wbr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.horiz,
            name: String_0 {
                data: b"horiz\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x80\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"-\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.horizup,
            name: String_0 {
                data: b"horizup\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xB4\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"-\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.horizdown,
            name: String_0 {
                data: b"horizdown\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xAC\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"-\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.vert,
            name: String_0 {
                data: b"vert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x82\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"|\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.vertleft,
            name: String_0 {
                data: b"vertleft\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xA4\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"|\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.vertright,
            name: String_0 {
                data: b"vertright\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x9C\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"|\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.verthoriz,
            name: String_0 {
                data: b"verthoriz\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xBC\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"+\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.fold,
            name: String_0 {
                data: b"fold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"\xC2\xB7\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"-\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.foldopen,
            name: String_0 {
                data: b"foldopen\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"-\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.foldclosed,
            name: String_0 {
                data: b"foldclose\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"+\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.foldsep,
            name: String_0 {
                data: b"foldsep\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x82\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: b"|\0".as_ptr() as *const ::core::ffi::c_char,
        },
        chars_tab {
            cp: &raw mut fcs_chars.foldinner,
            name: String_0 {
                data: b"foldinner\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.diff,
            name: String_0 {
                data: b"diff\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"-\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.msgsep,
            name: String_0 {
                data: b"msgsep\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.eob,
            name: String_0 {
                data: b"eob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b"~\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.lastline,
            name: String_0 {
                data: b"lastline\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"@\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.trunc,
            name: String_0 {
                data: b"trunc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b">\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut fcs_chars.truncrl,
            name: String_0 {
                data: b"truncrl\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"<\0".as_ptr() as *const ::core::ffi::c_char,
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    ];
    lcs_tab = [
        chars_tab {
            cp: &raw mut lcs_chars.eol,
            name: String_0 {
                data: b"eol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.ext,
            name: String_0 {
                data: b"extends\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.nbsp,
            name: String_0 {
                data: b"nbsp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.prec,
            name: String_0 {
                data: b"precedes\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.space,
            name: String_0 {
                data: b"space\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.tab2,
            name: String_0 {
                data: b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.leadtab2,
            name: String_0 {
                data: b"leadtab\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.lead,
            name: String_0 {
                data: b"lead\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.trail,
            name: String_0 {
                data: b"trail\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: &raw mut lcs_chars.conceal,
            name: String_0 {
                data: b"conceal\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: ::core::ptr::null_mut::<schar_T>(),
            name: String_0 {
                data: b"multispace\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        chars_tab {
            cp: ::core::ptr::null_mut::<schar_T>(),
            name: String_0 {
                data: b"leadmultispace\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<::core::ffi::c_char>(),
            fallback: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
