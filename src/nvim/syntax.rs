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
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut empty_string_option: [::core::ffi::c_char; 0];
    static mut p_cpo: *mut ::core::ffi::c_char;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strsave_up(string: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_strnsave_up(string: *const ::core::ffi::c_char, len: size_t)
        -> *mut ::core::ffi::c_char;
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
    fn buf_init_chartab(buf: *mut buf_T, global: bool) -> ::core::ffi::c_int;
    fn str_foldcase(
        str: *mut ::core::ffi::c_char,
        orglen: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_char,
        buflen: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_iswordp_buf(p: *const ::core::ffi::c_char, buf: *mut buf_T) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getdigits_int32(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: int32_t) -> int32_t;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_nogroup: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    fn set_internal_string_var(name: *const ::core::ffi::c_char, value: *mut ::core::ffi::c_char);
    fn do_unlet(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn get_var_value(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn expand_filename(
        eap: *mut exarg_T,
        cmdlinep: *mut *mut ::core::ffi::c_char,
        errormsgp: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn separate_nextcmd(eap: *mut exarg_T);
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn find_nextcmd(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn check_nextcmd(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn foldmethodIsSyntax(wp: *mut win_T) -> bool;
    fn foldUpdateAll(win: *mut win_T);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_set_growsize(gap: *mut garray_T, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    static mut hash_removed: ::core::ffi::c_char;
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut emsg_skip: ::core::ffi::c_int;
    static mut include_none: ::core::ffi::c_int;
    static mut include_default: ::core::ffi::c_int;
    static mut include_link: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut got_int: bool;
    static mut reg_do_extmatch: ::core::ffi::c_int;
    static mut re_extmatch_in: *mut reg_extmatch_T;
    static mut re_extmatch_out: *mut reg_extmatch_T;
    static mut display_tick: disptick_T;
    fn hash_init(ht: *mut hashtab_T);
    fn hash_clear(ht: *mut hashtab_T);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_lookup(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        hash: hash_T,
    ) -> *mut hashitem_T;
    fn hash_add_item(
        ht: *mut hashtab_T,
        hi: *mut hashitem_T,
        key: *mut ::core::ffi::c_char,
        hash: hash_T,
    );
    fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T);
    fn hash_lock(ht: *mut hashtab_T);
    fn hash_unlock(ht: *mut hashtab_T);
    fn hash_hash(key: *const ::core::ffi::c_char) -> hash_T;
    fn highlight_num_groups() -> ::core::ffi::c_int;
    fn highlight_group_name(id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn highlight_link_id(id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn init_highlight(both: bool, reset: bool);
    fn syn_list_header(
        did_header: bool,
        outlen: ::core::ffi::c_int,
        id: ::core::ffi::c_int,
        force_newline: bool,
    ) -> bool;
    fn syn_name2id(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn syn_name2id_len(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn find_start_comment(ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outnum(n: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_outtrans_len(
        msgstr: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_advance(col: ::core::ffi::c_int);
    fn clear_string_option(pp: *mut *mut ::core::ffi::c_char);
    fn line_breakcheck();
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn profile_start() -> proftime_T;
    fn profile_end(tm: proftime_T) -> proftime_T;
    fn profile_msg(tm: proftime_T) -> *const ::core::ffi::c_char;
    fn profile_zero() -> proftime_T;
    fn profile_divide(tm: proftime_T, count: ::core::ffi::c_int) -> proftime_T;
    fn profile_add(tm1: proftime_T, tm2: proftime_T) -> proftime_T;
    fn profile_cmp(tm1: proftime_T, tm2: proftime_T) -> ::core::ffi::c_int;
    fn skip_regexp(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ref_extmatch(em: *mut reg_extmatch_T) -> *mut reg_extmatch_T;
    fn unref_extmatch(em: *mut reg_extmatch_T);
    fn vim_regcomp_had_eol() -> ::core::ffi::c_int;
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
    fn source_runtime(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn do_source(
        fname: *mut ::core::ffi::c_char,
        check_other: bool,
        is_vimrc: ::core::ffi::c_int,
        ret_sid: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_14 = 76;
pub const HLF_PRE: C2Rust_Unnamed_14 = 75;
pub const HLF_OK: C2Rust_Unnamed_14 = 74;
pub const HLF_SO: C2Rust_Unnamed_14 = 73;
pub const HLF_SE: C2Rust_Unnamed_14 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_14 = 71;
pub const HLF_TS: C2Rust_Unnamed_14 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_14 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_14 = 68;
pub const HLF_CU: C2Rust_Unnamed_14 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_14 = 66;
pub const HLF_WBR: C2Rust_Unnamed_14 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_14 = 64;
pub const HLF_MSG: C2Rust_Unnamed_14 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_14 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_14 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_14 = 60;
pub const HLF_0: C2Rust_Unnamed_14 = 59;
pub const HLF_QFL: C2Rust_Unnamed_14 = 58;
pub const HLF_MC: C2Rust_Unnamed_14 = 57;
pub const HLF_CUL: C2Rust_Unnamed_14 = 56;
pub const HLF_CUC: C2Rust_Unnamed_14 = 55;
pub const HLF_TPF: C2Rust_Unnamed_14 = 54;
pub const HLF_TPS: C2Rust_Unnamed_14 = 53;
pub const HLF_TP: C2Rust_Unnamed_14 = 52;
pub const HLF_PBR: C2Rust_Unnamed_14 = 51;
pub const HLF_PST: C2Rust_Unnamed_14 = 50;
pub const HLF_PSB: C2Rust_Unnamed_14 = 49;
pub const HLF_PSX: C2Rust_Unnamed_14 = 48;
pub const HLF_PNX: C2Rust_Unnamed_14 = 47;
pub const HLF_PSK: C2Rust_Unnamed_14 = 46;
pub const HLF_PNK: C2Rust_Unnamed_14 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_14 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_14 = 43;
pub const HLF_PSI: C2Rust_Unnamed_14 = 42;
pub const HLF_PNI: C2Rust_Unnamed_14 = 41;
pub const HLF_SPL: C2Rust_Unnamed_14 = 40;
pub const HLF_SPR: C2Rust_Unnamed_14 = 39;
pub const HLF_SPC: C2Rust_Unnamed_14 = 38;
pub const HLF_SPB: C2Rust_Unnamed_14 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_14 = 36;
pub const HLF_SC: C2Rust_Unnamed_14 = 35;
pub const HLF_TXA: C2Rust_Unnamed_14 = 34;
pub const HLF_TXD: C2Rust_Unnamed_14 = 33;
pub const HLF_DED: C2Rust_Unnamed_14 = 32;
pub const HLF_CHD: C2Rust_Unnamed_14 = 31;
pub const HLF_ADD: C2Rust_Unnamed_14 = 30;
pub const HLF_FC: C2Rust_Unnamed_14 = 29;
pub const HLF_FL: C2Rust_Unnamed_14 = 28;
pub const HLF_WM: C2Rust_Unnamed_14 = 27;
pub const HLF_W: C2Rust_Unnamed_14 = 26;
pub const HLF_VNC: C2Rust_Unnamed_14 = 25;
pub const HLF_V: C2Rust_Unnamed_14 = 24;
pub const HLF_T: C2Rust_Unnamed_14 = 23;
pub const HLF_VSP: C2Rust_Unnamed_14 = 22;
pub const HLF_C: C2Rust_Unnamed_14 = 21;
pub const HLF_SNC: C2Rust_Unnamed_14 = 20;
pub const HLF_S: C2Rust_Unnamed_14 = 19;
pub const HLF_R: C2Rust_Unnamed_14 = 18;
pub const HLF_CLF: C2Rust_Unnamed_14 = 17;
pub const HLF_CLS: C2Rust_Unnamed_14 = 16;
pub const HLF_CLN: C2Rust_Unnamed_14 = 15;
pub const HLF_LNB: C2Rust_Unnamed_14 = 14;
pub const HLF_LNA: C2Rust_Unnamed_14 = 13;
pub const HLF_N: C2Rust_Unnamed_14 = 12;
pub const HLF_CM: C2Rust_Unnamed_14 = 11;
pub const HLF_M: C2Rust_Unnamed_14 = 10;
pub const HLF_LC: C2Rust_Unnamed_14 = 9;
pub const HLF_L: C2Rust_Unnamed_14 = 8;
pub const HLF_I: C2Rust_Unnamed_14 = 7;
pub const HLF_E: C2Rust_Unnamed_14 = 6;
pub const HLF_D: C2Rust_Unnamed_14 = 5;
pub const HLF_AT: C2Rust_Unnamed_14 = 4;
pub const HLF_TERM: C2Rust_Unnamed_14 = 3;
pub const HLF_EOB: C2Rust_Unnamed_14 = 2;
pub const HLF_8: C2Rust_Unnamed_14 = 1;
pub const HLF_NONE: C2Rust_Unnamed_14 = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const EXPAND_BUF_LEN: C2Rust_Unnamed_15 = 256;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_16 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_16 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_16 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_16 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_16 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_16 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_16 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_16 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_16 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_16 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_16 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_16 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_16 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_16 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_16 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_16 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_16 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_16 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_16 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_16 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_16 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_16 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_16 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_16 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_16 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_16 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_16 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_16 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_16 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_16 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_16 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_16 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_16 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_16 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_16 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_16 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_16 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_16 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_16 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_16 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_16 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_16 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_16 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_16 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_16 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_16 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_16 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_16 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_16 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_16 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_16 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_16 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_16 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_16 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_16 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_16 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_16 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NSUBEXP: C2Rust_Unnamed_17 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_18,
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type CMD_index = ::core::ffi::c_int;
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
pub type cmdidx_T = CMD_index;
pub type cmd_addr_T = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exarg {
    pub arg: *mut ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub arglens: *mut size_t,
    pub argc: size_t,
    pub nextcmd: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub cmdlinep: *mut *mut ::core::ffi::c_char,
    pub cmdline_tofree: *mut ::core::ffi::c_char,
    pub cmdidx: cmdidx_T,
    pub argt: uint32_t,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: cmd_addr_T,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: linenr_T,
    pub append: ::core::ffi::c_int,
    pub usefilter: ::core::ffi::c_int,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: ::core::ffi::c_int,
    pub mkdir_p: ::core::ffi::c_int,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    pub errmsg: *mut ::core::ffi::c_char,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut cstack_T,
}
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
pub type exarg_T = exarg;
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
pub struct keyvalue_T {
    pub key: ::core::ffi::c_int,
    pub value: *mut ::core::ffi::c_char,
    pub length: size_t,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_19 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_19 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_19 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_19 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_19 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_19 = 20;
pub const UPD_VALID: C2Rust_Unnamed_19 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sp_syn {
    pub inc_tag: ::core::ffi::c_int,
    pub id: int16_t,
    pub cont_in_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyentry {
    pub ke_next: *mut keyentry_T,
    pub k_syn: sp_syn,
    pub next_list: *mut int16_t,
    pub flags: ::core::ffi::c_int,
    pub k_char: ::core::ffi::c_int,
    pub keyword: [::core::ffi::c_char; 0],
}
pub type keyentry_T = keyentry;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAX_HL_ID: C2Rust_Unnamed_20 = 20000;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const DOSO_VIMRC: C2Rust_Unnamed_21 = 1;
pub const DOSO_NONE: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_22 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_22 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_22 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_22 = 32;
pub const DIP_OPT: C2Rust_Unnamed_22 = 16;
pub const DIP_START: C2Rust_Unnamed_22 = 8;
pub const DIP_ERR: C2Rust_Unnamed_22 = 4;
pub const DIP_DIR: C2Rust_Unnamed_22 = 2;
pub const DIP_ALL: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const HL_INCLUDED_TOPLEVEL: C2Rust_Unnamed_23 = 524288;
pub const HL_CONCEALENDS: C2Rust_Unnamed_23 = 262144;
pub const HL_CONCEAL: C2Rust_Unnamed_23 = 131072;
pub const HL_TRANS_CONT: C2Rust_Unnamed_23 = 65536;
pub const HL_MATCHCONT: C2Rust_Unnamed_23 = 32768;
pub const HL_EXTEND: C2Rust_Unnamed_23 = 16384;
pub const HL_FOLD: C2Rust_Unnamed_23 = 8192;
pub const HL_DISPLAY: C2Rust_Unnamed_23 = 4096;
pub const HL_EXCLUDENL: C2Rust_Unnamed_23 = 2048;
pub const HL_KEEPEND: C2Rust_Unnamed_23 = 1024;
pub const HL_SKIPEMPTY: C2Rust_Unnamed_23 = 512;
pub const HL_SKIPWHITE: C2Rust_Unnamed_23 = 256;
pub const HL_SKIPNL: C2Rust_Unnamed_23 = 128;
pub const HL_MATCH: C2Rust_Unnamed_23 = 64;
pub const HL_SYNC_THERE: C2Rust_Unnamed_23 = 32;
pub const HL_SYNC_HERE: C2Rust_Unnamed_23 = 16;
pub const HL_HAS_EOL: C2Rust_Unnamed_23 = 8;
pub const HL_ONELINE: C2Rust_Unnamed_23 = 4;
pub const HL_TRANSP: C2Rust_Unnamed_23 = 2;
pub const HL_CONTAINED: C2Rust_Unnamed_23 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stateitem_T {
    pub si_idx: ::core::ffi::c_int,
    pub si_id: ::core::ffi::c_int,
    pub si_trans_id: ::core::ffi::c_int,
    pub si_m_lnum: ::core::ffi::c_int,
    pub si_m_startcol: ::core::ffi::c_int,
    pub si_m_endpos: lpos_T,
    pub si_h_startpos: lpos_T,
    pub si_h_endpos: lpos_T,
    pub si_eoe_pos: lpos_T,
    pub si_end_idx: ::core::ffi::c_int,
    pub si_ends: ::core::ffi::c_int,
    pub si_attr: ::core::ffi::c_int,
    pub si_flags: ::core::ffi::c_int,
    pub si_seqnr: ::core::ffi::c_int,
    pub si_cchar: ::core::ffi::c_int,
    pub si_cont_list: *mut int16_t,
    pub si_next_list: *mut int16_t,
    pub si_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synpat_T {
    pub sp_type: ::core::ffi::c_char,
    pub sp_syncing: bool,
    pub sp_syn_match_id: int16_t,
    pub sp_off_flags: int16_t,
    pub sp_offsets: [::core::ffi::c_int; 7],
    pub sp_flags: ::core::ffi::c_int,
    pub sp_cchar: ::core::ffi::c_int,
    pub sp_ic: ::core::ffi::c_int,
    pub sp_sync_idx: ::core::ffi::c_int,
    pub sp_line_id: ::core::ffi::c_int,
    pub sp_startcol: ::core::ffi::c_int,
    pub sp_cont_list: *mut int16_t,
    pub sp_next_list: *mut int16_t,
    pub sp_syn: sp_syn,
    pub sp_pattern: *mut ::core::ffi::c_char,
    pub sp_prog: *mut regprog_T,
    pub sp_time: syn_time_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_cluster_T {
    pub scl_name: *mut ::core::ffi::c_char,
    pub scl_name_u: *mut ::core::ffi::c_char,
    pub scl_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct subcommand {
    pub name: *mut ::core::ffi::c_char,
    pub func: Option<unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_opt_arg_T {
    pub flags: ::core::ffi::c_int,
    pub keyword: bool,
    pub sync_idx: *mut ::core::ffi::c_int,
    pub has_cont_list: bool,
    pub cont_list: *mut int16_t,
    pub cont_in_list: *mut int16_t,
    pub next_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pat_ptr {
    pub pp_synp: *mut synpat_T,
    pub pp_matchgroup_id: ::core::ffi::c_int,
    pub pp_next: *mut pat_ptr,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct flag {
    pub name: *mut ::core::ffi::c_char,
    pub argtype: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const EXP_CLUSTER: C2Rust_Unnamed_24 = 4;
pub const EXP_SYNC: C2Rust_Unnamed_24 = 3;
pub const EXP_SPELL: C2Rust_Unnamed_24 = 2;
pub const EXP_CASE: C2Rust_Unnamed_24 = 1;
pub const EXP_SUBCMD: C2Rust_Unnamed_24 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct time_entry_T {
    pub total: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
    pub slowest: proftime_T,
    pub average: proftime_T,
    pub id: ::core::ffi::c_int,
    pub pattern: *mut ::core::ffi::c_char,
}
static mut namelist1: [keyvalue_T; 10] = [keyvalue_T {
    key: 0,
    value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    length: 0,
}; 10];
static mut namelist2: [keyvalue_T; 3] = [keyvalue_T {
    key: 0,
    value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    length: 0,
}; 3];
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const SYNSPL_DEFAULT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNSPL_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SYNSPL_NOTOP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SYNFLD_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNFLD_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SYNTAX_FNAME: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"$VIMRUNTIME/syntax/%s.vim\0")
};
pub const SST_MIN_ENTRIES: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
pub const SST_MAX_ENTRIES: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const SST_FIX_STATES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const SST_DIST: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
static mut did_syntax_onoff: bool = false_0 != 0;
pub const SPO_MS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPO_ME_OFF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPO_HS_OFF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPO_HE_OFF: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPO_RS_OFF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SPO_RE_OFF: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const SPO_LC_OFF: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const SPO_COUNT: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
static mut e_illegal_arg: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E390: Illegal argument: %s\0")
};
static mut e_contains_argument_not_accepted_here: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E395: Contains argument not accepted here\0",
    )
};
static mut e_invalid_cchar_value: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E844: Invalid cchar value\0")
};
static mut e_trailing_char_after_rsb_str_str: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E890: Trailing char after ']': %s]%s\0",
    )
};
static mut spo_name_tab: [*mut ::core::ffi::c_char; 7] = [
    b"ms=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"me=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"hs=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"he=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"rs=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"re=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"lc=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
];
pub const SPTYPE_MATCH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPTYPE_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPTYPE_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPTYPE_SKIP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NONE_IDX: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SF_CCOMMENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SF_MATCH: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MAXKEYWLEN: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
static mut current_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_trans_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_seqnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_sub_char: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CLUSTER_REPLACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLUSTER_ADD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CLUSTER_SUBTRACT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SYNID_TOP: ::core::ffi::c_int = 21000 as ::core::ffi::c_int;
pub const SYNID_CONTAINED: ::core::ffi::c_int = 22000 as ::core::ffi::c_int;
pub const SYNID_CLUSTER: ::core::ffi::c_int = 23000 as ::core::ffi::c_int;
pub const MAX_SYN_INC_TAG: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub const MAX_CLUSTER_ID: ::core::ffi::c_int = 32767 as ::core::ffi::c_int - SYNID_CLUSTER;
static mut syn_cmdlinep: *mut *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
static mut current_syn_inc_tag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut running_syn_inc_tag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut dumkey: keyentry_T = keyentry_T {
    ke_next: ::core::ptr::null_mut::<keyentry_T>(),
    k_syn: sp_syn {
        inc_tag: 0,
        id: 0,
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
    },
    next_list: ::core::ptr::null_mut::<int16_t>(),
    flags: 0,
    k_char: 0,
    keyword: [],
};
static mut keepend_level: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
static mut msg_no_items: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"No Syntax items defined for this buffer\0",
    )
};
pub const KEYWORD_IDX: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const ID_LIST_ALL: *mut int16_t = -1 as ::core::ffi::c_int as *mut int16_t;
static mut next_seqnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut next_match_col: ::core::ffi::c_int = 0;
static mut next_match_m_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
static mut next_match_h_startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
static mut next_match_h_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
static mut next_match_idx: ::core::ffi::c_int = 0;
static mut next_match_flags: ::core::ffi::c_int = 0;
static mut next_match_eos_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
static mut next_match_eoe_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
static mut next_match_end_idx: ::core::ffi::c_int = 0;
static mut next_match_extmatch: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
static mut syn_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
static mut syn_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
static mut syn_block: *mut synblock_T = ::core::ptr::null_mut::<synblock_T>();
static mut syn_tm: *mut proftime_T = ::core::ptr::null_mut::<proftime_T>();
static mut current_lnum: linenr_T = 0 as linenr_T;
static mut current_col: colnr_T = 0 as colnr_T;
static mut current_state_stored: bool = false_0 != 0;
static mut current_finished: bool = false_0 != 0;
static mut current_state: garray_T = GA_EMPTY_INIT_VALUE;
static mut current_next_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
static mut current_next_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut current_line_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut syn_time_on: bool = false_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn syn_set_timeout(mut tm: *mut proftime_T) {
    syn_tm = tm;
}
#[no_mangle]
pub unsafe extern "C" fn syntax_start(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut last_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut last_min_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut first_stored: linenr_T = 0;
    let mut dist: ::core::ffi::c_int = 0;
    static mut changedtick: varnumber_T = 0 as varnumber_T;
    current_sub_char = NUL;
    if syn_block != (*wp).w_s
        || syn_buf != (*wp).w_buffer
        || changedtick != buf_get_changedtick(syn_buf)
    {
        invalidate_current_state();
        syn_buf = (*wp).w_buffer;
        syn_block = (*wp).w_s;
    }
    changedtick = buf_get_changedtick(syn_buf);
    syn_win = wp;
    syn_stack_alloc();
    if (*syn_block).b_sst_array.is_null() {
        return;
    }
    (*syn_block).b_sst_lasttick = display_tick;
    if current_state.ga_itemsize != 0 as ::core::ffi::c_int
        && current_lnum < lnum
        && current_lnum < (*syn_buf).b_ml.ml_line_count
    {
        syn_finish_line(false_0 != 0);
        if !current_state_stored {
            current_lnum += 1;
            store_current_state();
        }
        if current_lnum != lnum {
            invalidate_current_state();
        }
    } else {
        invalidate_current_state();
    }
    if current_state.ga_itemsize == 0 as ::core::ffi::c_int && !(*syn_block).b_sst_array.is_null() {
        let mut p: *mut synstate_T = (*syn_block).b_sst_first;
        while !p.is_null() {
            if (*p).sst_lnum > lnum {
                break;
            }
            if (*p).sst_change_lnum == 0 as linenr_T {
                last_valid = p;
                if (*p).sst_lnum >= lnum - (*syn_block).b_syn_sync_minlines {
                    last_min_valid = p;
                }
            }
            p = (*p).sst_next;
        }
        if !last_min_valid.is_null() {
            load_current_state(last_min_valid);
        }
    }
    if current_state.ga_itemsize == 0 as ::core::ffi::c_int {
        syn_sync(wp, lnum, last_valid);
        if current_lnum == 1 as linenr_T {
            first_stored = 1 as ::core::ffi::c_int as linenr_T;
        } else {
            first_stored = current_lnum + (*syn_block).b_syn_sync_minlines;
        }
    } else {
        first_stored = current_lnum;
    }
    if (*syn_block).b_sst_len <= Rows {
        dist = 999999 as ::core::ffi::c_int;
    } else {
        dist = ((*syn_buf).b_ml.ml_line_count
            / ((*syn_block).b_sst_len as linenr_T - Rows as linenr_T)
            + 1 as linenr_T) as ::core::ffi::c_int;
    }
    while current_lnum < lnum {
        syn_start_line();
        syn_finish_line(false_0 != 0);
        current_lnum += 1;
        if current_lnum >= first_stored {
            if prev.is_null() {
                prev = syn_stack_find_entry(current_lnum - 1 as linenr_T);
            }
            if prev.is_null() {
                sp = (*syn_block).b_sst_first;
            } else {
                sp = prev;
            }
            while !sp.is_null() && (*sp).sst_lnum < current_lnum {
                sp = (*sp).sst_next;
            }
            if !sp.is_null()
                && (*sp).sst_lnum == current_lnum
                && syn_stack_equal(sp) as ::core::ffi::c_int != 0
            {
                let mut parsed_lnum: linenr_T = current_lnum;
                prev = sp;
                while !sp.is_null() && (*sp).sst_change_lnum <= parsed_lnum {
                    if (*sp).sst_lnum <= lnum {
                        prev = sp;
                    } else if (*sp).sst_change_lnum == 0 as linenr_T {
                        break;
                    }
                    (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
                    sp = (*sp).sst_next;
                }
                load_current_state(prev);
            } else if prev.is_null()
                || current_lnum == lnum
                || current_lnum >= (*prev).sst_lnum + dist as linenr_T
            {
                prev = store_current_state();
            }
        }
        line_breakcheck();
        if !got_int {
            continue;
        }
        current_lnum = lnum;
        break;
    }
    syn_start_line();
}
unsafe extern "C" fn clear_syn_state(mut p: *mut synstate_T) {
    if (*p).sst_stacksize > SST_FIX_STATES {
        let mut _gap: *mut garray_T = &raw mut (*p).sst_union.sst_ga;
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut bufstate_T =
                    ((*_gap).ga_data as *mut bufstate_T).offset(i as isize);
                unref_extmatch((*_item).bs_extmatch);
                i += 1;
            }
        }
        ga_clear(_gap);
    } else {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*p).sst_stacksize {
            unref_extmatch((*p).sst_union.sst_stack[i_0 as usize].bs_extmatch);
            i_0 += 1;
        }
    };
}
unsafe extern "C" fn clear_current_state() {
    let mut _gap: *mut garray_T = &raw mut current_state;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut stateitem_T =
                ((*_gap).ga_data as *mut stateitem_T).offset(i as isize);
            unref_extmatch((*_item).si_extmatch);
            i += 1;
        }
    }
    ga_clear(_gap);
}
unsafe extern "C" fn syn_sync(
    mut wp: *mut win_T,
    mut start_lnum: linenr_T,
    mut last_valid: *mut synstate_T,
) {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut lnum: linenr_T = 0;
    let mut break_lnum: linenr_T = 0;
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut found_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_match_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_current_lnum: linenr_T = 0 as linenr_T;
    let mut found_current_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_m_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    invalidate_current_state();
    if (*syn_block).b_syn_sync_minlines > start_lnum {
        start_lnum = 1 as ::core::ffi::c_int as linenr_T;
    } else {
        if (*syn_block).b_syn_sync_minlines == 1 as linenr_T {
            lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else if (*syn_block).b_syn_sync_minlines < 10 as linenr_T {
            lnum = (*syn_block).b_syn_sync_minlines * 2 as linenr_T;
        } else {
            lnum = (*syn_block).b_syn_sync_minlines * 3 as linenr_T / 2 as linenr_T;
        }
        if (*syn_block).b_syn_sync_maxlines != 0 as linenr_T
            && lnum > (*syn_block).b_syn_sync_maxlines
        {
            lnum = (*syn_block).b_syn_sync_maxlines;
        }
        if lnum >= start_lnum {
            start_lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else {
            start_lnum -= lnum;
        }
    }
    current_lnum = start_lnum;
    if (*syn_block).b_syn_sync_flags & SF_CCOMMENT != 0 {
        let mut curwin_save: *mut win_T = curwin;
        curwin = wp;
        let mut curbuf_save: *mut buf_T = curbuf;
        curbuf = syn_buf;
        while start_lnum > 1 as linenr_T {
            let mut l: *mut ::core::ffi::c_char = ml_get(start_lnum - 1 as linenr_T);
            if *l as ::core::ffi::c_int == NUL
                || *l
                    .offset(ml_get_len(start_lnum - 1 as linenr_T) as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                break;
            }
            start_lnum -= 1;
        }
        current_lnum = start_lnum;
        cursor_save = (*wp).w_cursor;
        (*wp).w_cursor.lnum = start_lnum;
        (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        if !find_start_comment((*syn_block).b_syn_sync_maxlines as ::core::ffi::c_int).is_null() {
            let mut idx: ::core::ffi::c_int = (*syn_block).b_syn_patterns.ga_len;
            loop {
                idx -= 1;
                if idx < 0 as ::core::ffi::c_int {
                    break;
                }
                if !((*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize))
                    .sp_syn
                    .id as ::core::ffi::c_int
                    == (*syn_block).b_syn_sync_id as ::core::ffi::c_int
                    && (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_START)
                {
                    continue;
                }
                validate_current_state();
                push_current_state(idx);
                update_si_attr(current_state.ga_len - 1 as ::core::ffi::c_int);
                break;
            }
        }
        (*wp).w_cursor = cursor_save;
        curwin = curwin_save;
        curbuf = curbuf_save;
    } else if (*syn_block).b_syn_sync_flags & SF_MATCH != 0 {
        if (*syn_block).b_syn_sync_maxlines != 0 as linenr_T
            && start_lnum > (*syn_block).b_syn_sync_maxlines
        {
            break_lnum = start_lnum - (*syn_block).b_syn_sync_maxlines;
        } else {
            break_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        found_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        found_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
        let mut end_lnum: linenr_T = start_lnum;
        lnum = start_lnum;
        loop {
            lnum -= 1;
            if lnum <= break_lnum {
                break;
            }
            line_breakcheck();
            if got_int {
                invalidate_current_state();
                current_lnum = start_lnum;
                break;
            } else if !last_valid.is_null() && lnum == (*last_valid).sst_lnum {
                load_current_state(last_valid);
                break;
            } else {
                if lnum > 1 as linenr_T && syn_match_linecont(lnum - 1 as linenr_T) != 0 {
                    continue;
                }
                validate_current_state();
                current_lnum = lnum;
                while current_lnum < end_lnum {
                    syn_start_line();
                    loop {
                        let mut had_sync_point: bool = syn_finish_line(true_0 != 0);
                        if !(had_sync_point as ::core::ffi::c_int != 0 && current_state.ga_len != 0)
                        {
                            break;
                        }
                        cur_si = (current_state.ga_data as *mut stateitem_T)
                            .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
                        if (*cur_si).si_m_endpos.lnum > start_lnum {
                            current_lnum = end_lnum;
                            break;
                        } else {
                            if (*cur_si).si_idx < 0 as ::core::ffi::c_int {
                                found_flags = 0 as ::core::ffi::c_int;
                                found_match_idx = KEYWORD_IDX;
                            } else {
                                spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                                    .offset((*cur_si).si_idx as isize);
                                found_flags = (*spp).sp_flags;
                                found_match_idx = (*spp).sp_sync_idx;
                            }
                            found_current_lnum = current_lnum;
                            found_current_col = current_col as ::core::ffi::c_int;
                            found_m_endpos = (*cur_si).si_m_endpos;
                            if found_m_endpos.lnum > current_lnum {
                                current_lnum = found_m_endpos.lnum;
                                current_col = found_m_endpos.col;
                                if current_lnum >= end_lnum {
                                    break;
                                }
                            } else if found_m_endpos.col > current_col {
                                current_col = found_m_endpos.col;
                            } else {
                                current_col += 1;
                            }
                            let mut prev_current_col: colnr_T = current_col;
                            if *syn_getcurline().offset(current_col as isize) as ::core::ffi::c_int
                                != NUL
                            {
                                current_col += 1;
                            }
                            check_state_ends();
                            current_col = prev_current_col;
                        }
                    }
                    current_lnum += 1;
                }
                if found_flags != 0 {
                    clear_current_state();
                    if found_match_idx >= 0 as ::core::ffi::c_int {
                        push_current_state(found_match_idx);
                        update_si_attr(current_state.ga_len - 1 as ::core::ffi::c_int);
                    }
                    if found_flags & HL_SYNC_HERE as ::core::ffi::c_int != 0 {
                        current_lnum = found_m_endpos.lnum;
                        current_col = found_m_endpos.col;
                        if !(current_state.ga_len <= 0 as ::core::ffi::c_int) {
                            cur_si = (current_state.ga_data as *mut stateitem_T)
                                .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
                            (*cur_si).si_h_startpos.lnum = found_current_lnum;
                            (*cur_si).si_h_startpos.col = found_current_col as colnr_T;
                            update_si_end(cur_si, current_col, true_0 != 0);
                            check_keepend();
                        }
                        syn_finish_line(false_0 != 0);
                        current_lnum += 1;
                    } else {
                        current_lnum = start_lnum;
                    }
                    break;
                } else {
                    end_lnum = lnum;
                    invalidate_current_state();
                }
            }
        }
        if lnum <= break_lnum {
            invalidate_current_state();
            current_lnum = break_lnum + 1 as linenr_T;
        }
    }
    validate_current_state();
}
unsafe extern "C" fn save_chartab(mut chartab: *mut ::core::ffi::c_char) {
    if (*syn_block).b_syn_isk == &raw mut empty_string_option as *mut ::core::ffi::c_char {
        return;
    }
    memmove(
        chartab as *mut ::core::ffi::c_void,
        &raw mut (*syn_buf).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
        32 as ::core::ffi::c_int as size_t,
    );
    memmove(
        &raw mut (*syn_buf).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
        &raw mut (*(*syn_win).w_s).b_syn_chartab as *mut uint8_t as *const ::core::ffi::c_void,
        32 as ::core::ffi::c_int as size_t,
    );
}
unsafe extern "C" fn restore_chartab(mut chartab: *mut ::core::ffi::c_char) {
    if (*(*syn_win).w_s).b_syn_isk != &raw mut empty_string_option as *mut ::core::ffi::c_char {
        memmove(
            &raw mut (*syn_buf).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            chartab as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
    }
}
unsafe extern "C" fn syn_match_linecont(mut lnum: linenr_T) -> ::core::ffi::c_int {
    if (*syn_block).b_syn_linecont_prog.is_null() {
        return false_0;
    }
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    regmatch.rmm_ic = (*syn_block).b_syn_linecont_ic;
    regmatch.regprog = (*syn_block).b_syn_linecont_prog;
    let mut r: ::core::ffi::c_int = syn_regexec(
        &raw mut regmatch,
        lnum,
        0 as colnr_T,
        &raw mut (*syn_block).b_syn_linecont_time,
    ) as ::core::ffi::c_int;
    (*syn_block).b_syn_linecont_prog = regmatch.regprog;
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    return r;
}
unsafe extern "C" fn syn_start_line() {
    current_finished = false_0 != 0;
    current_col = 0 as ::core::ffi::c_int as colnr_T;
    if !(current_state.ga_len <= 0 as ::core::ffi::c_int) {
        syn_update_ends(true_0 != 0);
        check_state_ends();
    }
    next_match_idx = -1 as ::core::ffi::c_int;
    current_line_id += 1;
    next_seqnr = 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn syn_update_ends(mut startofline: bool) {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    if startofline {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < current_state.ga_len {
            cur_si = (current_state.ga_data as *mut stateitem_T).offset(i as isize);
            if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_MATCH
                && (*cur_si).si_m_endpos.lnum < current_lnum
            {
                (*cur_si).si_flags |= HL_MATCHCONT as ::core::ffi::c_int;
                (*cur_si).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                (*cur_si).si_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
                (*cur_si).si_h_endpos = (*cur_si).si_m_endpos;
                (*cur_si).si_ends = true_0;
            }
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = current_state.ga_len - 1 as ::core::ffi::c_int;
    if keepend_level >= 0 as ::core::ffi::c_int {
        while i_0 > keepend_level {
            if (*(current_state.ga_data as *mut stateitem_T).offset(i_0 as isize)).si_flags
                & HL_EXTEND as ::core::ffi::c_int
                != 0
            {
                break;
            }
            i_0 -= 1;
        }
    }
    let mut seen_keepend: bool = false_0 != 0;
    while i_0 < current_state.ga_len {
        cur_si = (current_state.ga_data as *mut stateitem_T).offset(i_0 as isize);
        if (*cur_si).si_flags & HL_KEEPEND as ::core::ffi::c_int != 0
            || seen_keepend as ::core::ffi::c_int != 0 && !startofline
            || i_0 == current_state.ga_len - 1 as ::core::ffi::c_int
                && startofline as ::core::ffi::c_int != 0
        {
            (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
            (*cur_si).si_h_startpos.lnum = current_lnum;
            if (*cur_si).si_flags & HL_MATCHCONT as ::core::ffi::c_int == 0 {
                update_si_end(cur_si, current_col, !startofline);
            }
            if !startofline && (*cur_si).si_flags & HL_KEEPEND as ::core::ffi::c_int != 0 {
                seen_keepend = true_0 != 0;
            }
        }
        i_0 += 1;
    }
    check_keepend();
}
unsafe extern "C" fn syn_stack_free_block(mut block: *mut synblock_T) {
    if (*block).b_sst_array.is_null() {
        return;
    }
    let mut p: *mut synstate_T = (*block).b_sst_first;
    while !p.is_null() {
        clear_syn_state(p);
        p = (*p).sst_next;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*block).b_sst_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    (*block).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
    (*block).b_sst_len = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn syn_stack_free_all(mut block: *mut synblock_T) {
    syn_stack_free_block(block);
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_s == block && foldmethodIsSyntax(wp) as ::core::ffi::c_int != 0 {
            foldUpdateAll(wp);
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn syn_stack_alloc() {
    let mut len: ::core::ffi::c_int = (*syn_buf).b_ml.ml_line_count as ::core::ffi::c_int
        / SST_DIST
        + Rows * 2 as ::core::ffi::c_int;
    if len < SST_MIN_ENTRIES {
        len = SST_MIN_ENTRIES;
    } else if len > SST_MAX_ENTRIES {
        len = SST_MAX_ENTRIES;
    }
    if (*syn_block).b_sst_len > len * 2 as ::core::ffi::c_int || (*syn_block).b_sst_len < len {
        len = (*syn_buf).b_ml.ml_line_count as ::core::ffi::c_int;
        len = (len + len / 2 as ::core::ffi::c_int) / SST_DIST + Rows * 2 as ::core::ffi::c_int;
        if len < SST_MIN_ENTRIES {
            len = SST_MIN_ENTRIES;
        } else if len > SST_MAX_ENTRIES {
            len = SST_MAX_ENTRIES;
        }
        if !(*syn_block).b_sst_array.is_null() {
            while (*syn_block).b_sst_len - (*syn_block).b_sst_freecount + 2 as ::core::ffi::c_int
                > len
                && syn_stack_cleanup() as ::core::ffi::c_int != 0
            {}
            if len < (*syn_block).b_sst_len - (*syn_block).b_sst_freecount + 2 as ::core::ffi::c_int
            {
                len =
                    (*syn_block).b_sst_len - (*syn_block).b_sst_freecount + 2 as ::core::ffi::c_int;
            }
        }
        '_c2rust_label: {
            if len >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/syntax.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    926 as ::core::ffi::c_uint,
                    b"void syn_stack_alloc(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut sstp: *mut synstate_T =
            xcalloc(len as size_t, ::core::mem::size_of::<synstate_T>()) as *mut synstate_T;
        let mut to: *mut synstate_T = sstp.offset(-(1 as ::core::ffi::c_int as isize));
        if !(*syn_block).b_sst_array.is_null() {
            let mut from: *mut synstate_T = (*syn_block).b_sst_first;
            while !from.is_null() {
                to = to.offset(1);
                *to = *from;
                (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
                from = (*from).sst_next;
            }
        }
        if to != sstp.offset(-(1 as ::core::ffi::c_int as isize)) {
            (*to).sst_next = ::core::ptr::null_mut::<synstate_T>();
            (*syn_block).b_sst_first = sstp;
            (*syn_block).b_sst_freecount =
                len - to.offset_from(sstp) as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        } else {
            (*syn_block).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
            (*syn_block).b_sst_freecount = len;
        }
        (*syn_block).b_sst_firstfree = to.offset(1 as ::core::ffi::c_int as isize);
        loop {
            to = to.offset(1);
            if to >= sstp.offset(len as isize) {
                break;
            }
            (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
        }
        (*sstp
            .offset(len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)))
        .sst_next = ::core::ptr::null_mut::<synstate_T>();
        xfree((*syn_block).b_sst_array as *mut ::core::ffi::c_void);
        (*syn_block).b_sst_array = sstp;
        (*syn_block).b_sst_len = len;
    }
}
#[no_mangle]
pub unsafe extern "C" fn syn_stack_apply_changes(mut buf: *mut buf_T) {
    syn_stack_apply_changes_block(&raw mut (*buf).b_s, buf);
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf && (*wp).w_s != &raw mut (*buf).b_s {
            syn_stack_apply_changes_block((*wp).w_s, buf);
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn syn_stack_apply_changes_block(
    mut block: *mut synblock_T,
    mut buf: *mut buf_T,
) {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut p: *mut synstate_T = (*block).b_sst_first;
    while !p.is_null() {
        if (*p).sst_lnum + (*block).b_syn_sync_linebreaks > (*buf).b_mod_top {
            let mut n: linenr_T = (*p).sst_lnum + (*buf).b_mod_xlines;
            if n <= (*buf).b_mod_bot {
                let mut np: *mut synstate_T = (*p).sst_next;
                if prev.is_null() {
                    (*block).b_sst_first = np;
                } else {
                    (*prev).sst_next = np;
                }
                syn_stack_free_entry(block, p);
                p = np;
                continue;
            } else {
                if (*p).sst_change_lnum != 0 as linenr_T && (*p).sst_change_lnum > (*buf).b_mod_top
                {
                    if (*p).sst_change_lnum + (*buf).b_mod_xlines > (*buf).b_mod_top {
                        (*p).sst_change_lnum += (*buf).b_mod_xlines;
                    } else {
                        (*p).sst_change_lnum = (*buf).b_mod_top;
                    }
                }
                if (*p).sst_change_lnum == 0 as linenr_T || (*p).sst_change_lnum < (*buf).b_mod_bot
                {
                    (*p).sst_change_lnum = (*buf).b_mod_bot;
                }
                (*p).sst_lnum = n;
            }
        }
        prev = p;
        p = (*p).sst_next;
    }
}
unsafe extern "C" fn syn_stack_cleanup() -> bool {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut tick: disptick_T = 0;
    let mut dist: ::core::ffi::c_int = 0;
    let mut retval: bool = false_0 != 0;
    if (*syn_block).b_sst_first.is_null() {
        return retval;
    }
    if (*syn_block).b_sst_len <= Rows {
        dist = 999999 as ::core::ffi::c_int;
    } else {
        dist = ((*syn_buf).b_ml.ml_line_count
            / ((*syn_block).b_sst_len as linenr_T - Rows as linenr_T)
            + 1 as linenr_T) as ::core::ffi::c_int;
    }
    tick = (*syn_block).b_sst_lasttick;
    let mut above: bool = false_0 != 0;
    prev = (*syn_block).b_sst_first;
    let mut p: *mut synstate_T = (*prev).sst_next;
    while !p.is_null() {
        if (*prev).sst_lnum + dist as linenr_T > (*p).sst_lnum {
            if (*p).sst_tick > (*syn_block).b_sst_lasttick {
                if !above || (*p).sst_tick < tick {
                    tick = (*p).sst_tick;
                }
                above = true_0 != 0;
            } else if !above && (*p).sst_tick < tick {
                tick = (*p).sst_tick;
            }
        }
        prev = p;
        p = (*p).sst_next;
    }
    prev = (*syn_block).b_sst_first;
    let mut p_0: *mut synstate_T = (*prev).sst_next;
    while !p_0.is_null() {
        if (*p_0).sst_tick == tick && (*prev).sst_lnum + dist as linenr_T > (*p_0).sst_lnum {
            (*prev).sst_next = (*p_0).sst_next;
            syn_stack_free_entry(syn_block, p_0);
            p_0 = prev;
            retval = true_0 != 0;
        }
        prev = p_0;
        p_0 = (*p_0).sst_next;
    }
    return retval;
}
unsafe extern "C" fn syn_stack_free_entry(mut block: *mut synblock_T, mut p: *mut synstate_T) {
    clear_syn_state(p);
    (*p).sst_next = (*block).b_sst_firstfree;
    (*block).b_sst_firstfree = p;
    (*block).b_sst_freecount += 1;
}
unsafe extern "C" fn syn_stack_find_entry(mut lnum: linenr_T) -> *mut synstate_T {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut p: *mut synstate_T = (*syn_block).b_sst_first;
    while !p.is_null() {
        if (*p).sst_lnum == lnum {
            return p;
        }
        if (*p).sst_lnum > lnum {
            break;
        }
        prev = p;
        p = (*p).sst_next;
    }
    return prev;
}
unsafe extern "C" fn store_current_state() -> *mut synstate_T {
    let mut i: ::core::ffi::c_int = 0;
    let mut p: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut sp: *mut synstate_T = syn_stack_find_entry(current_lnum);
    i = current_state.ga_len - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        cur_si = (current_state.ga_data as *mut stateitem_T).offset(i as isize);
        if (*cur_si).si_h_startpos.lnum >= current_lnum
            || (*cur_si).si_m_endpos.lnum >= current_lnum
            || (*cur_si).si_h_endpos.lnum >= current_lnum
            || (*cur_si).si_end_idx != 0 && (*cur_si).si_eoe_pos.lnum >= current_lnum
        {
            break;
        }
        i -= 1;
    }
    if i >= 0 as ::core::ffi::c_int {
        if !sp.is_null() {
            if (*syn_block).b_sst_first == sp {
                (*syn_block).b_sst_first = (*sp).sst_next;
            } else {
                p = (*syn_block).b_sst_first;
                while !p.is_null() {
                    if (*p).sst_next == sp {
                        break;
                    }
                    p = (*p).sst_next;
                }
                if !p.is_null() {
                    (*p).sst_next = (*sp).sst_next;
                }
            }
            syn_stack_free_entry(syn_block, sp);
            sp = ::core::ptr::null_mut::<synstate_T>();
        }
    } else if sp.is_null() || (*sp).sst_lnum != current_lnum {
        if (*syn_block).b_sst_freecount == 0 as ::core::ffi::c_int {
            syn_stack_cleanup();
            sp = syn_stack_find_entry(current_lnum);
        }
        if (*syn_block).b_sst_freecount == 0 as ::core::ffi::c_int {
            sp = ::core::ptr::null_mut::<synstate_T>();
        } else {
            p = (*syn_block).b_sst_firstfree;
            (*syn_block).b_sst_firstfree = (*p).sst_next;
            (*syn_block).b_sst_freecount -= 1;
            if sp.is_null() {
                (*p).sst_next = (*syn_block).b_sst_first;
                (*syn_block).b_sst_first = p;
            } else {
                (*p).sst_next = (*sp).sst_next;
                (*sp).sst_next = p;
            }
            sp = p;
            (*sp).sst_stacksize = 0 as ::core::ffi::c_int;
            (*sp).sst_lnum = current_lnum;
        }
    }
    if !sp.is_null() {
        clear_syn_state(sp);
        (*sp).sst_stacksize = current_state.ga_len;
        if current_state.ga_len > SST_FIX_STATES {
            ga_init(
                &raw mut (*sp).sst_union.sst_ga,
                ::core::mem::size_of::<bufstate_T>() as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            ga_grow(&raw mut (*sp).sst_union.sst_ga, current_state.ga_len);
            (*sp).sst_union.sst_ga.ga_len = current_state.ga_len;
            bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
        } else {
            bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
        }
        i = 0 as ::core::ffi::c_int;
        while i < (*sp).sst_stacksize {
            (*bp.offset(i as isize)).bs_idx =
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx;
            (*bp.offset(i as isize)).bs_flags =
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_flags;
            (*bp.offset(i as isize)).bs_seqnr =
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_seqnr;
            (*bp.offset(i as isize)).bs_cchar =
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_cchar;
            (*bp.offset(i as isize)).bs_extmatch = ref_extmatch(
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch,
            );
            i += 1;
        }
        (*sp).sst_next_flags = current_next_flags;
        (*sp).sst_next_list = current_next_list;
        (*sp).sst_tick = display_tick;
        (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    current_state_stored = true_0 != 0;
    return sp;
}
unsafe extern "C" fn load_current_state(mut from: *mut synstate_T) {
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    clear_current_state();
    validate_current_state();
    keepend_level = -1 as ::core::ffi::c_int;
    if (*from).sst_stacksize != 0 {
        ga_grow(&raw mut current_state, (*from).sst_stacksize);
        if (*from).sst_stacksize > SST_FIX_STATES {
            bp = (*from).sst_union.sst_ga.ga_data as *mut bufstate_T;
        } else {
            bp = &raw mut (*from).sst_union.sst_stack as *mut bufstate_T;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*from).sst_stacksize {
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx =
                (*bp.offset(i as isize)).bs_idx;
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_flags =
                (*bp.offset(i as isize)).bs_flags;
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_seqnr =
                (*bp.offset(i as isize)).bs_seqnr;
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_cchar =
                (*bp.offset(i as isize)).bs_cchar;
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch =
                ref_extmatch((*bp.offset(i as isize)).bs_extmatch);
            if keepend_level < 0 as ::core::ffi::c_int
                && (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_flags
                    & HL_KEEPEND as ::core::ffi::c_int
                    != 0
            {
                keepend_level = i;
            }
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_ends = false_0;
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_m_lnum =
                0 as ::core::ffi::c_int;
            if (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                >= 0 as ::core::ffi::c_int
            {
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_next_list =
                    (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(
                        (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                            as isize,
                    ))
                    .sp_next_list;
            } else {
                (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_next_list =
                    ::core::ptr::null_mut::<int16_t>();
            }
            update_si_attr(i);
            i += 1;
        }
        current_state.ga_len = (*from).sst_stacksize;
    }
    current_next_list = (*from).sst_next_list;
    current_next_flags = (*from).sst_next_flags;
    current_lnum = (*from).sst_lnum;
}
unsafe extern "C" fn syn_stack_equal(mut sp: *mut synstate_T) -> bool {
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    if (*sp).sst_stacksize != current_state.ga_len || (*sp).sst_next_list != current_next_list {
        return false_0 != 0;
    }
    if (*sp).sst_stacksize > SST_FIX_STATES {
        bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
    } else {
        bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
    }
    let mut i: ::core::ffi::c_int = 0;
    i = current_state.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*bp.offset(i as isize)).bs_idx
            != (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx
        {
            break;
        }
        if (*bp.offset(i as isize)).bs_extmatch
            == (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch
        {
            continue;
        }
        let mut bsx: *mut reg_extmatch_T = (*bp.offset(i as isize)).bs_extmatch;
        let mut six: *mut reg_extmatch_T =
            (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch;
        if bsx.is_null() || six.is_null() {
            break;
        }
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while j < NSUBEXP as ::core::ffi::c_int {
            if (*bsx).matches[j as usize] != (*six).matches[j as usize] {
                if (*bsx).matches[j as usize].is_null() || (*six).matches[j as usize].is_null() {
                    break;
                }
                if mb_strcmp_ic(
                    (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(
                        (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                            as isize,
                    ))
                    .sp_ic
                        != 0,
                    (*bsx).matches[j as usize] as *const ::core::ffi::c_char,
                    (*six).matches[j as usize] as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
            }
            j += 1;
        }
        if j != NSUBEXP as ::core::ffi::c_int {
            break;
        }
    }
    return if i < 0 as ::core::ffi::c_int {
        true_0
    } else {
        false_0
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn syntax_end_parsing(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    if syn_block != (*wp).w_s {
        return;
    }
    sp = syn_stack_find_entry(lnum);
    if !sp.is_null() && (*sp).sst_lnum < lnum {
        sp = (*sp).sst_next;
    }
    if !sp.is_null() && (*sp).sst_change_lnum != 0 as linenr_T {
        (*sp).sst_change_lnum = lnum;
    }
}
unsafe extern "C" fn invalidate_current_state() {
    clear_current_state();
    current_state.ga_itemsize = 0 as ::core::ffi::c_int;
    current_next_list = ::core::ptr::null_mut::<int16_t>();
    keepend_level = -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn validate_current_state() {
    current_state.ga_itemsize = ::core::mem::size_of::<stateitem_T>() as ::core::ffi::c_int;
    ga_set_growsize(&raw mut current_state, 3 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn syntax_check_changed(mut lnum: linenr_T) -> bool {
    let mut retval: bool = true_0 != 0;
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    if current_state.ga_itemsize != 0 as ::core::ffi::c_int && lnum == current_lnum + 1 as linenr_T
    {
        sp = syn_stack_find_entry(lnum);
        if !sp.is_null() && (*sp).sst_lnum == lnum {
            syn_finish_line(false_0 != 0);
            if syn_stack_equal(sp) {
                retval = false_0 != 0;
            }
            current_lnum += 1;
            store_current_state();
        }
    }
    return retval;
}
unsafe extern "C" fn syn_finish_line(syncing: bool) -> bool {
    while !current_finished {
        syn_current_attr(
            syncing,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
            false_0 != 0,
        );
        if syncing as ::core::ffi::c_int != 0 && current_state.ga_len != 0 {
            let cur_si: *const stateitem_T = (current_state.ga_data as *mut stateitem_T)
                .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
            if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_flags
                    & (HL_SYNC_HERE as ::core::ffi::c_int | HL_SYNC_THERE as ::core::ffi::c_int)
                    != 0
            {
                return true_0 != 0;
            }
            let prev_current_col: colnr_T = current_col;
            if *syn_getcurline().offset(current_col as isize) as ::core::ffi::c_int != NUL {
                current_col += 1;
            }
            check_state_ends();
            current_col = prev_current_col;
        }
        current_col += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_syntax_attr(
    col: colnr_T,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !can_spell.is_null() {
        *can_spell = if (*syn_block).b_syn_spell == SYNSPL_DEFAULT {
            ((*syn_block).b_spell_cluster_id == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((*syn_block).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
        } != 0;
    }
    if (*syn_block).b_sst_array.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*syn_buf).b_p_smc > 0 as OptInt && col >= (*syn_buf).b_p_smc as colnr_T {
        clear_current_state();
        current_id = 0 as ::core::ffi::c_int;
        current_trans_id = 0 as ::core::ffi::c_int;
        current_flags = 0 as ::core::ffi::c_int;
        current_seqnr = 0 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    if current_state.ga_itemsize == 0 as ::core::ffi::c_int {
        validate_current_state();
    }
    while current_col <= col {
        attr = syn_current_attr(
            false_0 != 0,
            true_0 != 0,
            can_spell,
            if current_col == col {
                keep_state as ::core::ffi::c_int
            } else {
                false_0
            } != 0,
        );
        current_col += 1;
    }
    return attr;
}
unsafe extern "C" fn syn_current_attr(
    syncing: bool,
    displaying: bool,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    let mut endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut hl_startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut eos_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut eoe_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut end_idx: ::core::ffi::c_int = 0;
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut startcol: ::core::ffi::c_int = 0;
    let mut endcol: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut cchar: ::core::ffi::c_int = 0;
    let mut next_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    let mut found_match: bool = false;
    static mut try_next_column: bool = false_0 != 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut cur_extmatch: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keep_next_list: bool = false;
    let mut zero_width_next_list: bool = false_0 != 0;
    let mut zero_width_next_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    line = syn_getcurline();
    if *line.offset(current_col as isize) as ::core::ffi::c_int == NUL
        && current_col != 0 as ::core::ffi::c_int
    {
        if next_match_idx >= 0 as ::core::ffi::c_int
            && next_match_col >= current_col
            && next_match_col != MAXCOL as ::core::ffi::c_int
        {
            push_next_match();
        }
        current_finished = true_0 != 0;
        current_state_stored = false_0 != 0;
        return 0 as ::core::ffi::c_int;
    }
    if *line.offset(current_col as isize) as ::core::ffi::c_int == NUL
        || *line.offset((current_col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == NUL
    {
        current_finished = true_0 != 0;
        current_state_stored = false_0 != 0;
    }
    if try_next_column {
        next_match_idx = -1 as ::core::ffi::c_int;
        try_next_column = false_0 != 0;
    }
    let do_keywords: bool = !syncing
        && ((*syn_block).b_keywtab.ht_used > 0 as size_t
            || (*syn_block).b_keywtab_ic.ht_used > 0 as size_t);
    ga_init(
        &raw mut zero_width_next_ga,
        ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    loop {
        found_match = false_0 != 0;
        keep_next_list = false_0 != 0;
        let mut syn_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if current_state.ga_len != 0 {
            cur_si = (current_state.ga_data as *mut stateitem_T)
                .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
        } else {
            cur_si = ::core::ptr::null_mut::<stateitem_T>();
        }
        if (*syn_block).b_syn_containedin != 0
            || cur_si.is_null()
            || !(*cur_si).si_cont_list.is_null()
        {
            if do_keywords {
                line = syn_getcurline();
                let mut cur_pos: *const ::core::ffi::c_char = line.offset(current_col as isize);
                if vim_iswordp_buf(cur_pos, syn_buf) as ::core::ffi::c_int != 0
                    && (current_col == 0 as ::core::ffi::c_int
                        || !vim_iswordp_buf(
                            cur_pos.offset(-(1 as ::core::ffi::c_int as isize)).offset(
                                -(utf_head_off(
                                    line,
                                    cur_pos.offset(-(1 as ::core::ffi::c_int as isize)),
                                ) as isize),
                            ),
                            syn_buf,
                        ))
                {
                    syn_id = check_keyword_id(
                        line,
                        current_col,
                        &raw mut endcol,
                        &raw mut flags,
                        &raw mut next_list,
                        cur_si,
                        &raw mut cchar,
                    );
                    if syn_id != 0 as ::core::ffi::c_int {
                        push_current_state(KEYWORD_IDX);
                        cur_si = (current_state.ga_data as *mut stateitem_T)
                            .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
                        (*cur_si).si_m_startcol = current_col as ::core::ffi::c_int;
                        (*cur_si).si_h_startpos.lnum = current_lnum;
                        (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*cur_si).si_m_endpos.lnum = current_lnum;
                        (*cur_si).si_m_endpos.col = endcol as colnr_T;
                        (*cur_si).si_h_endpos.lnum = current_lnum;
                        (*cur_si).si_h_endpos.col = endcol as colnr_T;
                        (*cur_si).si_ends = true_0;
                        (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
                        (*cur_si).si_flags = flags;
                        let c2rust_fresh3 = next_seqnr;
                        next_seqnr = next_seqnr + 1;
                        (*cur_si).si_seqnr = c2rust_fresh3;
                        (*cur_si).si_cchar = cchar;
                        if current_state.ga_len > 1 as ::core::ffi::c_int {
                            (*cur_si).si_flags |= (*(current_state.ga_data as *mut stateitem_T)
                                .offset((current_state.ga_len - 2 as ::core::ffi::c_int) as isize))
                            .si_flags
                                & HL_CONCEAL as ::core::ffi::c_int;
                        }
                        (*cur_si).si_id = syn_id;
                        (*cur_si).si_trans_id = syn_id;
                        if flags & HL_TRANSP as ::core::ffi::c_int != 0 {
                            if current_state.ga_len < 2 as ::core::ffi::c_int {
                                (*cur_si).si_attr = 0 as ::core::ffi::c_int;
                                (*cur_si).si_trans_id = 0 as ::core::ffi::c_int;
                            } else {
                                (*cur_si).si_attr = (*(current_state.ga_data as *mut stateitem_T)
                                    .offset(
                                        (current_state.ga_len - 2 as ::core::ffi::c_int) as isize,
                                    ))
                                .si_attr;
                                (*cur_si).si_trans_id =
                                    (*(current_state.ga_data as *mut stateitem_T).offset(
                                        (current_state.ga_len - 2 as ::core::ffi::c_int) as isize,
                                    ))
                                    .si_trans_id;
                            }
                        } else {
                            (*cur_si).si_attr = syn_id2attr(syn_id);
                        }
                        (*cur_si).si_cont_list = ::core::ptr::null_mut::<int16_t>();
                        (*cur_si).si_next_list = next_list;
                        check_keepend();
                    }
                }
            }
            if syn_id == 0 as ::core::ffi::c_int && (*syn_block).b_syn_patterns.ga_len != 0 {
                if next_match_idx < 0 as ::core::ffi::c_int || next_match_col < current_col {
                    next_match_idx = 0 as ::core::ffi::c_int;
                    next_match_col = MAXCOL as ::core::ffi::c_int;
                    let mut idx: ::core::ffi::c_int = (*syn_block).b_syn_patterns.ga_len;
                    loop {
                        idx -= 1;
                        if idx < 0 as ::core::ffi::c_int {
                            break;
                        }
                        let spp: *mut synpat_T = ((*syn_block).b_syn_patterns.ga_data
                            as *mut synpat_T)
                            .offset(idx as isize);
                        if !((*spp).sp_syncing as ::core::ffi::c_int
                            == syncing as ::core::ffi::c_int
                            && (displaying as ::core::ffi::c_int != 0
                                || (*spp).sp_flags & HL_DISPLAY as ::core::ffi::c_int == 0)
                            && ((*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH
                                || (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START)
                            && (if !current_next_list.is_null() {
                                in_id_list(
                                    ::core::ptr::null_mut::<stateitem_T>(),
                                    current_next_list,
                                    &raw mut (*spp).sp_syn,
                                    0 as ::core::ffi::c_int,
                                )
                            } else {
                                if cur_si.is_null() {
                                    ((*spp).sp_flags & HL_CONTAINED as ::core::ffi::c_int == 0)
                                        as ::core::ffi::c_int
                                } else {
                                    in_id_list(
                                        cur_si,
                                        (*cur_si).si_cont_list,
                                        &raw mut (*spp).sp_syn,
                                        (*spp).sp_flags,
                                    )
                                }
                            }) != 0)
                        {
                            continue;
                        }
                        if (*spp).sp_line_id == current_line_id
                            && (*spp).sp_startcol >= next_match_col
                        {
                            continue;
                        }
                        (*spp).sp_line_id = current_line_id;
                        let mut lc_col: colnr_T =
                            current_col - (*spp).sp_offsets[SPO_LC_OFF as usize] as colnr_T;
                        if lc_col < 0 as ::core::ffi::c_int {
                            lc_col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        regmatch.rmm_ic = (*spp).sp_ic;
                        regmatch.regprog = (*spp).sp_prog;
                        let mut r: ::core::ffi::c_int = syn_regexec(
                            &raw mut regmatch,
                            current_lnum,
                            lc_col,
                            &raw mut (*spp).sp_time,
                        )
                            as ::core::ffi::c_int;
                        (*spp).sp_prog = regmatch.regprog;
                        if r == 0 {
                            (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                        } else {
                            syn_add_start_off(
                                &raw mut pos,
                                &raw mut regmatch,
                                spp,
                                SPO_MS_OFF,
                                -1 as ::core::ffi::c_int,
                            );
                            if pos.lnum > current_lnum {
                                (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                            } else {
                                startcol = pos.col as ::core::ffi::c_int;
                                (*spp).sp_startcol = startcol;
                                if startcol >= next_match_col {
                                    continue;
                                }
                                if did_match_already(idx, &raw mut zero_width_next_ga) {
                                    try_next_column = true_0 != 0;
                                } else {
                                    endpos.lnum =
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                                    endpos.col =
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                    syn_add_start_off(
                                        &raw mut hl_startpos,
                                        &raw mut regmatch,
                                        spp,
                                        SPO_HS_OFF,
                                        -1 as ::core::ffi::c_int,
                                    );
                                    syn_add_end_off(
                                        &raw mut eos_pos,
                                        &raw mut regmatch,
                                        spp,
                                        SPO_RS_OFF,
                                        0 as ::core::ffi::c_int,
                                    );
                                    unref_extmatch(cur_extmatch);
                                    cur_extmatch = re_extmatch_out;
                                    re_extmatch_out = ::core::ptr::null_mut::<reg_extmatch_T>();
                                    flags = 0 as ::core::ffi::c_int;
                                    eoe_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    eoe_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                                    end_idx = 0 as ::core::ffi::c_int;
                                    hl_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
                                        && (*spp).sp_flags & HL_ONELINE as ::core::ffi::c_int != 0
                                    {
                                        let mut startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
                                        startpos = endpos;
                                        find_endpos(
                                            idx,
                                            &raw mut startpos,
                                            &raw mut endpos,
                                            &raw mut hl_endpos,
                                            &raw mut flags,
                                            &raw mut eoe_pos,
                                            &raw mut end_idx,
                                            cur_extmatch,
                                        );
                                        if endpos.lnum == 0 as linenr_T {
                                            continue;
                                        }
                                    } else if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH {
                                        syn_add_end_off(
                                            &raw mut hl_endpos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_HE_OFF,
                                            0 as ::core::ffi::c_int,
                                        );
                                        syn_add_end_off(
                                            &raw mut endpos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_ME_OFF,
                                            0 as ::core::ffi::c_int,
                                        );
                                        if endpos.lnum == current_lnum
                                            && (endpos.col + syncing as ::core::ffi::c_int)
                                                < startcol
                                        {
                                            if regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .col
                                                == regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                    .col
                                            {
                                                try_next_column = true_0 != 0;
                                            }
                                            continue;
                                        }
                                    }
                                    if hl_startpos.lnum == current_lnum
                                        && hl_startpos.col < startcol
                                    {
                                        hl_startpos.col = startcol as colnr_T;
                                    }
                                    limit_pos_zero(&raw mut hl_endpos, &raw mut endpos);
                                    next_match_idx = idx;
                                    next_match_col = startcol;
                                    next_match_m_endpos = endpos;
                                    next_match_h_endpos = hl_endpos;
                                    next_match_h_startpos = hl_startpos;
                                    next_match_flags = flags;
                                    next_match_eos_pos = eos_pos;
                                    next_match_eoe_pos = eoe_pos;
                                    next_match_end_idx = end_idx;
                                    unref_extmatch(next_match_extmatch);
                                    next_match_extmatch = cur_extmatch;
                                    cur_extmatch = ::core::ptr::null_mut::<reg_extmatch_T>();
                                }
                            }
                        }
                    }
                }
                if next_match_idx >= 0 as ::core::ffi::c_int && next_match_col == current_col {
                    let mut lspp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
                    lspp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(next_match_idx as isize);
                    if next_match_m_endpos.lnum == current_lnum
                        && next_match_m_endpos.col == current_col
                        && !(*lspp).sp_next_list.is_null()
                    {
                        current_next_list = (*lspp).sp_next_list;
                        current_next_flags = (*lspp).sp_flags;
                        keep_next_list = true_0 != 0;
                        zero_width_next_list = true_0 != 0;
                        ga_grow(&raw mut zero_width_next_ga, 1 as ::core::ffi::c_int);
                        *(zero_width_next_ga.ga_data as *mut ::core::ffi::c_int)
                            .offset(zero_width_next_ga.ga_len as isize) = next_match_idx;
                        zero_width_next_ga.ga_len += 1;
                        next_match_idx = -1 as ::core::ffi::c_int;
                    } else {
                        cur_si = push_next_match();
                    }
                    found_match = true_0 != 0;
                }
            }
        }
        if !current_next_list.is_null() && !keep_next_list {
            if !found_match {
                line = syn_getcurline();
                if current_next_flags & HL_SKIPWHITE as ::core::ffi::c_int != 0
                    && ascii_iswhite(*line.offset(current_col as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                    || current_next_flags & HL_SKIPEMPTY as ::core::ffi::c_int != 0
                        && *line as ::core::ffi::c_int == NUL
                {
                    break;
                }
            }
            current_next_list = ::core::ptr::null_mut::<int16_t>();
            next_match_idx = -1 as ::core::ffi::c_int;
            if !zero_width_next_list {
                found_match = true_0 != 0;
            }
        }
        if !found_match {
            break;
        }
    }
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    current_attr = 0 as ::core::ffi::c_int;
    current_id = 0 as ::core::ffi::c_int;
    current_trans_id = 0 as ::core::ffi::c_int;
    current_flags = 0 as ::core::ffi::c_int;
    current_seqnr = 0 as ::core::ffi::c_int;
    if !cur_si.is_null() {
        let mut idx_0: ::core::ffi::c_int = current_state.ga_len - 1 as ::core::ffi::c_int;
        while idx_0 >= 0 as ::core::ffi::c_int {
            sip = (current_state.ga_data as *mut stateitem_T).offset(idx_0 as isize);
            if (current_lnum > (*sip).si_h_startpos.lnum
                || current_lnum == (*sip).si_h_startpos.lnum
                    && current_col >= (*sip).si_h_startpos.col)
                && ((*sip).si_h_endpos.lnum == 0 as linenr_T
                    || current_lnum < (*sip).si_h_endpos.lnum
                    || current_lnum == (*sip).si_h_endpos.lnum
                        && current_col < (*sip).si_h_endpos.col)
            {
                current_attr = (*sip).si_attr;
                current_id = (*sip).si_id;
                current_trans_id = (*sip).si_trans_id;
                current_flags = (*sip).si_flags;
                current_seqnr = (*sip).si_seqnr;
                current_sub_char = (*sip).si_cchar;
                break;
            } else {
                idx_0 -= 1;
            }
        }
        if !can_spell.is_null() {
            let mut sps: sp_syn = sp_syn {
                inc_tag: 0,
                id: 0,
                cont_in_list: ::core::ptr::null_mut::<int16_t>(),
            };
            if (*syn_block).b_spell_cluster_id == 0 as ::core::ffi::c_int {
                if (*syn_block).b_nospell_cluster_id == 0 as ::core::ffi::c_int
                    || current_trans_id == 0 as ::core::ffi::c_int
                {
                    *can_spell = (*syn_block).b_syn_spell != SYNSPL_NOTOP;
                } else {
                    sps.inc_tag = 0 as ::core::ffi::c_int;
                    sps.id = (*syn_block).b_nospell_cluster_id as int16_t;
                    sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                    *can_spell = in_id_list(
                        sip,
                        (*sip).si_cont_list,
                        &raw mut sps,
                        0 as ::core::ffi::c_int,
                    ) == 0;
                }
            } else if current_trans_id == 0 as ::core::ffi::c_int {
                *can_spell = (*syn_block).b_syn_spell == SYNSPL_TOP;
            } else {
                sps.inc_tag = 0 as ::core::ffi::c_int;
                sps.id = (*syn_block).b_spell_cluster_id as int16_t;
                sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                *can_spell = in_id_list(
                    sip,
                    (*sip).si_cont_list,
                    &raw mut sps,
                    0 as ::core::ffi::c_int,
                ) != 0;
                if (*syn_block).b_nospell_cluster_id != 0 as ::core::ffi::c_int {
                    sps.id = (*syn_block).b_nospell_cluster_id as int16_t;
                    if in_id_list(
                        sip,
                        (*sip).si_cont_list,
                        &raw mut sps,
                        0 as ::core::ffi::c_int,
                    ) != 0
                    {
                        *can_spell = false_0 != 0;
                    }
                }
            }
        }
        if !syncing && !keep_state {
            check_state_ends();
            if !(current_state.ga_len <= 0 as ::core::ffi::c_int)
                && *syn_getcurline().offset(current_col as isize) as ::core::ffi::c_int != NUL
            {
                current_col += 1;
                check_state_ends();
                current_col -= 1;
            }
        }
    } else if !can_spell.is_null() {
        *can_spell = if (*syn_block).b_syn_spell == SYNSPL_DEFAULT {
            ((*syn_block).b_spell_cluster_id == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((*syn_block).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
        } != 0;
    }
    if !current_next_list.is_null()
        && {
            line = syn_getcurline();
            *line.offset(current_col as isize) as ::core::ffi::c_int != NUL
        }
        && *line.offset((current_col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == NUL
        && current_next_flags
            & (HL_SKIPNL as ::core::ffi::c_int | HL_SKIPEMPTY as ::core::ffi::c_int)
            == 0
    {
        current_next_list = ::core::ptr::null_mut::<int16_t>();
    }
    if !(zero_width_next_ga.ga_len <= 0 as ::core::ffi::c_int) {
        ga_clear(&raw mut zero_width_next_ga);
    }
    unref_extmatch(re_extmatch_out);
    re_extmatch_out = ::core::ptr::null_mut::<reg_extmatch_T>();
    unref_extmatch(cur_extmatch);
    return current_attr;
}
unsafe extern "C" fn did_match_already(
    mut idx: ::core::ffi::c_int,
    mut gap: *mut garray_T,
) -> bool {
    let mut i: ::core::ffi::c_int = current_state.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_m_startcol
            == current_col
            && (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_m_lnum
                == current_lnum as ::core::ffi::c_int
            && (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_idx == idx
        {
            return true_0 != 0;
        }
    }
    let mut i_0: ::core::ffi::c_int = (*gap).ga_len;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        if *((*gap).ga_data as *mut ::core::ffi::c_int).offset(i_0 as isize) == idx {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn push_next_match() -> *mut stateitem_T {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut save_flags: ::core::ffi::c_int = 0;
    spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(next_match_idx as isize);
    push_current_state(next_match_idx);
    cur_si = (current_state.ga_data as *mut stateitem_T)
        .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
    (*cur_si).si_h_startpos = next_match_h_startpos;
    (*cur_si).si_m_startcol = current_col as ::core::ffi::c_int;
    (*cur_si).si_m_lnum = current_lnum as ::core::ffi::c_int;
    (*cur_si).si_flags = (*spp).sp_flags;
    let c2rust_fresh4 = next_seqnr;
    next_seqnr = next_seqnr + 1;
    (*cur_si).si_seqnr = c2rust_fresh4;
    (*cur_si).si_cchar = (*spp).sp_cchar;
    if current_state.ga_len > 1 as ::core::ffi::c_int {
        (*cur_si).si_flags |= (*(current_state.ga_data as *mut stateitem_T)
            .offset((current_state.ga_len - 2 as ::core::ffi::c_int) as isize))
        .si_flags
            & HL_CONCEAL as ::core::ffi::c_int;
    }
    (*cur_si).si_next_list = (*spp).sp_next_list;
    (*cur_si).si_extmatch = ref_extmatch(next_match_extmatch);
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
        && (*spp).sp_flags & HL_ONELINE as ::core::ffi::c_int == 0
    {
        update_si_end(cur_si, next_match_m_endpos.col, true_0 != 0);
        check_keepend();
    } else {
        (*cur_si).si_m_endpos = next_match_m_endpos;
        (*cur_si).si_h_endpos = next_match_h_endpos;
        (*cur_si).si_ends = true_0;
        (*cur_si).si_flags |= next_match_flags;
        (*cur_si).si_eoe_pos = next_match_eoe_pos;
        (*cur_si).si_end_idx = next_match_end_idx;
    }
    if keepend_level < 0 as ::core::ffi::c_int
        && (*cur_si).si_flags & HL_KEEPEND as ::core::ffi::c_int != 0
    {
        keepend_level = current_state.ga_len - 1 as ::core::ffi::c_int;
    }
    check_keepend();
    update_si_attr(current_state.ga_len - 1 as ::core::ffi::c_int);
    save_flags = (*cur_si).si_flags
        & (HL_CONCEAL as ::core::ffi::c_int | HL_CONCEALENDS as ::core::ffi::c_int);
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
        && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        push_current_state(next_match_idx);
        cur_si = (current_state.ga_data as *mut stateitem_T)
            .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
        (*cur_si).si_h_startpos = next_match_h_startpos;
        (*cur_si).si_m_startcol = current_col as ::core::ffi::c_int;
        (*cur_si).si_m_lnum = current_lnum as ::core::ffi::c_int;
        (*cur_si).si_m_endpos = next_match_eos_pos;
        (*cur_si).si_h_endpos = next_match_eos_pos;
        (*cur_si).si_ends = true_0;
        (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
        (*cur_si).si_flags = HL_MATCH as ::core::ffi::c_int;
        let c2rust_fresh5 = next_seqnr;
        next_seqnr = next_seqnr + 1;
        (*cur_si).si_seqnr = c2rust_fresh5;
        (*cur_si).si_flags |= save_flags;
        if (*cur_si).si_flags & HL_CONCEALENDS as ::core::ffi::c_int != 0 {
            (*cur_si).si_flags |= HL_CONCEAL as ::core::ffi::c_int;
        }
        (*cur_si).si_next_list = ::core::ptr::null_mut::<int16_t>();
        check_keepend();
        update_si_attr(current_state.ga_len - 1 as ::core::ffi::c_int);
    }
    next_match_idx = -1 as ::core::ffi::c_int;
    return cur_si;
}
unsafe extern "C" fn check_state_ends() {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut had_extend: ::core::ffi::c_int = 0;
    cur_si = (current_state.ga_data as *mut stateitem_T)
        .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
    while (*cur_si).si_ends != 0
        && ((*cur_si).si_m_endpos.lnum < current_lnum
            || (*cur_si).si_m_endpos.lnum == current_lnum
                && (*cur_si).si_m_endpos.col <= current_col)
    {
        if (*cur_si).si_end_idx != 0
            && ((*cur_si).si_eoe_pos.lnum > current_lnum
                || (*cur_si).si_eoe_pos.lnum == current_lnum
                    && (*cur_si).si_eoe_pos.col > current_col)
        {
            (*cur_si).si_idx = (*cur_si).si_end_idx;
            (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
            (*cur_si).si_m_endpos = (*cur_si).si_eoe_pos;
            (*cur_si).si_h_endpos = (*cur_si).si_eoe_pos;
            (*cur_si).si_flags |= HL_MATCH as ::core::ffi::c_int;
            let c2rust_fresh0 = next_seqnr;
            next_seqnr = next_seqnr + 1;
            (*cur_si).si_seqnr = c2rust_fresh0;
            if (*cur_si).si_flags & HL_CONCEALENDS as ::core::ffi::c_int != 0 {
                (*cur_si).si_flags |= HL_CONCEAL as ::core::ffi::c_int;
            }
            update_si_attr(current_state.ga_len - 1 as ::core::ffi::c_int);
            current_next_list = ::core::ptr::null_mut::<int16_t>();
            next_match_idx = 0 as ::core::ffi::c_int;
            next_match_col = MAXCOL as ::core::ffi::c_int;
            break;
        } else {
            current_next_list = (*cur_si).si_next_list;
            current_next_flags = (*cur_si).si_flags;
            if current_next_flags
                & (HL_SKIPNL as ::core::ffi::c_int | HL_SKIPEMPTY as ::core::ffi::c_int)
                == 0
                && *syn_getcurline().offset(current_col as isize) as ::core::ffi::c_int == NUL
            {
                current_next_list = ::core::ptr::null_mut::<int16_t>();
            }
            had_extend = (*cur_si).si_flags & HL_EXTEND as ::core::ffi::c_int;
            pop_current_state();
            if current_state.ga_len <= 0 as ::core::ffi::c_int {
                break;
            }
            if had_extend != 0 && keepend_level >= 0 as ::core::ffi::c_int {
                syn_update_ends(false_0 != 0);
                if current_state.ga_len <= 0 as ::core::ffi::c_int {
                    break;
                }
            }
            cur_si = (current_state.ga_data as *mut stateitem_T)
                .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize);
            if !((*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_START
                && (*cur_si).si_flags
                    & (HL_MATCH as ::core::ffi::c_int | HL_KEEPEND as ::core::ffi::c_int)
                    == 0)
            {
                continue;
            }
            update_si_end(cur_si, current_col, true_0 != 0);
            check_keepend();
            if current_next_flags & HL_HAS_EOL as ::core::ffi::c_int != 0
                && keepend_level < 0 as ::core::ffi::c_int
                && *syn_getcurline().offset(current_col as isize) as ::core::ffi::c_int == NUL
            {
                break;
            }
        }
    }
}
unsafe extern "C" fn update_si_attr(mut idx: ::core::ffi::c_int) {
    let mut sip: *mut stateitem_T =
        (current_state.ga_data as *mut stateitem_T).offset(idx as isize);
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if (*sip).si_idx < 0 as ::core::ffi::c_int {
        return;
    }
    spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset((*sip).si_idx as isize);
    if (*sip).si_flags & HL_MATCH as ::core::ffi::c_int != 0 {
        (*sip).si_id = (*spp).sp_syn_match_id as ::core::ffi::c_int;
    } else {
        (*sip).si_id = (*spp).sp_syn.id as ::core::ffi::c_int;
    }
    (*sip).si_attr = syn_id2attr((*sip).si_id);
    (*sip).si_trans_id = (*sip).si_id;
    if (*sip).si_flags & HL_MATCH as ::core::ffi::c_int != 0 {
        (*sip).si_cont_list = ::core::ptr::null_mut::<int16_t>();
    } else {
        (*sip).si_cont_list = (*spp).sp_cont_list;
    }
    if (*spp).sp_flags & HL_TRANSP as ::core::ffi::c_int != 0
        && (*sip).si_flags & HL_MATCH as ::core::ffi::c_int == 0
    {
        if idx == 0 as ::core::ffi::c_int {
            (*sip).si_attr = 0 as ::core::ffi::c_int;
            (*sip).si_trans_id = 0 as ::core::ffi::c_int;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_cont_list = ID_LIST_ALL;
            }
        } else {
            (*sip).si_attr = (*(current_state.ga_data as *mut stateitem_T)
                .offset((idx - 1 as ::core::ffi::c_int) as isize))
            .si_attr;
            (*sip).si_trans_id = (*(current_state.ga_data as *mut stateitem_T)
                .offset((idx - 1 as ::core::ffi::c_int) as isize))
            .si_trans_id;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_flags |= HL_TRANS_CONT as ::core::ffi::c_int;
                (*sip).si_cont_list = (*(current_state.ga_data as *mut stateitem_T)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize))
                .si_cont_list;
            }
        }
    }
}
unsafe extern "C" fn check_keepend() {
    let mut i: ::core::ffi::c_int = 0;
    let mut maxpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut maxpos_h: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    if keepend_level < 0 as ::core::ffi::c_int {
        return;
    }
    i = current_state.ga_len - 1 as ::core::ffi::c_int;
    while i > keepend_level {
        if (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_flags
            & HL_EXTEND as ::core::ffi::c_int
            != 0
        {
            break;
        }
        i -= 1;
    }
    maxpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
    maxpos.col = 0 as ::core::ffi::c_int as colnr_T;
    maxpos_h.lnum = 0 as ::core::ffi::c_int as linenr_T;
    maxpos_h.col = 0 as ::core::ffi::c_int as colnr_T;
    while i < current_state.ga_len {
        sip = (current_state.ga_data as *mut stateitem_T).offset(i as isize);
        if maxpos.lnum != 0 as linenr_T {
            limit_pos_zero(&raw mut (*sip).si_m_endpos, &raw mut maxpos);
            limit_pos_zero(&raw mut (*sip).si_h_endpos, &raw mut maxpos_h);
            limit_pos_zero(&raw mut (*sip).si_eoe_pos, &raw mut maxpos);
            (*sip).si_ends = true_0;
        }
        if (*sip).si_ends != 0 && (*sip).si_flags & HL_KEEPEND as ::core::ffi::c_int != 0 {
            if maxpos.lnum == 0 as linenr_T
                || maxpos.lnum > (*sip).si_m_endpos.lnum
                || maxpos.lnum == (*sip).si_m_endpos.lnum && maxpos.col > (*sip).si_m_endpos.col
            {
                maxpos = (*sip).si_m_endpos;
            }
            if maxpos_h.lnum == 0 as linenr_T
                || maxpos_h.lnum > (*sip).si_h_endpos.lnum
                || maxpos_h.lnum == (*sip).si_h_endpos.lnum && maxpos_h.col > (*sip).si_h_endpos.col
            {
                maxpos_h = (*sip).si_h_endpos;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn update_si_end(
    mut sip: *mut stateitem_T,
    mut startcol: ::core::ffi::c_int,
    mut force: bool,
) {
    let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut end_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    if (*sip).si_idx < 0 as ::core::ffi::c_int {
        return;
    }
    if !force && (*sip).si_m_endpos.lnum >= current_lnum {
        return;
    }
    let mut end_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut startpos: lpos_T = lpos_T {
        lnum: current_lnum,
        col: startcol as colnr_T,
    };
    let mut endpos: lpos_T = lpos_T {
        lnum: 0 as linenr_T,
        col: 0,
    };
    find_endpos(
        (*sip).si_idx,
        &raw mut startpos,
        &raw mut endpos,
        &raw mut hl_endpos,
        &raw mut (*sip).si_flags,
        &raw mut end_endpos,
        &raw mut end_idx,
        (*sip).si_extmatch,
    );
    if endpos.lnum == 0 as linenr_T {
        if (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset((*sip).si_idx as isize))
            .sp_flags
            & HL_ONELINE as ::core::ffi::c_int
            != 0
        {
            (*sip).si_ends = true_0;
            (*sip).si_m_endpos.lnum = current_lnum;
            (*sip).si_m_endpos.col = syn_getcurline_len();
        } else {
            (*sip).si_ends = false_0;
            (*sip).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        (*sip).si_h_endpos = (*sip).si_m_endpos;
    } else {
        (*sip).si_m_endpos = endpos;
        (*sip).si_h_endpos = hl_endpos;
        (*sip).si_eoe_pos = end_endpos;
        (*sip).si_ends = true_0;
        (*sip).si_end_idx = end_idx;
    };
}
unsafe extern "C" fn push_current_state(mut idx: ::core::ffi::c_int) {
    let mut p: *mut stateitem_T = ga_append_via_ptr(
        &raw mut current_state,
        ::core::mem::size_of::<stateitem_T>(),
    ) as *mut stateitem_T;
    memset(
        p as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<stateitem_T>(),
    );
    (*p).si_idx = idx;
}
unsafe extern "C" fn pop_current_state() {
    if !(current_state.ga_len <= 0 as ::core::ffi::c_int) {
        unref_extmatch(
            (*(current_state.ga_data as *mut stateitem_T)
                .offset((current_state.ga_len - 1 as ::core::ffi::c_int) as isize))
            .si_extmatch,
        );
        current_state.ga_len -= 1;
    }
    next_match_idx = -1 as ::core::ffi::c_int;
    if keepend_level >= current_state.ga_len {
        keepend_level = -1 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn find_endpos(
    mut idx: ::core::ffi::c_int,
    mut startpos: *mut lpos_T,
    mut m_endpos: *mut lpos_T,
    mut hl_endpos: *mut lpos_T,
    mut flagsp: *mut ::core::ffi::c_int,
    mut end_endpos: *mut lpos_T,
    mut end_idx: *mut ::core::ffi::c_int,
    mut start_ext: *mut reg_extmatch_T,
) {
    let mut spp_skip: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut best_idx: ::core::ffi::c_int = 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut best_regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut had_match: bool = false_0 != 0;
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    if idx < 0 as ::core::ffi::c_int {
        return;
    }
    let mut spp: *mut synpat_T =
        ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
    if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
        *hl_endpos = *startpos;
        return;
    }
    loop {
        spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
            break;
        }
        idx += 1;
    }
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_SKIP {
        spp_skip = spp;
        idx += 1;
    } else {
        spp_skip = ::core::ptr::null_mut::<synpat_T>();
    }
    unref_extmatch(re_extmatch_in);
    re_extmatch_in = ref_extmatch(start_ext);
    let mut matchcol: colnr_T = (*startpos).col;
    let mut start_idx: ::core::ffi::c_int = idx;
    best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col =
        0 as ::core::ffi::c_int as colnr_T;
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    loop {
        best_idx = -1 as ::core::ffi::c_int;
        idx = start_idx;
        while idx < (*syn_block).b_syn_patterns.ga_len {
            let mut lc_col: ::core::ffi::c_int = matchcol as ::core::ffi::c_int;
            spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
            if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_END {
                break;
            }
            lc_col -= (*spp).sp_offsets[SPO_LC_OFF as usize];
            if lc_col < 0 as ::core::ffi::c_int {
                lc_col = 0 as ::core::ffi::c_int;
            }
            regmatch.rmm_ic = (*spp).sp_ic;
            regmatch.regprog = (*spp).sp_prog;
            let mut r: bool = syn_regexec(
                &raw mut regmatch,
                (*startpos).lnum,
                lc_col as colnr_T,
                &raw mut (*spp).sp_time,
            );
            (*spp).sp_prog = regmatch.regprog;
            if r {
                if best_idx == -1 as ::core::ffi::c_int
                    || regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                        < best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                {
                    best_idx = idx;
                    best_regmatch.startpos[0 as ::core::ffi::c_int as usize] =
                        regmatch.startpos[0 as ::core::ffi::c_int as usize];
                    best_regmatch.endpos[0 as ::core::ffi::c_int as usize] =
                        regmatch.endpos[0 as ::core::ffi::c_int as usize];
                }
            }
            idx += 1;
        }
        if best_idx == -1 as ::core::ffi::c_int {
            break;
        }
        if !spp_skip.is_null() {
            let mut lc_col_0: ::core::ffi::c_int =
                matchcol as ::core::ffi::c_int - (*spp_skip).sp_offsets[SPO_LC_OFF as usize];
            if lc_col_0 < 0 as ::core::ffi::c_int {
                lc_col_0 = 0 as ::core::ffi::c_int;
            }
            regmatch.rmm_ic = (*spp_skip).sp_ic;
            regmatch.regprog = (*spp_skip).sp_prog;
            let mut r_0: ::core::ffi::c_int = syn_regexec(
                &raw mut regmatch,
                (*startpos).lnum,
                lc_col_0 as colnr_T,
                &raw mut (*spp_skip).sp_time,
            ) as ::core::ffi::c_int;
            (*spp_skip).sp_prog = regmatch.regprog;
            if r_0 != 0
                && regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                    <= best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
            {
                syn_add_end_off(
                    &raw mut pos,
                    &raw mut regmatch,
                    spp_skip,
                    SPO_ME_OFF,
                    1 as ::core::ffi::c_int,
                );
                if pos.lnum > (*startpos).lnum {
                    break;
                }
                let mut line_len: ::core::ffi::c_int = ml_get_buf_len(syn_buf, (*startpos).lnum);
                if pos.col <= matchcol {
                    matchcol += 1;
                } else if pos.col <= regmatch.endpos[0 as ::core::ffi::c_int as usize].col {
                    matchcol = pos.col;
                } else {
                    matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                    while matchcol < line_len && matchcol < pos.col {
                        matchcol += 1;
                    }
                }
                if matchcol >= line_len {
                    break;
                } else {
                    continue;
                }
            }
        }
        spp = ((*syn_block).b_syn_patterns.ga_data as *mut synpat_T).offset(best_idx as isize);
        syn_add_end_off(
            m_endpos,
            &raw mut best_regmatch,
            spp,
            SPO_ME_OFF,
            1 as ::core::ffi::c_int,
        );
        if (*m_endpos).lnum == (*startpos).lnum && (*m_endpos).col < (*startpos).col {
            (*m_endpos).col = (*startpos).col;
        }
        syn_add_end_off(
            end_endpos,
            &raw mut best_regmatch,
            spp,
            SPO_HE_OFF,
            1 as ::core::ffi::c_int,
        );
        if (*end_endpos).lnum == (*startpos).lnum && (*end_endpos).col < (*startpos).col {
            (*end_endpos).col = (*startpos).col;
        }
        limit_pos(end_endpos, m_endpos);
        if (*spp).sp_syn_match_id as ::core::ffi::c_int != (*spp).sp_syn.id as ::core::ffi::c_int
            && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *end_idx = best_idx;
            if (*spp).sp_off_flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << SPO_RE_OFF + SPO_COUNT
                != 0
            {
                (*hl_endpos).lnum = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                (*hl_endpos).col = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*hl_endpos).lnum = best_regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                (*hl_endpos).col = best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
            }
            (*hl_endpos).col += (*spp).sp_offsets[SPO_RE_OFF as usize];
            if (*hl_endpos).lnum == (*startpos).lnum && (*hl_endpos).col < (*startpos).col {
                (*hl_endpos).col = (*startpos).col;
            }
            limit_pos(hl_endpos, m_endpos);
            *m_endpos = *hl_endpos;
        } else {
            *end_idx = 0 as ::core::ffi::c_int;
            *hl_endpos = *end_endpos;
        }
        *flagsp = (*spp).sp_flags;
        had_match = true_0 != 0;
        break;
    }
    if !had_match {
        (*m_endpos).lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    unref_extmatch(re_extmatch_in);
    re_extmatch_in = ::core::ptr::null_mut::<reg_extmatch_T>();
}
unsafe extern "C" fn limit_pos(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    if (*pos).lnum > (*limit).lnum {
        *pos = *limit;
    } else if (*pos).lnum == (*limit).lnum && (*pos).col > (*limit).col {
        (*pos).col = (*limit).col;
    }
}
unsafe extern "C" fn limit_pos_zero(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    if (*pos).lnum == 0 as linenr_T {
        *pos = *limit;
    } else {
        limit_pos(pos, limit);
    };
}
unsafe extern "C" fn syn_add_end_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    let mut col: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx != 0 {
        (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize] + extra;
    } else {
        (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize];
    }
    if (*result).lnum > (*syn_buf).b_ml.ml_line_count {
        col = 0 as ::core::ffi::c_int;
    } else if off != 0 as ::core::ffi::c_int {
        base = ml_get_buf(syn_buf, (*result).lnum);
        p = base.offset(col as isize);
        if off > 0 as ::core::ffi::c_int {
            loop {
                let c2rust_fresh1 = off;
                off = off - 1;
                if !(c2rust_fresh1 > 0 as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL) {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        } else {
            loop {
                let c2rust_fresh2 = off;
                off = off + 1;
                if !(c2rust_fresh2 < 0 as ::core::ffi::c_int && base < p) {
                    break;
                }
                p = p.offset(
                    -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
        }
        col = p.offset_from(base) as ::core::ffi::c_int;
    }
    (*result).col = col as colnr_T;
}
unsafe extern "C" fn syn_add_start_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    let mut col: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx + SPO_COUNT != 0
    {
        (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize] + extra;
    } else {
        (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize];
    }
    if (*result).lnum > (*syn_buf).b_ml.ml_line_count {
        (*result).lnum = (*syn_buf).b_ml.ml_line_count;
        col = ml_get_buf_len(syn_buf, (*result).lnum) as ::core::ffi::c_int;
    }
    if off != 0 as ::core::ffi::c_int {
        base = ml_get_buf(syn_buf, (*result).lnum);
        p = base.offset(col as isize);
        if off > 0 as ::core::ffi::c_int {
            loop {
                let c2rust_fresh6 = off;
                off = off - 1;
                if !(c2rust_fresh6 != 0 && *p as ::core::ffi::c_int != NUL) {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        } else {
            loop {
                let c2rust_fresh7 = off;
                off = off + 1;
                if !(c2rust_fresh7 != 0 && base < p) {
                    break;
                }
                p = p.offset(
                    -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
        }
        col = p.offset_from(base) as ::core::ffi::c_int;
    }
    (*result).col = col as colnr_T;
}
unsafe extern "C" fn syn_getcurline() -> *mut ::core::ffi::c_char {
    return ml_get_buf(syn_buf, current_lnum);
}
unsafe extern "C" fn syn_getcurline_len() -> colnr_T {
    return ml_get_buf_len(syn_buf, current_lnum);
}
unsafe extern "C" fn syn_regexec(
    mut rmp: *mut regmmatch_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut st: *mut syn_time_T,
) -> bool {
    let mut timed_out: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pt: proftime_T = 0;
    let l_syn_time_on: bool = syn_time_on;
    if l_syn_time_on {
        pt = profile_start();
    }
    if (*rmp).regprog.is_null() {
        return false_0 != 0;
    }
    (*rmp).rmm_maxcol = (*syn_buf).b_p_smc as colnr_T;
    let mut r: ::core::ffi::c_int =
        vim_regexec_multi(rmp, syn_win, syn_buf, lnum, col, syn_tm, &raw mut timed_out);
    if l_syn_time_on {
        pt = profile_end(pt);
        (*st).total = profile_add((*st).total, pt);
        if profile_cmp(pt, (*st).slowest) < 0 as ::core::ffi::c_int {
            (*st).slowest = pt;
        }
        (*st).count += 1;
        if r > 0 as ::core::ffi::c_int {
            (*st).match_0 += 1;
        }
    }
    if timed_out != 0 && !(*(*syn_win).w_s).b_syn_slow {
        (*(*syn_win).w_s).b_syn_slow = true_0 != 0;
        msg(
            gettext(
                b"'redrawtime' exceeded, syntax highlighting disabled\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    if r > 0 as ::core::ffi::c_int {
        (*rmp).startpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
        (*rmp).endpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn check_keyword_id(
    line: *mut ::core::ffi::c_char,
    startcol: ::core::ffi::c_int,
    endcolp: *mut ::core::ffi::c_int,
    flagsp: *mut ::core::ffi::c_int,
    next_listp: *mut *mut int16_t,
    cur_si: *mut stateitem_T,
    ccharp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let kwp: *mut ::core::ffi::c_char = line.offset(startcol as isize);
    let mut kwlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        kwlen += utfc_ptr2len(kwp.offset(kwlen as isize));
        if !vim_iswordp_buf(kwp.offset(kwlen as isize), syn_buf) {
            break;
        }
    }
    if kwlen > MAXKEYWLEN {
        return 0 as ::core::ffi::c_int;
    }
    let mut keyword: [::core::ffi::c_char; 81] = [0; 81];
    xmemcpyz(
        &raw mut keyword as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        kwp as *const ::core::ffi::c_void,
        kwlen as size_t,
    );
    let mut kp: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
    if (*syn_block).b_keywtab.ht_used != 0 as size_t {
        kp = match_keyword(
            &raw mut keyword as *mut ::core::ffi::c_char,
            &raw mut (*syn_block).b_keywtab,
            cur_si,
        );
    }
    if kp.is_null() && (*syn_block).b_keywtab_ic.ht_used != 0 as size_t {
        str_foldcase(
            kwp,
            kwlen,
            &raw mut keyword as *mut ::core::ffi::c_char,
            MAXKEYWLEN + 1 as ::core::ffi::c_int,
        );
        kp = match_keyword(
            &raw mut keyword as *mut ::core::ffi::c_char,
            &raw mut (*syn_block).b_keywtab_ic,
            cur_si,
        );
    }
    if !kp.is_null() {
        *endcolp = startcol + kwlen;
        *flagsp = (*kp).flags;
        *next_listp = (*kp).next_list;
        *ccharp = (*kp).k_char;
        return (*kp).k_syn.id as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn match_keyword(
    mut keyword: *mut ::core::ffi::c_char,
    mut ht: *mut hashtab_T,
    mut cur_si: *mut stateitem_T,
) -> *mut keyentry_T {
    let mut hi: *mut hashitem_T = hash_find(ht, keyword);
    if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
        let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
            -((&raw mut dumkey.keyword as *mut ::core::ffi::c_char)
                .offset_from(&raw mut dumkey as *mut ::core::ffi::c_char) as isize),
        ) as *mut keyentry_T;
        while !kp.is_null() {
            if if !current_next_list.is_null() {
                in_id_list(
                    ::core::ptr::null_mut::<stateitem_T>(),
                    current_next_list,
                    &raw mut (*kp).k_syn,
                    0 as ::core::ffi::c_int,
                )
            } else if cur_si.is_null() {
                ((*kp).flags & HL_CONTAINED as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            } else {
                in_id_list(
                    cur_si,
                    (*cur_si).si_cont_list,
                    &raw mut (*kp).k_syn,
                    (*kp).flags,
                )
            } != 0
            {
                return kp;
            }
            kp = (*kp).ke_next;
        }
    }
    return ::core::ptr::null_mut::<keyentry_T>();
}
unsafe extern "C" fn syn_cmd_conceal(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin).w_s).b_syn_conceal != 0 {
            msg(
                b"syntax conceal on\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax conceal off\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 2 as isize
    {
        (*(*curwin).w_s).b_syn_conceal = true_0;
    } else if strncasecmp(
        arg,
        b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        3 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 3 as isize
    {
        (*(*curwin).w_s).b_syn_conceal = false_0;
    } else {
        semsg(
            gettext(&raw const e_illegal_arg as *const ::core::ffi::c_char),
            arg,
        );
    };
}
unsafe extern "C" fn syn_cmd_case(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin).w_s).b_syn_ic != 0 {
            msg(
                b"syntax case ignore\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax case match\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 5 as isize
    {
        (*(*curwin).w_s).b_syn_ic = false_0;
    } else if strncasecmp(
        arg,
        b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        6 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 6 as isize
    {
        (*(*curwin).w_s).b_syn_ic = true_0;
    } else {
        semsg(
            gettext(&raw const e_illegal_arg as *const ::core::ffi::c_char),
            arg,
        );
    };
}
unsafe extern "C" fn syn_cmd_foldlevel(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if *arg as ::core::ffi::c_int == NUL {
        match (*(*curwin).w_s).b_syn_foldlevel {
            SYNFLD_START => {
                msg(
                    b"syntax foldlevel start\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
            SYNFLD_MINIMUM => {
                msg(
                    b"syntax foldlevel minimum\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
            _ => {}
        }
        return;
    }
    arg_end = skiptowhite(arg);
    if strncasecmp(
        arg,
        b"start\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && arg_end.offset_from(arg) == 5 as isize
    {
        (*(*curwin).w_s).b_syn_foldlevel = SYNFLD_START;
    } else if strncasecmp(
        arg,
        b"minimum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        7 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && arg_end.offset_from(arg) == 7 as isize
    {
        (*(*curwin).w_s).b_syn_foldlevel = SYNFLD_MINIMUM;
    } else {
        semsg(
            gettext(&raw const e_illegal_arg as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    arg = skipwhite(arg_end);
    if *arg as ::core::ffi::c_int != NUL {
        semsg(
            gettext(&raw const e_illegal_arg as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn syn_cmd_spell(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin).w_s).b_syn_spell == SYNSPL_TOP {
            msg(
                b"syntax spell toplevel\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else if (*(*curwin).w_s).b_syn_spell == SYNSPL_NOTOP {
            msg(
                b"syntax spell notoplevel\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax spell default\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"toplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        8 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 8 as isize
    {
        (*(*curwin).w_s).b_syn_spell = SYNSPL_TOP;
    } else if strncasecmp(
        arg,
        b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        10 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 10 as isize
    {
        (*(*curwin).w_s).b_syn_spell = SYNSPL_NOTOP;
    } else if strncasecmp(
        arg,
        b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        7 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 7 as isize
    {
        (*(*curwin).w_s).b_syn_spell = SYNSPL_DEFAULT;
    } else {
        semsg(
            gettext(&raw const e_illegal_arg as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    redraw_later(curwin, UPD_NOT_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn syn_cmd_iskeyword(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut save_chartab_0: [::core::ffi::c_char; 32] = [0; 32];
    let mut save_isk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).skip != 0 {
        return;
    }
    arg = skipwhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if (*(*curwin).w_s).b_syn_isk != &raw mut empty_string_option as *mut ::core::ffi::c_char {
            msg_puts(b"syntax iskeyword \0".as_ptr() as *const ::core::ffi::c_char);
            msg_outtrans(
                (*(*curwin).w_s).b_syn_isk,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                gettext(b"syntax iskeyword not set\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
    } else if strncasecmp(
        arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            &raw mut (*(*curwin).w_s).b_syn_chartab as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*curbuf).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        clear_string_option(&raw mut (*(*curwin).w_s).b_syn_isk);
    } else {
        memmove(
            &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            &raw mut (*curbuf).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        save_isk = (*curbuf).b_p_isk;
        (*curbuf).b_p_isk = xstrdup(arg);
        buf_init_chartab(curbuf, false_0 != 0);
        memmove(
            &raw mut (*(*curwin).w_s).b_syn_chartab as *mut uint8_t as *mut ::core::ffi::c_void,
            &raw mut (*curbuf).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        memmove(
            &raw mut (*curbuf).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        clear_string_option(&raw mut (*(*curwin).w_s).b_syn_isk);
        (*(*curwin).w_s).b_syn_isk = (*curbuf).b_p_isk;
        (*curbuf).b_p_isk = save_isk;
    }
    redraw_later(curwin, UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn syntax_clear(mut block: *mut synblock_T) {
    (*block).b_syn_error = false_0 != 0;
    (*block).b_syn_slow = false_0 != 0;
    (*block).b_syn_ic = false_0;
    (*block).b_syn_foldlevel = SYNFLD_START;
    (*block).b_syn_spell = SYNSPL_DEFAULT;
    (*block).b_syn_containedin = false_0;
    (*block).b_syn_conceal = false_0;
    clear_keywtab(&raw mut (*block).b_keywtab);
    clear_keywtab(&raw mut (*block).b_keywtab_ic);
    let mut i: ::core::ffi::c_int = (*block).b_syn_patterns.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        syn_clear_pattern(block, i);
    }
    ga_clear(&raw mut (*block).b_syn_patterns);
    let mut i_0: ::core::ffi::c_int = (*block).b_syn_clusters.ga_len;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        syn_clear_cluster(block, i_0);
    }
    ga_clear(&raw mut (*block).b_syn_clusters);
    (*block).b_spell_cluster_id = 0 as ::core::ffi::c_int;
    (*block).b_nospell_cluster_id = 0 as ::core::ffi::c_int;
    (*block).b_syn_sync_flags = 0 as ::core::ffi::c_int;
    (*block).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
    (*block).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
    (*block).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
    vim_regfree((*block).b_syn_linecont_prog);
    (*block).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*block).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    (*block).b_syn_folditems = 0 as ::core::ffi::c_int;
    clear_string_option(&raw mut (*block).b_syn_isk);
    syn_stack_free_all(block);
    invalidate_current_state();
    running_syn_inc_tag = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn reset_synblock(mut wp: *mut win_T) {
    if (*wp).w_s != &raw mut (*(*wp).w_buffer).b_s {
        syntax_clear((*wp).w_s);
        xfree((*wp).w_s as *mut ::core::ffi::c_void);
        (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
    }
}
unsafe extern "C" fn syntax_sync_clear() {
    let mut i: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_patterns.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
            .sp_syncing
        {
            syn_remove_pattern((*curwin).w_s, i);
        }
    }
    (*(*curwin).w_s).b_syn_sync_flags = 0 as ::core::ffi::c_int;
    (*(*curwin).w_s).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
    (*(*curwin).w_s).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
    (*(*curwin).w_s).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
    vim_regfree((*(*curwin).w_s).b_syn_linecont_prog);
    (*(*curwin).w_s).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*(*curwin).w_s).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    clear_string_option(&raw mut (*(*curwin).w_s).b_syn_isk);
    syn_stack_free_all((*curwin).w_s);
}
unsafe extern "C" fn syn_remove_pattern(mut block: *mut synblock_T, mut idx: ::core::ffi::c_int) {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    spp = ((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
    if (*spp).sp_flags & HL_FOLD as ::core::ffi::c_int != 0 {
        (*block).b_syn_folditems -= 1;
    }
    syn_clear_pattern(block, idx);
    memmove(
        spp as *mut ::core::ffi::c_void,
        spp.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<synpat_T>().wrapping_mul(
            ((*block).b_syn_patterns.ga_len - idx - 1 as ::core::ffi::c_int) as size_t,
        ),
    );
    (*block).b_syn_patterns.ga_len -= 1;
}
unsafe extern "C" fn syn_clear_pattern(mut block: *mut synblock_T, mut i: ::core::ffi::c_int) {
    xfree(
        (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_pattern
            as *mut ::core::ffi::c_void,
    );
    vim_regfree((*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_prog);
    if i == 0 as ::core::ffi::c_int
        || (*((*block).b_syn_patterns.ga_data as *mut synpat_T)
            .offset((i - 1 as ::core::ffi::c_int) as isize))
        .sp_type as ::core::ffi::c_int
            != SPTYPE_START
    {
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_cont_list
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_next_list
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
                .sp_syn
                .cont_in_list as *mut ::core::ffi::c_void,
        );
    }
}
unsafe extern "C" fn syn_clear_cluster(mut block: *mut synblock_T, mut i: ::core::ffi::c_int) {
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name
            as *mut ::core::ffi::c_void,
    );
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name_u
            as *mut ::core::ffi::c_void,
    );
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_list
            as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn syn_cmd_clear(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if (*(*curwin).w_s).b_syn_topgrp != 0 as ::core::ffi::c_int {
        return;
    }
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
        if syncing != 0 {
            syntax_sync_clear();
        } else {
            syntax_clear((*curwin).w_s);
            if (*curwin).w_s == &raw mut (*(*curwin).w_buffer).b_s {
                do_unlet(
                    b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                    true_0 != 0,
                );
            }
            do_unlet(
                b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        }
    } else {
        while ends_excmd(*arg as ::core::ffi::c_int) == 0 {
            arg_end = skiptowhite(arg);
            if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                id = syn_scl_namen2id(
                    arg.offset(1 as ::core::ffi::c_int as isize),
                    (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                );
                if id == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E391: No such syntax cluster: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                } else {
                    let mut scl_id: ::core::ffi::c_int = id - SYNID_CLUSTER;
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                            .offset(scl_id as isize))
                        .scl_list as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    *ptr_;
                }
            } else {
                id = syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                if id == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                } else {
                    syn_clear_one(id, syncing != 0);
                }
            }
            arg = skipwhite(arg_end);
        }
    }
    redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
    syn_stack_free_all((*curwin).w_s);
}
unsafe extern "C" fn syn_clear_one(id: ::core::ffi::c_int, syncing: bool) {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if !syncing {
        syn_clear_keyword(id, &raw mut (*(*curwin).w_s).b_keywtab);
        syn_clear_keyword(id, &raw mut (*(*curwin).w_s).b_keywtab_ic);
    }
    let mut idx: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_patterns.ga_len;
    loop {
        idx -= 1;
        if idx < 0 as ::core::ffi::c_int {
            break;
        }
        spp = ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_syn.id as ::core::ffi::c_int != id
            || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int
        {
            continue;
        }
        syn_remove_pattern((*curwin).w_s, idx);
    }
}
unsafe extern "C" fn syn_cmd_on(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_reset(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    (*eap).nextcmd = check_nextcmd((*eap).arg);
    if (*eap).skip == 0 {
        init_highlight(true_0 != 0, true_0 != 0);
    }
}
unsafe extern "C" fn syn_cmd_manual(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_off(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"nosyntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_onoff(mut eap: *mut exarg_T, mut name: *mut ::core::ffi::c_char) {
    (*eap).nextcmd = check_nextcmd((*eap).arg);
    if (*eap).skip == 0 {
        did_syntax_onoff = true_0 != 0;
        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
        memcpy(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            b"so \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            4 as size_t,
        );
        vim_snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>().wrapping_sub(3 as size_t),
            SYNTAX_FNAME.as_ptr(),
            name,
        );
        do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn syn_maybe_enable() {
    if !did_syntax_onoff {
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
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
        ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        ea.skip = false_0;
        syn_cmd_on(&raw mut ea, false_0);
    }
}
unsafe extern "C" fn syn_cmd_list(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if !syntax_present(curwin) {
        msg(
            gettext(&raw mut msg_no_items as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    if syncing != 0 {
        if (*(*curwin).w_s).b_syn_sync_flags & SF_CCOMMENT != 0 {
            msg_puts(gettext(
                b"syncing on C-style comments\0".as_ptr() as *const ::core::ffi::c_char
            ));
            syn_lines_msg();
            syn_match_msg();
        } else if (*(*curwin).w_s).b_syn_sync_flags & SF_MATCH != 0 {
            msg_puts_title(gettext(
                b"\n--- Syntax sync items ---\0".as_ptr() as *const ::core::ffi::c_char
            ));
            if (*(*curwin).w_s).b_syn_sync_minlines > 0 as linenr_T
                || (*(*curwin).w_s).b_syn_sync_maxlines > 0 as linenr_T
                || (*(*curwin).w_s).b_syn_sync_linebreaks > 0 as linenr_T
            {
                msg_puts(gettext(
                    b"\nsyncing on items\0".as_ptr() as *const ::core::ffi::c_char
                ));
                syn_lines_msg();
                syn_match_msg();
            }
            let mut id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while id <= highlight_num_groups() && !got_int {
                syn_list_one(id, syncing != 0, false_0 != 0);
                id += 1;
            }
        } else if (*(*curwin).w_s).b_syn_sync_minlines == 0 as linenr_T {
            msg_puts(gettext(
                b"no syncing\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            if (*(*curwin).w_s).b_syn_sync_minlines == MAXLNUM as ::core::ffi::c_int as linenr_T {
                msg_puts(gettext(
                    b"syncing starts at the first line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                msg_puts(gettext(
                    b"syncing starts \0".as_ptr() as *const ::core::ffi::c_char
                ));
                msg_outnum((*(*curwin).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                msg_puts(gettext(
                    b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            syn_match_msg();
        }
        return;
    }
    msg_puts_title(gettext(
        b"\n--- Syntax items ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
        let mut id_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while id_0 <= highlight_num_groups() && !got_int {
            syn_list_one(id_0, syncing != 0, false_0 != 0);
            id_0 += 1;
        }
        let mut id_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while id_1 < (*(*curwin).w_s).b_syn_clusters.ga_len && !got_int {
            syn_list_cluster(id_1);
            id_1 += 1;
        }
    } else {
        while ends_excmd(*arg as ::core::ffi::c_int) == 0 && !got_int {
            arg_end = skiptowhite(arg);
            if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                let mut id_2: ::core::ffi::c_int = syn_scl_namen2id(
                    arg.offset(1 as ::core::ffi::c_int as isize),
                    (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                );
                if id_2 == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E392: No such syntax cluster: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        arg,
                    );
                } else {
                    syn_list_cluster(id_2 - SYNID_CLUSTER);
                }
            } else {
                let mut id_3: ::core::ffi::c_int =
                    syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                if id_3 == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                        arg,
                    );
                } else {
                    syn_list_one(id_3, syncing != 0, true_0 != 0);
                }
            }
            arg = skipwhite(arg_end);
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
}
unsafe extern "C" fn syn_lines_msg() {
    if (*(*curwin).w_s).b_syn_sync_maxlines > 0 as linenr_T
        || (*(*curwin).w_s).b_syn_sync_minlines > 0 as linenr_T
    {
        msg_puts(b"; \0".as_ptr() as *const ::core::ffi::c_char);
        if (*(*curwin).w_s).b_syn_sync_minlines == MAXLNUM as ::core::ffi::c_int as linenr_T {
            msg_puts(gettext(
                b"from the first line\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            if (*(*curwin).w_s).b_syn_sync_minlines > 0 as linenr_T {
                msg_puts(gettext(b"minimal \0".as_ptr() as *const ::core::ffi::c_char));
                msg_outnum((*(*curwin).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                if (*(*curwin).w_s).b_syn_sync_maxlines != 0 {
                    msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
            if (*(*curwin).w_s).b_syn_sync_maxlines > 0 as linenr_T {
                msg_puts(gettext(b"maximal \0".as_ptr() as *const ::core::ffi::c_char));
                msg_outnum((*(*curwin).w_s).b_syn_sync_maxlines as ::core::ffi::c_int);
            }
            msg_puts(gettext(
                b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}
unsafe extern "C" fn syn_match_msg() {
    if (*(*curwin).w_s).b_syn_sync_linebreaks > 0 as linenr_T {
        msg_puts(gettext(b"; match \0".as_ptr() as *const ::core::ffi::c_char));
        msg_outnum((*(*curwin).w_s).b_syn_sync_linebreaks as ::core::ffi::c_int);
        msg_puts(gettext(
            b" line breaks\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
static mut last_matchgroup: ::core::ffi::c_int = 0;
unsafe extern "C" fn syn_list_one(id: ::core::ffi::c_int, syncing: bool, link_only: bool) {
    let mut did_header: bool = false_0 != 0;
    let hl_id: ::core::ffi::c_int = HLF_D as ::core::ffi::c_int;
    if !syncing {
        did_header =
            syn_list_keywords(id, &raw mut (*(*curwin).w_s).b_keywtab, false_0 != 0, hl_id);
        did_header = syn_list_keywords(
            id,
            &raw mut (*(*curwin).w_s).b_keywtab_ic,
            did_header,
            hl_id,
        );
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin).w_s).b_syn_patterns.ga_len && !got_int {
        let spp: *const synpat_T =
            ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if !((*spp).sp_syn.id as ::core::ffi::c_int != id
            || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int)
        {
            syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
            did_header = true_0 != 0;
            last_matchgroup = 0 as ::core::ffi::c_int;
            if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH {
                put_pattern(
                    b"match\0".as_ptr() as *const ::core::ffi::c_char,
                    ' ' as ::core::ffi::c_int,
                    spp,
                    hl_id,
                );
                msg_putchar(' ' as ::core::ffi::c_int);
            } else if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START {
                while (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_START
                {
                    let c2rust_fresh8 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"start\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh8 as isize),
                        hl_id,
                    );
                }
                if (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_SKIP
                {
                    let c2rust_fresh9 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"skip\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh9 as isize),
                        hl_id,
                    );
                }
                while idx < (*(*curwin).w_s).b_syn_patterns.ga_len
                    && (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_END
                {
                    let c2rust_fresh10 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"end\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh10 as isize),
                        hl_id,
                    );
                }
                idx -= 1;
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            syn_list_flags(
                &raw mut namelist1 as *mut keyvalue_T,
                ::core::mem::size_of::<[keyvalue_T; 10]>()
                    .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                    .wrapping_div(
                        (::core::mem::size_of::<[keyvalue_T; 10]>()
                            .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                (*spp).sp_flags,
                hl_id,
            );
            if !(*spp).sp_cont_list.is_null() {
                put_id_list(
                    b"contains\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_cont_list,
                    hl_id,
                );
            }
            if !(*spp).sp_syn.cont_in_list.is_null() {
                put_id_list(
                    b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_syn.cont_in_list,
                    hl_id,
                );
            }
            if !(*spp).sp_next_list.is_null() {
                put_id_list(
                    b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_next_list,
                    hl_id,
                );
                syn_list_flags(
                    &raw mut namelist2 as *mut keyvalue_T,
                    ::core::mem::size_of::<[keyvalue_T; 3]>()
                        .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                        .wrapping_div(
                            (::core::mem::size_of::<[keyvalue_T; 3]>()
                                .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    (*spp).sp_flags,
                    hl_id,
                );
            }
            if (*spp).sp_flags
                & (HL_SYNC_HERE as ::core::ffi::c_int | HL_SYNC_THERE as ::core::ffi::c_int)
                != 0
            {
                if (*spp).sp_flags & HL_SYNC_HERE as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        b"grouphere\0".as_ptr() as *const ::core::ffi::c_char,
                        hl_id,
                        false_0 != 0,
                    );
                } else {
                    msg_puts_hl(
                        b"groupthere\0".as_ptr() as *const ::core::ffi::c_char,
                        hl_id,
                        false_0 != 0,
                    );
                }
                msg_putchar(' ' as ::core::ffi::c_int);
                if (*spp).sp_sync_idx >= 0 as ::core::ffi::c_int {
                    msg_outtrans(
                        highlight_group_name(
                            (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset((*spp).sp_sync_idx as isize))
                            .sp_syn
                            .id as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                        ),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                } else {
                    msg_puts(b"NONE\0".as_ptr() as *const ::core::ffi::c_char);
                }
                msg_putchar(' ' as ::core::ffi::c_int);
            }
        }
        idx += 1;
    }
    if highlight_link_id(id - 1 as ::core::ffi::c_int) != 0
        && (did_header as ::core::ffi::c_int != 0 || link_only as ::core::ffi::c_int != 0)
        && !got_int
    {
        syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
        msg_puts_hl(
            b"links to\0".as_ptr() as *const ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        msg_outtrans(
            highlight_group_name(
                highlight_link_id(id - 1 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
}
unsafe extern "C" fn syn_list_flags(
    mut nlist: *mut keyvalue_T,
    mut nr_entries: size_t,
    mut flags: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut i: size_t = 0 as size_t;
    while i < nr_entries {
        if flags & (*nlist.offset(i as isize)).key != 0 {
            msg_puts_hl((*nlist.offset(i as isize)).value, hl_id, false_0 != 0);
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn syn_list_cluster(mut id: ::core::ffi::c_int) {
    let mut endcol: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    msg_putchar('\n' as ::core::ffi::c_int);
    msg_outtrans(
        (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(id as isize))
            .scl_name,
        0 as ::core::ffi::c_int,
        false_0 != 0,
    );
    if msg_col >= endcol {
        endcol = msg_col + 1 as ::core::ffi::c_int;
    }
    if Columns <= endcol {
        endcol = Columns - 1 as ::core::ffi::c_int;
    }
    msg_advance(endcol);
    if !(*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(id as isize))
        .scl_list
        .is_null()
    {
        put_id_list(
            b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
            (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(id as isize))
                .scl_list,
            HLF_D as ::core::ffi::c_int,
        );
    } else {
        msg_puts_hl(
            b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_D as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_puts(b"=NONE\0".as_ptr() as *const ::core::ffi::c_char);
    };
}
unsafe extern "C" fn put_id_list(
    name: *const ::core::ffi::c_char,
    list: *const int16_t,
    hl_id: ::core::ffi::c_int,
) {
    msg_puts_hl(name, hl_id, false_0 != 0);
    msg_putchar('=' as ::core::ffi::c_int);
    let mut p: *const int16_t = list;
    while *p != 0 {
        if *p as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
            && (*p as ::core::ffi::c_int) < SYNID_TOP
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
                msg_puts(b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                msg_puts(b"ALL\0".as_ptr() as *const ::core::ffi::c_char);
            }
        } else if *p as ::core::ffi::c_int >= SYNID_TOP
            && (*p as ::core::ffi::c_int) < SYNID_CONTAINED
        {
            msg_puts(b"TOP\0".as_ptr() as *const ::core::ffi::c_char);
        } else if *p as ::core::ffi::c_int >= SYNID_CONTAINED
            && (*p as ::core::ffi::c_int) < SYNID_CLUSTER
        {
            msg_puts(b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char);
        } else if *p as ::core::ffi::c_int >= SYNID_CLUSTER {
            let mut scl_id: ::core::ffi::c_int = *p as ::core::ffi::c_int - SYNID_CLUSTER;
            msg_putchar('@' as ::core::ffi::c_int);
            msg_outtrans(
                (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(scl_id as isize))
                .scl_name,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                highlight_group_name(*p as ::core::ffi::c_int - 1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
            msg_putchar(',' as ::core::ffi::c_int);
        }
        p = p.offset(1);
    }
    msg_putchar(' ' as ::core::ffi::c_int);
}
unsafe extern "C" fn put_pattern(
    s: *const ::core::ffi::c_char,
    c: ::core::ffi::c_int,
    spp: *const synpat_T,
    hl_id: ::core::ffi::c_int,
) {
    static mut sepchars: *const ::core::ffi::c_char =
        b"/+=-#@\"|'^&\0".as_ptr() as *const ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0;
    if last_matchgroup != (*spp).sp_syn_match_id as ::core::ffi::c_int {
        last_matchgroup = (*spp).sp_syn_match_id as ::core::ffi::c_int;
        msg_puts_hl(
            b"matchgroup\0".as_ptr() as *const ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        msg_putchar('=' as ::core::ffi::c_int);
        if last_matchgroup == 0 as ::core::ffi::c_int {
            msg_outtrans(
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                highlight_group_name(last_matchgroup - 1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    msg_puts_hl(s, hl_id, false_0 != 0);
    msg_putchar(c);
    i = 0 as ::core::ffi::c_int;
    while !vim_strchr(
        (*spp).sp_pattern,
        *sepchars.offset(i as isize) as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        i += 1;
        if *sepchars.offset(i as isize) as ::core::ffi::c_int != NUL {
            continue;
        }
        i = 0 as ::core::ffi::c_int;
        break;
    }
    msg_putchar(*sepchars.offset(i as isize) as ::core::ffi::c_int);
    msg_outtrans((*spp).sp_pattern, 0 as ::core::ffi::c_int, false_0 != 0);
    msg_putchar(*sepchars.offset(i as isize) as ::core::ffi::c_int);
    let mut first: bool = true_0 != 0;
    i = 0 as ::core::ffi::c_int;
    while i < SPO_COUNT {
        let mask: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << i;
        if (*spp).sp_off_flags as ::core::ffi::c_int & mask + (mask << SPO_COUNT) != 0 {
            if !first {
                msg_putchar(',' as ::core::ffi::c_int);
            }
            msg_puts(spo_name_tab[i as usize] as *const ::core::ffi::c_char);
            let n: ::core::ffi::c_int = (*spp).sp_offsets[i as usize];
            if i != SPO_LC_OFF {
                if (*spp).sp_off_flags as ::core::ffi::c_int & mask != 0 {
                    msg_putchar('s' as ::core::ffi::c_int);
                } else {
                    msg_putchar('e' as ::core::ffi::c_int);
                }
                if n > 0 as ::core::ffi::c_int {
                    msg_putchar('+' as ::core::ffi::c_int);
                }
            }
            if n != 0 || i == SPO_LC_OFF {
                msg_outnum(n);
            }
            first = false_0 != 0;
        }
        i += 1;
    }
    msg_putchar(' ' as ::core::ffi::c_int);
}
unsafe extern "C" fn syn_list_keywords(
    id: ::core::ffi::c_int,
    ht: *const hashtab_T,
    mut did_header: bool,
    hl_id: ::core::ffi::c_int,
) -> bool {
    let mut prev_contained: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_next_list: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut prev_cont_in_list: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut prev_skipnl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_skipwhite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_skipempty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut todo: size_t = (*ht).ht_used;
    let mut hi: *const hashitem_T = (*ht).ht_array;
    while todo > 0 as size_t && !got_int {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            todo = todo.wrapping_sub(1);
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut dumkey.keyword as *mut ::core::ffi::c_char)
                    .offset_from(&raw mut dumkey as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() && !got_int {
                if (*kp).k_syn.id as ::core::ffi::c_int == id {
                    let mut outlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut force_newline: bool = false_0 != 0;
                    if prev_contained != (*kp).flags & HL_CONTAINED as ::core::ffi::c_int
                        || prev_skipnl != (*kp).flags & HL_SKIPNL as ::core::ffi::c_int
                        || prev_skipwhite != (*kp).flags & HL_SKIPWHITE as ::core::ffi::c_int
                        || prev_skipempty != (*kp).flags & HL_SKIPEMPTY as ::core::ffi::c_int
                        || prev_cont_in_list != (*kp).k_syn.cont_in_list as *const int16_t
                        || prev_next_list != (*kp).next_list as *const int16_t
                    {
                        force_newline = true_0 != 0;
                    } else {
                        outlen = strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int;
                    }
                    if syn_list_header(did_header, outlen, id, force_newline) {
                        prev_contained = 0 as ::core::ffi::c_int;
                        prev_next_list = ::core::ptr::null::<int16_t>();
                        prev_cont_in_list = ::core::ptr::null::<int16_t>();
                        prev_skipnl = 0 as ::core::ffi::c_int;
                        prev_skipwhite = 0 as ::core::ffi::c_int;
                        prev_skipempty = 0 as ::core::ffi::c_int;
                    }
                    did_header = true_0 != 0;
                    if prev_contained != (*kp).flags & HL_CONTAINED as ::core::ffi::c_int {
                        msg_puts_hl(
                            b"contained\0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_contained = (*kp).flags & HL_CONTAINED as ::core::ffi::c_int;
                    }
                    if (*kp).k_syn.cont_in_list != prev_cont_in_list as *mut int16_t {
                        put_id_list(
                            b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                            (*kp).k_syn.cont_in_list,
                            hl_id,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_cont_in_list = (*kp).k_syn.cont_in_list;
                    }
                    if (*kp).next_list != prev_next_list as *mut int16_t {
                        put_id_list(
                            b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                            (*kp).next_list,
                            hl_id,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_next_list = (*kp).next_list;
                        if (*kp).flags & HL_SKIPNL as ::core::ffi::c_int != 0 {
                            msg_puts_hl(
                                b"skipnl\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipnl = (*kp).flags & HL_SKIPNL as ::core::ffi::c_int;
                        }
                        if (*kp).flags & HL_SKIPWHITE as ::core::ffi::c_int != 0 {
                            msg_puts_hl(
                                b"skipwhite\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipwhite = (*kp).flags & HL_SKIPWHITE as ::core::ffi::c_int;
                        }
                        if (*kp).flags & HL_SKIPEMPTY as ::core::ffi::c_int != 0 {
                            msg_puts_hl(
                                b"skipempty\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipempty = (*kp).flags & HL_SKIPEMPTY as ::core::ffi::c_int;
                        }
                    }
                    msg_outtrans(
                        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                }
                kp = (*kp).ke_next;
            }
        }
        hi = hi.offset(1);
    }
    return did_header;
}
unsafe extern "C" fn syn_clear_keyword(mut id: ::core::ffi::c_int, mut ht: *mut hashtab_T) {
    hash_lock(ht);
    let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            todo -= 1;
            let mut kp_prev: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut dumkey.keyword as *mut ::core::ffi::c_char)
                    .offset_from(&raw mut dumkey as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() {
                if (*kp).k_syn.id as ::core::ffi::c_int == id {
                    let mut kp_next: *mut keyentry_T = (*kp).ke_next;
                    if kp_prev.is_null() {
                        if kp_next.is_null() {
                            hash_remove(ht, hi);
                        } else {
                            (*hi).hi_key = &raw mut (*kp_next).keyword as *mut ::core::ffi::c_char;
                        }
                    } else {
                        (*kp_prev).ke_next = kp_next;
                    }
                    xfree((*kp).next_list as *mut ::core::ffi::c_void);
                    xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                    xfree(kp as *mut ::core::ffi::c_void);
                    kp = kp_next;
                } else {
                    kp_prev = kp;
                    kp = (*kp).ke_next;
                }
            }
        }
        hi = hi.offset(1);
    }
    hash_unlock(ht);
}
unsafe extern "C" fn clear_keywtab(mut ht: *mut hashtab_T) {
    let mut kp_next: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
    let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            todo -= 1;
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut dumkey.keyword as *mut ::core::ffi::c_char)
                    .offset_from(&raw mut dumkey as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() {
                kp_next = (*kp).ke_next;
                xfree((*kp).next_list as *mut ::core::ffi::c_void);
                xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                xfree(kp as *mut ::core::ffi::c_void);
                kp = kp_next;
            }
        }
        hi = hi.offset(1);
    }
    hash_clear(ht);
    hash_init(ht);
}
unsafe extern "C" fn add_keyword(
    name: *mut ::core::ffi::c_char,
    mut namelen: size_t,
    id: ::core::ffi::c_int,
    flags: ::core::ffi::c_int,
    cont_in_list: *mut int16_t,
    next_list: *mut int16_t,
    conceal_char: ::core::ffi::c_int,
) {
    let mut name_folded: [::core::ffi::c_char; 81] = [0; 81];
    let mut name_ic: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name_iclen: size_t = 0;
    if (*(*curwin).w_s).b_syn_ic != 0 {
        name_ic = str_foldcase(
            name,
            namelen as ::core::ffi::c_int,
            &raw mut name_folded as *mut ::core::ffi::c_char,
            MAXKEYWLEN + 1 as ::core::ffi::c_int,
        );
        name_iclen = strlen(name_ic);
    } else {
        name_ic = name;
        name_iclen = namelen;
    }
    let kp: *mut keyentry_T = xmalloc(
        (40 as size_t)
            .wrapping_add(name_iclen)
            .wrapping_add(1 as size_t),
    ) as *mut keyentry_T;
    strcpy(
        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
        name_ic as *mut ::core::ffi::c_char,
    );
    (*kp).k_syn.id = id as int16_t;
    (*kp).k_syn.inc_tag = current_syn_inc_tag;
    (*kp).flags = flags;
    (*kp).k_char = conceal_char;
    (*kp).k_syn.cont_in_list = copy_id_list(cont_in_list);
    if !cont_in_list.is_null() {
        (*(*curwin).w_s).b_syn_containedin = true_0;
    }
    (*kp).next_list = copy_id_list(next_list);
    let hash: hash_T = hash_hash(&raw mut (*kp).keyword as *mut ::core::ffi::c_char);
    let ht: *mut hashtab_T = if (*(*curwin).w_s).b_syn_ic != 0 {
        &raw mut (*(*curwin).w_s).b_keywtab_ic
    } else {
        &raw mut (*(*curwin).w_s).b_keywtab
    };
    let hi: *mut hashitem_T = hash_lookup(
        ht,
        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
        strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char),
        hash,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed {
        (*kp).ke_next = ::core::ptr::null_mut::<keyentry_T>();
        hash_add_item(
            ht,
            hi,
            &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
            hash,
        );
    } else {
        (*kp).ke_next = (*hi).hi_key.offset(
            -((&raw mut dumkey.keyword as *mut ::core::ffi::c_char)
                .offset_from(&raw mut dumkey as *mut ::core::ffi::c_char) as isize),
        ) as *mut keyentry_T;
        (*hi).hi_key = &raw mut (*kp).keyword as *mut ::core::ffi::c_char;
    };
}
unsafe extern "C" fn get_group_name(
    mut arg: *mut ::core::ffi::c_char,
    mut name_end: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    *name_end = skiptowhite(arg);
    let mut rest: *mut ::core::ffi::c_char = skipwhite(*name_end);
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 || *rest as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return rest;
}
unsafe extern "C" fn get_syn_options(
    mut arg: *mut ::core::ffi::c_char,
    mut opt: *mut syn_opt_arg_T,
    mut conceal_char: *mut ::core::ffi::c_int,
    mut skip: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fidx: ::core::ffi::c_int = 0;
    static mut flagtab: [flag; 19] = [
        flag {
            name: b"cCoOnNtTaAiInNeEdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONTAINED as ::core::ffi::c_int,
        },
        flag {
            name: b"oOnNeElLiInNeE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_ONELINE as ::core::ffi::c_int,
        },
        flag {
            name: b"kKeEeEpPeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_KEEPEND as ::core::ffi::c_int,
        },
        flag {
            name: b"eExXtTeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_EXTEND as ::core::ffi::c_int,
        },
        flag {
            name: b"eExXcClLuUdDeEnNlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_EXCLUDENL as ::core::ffi::c_int,
        },
        flag {
            name: b"tTrRaAnNsSpPaArReEnNtT\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_TRANSP as ::core::ffi::c_int,
        },
        flag {
            name: b"sSkKiIpPnNlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPNL as ::core::ffi::c_int,
        },
        flag {
            name: b"sSkKiIpPwWhHiItTeE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPWHITE as ::core::ffi::c_int,
        },
        flag {
            name: b"sSkKiIpPeEmMpPtTyY\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPEMPTY as ::core::ffi::c_int,
        },
        flag {
            name: b"gGrRoOuUpPhHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SYNC_HERE as ::core::ffi::c_int,
        },
        flag {
            name: b"gGrRoOuUpPtThHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SYNC_THERE as ::core::ffi::c_int,
        },
        flag {
            name: b"dDiIsSpPlLaAyY\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_DISPLAY as ::core::ffi::c_int,
        },
        flag {
            name: b"fFoOlLdD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_FOLD as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNcCeEaAlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONCEAL as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNcCeEaAlLeEnNdDsS\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONCEALENDS as ::core::ffi::c_int,
        },
        flag {
            name: b"cCcChHaArR\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 11 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNtTaAiInNsS\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 1 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNtTaAiInNeEdDiInN\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 2 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"nNeExXtTgGrRoOuUpP\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 3 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
    ];
    static mut first_letters: *const ::core::ffi::c_char =
        b"cCoOkKeEtTsSgGdDfFnN\0".as_ptr() as *const ::core::ffi::c_char;
    if arg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*(*curwin).w_s).b_syn_conceal != 0 {
        (*opt).flags |= HL_CONCEAL as ::core::ffi::c_int;
    }
    while !strchr(first_letters, *arg as ::core::ffi::c_int).is_null() {
        fidx = ::core::mem::size_of::<[flag; 19]>()
            .wrapping_div(::core::mem::size_of::<flag>())
            .wrapping_div(
                (::core::mem::size_of::<[flag; 19]>().wrapping_rem(::core::mem::size_of::<flag>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int;
        loop {
            fidx -= 1;
            if fidx < 0 as ::core::ffi::c_int {
                break;
            }
            let mut p: *mut ::core::ffi::c_char = flagtab[fidx as usize].name;
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            len = 0 as ::core::ffi::c_int;
            while *p.offset(i as isize) as ::core::ffi::c_int != NUL {
                if *arg.offset(len as isize) as ::core::ffi::c_int
                    != *p.offset(i as isize) as ::core::ffi::c_int
                    && *arg.offset(len as isize) as ::core::ffi::c_int
                        != *p.offset((i + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                {
                    break;
                }
                i += 2 as ::core::ffi::c_int;
                len += 1;
            }
            if !(*p.offset(i as isize) as ::core::ffi::c_int == NUL
                && (ascii_iswhite(*arg.offset(len as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                    || (if flagtab[fidx as usize].argtype > 0 as ::core::ffi::c_int {
                        (*arg.offset(len as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } else {
                        ends_excmd(*arg.offset(len as isize) as ::core::ffi::c_int)
                    }) != 0))
            {
                continue;
            }
            if (*opt).keyword as ::core::ffi::c_int != 0
                && (flagtab[fidx as usize].flags == HL_DISPLAY as ::core::ffi::c_int
                    || flagtab[fidx as usize].flags == HL_FOLD as ::core::ffi::c_int
                    || flagtab[fidx as usize].flags == HL_EXTEND as ::core::ffi::c_int)
            {
                fidx = -1 as ::core::ffi::c_int;
            }
            break;
        }
        if fidx < 0 as ::core::ffi::c_int {
            break;
        }
        if flagtab[fidx as usize].argtype == 1 as ::core::ffi::c_int {
            if !(*opt).has_cont_list {
                emsg(gettext(
                    &raw const e_contains_argument_not_accepted_here as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if get_id_list(
                &raw mut arg,
                8 as ::core::ffi::c_int,
                &raw mut (*opt).cont_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if flagtab[fidx as usize].argtype == 2 as ::core::ffi::c_int {
            if get_id_list(
                &raw mut arg,
                11 as ::core::ffi::c_int,
                &raw mut (*opt).cont_in_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if flagtab[fidx as usize].argtype == 3 as ::core::ffi::c_int {
            if get_id_list(
                &raw mut arg,
                9 as ::core::ffi::c_int,
                &raw mut (*opt).next_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if flagtab[fidx as usize].argtype == 11 as ::core::ffi::c_int
            && *arg.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            *conceal_char = utf_ptr2char(arg.offset(6 as ::core::ffi::c_int as isize));
            arg = arg.offset(
                (utfc_ptr2len(arg.offset(6 as ::core::ffi::c_int as isize))
                    - 1 as ::core::ffi::c_int) as isize,
            );
            if !vim_isprintc(*conceal_char) {
                emsg(gettext(
                    &raw const e_invalid_cchar_value as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
        } else {
            (*opt).flags |= flagtab[fidx as usize].flags;
            arg = skipwhite(arg.offset(len as isize));
            if flagtab[fidx as usize].flags == HL_SYNC_HERE as ::core::ffi::c_int
                || flagtab[fidx as usize].flags == HL_SYNC_THERE as ::core::ffi::c_int
            {
                if (*opt).sync_idx.is_null() {
                    emsg(gettext(b"E393: group[t]here not accepted here\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                let mut gname_start: *mut ::core::ffi::c_char = arg;
                arg = skiptowhite(arg);
                if gname_start == arg {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                let mut gname: *mut ::core::ffi::c_char =
                    xstrnsave(gname_start, arg.offset_from(gname_start) as size_t);
                if strcmp(gname, b"NONE\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    *(*opt).sync_idx = NONE_IDX;
                } else {
                    let mut syn_id: ::core::ffi::c_int = syn_name2id(gname);
                    let mut i_0: ::core::ffi::c_int = 0;
                    i_0 = (*(*curwin).w_s).b_syn_patterns.ga_len;
                    loop {
                        i_0 -= 1;
                        if i_0 < 0 as ::core::ffi::c_int {
                            break;
                        }
                        if !((*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(i_0 as isize))
                        .sp_syn
                        .id as ::core::ffi::c_int
                            == syn_id
                            && (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(i_0 as isize))
                            .sp_type as ::core::ffi::c_int
                                == SPTYPE_START)
                        {
                            continue;
                        }
                        *(*opt).sync_idx = i_0;
                        break;
                    }
                    if i_0 < 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E394: Didn't find region item for %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            gname,
                        );
                        xfree(gname as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                }
                xfree(gname as *mut ::core::ffi::c_void);
                arg = skipwhite(arg);
            } else if flagtab[fidx as usize].flags == HL_FOLD as ::core::ffi::c_int
                && foldmethodIsSyntax(curwin) as ::core::ffi::c_int != 0
            {
                foldUpdateAll(curwin);
            }
        }
    }
    return arg;
}
unsafe extern "C" fn syn_incl_toplevel(
    mut id: ::core::ffi::c_int,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    if *flagsp & HL_CONTAINED as ::core::ffi::c_int != 0
        || (*(*curwin).w_s).b_syn_topgrp == 0 as ::core::ffi::c_int
    {
        return;
    }
    *flagsp |= HL_CONTAINED as ::core::ffi::c_int | HL_INCLUDED_TOPLEVEL as ::core::ffi::c_int;
    if (*(*curwin).w_s).b_syn_topgrp >= SYNID_CLUSTER {
        let mut grp_list: *mut int16_t =
            xmalloc((2 as size_t).wrapping_mul(::core::mem::size_of::<int16_t>())) as *mut int16_t;
        let mut tlg_id: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_topgrp - SYNID_CLUSTER;
        *grp_list.offset(0 as ::core::ffi::c_int as isize) = id as int16_t;
        *grp_list.offset(1 as ::core::ffi::c_int as isize) = 0 as int16_t;
        syn_combine_list(
            &raw mut (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                .offset(tlg_id as isize))
            .scl_list,
            &raw mut grp_list,
            CLUSTER_ADD,
        );
    }
}
unsafe extern "C" fn syn_cmd_include(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut sgl_id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut source: bool = false_0 != 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '@' as ::core::ffi::c_int
    {
        arg = arg.offset(1);
        let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
        if rest.is_null() {
            emsg(gettext(
                b"E397: Filename required\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        sgl_id = syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
        if sgl_id == 0 as ::core::ffi::c_int {
            return;
        }
        (*eap).arg = rest;
    }
    (*eap).argt = ((*eap).argt as ::core::ffi::c_uint | (EX_XFILE | EX_NOSPC)) as uint32_t;
    separate_nextcmd(eap);
    if *(*eap).arg as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        || *(*eap).arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int
        || path_is_absolute((*eap).arg) as ::core::ffi::c_int != 0
    {
        source = true_0 != 0;
        if expand_filename(eap, syn_cmdlinep, &raw mut errormsg) == FAIL {
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            return;
        }
    }
    if running_syn_inc_tag >= MAX_SYN_INC_TAG {
        emsg(gettext(
            b"E847: Too many syntax includes\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut prev_syn_inc_tag: ::core::ffi::c_int = current_syn_inc_tag;
    running_syn_inc_tag += 1;
    current_syn_inc_tag = running_syn_inc_tag;
    let mut prev_toplvl_grp: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_topgrp;
    (*(*curwin).w_s).b_syn_topgrp = sgl_id;
    if if source as ::core::ffi::c_int != 0 {
        (do_source(
            (*eap).arg,
            false_0 != 0,
            DOSO_NONE as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == FAIL) as ::core::ffi::c_int
    } else {
        (source_runtime((*eap).arg, DIP_ALL as ::core::ffi::c_int) == FAIL) as ::core::ffi::c_int
    } != 0
    {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    (*(*curwin).w_s).b_syn_topgrp = prev_toplvl_grp;
    current_syn_inc_tag = prev_syn_inc_tag;
}
unsafe extern "C" fn syn_cmd_keyword(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut keyword_copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut conceal_char: ::core::ffi::c_int = NUL;
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    if !rest.is_null() {
        if (*eap).skip != 0 {
            syn_id = -1 as ::core::ffi::c_int;
        } else {
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
        }
        if syn_id != 0 as ::core::ffi::c_int {
            keyword_copy =
                xmalloc(strlen(rest).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
        if !keyword_copy.is_null() {
            syn_opt_arg.flags = 0 as ::core::ffi::c_int;
            syn_opt_arg.keyword = true_0 != 0;
            syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
            syn_opt_arg.has_cont_list = false_0 != 0;
            syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
            syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
            let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut p: *mut ::core::ffi::c_char = keyword_copy;
            while !rest.is_null() && ends_excmd(*rest as ::core::ffi::c_int) == 0 {
                rest = get_syn_options(
                    rest,
                    &raw mut syn_opt_arg,
                    &raw mut conceal_char,
                    (*eap).skip,
                );
                if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) != 0 {
                    break;
                }
                while *rest as ::core::ffi::c_int != NUL
                    && !ascii_iswhite(*rest as ::core::ffi::c_int)
                {
                    if *rest as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        && *rest.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        rest = rest.offset(1);
                    }
                    let c2rust_fresh11 = rest;
                    rest = rest.offset(1);
                    let c2rust_fresh12 = p;
                    p = p.offset(1);
                    *c2rust_fresh12 = *c2rust_fresh11;
                }
                let c2rust_fresh13 = p;
                p = p.offset(1);
                *c2rust_fresh13 = NUL as ::core::ffi::c_char;
                cnt += 1;
                rest = skipwhite(rest);
            }
            '_error: {
                if (*eap).skip == 0 {
                    syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                    let mut kwlen: size_t = 0 as size_t;
                    let mut kw: *mut ::core::ffi::c_char = keyword_copy;
                    loop {
                        cnt -= 1;
                        if cnt < 0 as ::core::ffi::c_int {
                            break '_error;
                        }
                        p = vim_strchr(kw, '[' as ::core::ffi::c_int);
                        loop {
                            if p.is_null() {
                                kwlen = strlen(kw);
                            } else {
                                *p = NUL as ::core::ffi::c_char;
                                kwlen = p.offset_from(kw) as size_t;
                            }
                            add_keyword(
                                kw,
                                kwlen,
                                syn_id,
                                syn_opt_arg.flags,
                                syn_opt_arg.cont_in_list,
                                syn_opt_arg.next_list,
                                conceal_char,
                            );
                            if p.is_null() {
                                break;
                            }
                            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == NUL
                            {
                                semsg(
                                    gettext(b"E789: Missing ']': %s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    kw,
                                );
                                break '_error;
                            } else if *p.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ']' as ::core::ffi::c_int
                            {
                                if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    != NUL
                                {
                                    semsg(
                                        gettext(
                                            &raw const e_trailing_char_after_rsb_str_str
                                                as *const ::core::ffi::c_char,
                                        ),
                                        kw,
                                        p.offset(2 as ::core::ffi::c_int as isize),
                                    );
                                    break '_error;
                                } else {
                                    kw = p.offset(1 as ::core::ffi::c_int as isize);
                                    kwlen = 1 as size_t;
                                    break;
                                }
                            } else {
                                let l: ::core::ffi::c_int =
                                    utfc_ptr2len(p.offset(1 as ::core::ffi::c_int as isize));
                                memmove(
                                    p as *mut ::core::ffi::c_void,
                                    p.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    l as size_t,
                                );
                                p = p.offset(l as isize);
                            }
                        }
                        kw = kw.offset(kwlen.wrapping_add(1 as size_t) as isize);
                    }
                }
            }
            xfree(keyword_copy as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
        }
    }
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
    redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
    syn_stack_free_all((*curwin).w_s);
}
unsafe extern "C" fn syn_cmd_match(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut item: synpat_T = synpat_T {
        sp_type: 0,
        sp_syncing: false,
        sp_syn_match_id: 0,
        sp_off_flags: 0,
        sp_offsets: [0; 7],
        sp_flags: 0,
        sp_cchar: 0,
        sp_ic: 0,
        sp_sync_idx: 0,
        sp_line_id: 0,
        sp_startcol: 0,
        sp_cont_list: ::core::ptr::null_mut::<int16_t>(),
        sp_next_list: ::core::ptr::null_mut::<int16_t>(),
        sp_syn: sp_syn {
            inc_tag: 0,
            id: 0,
            cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        },
        sp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sp_prog: ::core::ptr::null_mut::<regprog_T>(),
        sp_time: syn_time_T {
            total: 0,
            slowest: 0,
            count: 0,
            match_0: 0,
        },
    };
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut sync_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal_char: ::core::ffi::c_int = NUL;
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    syn_opt_arg.flags = 0 as ::core::ffi::c_int;
    syn_opt_arg.keyword = false_0 != 0;
    syn_opt_arg.sync_idx = if syncing != 0 {
        &raw mut sync_idx
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_int>()
    };
    syn_opt_arg.has_cont_list = true_0 != 0;
    syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
    rest = get_syn_options(
        rest,
        &raw mut syn_opt_arg,
        &raw mut conceal_char,
        (*eap).skip,
    );
    init_syn_patterns();
    memset(
        &raw mut item as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<synpat_T>(),
    );
    rest = get_syn_pattern(rest, &raw mut item);
    if vim_regcomp_had_eol() != 0 && syn_opt_arg.flags & HL_EXCLUDENL as ::core::ffi::c_int == 0 {
        syn_opt_arg.flags |= HL_HAS_EOL as ::core::ffi::c_int;
    }
    rest = get_syn_options(
        rest,
        &raw mut syn_opt_arg,
        &raw mut conceal_char,
        (*eap).skip,
    );
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
        if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
            if syn_id != 0 as ::core::ffi::c_int {
                syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                let mut spp: *mut synpat_T = ga_append_via_ptr(
                    &raw mut (*(*curwin).w_s).b_syn_patterns,
                    ::core::mem::size_of::<synpat_T>(),
                ) as *mut synpat_T;
                *spp = item;
                (*spp).sp_syncing = syncing != 0;
                (*spp).sp_type = SPTYPE_MATCH as ::core::ffi::c_char;
                (*spp).sp_syn.id = syn_id as int16_t;
                (*spp).sp_syn.inc_tag = current_syn_inc_tag;
                (*spp).sp_flags = syn_opt_arg.flags;
                (*spp).sp_sync_idx = sync_idx;
                (*spp).sp_cont_list = syn_opt_arg.cont_list;
                (*spp).sp_syn.cont_in_list = syn_opt_arg.cont_in_list;
                (*spp).sp_cchar = conceal_char;
                if !syn_opt_arg.cont_in_list.is_null() {
                    (*(*curwin).w_s).b_syn_containedin = true_0;
                }
                (*spp).sp_next_list = syn_opt_arg.next_list;
                if syn_opt_arg.flags
                    & (HL_SYNC_HERE as ::core::ffi::c_int | HL_SYNC_THERE as ::core::ffi::c_int)
                    != 0
                {
                    (*(*curwin).w_s).b_syn_sync_flags |= SF_MATCH;
                }
                if syn_opt_arg.flags & HL_FOLD as ::core::ffi::c_int != 0 {
                    (*(*curwin).w_s).b_syn_folditems += 1;
                }
                redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
                syn_stack_free_all((*curwin).w_s);
                return;
            }
        }
    }
    vim_regfree(item.sp_prog);
    xfree(item.sp_pattern as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
    if rest.is_null() {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn syn_cmd_region(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rest: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut item: ::core::ffi::c_int = 0;
    let mut pat_ptrs: [*mut pat_ptr; 3] = [::core::ptr::null_mut::<pat_ptr>(); 3];
    let mut ppp: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
    let mut ppp_next: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
    let mut pat_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut matchgroup_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut not_enough: bool = false_0 != 0;
    let mut illegal: bool = false_0 != 0;
    let mut success: bool = false_0 != 0;
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut conceal_char: ::core::ffi::c_int = NUL;
    rest = get_group_name(arg, &raw mut group_name_end);
    pat_ptrs[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    pat_ptrs[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    pat_ptrs[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    init_syn_patterns();
    syn_opt_arg.flags = 0 as ::core::ffi::c_int;
    syn_opt_arg.keyword = false_0 != 0;
    syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
    syn_opt_arg.has_cont_list = true_0 != 0;
    syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
    while !rest.is_null() && ends_excmd(*rest as ::core::ffi::c_int) == 0 {
        rest = get_syn_options(
            rest,
            &raw mut syn_opt_arg,
            &raw mut conceal_char,
            (*eap).skip,
        );
        if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) != 0 {
            break;
        }
        key_end = rest;
        while *key_end as ::core::ffi::c_int != 0
            && !ascii_iswhite(*key_end as ::core::ffi::c_int)
            && *key_end as ::core::ffi::c_int != '=' as ::core::ffi::c_int
        {
            key_end = key_end.offset(1);
        }
        xfree(key as *mut ::core::ffi::c_void);
        key = vim_strnsave_up(rest, key_end.offset_from(rest) as size_t);
        if strcmp(key, b"MATCHGROUP\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_MATCHGROUP;
        } else if strcmp(key, b"START\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_START;
        } else if strcmp(key, b"END\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_END;
        } else {
            if strcmp(key, b"SKIP\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as ::core::ffi::c_int
            {
                break;
            }
            if !pat_ptrs[ITEM_SKIP as usize].is_null() {
                illegal = true_0 != 0;
                break;
            } else {
                item = ITEM_SKIP;
            }
        }
        rest = skipwhite(key_end);
        if *rest as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
            semsg(
                gettext(b"E398: Missing '=': %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
            break;
        } else {
            rest = skipwhite(rest.offset(1 as ::core::ffi::c_int as isize));
            if *rest as ::core::ffi::c_int == NUL {
                not_enough = true_0 != 0;
                break;
            } else if item == ITEM_MATCHGROUP {
                let mut p: *mut ::core::ffi::c_char = skiptowhite(rest);
                if p.offset_from(rest) == 4 as isize
                    && strncmp(
                        rest,
                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || (*eap).skip != 0
                {
                    matchgroup_id = 0 as ::core::ffi::c_int;
                } else {
                    matchgroup_id = syn_check_group(rest, p.offset_from(rest) as size_t);
                    if matchgroup_id == 0 as ::core::ffi::c_int {
                        illegal = true_0 != 0;
                        break;
                    }
                }
                rest = skipwhite(p);
            } else {
                ppp = xmalloc(::core::mem::size_of::<pat_ptr>()) as *mut pat_ptr;
                (*ppp).pp_next = pat_ptrs[item as usize] as *mut pat_ptr;
                pat_ptrs[item as usize] = ppp as *mut pat_ptr;
                (*ppp).pp_synp =
                    xcalloc(1 as size_t, ::core::mem::size_of::<synpat_T>()) as *mut synpat_T;
                if item == ITEM_START {
                    reg_do_extmatch = REX_SET;
                } else {
                    '_c2rust_label: {
                        if item == 1 as ::core::ffi::c_int || item == 2 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"item == ITEM_SKIP || item == ITEM_END\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/syntax.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                4333 as ::core::ffi::c_uint,
                                b"void syn_cmd_region(exarg_T *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    reg_do_extmatch = REX_USE;
                }
                rest = get_syn_pattern(rest, (*ppp).pp_synp);
                reg_do_extmatch = 0 as ::core::ffi::c_int;
                if item == ITEM_END
                    && vim_regcomp_had_eol() != 0
                    && syn_opt_arg.flags & HL_EXCLUDENL as ::core::ffi::c_int == 0
                {
                    (*(*ppp).pp_synp).sp_flags |= HL_HAS_EOL as ::core::ffi::c_int;
                }
                (*ppp).pp_matchgroup_id = matchgroup_id;
                pat_count += 1;
            }
        }
    }
    xfree(key as *mut ::core::ffi::c_void);
    if illegal as ::core::ffi::c_int != 0 || not_enough as ::core::ffi::c_int != 0 {
        rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !rest.is_null()
        && (pat_ptrs[ITEM_START as usize].is_null() || pat_ptrs[ITEM_END as usize].is_null())
    {
        not_enough = true_0 != 0;
        rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
        if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            ga_grow(&raw mut (*(*curwin).w_s).b_syn_patterns, pat_count);
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
            if syn_id != 0 as ::core::ffi::c_int {
                syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                let mut idx: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_patterns.ga_len;
                item = ITEM_START;
                while item <= ITEM_END {
                    ppp = pat_ptrs[item as usize] as *mut pat_ptr;
                    while !ppp.is_null() {
                        *((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize) = *(*ppp).pp_synp;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syncing = syncing != 0;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_type = (if item == ITEM_START {
                            SPTYPE_START
                        } else if item == ITEM_SKIP {
                            SPTYPE_SKIP
                        } else {
                            SPTYPE_END
                        }) as ::core::ffi::c_char;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_flags |= syn_opt_arg.flags;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn
                        .id = syn_id as int16_t;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn
                        .inc_tag = current_syn_inc_tag;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn_match_id = (*ppp).pp_matchgroup_id as int16_t;
                        (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_cchar = conceal_char;
                        if item == ITEM_START {
                            (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_cont_list = syn_opt_arg.cont_list;
                            (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syn
                            .cont_in_list = syn_opt_arg.cont_in_list;
                            if !syn_opt_arg.cont_in_list.is_null() {
                                (*(*curwin).w_s).b_syn_containedin = true_0;
                            }
                            (*((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_next_list = syn_opt_arg.next_list;
                        }
                        (*(*curwin).w_s).b_syn_patterns.ga_len += 1;
                        idx += 1;
                        if syn_opt_arg.flags & HL_FOLD as ::core::ffi::c_int != 0 {
                            (*(*curwin).w_s).b_syn_folditems += 1;
                        }
                        ppp = (*ppp).pp_next;
                    }
                    item += 1;
                }
                redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
                syn_stack_free_all((*curwin).w_s);
                success = true_0 != 0;
            }
        }
    }
    item = ITEM_START;
    while item <= ITEM_END {
        ppp = pat_ptrs[item as usize] as *mut pat_ptr;
        while !ppp.is_null() {
            if !success && !(*ppp).pp_synp.is_null() {
                vim_regfree((*(*ppp).pp_synp).sp_prog);
                xfree((*(*ppp).pp_synp).sp_pattern as *mut ::core::ffi::c_void);
            }
            xfree((*ppp).pp_synp as *mut ::core::ffi::c_void);
            ppp_next = (*ppp).pp_next;
            xfree(ppp as *mut ::core::ffi::c_void);
            ppp = ppp_next;
        }
        item += 1;
    }
    if !success {
        xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
        if not_enough {
            semsg(
                gettext(b"E399: Not enough arguments: syntax region %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                arg,
            );
        } else if illegal as ::core::ffi::c_int != 0 || rest.is_null() {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
    }
}
pub const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn syn_compare_stub(
    v1: *const ::core::ffi::c_void,
    v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let s1: *const int16_t = v1 as *const int16_t;
    let s2: *const int16_t = v2 as *const int16_t;
    return if *s1 as ::core::ffi::c_int > *s2 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else if (*s1 as ::core::ffi::c_int) < *s2 as ::core::ffi::c_int {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn syn_combine_list(
    clstr1: *mut *mut int16_t,
    clstr2: *mut *mut int16_t,
    list_op: ::core::ffi::c_int,
) {
    let mut count1: size_t = 0 as size_t;
    let mut count2: size_t = 0 as size_t;
    let mut g1: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut g2: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut clstr: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    if (*clstr2).is_null() {
        return;
    }
    if (*clstr1).is_null() || list_op == CLUSTER_REPLACE {
        if list_op == CLUSTER_REPLACE {
            xfree(*clstr1 as *mut ::core::ffi::c_void);
        }
        if list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD {
            *clstr1 = *clstr2;
        } else {
            xfree(*clstr2 as *mut ::core::ffi::c_void);
        }
        return;
    }
    g1 = *clstr1;
    while *g1 != 0 {
        count1 = count1.wrapping_add(1);
        g1 = g1.offset(1);
    }
    g2 = *clstr2;
    while *g2 != 0 {
        count2 = count2.wrapping_add(1);
        g2 = g2.offset(1);
    }
    qsort(
        *clstr1 as *mut ::core::ffi::c_void,
        count1,
        ::core::mem::size_of::<int16_t>(),
        Some(
            syn_compare_stub
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    qsort(
        *clstr2 as *mut ::core::ffi::c_void,
        count2,
        ::core::mem::size_of::<int16_t>(),
        Some(
            syn_compare_stub
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        g1 = *clstr1;
        g2 = *clstr2;
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *g1 as ::core::ffi::c_int != 0 && *g2 as ::core::ffi::c_int != 0 {
            if (*g1 as ::core::ffi::c_int) < *g2 as ::core::ffi::c_int {
                if round == 2 as ::core::ffi::c_int {
                    *clstr.offset(count as isize) = *g1;
                }
                count += 1;
                g1 = g1.offset(1);
            } else {
                if list_op == CLUSTER_ADD {
                    if round == 2 as ::core::ffi::c_int {
                        *clstr.offset(count as isize) = *g2;
                    }
                    count += 1;
                }
                if *g1 as ::core::ffi::c_int == *g2 as ::core::ffi::c_int {
                    g1 = g1.offset(1);
                }
                g2 = g2.offset(1);
            }
        }
        while *g1 != 0 {
            if round == 2 as ::core::ffi::c_int {
                *clstr.offset(count as isize) = *g1;
            }
            g1 = g1.offset(1);
            count += 1;
        }
        if list_op == CLUSTER_ADD {
            while *g2 != 0 {
                if round == 2 as ::core::ffi::c_int {
                    *clstr.offset(count as isize) = *g2;
                }
                g2 = g2.offset(1);
                count += 1;
            }
        }
        if round == 1 as ::core::ffi::c_int {
            if count == 0 as ::core::ffi::c_int {
                clstr = ::core::ptr::null_mut::<int16_t>();
                break;
            } else {
                clstr = xmalloc(
                    (count as size_t)
                        .wrapping_add(1 as size_t)
                        .wrapping_mul(::core::mem::size_of::<int16_t>()),
                ) as *mut int16_t;
                *clstr.offset(count as isize) = 0 as int16_t;
            }
        }
        round += 1;
    }
    xfree(*clstr1 as *mut ::core::ffi::c_void);
    xfree(*clstr2 as *mut ::core::ffi::c_void);
    *clstr1 = clstr;
}
unsafe extern "C" fn syn_scl_name2id(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut name_u: *mut ::core::ffi::c_char = vim_strsave_up(name);
    let mut i: ::core::ffi::c_int = 0;
    i = (*(*curwin).w_s).b_syn_clusters.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if !(*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize))
            .scl_name_u
            .is_null()
            && strcmp(
                name_u,
                (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(i as isize))
                .scl_name_u,
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
    }
    xfree(name_u as *mut ::core::ffi::c_void);
    return if i < 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        i + SYNID_CLUSTER
    };
}
unsafe extern "C" fn syn_scl_namen2id(
    mut linep: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = xstrnsave(linep, len as size_t);
    let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
    xfree(name as *mut ::core::ffi::c_void);
    return id;
}
unsafe extern "C" fn syn_check_cluster(
    mut pp: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = xstrnsave(pp, len as size_t);
    let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
    if id == 0 as ::core::ffi::c_int {
        id = syn_add_cluster(name);
    } else {
        xfree(name as *mut ::core::ffi::c_void);
    }
    return id;
}
unsafe extern "C" fn syn_add_cluster(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    if (*(*curwin).w_s).b_syn_clusters.ga_data.is_null() {
        (*(*curwin).w_s).b_syn_clusters.ga_itemsize =
            ::core::mem::size_of::<syn_cluster_T>() as ::core::ffi::c_int;
        ga_set_growsize(
            &raw mut (*(*curwin).w_s).b_syn_clusters,
            10 as ::core::ffi::c_int,
        );
    }
    let mut len: ::core::ffi::c_int = (*(*curwin).w_s).b_syn_clusters.ga_len;
    if len >= MAX_CLUSTER_ID {
        emsg(gettext(
            b"E848: Too many syntax clusters\0".as_ptr() as *const ::core::ffi::c_char
        ));
        xfree(name as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    let mut scp: *mut syn_cluster_T = ga_append_via_ptr(
        &raw mut (*(*curwin).w_s).b_syn_clusters,
        ::core::mem::size_of::<syn_cluster_T>(),
    ) as *mut syn_cluster_T;
    memset(
        scp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<syn_cluster_T>(),
    );
    (*scp).scl_name = name;
    (*scp).scl_name_u = vim_strsave_up(name);
    (*scp).scl_list = ::core::ptr::null_mut::<int16_t>();
    if strcasecmp(
        name,
        b"Spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*(*curwin).w_s).b_spell_cluster_id = len + SYNID_CLUSTER;
    }
    if strcasecmp(
        name,
        b"NoSpell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*(*curwin).w_s).b_nospell_cluster_id = len + SYNID_CLUSTER;
    }
    return len + SYNID_CLUSTER;
}
unsafe extern "C" fn syn_cmd_cluster(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut got_clstr: bool = false_0 != 0;
    let mut opt_len: ::core::ffi::c_int = 0;
    let mut list_op: ::core::ffi::c_int = 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    if !rest.is_null() {
        let mut scl_id: ::core::ffi::c_int =
            syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
        if scl_id == 0 as ::core::ffi::c_int {
            return;
        }
        scl_id -= SYNID_CLUSTER;
        loop {
            if strncasecmp(
                rest,
                b"add\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (ascii_iswhite(
                    *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                opt_len = 3 as ::core::ffi::c_int;
                list_op = CLUSTER_ADD;
            } else if strncasecmp(
                rest,
                b"remove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                6 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (ascii_iswhite(
                    *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                opt_len = 6 as ::core::ffi::c_int;
                list_op = CLUSTER_SUBTRACT;
            } else {
                if !(strncasecmp(
                    rest,
                    b"contains\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    8 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(
                        *rest.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                        || *rest.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int))
                {
                    break;
                }
                opt_len = 8 as ::core::ffi::c_int;
                list_op = CLUSTER_REPLACE;
            }
            let mut clstr_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
            if get_id_list(
                &raw mut rest,
                opt_len,
                &raw mut clstr_list,
                (*eap).skip != 0,
            ) == FAIL
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    rest,
                );
                break;
            } else {
                if scl_id >= 0 as ::core::ffi::c_int {
                    syn_combine_list(
                        &raw mut (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                            .offset(scl_id as isize))
                        .scl_list,
                        &raw mut clstr_list,
                        list_op,
                    );
                } else {
                    xfree(clstr_list as *mut ::core::ffi::c_void);
                }
                got_clstr = true_0 != 0;
            }
        }
        if got_clstr {
            redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
            syn_stack_free_all((*curwin).w_s);
        }
    }
    if !got_clstr {
        emsg(gettext(
            b"E400: No cluster specified\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn init_syn_patterns() {
    (*(*curwin).w_s).b_syn_patterns.ga_itemsize =
        ::core::mem::size_of::<synpat_T>() as ::core::ffi::c_int;
    ga_set_growsize(
        &raw mut (*(*curwin).w_s).b_syn_patterns,
        10 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn get_syn_pattern(
    mut arg: *mut ::core::ffi::c_char,
    mut ci: *mut synpat_T,
) -> *mut ::core::ffi::c_char {
    let mut idx: ::core::ffi::c_int = 0;
    if arg.is_null()
        || *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *arg.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut end: *mut ::core::ffi::c_char = skip_regexp(
        arg.offset(1 as ::core::ffi::c_int as isize),
        *arg as ::core::ffi::c_int,
        true_0,
    );
    if *end as ::core::ffi::c_int != *arg as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E401: Pattern delimiter not found: %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            arg,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ci).sp_pattern = xstrnsave(
        arg.offset(1 as ::core::ffi::c_int as isize),
        (end.offset_from(arg) as size_t).wrapping_sub(1 as size_t),
    );
    let mut cpo_save: *mut ::core::ffi::c_char = p_cpo;
    p_cpo = &raw mut empty_string_option as *mut ::core::ffi::c_char;
    (*ci).sp_prog = vim_regcomp((*ci).sp_pattern, RE_MAGIC);
    p_cpo = cpo_save;
    if (*ci).sp_prog.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ci).sp_ic = (*(*curwin).w_s).b_syn_ic;
    syn_clear_time(&raw mut (*ci).sp_time);
    end = end.offset(1);
    loop {
        idx = SPO_COUNT;
        loop {
            idx -= 1;
            if idx < 0 as ::core::ffi::c_int {
                break;
            }
            if strncmp(
                end,
                spo_name_tab[idx as usize] as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
        }
        if idx >= 0 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_int =
                (&raw mut (*ci).sp_offsets as *mut ::core::ffi::c_int).offset(idx as isize);
            if idx != SPO_LC_OFF {
                match *end.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                    115 | 98 => {}
                    101 => {
                        idx += SPO_COUNT;
                    }
                    _ => {
                        idx = -1 as ::core::ffi::c_int;
                    }
                }
            }
            if idx >= 0 as ::core::ffi::c_int {
                (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                    | ((1 as ::core::ffi::c_int) << idx) as int16_t as ::core::ffi::c_int)
                    as int16_t;
                if idx == SPO_LC_OFF {
                    end = end.offset(3 as ::core::ffi::c_int as isize);
                    *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    if (*ci).sp_off_flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << SPO_MS_OFF
                        == 0
                    {
                        (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                            | (1 as ::core::ffi::c_int) << SPO_MS_OFF)
                            as int16_t;
                        (*ci).sp_offsets[SPO_MS_OFF as usize] = *p;
                    }
                } else {
                    end = end.offset(4 as ::core::ffi::c_int as isize);
                    if *end as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                        end = end.offset(1);
                        *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    } else if *end as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        end = end.offset(1);
                        *p = -getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    }
                }
                if *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                    break;
                }
                end = end.offset(1);
            }
        }
        if idx < 0 as ::core::ffi::c_int {
            break;
        }
    }
    if ends_excmd(*end as ::core::ffi::c_int) == 0 && !ascii_iswhite(*end as ::core::ffi::c_int) {
        semsg(
            gettext(b"E402: Garbage after pattern: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return skipwhite(end);
}
unsafe extern "C" fn syn_cmd_sync(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg_start: *mut ::core::ffi::c_char = (*eap).arg;
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut illegal: bool = false_0 != 0;
    let mut finished: bool = false_0 != 0;
    if ends_excmd(*arg_start as ::core::ffi::c_int) != 0 {
        syn_cmd_list(eap, true_0);
        return;
    }
    while ends_excmd(*arg_start as ::core::ffi::c_int) == 0 {
        let mut arg_end: *mut ::core::ffi::c_char = skiptowhite(arg_start);
        let mut next_arg: *mut ::core::ffi::c_char = skipwhite(arg_end);
        xfree(key as *mut ::core::ffi::c_void);
        key = vim_strnsave_up(arg_start, arg_end.offset_from(arg_start) as size_t);
        if strcmp(key, b"CCOMMENT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if (*eap).skip == 0 {
                (*(*curwin).w_s).b_syn_sync_flags |= SF_CCOMMENT;
            }
            if ends_excmd(*next_arg as ::core::ffi::c_int) == 0 {
                arg_end = skiptowhite(next_arg);
                if (*eap).skip == 0 {
                    (*(*curwin).w_s).b_syn_sync_id =
                        syn_check_group(next_arg, arg_end.offset_from(next_arg) as size_t)
                            as int16_t;
                }
                next_arg = skipwhite(arg_end);
            } else if (*eap).skip == 0 {
                (*(*curwin).w_s).b_syn_sync_id =
                    syn_name2id(b"Comment\0".as_ptr() as *const ::core::ffi::c_char) as int16_t;
            }
        } else if strncmp(
            key,
            b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"MINLINES\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"MAXLINES\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"LINEBREAKS\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'S' as ::core::ffi::c_int
            {
                arg_end = key.offset(6 as ::core::ffi::c_int as isize);
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'L' as ::core::ffi::c_int
            {
                arg_end = key.offset(11 as ::core::ffi::c_int as isize);
            } else {
                arg_end = key.offset(9 as ::core::ffi::c_int as isize);
            }
            if *arg_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '=' as ::core::ffi::c_int
                || !ascii_isdigit(*arg_end as ::core::ffi::c_int)
            {
                illegal = true_0 != 0;
                break;
            } else {
                let mut n: linenr_T = getdigits_int32(&raw mut arg_end, false_0 != 0, 0 as int32_t);
                if (*eap).skip == 0 {
                    if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'B' as ::core::ffi::c_int
                    {
                        (*(*curwin).w_s).b_syn_sync_linebreaks = n;
                    } else if *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'A' as ::core::ffi::c_int
                    {
                        (*(*curwin).w_s).b_syn_sync_maxlines = n;
                    } else {
                        (*(*curwin).w_s).b_syn_sync_minlines = n;
                    }
                }
            }
        } else if strcmp(key, b"FROMSTART\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if (*eap).skip == 0 {
                (*(*curwin).w_s).b_syn_sync_minlines = MAXLNUM as ::core::ffi::c_int as linenr_T;
                (*(*curwin).w_s).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
            }
        } else if strcmp(key, b"LINECONT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if *next_arg as ::core::ffi::c_int == NUL {
                illegal = true_0 != 0;
                break;
            } else if !(*(*curwin).w_s).b_syn_linecont_pat.is_null() {
                emsg(gettext(
                    b"E403: syntax sync: line continuations pattern specified twice\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                finished = true_0 != 0;
                break;
            } else {
                arg_end = skip_regexp(
                    next_arg.offset(1 as ::core::ffi::c_int as isize),
                    *next_arg as ::core::ffi::c_int,
                    true_0,
                );
                if *arg_end as ::core::ffi::c_int != *next_arg as ::core::ffi::c_int {
                    illegal = true_0 != 0;
                    break;
                } else {
                    if (*eap).skip == 0 {
                        (*(*curwin).w_s).b_syn_linecont_pat = xstrnsave(
                            next_arg.offset(1 as ::core::ffi::c_int as isize),
                            (arg_end.offset_from(next_arg) as size_t).wrapping_sub(1 as size_t),
                        );
                        (*(*curwin).w_s).b_syn_linecont_ic = (*(*curwin).w_s).b_syn_ic;
                        let mut cpo_save: *mut ::core::ffi::c_char = p_cpo;
                        p_cpo = &raw mut empty_string_option as *mut ::core::ffi::c_char;
                        (*(*curwin).w_s).b_syn_linecont_prog =
                            vim_regcomp((*(*curwin).w_s).b_syn_linecont_pat, RE_MAGIC);
                        p_cpo = cpo_save;
                        syn_clear_time(&raw mut (*(*curwin).w_s).b_syn_linecont_time);
                        if (*(*curwin).w_s).b_syn_linecont_prog.is_null() {
                            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(*curwin).w_s)
                                .b_syn_linecont_pat
                                as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            *ptr_;
                            finished = true_0 != 0;
                            break;
                        }
                    }
                    next_arg = skipwhite(arg_end.offset(1 as ::core::ffi::c_int as isize));
                }
            }
        } else {
            (*eap).arg = next_arg;
            if strcmp(key, b"MATCH\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_match(eap, true_0);
            } else if strcmp(key, b"REGION\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_region(eap, true_0);
            } else if strcmp(key, b"CLEAR\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_clear(eap, true_0);
            } else {
                illegal = true_0 != 0;
            }
            finished = true_0 != 0;
            break;
        }
        arg_start = next_arg;
    }
    xfree(key as *mut ::core::ffi::c_void);
    if illegal {
        semsg(
            gettext(b"E404: Illegal arguments: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg_start,
        );
    } else if !finished {
        (*eap).nextcmd = check_nextcmd(arg_start);
        redraw_curbuf_later(UPD_SOME_VALID as ::core::ffi::c_int);
        syn_stack_free_all((*curwin).w_s);
    }
}
unsafe extern "C" fn get_id_list(
    arg: *mut *mut ::core::ffi::c_char,
    keylen: ::core::ffi::c_int,
    list: *mut *mut int16_t,
    skip: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retval: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut id: ::core::ffi::c_int = 0;
    let mut failed: bool = false_0 != 0;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        p = skipwhite((*arg).offset(keylen as isize));
        if *p as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            semsg(
                gettext(b"E405: Missing equal sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *arg,
            );
            break;
        } else {
            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                semsg(
                    gettext(b"E406: Empty argument: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    *arg,
                );
                break;
            } else {
                let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                loop {
                    end = p;
                    while *end as ::core::ffi::c_int != 0
                        && !ascii_iswhite(*end as ::core::ffi::c_int)
                        && *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                    {
                        end = end.offset(1);
                    }
                    let name: *mut ::core::ffi::c_char =
                        xmalloc((end.offset_from(p) as size_t).wrapping_add(3 as size_t))
                            as *mut ::core::ffi::c_char;
                    xmemcpyz(
                        name.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        end.offset_from(p) as size_t,
                    );
                    if strcmp(
                        name.offset(1 as ::core::ffi::c_int as isize),
                        b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"ALL\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"TOP\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        if (if (**arg as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                            || **arg as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                        {
                            **arg as ::core::ffi::c_int
                        } else {
                            **arg as ::core::ffi::c_int
                                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }) != 'C' as ::core::ffi::c_int
                        {
                            semsg(
                                gettext(b"E407: %s not allowed here\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                name.offset(1 as ::core::ffi::c_int as isize),
                            );
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else if count != 0 as ::core::ffi::c_int {
                            semsg(
                                gettext(b"E408: %s must be first in contains list\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                name.offset(1 as ::core::ffi::c_int as isize),
                            );
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'A' as ::core::ffi::c_int
                            {
                                id = MAX_HL_ID as ::core::ffi::c_int;
                            } else if *name.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'T' as ::core::ffi::c_int
                            {
                                id = SYNID_TOP;
                            } else {
                                id = SYNID_CONTAINED;
                            }
                            id += current_syn_inc_tag;
                        }
                    } else if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '@' as ::core::ffi::c_int
                    {
                        if skip {
                            id = -1 as ::core::ffi::c_int;
                        } else {
                            id = syn_check_cluster(
                                name.offset(2 as ::core::ffi::c_int as isize),
                                (end.offset_from(p) - 1 as isize) as ::core::ffi::c_int,
                            );
                        }
                    } else if strpbrk(
                        name.offset(1 as ::core::ffi::c_int as isize),
                        b"\\.*^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                    .is_null()
                    {
                        id = syn_check_group(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            end.offset_from(p) as size_t,
                        );
                    } else {
                        *name = '^' as ::core::ffi::c_char;
                        strcat(name, b"$\0".as_ptr() as *const ::core::ffi::c_char);
                        regmatch.regprog = vim_regcomp(name, RE_MAGIC);
                        if regmatch.regprog.is_null() {
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            regmatch.rm_ic = true_0 != 0;
                            id = 0 as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = highlight_num_groups();
                            loop {
                                i -= 1;
                                if i < 0 as ::core::ffi::c_int {
                                    break;
                                }
                                if vim_regexec(
                                    &raw mut regmatch,
                                    highlight_group_name(i),
                                    0 as colnr_T,
                                ) {
                                    if round == 2 as ::core::ffi::c_int {
                                        if count >= total_count {
                                            xfree(retval as *mut ::core::ffi::c_void);
                                            round = 1 as ::core::ffi::c_int;
                                        } else {
                                            *retval.offset(count as isize) =
                                                (i + 1 as ::core::ffi::c_int) as int16_t;
                                        }
                                    }
                                    count += 1;
                                    id = -1 as ::core::ffi::c_int;
                                }
                            }
                            vim_regfree(regmatch.regprog);
                        }
                    }
                    xfree(name as *mut ::core::ffi::c_void);
                    if id == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E409: Unknown group name: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            p,
                        );
                        failed = true_0 != 0;
                        break;
                    } else {
                        if id > 0 as ::core::ffi::c_int {
                            if round == 2 as ::core::ffi::c_int {
                                if count >= total_count {
                                    xfree(retval as *mut ::core::ffi::c_void);
                                    round = 1 as ::core::ffi::c_int;
                                } else {
                                    *retval.offset(count as isize) = id as int16_t;
                                }
                            }
                            count += 1;
                        }
                        p = skipwhite(end);
                        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                            break;
                        }
                        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                        if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                            break;
                        }
                    }
                }
                if failed {
                    break;
                }
                if round == 1 as ::core::ffi::c_int {
                    retval = xmalloc(
                        (count as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<int16_t>()),
                    ) as *mut int16_t;
                    *retval.offset(count as isize) = 0 as int16_t;
                    total_count = count;
                }
                round += 1;
            }
        }
    }
    *arg = p;
    if failed as ::core::ffi::c_int != 0 || retval.is_null() {
        xfree(retval as *mut ::core::ffi::c_void);
        return FAIL;
    }
    if (*list).is_null() {
        *list = retval;
    } else {
        xfree(retval as *mut ::core::ffi::c_void);
    }
    return OK;
}
unsafe extern "C" fn copy_id_list(list: *const int16_t) -> *mut int16_t {
    if list.is_null() {
        return ::core::ptr::null_mut::<int16_t>();
    }
    let mut count: ::core::ffi::c_int = 0;
    count = 0 as ::core::ffi::c_int;
    while *list.offset(count as isize) != 0 {
        count += 1;
    }
    let len: size_t = (count as size_t)
        .wrapping_add(1 as size_t)
        .wrapping_mul(::core::mem::size_of::<int16_t>());
    let retval: *mut int16_t = xmalloc(len) as *mut int16_t;
    memmove(
        retval as *mut ::core::ffi::c_void,
        list as *const ::core::ffi::c_void,
        len,
    );
    return retval;
}
unsafe extern "C" fn in_id_list(
    mut cur_si: *mut stateitem_T,
    mut list: *mut int16_t,
    mut ssp: *mut sp_syn,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    let mut id: int16_t = (*ssp).id;
    static mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !cur_si.is_null()
        && !(*ssp).cont_in_list.is_null()
        && (*cur_si).si_flags & HL_MATCH as ::core::ffi::c_int == 0
    {
        while (*cur_si).si_flags & HL_TRANS_CONT as ::core::ffi::c_int != 0
            && cur_si > current_state.ga_data as *mut stateitem_T
        {
            cur_si = cur_si.offset(-1);
        }
        if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
            && in_id_list(
                ::core::ptr::null_mut::<stateitem_T>(),
                (*ssp).cont_in_list,
                &raw mut (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_syn,
                (*((*syn_block).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_flags,
            ) != 0
        {
            return true_0;
        }
    }
    if list.is_null() {
        return false_0;
    }
    if list == ID_LIST_ALL {
        return (flags & HL_CONTAINED as ::core::ffi::c_int == 0) as ::core::ffi::c_int;
    }
    let mut toplevel: bool = flags & HL_CONTAINED as ::core::ffi::c_int == 0
        || flags & HL_INCLUDED_TOPLEVEL as ::core::ffi::c_int != 0;
    let mut item: int16_t = *list;
    if item as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
        && (item as ::core::ffi::c_int) < SYNID_CLUSTER
    {
        if (item as ::core::ffi::c_int) < SYNID_TOP {
            if item as ::core::ffi::c_int - MAX_HL_ID as ::core::ffi::c_int != (*ssp).inc_tag {
                return false_0;
            }
        } else if (item as ::core::ffi::c_int) < SYNID_CONTAINED {
            if item as ::core::ffi::c_int - SYNID_TOP != (*ssp).inc_tag || !toplevel {
                return false_0;
            }
        } else if item as ::core::ffi::c_int - SYNID_CONTAINED != (*ssp).inc_tag
            || toplevel as ::core::ffi::c_int != 0
        {
            return false_0;
        }
        list = list.offset(1);
        item = *list;
        retval = false_0;
    } else {
        retval = true_0;
    }
    while item as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if item as ::core::ffi::c_int == id as ::core::ffi::c_int {
            return retval;
        }
        if item as ::core::ffi::c_int >= SYNID_CLUSTER {
            let mut scl_list: *mut int16_t = (*((*syn_block).b_syn_clusters.ga_data
                as *mut syn_cluster_T)
                .offset((item as ::core::ffi::c_int - SYNID_CLUSTER) as isize))
            .scl_list;
            if !scl_list.is_null() && depth < 30 as ::core::ffi::c_int {
                depth += 1;
                let mut r: ::core::ffi::c_int =
                    in_id_list(::core::ptr::null_mut::<stateitem_T>(), scl_list, ssp, flags);
                depth -= 1;
                if r != 0 {
                    return retval;
                }
            }
        }
        list = list.offset(1);
        item = *list;
    }
    return (retval == 0) as ::core::ffi::c_int;
}
static mut subcommands: [subcommand; 19] = [
    subcommand {
        name: b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_case as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_clear as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"cluster\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_cluster as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_conceal as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"enable\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"foldlevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_foldlevel as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"include\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_include as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"iskeyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_iskeyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_keyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_manual as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_match as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_off as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_region as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"reset\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_reset as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_spell as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_sync as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
];
#[no_mangle]
pub unsafe extern "C" fn ex_syntax(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut subcmd_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    syn_cmdlinep = (*eap).cmdlinep;
    subcmd_end = arg;
    while *subcmd_end as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *subcmd_end as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *subcmd_end as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *subcmd_end as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        subcmd_end = subcmd_end.offset(1);
    }
    let subcmd_name: *mut ::core::ffi::c_char =
        xstrnsave(arg, subcmd_end.offset_from(arg) as size_t);
    if (*eap).skip != 0 {
        emsg_skip += 1;
    }
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < ::core::mem::size_of::<[subcommand; 19]>()
        .wrapping_div(::core::mem::size_of::<subcommand>())
        .wrapping_div(
            (::core::mem::size_of::<[subcommand; 19]>()
                .wrapping_rem(::core::mem::size_of::<subcommand>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        if strcmp(subcmd_name, subcommands[i as usize].name) == 0 as ::core::ffi::c_int {
            (*eap).arg = skipwhite(subcmd_end);
            subcommands[i as usize]
                .func
                .expect("non-null function pointer")(eap, false_0);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if i == ::core::mem::size_of::<[subcommand; 19]>()
        .wrapping_div(::core::mem::size_of::<subcommand>())
        .wrapping_div(
            (::core::mem::size_of::<[subcommand; 19]>()
                .wrapping_rem(::core::mem::size_of::<subcommand>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        semsg(
            gettext(
                b"E410: Invalid :syntax subcommand: %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            subcmd_name,
        );
    }
    xfree(subcmd_name as *mut ::core::ffi::c_void);
    if (*eap).skip != 0 {
        emsg_skip -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_ownsyntax(mut eap: *mut exarg_T) {
    if (*curwin).w_s == &raw mut (*(*curwin).w_buffer).b_s {
        (*curwin).w_s =
            xcalloc(1 as size_t, ::core::mem::size_of::<synblock_T>()) as *mut synblock_T;
        hash_init(&raw mut (*(*curwin).w_s).b_keywtab);
        hash_init(&raw mut (*(*curwin).w_s).b_keywtab_ic);
        (*curwin).w_onebuf_opt.wo_spell = false_0;
        clear_string_option(&raw mut (*(*curwin).w_s).b_p_spc);
        clear_string_option(&raw mut (*(*curwin).w_s).b_p_spf);
        clear_string_option(&raw mut (*(*curwin).w_s).b_p_spl);
        clear_string_option(&raw mut (*(*curwin).w_s).b_p_spo);
        clear_string_option(&raw mut (*(*curwin).w_s).b_syn_isk);
    }
    let mut old_value: *mut ::core::ffi::c_char =
        get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
    if !old_value.is_null() {
        old_value = xstrdup(old_value);
    }
    apply_autocmds(
        EVENT_SYNTAX,
        (*eap).arg,
        (*curbuf).b_fname,
        true_0 != 0,
        curbuf,
    );
    let mut new_value: *mut ::core::ffi::c_char =
        get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
    if !new_value.is_null() {
        set_internal_string_var(
            b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            new_value,
        );
    }
    if old_value.is_null() {
        do_unlet(
            b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
    } else {
        set_internal_string_var(
            b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            old_value,
        );
        xfree(old_value as *mut ::core::ffi::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn syntax_present(mut win: *mut win_T) -> bool {
    return (*(*win).w_s).b_syn_patterns.ga_len != 0 as ::core::ffi::c_int
        || (*(*win).w_s).b_syn_clusters.ga_len != 0 as ::core::ffi::c_int
        || (*(*win).w_s).b_keywtab.ht_used > 0 as size_t
        || (*(*win).w_s).b_keywtab_ic.ht_used > 0 as size_t;
}
static mut expand_what: C2Rust_Unnamed_24 = EXP_SUBCMD;
#[no_mangle]
pub unsafe extern "C" fn reset_expand_highlight() {
    include_none = 0 as ::core::ffi::c_int;
    include_default = include_none;
    include_link = include_default;
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_echohl_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_none = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_syntax_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_SYNTAX as ::core::ffi::c_int;
    expand_what = EXP_SUBCMD;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_link = 0 as ::core::ffi::c_int;
    include_default = 0 as ::core::ffi::c_int;
    if *arg as ::core::ffi::c_int == NUL {
        return;
    }
    let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    (*xp).xp_pattern = skipwhite(p);
    if *skiptowhite((*xp).xp_pattern) as ::core::ffi::c_int != NUL {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what = EXP_CASE;
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what = EXP_SPELL;
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what = EXP_SYNC;
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
            expand_what = EXP_CLUSTER;
        } else {
            (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
        }
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_syntax_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match expand_what as ::core::ffi::c_uint {
        0 => {
            if idx < 0 as ::core::ffi::c_int
                || idx
                    >= ::core::mem::size_of::<[subcommand; 19]>()
                        .wrapping_div(::core::mem::size_of::<subcommand>())
                        .wrapping_div(
                            (::core::mem::size_of::<[subcommand; 19]>()
                                .wrapping_rem(::core::mem::size_of::<subcommand>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) as ::core::ffi::c_int
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            return subcommands[idx as usize].name;
        }
        1 => {
            static mut case_args: [*mut ::core::ffi::c_char; 3] = [
                b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return case_args[idx as usize];
        }
        2 => {
            static mut spell_args: [*mut ::core::ffi::c_char; 4] = [
                b"toplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return spell_args[idx as usize];
        }
        3 => {
            static mut sync_args: [*mut ::core::ffi::c_char; 11] = [
                b"ccomment\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"fromstart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linebreaks=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linecont\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"lines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"maxlines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"minlines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return sync_args[idx as usize];
        }
        4 => {
            if idx < (*(*curwin).w_s).b_syn_clusters.ga_len {
                vim_snprintf(
                    &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
                    EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"@%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*((*(*curwin).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                        .offset(idx as isize))
                    .scl_name,
                );
                return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
            } else {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_id(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut trans: ::core::ffi::c_int,
    mut spellp: *mut bool,
    mut keep_state: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if wp != syn_win || (*wp).w_buffer != syn_buf || lnum != current_lnum || col < current_col {
        syntax_start(wp, lnum);
    } else if col > current_col {
        next_match_idx = -1 as ::core::ffi::c_int;
    }
    get_syntax_attr(col, spellp, keep_state != 0);
    return if trans != 0 {
        current_trans_id
    } else {
        current_id
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_syntax_info(
    mut seqnrp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    *seqnrp = current_seqnr;
    return current_flags;
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_concealed_id(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    let mut seqnr: ::core::ffi::c_int = 0;
    syn_get_id(
        wp,
        lnum,
        col,
        false_0,
        ::core::ptr::null_mut::<bool>(),
        false_0,
    );
    let mut syntax_flags: ::core::ffi::c_int = get_syntax_info(&raw mut seqnr);
    if syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0 {
        return seqnr;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_sub_char() -> ::core::ffi::c_int {
    return current_sub_char;
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_stack_item(mut i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if i >= current_state.ga_len {
        invalidate_current_state();
        current_col = MAXCOL as ::core::ffi::c_int as colnr_T;
        return -1 as ::core::ffi::c_int;
    }
    return (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_id;
}
unsafe extern "C" fn syn_cur_foldlevel() -> ::core::ffi::c_int {
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < current_state.ga_len {
        if (*(current_state.ga_data as *mut stateitem_T).offset(i as isize)).si_flags
            & HL_FOLD as ::core::ffi::c_int
            != 0
        {
            level += 1;
        }
        i += 1;
    }
    return level;
}
#[no_mangle]
pub unsafe extern "C" fn syn_get_foldlevel(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*(*wp).w_s).b_syn_folditems != 0 as ::core::ffi::c_int
        && !(*(*wp).w_s).b_syn_error
        && !(*(*wp).w_s).b_syn_slow
    {
        syntax_start(wp, lnum);
        level = syn_cur_foldlevel();
        if (*(*wp).w_s).b_syn_foldlevel == SYNFLD_MINIMUM {
            let mut cur_level: ::core::ffi::c_int = level;
            let mut low_level: ::core::ffi::c_int = cur_level;
            while !current_finished {
                syn_current_attr(
                    false_0 != 0,
                    false_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                    false_0 != 0,
                );
                cur_level = syn_cur_foldlevel();
                if cur_level < low_level {
                    low_level = cur_level;
                } else if cur_level > low_level {
                    level = low_level;
                }
                current_col += 1;
            }
        }
    }
    if level as OptInt > (*wp).w_onebuf_opt.wo_fdn {
        level = (*wp).w_onebuf_opt.wo_fdn as ::core::ffi::c_int;
        if level < 0 as ::core::ffi::c_int {
            level = 0 as ::core::ffi::c_int;
        }
    }
    return level;
}
#[no_mangle]
pub unsafe extern "C" fn ex_syntime(mut eap: *mut exarg_T) {
    if strcmp((*eap).arg, b"on\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        syn_time_on = true_0 != 0;
    } else if strcmp((*eap).arg, b"off\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        syn_time_on = false_0 != 0;
    } else if strcmp(
        (*eap).arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        syntime_clear();
    } else if strcmp(
        (*eap).arg,
        b"report\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        syntime_report();
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    };
}
unsafe extern "C" fn syn_clear_time(mut st: *mut syn_time_T) {
    (*st).total = profile_zero();
    (*st).slowest = profile_zero();
    (*st).count = 0 as ::core::ffi::c_int;
    (*st).match_0 = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn syntime_clear() {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if !syntax_present(curwin) {
        msg(
            gettext(&raw mut msg_no_items as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin).w_s).b_syn_patterns.ga_len {
        spp = ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        syn_clear_time(&raw mut (*spp).sp_time);
        idx += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_syntime_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match idx {
        0 => {
            return b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        1 => {
            return b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        2 => {
            return b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        3 => {
            return b"report\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn syn_compare_syntime(
    mut v1: *const ::core::ffi::c_void,
    mut v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const time_entry_T = v1 as *const time_entry_T;
    let mut s2: *const time_entry_T = v2 as *const time_entry_T;
    return profile_cmp((*s1).total, (*s2).total);
}
unsafe extern "C" fn syntime_report() {
    if !syntax_present(curwin) {
        msg(
            gettext(&raw mut msg_no_items as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<time_entry_T>() as ::core::ffi::c_int,
        50 as ::core::ffi::c_int,
    );
    let mut total_total: proftime_T = profile_zero();
    let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut time_entry_T = ::core::ptr::null_mut::<time_entry_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin).w_s).b_syn_patterns.ga_len {
        let mut spp: *mut synpat_T =
            ((*(*curwin).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_time.count > 0 as ::core::ffi::c_int {
            p = ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<time_entry_T>())
                as *mut time_entry_T;
            (*p).total = (*spp).sp_time.total;
            total_total = profile_add(total_total, (*spp).sp_time.total);
            (*p).count = (*spp).sp_time.count;
            (*p).match_0 = (*spp).sp_time.match_0;
            total_count += (*spp).sp_time.count;
            (*p).slowest = (*spp).sp_time.slowest;
            let mut tm: proftime_T = profile_divide((*spp).sp_time.total, (*spp).sp_time.count);
            (*p).average = tm;
            (*p).id = (*spp).sp_syn.id as ::core::ffi::c_int;
            (*p).pattern = (*spp).sp_pattern;
        }
        idx += 1;
    }
    if ga.ga_len > 1 as ::core::ffi::c_int {
        qsort(
            ga.ga_data,
            ga.ga_len as size_t,
            ::core::mem::size_of::<time_entry_T>(),
            Some(
                syn_compare_syntime
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    msg_puts_title(gettext(
        b"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx_0 < ga.ga_len && !got_int {
        p = (ga.ga_data as *mut time_entry_T).offset(idx_0 as isize);
        msg_puts(profile_msg((*p).total));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(13 as ::core::ffi::c_int);
        msg_outnum((*p).count);
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(20 as ::core::ffi::c_int);
        msg_outnum((*p).match_0);
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(26 as ::core::ffi::c_int);
        msg_puts(profile_msg((*p).slowest));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(38 as ::core::ffi::c_int);
        msg_puts(profile_msg((*p).average));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(50 as ::core::ffi::c_int);
        msg_outtrans(
            highlight_group_name((*p).id - 1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(69 as ::core::ffi::c_int);
        let mut len: ::core::ffi::c_int = 0;
        if Columns < 80 as ::core::ffi::c_int {
            len = 20 as ::core::ffi::c_int;
        } else {
            len = Columns - 70 as ::core::ffi::c_int;
        }
        let mut patlen: ::core::ffi::c_int = strlen((*p).pattern) as ::core::ffi::c_int;
        len = if len < patlen { len } else { patlen };
        msg_outtrans_len((*p).pattern, len, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        idx_0 += 1;
    }
    ga_clear(&raw mut ga);
    if !got_int {
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts(profile_msg(total_total));
        msg_advance(13 as ::core::ffi::c_int);
        msg_outnum(total_count);
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    namelist1 = [
        keyvalue_T {
            key: HL_DISPLAY as ::core::ffi::c_int,
            value: b"display\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONTAINED as ::core::ffi::c_int,
            value: b"contained\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_ONELINE as ::core::ffi::c_int,
            value: b"oneline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_KEEPEND as ::core::ffi::c_int,
            value: b"keepend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_EXTEND as ::core::ffi::c_int,
            value: b"extend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_EXCLUDENL as ::core::ffi::c_int,
            value: b"excludenl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_TRANSP as ::core::ffi::c_int,
            value: b"transparent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_FOLD as ::core::ffi::c_int,
            value: b"fold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONCEAL as ::core::ffi::c_int,
            value: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONCEALENDS as ::core::ffi::c_int,
            value: b"concealends\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        },
    ];
    namelist2 = [
        keyvalue_T {
            key: HL_SKIPWHITE as ::core::ffi::c_int,
            value: b"skipwhite\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_SKIPNL as ::core::ffi::c_int,
            value: b"skipnl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_SKIPEMPTY as ::core::ffi::c_int,
            value: b"skipempty\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
