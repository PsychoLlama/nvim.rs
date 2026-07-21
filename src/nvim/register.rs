use crate::src::nvim::clipboard;
use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn memcnt(data: *const ::core::ffi::c_void, c: ::core::ffi::c_char, len: size_t) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cbuf_to_string(buf: *const ::core::ffi::c_char, size: size_t) -> String_0;
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    fn buflist_findpat(
        pattern: *const ::core::ffi::c_char,
        pattern_end: *const ::core::ffi::c_char,
        unlisted: bool,
        diffmode: bool,
        curtab_only: bool,
    ) -> ::core::ffi::c_int;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buflist_name_nr(
        fnum: ::core::ffi::c_int,
        fname: *mut *mut ::core::ffi::c_char,
        lnum: *mut linenr_T,
    ) -> ::core::ffi::c_int;
    fn getaltfname(errmsg: bool) -> *mut ::core::ffi::c_char;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn buf_updates_send_changes(
        buf: *mut buf_T,
        firstline: linenr_T,
        num_added: int64_t,
        num_removed: int64_t,
    );
    fn changed_bytes(lnum: linenr_T, col: colnr_T);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn del_chars(count: ::core::ffi::c_int, fixpos: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static p_ch: GlobalCell<OptInt>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_report: GlobalCell<OptInt>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    fn vim_strsave_escaped_ext(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
        cc: ::core::ffi::c_char,
        bsl: bool,
    ) -> *mut ::core::ffi::c_char;
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
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getviscol() -> ::core::ffi::c_int;
    fn coladvance_force(wcol: colnr_T) -> ::core::ffi::c_int;
    fn getvpos(wp: *mut win_T, pos: *mut pos_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn get_cursor_pos_len() -> colnr_T;
    fn update_screen() -> ::core::ffi::c_int;
    fn showmode() -> ::core::ffi::c_int;
    fn beginline(flags: ::core::ffi::c_int);
    fn oneright() -> ::core::ffi::c_int;
    fn stuff_inserted(
        c: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        no_esc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn get_last_insert() -> String_0;
    fn get_last_insert_save() -> *mut ::core::ffi::c_char;
    static e_noinstext: [::core::ffi::c_char; 0];
    static e_nolastcmd: [::core::ffi::c_char; 0];
    static e_noprevre: [::core::ffi::c_char; 0];
    static e_nobufnr: [::core::ffi::c_char; 0];
    static e_resulting_text_too_long: [::core::ffi::c_char; 0];
    fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T;
    fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T);
    fn eval_to_string(
        arg: *mut ::core::ffi::c_char,
        join_list: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    static msg_ext_skip_flush: GlobalCell<bool>;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn emsg_invreg(name: ::core::ffi::c_int);
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msgmore(n: ::core::ffi::c_int);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans_len(
        msgstr: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn message_filtered(msg_0: *const ::core::ffi::c_char) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_append_allocated_string(l: *mut list_T, str: *mut ::core::ffi::c_char);
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_bool(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: BoolVarValue,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_str(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_dict_set_keys_readonly(dict: *mut dict_T);
    fn check_fname() -> ::core::ffi::c_int;
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
    fn extmark_splice_cols(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        old_col: colnr_T,
        new_col: colnr_T,
        undo: ExtmarkOp,
    );
    fn getcmdline(
        firstc: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn cmdline_paste_str(s: *const ::core::ffi::c_char, literally: bool);
    fn file_name_at_cursor(
        options: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        file_lnum: *mut linenr_T,
    ) -> *mut ::core::ffi::c_char;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_set_growsize(gap: *mut garray_T, growsize: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn get_recorded() -> *mut ::core::ffi::c_char;
    fn beep_flush();
    fn AppendCharToRedobuff(c: ::core::ffi::c_int);
    fn stuffReadbuff(s: *const ::core::ffi::c_char);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn stuffescaped(arg: *const ::core::ffi::c_char, literally: bool);
    fn ins_typebuf(
        str: *mut ::core::ffi::c_char,
        noremap: ::core::ffi::c_int,
        offset: ::core::ffi::c_int,
        nottyped: bool,
        silent: bool,
    ) -> ::core::ffi::c_int;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static curwin: GlobalCell<*mut win_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static textlock: GlobalCell<::core::ffi::c_int>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static State: GlobalCell<::core::ffi::c_int>;
    static reg_recording: GlobalCell<::core::ffi::c_int>;
    static reg_executing: GlobalCell<::core::ffi::c_int>;
    static pending_end_reg_executing: GlobalCell<bool>;
    static reg_recorded: GlobalCell<::core::ffi::c_int>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static cmdmod: GlobalCell<cmdmod_T>;
    static must_redraw: GlobalCell<::core::ffi::c_int>;
    static got_int: GlobalCell<bool>;
    static last_cmdline: GlobalCell<*mut ::core::ffi::c_char>;
    static new_last_cmdline: GlobalCell<*mut ::core::ffi::c_char>;
    static redir_reg: GlobalCell<::core::ffi::c_int>;
    fn tabstop_padding(col: colnr_T, ts_arg: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int;
    fn get_indent() -> ::core::ffi::c_int;
    fn set_indent(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> bool;
    fn preprocs_left() -> bool;
    fn ins_compl_preinsert_effect() -> bool;
    fn ins_compl_delete(new_leader: bool);
    fn vim_strsave_escape_ks(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_unescape_ks(p: *mut ::core::ffi::c_char);
    fn os_time() -> Timestamp;
    fn mark_adjust(
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
        op: ExtmarkOp,
    );
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utf_ptr2cells_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn mb_string2cells_len(str: *const ::core::ffi::c_char, size: size_t) -> size_t;
    fn utf_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    static utf8len_tab: [uint8_t; 256];
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace(lnum: linenr_T, line: *mut ::core::ffi::c_char, copy: bool)
        -> ::core::ffi::c_int;
    fn decl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn changed_cline_bef_curs(wp: *mut win_T);
    fn invalidate_botline_win(wp: *mut win_T);
    fn find_ident_under_cursor(
        text: *mut *mut ::core::ffi::c_char,
        find_type: ::core::ffi::c_int,
        offset: *mut ::core::ffi::c_int,
    ) -> size_t;
    fn get_op_char(optype: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn adjust_cursor_eol();
    fn block_prep(oap: *mut oparg_T, bdp: *mut block_def, lnum: linenr_T, is_del: bool);
    fn charwise_block_prep(
        start: pos_T,
        end: pos_T,
        bdp: *mut block_def,
        lnum: linenr_T,
        inclusive: bool,
    );
    fn get_ve_flags(wp: *mut win_T) -> ::core::ffi::c_uint;
    fn os_breakcheck();
    fn init_charsize_arg(
        csarg: *mut CharsizeArg,
        wp: *mut win_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
    ) -> CSType;
    fn charsize_regular(
        csarg: *mut CharsizeArg,
        cur: *mut ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn charsize_fast(
        csarg: *mut CharsizeArg,
        cur: *const ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn last_search_pat() -> *mut ::core::ffi::c_char;
    fn set_last_search_pat(
        s: *const ::core::ffi::c_char,
        idx: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
        setlast: bool,
    );
    fn terminal_paste(count: ::core::ffi::c_int, y_array: *mut String_0, y_size: size_t);
    fn ui_has(ext: UIExtension) -> bool;
    fn u_save_cursor() -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
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
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
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
    pub b_wininfo: C2Rust_Unnamed_11,
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
    pub uh_next: C2Rust_Unnamed_10,
    pub uh_prev: C2Rust_Unnamed_9,
    pub uh_alt_next: C2Rust_Unnamed_8,
    pub uh_alt_prev: C2Rust_Unnamed_7,
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
    pub data: C2Rust_Unnamed_6,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
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
pub struct C2Rust_Unnamed_11 {
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
    pub type_0: C2Rust_Unnamed_12,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_16; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
pub type ExtmarkOp = ::core::ffi::c_uint;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
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
    pub cs_pend: C2Rust_Unnamed_17,
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
pub union C2Rust_Unnamed_17 {
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_18 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_18 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_18 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_18 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_18 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_18 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_18 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_18 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_18 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_18 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_18 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_18 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_18 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_18 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    pub cmod_flags: ::core::ffi::c_int,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: regmatch_T,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_19 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_19 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_19 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_19 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_19 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_19 = 4;
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct oparg_T {
    pub op_type: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub motion_type: MotionType,
    pub motion_force: ::core::ffi::c_int,
    pub use_reg_one: bool,
    pub inclusive: bool,
    pub end_adjusted: bool,
    pub start: pos_T,
    pub end: pos_T,
    pub cursor_start: pos_T,
    pub line_count: linenr_T,
    pub empty: bool,
    pub is_VIsual: bool,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub prev_opcount: ::core::ffi::c_int,
    pub prev_count0: ::core::ffi::c_int,
    pub excl_tr_ws: bool,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_20 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_20 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_20 = 16;
pub const PUT_LINE: C2Rust_Unnamed_20 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_20 = 4;
pub const PUT_CURSEND: C2Rust_Unnamed_20 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NUM_REGISTERS: C2Rust_Unnamed_21 = 39;
pub const PLUS_REGISTER: C2Rust_Unnamed_21 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_21 = 37;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_21 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_21 = 36;
pub type GRegFlags = ::core::ffi::c_uint;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const kGRegNoExpr: GRegFlags = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct block_def {
    pub startspaces: ::core::ffi::c_int,
    pub endspaces: ::core::ffi::c_int,
    pub textlen: ::core::ffi::c_int,
    pub textstart: *mut ::core::ffi::c_char,
    pub textcol: colnr_T,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub is_short: ::core::ffi::c_int,
    pub is_MAX: ::core::ffi::c_int,
    pub is_oneChar: ::core::ffi::c_int,
    pub pre_whitesp: ::core::ffi::c_int,
    pub pre_whitesp_c: ::core::ffi::c_int,
    pub end_char_vcols: colnr_T,
    pub start_char_vcols: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_22 = 2;
pub const YREG_YANK: C2Rust_Unnamed_22 = 1;
pub const YREG_PASTE: C2Rust_Unnamed_22 = 0;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_23 = 4;
pub const BL_SOL: C2Rust_Unnamed_23 = 2;
pub const BL_WHITE: C2Rust_Unnamed_23 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_24 = 32;
pub const FNAME_REL: C2Rust_Unnamed_24 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_24 = 8;
pub const FNAME_HYP: C2Rust_Unnamed_24 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_24 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_24 = 1;
pub type RemapValues = ::core::ffi::c_int;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_25 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_25 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_25 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_25 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_25 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_25 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_25 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_25 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_25 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_25 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_25 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_25 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_25 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_25 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_25 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_25 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_25 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_25 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_25 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const SIN_NOMARK: C2Rust_Unnamed_26 = 8;
pub const SIN_UNDO: C2Rust_Unnamed_26 = 4;
pub const SIN_INSERT: C2Rust_Unnamed_26 = 2;
pub const SIN_CHANGED: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_27 = 4;
pub const FIND_STRING: C2Rust_Unnamed_27 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_27 = 1;
pub type CSType = bool;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const kCharsizeFast: C2Rust_Unnamed_28 = 1;
pub const kCharsizeRegular: C2Rust_Unnamed_28 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharsizeArg {
    pub win: *mut win_T,
    pub line: *mut ::core::ffi::c_char,
    pub use_tabstop: bool,
    pub indent_width: ::core::ffi::c_int,
    pub virt_row: ::core::ffi::c_int,
    pub cur_text_width_left: ::core::ffi::c_int,
    pub cur_text_width_right: ::core::ffi::c_int,
    pub max_head_vcol: ::core::ffi::c_int,
    pub iter: [MarkTreeIter; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
pub const RE_SEARCH: C2Rust_Unnamed_29 = 0;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_29 = 2;
pub const RE_BOTH: C2Rust_Unnamed_29 = 2;
pub const RE_SUBST: C2Rust_Unnamed_29 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_REGAPPEND: ::core::ffi::c_int = '>' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
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
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
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
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
static expr_line: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static execreg_lastc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(NUL);
static y_regs: GlobalCell<[yankreg_T; 39]> = GlobalCell::new([
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
]);
static y_previous: GlobalCell<*mut yankreg_T> =
    GlobalCell::new(::core::ptr::null_mut::<yankreg_T>());
static e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines: GlobalCell<
    [::core::ffi::c_char; 79],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 79], [::core::ffi::c_char; 79]>(
        *b"E883: Search pattern and expression register may not contain two or more lines\0",
    )
});
#[no_mangle]
pub unsafe extern "C" fn get_unname_register() -> ::core::ffi::c_int {
    return if (*y_previous.ptr()).is_null() {
        -1 as ::core::ffi::c_int
    } else {
        (*y_previous.ptr())
            .offset_from((y_regs.ptr() as *mut yankreg_T).offset(0 as ::core::ffi::c_int as isize))
            as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_y_register(mut reg: ::core::ffi::c_int) -> *mut yankreg_T {
    return (y_regs.ptr() as *mut yankreg_T).offset(reg as isize);
}
#[no_mangle]
pub unsafe extern "C" fn get_y_previous() -> *mut yankreg_T {
    return y_previous.get();
}
#[no_mangle]
pub unsafe extern "C" fn get_expr_register() -> ::core::ffi::c_int {
    let mut new_line: *mut ::core::ffi::c_char = getcmdline(
        '=' as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
    );
    if new_line.is_null() {
        return NUL;
    }
    if *new_line as ::core::ffi::c_int == NUL {
        xfree(new_line as *mut ::core::ffi::c_void);
    } else {
        set_expr_line(new_line);
    }
    return '=' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_expr_line(mut new_line: *mut ::core::ffi::c_char) {
    xfree(expr_line.get() as *mut ::core::ffi::c_void);
    expr_line.set(new_line);
}
#[no_mangle]
pub unsafe extern "C" fn get_expr_line() -> *mut ::core::ffi::c_char {
    static nested: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if (*expr_line.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut expr_copy: *mut ::core::ffi::c_char = xstrdup(expr_line.get());
    if nested.get() >= 10 as ::core::ffi::c_int {
        return expr_copy;
    }
    (*nested.ptr()) += 1;
    let mut rv: *mut ::core::ffi::c_char = eval_to_string(expr_copy, true_0 != 0, false_0 != 0);
    (*nested.ptr()) -= 1;
    xfree(expr_copy as *mut ::core::ffi::c_void);
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn get_expr_line_src() -> *mut ::core::ffi::c_char {
    if (*expr_line.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup(expr_line.get());
}
#[no_mangle]
pub unsafe extern "C" fn valid_yank_reg(
    mut regname: ::core::ffi::c_int,
    mut writing: bool,
) -> bool {
    if regname > 0 as ::core::ffi::c_int
        && (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0)
        || !writing
            && !vim_strchr(b"/.%:=\0".as_ptr() as *const ::core::ffi::c_char, regname).is_null()
        || regname == '#' as ::core::ffi::c_int
        || regname == '"' as ::core::ffi::c_int
        || regname == '-' as ::core::ffi::c_int
        || regname == '_' as ::core::ffi::c_int
        || regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_default_register_name() -> ::core::ffi::c_int {
    let mut name: ::core::ffi::c_int = NUL;
    clipboard::adjust_clipboard_name(&mut name, true, false);
    return name;
}
#[no_mangle]
pub unsafe extern "C" fn op_reg_iter(
    iter: *const ::core::ffi::c_void,
    regs: *const yankreg_T,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    *name = NUL as ::core::ffi::c_char;
    let mut iter_reg: *const yankreg_T = if iter.is_null() {
        regs.offset(0 as ::core::ffi::c_int as isize)
    } else {
        iter as *const yankreg_T
    };
    while iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
        < NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        && reg_empty(iter_reg) as ::core::ffi::c_int != 0
    {
        iter_reg = iter_reg.offset(1);
    }
    if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
        == NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        || reg_empty(iter_reg) as ::core::ffi::c_int != 0
    {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let mut iter_off: ::core::ffi::c_int =
        iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    *name = get_register_name(iter_off) as ::core::ffi::c_char;
    *reg = *iter_reg;
    *is_unnamed = iter_reg == y_previous.get() as *const yankreg_T;
    loop {
        iter_reg = iter_reg.offset(1);
        if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
            >= NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        {
            break;
        }
        if !reg_empty(iter_reg) {
            return iter_reg as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_void>();
}
#[no_mangle]
pub unsafe extern "C" fn op_global_reg_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    return op_reg_iter(iter, y_regs.ptr() as *mut yankreg_T, name, reg, is_unnamed);
}
#[no_mangle]
pub unsafe extern "C" fn op_reg_amount() -> size_t {
    let mut ret: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < NUM_SAVED_REGISTERS as ::core::ffi::c_int as size_t {
        if !reg_empty((y_regs.ptr() as *mut yankreg_T).offset(i as isize)) {
            ret = ret.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn op_reg_set(
    name: ::core::ffi::c_char,
    reg: yankreg_T,
    mut is_unnamed: bool,
) -> bool {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    free_register((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    (*y_regs.ptr())[i as usize] = reg;
    if is_unnamed {
        y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn op_reg_get(name: ::core::ffi::c_char) -> *const yankreg_T {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return ::core::ptr::null::<yankreg_T>();
    }
    return (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn op_reg_set_previous(name: ::core::ffi::c_char) -> bool {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn update_yankreg_width(mut reg: *mut yankreg_T) {
    if (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        let mut maxlen: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            let mut rowlen: size_t = mb_string2cells_len(
                (*(*reg).y_array.offset(i as isize)).data,
                (*(*reg).y_array.offset(i as isize)).size,
            );
            maxlen = if maxlen > rowlen { maxlen } else { rowlen };
            i = i.wrapping_add(1);
        }
        '_c2rust_label: {
            if maxlen <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"maxlen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    295 as ::core::ffi::c_uint,
                    b"void update_yankreg_width(yankreg_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*reg).y_width = (if (*reg).y_width > maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            (*reg).y_width as ::core::ffi::c_int
        } else {
            maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        }) as colnr_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_yank_register(
    mut regname: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> *mut yankreg_T {
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    if (mode == YREG_PASTE as ::core::ffi::c_int || mode == YREG_PUT as ::core::ffi::c_int)
        && clipboard::get_clipboard(regname, &mut reg, false)
    {
        return reg;
    } else if mode == YREG_PUT as ::core::ffi::c_int
        && (regname == '*' as ::core::ffi::c_int || regname == '+' as ::core::ffi::c_int)
    {
        static empty_reg: GlobalCell<yankreg_T> = GlobalCell::new(yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        });
        return empty_reg.ptr();
    } else if mode != YREG_YANK as ::core::ffi::c_int
        && (regname == 0 as ::core::ffi::c_int
            || regname == '"' as ::core::ffi::c_int
            || regname == '*' as ::core::ffi::c_int
            || regname == '+' as ::core::ffi::c_int)
        && !(*y_previous.ptr()).is_null()
    {
        return y_previous.get();
    }
    let mut i: ::core::ffi::c_int = op_reg_index(regname);
    if i == -1 as ::core::ffi::c_int {
        i = 0 as ::core::ffi::c_int;
    }
    reg = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
    if mode == YREG_YANK as ::core::ffi::c_int {
        y_previous.set(reg);
    }
    return reg;
}
#[no_mangle]
pub unsafe extern "C" fn yank_register_mline(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut *mut yankreg_T,
) -> bool {
    *reg = ::core::ptr::null_mut::<yankreg_T>();
    if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, false_0 != 0) {
        return false_0 != 0;
    }
    if regname == '_' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    *reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    return (**reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn copy_register(mut name: ::core::ffi::c_int) -> *mut yankreg_T {
    let mut reg: *mut yankreg_T = get_yank_register(name, YREG_PASTE as ::core::ffi::c_int);
    let mut copy: *mut yankreg_T = xmalloc(::core::mem::size_of::<yankreg_T>()) as *mut yankreg_T;
    *copy = *reg;
    if (*copy).y_size == 0 as size_t {
        (*copy).y_array = ::core::ptr::null_mut::<String_0>();
    } else {
        (*copy).y_array =
            xcalloc((*copy).y_size, ::core::mem::size_of::<String_0>()) as *mut String_0;
        let mut i: size_t = 0 as size_t;
        while i < (*copy).y_size {
            *(*copy).y_array.offset(i as isize) = copy_string(
                *(*reg).y_array.offset(i as isize),
                ::core::ptr::null_mut::<Arena>(),
            );
            i = i.wrapping_add(1);
        }
    }
    return copy;
}
unsafe extern "C" fn stuff_yank(
    mut regname: ::core::ffi::c_int,
    mut p: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, true_0 != 0) {
        xfree(p as *mut ::core::ffi::c_void);
        return FAIL;
    }
    if regname == '_' as ::core::ffi::c_int {
        xfree(p as *mut ::core::ffi::c_void);
        return OK;
    }
    let plen: size_t = strlen(p);
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_YANK as ::core::ffi::c_int);
    if is_append_register(regname) as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
        let mut pp: *mut String_0 = (*reg)
            .y_array
            .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize);
        let tmplen: size_t = (*pp).size.wrapping_add(plen);
        let mut tmp: *mut ::core::ffi::c_char =
            xmalloc(tmplen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            tmp as *mut ::core::ffi::c_void,
            (*pp).data as *const ::core::ffi::c_void,
            (*pp).size,
        );
        memcpy(
            tmp.offset((*pp).size as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            plen,
        );
        *tmp.offset(tmplen as isize) = NUL as ::core::ffi::c_char;
        xfree(p as *mut ::core::ffi::c_void);
        xfree((*pp).data as *mut ::core::ffi::c_void);
        *pp = String_0 {
            data: tmp,
            size: tmplen,
        };
    } else {
        free_register(reg);
        (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        (*reg).y_array = xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
        *(*reg).y_array.offset(0 as ::core::ffi::c_int as isize) = String_0 {
            data: p,
            size: plen,
        };
        (*reg).y_size = 1 as size_t;
        (*reg).y_type = kMTCharWise;
    }
    (*reg).timestamp = os_time();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn do_record(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static regname: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    let mut retval: ::core::ffi::c_int = 0;
    if reg_recording.get() == 0 as ::core::ffi::c_int {
        if c < 0 as ::core::ffi::c_int
            || !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(c) as ::core::ffi::c_int != 0)
                && c != '"' as ::core::ffi::c_int
        {
            retval = FAIL;
        } else {
            reg_recording.set(c);
            showmode();
            regname.set(c);
            retval = OK;
            apply_autocmds(
                EVENT_RECORDINGENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    } else {
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        let mut p: *mut ::core::ffi::c_char = get_recorded();
        if !p.is_null() {
            vim_unescape_ks(p);
            tv_dict_add_str(
                dict,
                b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                p,
            );
        }
        let mut buf: [::core::ffi::c_char; 67] = [0; 67];
        buf[0 as ::core::ffi::c_int as usize] = regname.get() as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            b"regname\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        tv_dict_set_keys_readonly(dict);
        apply_autocmds(
            EVENT_RECORDINGLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
        reg_recorded.set(reg_recording.get());
        reg_recording.set(0 as ::core::ffi::c_int);
        if p_ch.get() == 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            showmode();
        } else {
            msg(
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
        if p.is_null() {
            retval = FAIL;
        } else {
            let mut old_y_previous: *mut yankreg_T = y_previous.get();
            retval = stuff_yank(regname.get(), p);
            y_previous.set(old_y_previous);
        }
    }
    return retval;
}
unsafe extern "C" fn put_in_typebuf(
    mut s: *mut ::core::ffi::c_char,
    mut esc: bool,
    mut colon: bool,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    put_reedit_in_typebuf(silent);
    if colon {
        retval = ins_typebuf(
            b"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            silent != 0,
        );
    }
    if retval == OK {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if esc {
            p = vim_strsave_escape_ks(s);
        } else {
            p = s;
        }
        if p.is_null() {
            retval = FAIL;
        } else {
            retval = ins_typebuf(
                p,
                if esc as ::core::ffi::c_int != 0 {
                    REMAP_NONE as ::core::ffi::c_int
                } else {
                    REMAP_YES as ::core::ffi::c_int
                },
                0 as ::core::ffi::c_int,
                true_0 != 0,
                silent != 0,
            );
        }
        if esc {
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
    if colon as ::core::ffi::c_int != 0 && retval == OK {
        retval = ins_typebuf(
            b":\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            silent != 0,
        );
    }
    return retval;
}
unsafe extern "C" fn put_reedit_in_typebuf(mut silent: ::core::ffi::c_int) {
    let mut buf: [uint8_t; 3] = [0; 3];
    if restart_edit.get() == NUL {
        return;
    }
    if restart_edit.get() == 'V' as ::core::ffi::c_int {
        buf[0 as ::core::ffi::c_int as usize] = 'g' as uint8_t;
        buf[1 as ::core::ffi::c_int as usize] = 'R' as uint8_t;
        buf[2 as ::core::ffi::c_int as usize] = NUL as uint8_t;
    } else {
        buf[0 as ::core::ffi::c_int as usize] = (if restart_edit.get() == 'I' as ::core::ffi::c_int
        {
            'i' as ::core::ffi::c_int
        } else {
            restart_edit.get()
        }) as uint8_t;
        buf[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
    }
    if ins_typebuf(
        &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
        REMAP_NONE as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
        silent != 0,
    ) == OK
    {
        restart_edit.set(NUL);
    }
}
unsafe extern "C" fn execreg_line_continuation(
    mut lines: *mut String_0,
    mut idx: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut cmd_start: size_t = *idx;
    '_c2rust_label: {
        if cmd_start > 0 as size_t {
        } else {
            __assert_fail(
                b"cmd_start > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                575 as ::core::ffi::c_uint,
                b"char *execreg_line_continuation(String *, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let cmd_end: size_t = cmd_start;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        400 as ::core::ffi::c_int,
    );
    loop {
        cmd_start = cmd_start.wrapping_sub(1);
        if cmd_start <= 0 as size_t {
            break;
        }
        let mut p: *mut ::core::ffi::c_char = skipwhite((*lines.offset(cmd_start as isize)).data);
        if *p as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
            && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ' ' as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut tmp: *mut String_0 = lines.offset(cmd_start as isize);
    ga_concat_len(&raw mut ga, (*tmp).data, (*tmp).size);
    let mut j: size_t = cmd_start.wrapping_add(1 as size_t);
    while j <= cmd_end {
        tmp = lines.offset(j as isize);
        let mut p_0: *mut ::core::ffi::c_char = skipwhite((*tmp).data);
        if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            if ga.ga_len > 400 as ::core::ffi::c_int {
                ga_set_growsize(
                    &raw mut ga,
                    if ga.ga_len < 8000 as ::core::ffi::c_int {
                        ga.ga_len
                    } else {
                        8000 as ::core::ffi::c_int
                    },
                );
            }
            p_0 = p_0.offset(1);
            ga_concat_len(
                &raw mut ga,
                p_0,
                (*tmp).data.offset((*tmp).size as isize).offset_from(p_0) as size_t,
            );
        }
        j = j.wrapping_add(1);
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    let mut str: *mut ::core::ffi::c_char =
        xmemdupz(ga.ga_data, ga.ga_len as size_t) as *mut ::core::ffi::c_char;
    ga_clear(&raw mut ga);
    *idx = cmd_start;
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn do_execreg(
    mut regname: ::core::ffi::c_int,
    mut colon: ::core::ffi::c_int,
    mut addcr: ::core::ffi::c_int,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    if regname == '@' as ::core::ffi::c_int {
        if execreg_lastc.get() == NUL {
            emsg(gettext(
                b"E748: No previously used register\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        regname = execreg_lastc.get();
    }
    if regname == '%' as ::core::ffi::c_int
        || regname == '#' as ::core::ffi::c_int
        || !valid_yank_reg(regname, false_0 != 0)
    {
        emsg_invreg(regname);
        return FAIL;
    }
    execreg_lastc.set(regname);
    if regname == '_' as ::core::ffi::c_int {
        return OK;
    }
    if regname == ':' as ::core::ffi::c_int {
        if (*last_cmdline.ptr()).is_null() {
            emsg(gettext(
                &raw const e_nolastcmd as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            new_last_cmdline.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped_ext(
            last_cmdline.get(),
            b"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\0"
                .as_ptr() as *const ::core::ffi::c_char,
            Ctrl_V as ::core::ffi::c_char,
            false_0 != 0,
        );
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && strncmp(
                p,
                b"'<,'>\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            retval = put_in_typebuf(
                p.offset(5 as ::core::ffi::c_int as isize),
                true_0 != 0,
                true_0 != 0,
                silent,
            );
        } else {
            retval = put_in_typebuf(p, true_0 != 0, true_0 != 0, silent);
        }
        xfree(p as *mut ::core::ffi::c_void);
    } else if regname == '=' as ::core::ffi::c_int {
        let mut p_0: *mut ::core::ffi::c_char = get_expr_line();
        if p_0.is_null() {
            return FAIL;
        }
        retval = put_in_typebuf(p_0, true_0 != 0, colon != 0, silent);
        xfree(p_0 as *mut ::core::ffi::c_void);
    } else if regname == '.' as ::core::ffi::c_int {
        let mut p_1: *mut ::core::ffi::c_char = get_last_insert_save();
        if p_1.is_null() {
            emsg(gettext(
                &raw const e_noinstext as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        retval = put_in_typebuf(p_1, false_0 != 0, colon != 0, silent);
        xfree(p_1 as *mut ::core::ffi::c_void);
    } else {
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        if (*reg).y_array.is_null() {
            return FAIL;
        }
        let mut remap: ::core::ffi::c_int = if colon != 0 {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            REMAP_YES as ::core::ffi::c_int
        };
        put_reedit_in_typebuf(silent);
        let mut i: size_t = (*reg).y_size;
        loop {
            let c2rust_fresh1 = i;
            i = i.wrapping_sub(1);
            if c2rust_fresh1 <= 0 as size_t {
                break;
            }
            if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                || i < (*reg).y_size.wrapping_sub(1 as size_t)
                || addcr != 0
            {
                if ins_typebuf(
                    b"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    remap,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent != 0,
                ) == FAIL
                {
                    return FAIL;
                }
            }
            let mut str: *mut ::core::ffi::c_char = (*(*reg).y_array.offset(i as isize)).data;
            let mut free_str: bool = false_0 != 0;
            if colon != 0 && i > 0 as size_t {
                let mut p_2: *mut ::core::ffi::c_char = skipwhite(str);
                if *p_2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                        && *p_2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p_2.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                {
                    str = execreg_line_continuation((*reg).y_array, &raw mut i);
                    free_str = true_0 != 0;
                }
            }
            let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(str);
            if free_str {
                xfree(str as *mut ::core::ffi::c_void);
            }
            retval = ins_typebuf(
                escaped,
                remap,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                silent != 0,
            );
            xfree(escaped as *mut ::core::ffi::c_void);
            if retval == FAIL {
                return FAIL;
            }
            if colon != 0
                && ins_typebuf(
                    b":\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    remap,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent != 0,
                ) == FAIL
            {
                return FAIL;
            }
        }
        reg_executing.set(if regname == 0 as ::core::ffi::c_int {
            '"' as ::core::ffi::c_int
        } else {
            regname
        });
        pending_end_reg_executing.set(false_0 != 0);
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn insert_reg(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut literally_arg: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    let mut allocated: bool = false;
    let literally: bool = literally_arg as ::core::ffi::c_int != 0
        || is_literal_register(regname) as ::core::ffi::c_int != 0;
    os_breakcheck();
    if got_int.get() {
        return FAIL;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return FAIL;
    }
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if regname == '.' as ::core::ffi::c_int {
        retval = stuff_inserted(NUL, 1 as ::core::ffi::c_int, true_0);
    } else if get_spec_reg(regname, &raw mut arg, &raw mut allocated, true_0 != 0) {
        if arg.is_null() {
            return FAIL;
        }
        stuffescaped(arg, literally);
        if allocated {
            xfree(arg as *mut ::core::ffi::c_void);
        }
    } else {
        if reg.is_null() {
            reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        }
        if (*reg).y_array.is_null() {
            retval = FAIL;
        } else {
            let mut i: size_t = 0 as size_t;
            while i < (*reg).y_size {
                if regname == '-' as ::core::ffi::c_int
                    && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                {
                    let mut dir: Direction = BACKWARD;
                    if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        let mut curpos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        if u_save_cursor() == FAIL {
                            return FAIL;
                        }
                        del_chars(
                            mb_charlen(
                                (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
                            ),
                            true_0,
                        );
                        curpos = (*curwin.get()).w_cursor;
                        if oneright() == FAIL {
                            dir = FORWARD;
                        }
                        (*curwin.get()).w_cursor = curpos;
                    }
                    AppendCharToRedobuff(Ctrl_R);
                    AppendCharToRedobuff(regname);
                    do_put(
                        regname,
                        ::core::ptr::null_mut::<yankreg_T>(),
                        dir as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        PUT_CURSEND as ::core::ffi::c_int,
                    );
                } else {
                    stuffescaped((*(*reg).y_array.offset(i as isize)).data, literally);
                    if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                        || i < (*reg).y_size.wrapping_sub(1 as size_t)
                    {
                        stuffcharReadbuff('\n' as ::core::ffi::c_int);
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn get_spec_reg(
    mut regname: ::core::ffi::c_int,
    mut argp: *mut *mut ::core::ffi::c_char,
    mut allocated: *mut bool,
    mut errmsg: bool,
) -> bool {
    *argp = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *allocated = false_0 != 0;
    let mut cnt: size_t = 0;
    match regname {
        37 => {
            if errmsg {
                check_fname();
            }
            *argp = (*curbuf.get()).b_fname;
            return true_0 != 0;
        }
        35 => {
            *argp = getaltfname(errmsg);
            return true_0 != 0;
        }
        61 => {
            *argp = get_expr_line();
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        58 => {
            if (*last_cmdline.ptr()).is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    &raw const e_nolastcmd as *const ::core::ffi::c_char,
                ));
            }
            *argp = last_cmdline.get();
            return true_0 != 0;
        }
        47 => {
            if last_search_pat().is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
            }
            *argp = last_search_pat();
            return true_0 != 0;
        }
        46 => {
            *argp = get_last_insert_save();
            *allocated = true_0 != 0;
            if (*argp).is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    &raw const e_noinstext as *const ::core::ffi::c_char,
                ));
            }
            return true_0 != 0;
        }
        Ctrl_F | Ctrl_P => {
            if !errmsg {
                return false_0 != 0;
            }
            *argp = file_name_at_cursor(
                FNAME_MESS as ::core::ffi::c_int
                    | FNAME_HYP as ::core::ffi::c_int
                    | (if regname == Ctrl_P {
                        FNAME_EXP as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        Ctrl_W | Ctrl_A => {
            if !errmsg {
                return false_0 != 0;
            }
            cnt = find_ident_under_cursor(
                argp,
                if regname == Ctrl_W {
                    FIND_IDENT as ::core::ffi::c_int | FIND_STRING as ::core::ffi::c_int
                } else {
                    FIND_STRING as ::core::ffi::c_int
                },
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            *argp = (if cnt != 0 {
                xmemdupz(*argp as *const ::core::ffi::c_void, cnt)
            } else {
                NULL_0
            }) as *mut ::core::ffi::c_char;
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        Ctrl_L => {
            if !errmsg {
                return false_0 != 0;
            }
            *argp = ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum);
            return true_0 != 0;
        }
        95 => {
            *argp = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            return true_0 != 0;
        }
        _ => {}
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_paste_reg(
    mut regname: ::core::ffi::c_int,
    mut literally_arg: bool,
    mut remcr: bool,
) -> bool {
    let literally: bool = literally_arg as ::core::ffi::c_int != 0
        || is_literal_register(regname) as ::core::ffi::c_int != 0;
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    if (*reg).y_array.is_null() {
        return FAIL != 0;
    }
    let mut i: size_t = 0 as size_t;
    while i < (*reg).y_size {
        cmdline_paste_str((*(*reg).y_array.offset(i as isize)).data, literally);
        if i < (*reg).y_size.wrapping_sub(1 as size_t) && !remcr {
            cmdline_paste_str(b"\r\0".as_ptr() as *const ::core::ffi::c_char, literally);
        }
        os_breakcheck();
        if got_int.get() {
            return FAIL != 0;
        }
        i = i.wrapping_add(1);
    }
    return OK != 0;
}
#[no_mangle]
pub unsafe extern "C" fn shift_delete_registers(mut y_append: bool) {
    free_register((y_regs.ptr() as *mut yankreg_T).offset(9 as ::core::ffi::c_int as isize));
    let mut n: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
    while n > 1 as ::core::ffi::c_int {
        (*y_regs.ptr())[n as usize] = (*y_regs.ptr())[(n - 1 as ::core::ffi::c_int) as usize];
        n -= 1;
    }
    if !y_append {
        y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(1 as ::core::ffi::c_int as isize));
    }
    (*y_regs.ptr())[1 as ::core::ffi::c_int as usize].y_array = ::core::ptr::null_mut::<String_0>();
}
#[no_mangle]
pub unsafe extern "C" fn free_register(mut reg: *mut yankreg_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*reg).additional_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    if (*reg).y_array.is_null() {
        return;
    }
    let mut i: size_t = (*reg).y_size;
    loop {
        let c2rust_fresh0 = i;
        i = i.wrapping_sub(1);
        if c2rust_fresh0 <= 0 as size_t {
            break;
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*(*reg).y_array.offset(i as isize)).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        (*(*reg).y_array.offset(i as isize)).size = 0 as size_t;
    }
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*reg).y_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    let _ = *ptr__1;
}
unsafe extern "C" fn yank_copy_line(
    mut reg: *mut yankreg_T,
    mut bd: *mut block_def,
    mut y_idx: size_t,
    mut exclude_trailing_space: bool,
) {
    if exclude_trailing_space {
        (*bd).endspaces = 0 as ::core::ffi::c_int;
    }
    let mut size: ::core::ffi::c_int = (*bd).startspaces + (*bd).endspaces + (*bd).textlen;
    '_c2rust_label: {
        if size >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"size >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                985 as ::core::ffi::c_uint,
                b"void yank_copy_line(yankreg_T *, struct block_def *, size_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut pnew: *mut ::core::ffi::c_char = xmallocz(size as size_t) as *mut ::core::ffi::c_char;
    (*(*reg).y_array.offset(y_idx as isize)).data = pnew;
    memset(
        pnew as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).startspaces as size_t,
    );
    pnew = pnew.offset((*bd).startspaces as isize);
    memmove(
        pnew as *mut ::core::ffi::c_void,
        (*bd).textstart as *const ::core::ffi::c_void,
        (*bd).textlen as size_t,
    );
    pnew = pnew.offset((*bd).textlen as isize);
    memset(
        pnew as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).endspaces as size_t,
    );
    pnew = pnew.offset((*bd).endspaces as isize);
    if exclude_trailing_space {
        let mut s: ::core::ffi::c_int = (*bd).textlen + (*bd).endspaces;
        while s > 0 as ::core::ffi::c_int
            && ascii_iswhite(
                *(*bd)
                    .textstart
                    .offset(s as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        {
            s =
                s - utf_head_off(
                    (*bd).textstart,
                    (*bd)
                        .textstart
                        .offset(s as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize)),
                ) - 1 as ::core::ffi::c_int;
            pnew = pnew.offset(-1);
        }
    }
    *pnew = NUL as ::core::ffi::c_char;
    (*(*reg).y_array.offset(y_idx as isize)).size =
        pnew.offset_from((*(*reg).y_array.offset(y_idx as isize)).data) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn op_yank_reg(
    mut oap: *mut oparg_T,
    mut message: bool,
    mut reg: *mut yankreg_T,
    mut append: bool,
) {
    let mut newreg: yankreg_T = yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut yank_type: MotionType = (*oap).motion_type;
    let mut yanklines: size_t = (*oap).line_count as size_t;
    let mut yankendlnum: linenr_T = (*oap).end.lnum;
    let mut bd: block_def = block_def {
        startspaces: 0,
        endspaces: 0,
        textlen: 0,
        textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        textcol: 0,
        start_vcol: 0,
        end_vcol: 0,
        is_short: 0,
        is_MAX: 0,
        is_oneChar: 0,
        pre_whitesp: 0,
        pre_whitesp_c: 0,
        end_char_vcols: 0,
        start_char_vcols: 0,
    };
    let mut curr: *mut yankreg_T = reg;
    if append as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
        reg = &raw mut newreg;
    } else {
        free_register(reg);
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
        && (*oap).start.col == 0 as ::core::ffi::c_int
        && !(*oap).inclusive
        && (!(*oap).is_VIsual || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
        && (*oap).end.col == 0 as ::core::ffi::c_int
        && yanklines > 1 as size_t
    {
        yank_type = kMTLineWise;
        yankendlnum -= 1;
        yanklines = yanklines.wrapping_sub(1);
    }
    (*reg).y_size = yanklines;
    (*reg).y_type = yank_type;
    (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
    (*reg).y_array = xcalloc(yanklines, ::core::mem::size_of::<String_0>()) as *mut String_0;
    (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*reg).timestamp = os_time();
    let mut y_idx: size_t = 0 as size_t;
    let mut lnum: linenr_T = (*oap).start.lnum;
    if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        (*reg).y_width = (*oap).end_vcol - (*oap).start_vcol;
        if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int
            && (*reg).y_width > 0 as ::core::ffi::c_int
        {
            (*reg).y_width -= 1;
        }
    }
    while lnum <= yankendlnum {
        let mut tmp: ::core::ffi::c_int = 0;
        match (*reg).y_type as ::core::ffi::c_int {
            2 => {
                block_prep(oap, &raw mut bd, lnum, false_0 != 0);
                yank_copy_line(reg, &raw mut bd, y_idx, (*oap).excl_tr_ws);
            }
            1 => {
                *(*reg).y_array.offset(y_idx as isize) =
                    cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
            }
            0 => {
                charwise_block_prep(
                    (*oap).start,
                    (*oap).end,
                    &raw mut bd,
                    lnum,
                    (*oap).inclusive,
                );
                tmp = strlen(bd.textstart) as ::core::ffi::c_int;
                if tmp < bd.textlen {
                    bd.textlen = tmp;
                }
                yank_copy_line(reg, &raw mut bd, y_idx, false_0 != 0);
            }
            -1 => {
                abort();
            }
            _ => {}
        }
        lnum += 1;
        y_idx = y_idx.wrapping_add(1);
    }
    if curr != reg {
        let mut j: size_t = 0;
        let mut new_ptr: *mut String_0 = xmalloc(
            ::core::mem::size_of::<String_0>()
                .wrapping_mul((*curr).y_size.wrapping_add((*reg).y_size)),
        ) as *mut String_0;
        j = 0 as size_t;
        while j < (*curr).y_size {
            *new_ptr.offset(j as isize) = *(*curr).y_array.offset(j as isize);
            j = j.wrapping_add(1);
        }
        xfree((*curr).y_array as *mut ::core::ffi::c_void);
        (*curr).y_array = new_ptr;
        if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*curr).y_type = kMTLineWise;
        }
        if (*curr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && vim_strchr(p_cpo.get(), CPO_REGAPPEND).is_null()
        {
            let mut pnew: *mut ::core::ffi::c_char = xmalloc(
                (*(*curr)
                    .y_array
                    .offset((*curr).y_size.wrapping_sub(1 as size_t) as isize))
                .size
                .wrapping_add((*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size)
                .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            j = j.wrapping_sub(1);
            strcpy(pnew, (*(*curr).y_array.offset(j as isize)).data);
            strcpy(
                pnew.offset((*(*curr).y_array.offset(j as isize)).size as isize),
                (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
            );
            xfree((*(*curr).y_array.offset(j as isize)).data as *mut ::core::ffi::c_void);
            *(*curr).y_array.offset(j as isize) = String_0 {
                data: pnew,
                size: (*(*curr).y_array.offset(j as isize))
                    .size
                    .wrapping_add((*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size),
            };
            j = j.wrapping_add(1);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data
                    as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size = 0 as size_t;
            y_idx = 1 as size_t;
        } else {
            y_idx = 0 as size_t;
        }
        while y_idx < (*reg).y_size {
            let c2rust_fresh2 = y_idx;
            y_idx = y_idx.wrapping_add(1);
            let c2rust_fresh3 = j;
            j = j.wrapping_add(1);
            *(*curr).y_array.offset(c2rust_fresh3 as isize) =
                *(*reg).y_array.offset(c2rust_fresh2 as isize);
        }
        (*curr).y_size = j;
        xfree((*reg).y_array as *mut ::core::ffi::c_void);
    }
    if message {
        if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && yanklines == 1 as size_t
        {
            yanklines = 0 as size_t;
        }
        if yanklines > p_report.get() as size_t {
            let mut namebuf: [::core::ffi::c_char; 100] = [0; 100];
            if (*oap).regname == NUL {
                *(&raw mut namebuf as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
            } else {
                vim_snprintf(
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                    gettext(b" into \"%c\0".as_ptr() as *const ::core::ffi::c_char),
                    (*oap).regname,
                );
            }
            update_topline(curwin.get());
            if must_redraw.get() != 0 {
                update_screen();
            }
            if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"block of %ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"block of %ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        yanklines as ::core::ffi::c_ulong,
                    ),
                    yanklines as int64_t,
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        yanklines as ::core::ffi::c_ulong,
                    ),
                    yanklines as int64_t,
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                );
            }
        }
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
        if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_end.col = MAXCOL as ::core::ffi::c_int as colnr_T;
        }
        if yank_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int && !(*oap).inclusive
        {
            decl(&raw mut (*curbuf.get()).b_op_end);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_reg_type(
    mut reg_type: MotionType,
    mut reg_width: colnr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
) {
    '_c2rust_label: {
        if buf_len > 1 as size_t {
        } else {
            __assert_fail(
                b"buf_len > 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1176 as ::core::ffi::c_uint,
                b"void format_reg_type(MotionType, colnr_T, char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    match reg_type as ::core::ffi::c_int {
        1 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = 'V' as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        0 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = 'v' as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        2 => {
            snprintf(
                buf,
                buf_len,
                b"\x16%d\0".as_ptr() as *const ::core::ffi::c_char,
                reg_width as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        -1 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_textyankpost(mut oap: *mut oparg_T, mut reg: *mut yankreg_T) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() as ::core::ffi::c_int != 0 || !has_event(EVENT_TEXTYANKPOST) {
        return;
    }
    recursive.set(true_0 != 0);
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < (*reg).y_size {
        tv_list_append_string(
            list,
            (*(*reg).y_array.offset(i as isize)).data,
            (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
        );
        i = i.wrapping_add(1);
    }
    tv_list_set_lock(list, VAR_FIXED);
    tv_dict_add_list(
        dict,
        b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        list,
    );
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    format_reg_type(
        (*reg).y_type,
        (*reg).y_width,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 67]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    tv_dict_add_str(
        dict,
        b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    buf[0 as ::core::ffi::c_int as usize] = (*oap).regname as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"regname\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"inclusive\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (if (*oap).inclusive as ::core::ffi::c_int != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
    buf[0 as ::core::ffi::c_int as usize] = get_op_char((*oap).op_type) as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"operator\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"visual\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
    tv_dict_set_keys_readonly(dict);
    (*textlock.ptr()) += 1;
    apply_autocmds(
        EVENT_TEXTYANKPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    (*textlock.ptr()) -= 1;
    restore_v_event(dict, &raw mut save_v_event);
    recursive.set(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn op_yank(mut oap: *mut oparg_T, mut message: bool) -> bool {
    if (*oap).regname != 0 as ::core::ffi::c_int && !valid_yank_reg((*oap).regname, true_0 != 0) {
        beep_flush();
        return false_0 != 0;
    }
    if (*oap).regname == '_' as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut reg: *mut yankreg_T =
        get_yank_register((*oap).regname, YREG_YANK as ::core::ffi::c_int);
    op_yank_reg(oap, message, reg, is_append_register((*oap).regname));
    clipboard::set_clipboard((*oap).regname, reg);
    do_autocmd_textyankpost(oap, reg);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_put(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut split_pos: colnr_T = 0;
    let mut col: colnr_T = 0;
    let mut len_0: ::core::ffi::c_int = 0;
    let mut totlen: size_t = 0 as size_t;
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut y_type: MotionType = kMTCharWise;
    let mut y_size: size_t = 0;
    let mut y_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut y_array: *mut String_0 = ::core::ptr::null_mut::<String_0>();
    let mut nr_lines: linenr_T = 0 as linenr_T;
    let mut allocated: bool = false_0 != 0;
    let orig_start: pos_T = (*curbuf.get()).b_op_start;
    let orig_end: pos_T = (*curbuf.get()).b_op_end;
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(curwin.get());
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false_0 != 0);
    }
    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
    (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
    if regname == '.' as ::core::ffi::c_int && reg.is_null() {
        let mut non_linewise_vis: bool = VIsual_active.get() as ::core::ffi::c_int != 0
            && VIsual_mode.get() != 'V' as ::core::ffi::c_int;
        let mut command_start_char: ::core::ffi::c_char =
            (if non_linewise_vis as ::core::ffi::c_int != 0 {
                'c' as ::core::ffi::c_int
            } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                'i' as ::core::ffi::c_int
            } else if dir == FORWARD as ::core::ffi::c_int {
                'a' as ::core::ffi::c_int
            } else {
                'i' as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
        if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            do_put(
                '_' as ::core::ffi::c_int,
                ::core::ptr::null_mut::<yankreg_T>(),
                dir,
                1 as ::core::ffi::c_int,
                PUT_LINE as ::core::ffi::c_int,
            );
        }
        if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(command_start_char as ::core::ffi::c_int);
            while count > 0 as ::core::ffi::c_int {
                stuff_inserted(
                    NUL,
                    1 as ::core::ffi::c_int,
                    (count != 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                );
                if count != 1 as ::core::ffi::c_int {
                    stuffReadbuff(b"\n \0".as_ptr() as *const ::core::ffi::c_char);
                    stuffcharReadbuff(Ctrl_U);
                }
                count -= 1;
            }
        } else {
            stuff_inserted(command_start_char as ::core::ffi::c_int, count, false_0);
        }
        if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                stuffReadbuff(b"j0\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                let mut cursor_pos: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                let mut one_past_line: bool = *cursor_pos as ::core::ffi::c_int == NUL;
                let mut eol: bool = false_0 != 0;
                if !one_past_line {
                    eol = *cursor_pos.offset(utfc_ptr2len(cursor_pos) as isize)
                        as ::core::ffi::c_int
                        == NUL;
                }
                let mut ve_allows: bool = cur_ve_flags
                    == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                    || cur_ve_flags
                        == kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint;
                let mut eof: bool = (*curbuf.get()).b_ml.ml_line_count
                    == (*curwin.get()).w_cursor.lnum
                    && one_past_line as ::core::ffi::c_int != 0;
                if ve_allows as ::core::ffi::c_int != 0
                    || !(eol as ::core::ffi::c_int != 0 || eof as ::core::ffi::c_int != 0)
                {
                    stuffcharReadbuff('l' as ::core::ffi::c_int);
                }
            }
        } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            stuffReadbuff(b"g'[\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if command_start_char as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
            if u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return;
            }
        }
        return;
    }
    let mut insert_string: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    if reg.is_null()
        && get_spec_reg(
            regname,
            &raw mut insert_string.data,
            &raw mut allocated,
            true_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        if insert_string.data.is_null() {
            return;
        }
    }
    if (*curbuf.get()).terminal.is_null() {
        if u_save(
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
        ) == FAIL
        {
            return;
        }
    }
    if !insert_string.data.is_null() {
        insert_string.size = strlen(insert_string.data);
        y_type = kMTCharWise;
        if regname == '=' as ::core::ffi::c_int {
            loop {
                y_size = 0 as size_t;
                let mut ptr: *mut ::core::ffi::c_char = insert_string.data;
                let mut ptrlen: size_t = insert_string.size;
                while !ptr.is_null() {
                    if !y_array.is_null() {
                        (*y_array.offset(y_size as isize)).data = ptr;
                    }
                    y_size = y_size.wrapping_add(1);
                    let mut tmp: *mut ::core::ffi::c_char =
                        vim_strchr(ptr, '\n' as ::core::ffi::c_int);
                    if tmp.is_null() {
                        if !y_array.is_null() {
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size =
                                ptrlen;
                        }
                    } else {
                        if !y_array.is_null() {
                            *tmp = NUL as ::core::ffi::c_char;
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size =
                                tmp.offset_from(ptr) as size_t;
                            ptrlen = ptrlen.wrapping_sub(
                                (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                    .size
                                    .wrapping_add(1 as size_t),
                            );
                        }
                        tmp = tmp.offset(1);
                        if *tmp as ::core::ffi::c_int == NUL {
                            y_type = kMTLineWise;
                            break;
                        }
                    }
                    ptr = tmp;
                }
                if !y_array.is_null() {
                    break;
                }
                y_array = xmalloc(y_size.wrapping_mul(::core::mem::size_of::<String_0>()))
                    as *mut String_0;
            }
        } else {
            y_size = 1 as size_t;
            y_array = &raw mut insert_string;
        }
    } else {
        if reg.is_null() {
            reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        }
        y_type = (*reg).y_type;
        y_width = (*reg).y_width as ::core::ffi::c_int;
        y_size = (*reg).y_size;
        y_array = (*reg).y_array;
    }
    '_end: {
        if !(*curbuf.get()).terminal.is_null() {
            terminal_paste(count, y_array, y_size);
        } else {
            split_pos = 0 as colnr_T;
            if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                if flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0 {
                    if u_save_cursor() == FAIL {
                        break '_end;
                    } else {
                        let mut curline: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                        let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        let p_orig: *mut ::core::ffi::c_char = p;
                        let plen: size_t = get_cursor_pos_len() as size_t;
                        if dir == FORWARD as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL {
                            p = p.offset(utfc_ptr2len(p) as isize);
                        }
                        split_pos = p.offset_from(curline) as colnr_T;
                        let mut ptr_0: *mut ::core::ffi::c_char = xmemdupz(
                            p as *const ::core::ffi::c_void,
                            plen.wrapping_sub(p.offset_from(p_orig) as size_t),
                        )
                            as *mut ::core::ffi::c_char;
                        ml_append(
                            (*curwin.get()).w_cursor.lnum,
                            ptr_0,
                            0 as colnr_T,
                            false_0 != 0,
                        );
                        xfree(ptr_0 as *mut ::core::ffi::c_void);
                        ptr_0 = xmemdupz(
                            get_cursor_line_ptr() as *const ::core::ffi::c_void,
                            split_pos as size_t,
                        ) as *mut ::core::ffi::c_char;
                        ml_replace((*curwin.get()).w_cursor.lnum, ptr_0, false_0 != 0);
                        nr_lines += 1;
                        dir = FORWARD as ::core::ffi::c_int;
                        buf_updates_send_changes(
                            curbuf.get(),
                            (*curwin.get()).w_cursor.lnum,
                            1 as int64_t,
                            1 as int64_t,
                        );
                    }
                }
                if flags & PUT_LINE_FORWARD as ::core::ffi::c_int != 0 {
                    (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_end;
                    dir = FORWARD as ::core::ffi::c_int;
                }
                (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
            }
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                y_type = kMTLineWise;
            }
            if y_size == 0 as size_t || y_array.is_null() {
                semsg(
                    gettext(
                        b"E353: Nothing in register %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    if regname == 0 as ::core::ffi::c_int {
                        b"\"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        transchar(regname) as *const ::core::ffi::c_char
                    },
                );
            } else {
                if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    lnum = (*curwin.get()).w_cursor.lnum + y_size as linenr_T + 1 as linenr_T;
                    lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
                        lnum
                    } else {
                        (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
                    };
                    if u_save((*curwin.get()).w_cursor.lnum - 1 as linenr_T, lnum) == FAIL {
                        break '_end;
                    }
                } else if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                    lnum = (*curwin.get()).w_cursor.lnum;
                    if dir == BACKWARD as ::core::ffi::c_int {
                        hasFolding(
                            curwin.get(),
                            lnum,
                            &raw mut lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                    } else {
                        hasFolding(
                            curwin.get(),
                            lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut lnum,
                        );
                    }
                    if dir == FORWARD as ::core::ffi::c_int {
                        lnum += 1;
                    }
                    if (if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0 {
                        u_save(0 as linenr_T, 2 as linenr_T)
                    } else {
                        u_save(lnum - 1 as linenr_T, lnum)
                    }) == FAIL
                    {
                        break '_end;
                    } else {
                        if dir == FORWARD as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.lnum = lnum - 1 as linenr_T;
                        } else {
                            (*curwin.get()).w_cursor.lnum = lnum;
                        }
                        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    }
                } else if u_save_cursor() == FAIL {
                    break '_end;
                }
                if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                    && y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                {
                    if gchar_cursor() == TAB {
                        let mut viscol: ::core::ffi::c_int = getviscol();
                        let mut ts: OptInt = (*curbuf.get()).b_p_ts;
                        if if dir == FORWARD as ::core::ffi::c_int {
                            (tabstop_padding(viscol as colnr_T, ts, (*curbuf.get()).b_p_vts_array)
                                != 1 as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } else {
                            ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } != 0
                        {
                            coladvance_force(viscol as colnr_T);
                        } else {
                            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                    } else if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                        || gchar_cursor() == NUL
                    {
                        coladvance_force(
                            getviscol()
                                + (dir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
                        );
                    }
                }
                lnum = (*curwin.get()).w_cursor.lnum;
                col = (*curwin.get()).w_cursor.col;
                if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    let mut incr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut bd: block_def = block_def {
                        startspaces: 0,
                        endspaces: 0,
                        textlen: 0,
                        textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        textcol: 0,
                        start_vcol: 0,
                        end_vcol: 0,
                        is_short: 0,
                        is_MAX: 0,
                        is_oneChar: 0,
                        pre_whitesp: 0,
                        pre_whitesp_c: 0,
                        end_char_vcols: 0,
                        start_char_vcols: 0,
                    };
                    let mut c: ::core::ffi::c_int = gchar_cursor();
                    let mut endcol2: colnr_T = 0 as colnr_T;
                    if dir == FORWARD as ::core::ffi::c_int && c != NUL {
                        if cur_ve_flags
                            == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            getvcol(
                                curwin.get(),
                                &raw mut (*curwin.get()).w_cursor,
                                &raw mut col,
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut endcol2,
                            );
                        } else {
                            getvcol(
                                curwin.get(),
                                &raw mut (*curwin.get()).w_cursor,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut col,
                            );
                        }
                        (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                        col += 1;
                    } else {
                        getvcol(
                            curwin.get(),
                            &raw mut (*curwin.get()).w_cursor,
                            &raw mut col,
                            ::core::ptr::null_mut::<colnr_T>(),
                            &raw mut endcol2,
                        );
                    }
                    col += (*curwin.get()).w_cursor.coladd;
                    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                            || endcol2 == (*curwin.get()).w_cursor.col)
                    {
                        if dir == FORWARD as ::core::ffi::c_int && c == NUL {
                            col += 1;
                        }
                        if dir != FORWARD as ::core::ffi::c_int
                            && c != NUL
                            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_cursor.col += 1;
                        }
                        if c == TAB {
                            if dir == BACKWARD as ::core::ffi::c_int
                                && (*curwin.get()).w_cursor.col != 0
                            {
                                (*curwin.get()).w_cursor.col -= 1;
                            }
                            if dir == FORWARD as ::core::ffi::c_int
                                && col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == endcol2
                            {
                                (*curwin.get()).w_cursor.col += 1;
                            }
                        }
                    }
                    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    bd.textcol = 0 as ::core::ffi::c_int as colnr_T;
                    let mut i: size_t = 0 as size_t;
                    while i < y_size {
                        let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut shortline: ::core::ffi::c_char = 0;
                        let mut lines_appended: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        bd.startspaces = 0 as ::core::ffi::c_int;
                        bd.endspaces = 0 as ::core::ffi::c_int;
                        vcol = 0 as ::core::ffi::c_int as colnr_T;
                        let mut delcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
                            if ml_append(
                                (*curbuf.get()).b_ml.ml_line_count,
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                1 as colnr_T,
                                false_0 != 0,
                            ) == FAIL
                            {
                                break;
                            }
                            nr_lines += 1;
                            lines_appended = 1 as ::core::ffi::c_int;
                        }
                        let mut oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                        let mut oldlen: colnr_T = get_cursor_line_len();
                        let mut csarg: CharsizeArg = CharsizeArg {
                            win: ::core::ptr::null_mut::<win_T>(),
                            line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            use_tabstop: false,
                            indent_width: 0,
                            virt_row: 0,
                            cur_text_width_left: 0,
                            cur_text_width_right: 0,
                            max_head_vcol: 0,
                            iter: [MarkTreeIter {
                                pos: MTPos { row: 0, col: 0 },
                                lvl: 0,
                                x: ::core::ptr::null_mut::<MTNode>(),
                                i: 0,
                                s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                                intersect_idx: 0,
                                intersect_pos: MTPos { row: 0, col: 0 },
                                intersect_pos_x: MTPos { row: 0, col: 0 },
                            }; 1],
                        };
                        let mut cstype: CSType = init_charsize_arg(
                            &raw mut csarg,
                            curwin.get(),
                            (*curwin.get()).w_cursor.lnum,
                            oldp,
                        );
                        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(oldp);
                        vcol = 0 as ::core::ffi::c_int as colnr_T;
                        while vcol < col && *ci.ptr as ::core::ffi::c_int != NUL {
                            incr = win_charsize(
                                cstype,
                                vcol as ::core::ffi::c_int,
                                ci.ptr,
                                ci.chr.value,
                                &raw mut csarg,
                            )
                            .width;
                            vcol += incr;
                            ci = utfc_next(ci);
                        }
                        let mut ptr_1: *mut ::core::ffi::c_char = ci.ptr;
                        bd.textcol = ptr_1.offset_from(oldp) as colnr_T;
                        shortline = (vcol < col || vcol == col && *ptr_1 == 0) as ::core::ffi::c_int
                            as ::core::ffi::c_char;
                        if vcol < col {
                            bd.startspaces = (col - vcol) as ::core::ffi::c_int;
                        } else if vcol > col {
                            bd.endspaces = (vcol - col) as ::core::ffi::c_int;
                            bd.startspaces = incr - bd.endspaces;
                            bd.textcol -= 1;
                            delcount = 1 as ::core::ffi::c_int;
                            bd.textcol -= utf_head_off(oldp, oldp.offset(bd.textcol as isize));
                            if *oldp.offset(bd.textcol as isize) as ::core::ffi::c_int != TAB {
                                delcount = 0 as ::core::ffi::c_int;
                                bd.endspaces = 0 as ::core::ffi::c_int;
                            }
                        }
                        let yanklen: ::core::ffi::c_int =
                            (*y_array.offset(i as isize)).size as ::core::ffi::c_int;
                        if flags & PUT_BLOCK_INNER as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        {
                            spaces = y_width + 1 as ::core::ffi::c_int;
                            cstype = init_charsize_arg(
                                &raw mut csarg,
                                curwin.get(),
                                0 as linenr_T,
                                (*y_array.offset(i as isize)).data,
                            );
                            ci = utf_ptr2StrCharInfo((*y_array.offset(i as isize)).data);
                            while *ci.ptr as ::core::ffi::c_int != NUL {
                                spaces -= win_charsize(
                                    cstype,
                                    0 as ::core::ffi::c_int,
                                    ci.ptr,
                                    ci.chr.value,
                                    &raw mut csarg,
                                )
                                .width;
                                ci = utfc_next(ci);
                            }
                            spaces = if spaces > 0 as ::core::ffi::c_int {
                                spaces
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        }
                        if yanklen + spaces != 0 as ::core::ffi::c_int
                            && count
                                > (INT_MAX - (bd.startspaces + bd.endspaces)) / (yanklen + spaces)
                        {
                            emsg(gettext(
                                &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
                            ));
                            break;
                        } else {
                            totlen = (count as size_t)
                                .wrapping_mul((yanklen + spaces) as size_t)
                                .wrapping_add(bd.startspaces as size_t)
                                .wrapping_add(bd.endspaces as size_t);
                            let mut newp: *mut ::core::ffi::c_char = xmalloc(
                                totlen
                                    .wrapping_add(oldlen as size_t)
                                    .wrapping_add(1 as size_t),
                            )
                                as *mut ::core::ffi::c_char;
                            ptr_1 = newp;
                            memmove(
                                ptr_1 as *mut ::core::ffi::c_void,
                                oldp as *const ::core::ffi::c_void,
                                bd.textcol as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.textcol as isize);
                            memset(
                                ptr_1 as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                bd.startspaces as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.startspaces as isize);
                            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while j < count {
                                memmove(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    (*y_array.offset(i as isize)).data
                                        as *const ::core::ffi::c_void,
                                    yanklen as size_t,
                                );
                                ptr_1 = ptr_1.offset(yanklen as isize);
                                if (j < count - 1 as ::core::ffi::c_int || shortline == 0)
                                    && spaces > 0 as ::core::ffi::c_int
                                {
                                    memset(
                                        ptr_1 as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        spaces as size_t,
                                    );
                                    ptr_1 = ptr_1.offset(spaces as isize);
                                } else {
                                    totlen = totlen.wrapping_sub(spaces as size_t);
                                }
                                j += 1;
                            }
                            memset(
                                ptr_1 as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                bd.endspaces as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.endspaces as isize);
                            let mut columns: ::core::ffi::c_int = oldlen as ::core::ffi::c_int
                                - bd.textcol as ::core::ffi::c_int
                                - delcount
                                + 1 as ::core::ffi::c_int;
                            '_c2rust_label: {
                                if columns >= 0 as ::core::ffi::c_int {
                                } else {
                                    __assert_fail(
                                        b"columns >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/register.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        1731 as ::core::ffi::c_uint,
                                        b"void do_put(int, yankreg_T *, int, int, int)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            memmove(
                                ptr_1 as *mut ::core::ffi::c_void,
                                oldp.offset(bd.textcol as isize).offset(delcount as isize)
                                    as *const ::core::ffi::c_void,
                                columns as size_t,
                            );
                            ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
                            extmark_splice_cols(
                                curbuf.get(),
                                (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                                bd.textcol,
                                delcount as colnr_T,
                                totlen as colnr_T + lines_appended as colnr_T,
                                kExtmarkUndo,
                            );
                            (*curwin.get()).w_cursor.lnum += 1;
                            if i == 0 as size_t {
                                (*curwin.get()).w_cursor.col += bd.startspaces;
                            }
                            i = i.wrapping_add(1);
                        }
                    }
                    changed_lines(
                        curbuf.get(),
                        lnum,
                        0 as colnr_T,
                        (*curbuf.get()).b_op_start.lnum + y_size as linenr_T - nr_lines,
                        nr_lines,
                        true_0 != 0,
                    );
                    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    (*curbuf.get()).b_op_start.lnum = lnum;
                    (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum - 1 as linenr_T;
                    (*curbuf.get()).b_op_end.col = (if bd.textcol as ::core::ffi::c_int
                        + totlen as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                        > 0 as ::core::ffi::c_int
                    {
                        bd.textcol as ::core::ffi::c_int + totlen as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as colnr_T;
                    (*curbuf.get()).b_op_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                        (*curwin.get()).w_cursor = (*curbuf.get()).b_op_end;
                        (*curwin.get()).w_cursor.col += 1;
                        let mut len: colnr_T = get_cursor_line_len();
                        (*curwin.get()).w_cursor.col = if (*curwin.get()).w_cursor.col < len {
                            (*curwin.get()).w_cursor.col
                        } else {
                            len
                        };
                    } else {
                        (*curwin.get()).w_cursor.lnum = lnum;
                    }
                } else {
                    let yanklen_0: ::core::ffi::c_int =
                        (*y_array.offset(0 as ::core::ffi::c_int as isize)).size
                            as ::core::ffi::c_int;
                    if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                        if dir == FORWARD as ::core::ffi::c_int && gchar_cursor() != NUL {
                            let mut bytelen: ::core::ffi::c_int =
                                utfc_ptr2len(get_cursor_pos_ptr());
                            col += bytelen;
                            if yanklen_0 != 0 {
                                (*curwin.get()).w_cursor.col += bytelen;
                                (*curbuf.get()).b_op_end.col += bytelen;
                            }
                        }
                        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    } else if dir == BACKWARD as ::core::ffi::c_int {
                        lnum -= 1;
                    }
                    let mut new_cursor: pos_T = (*curwin.get()).w_cursor;
                    if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                        && y_size == 1 as size_t
                    {
                        let mut end_lnum: linenr_T = 0 as linenr_T;
                        let mut start_lnum: linenr_T = lnum;
                        let mut first_byte_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if VIsual_active.get() {
                            end_lnum = if (*curbuf.get()).b_visual.vi_end.lnum
                                > (*curbuf.get()).b_visual.vi_start.lnum
                            {
                                (*curbuf.get()).b_visual.vi_end.lnum
                            } else {
                                (*curbuf.get()).b_visual.vi_start.lnum
                            };
                            if end_lnum > start_lnum {
                                let mut pos: pos_T = pos_T {
                                    lnum: lnum,
                                    col: col,
                                    coladd: 0 as colnr_T,
                                };
                                getvcol(
                                    curwin.get(),
                                    &raw mut pos,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    &raw mut vcol,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                            }
                        }
                        if count == 0 as ::core::ffi::c_int || yanklen_0 == 0 as ::core::ffi::c_int
                        {
                            if VIsual_active.get() {
                                lnum = end_lnum;
                            }
                        } else if count > INT_MAX / yanklen_0 {
                            emsg(gettext(
                                &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
                            ));
                        } else {
                            totlen = (count as size_t).wrapping_mul(yanklen_0 as size_t);
                            loop {
                                let mut oldp_0: *mut ::core::ffi::c_char = ml_get(lnum);
                                let mut oldlen_0: colnr_T = ml_get_len(lnum);
                                if lnum > start_lnum {
                                    let mut pos_0: pos_T = pos_T {
                                        lnum: lnum,
                                        col: 0,
                                        coladd: 0,
                                    };
                                    if getvpos(curwin.get(), &raw mut pos_0, vcol) == OK {
                                        col = pos_0.col;
                                    } else {
                                        col = MAXCOL as ::core::ffi::c_int as colnr_T;
                                    }
                                }
                                if VIsual_active.get() as ::core::ffi::c_int != 0 && col > oldlen_0
                                {
                                    lnum += 1;
                                } else {
                                    let mut newp_0: *mut ::core::ffi::c_char = xmalloc(
                                        totlen
                                            .wrapping_add(oldlen_0 as size_t)
                                            .wrapping_add(1 as size_t),
                                    )
                                        as *mut ::core::ffi::c_char;
                                    memmove(
                                        newp_0 as *mut ::core::ffi::c_void,
                                        oldp_0 as *const ::core::ffi::c_void,
                                        col as size_t,
                                    );
                                    let mut ptr_2: *mut ::core::ffi::c_char =
                                        newp_0.offset(col as isize);
                                    let mut i_0: size_t = 0 as size_t;
                                    while i_0 < count as size_t {
                                        memmove(
                                            ptr_2 as *mut ::core::ffi::c_void,
                                            (*y_array.offset(0 as ::core::ffi::c_int as isize)).data
                                                as *const ::core::ffi::c_void,
                                            yanklen_0 as size_t,
                                        );
                                        ptr_2 = ptr_2.offset(yanklen_0 as isize);
                                        i_0 = i_0.wrapping_add(1);
                                    }
                                    memmove(
                                        ptr_2 as *mut ::core::ffi::c_void,
                                        oldp_0.offset(col as isize) as *const ::core::ffi::c_void,
                                        ((oldlen_0 - col) as size_t).wrapping_add(1 as size_t),
                                    );
                                    ml_replace(lnum, newp_0, false_0 != 0);
                                    first_byte_off = utf_head_off(
                                        newp_0,
                                        ptr_2.offset(-(1 as ::core::ffi::c_int as isize)),
                                    );
                                    if lnum == (*curwin.get()).w_cursor.lnum {
                                        changed_cline_bef_curs(curwin.get());
                                        invalidate_botline_win(curwin.get());
                                        (*curwin.get()).w_cursor.col +=
                                            totlen.wrapping_sub(1 as size_t) as colnr_T;
                                    }
                                    changed_bytes(lnum, col);
                                    extmark_splice_cols(
                                        curbuf.get(),
                                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                        col,
                                        0 as colnr_T,
                                        totlen as colnr_T,
                                        kExtmarkUndo,
                                    );
                                    if VIsual_active.get() {
                                        lnum += 1;
                                    }
                                }
                                if !(VIsual_active.get() as ::core::ffi::c_int != 0
                                    && lnum <= end_lnum)
                                {
                                    break;
                                }
                            }
                            if VIsual_active.get() {
                                lnum -= 1;
                            }
                        }
                        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                        (*curbuf.get()).b_op_end.col -= first_byte_off;
                        if totlen != 0
                            && (restart_edit.get() != 0 as ::core::ffi::c_int
                                || flags & PUT_CURSEND as ::core::ffi::c_int != 0)
                        {
                            (*curwin.get()).w_cursor.col += 1;
                        } else {
                            (*curwin.get()).w_cursor.col -= first_byte_off;
                        }
                    } else {
                        let mut new_lnum: linenr_T = new_cursor.lnum;
                        let mut indent: ::core::ffi::c_int = 0;
                        let mut orig_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut indent_diff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut first_indent: bool = true_0 != 0;
                        let mut lendiff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                            orig_indent = get_indent();
                        }
                        let mut cnt: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        '_error: while cnt <= count {
                            let mut i_1: size_t = 0 as size_t;
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                                lnum = new_cursor.lnum;
                                let mut ptr_3: *mut ::core::ffi::c_char =
                                    ml_get(lnum).offset(col as isize);
                                let mut ptrlen_0: size_t =
                                    (ml_get_len(lnum) as size_t).wrapping_sub(col as size_t);
                                totlen = (*y_array
                                    .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                .size;
                                let mut newp_1: *mut ::core::ffi::c_char = xmalloc(
                                    ptrlen_0.wrapping_add(totlen).wrapping_add(1 as size_t),
                                )
                                    as *mut ::core::ffi::c_char;
                                strcpy(
                                    newp_1,
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data,
                                );
                                strcpy(newp_1.offset(totlen as isize), ptr_3);
                                ml_append(lnum, newp_1, 0 as colnr_T, false_0 != 0);
                                new_lnum += 1;
                                xfree(newp_1 as *mut ::core::ffi::c_void);
                                let mut oldp_1: *mut ::core::ffi::c_char = ml_get(lnum);
                                newp_1 = xmalloc(
                                    (col as size_t)
                                        .wrapping_add(yanklen_0 as size_t)
                                        .wrapping_add(1 as size_t),
                                )
                                    as *mut ::core::ffi::c_char;
                                memmove(
                                    newp_1 as *mut ::core::ffi::c_void,
                                    oldp_1 as *const ::core::ffi::c_void,
                                    col as size_t,
                                );
                                memmove(
                                    newp_1.offset(col as isize) as *mut ::core::ffi::c_void,
                                    (*y_array.offset(0 as ::core::ffi::c_int as isize)).data
                                        as *const ::core::ffi::c_void,
                                    (yanklen_0 as size_t).wrapping_add(1 as size_t),
                                );
                                ml_replace(lnum, newp_1, false_0 != 0);
                                (*curwin.get()).w_cursor.lnum = lnum;
                                i_1 = 1 as size_t;
                            }
                            while i_1 < y_size {
                                if y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int
                                    || i_1 < y_size.wrapping_sub(1 as size_t)
                                {
                                    if ml_append(
                                        lnum,
                                        (*y_array.offset(i_1 as isize)).data,
                                        0 as colnr_T,
                                        false_0 != 0,
                                    ) == FAIL
                                    {
                                        break '_error;
                                    }
                                    new_lnum += 1;
                                }
                                lnum += 1;
                                nr_lines += 1;
                                if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                                    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
                                    (*curwin.get()).w_cursor.lnum = lnum;
                                    let mut ptr_4: *mut ::core::ffi::c_char = ml_get(lnum);
                                    if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                        lendiff = ml_get_len(lnum) as ::core::ffi::c_int;
                                    }
                                    if *ptr_4 as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                                        && preprocs_left() as ::core::ffi::c_int != 0
                                    {
                                        indent = 0 as ::core::ffi::c_int;
                                    } else if *ptr_4 as ::core::ffi::c_int == NUL {
                                        indent = 0 as ::core::ffi::c_int;
                                    } else if first_indent {
                                        indent_diff = orig_indent - get_indent();
                                        indent = orig_indent;
                                        first_indent = false_0 != 0;
                                    } else {
                                        indent = get_indent() + indent_diff;
                                        if indent < 0 as ::core::ffi::c_int {
                                            indent = 0 as ::core::ffi::c_int;
                                        }
                                    }
                                    set_indent(indent, SIN_NOMARK as ::core::ffi::c_int);
                                    (*curwin.get()).w_cursor = old_pos;
                                    if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                        lendiff -= ml_get_len(lnum) as ::core::ffi::c_int;
                                    }
                                }
                                i_1 = i_1.wrapping_add(1);
                            }
                            let mut totsize: bcount_t = 0 as bcount_t;
                            let mut lastsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                                || y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                                    && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                            {
                                i_1 = 0 as size_t;
                                while i_1 < y_size.wrapping_sub(1 as size_t) {
                                    totsize += (*y_array.offset(i_1 as isize)).size as bcount_t
                                        + 1 as bcount_t;
                                    i_1 = i_1.wrapping_add(1);
                                }
                                lastsize = (*y_array
                                    .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                .size
                                    as ::core::ffi::c_int;
                                totsize += lastsize as bcount_t;
                            }
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                                extmark_splice(
                                    curbuf.get(),
                                    new_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    col,
                                    0 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    0 as bcount_t,
                                    y_size as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    lastsize as colnr_T,
                                    totsize,
                                    kExtmarkUndo,
                                );
                            } else if y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                                && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                            {
                                extmark_splice(
                                    curbuf.get(),
                                    new_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    split_pos,
                                    0 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    0 as bcount_t,
                                    y_size as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    totsize + 2 as bcount_t,
                                    kExtmarkUndo,
                                );
                            }
                            if cnt == 1 as ::core::ffi::c_int {
                                new_lnum = lnum;
                            }
                            cnt += 1;
                        }
                        if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                            (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*curbuf.get()).b_op_start.lnum += 1;
                            }
                        }
                        let mut kind: ExtmarkOp = (if y_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int
                            && flags & PUT_LINE_SPLIT as ::core::ffi::c_int == 0
                        {
                            kExtmarkUndo as ::core::ffi::c_int
                        } else {
                            kExtmarkNOOP as ::core::ffi::c_int
                        }) as ExtmarkOp;
                        mark_adjust(
                            (*curbuf.get()).b_op_start.lnum
                                + (y_type as ::core::ffi::c_int
                                    == kMTCharWise as ::core::ffi::c_int)
                                    as ::core::ffi::c_int,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            nr_lines,
                            0 as linenr_T,
                            kind,
                        );
                        if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                            changed_lines(
                                curbuf.get(),
                                (*curwin.get()).w_cursor.lnum,
                                col,
                                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                                nr_lines,
                                true_0 != 0,
                            );
                        } else {
                            changed_lines(
                                curbuf.get(),
                                (*curbuf.get()).b_op_start.lnum,
                                0 as colnr_T,
                                (*curbuf.get()).b_op_start.lnum,
                                nr_lines,
                                true_0 != 0,
                            );
                        }
                        (*curbuf.get()).b_op_end.lnum = new_lnum;
                        col = (if 0 as ::core::ffi::c_int
                            > (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                as ::core::ffi::c_int
                                - lendiff
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                as ::core::ffi::c_int
                                - lendiff
                        }) as colnr_T;
                        if col > 1 as ::core::ffi::c_int {
                            (*curbuf.get()).b_op_end.col =
                                (col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
                            if (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                > 0 as size_t
                            {
                                (*curbuf.get()).b_op_end.col -= utf_head_off(
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data,
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data
                                        .offset(
                                            (*y_array
                                                .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                            .size
                                                as isize,
                                        )
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                );
                            }
                        } else {
                            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if flags & PUT_CURSLINE as ::core::ffi::c_int != 0 {
                            (*curwin.get()).w_cursor.lnum = lnum;
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                        } else if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                            if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                                if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                                    (*curwin.get()).w_cursor.lnum =
                                        (*curbuf.get()).b_ml.ml_line_count;
                                } else {
                                    (*curwin.get()).w_cursor.lnum = lnum + 1 as linenr_T;
                                }
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                (*curwin.get()).w_cursor.lnum = new_lnum;
                                (*curwin.get()).w_cursor.col = col;
                                (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                                if col > 1 as ::core::ffi::c_int {
                                    (*curbuf.get()).b_op_end.col = (col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as colnr_T;
                                }
                            }
                        } else if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*curwin.get()).w_cursor.lnum += 1;
                            }
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                        } else {
                            (*curwin.get()).w_cursor = new_cursor;
                        }
                    }
                }
                msgmore(nr_lines as ::core::ffi::c_int);
                (*curwin.get()).w_set_curswant = true_0;
                len_0 = get_cursor_line_len();
                if (*curwin.get()).w_cursor.col > len_0 {
                    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
                        (*curwin.get()).w_cursor.coladd =
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - len_0) as colnr_T;
                    }
                    (*curwin.get()).w_cursor.col = len_0 as colnr_T;
                }
            }
        }
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*curbuf.get()).b_op_start = orig_start;
        (*curbuf.get()).b_op_end = orig_end;
    }
    if allocated {
        xfree(insert_string.data as *mut ::core::ffi::c_void);
    }
    if regname == '=' as ::core::ffi::c_int {
        xfree(y_array as *mut ::core::ffi::c_void);
    }
    if (*curbuf.get()).terminal.is_null() {
        VIsual_active.set(false_0 != 0);
    }
    adjust_cursor_eol();
}
unsafe extern "C" fn dis_msg(mut p: *const ::core::ffi::c_char, mut skip_esc: bool) {
    let mut n: ::core::ffi::c_int = Columns.get() - 6 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL
        && !(*p as ::core::ffi::c_int == ESC
            && skip_esc as ::core::ffi::c_int != 0
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        && {
            n -= ptr2cells(p);
            n >= 0 as ::core::ffi::c_int
        }
    {
        let mut l: ::core::ffi::c_int = 0;
        l = utfc_ptr2len(p);
        if l > 1 as ::core::ffi::c_int {
            msg_outtrans_len(p, l, 0 as ::core::ffi::c_int, false_0 != 0);
            p = p.offset(l as isize);
        } else {
            let c2rust_fresh4 = p;
            p = p.offset(1);
            msg_outtrans_len(
                c2rust_fresh4,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
    }
    os_breakcheck();
}
#[no_mangle]
pub unsafe extern "C" fn ex_display(mut eap: *mut exarg_T) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut yb: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut type_0: ::core::ffi::c_int = 0;
    if !arg.is_null() && *arg as ::core::ffi::c_int == NUL {
        arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut hl_id: ::core::ffi::c_int = HLF_8 as ::core::ffi::c_int;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_ext_skip_flush.set(true_0 != 0);
    msg_puts_title(gettext(
        b"\nType Name Content\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut i: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    while i < NUM_REGISTERS as ::core::ffi::c_int && !got_int.get() {
        let mut name: ::core::ffi::c_int = get_register_name(i);
        if !(!arg.is_null() && vim_strchr(arg, name).is_null()) {
            match get_reg_type(name, ::core::ptr::null_mut::<colnr_T>()) as ::core::ffi::c_int {
                1 => {
                    type_0 = 'l' as ::core::ffi::c_int;
                }
                0 => {
                    type_0 = 'c' as ::core::ffi::c_int;
                }
                _ => {
                    type_0 = 'b' as ::core::ffi::c_int;
                }
            }
            if i == -1 as ::core::ffi::c_int {
                if !(*y_previous.ptr()).is_null() {
                    yb = y_previous.get();
                } else {
                    yb = (y_regs.ptr() as *mut yankreg_T).offset(0 as ::core::ffi::c_int as isize);
                }
            } else {
                yb = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
            }
            clipboard::get_clipboard(name, &mut yb, true);
            if !(name == mb_tolower(redir_reg.get())
                || redir_reg.get() == '"' as ::core::ffi::c_int && yb == y_previous.get())
            {
                if !(*yb).y_array.is_null() {
                    let mut do_show: bool = false_0 != 0;
                    let mut j: size_t = 0 as size_t;
                    while !do_show && j < (*yb).y_size {
                        do_show = !message_filtered((*(*yb).y_array.offset(j as isize)).data);
                        j = j.wrapping_add(1);
                    }
                    if do_show as ::core::ffi::c_int != 0 || (*yb).y_size == 0 as size_t {
                        msg_putchar('\n' as ::core::ffi::c_int);
                        msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        msg_putchar(type_0);
                        msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        msg_putchar('"' as ::core::ffi::c_int);
                        msg_putchar(name);
                        msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
                        let mut n: ::core::ffi::c_int = Columns.get() - 11 as ::core::ffi::c_int;
                        let mut j_0: size_t = 0 as size_t;
                        while j_0 < (*yb).y_size && n > 1 as ::core::ffi::c_int {
                            if j_0 != 0 {
                                msg_puts_hl(
                                    b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                n -= 2 as ::core::ffi::c_int;
                            }
                            p = (*(*yb).y_array.offset(j_0 as isize)).data;
                            while *p as ::core::ffi::c_int != NUL && {
                                n -= ptr2cells(p);
                                n >= 0 as ::core::ffi::c_int
                            } {
                                let mut clen: ::core::ffi::c_int = utfc_ptr2len(p);
                                msg_outtrans_len(p, clen, 0 as ::core::ffi::c_int, false_0 != 0);
                                p = p.offset((clen - 1 as ::core::ffi::c_int) as isize);
                                p = p.offset(1);
                            }
                            j_0 = j_0.wrapping_add(1);
                        }
                        if n > 1 as ::core::ffi::c_int
                            && (*yb).y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                        {
                            msg_puts_hl(
                                b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                        }
                    }
                    os_breakcheck();
                }
            }
        }
        i += 1;
    }
    let mut insert: String_0 = get_last_insert();
    p = insert.data;
    if !p.is_null()
        && (arg.is_null() || !vim_strchr(arg, '.' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(p)
    {
        msg_puts(b"\n  c  \".   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(p, true_0 != 0);
    }
    if !(*last_cmdline.ptr()).is_null()
        && (arg.is_null() || !vim_strchr(arg, ':' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(last_cmdline.get())
    {
        msg_puts(b"\n  c  \":   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(last_cmdline.get(), false_0 != 0);
    }
    if !(*curbuf.get()).b_fname.is_null()
        && (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered((*curbuf.get()).b_fname)
    {
        msg_puts(b"\n  c  \"%   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg((*curbuf.get()).b_fname, false_0 != 0);
    }
    if (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null()) && !got_int.get() {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dummy: linenr_T = 0;
        if buflist_name_nr(0 as ::core::ffi::c_int, &raw mut fname, &raw mut dummy) != FAIL
            && !message_filtered(fname)
        {
            msg_puts(b"\n  c  \"#   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(fname, false_0 != 0);
        }
    }
    if !last_search_pat().is_null()
        && (arg.is_null() || !vim_strchr(arg, '/' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(last_search_pat())
    {
        msg_puts(b"\n  c  \"/   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(last_search_pat(), false_0 != 0);
    }
    if !(*expr_line.ptr()).is_null()
        && (arg.is_null() || !vim_strchr(arg, '=' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(expr_line.get())
    {
        msg_puts(b"\n  c  \"=   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(expr_line.get(), false_0 != 0);
    }
    msg_ext_skip_flush.set(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn get_reg_type(
    mut regname: ::core::ffi::c_int,
    mut reg_width: *mut colnr_T,
) -> MotionType {
    's_19: {
        'c_46756: {
            'c_46754: {
                'c_46752: {
                    'c_46750: {
                        'c_46748: {
                            'c_46746: {
                                'c_46744: {
                                    'c_46742: {
                                        match regname {
                                            35 => {}
                                            61 => {}
                                            58 => {
                                                break 'c_46742;
                                            }
                                            47 => {
                                                break 'c_46744;
                                            }
                                            46 => {
                                                break 'c_46746;
                                            }
                                            Ctrl_F => {
                                                break 'c_46748;
                                            }
                                            Ctrl_P => {
                                                break 'c_46750;
                                            }
                                            Ctrl_W => {
                                                break 'c_46752;
                                            }
                                            Ctrl_A => {
                                                break 'c_46754;
                                            }
                                            37 | 95 => {
                                                break 'c_46756;
                                            }
                                            _ => {
                                                break 's_19;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return kMTCharWise;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return kMTUnknown;
    }
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    if !(*reg).y_array.is_null() {
        if !reg_width.is_null()
            && (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
        {
            *reg_width = (*reg).y_width;
        }
        return (*reg).y_type;
    }
    return kMTUnknown;
}
unsafe extern "C" fn get_reg_wrap_one_line(
    mut s: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if flags & kGRegList as ::core::ffi::c_int == 0 {
        return s as *mut ::core::ffi::c_void;
    }
    let list: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_allocated_string(list, s);
    return list as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn get_reg_contents(
    mut regname: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if regname == '=' as ::core::ffi::c_int {
        if flags & kGRegNoExpr as ::core::ffi::c_int != 0 {
            return NULL_0;
        }
        if flags & kGRegExprSrc as ::core::ffi::c_int != 0 {
            return get_reg_wrap_one_line(get_expr_line_src(), flags);
        }
        return get_reg_wrap_one_line(get_expr_line(), flags);
    }
    if regname == '@' as ::core::ffi::c_int {
        regname = '"' as ::core::ffi::c_int;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return NULL_0;
    }
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut allocated: bool = false;
    if get_spec_reg(regname, &raw mut retval, &raw mut allocated, false_0 != 0) {
        if retval.is_null() {
            return NULL_0;
        }
        if allocated {
            return get_reg_wrap_one_line(retval, flags);
        }
        return get_reg_wrap_one_line(xstrdup(retval), flags);
    }
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PUT as ::core::ffi::c_int);
    if (*reg).y_array.is_null() {
        return NULL_0;
    }
    if flags & kGRegList as ::core::ffi::c_int != 0 {
        let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            tv_list_append_string(
                list,
                (*(*reg).y_array.offset(i as isize)).data,
                (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
            );
            i = i.wrapping_add(1);
        }
        return list as *mut ::core::ffi::c_void;
    }
    let mut len: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*reg).y_size {
        len = len.wrapping_add((*(*reg).y_array.offset(i_0 as isize)).size);
        if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || i_0 < (*reg).y_size.wrapping_sub(1 as size_t)
        {
            len = len.wrapping_add(1);
        }
        i_0 = i_0.wrapping_add(1);
    }
    retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    len = 0 as size_t;
    let mut i_1: size_t = 0 as size_t;
    while i_1 < (*reg).y_size {
        strcpy(
            retval.offset(len as isize),
            (*(*reg).y_array.offset(i_1 as isize)).data,
        );
        len = len.wrapping_add((*(*reg).y_array.offset(i_1 as isize)).size);
        if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || i_1 < (*reg).y_size.wrapping_sub(1 as size_t)
        {
            let c2rust_fresh5 = len;
            len = len.wrapping_add(1);
            *retval.offset(c2rust_fresh5 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_1 = i_1.wrapping_add(1);
    }
    *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
    return retval as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn init_write_reg(
    mut name: ::core::ffi::c_int,
    mut old_y_previous: *mut *mut yankreg_T,
    mut must_append: bool,
) -> *mut yankreg_T {
    if !valid_yank_reg(name, true_0 != 0) {
        emsg_invreg(name);
        return ::core::ptr::null_mut::<yankreg_T>();
    }
    *old_y_previous = y_previous.get();
    let mut reg: *mut yankreg_T = get_yank_register(name, YREG_YANK as ::core::ffi::c_int);
    if !is_append_register(name) && !must_append {
        free_register(reg);
    }
    return reg;
}
unsafe extern "C" fn str_to_reg(
    mut y_ptr: *mut yankreg_T,
    mut yank_type: MotionType,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    mut blocklen: colnr_T,
    mut str_list: bool,
) {
    if (*y_ptr).y_array.is_null() {
        (*y_ptr).y_size = 0 as size_t;
    }
    if yank_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
        yank_type = (if str_list as ::core::ffi::c_int != 0
            || len > 0 as size_t
                && (*str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == NL
                    || *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == CAR)
        {
            kMTLineWise as ::core::ffi::c_int
        } else {
            kMTCharWise as ::core::ffi::c_int
        }) as MotionType;
    }
    let mut newlines: size_t = 0 as size_t;
    let mut extraline: bool = false_0 != 0;
    let mut append: bool = false_0 != 0;
    if str_list {
        let mut ss: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
        while !(*ss).is_null() {
            newlines = newlines.wrapping_add(1);
            ss = ss.offset(1);
        }
    } else {
        newlines = memcnt(
            str as *const ::core::ffi::c_void,
            '\n' as ::core::ffi::c_char,
            len,
        );
        if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            || len == 0 as size_t
            || *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                != '\n' as ::core::ffi::c_int
        {
            extraline = true;
            newlines = newlines.wrapping_add(1);
        }
        if (*y_ptr).y_size > 0 as size_t
            && (*y_ptr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
        {
            append = true_0 != 0;
            newlines = newlines.wrapping_sub(1);
        }
    }
    if (*y_ptr).y_size.wrapping_add(newlines) == 0 as size_t {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*y_ptr).y_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        return;
    }
    let mut pp: *mut String_0 = xrealloc(
        (*y_ptr).y_array as *mut ::core::ffi::c_void,
        (*y_ptr)
            .y_size
            .wrapping_add(newlines)
            .wrapping_mul(::core::mem::size_of::<String_0>()),
    ) as *mut String_0;
    (*y_ptr).y_array = pp;
    let mut lnum: size_t = (*y_ptr).y_size;
    let mut maxlen: size_t = 0 as size_t;
    if str_list {
        let mut ss_0: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
        while !(*ss_0).is_null() {
            *pp.offset(lnum as isize) = cstr_to_string(*ss_0);
            if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                let mut charlen: size_t = mb_string2cells(*ss_0);
                maxlen = if maxlen > charlen { maxlen } else { charlen };
            }
            ss_0 = ss_0.offset(1);
            lnum = lnum.wrapping_add(1);
        }
    } else {
        let mut line_len: size_t = 0;
        let mut start: *const ::core::ffi::c_char = str;
        let mut end: *const ::core::ffi::c_char = str.offset(len as isize);
        while start < end.offset(extraline as ::core::ffi::c_int as isize) {
            let mut charlen_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut line_end: *const ::core::ffi::c_char = start;
            while line_end < end {
                if *line_end as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                    break;
                }
                if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    charlen_0 += utf_ptr2cells_len(
                        line_end,
                        end.offset_from(line_end) as ::core::ffi::c_int,
                    );
                }
                if *line_end as ::core::ffi::c_int == NUL {
                    line_end = line_end.offset(1);
                } else {
                    line_end = line_end.offset(utf_ptr2len_len(
                        line_end,
                        end.offset_from(line_end) as ::core::ffi::c_int,
                    ) as isize);
                }
            }
            '_c2rust_label: {
                if line_end.offset_from(start) >= 0 as isize {
                } else {
                    __assert_fail(
                        b"line_end - start >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/register.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        2491 as ::core::ffi::c_uint,
                        b"void str_to_reg(yankreg_T *, MotionType, const char *, size_t, colnr_T, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            line_len = line_end.offset_from(start) as size_t;
            maxlen = if maxlen > charlen_0 as size_t {
                maxlen
            } else {
                charlen_0 as size_t
            };
            let mut extra: size_t = if append as ::core::ffi::c_int != 0 {
                lnum = lnum.wrapping_sub(1);
                (*pp.offset(lnum as isize)).size
            } else {
                0 as size_t
            };
            let mut s: *mut ::core::ffi::c_char =
                xmallocz(line_len.wrapping_add(extra)) as *mut ::core::ffi::c_char;
            if extra > 0 as size_t {
                memcpy(
                    s as *mut ::core::ffi::c_void,
                    (*pp.offset(lnum as isize)).data as *const ::core::ffi::c_void,
                    extra,
                );
            }
            if line_len > 0 as size_t {
                memcpy(
                    s.offset(extra as isize) as *mut ::core::ffi::c_void,
                    start as *const ::core::ffi::c_void,
                    line_len,
                );
            }
            let mut s_len: size_t = extra.wrapping_add(line_len);
            if append {
                xfree((*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void);
                append = false_0 != 0;
            }
            *pp.offset(lnum as isize) = String_0 {
                data: s,
                size: s_len,
            };
            memchrsub(
                (*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                '\n' as ::core::ffi::c_char,
                s_len,
            );
            start = start.offset(line_len.wrapping_add(1 as size_t) as isize);
            lnum = lnum.wrapping_add(1);
        }
    }
    (*y_ptr).y_type = yank_type;
    (*y_ptr).y_size = lnum;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*y_ptr).additional_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    (*y_ptr).timestamp = os_time();
    if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        (*y_ptr).y_width = (if blocklen == -1 as ::core::ffi::c_int {
            maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            blocklen as ::core::ffi::c_int
        }) as colnr_T;
    } else {
        (*y_ptr).y_width = 0 as ::core::ffi::c_int as colnr_T;
    };
}
unsafe extern "C" fn finish_write_reg(
    mut name: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut old_y_previous: *mut yankreg_T,
) {
    clipboard::set_clipboard(name, reg);
    if name != '"' as ::core::ffi::c_int {
        y_previous.set(old_y_previous);
    }
}
#[no_mangle]
pub unsafe extern "C" fn write_reg_contents(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: ::core::ffi::c_int,
) {
    write_reg_contents_ex(name, str, len, must_append != 0, kMTUnknown, 0 as colnr_T);
}
#[no_mangle]
pub unsafe extern "C" fn write_reg_contents_lst(
    mut name: ::core::ffi::c_int,
    mut strings: *mut *mut ::core::ffi::c_char,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    if name == '/' as ::core::ffi::c_int || name == '=' as ::core::ffi::c_int {
        let mut s: *mut ::core::ffi::c_char = *strings.offset(0 as ::core::ffi::c_int as isize);
        if (*strings.offset(0 as ::core::ffi::c_int as isize)).is_null() {
            s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if !(*strings.offset(1 as ::core::ffi::c_int as isize)).is_null() {
            emsg(gettext(
                (e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines.ptr()
                    as *const _) as *const ::core::ffi::c_char,
            ));
            return;
        }
        write_reg_contents_ex(name, s, -1 as ssize_t, must_append, yank_type, block_len);
        return;
    }
    if name == '_' as ::core::ffi::c_int {
        return;
    }
    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    reg = init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }
    str_to_reg(
        reg,
        yank_type,
        strings as *mut ::core::ffi::c_char,
        strlen(strings as *mut ::core::ffi::c_char),
        block_len,
        true_0 != 0,
    );
    finish_write_reg(name, reg, old_y_previous);
}
#[no_mangle]
pub unsafe extern "C" fn write_reg_contents_ex(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    if len < 0 as ssize_t {
        len = strlen(str) as ssize_t;
    }
    if name == '/' as ::core::ffi::c_int {
        set_last_search_pat(str, RE_SEARCH as ::core::ffi::c_int, true_0, true_0 != 0);
        return;
    }
    if name == '#' as ::core::ffi::c_int {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if ascii_isdigit(*str as ::core::ffi::c_int) {
            let mut num: ::core::ffi::c_int = atoi(str);
            buf = buflist_findnr(num);
            if buf.is_null() {
                semsg(
                    gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                    num as int64_t,
                );
            }
        } else {
            buf = buflist_findnr(buflist_findpat(
                str,
                str.offset(len as isize),
                true_0 != 0,
                false_0 != 0,
                false_0 != 0,
            ));
        }
        if buf.is_null() {
            return;
        }
        (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
        return;
    }
    if name == '=' as ::core::ffi::c_int {
        let mut offset: size_t = 0 as size_t;
        let mut totlen: size_t = len as size_t;
        if must_append as ::core::ffi::c_int != 0 && !(*expr_line.ptr()).is_null() {
            let mut exprlen: size_t = strlen(expr_line.get());
            totlen = totlen.wrapping_add(exprlen);
            offset = exprlen;
        }
        expr_line.set(xrealloc(
            expr_line.get() as *mut ::core::ffi::c_void,
            totlen.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char);
        memcpy(
            (*expr_line.ptr()).offset(offset as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
        *(*expr_line.ptr()).offset(totlen as isize) = NUL as ::core::ffi::c_char;
        return;
    }
    if name == '_' as ::core::ffi::c_int {
        return;
    }
    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    reg = init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }
    str_to_reg(reg, yank_type, str, len as size_t, block_len, false_0 != 0);
    finish_write_reg(name, reg, old_y_previous);
}
#[no_mangle]
pub unsafe extern "C" fn prepare_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut regtype: String_0,
    mut _lines: size_t,
) -> bool {
    let mut type_0: ::core::ffi::c_char = (if !regtype.data.is_null() {
        *regtype.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        NUL
    }) as ::core::ffi::c_char;
    match type_0 as ::core::ffi::c_int {
        0 => {
            (*reg).y_type = kMTUnknown;
        }
        118 | 99 => {
            (*reg).y_type = kMTCharWise;
        }
        86 | 108 => {
            (*reg).y_type = kMTLineWise;
        }
        98 | Ctrl_V => {
            (*reg).y_type = kMTBlockWise;
        }
        _ => return false_0 != 0,
    }
    (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
    if regtype.size > 1 as size_t {
        if (*reg).y_type as ::core::ffi::c_int != kMTBlockWise as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if !ascii_isdigit(
            *regtype.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        ) {
            return false_0 != 0;
        }
        let mut p: *const ::core::ffi::c_char =
            regtype.data.offset(1 as ::core::ffi::c_int as isize);
        (*reg).y_width = (getdigits_int(
            &raw mut p as *mut *mut ::core::ffi::c_char,
            false_0 != 0,
            1 as ::core::ffi::c_int,
        ) - 1 as ::core::ffi::c_int) as colnr_T;
        if regtype.size > p.offset_from(regtype.data) as size_t {
            return false_0 != 0;
        }
    }
    (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*reg).timestamp = 0 as Timestamp;
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn finish_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut clipboard_adjust: bool,
) {
    if (*reg).y_size > 0 as size_t
        && (*(*reg)
            .y_array
            .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize))
        .size
            == 0 as size_t
    {
        if (*reg).y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int {
            if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int
                || clipboard_adjust as ::core::ffi::c_int != 0
            {
                (*reg).y_size = (*reg).y_size.wrapping_sub(1);
            }
            if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
                (*reg).y_type = kMTLineWise;
            }
        }
    } else if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
        (*reg).y_type = kMTCharWise;
    }
    update_yankreg_width(reg);
}
#[inline]
unsafe extern "C" fn is_literal_register(regname: ::core::ffi::c_int) -> bool {
    return regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
        || (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0);
}
#[inline]
unsafe extern "C" fn op_reg_index(regname: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if ascii_isdigit(regname) {
        return regname - '0' as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname == '-' as ::core::ffi::c_int {
        return DELETION_REGISTER as ::core::ffi::c_int;
    } else if regname == '*' as ::core::ffi::c_int {
        return STAR_REGISTER as ::core::ffi::c_int;
    } else if regname == '+' as ::core::ffi::c_int {
        return PLUS_REGISTER as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}
#[inline]
unsafe extern "C" fn is_append_register(mut regname: ::core::ffi::c_int) -> bool {
    return regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn get_register_name(mut num: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if num == -1 as ::core::ffi::c_int {
        return '"' as ::core::ffi::c_int;
    } else if num < 10 as ::core::ffi::c_int {
        return num + '0' as ::core::ffi::c_int;
    } else if num == DELETION_REGISTER as ::core::ffi::c_int {
        return '-' as ::core::ffi::c_int;
    } else if num == STAR_REGISTER as ::core::ffi::c_int {
        return '*' as ::core::ffi::c_int;
    } else if num == PLUS_REGISTER as ::core::ffi::c_int {
        return '+' as ::core::ffi::c_int;
    } else {
        return num + 'a' as ::core::ffi::c_int - 10 as ::core::ffi::c_int;
    };
}
#[inline]
unsafe extern "C" fn reg_empty(reg: *const yankreg_T) -> bool {
    return (*reg).y_array.is_null()
        || (*reg).y_size == 0 as size_t
        || (*reg).y_size == 1 as size_t
            && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size == 0 as size_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
