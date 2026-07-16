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
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn toupper(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn api_free_array(value: Array);
    fn is_aucmd_win(win: *mut win_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut msg_loclist: *mut ::core::ffi::c_char;
    static mut msg_qflist: *mut ::core::ffi::c_char;
    fn calc_percentage(part: int64_t, whole: int64_t) -> ::core::ffi::c_int;
    fn col_print(
        buf: *mut ::core::ffi::c_char,
        buflen: size_t,
        col: ::core::ffi::c_int,
        vcol: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn get_rel_pos(
        wp: *mut win_T,
        buf: *mut ::core::ffi::c_char,
        buflen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn append_arg_number(
        wp: *mut win_T,
        buf: *mut ::core::ffi::c_char,
        buflen: size_t,
    ) -> ::core::ffi::c_int;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn buf_spname(buf: *mut buf_T) -> *mut ::core::ffi::c_char;
    static mut p_ch: OptInt;
    static mut p_ru: ::core::ffi::c_int;
    static mut p_ruf: *mut ::core::ffi::c_char;
    static mut p_stl: *mut ::core::ffi::c_char;
    static mut p_wbr: *mut ::core::ffi::c_char;
    static mut p_sc: ::core::ffi::c_int;
    static mut p_sloc: *mut ::core::ffi::c_char;
    static mut p_tal: *mut ::core::ffi::c_char;
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
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn trans_characters(buf: *mut ::core::ffi::c_char, bufsize: ::core::ffi::c_int);
    fn transstr_buf(
        s: *const ::core::ffi::c_char,
        slen: ssize_t,
        buf: *mut ::core::ffi::c_char,
        buflen: size_t,
        untab: bool,
    ) -> size_t;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strnsize(s: *const ::core::ffi::c_char, len: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn get_keymap_str(
        wp: *mut win_T,
        fmt: *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn use_cursor_line_highlight(wp: *mut win_T, lnum: linenr_T) -> bool;
    fn fill_foldcolumn(
        wp: *mut win_T,
        foldinfo: foldinfo_T,
        lnum: linenr_T,
        attr: ::core::ffi::c_int,
        fdc: ::core::ffi::c_int,
        is_virt: bool,
        wlv_off: *mut ::core::ffi::c_int,
        out_vcol: *mut colnr_T,
        out_buffer: *mut schar_T,
    );
    fn redrawing() -> bool;
    fn compute_foldcolumn(wp: *mut win_T, col: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static mut updating_screen: bool;
    static mut redraw_not_allowed: bool;
    fn eval_to_string_safe(
        arg: *mut ::core::ffi::c_char,
        use_sandbox: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    fn set_internal_string_var(name: *const ::core::ffi::c_char, value: *mut ::core::ffi::c_char);
    fn do_unlet(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_var(name: *const ::core::ffi::c_char, name_len: size_t, tv: *mut typval_T, copy: bool);
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut edit_submode: *mut ::core::ffi::c_char;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut did_emsg: ::core::ffi::c_int;
    static mut t_colors: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut topframe: *mut frame_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut redraw_tabline: bool;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut ru_col: ::core::ffi::c_int;
    static mut VIsual_active: bool;
    static mut State: ::core::ffi::c_int;
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut KeyTyped: bool;
    static mut wild_menu_showing: ::core::ffi::c_int;
    static mut default_grid: ScreenGrid;
    static mut default_gridview: GridView;
    fn grid_adjust(
        grid: *mut GridView,
        row_off: *mut ::core::ffi::c_int,
        col_off: *mut ::core::ffi::c_int,
    ) -> *mut ScreenGrid;
    fn schar_get(buf_out: *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn schar_get_adv(buf_out: *mut *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn schar_len(sc: schar_T) -> size_t;
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn screengrid_line_start(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
    );
    fn grid_line_put_schar(col: ::core::ffi::c_int, schar: schar_T, attr: ::core::ffi::c_int);
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_fill(
        start_col: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        sc: schar_T,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_flush();
    static mut highlight_user: [::core::ffi::c_int; 9];
    static mut highlight_stlnc: [::core::ffi::c_int; 9];
    static mut ns_hl_fast: NS;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn syn_name2id_len(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_find_line_or_offset(
        buf: *mut buf_T,
        lnum: linenr_T,
        offp: *mut ::core::ffi::c_int,
        no_ff: bool,
    ) -> ::core::ffi::c_int;
    static mut msg_grid_adj: GridView;
    fn msg_clr_eos();
    static mut showcmd_buf: [::core::ffi::c_char; 41];
    fn get_option_default(opt_idx: OptIndex, opt_flags: ::core::ffi::c_int) -> OptVal;
    fn was_set_insecurely(
        wp: *mut win_T,
        opt_idx: OptIndex,
        opt_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn find_option(name: *const ::core::ffi::c_char) -> OptIndex;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn shorten_dir(str: *mut ::core::ffi::c_char);
    fn getvvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    static mut tab_page_click_defs: *mut StlClickDefinition;
    static mut tab_page_click_defs_size: size_t;
    fn describe_sign_text(buf: *mut ::core::ffi::c_char, sign_text: *mut schar_T) -> size_t;
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_tabline_update(
        current: Tabpage,
        tabs: Array,
        current_buffer: Buffer,
        buffers: Array,
    );
    fn ui_call_msg_ruler(content: Array);
    fn tabline_height() -> ::core::ffi::c_int;
    fn global_stl_height() -> ::core::ffi::c_int;
    fn lastwin_nofloating(tp: *mut tabpage_T) -> *mut win_T;
    fn bufIsChanged(buf: *mut buf_T) -> bool;
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type ssize_t = isize;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
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
pub type NS = handle_T;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type OptInt = int64_t;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed_0 = 2;
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
pub type Buffer = handle_T;
pub type Tabpage = handle_T;
pub type hlf_T = ::core::ffi::c_uint;
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
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_14 = 9;
pub type StlFlag = ::core::ffi::c_uint;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickRecord {
    pub def: StlClickDefinition,
    pub start: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stl_hlrec {
    pub start: *mut ::core::ffi::c_char,
    pub userhl: ::core::ffi::c_int,
    pub item: StlFlag,
}
pub type stl_hlrec_t = stl_hlrec;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stl_item {
    pub start: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub minwid: ::core::ffi::c_int,
    pub maxwid: ::core::ffi::c_int,
    pub type_0: C2Rust_Unnamed_15,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const Trunc: C2Rust_Unnamed_15 = 10;
pub const ClickFunc: C2Rust_Unnamed_15 = 9;
pub const TabPage: C2Rust_Unnamed_15 = 8;
pub const HighlightFold: C2Rust_Unnamed_15 = 7;
pub const HighlightSign: C2Rust_Unnamed_15 = 6;
pub const HighlightCombining: C2Rust_Unnamed_15 = 5;
pub const Highlight: C2Rust_Unnamed_15 = 4;
pub const Separate: C2Rust_Unnamed_15 = 3;
pub const Group: C2Rust_Unnamed_15 = 2;
pub const Empty: C2Rust_Unnamed_15 = 1;
pub const Normal: C2Rust_Unnamed_15 = 0;
pub type stl_item_t = stl_item;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct statuscol_T {
    pub width: ::core::ffi::c_int,
    pub lnum: linenr_T,
    pub sign_cul_id: ::core::ffi::c_int,
    pub draw: bool,
    pub hlrec: *mut stl_hlrec_t,
    pub foldinfo: foldinfo_T,
    pub fold_vcol: [colnr_T; 9],
    pub sattrs: *mut SignTextAttrs,
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_16 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_16 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_16 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_16 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_16 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_16 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_16 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_16 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_16 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_16 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_16 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_16 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_16 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_16 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_16 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_16 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_16 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_16 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_16 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_17 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_17 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_17 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_17 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_17 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_17 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_17 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_17 = 1;
pub type NumberBase = ::core::ffi::c_uint;
pub const kNumBaseHexadecimal: NumberBase = 16;
pub const kNumBaseDecimal: NumberBase = 10;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SID_ERROR: ::core::ffi::c_int = -5 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active
    }
    .offset(hlf as isize);
}
pub const MAX_STL_EVAL_DEPTH: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn win_redr_status(mut wp: *mut win_T) {
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    static mut busy: bool = false_0 != 0;
    if busy as ::core::ffi::c_int != 0
        || wild_menu_showing != 0 as ::core::ffi::c_int && !ui_has(kUIWildmenu)
    {
        return;
    }
    busy = true_0 != 0;
    (*wp).w_redr_status = false_0 != 0;
    if (*wp).w_status_height == 0 as ::core::ffi::c_int
        && !(is_stl_global as ::core::ffi::c_int != 0 && wp == curwin)
    {
        redraw_cmdline = true_0 != 0;
    } else if !redrawing() {
        (*wp).w_redr_status = true_0 != 0;
    } else if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
        || !(*wp).w_floating
        || is_stl_global as ::core::ffi::c_int != 0 && wp == curwin
    {
        redraw_custom_statusline(wp);
    }
    let mut group: hlf_T = HLF_C;
    if (*wp).w_vsep_width != 0 as ::core::ffi::c_int
        && (*wp).w_status_height != 0 as ::core::ffi::c_int
        && redrawing() as ::core::ffi::c_int != 0
    {
        let mut fillchar: schar_T = 0;
        if stl_connected(wp) {
            fillchar = fillchar_status(&raw mut group, wp);
        } else {
            fillchar = (*wp).w_p_fcs_chars.vert;
        }
        let mut attr: ::core::ffi::c_int = win_hl_attr(wp, group as ::core::ffi::c_int);
        grid_line_start(&raw mut default_gridview, (*wp).w_winrow + (*wp).w_height);
        grid_line_put_schar((*wp).w_wincol + (*wp).w_width, fillchar, attr);
        grid_line_flush();
    }
    busy = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_trans_bufname(mut buf: *mut buf_T) {
    if !buf_spname(buf).is_null() {
        xstrlcpy(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            buf_spname(buf),
            MAXPATHL as size_t,
        );
    } else {
        home_replace(
            buf,
            (*buf).b_fname,
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
    }
    trans_characters(&raw mut NameBuff as *mut ::core::ffi::c_char, MAXPATHL);
}
#[no_mangle]
pub unsafe extern "C" fn stl_connected(mut wp: *mut win_T) -> bool {
    let mut fr: *mut frame_T = (*wp).w_frame;
    while !(*fr).fr_parent.is_null() {
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
            if !(*fr).fr_next.is_null() {
                break;
            }
        } else if !(*fr).fr_next.is_null() {
            return true_0 != 0;
        }
        fr = (*fr).fr_parent;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn stl_clear_click_defs(
    click_defs: *mut StlClickDefinition,
    click_defs_size: size_t,
) {
    if !click_defs.is_null() {
        let mut i: size_t = 0 as size_t;
        while i < click_defs_size {
            if i == 0 as size_t
                || (*click_defs.offset(i as isize)).func
                    != (*click_defs.offset(i.wrapping_sub(1 as size_t) as isize)).func
            {
                xfree((*click_defs.offset(i as isize)).func as *mut ::core::ffi::c_void);
            }
            i = i.wrapping_add(1);
        }
        memset(
            click_defs as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            click_defs_size.wrapping_mul(::core::mem::size_of::<StlClickDefinition>()),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn stl_alloc_click_defs(
    mut cdp: *mut StlClickDefinition,
    mut width: ::core::ffi::c_int,
    mut size: *mut size_t,
) -> *mut StlClickDefinition {
    if *size < width as size_t {
        xfree(cdp as *mut ::core::ffi::c_void);
        *size = width as size_t;
        cdp =
            xcalloc(*size, ::core::mem::size_of::<StlClickDefinition>()) as *mut StlClickDefinition;
    }
    return cdp;
}
#[no_mangle]
pub unsafe extern "C" fn stl_fill_click_defs(
    mut click_defs: *mut StlClickDefinition,
    mut click_recs: *mut StlClickRecord,
    mut buf: *const ::core::ffi::c_char,
    mut width: ::core::ffi::c_int,
    mut tabline: bool,
) {
    if click_defs.is_null() {
        return;
    }
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_click_def: StlClickDefinition = StlClickDefinition {
        type_0: kStlClickDisabled,
        tabnr: 0,
        func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*click_recs.offset(i as isize)).start.is_null() {
        len += vim_strnsize(
            buf,
            (*click_recs.offset(i as isize)).start.offset_from(buf) as ::core::ffi::c_int,
        );
        '_c2rust_label: {
            if len <= width {
            } else {
                __assert_fail(
                    b"len <= width\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/statusline.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    187 as ::core::ffi::c_uint,
                    b"void stl_fill_click_defs(StlClickDefinition *, StlClickRecord *, const char *, int, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if col < len {
            while col < len {
                let c2rust_fresh5 = col;
                col = col + 1;
                *click_defs.offset(c2rust_fresh5 as isize) = cur_click_def;
            }
        } else {
            xfree(cur_click_def.func as *mut ::core::ffi::c_void);
        }
        buf = (*click_recs.offset(i as isize)).start;
        cur_click_def = (*click_recs.offset(i as isize)).def;
        if !tabline
            && !(cur_click_def.type_0 as ::core::ffi::c_uint
                == kStlClickDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
                || cur_click_def.type_0 as ::core::ffi::c_uint
                    == kStlClickFuncRun as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            cur_click_def.type_0 = kStlClickDisabled;
        }
        i += 1;
    }
    if col < width {
        while col < width {
            let c2rust_fresh6 = col;
            col = col + 1;
            *click_defs.offset(c2rust_fresh6 as isize) = cur_click_def;
        }
    } else {
        xfree(cur_click_def.func as *mut ::core::ffi::c_void);
    };
}
static mut did_show_ext_ruler: bool = false_0 != 0;
unsafe extern "C" fn win_redr_custom(
    mut wp: *mut win_T,
    mut draw_winbar: bool,
    mut draw_ruler: bool,
    mut ui_event: bool,
) {
    let mut ewp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut p_crb_save: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut start_col: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curattr: ::core::ffi::c_int = 0;
    let mut curgroup: ::core::ffi::c_int = 0;
    let mut content: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut maxcol: ::core::ffi::c_int = 0;
    let mut click_defs: *mut StlClickDefinition = ::core::ptr::null_mut::<StlClickDefinition>();
    static mut entered: bool = false_0 != 0;
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut maxwidth: ::core::ffi::c_int = 0;
    let mut group: hlf_T = HLF_NONE;
    let mut fillchar: schar_T = 0;
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut transbuf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut stl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut opt_idx: OptIndex = kOptInvalid;
    let mut opt_scope: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
    let mut tabtab: *mut StlClickRecord = ::core::ptr::null_mut::<StlClickRecord>();
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid =
        if !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 && !is_stl_global {
            &raw mut (*wp).w_grid_alloc
        } else {
            &raw mut default_grid
        };
    if entered {
        return;
    }
    entered = true_0 != 0;
    '_theend: {
        if wp.is_null() {
            stl = p_tal;
            row = 0 as ::core::ffi::c_int;
            fillchar = ' ' as ::core::ffi::c_int as schar_T;
            group = HLF_TPF;
            attr = *hl_attr_active.offset(group as ::core::ffi::c_int as isize);
            maxwidth = Columns;
            opt_idx = kOptTabline;
        } else if draw_winbar {
            opt_idx = kOptWinbar;
            stl = if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
                (*wp).w_onebuf_opt.wo_wbr
            } else {
                p_wbr
            };
            opt_scope = if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
                OPT_LOCAL as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            row = -1 as ::core::ffi::c_int;
            col = 0 as ::core::ffi::c_int;
            grid = grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
            if row < 0 as ::core::ffi::c_int {
                break '_theend;
            } else {
                fillchar = (*wp).w_p_fcs_chars.wbr;
                group = (if wp == curwin {
                    HLF_WBR as ::core::ffi::c_int
                } else {
                    HLF_WBRNC as ::core::ffi::c_int
                }) as hlf_T;
                attr = win_hl_attr(wp, group as ::core::ffi::c_int);
                maxwidth = (*wp).w_view_width;
                stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
                (*wp).w_winbar_click_defs = stl_alloc_click_defs(
                    (*wp).w_winbar_click_defs,
                    maxwidth,
                    &raw mut (*wp).w_winbar_click_defs_size,
                );
            }
        } else {
            let in_status_line: bool = (*wp).w_status_height != 0 as ::core::ffi::c_int
                || is_stl_global as ::core::ffi::c_int != 0;
            if (*wp).w_floating as ::core::ffi::c_int != 0 && !is_stl_global && !draw_ruler {
                row = (*wp).w_winrow_off + (*wp).w_view_height;
                col = (*wp).w_wincol_off;
                maxwidth = (*wp).w_view_width;
            } else {
                row = if is_stl_global as ::core::ffi::c_int != 0 {
                    Rows - p_ch as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                } else {
                    (*wp).w_winrow + (*wp).w_height
                };
                maxwidth = if in_status_line as ::core::ffi::c_int != 0 && !is_stl_global {
                    (*wp).w_width
                } else {
                    Columns
                };
            }
            fillchar = fillchar_status(&raw mut group, wp);
            stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
            (*wp).w_status_click_defs = stl_alloc_click_defs(
                (*wp).w_status_click_defs,
                maxwidth,
                &raw mut (*wp).w_status_click_defs_size,
            );
            if draw_ruler {
                stl = p_ruf;
                opt_idx = kOptRulerformat;
                if *stl as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                    stl = stl.offset(1);
                    if *stl as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        stl = stl.offset(1);
                    }
                    if atoi(stl) != 0 {
                        while ascii_isdigit(*stl as ::core::ffi::c_int) {
                            stl = stl.offset(1);
                        }
                    }
                    let c2rust_fresh0 = stl;
                    stl = stl.offset(1);
                    if *c2rust_fresh0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                        stl = p_ruf;
                    }
                }
                col = if ru_col - (Columns - maxwidth)
                    > (maxwidth + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int
                {
                    ru_col - (Columns - maxwidth)
                } else {
                    (maxwidth + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int
                };
                maxwidth -= col;
                if !in_status_line {
                    row = Rows - 1 as ::core::ffi::c_int;
                    grid = grid_adjust(&raw mut msg_grid_adj, &raw mut row, &raw mut col);
                    maxwidth -= 1;
                    fillchar = ' ' as ::core::ffi::c_int as schar_T;
                    group = HLF_MSG;
                }
            } else {
                opt_idx = kOptStatusline;
                stl = if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL {
                    (*wp).w_onebuf_opt.wo_stl
                } else {
                    p_stl
                };
                opt_scope = if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL {
                    OPT_LOCAL as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            attr = win_hl_attr(wp, group as ::core::ffi::c_int);
            if !(*wp).w_floating && in_status_line as ::core::ffi::c_int != 0 && !is_stl_global {
                col += (*wp).w_wincol;
            }
        }
        if maxwidth > 0 as ::core::ffi::c_int {
            ewp = if wp.is_null() { curwin } else { wp };
            p_crb_save = (*ewp).w_onebuf_opt.wo_crb;
            (*ewp).w_onebuf_opt.wo_crb = false_0;
            stl = xstrdup(stl);
            build_stl_str_hl(
                ewp,
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                stl,
                opt_idx,
                opt_scope,
                fillchar,
                maxwidth,
                &raw mut hltab,
                ::core::ptr::null_mut::<size_t>(),
                &raw mut tabtab,
                ::core::ptr::null_mut::<statuscol_T>(),
            );
            xfree(stl as *mut ::core::ffi::c_void);
            (*ewp).w_onebuf_opt.wo_crb = p_crb_save;
            len = strlen(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            start_col = col;
            if !ui_event {
                screengrid_line_start(grid, row, 0 as ::core::ffi::c_int);
            }
            p = &raw mut buf as *mut ::core::ffi::c_char;
            curattr = attr;
            curgroup = group as ::core::ffi::c_int;
            content = ARRAY_DICT_INIT;
            let mut sp: *mut stl_hlrec_t = hltab;
            loop {
                let mut textlen: ::core::ffi::c_int = (if !(*sp).start.is_null() {
                    (*sp).start.offset_from(p)
                } else {
                    (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(len as isize)
                        .offset_from(p)
                }) as ::core::ffi::c_int;
                let mut tsize: size_t = transstr_buf(
                    if p >= (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize) {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        p as *const ::core::ffi::c_char
                    },
                    textlen as ssize_t,
                    &raw mut transbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    true_0 != 0,
                );
                if !ui_event {
                    col += grid_line_puts(
                        col,
                        &raw mut transbuf as *mut ::core::ffi::c_char,
                        tsize as ::core::ffi::c_int,
                        curattr,
                    );
                } else {
                    let mut chunk: Array = ARRAY_DICT_INIT;
                    if chunk.size == chunk.capacity {
                        chunk.capacity = (if chunk.capacity != 0 {
                            chunk.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        });
                        chunk.items = xrealloc(
                            chunk.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh1 = chunk.size;
                    chunk.size = chunk.size.wrapping_add(1);
                    *chunk.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: curattr as Integer,
                        },
                    };
                    if chunk.size == chunk.capacity {
                        chunk.capacity = (if chunk.capacity != 0 {
                            chunk.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        });
                        chunk.items = xrealloc(
                            chunk.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh2 = chunk.size;
                    chunk.size = chunk.size.wrapping_add(1);
                    *chunk.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: String_0 {
                                data: xmemdupz(
                                    &raw mut transbuf as *mut ::core::ffi::c_char
                                        as *const ::core::ffi::c_void,
                                    tsize,
                                ) as *mut ::core::ffi::c_char,
                                size: tsize,
                            },
                        },
                    };
                    if chunk.size == chunk.capacity {
                        chunk.capacity = (if chunk.capacity != 0 {
                            chunk.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        });
                        chunk.items = xrealloc(
                            chunk.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh3 = chunk.size;
                    chunk.size = chunk.size.wrapping_add(1);
                    *chunk.items.offset(c2rust_fresh3 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: curgroup as Integer,
                        },
                    };
                    if content.size == content.capacity {
                        content.capacity = (if content.capacity != 0 {
                            content.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        });
                        content.items = xrealloc(
                            content.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh4 = content.size;
                    content.size = content.size.wrapping_add(1);
                    *content.items.offset(c2rust_fresh4 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: chunk },
                    };
                }
                p = (*sp).start;
                if p.is_null() {
                    break;
                }
                if (*sp).userhl == 0 as ::core::ffi::c_int {
                    curattr = attr;
                    curgroup = group as ::core::ffi::c_int;
                } else if (*sp).userhl < 0 as ::core::ffi::c_int {
                    let mut new_attr: ::core::ffi::c_int = syn_id2attr(-(*sp).userhl);
                    if (*sp).item as ::core::ffi::c_uint
                        == STL_HIGHLIGHT_COMB as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        curattr = hl_combine_attr(curattr, new_attr);
                    } else {
                        curattr = new_attr;
                    }
                    curgroup = -(*sp).userhl;
                } else {
                    let mut userhl: *mut ::core::ffi::c_int = if !wp.is_null()
                        && wp != curwin
                        && (*wp).w_status_height != 0 as ::core::ffi::c_int
                    {
                        &raw mut highlight_stlnc as *mut ::core::ffi::c_int
                    } else {
                        &raw mut highlight_user as *mut ::core::ffi::c_int
                    };
                    let mut userbuf: [::core::ffi::c_char; 5] =
                        ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"User\0");
                    userbuf[4 as ::core::ffi::c_int as usize] = ((*sp).userhl as ::core::ffi::c_char
                        as ::core::ffi::c_int
                        + '0' as ::core::ffi::c_int)
                        as ::core::ffi::c_char;
                    curattr = *userhl.offset(((*sp).userhl - 1 as ::core::ffi::c_int) as isize);
                    curgroup =
                        syn_name2id_len(&raw mut userbuf as *mut ::core::ffi::c_char, 5 as size_t);
                }
                if curattr != attr {
                    curattr = hl_combine_attr(attr, curattr);
                }
                sp = sp.offset(1);
            }
            if ui_event {
                ui_call_msg_ruler(content);
                did_show_ext_ruler = true_0 != 0;
                api_free_array(content);
            } else {
                maxcol = start_col + maxwidth;
                grid_line_fill(col, maxcol, fillchar, curattr);
                grid_line_flush();
                click_defs = if wp.is_null() {
                    tab_page_click_defs
                } else if draw_winbar as ::core::ffi::c_int != 0 {
                    (*wp).w_winbar_click_defs
                } else {
                    (*wp).w_status_click_defs
                };
                stl_fill_click_defs(
                    click_defs,
                    tabtab,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    maxwidth,
                    wp.is_null(),
                );
            }
        }
    }
    entered = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_redr_winbar(mut wp: *mut win_T) {
    static mut entered: bool = false_0 != 0;
    if entered {
        return;
    }
    entered = true_0 != 0;
    if !((*wp).w_winbar_height == 0 as ::core::ffi::c_int || !redrawing()) {
        if *p_wbr as ::core::ffi::c_int != NUL
            || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
        {
            win_redr_custom(wp, true_0 != 0, false_0 != 0, false_0 != 0);
        }
    }
    entered = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_ruler() {
    static mut did_ruler_col: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut wp: *mut win_T =
        if !is_aucmd_win(curwin) && (*curwin).w_status_height == 0 as ::core::ffi::c_int {
            curwin
        } else {
            lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>())
        };
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    if p_ru == 0
        || (*wp).w_status_height > 0 as ::core::ffi::c_int
        || is_stl_global as ::core::ffi::c_int != 0
        || p_ch == 0 as OptInt && !ui_has(kUIMessages)
    {
        if did_show_ext_ruler as ::core::ffi::c_int != 0
            && ui_has(kUIMessages) as ::core::ffi::c_int != 0
        {
            ui_call_msg_ruler(ARRAY_DICT_INIT);
            did_show_ext_ruler = false_0 != 0;
        } else if did_ruler_col > 0 as ::core::ffi::c_int {
            msg_col = did_ruler_col;
            msg_row = Rows - 1 as ::core::ffi::c_int;
            msg_clr_eos();
        }
        did_ruler_col = -1 as ::core::ffi::c_int;
        return;
    }
    if (*wp).w_cursor.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
        return;
    }
    if (*wp).w_status_height == 0 as ::core::ffi::c_int && !is_stl_global && !edit_submode.is_null()
    {
        return;
    }
    let mut part_of_status: bool =
        (*wp).w_status_height != 0 || is_stl_global as ::core::ffi::c_int != 0;
    if *p_ruf as ::core::ffi::c_int != 0
        && (p_ch > 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0 && !part_of_status)
    {
        win_redr_custom(wp, false_0 != 0, true_0 != 0, ui_has(kUIMessages));
        return;
    }
    let mut group: hlf_T = HLF_MSG;
    let mut off: ::core::ffi::c_int = if (*wp).w_status_height != 0 {
        (*wp).w_wincol
    } else {
        0 as ::core::ffi::c_int
    };
    let mut width: ::core::ffi::c_int = if (*wp).w_status_height != 0 {
        (*wp).w_width
    } else {
        Columns
    };
    let mut fillchar: schar_T = if part_of_status as ::core::ffi::c_int != 0 {
        fillchar_status(&raw mut group, wp)
    } else {
        ' ' as ::core::ffi::c_int as schar_T
    };
    let mut attr: ::core::ffi::c_int = if part_of_status as ::core::ffi::c_int != 0 {
        win_hl_attr(wp, group as ::core::ffi::c_int)
    } else {
        *hl_attr_active.offset(group as ::core::ffi::c_int as isize)
    };
    let mut virtcol: colnr_T = (*wp).w_virtcol;
    if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.tab1 == NUL as schar_T {
        (*wp).w_onebuf_opt.wo_list = false_0;
        getvvcol(
            wp,
            &raw mut (*wp).w_cursor,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut virtcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
        (*wp).w_onebuf_opt.wo_list = true_0;
    }
    let mut empty_line: ::core::ffi::c_int = (State & MODE_INSERT as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum) as ::core::ffi::c_int == NUL)
        as ::core::ffi::c_int;
    let mut buffer: [::core::ffi::c_char; 70] = [0; 70];
    let mut bufferlen: ::core::ffi::c_int = vim_snprintf(
        &raw mut buffer as *mut ::core::ffi::c_char,
        RULER_BUF_LEN as size_t,
        gettext(b"%ld,\0".as_ptr() as *const ::core::ffi::c_char),
        if (*(*wp).w_buffer).b_ml.ml_flags & ML_EMPTY != 0 {
            0 as int64_t
        } else {
            (*wp).w_cursor.lnum as int64_t
        },
    );
    bufferlen += col_print(
        (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
        (RULER_BUF_LEN as size_t).wrapping_sub(bufferlen as size_t),
        if empty_line != 0 {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_cursor.col + 1 as ::core::ffi::c_int
        },
        virtcol + 1 as ::core::ffi::c_int,
    );
    let mut rel_pos: [::core::ffi::c_char; 70] = [0; 70];
    let mut rel_poslen: ::core::ffi::c_int = get_rel_pos(
        wp,
        &raw mut rel_pos as *mut ::core::ffi::c_char,
        RULER_BUF_LEN,
    );
    let mut n1: ::core::ffi::c_int =
        bufferlen + vim_strsize(&raw mut rel_pos as *mut ::core::ffi::c_char);
    if (*wp).w_status_height == 0 as ::core::ffi::c_int && !is_stl_global {
        n1 += 1;
    }
    let mut this_ru_col: ::core::ffi::c_int = ru_col - (Columns - width);
    let mut n2: ::core::ffi::c_int = (width + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
    this_ru_col = if this_ru_col > n2 { this_ru_col } else { n2 };
    if this_ru_col + n1 < width {
        while this_ru_col + n1 < width
            && RULER_BUF_LEN > bufferlen + rel_poslen + 1 as ::core::ffi::c_int
        {
            bufferlen += schar_get(
                (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
                fillchar,
            ) as ::core::ffi::c_int;
            n1 += 1;
        }
        bufferlen += vim_snprintf(
            (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
            (RULER_BUF_LEN as size_t).wrapping_sub(bufferlen as size_t),
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut rel_pos as *mut ::core::ffi::c_char,
        );
    }
    if ui_has(kUIMessages) as ::core::ffi::c_int != 0 && !part_of_status {
        let mut content: Array = ARRAY_DICT_INIT;
        let mut content__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        content.capacity = 1 as size_t;
        content.items = &raw mut content__items as *mut Object;
        let mut chunk: Array = ARRAY_DICT_INIT;
        let mut chunk__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 3];
        chunk.capacity = 3 as size_t;
        chunk.items = &raw mut chunk__items as *mut Object;
        let c2rust_fresh35 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh35 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: attr as Integer,
            },
        };
        let c2rust_fresh36 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh36 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(&raw mut buffer as *mut ::core::ffi::c_char),
            },
        };
        let c2rust_fresh37 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh37 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: HLF_MSG as ::core::ffi::c_int as Integer,
            },
        };
        '_c2rust_label: {
            if attr == *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize) {
            } else {
                __assert_fail(
                    b"attr == HL_ATTR(HLF_MSG)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/statusline.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    546 as ::core::ffi::c_uint,
                    b"void redraw_ruler(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let c2rust_fresh38 = content.size;
        content.size = content.size.wrapping_add(1);
        *content.items.offset(c2rust_fresh38 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: chunk },
        };
        ui_call_msg_ruler(content);
        did_show_ext_ruler = true_0 != 0;
        did_ruler_col = 1 as ::core::ffi::c_int;
    } else {
        if did_show_ext_ruler {
            ui_call_msg_ruler(ARRAY_DICT_INIT);
            did_show_ext_ruler = false_0 != 0;
        }
        n1 = 0 as ::core::ffi::c_int;
        n2 = 0 as ::core::ffi::c_int;
        while buffer[n1 as usize] as ::core::ffi::c_int != NUL {
            n2 += utf_ptr2cells((&raw mut buffer as *mut ::core::ffi::c_char).offset(n1 as isize));
            if this_ru_col + n2 > width {
                bufferlen = n1;
                buffer[bufferlen as usize] = NUL as ::core::ffi::c_char;
                break;
            } else {
                n1 +=
                    utfc_ptr2len((&raw mut buffer as *mut ::core::ffi::c_char).offset(n1 as isize));
            }
        }
        grid_line_start(&raw mut msg_grid_adj, Rows - 1 as ::core::ffi::c_int);
        did_ruler_col = off + this_ru_col;
        let mut w: ::core::ffi::c_int = grid_line_puts(
            did_ruler_col,
            &raw mut buffer as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            attr,
        );
        grid_line_fill(did_ruler_col + w, off + width, fillchar, attr);
        grid_line_flush();
    };
}
pub const RULER_BUF_LEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn fillchar_status(mut group: *mut hlf_T, mut wp: *mut win_T) -> schar_T {
    if wp == curwin {
        *group = HLF_S;
        return (*wp).w_p_fcs_chars.stl;
    } else {
        *group = HLF_SNC;
        return (*wp).w_p_fcs_chars.stlnc;
    };
}
#[no_mangle]
pub unsafe extern "C" fn redraw_custom_statusline(mut wp: *mut win_T) {
    static mut entered: bool = false_0 != 0;
    if entered {
        return;
    }
    entered = true_0 != 0;
    win_redr_custom(wp, false_0 != 0, false_0 != 0, false_0 != 0);
    entered = false_0 != 0;
}
unsafe extern "C" fn ui_ext_tabline_update() {
    let mut arena: Arena = ARENA_EMPTY;
    let mut n_tabs: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        n_tabs = n_tabs.wrapping_add(1);
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut tabs: Array = arena_array(&raw mut arena, n_tabs);
    let mut tp_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut tab_info: Dict = arena_dict(&raw mut arena, 2 as size_t);
        let c2rust_fresh45 = tab_info.size;
        tab_info.size = tab_info.size.wrapping_add(1);
        *tab_info.items.offset(c2rust_fresh45 as isize) = key_value_pair {
            key: cstr_as_string(b"tab\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeTabpage,
                data: C2Rust_Unnamed {
                    integer: (*tp_0).handle as Integer,
                },
            },
        };
        let mut cwp: *mut win_T = if tp_0 == curtab {
            curwin
        } else {
            (*tp_0).tp_curwin
        };
        get_trans_bufname((*cwp).w_buffer);
        let c2rust_fresh46 = tab_info.size;
        tab_info.size = tab_info.size.wrapping_add(1);
        *tab_info.items.offset(c2rust_fresh46 as isize) = key_value_pair {
            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        &raw mut arena,
                        cstr_as_string(&raw mut NameBuff as *mut ::core::ffi::c_char),
                    ),
                },
            },
        };
        let c2rust_fresh47 = tabs.size;
        tabs.size = tabs.size.wrapping_add(1);
        *tabs.items.offset(c2rust_fresh47 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: tab_info },
        };
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    let mut n_buffers: size_t = 0 as size_t;
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        n_buffers = n_buffers.wrapping_add(
            (if (*buf).b_p_bl != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t,
        );
        buf = (*buf).b_next;
    }
    let mut buffers: Array = arena_array(&raw mut arena, n_buffers);
    let mut buf_0: *mut buf_T = firstbuf;
    while !buf_0.is_null() {
        if (*buf_0).b_p_bl != 0 {
            let mut buffer_info: Dict = arena_dict(&raw mut arena, 2 as size_t);
            let c2rust_fresh48 = buffer_info.size;
            buffer_info.size = buffer_info.size.wrapping_add(1);
            *buffer_info.items.offset(c2rust_fresh48 as isize) = key_value_pair {
                key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBuffer,
                    data: C2Rust_Unnamed {
                        integer: (*buf_0).handle as Integer,
                    },
                },
            };
            get_trans_bufname(buf_0);
            let c2rust_fresh49 = buffer_info.size;
            buffer_info.size = buffer_info.size.wrapping_add(1);
            *buffer_info.items.offset(c2rust_fresh49 as isize) = key_value_pair {
                key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(
                            &raw mut arena,
                            cstr_as_string(&raw mut NameBuff as *mut ::core::ffi::c_char),
                        ),
                    },
                },
            };
            let c2rust_fresh50 = buffers.size;
            buffers.size = buffers.size.wrapping_add(1);
            *buffers.items.offset(c2rust_fresh50 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: buffer_info },
            };
        }
        buf_0 = (*buf_0).b_next;
    }
    ui_call_tabline_update(
        (*curtab).handle as Tabpage,
        tabs,
        (*curbuf).handle as Buffer,
        buffers,
    );
    arena_mem_free(arena_finish(&raw mut arena));
}
#[no_mangle]
pub unsafe extern "C" fn draw_tabline() {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut attr_nosel: ::core::ffi::c_int =
        *hl_attr_active.offset(HLF_TP as ::core::ffi::c_int as isize);
    let mut attr_fill: ::core::ffi::c_int =
        *hl_attr_active.offset(HLF_TPF as ::core::ffi::c_int as isize);
    let mut use_sep_chars: bool = t_colors < 8 as ::core::ffi::c_int;
    if default_grid.chars.is_null() {
        return;
    }
    redraw_tabline = false_0 != 0;
    if ui_has(kUITabline) {
        ui_ext_tabline_update();
        return;
    }
    if tabline_height() < 1 as ::core::ffi::c_int {
        return;
    }
    '_c2rust_label: {
        if tab_page_click_defs_size >= Columns as size_t {
        } else {
            __assert_fail(
                b"tab_page_click_defs_size >= (size_t)Columns\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/statusline.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                672 as ::core::ffi::c_uint,
                b"void draw_tabline(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    stl_clear_click_defs(tab_page_click_defs, tab_page_click_defs_size);
    if *p_tal as ::core::ffi::c_int != NUL {
        win_redr_custom(
            ::core::ptr::null_mut::<win_T>(),
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
    } else {
        let mut tabcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cwp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wincount: ::core::ffi::c_int = 0;
        grid_line_start(&raw mut default_gridview, 0 as ::core::ffi::c_int);
        let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
        while !tp.is_null() {
            tabcount += 1;
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        let mut tabwidth: ::core::ffi::c_int = if (if tabcount > 0 as ::core::ffi::c_int {
            (Columns - 1 as ::core::ffi::c_int + tabcount / 2 as ::core::ffi::c_int) / tabcount
        } else {
            0 as ::core::ffi::c_int
        }) > 6 as ::core::ffi::c_int
        {
            if tabcount > 0 as ::core::ffi::c_int {
                (Columns - 1 as ::core::ffi::c_int + tabcount / 2 as ::core::ffi::c_int) / tabcount
            } else {
                0 as ::core::ffi::c_int
            }
        } else {
            6 as ::core::ffi::c_int
        };
        let mut attr: ::core::ffi::c_int = attr_nosel;
        tabcount = 0 as ::core::ffi::c_int;
        let mut tp_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
        while !tp_0.is_null() {
            if col >= Columns - 4 as ::core::ffi::c_int {
                break;
            }
            let mut scol: ::core::ffi::c_int = col;
            if tp_0 == curtab {
                cwp = curwin;
                wp = firstwin;
            } else {
                cwp = (*tp_0).tp_curwin;
                wp = (*tp_0).tp_firstwin;
            }
            if (*tp_0).tp_topframe == topframe {
                attr = win_hl_attr(cwp, HLF_TPS as ::core::ffi::c_int);
            }
            if use_sep_chars as ::core::ffi::c_int != 0 && col > 0 as ::core::ffi::c_int {
                let c2rust_fresh39 = col;
                col = col + 1;
                grid_line_put_schar(c2rust_fresh39, '|' as ::core::ffi::c_int as schar_T, attr);
            }
            if (*tp_0).tp_topframe != topframe {
                attr = win_hl_attr(cwp, HLF_TP as ::core::ffi::c_int);
            }
            let c2rust_fresh40 = col;
            col = col + 1;
            grid_line_put_schar(c2rust_fresh40, ' ' as ::core::ffi::c_int as schar_T, attr);
            let mut modified: bool = false_0 != 0;
            wincount = 0 as ::core::ffi::c_int;
            while !wp.is_null() {
                if !(*wp).w_config.focusable || (*wp).w_config.hide as ::core::ffi::c_int != 0 {
                    wincount -= 1;
                } else if bufIsChanged((*wp).w_buffer) {
                    modified = true_0 != 0;
                }
                wp = (*wp).w_next;
                wincount += 1;
            }
            if modified as ::core::ffi::c_int != 0 || wincount > 1 as ::core::ffi::c_int {
                if wincount > 1 as ::core::ffi::c_int {
                    let mut len: ::core::ffi::c_int = vim_snprintf(
                        &raw mut NameBuff as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                        wincount,
                    );
                    if col + len >= Columns - 3 as ::core::ffi::c_int {
                        break;
                    }
                    grid_line_puts(
                        col,
                        &raw mut NameBuff as *mut ::core::ffi::c_char,
                        len,
                        hl_combine_attr(attr, win_hl_attr(cwp, HLF_T as ::core::ffi::c_int)),
                    );
                    col += len;
                }
                if modified {
                    let c2rust_fresh41 = col;
                    col = col + 1;
                    grid_line_put_schar(c2rust_fresh41, '+' as ::core::ffi::c_int as schar_T, attr);
                }
                let c2rust_fresh42 = col;
                col = col + 1;
                grid_line_put_schar(c2rust_fresh42, ' ' as ::core::ffi::c_int as schar_T, attr);
            }
            let mut room: ::core::ffi::c_int = scol - col + tabwidth - 1 as ::core::ffi::c_int;
            if room > 0 as ::core::ffi::c_int {
                get_trans_bufname((*cwp).w_buffer);
                shorten_dir(&raw mut NameBuff as *mut ::core::ffi::c_char);
                let mut len_0: ::core::ffi::c_int =
                    vim_strsize(&raw mut NameBuff as *mut ::core::ffi::c_char);
                let mut p: *mut ::core::ffi::c_char = &raw mut NameBuff as *mut ::core::ffi::c_char;
                while len_0 > room {
                    len_0 -= ptr2cells(p);
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                let mut n: ::core::ffi::c_int = Columns - col - 1 as ::core::ffi::c_int;
                len_0 = if len_0 < n { len_0 } else { n };
                grid_line_puts(col, p, -1 as ::core::ffi::c_int, attr);
                col += len_0;
            }
            let c2rust_fresh43 = col;
            col = col + 1;
            grid_line_put_schar(c2rust_fresh43, ' ' as ::core::ffi::c_int as schar_T, attr);
            tabcount += 1;
            while scol < col {
                let c2rust_fresh44 = scol;
                scol = scol + 1;
                *tab_page_click_defs.offset(c2rust_fresh44 as isize) = StlClickDefinition {
                    type_0: kStlClickTabSwitch,
                    tabnr: tabcount,
                    func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        let mut scol_0: ::core::ffi::c_int = col;
        while scol_0 < Columns {
            *tab_page_click_defs.offset(scol_0 as isize) = StlClickDefinition {
                type_0: kStlClickTabSwitch,
                tabnr: 0 as ::core::ffi::c_int,
                func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            scol_0 += 1;
        }
        let mut c: ::core::ffi::c_char = (if use_sep_chars as ::core::ffi::c_int != 0 {
            '_' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        grid_line_fill(col, Columns, c as schar_T, attr_fill);
        if p_sc != 0 && *p_sloc as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
            let mut n_0: ::core::ffi::c_int = Columns
                - col
                - (tabcount > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
                    * 3 as ::core::ffi::c_int;
            let sc_width: ::core::ffi::c_int = if (10 as ::core::ffi::c_int) < n_0 {
                10 as ::core::ffi::c_int
            } else {
                n_0
            };
            if sc_width > 0 as ::core::ffi::c_int {
                grid_line_puts(
                    Columns
                        - sc_width
                        - (tabcount > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
                            * 2 as ::core::ffi::c_int,
                    &raw mut showcmd_buf as *mut ::core::ffi::c_char,
                    sc_width,
                    attr_nosel,
                );
            }
        }
        if tabcount > 1 as ::core::ffi::c_int {
            grid_line_put_schar(
                Columns - 1 as ::core::ffi::c_int,
                'X' as ::core::ffi::c_int as schar_T,
                attr_nosel,
            );
            *tab_page_click_defs.offset((Columns - 1 as ::core::ffi::c_int) as isize) =
                StlClickDefinition {
                    type_0: kStlClickTabClose,
                    tabnr: 999 as ::core::ffi::c_int,
                    func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
        }
        grid_line_flush();
    }
    redraw_tabline = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn build_statuscol_str(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut relnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut stcp: *mut statuscol_T,
) -> ::core::ffi::c_int {
    let fillclick: bool = relnum >= 0 as linenr_T
        && (*stcp).width > 0 as ::core::ffi::c_int
        && lnum == (*wp).w_topline;
    if relnum >= 0 as linenr_T {
        set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
        set_vim_var_nr(VV_RELNUM, relnum as varnumber_T);
    }
    let mut clickrec: *mut StlClickRecord = ::core::ptr::null_mut::<StlClickRecord>();
    let mut stc: *mut ::core::ffi::c_char = xstrdup((*wp).w_onebuf_opt.wo_stc);
    let mut width: ::core::ffi::c_int = build_stl_str_hl(
        wp,
        buf,
        MAXPATHL as size_t,
        stc,
        kOptStatuscolumn,
        OPT_LOCAL as ::core::ffi::c_int,
        0 as schar_T,
        (*stcp).width,
        &raw mut (*stcp).hlrec,
        ::core::ptr::null_mut::<size_t>(),
        if fillclick as ::core::ffi::c_int != 0 {
            &raw mut clickrec
        } else {
            ::core::ptr::null_mut::<*mut StlClickRecord>()
        },
        stcp,
    );
    xfree(stc as *mut ::core::ffi::c_void);
    if fillclick {
        stl_clear_click_defs(
            (*wp).w_statuscol_click_defs,
            (*wp).w_statuscol_click_defs_size,
        );
        (*wp).w_statuscol_click_defs = stl_alloc_click_defs(
            (*wp).w_statuscol_click_defs,
            width,
            &raw mut (*wp).w_statuscol_click_defs_size,
        );
        stl_fill_click_defs(
            (*wp).w_statuscol_click_defs,
            clickrec,
            buf,
            width,
            false_0 != 0,
        );
    }
    return width;
}
#[no_mangle]
pub unsafe extern "C" fn build_stl_str_hl(
    mut wp: *mut win_T,
    mut out: *mut ::core::ffi::c_char,
    mut outlen: size_t,
    mut fmt: *mut ::core::ffi::c_char,
    mut opt_idx: OptIndex,
    mut opt_scope: ::core::ffi::c_int,
    mut fillchar: schar_T,
    mut maxwidth: ::core::ffi::c_int,
    mut hltab: *mut *mut stl_hlrec_t,
    mut hltab_len: *mut size_t,
    mut tabtab: *mut *mut StlClickRecord,
    mut stcp: *mut statuscol_T,
) -> ::core::ffi::c_int {
    static mut stl_items_len: size_t = 20 as size_t;
    static mut stl_items: *mut stl_item_t = ::core::ptr::null_mut::<stl_item_t>();
    static mut stl_groupitems: *mut ::core::ffi::c_int =
        ::core::ptr::null_mut::<::core::ffi::c_int>();
    static mut stl_hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
    static mut stl_tabtab: *mut StlClickRecord = ::core::ptr::null_mut::<StlClickRecord>();
    static mut stl_separator_locations: *mut ::core::ffi::c_int =
        ::core::ptr::null_mut::<::core::ffi::c_int>();
    static mut curitem: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf_tmp: [::core::ffi::c_char; 70] = [0; 70];
    let mut usefmt: *mut ::core::ffi::c_char = fmt;
    let save_redraw_not_allowed: bool = redraw_not_allowed;
    let save_KeyTyped: bool = KeyTyped;
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    if updating_screen {
        redraw_not_allowed = true_0 != 0;
    }
    if stl_items.is_null() {
        stl_items = xmalloc(::core::mem::size_of::<stl_item_t>().wrapping_mul(stl_items_len))
            as *mut stl_item_t;
        stl_groupitems =
            xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(stl_items_len))
                as *mut ::core::ffi::c_int;
        stl_hltab = xmalloc(
            ::core::mem::size_of::<stl_hlrec_t>()
                .wrapping_mul(stl_items_len.wrapping_add(1 as size_t)),
        ) as *mut stl_hlrec_t;
        stl_tabtab = xmalloc(
            ::core::mem::size_of::<StlClickRecord>()
                .wrapping_mul(stl_items_len.wrapping_add(1 as size_t)),
        ) as *mut StlClickRecord;
        stl_separator_locations =
            xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(stl_items_len))
                as *mut ::core::ffi::c_int;
    }
    let use_sandbox: bool = if opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int {
        was_set_insecurely(wp, opt_idx, opt_scope)
    } else {
        false_0
    } != 0;
    if *fmt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '%' as ::core::ffi::c_int
        && *fmt.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '!' as ::core::ffi::c_int
    {
        let mut tv: typval_T = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: (*wp).handle as varnumber_T,
            },
        };
        set_var(
            b"g:statusline_winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
            &raw mut tv,
            false_0 != 0,
        );
        usefmt = eval_to_string_safe(
            fmt.offset(2 as ::core::ffi::c_int as isize),
            use_sandbox,
            false_0 != 0,
        );
        if usefmt.is_null() {
            usefmt = fmt;
        }
        do_unlet(
            b"g:statusline_winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
    }
    if fillchar == 0 as schar_T {
        fillchar = ' ' as ::core::ffi::c_int as schar_T;
    }
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    if lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
        lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
        (*wp).w_cursor.lnum = lnum;
    }
    let mut line_ptr: *const ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut empty_line: bool = *line_ptr as ::core::ffi::c_int == NUL;
    let mut byteval: ::core::ffi::c_int = 0;
    let len: colnr_T = ml_get_buf_len((*wp).w_buffer, lnum);
    if (*wp).w_cursor.col > len {
        (*wp).w_cursor.col = len;
        (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        byteval = 0 as ::core::ffi::c_int;
    } else {
        byteval = utf_ptr2char(line_ptr.offset((*wp).w_cursor.col as isize));
    }
    let mut groupdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut evaldepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut evalstart: ::core::ffi::c_int = curitem;
    let mut prevchar_isflag: bool = true_0 != 0;
    let mut prevchar_isitem: bool = false_0 != 0;
    let mut out_p: *mut ::core::ffi::c_char = out;
    let mut out_end_p: *mut ::core::ffi::c_char = out
        .offset(outlen as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    let mut fmt_p: *mut ::core::ffi::c_char = usefmt;
    's_2297: while *fmt_p as ::core::ffi::c_int != NUL {
        if curitem == stl_items_len as ::core::ffi::c_int {
            let mut new_len: size_t = stl_items_len
                .wrapping_mul(3 as size_t)
                .wrapping_div(2 as size_t);
            stl_items = xrealloc(
                stl_items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<stl_item_t>().wrapping_mul(new_len),
            ) as *mut stl_item_t;
            stl_groupitems = xrealloc(
                stl_groupitems as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(new_len),
            ) as *mut ::core::ffi::c_int;
            stl_hltab = xrealloc(
                stl_hltab as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<stl_hlrec_t>()
                    .wrapping_mul(new_len.wrapping_add(1 as size_t)),
            ) as *mut stl_hlrec_t;
            stl_tabtab = xrealloc(
                stl_tabtab as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<StlClickRecord>()
                    .wrapping_mul(new_len.wrapping_add(1 as size_t)),
            ) as *mut StlClickRecord;
            stl_separator_locations = xrealloc(
                stl_separator_locations as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(new_len),
            ) as *mut ::core::ffi::c_int;
            stl_items_len = new_len;
        }
        if *fmt_p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
            prevchar_isitem = false_0 != 0;
            prevchar_isflag = prevchar_isitem;
        }
        while *fmt_p as ::core::ffi::c_int != NUL
            && *fmt_p as ::core::ffi::c_int != '%' as ::core::ffi::c_int
            && out_p < out_end_p
        {
            let c2rust_fresh7 = fmt_p;
            fmt_p = fmt_p.offset(1);
            let c2rust_fresh8 = out_p;
            out_p = out_p.offset(1);
            *c2rust_fresh8 = *c2rust_fresh7;
        }
        if *fmt_p as ::core::ffi::c_int == NUL || out_p >= out_end_p {
            break;
        }
        fmt_p = fmt_p.offset(1);
        if *fmt_p as ::core::ffi::c_int == NUL {
            break;
        }
        if *fmt_p as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            let c2rust_fresh9 = fmt_p;
            fmt_p = fmt_p.offset(1);
            let c2rust_fresh10 = out_p;
            out_p = out_p.offset(1);
            *c2rust_fresh10 = *c2rust_fresh9;
            prevchar_isitem = false_0 != 0;
            prevchar_isflag = prevchar_isitem;
        } else if *fmt_p as ::core::ffi::c_int == STL_SEPARATE as ::core::ffi::c_int {
            fmt_p = fmt_p.offset(1);
            if groupdepth > 0 as ::core::ffi::c_int {
                continue;
            }
            (*stl_items.offset(curitem as isize)).type_0 = Separate;
            let c2rust_fresh11 = curitem;
            curitem = curitem + 1;
            let c2rust_lvalue_ptr = &raw mut (*stl_items.offset(c2rust_fresh11 as isize)).start;
            *c2rust_lvalue_ptr = out_p;
        } else if *fmt_p as ::core::ffi::c_int == STL_TRUNCMARK as ::core::ffi::c_int {
            fmt_p = fmt_p.offset(1);
            (*stl_items.offset(curitem as isize)).type_0 = Trunc;
            let c2rust_fresh12 = curitem;
            curitem = curitem + 1;
            let c2rust_lvalue_ptr_0 = &raw mut (*stl_items.offset(c2rust_fresh12 as isize)).start;
            *c2rust_lvalue_ptr_0 = out_p;
        } else if *fmt_p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
            fmt_p = fmt_p.offset(1);
            if groupdepth < 1 as ::core::ffi::c_int {
                continue;
            }
            groupdepth -= 1;
            let mut t: *mut ::core::ffi::c_char =
                (*stl_items.offset(*stl_groupitems.offset(groupdepth as isize) as isize)).start;
            *out_p = NUL as ::core::ffi::c_char;
            let mut group_len: ptrdiff_t = vim_strsize(t) as ptrdiff_t;
            if curitem > *stl_groupitems.offset(groupdepth as isize) + 1 as ::core::ffi::c_int
                && (*stl_items.offset(*stl_groupitems.offset(groupdepth as isize) as isize)).minwid
                    == 0 as ::core::ffi::c_int
            {
                let mut group_start_userhl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut group_end_userhl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut n: ::core::ffi::c_int = 0;
                n = *stl_groupitems.offset(groupdepth as isize) - 1 as ::core::ffi::c_int;
                while n >= 0 as ::core::ffi::c_int {
                    if (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                        == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        group_end_userhl = (*stl_items.offset(n as isize)).minwid;
                        group_start_userhl = group_end_userhl;
                        break;
                    } else {
                        n -= 1;
                    }
                }
                n = *stl_groupitems.offset(groupdepth as isize) + 1 as ::core::ffi::c_int;
                while n < curitem {
                    if (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                        == Normal as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                    if (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                        == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        group_end_userhl = (*stl_items.offset(n as isize)).minwid;
                    }
                    n += 1;
                }
                if n == curitem && group_start_userhl == group_end_userhl {
                    out_p = t;
                    group_len = 0 as ptrdiff_t;
                    n = *stl_groupitems.offset(groupdepth as isize) + 1 as ::core::ffi::c_int;
                    while n < curitem {
                        if (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                                == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            (*stl_items.offset(n as isize)).type_0 = Empty;
                        }
                        if (*stl_items.offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            (*stl_items.offset(n as isize)).start = out_p;
                        }
                        n += 1;
                    }
                }
            }
            let mut minwid: ::core::ffi::c_int =
                (*stl_items.offset(*stl_groupitems.offset(groupdepth as isize) as isize)).minwid;
            if group_len
                > (*stl_items.offset(*stl_groupitems.offset(groupdepth as isize) as isize)).maxwid
                    as ptrdiff_t
                && (*stl_items.offset(*stl_groupitems.offset(groupdepth as isize) as isize)).type_0
                    as ::core::ffi::c_uint
                    != HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut maxwid: ::core::ffi::c_int = (*stl_items
                    .offset(*stl_groupitems.offset(groupdepth as isize) as isize))
                .maxwid;
                let mut n_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while group_len >= maxwid as ptrdiff_t {
                    group_len -= ptr2cells(t.offset(n_0 as isize)) as ptrdiff_t;
                    n_0 += utfc_ptr2len(t.offset(n_0 as isize));
                }
                *t = '<' as ::core::ffi::c_char;
                memmove(
                    t.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    t.offset(n_0 as isize) as *const ::core::ffi::c_void,
                    out_p.offset_from(t.offset(n_0 as isize)) as size_t,
                );
                out_p = out_p
                    .offset(-(n_0 as isize))
                    .offset(1 as ::core::ffi::c_int as isize);
                minwid = if minwid < maxwid { minwid } else { maxwid };
                loop {
                    group_len += 1;
                    if group_len >= minwid as ptrdiff_t {
                        break;
                    }
                    schar_get_adv(&raw mut out_p, fillchar);
                }
                let mut idx: ::core::ffi::c_int =
                    *stl_groupitems.offset(groupdepth as isize) + 1 as ::core::ffi::c_int;
                while idx < curitem {
                    (*stl_items.offset(idx as isize)).start = (*stl_items.offset(idx as isize))
                        .start
                        .offset(-((n_0 - 1 as ::core::ffi::c_int) as isize));
                    (*stl_items.offset(idx as isize)).start =
                        if (*stl_items.offset(idx as isize)).start > t {
                            (*stl_items.offset(idx as isize)).start
                        } else {
                            t
                        };
                    idx += 1;
                }
            } else if abs(minwid) as ptrdiff_t > group_len {
                let mut fillchar_bytes: ptrdiff_t = schar_len(fillchar) as ptrdiff_t;
                if minwid < 0 as ::core::ffi::c_int {
                    minwid = 0 as ::core::ffi::c_int - minwid;
                    loop {
                        let c2rust_fresh13 = group_len;
                        group_len = group_len + 1;
                        if !(c2rust_fresh13 < minwid as ptrdiff_t
                            && out_p.offset(fillchar_bytes as isize) <= out_end_p)
                        {
                            break;
                        }
                        schar_get_adv(&raw mut out_p, fillchar);
                    }
                } else {
                    let mut added_cells: ptrdiff_t = minwid as ptrdiff_t - group_len;
                    let mut added_bytes: ptrdiff_t = added_cells * fillchar_bytes;
                    if out_p.offset(added_bytes as isize) > out_end_p {
                        added_cells =
                            (out_end_p.offset_from(out_p) / fillchar_bytes as isize) as ptrdiff_t;
                        added_bytes = added_cells * fillchar_bytes;
                    }
                    memmove(
                        t.offset(added_bytes as isize) as *mut ::core::ffi::c_void,
                        t as *const ::core::ffi::c_void,
                        out_p.offset_from(t) as size_t,
                    );
                    out_p = out_p.offset(added_bytes as isize);
                    let mut n_1: ::core::ffi::c_int =
                        *stl_groupitems.offset(groupdepth as isize) + 1 as ::core::ffi::c_int;
                    while n_1 < curitem {
                        (*stl_items.offset(n_1 as isize)).start = (*stl_items.offset(n_1 as isize))
                            .start
                            .offset(added_bytes as isize);
                        n_1 += 1;
                    }
                    while added_cells > 0 as ptrdiff_t {
                        schar_get_adv(&raw mut t, fillchar);
                        added_cells -= 1;
                    }
                }
            }
        } else {
            let mut minwid_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut maxwid_0: ::core::ffi::c_int = 9999 as ::core::ffi::c_int;
            let mut foldsignitem: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut left_align_num: bool = false_0 != 0;
            let mut left_align: bool = false_0 != 0;
            let mut zeropad: bool = *fmt_p as ::core::ffi::c_int == '0' as ::core::ffi::c_int;
            if zeropad {
                fmt_p = fmt_p.offset(1);
            }
            if *fmt_p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                fmt_p = fmt_p.offset(1);
                left_align = true_0 != 0;
            }
            if ascii_isdigit(*fmt_p as ::core::ffi::c_int) {
                minwid_0 = getdigits_int(&raw mut fmt_p, false_0 != 0, 0 as ::core::ffi::c_int);
            }
            if *fmt_p as ::core::ffi::c_int == STL_USER_HL as ::core::ffi::c_int {
                (*stl_items.offset(curitem as isize)).type_0 = Highlight;
                (*stl_items.offset(curitem as isize)).start = out_p;
                (*stl_items.offset(curitem as isize)).minwid = if minwid_0 > 9 as ::core::ffi::c_int
                {
                    1 as ::core::ffi::c_int
                } else {
                    minwid_0
                };
                fmt_p = fmt_p.offset(1);
                curitem += 1;
            } else if *fmt_p as ::core::ffi::c_int == STL_TABPAGENR as ::core::ffi::c_int
                || *fmt_p as ::core::ffi::c_int == STL_TABCLOSENR as ::core::ffi::c_int
            {
                if *fmt_p as ::core::ffi::c_int == STL_TABCLOSENR as ::core::ffi::c_int {
                    if minwid_0 == 0 as ::core::ffi::c_int {
                        let mut n_2: ::core::ffi::c_int = curitem - 1 as ::core::ffi::c_int;
                        while n_2 >= 0 as ::core::ffi::c_int {
                            if (*stl_items.offset(n_2 as isize)).type_0 as ::core::ffi::c_uint
                                == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
                                && (*stl_items.offset(n_2 as isize)).minwid
                                    >= 0 as ::core::ffi::c_int
                            {
                                minwid_0 = (*stl_items.offset(n_2 as isize)).minwid;
                                break;
                            } else {
                                n_2 -= 1;
                            }
                        }
                    } else {
                        minwid_0 = -minwid_0;
                    }
                }
                (*stl_items.offset(curitem as isize)).type_0 = TabPage;
                (*stl_items.offset(curitem as isize)).start = out_p;
                (*stl_items.offset(curitem as isize)).minwid = minwid_0;
                fmt_p = fmt_p.offset(1);
                curitem += 1;
            } else if *fmt_p as ::core::ffi::c_int == STL_CLICK_FUNC as ::core::ffi::c_int {
                fmt_p = fmt_p.offset(1);
                let mut t_0: *mut ::core::ffi::c_char = fmt_p;
                while *fmt_p as ::core::ffi::c_int != STL_CLICK_FUNC as ::core::ffi::c_int
                    && *fmt_p as ::core::ffi::c_int != 0
                {
                    fmt_p = fmt_p.offset(1);
                }
                if *fmt_p as ::core::ffi::c_int != STL_CLICK_FUNC as ::core::ffi::c_int {
                    break;
                }
                (*stl_items.offset(curitem as isize)).type_0 = ClickFunc;
                (*stl_items.offset(curitem as isize)).start = out_p;
                (*stl_items.offset(curitem as isize)).cmd = (if !tabtab.is_null() {
                    xmemdupz(
                        t_0 as *const ::core::ffi::c_void,
                        fmt_p.offset_from(t_0) as size_t,
                    )
                } else {
                    NULL
                })
                    as *mut ::core::ffi::c_char;
                (*stl_items.offset(curitem as isize)).minwid = minwid_0;
                fmt_p = fmt_p.offset(1);
                curitem += 1;
            } else {
                if *fmt_p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    fmt_p = fmt_p.offset(1);
                    if ascii_isdigit(*fmt_p as ::core::ffi::c_int) {
                        maxwid_0 =
                            getdigits_int(&raw mut fmt_p, false_0 != 0, 50 as ::core::ffi::c_int);
                    }
                }
                minwid_0 = (if minwid_0 > 50 as ::core::ffi::c_int {
                    50 as ::core::ffi::c_int
                } else {
                    minwid_0
                }) * (if left_align as ::core::ffi::c_int != 0 {
                    -1 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                });
                if *fmt_p as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                    let c2rust_fresh14 = groupdepth;
                    groupdepth = groupdepth + 1;
                    *stl_groupitems.offset(c2rust_fresh14 as isize) = curitem;
                    (*stl_items.offset(curitem as isize)).type_0 = Group;
                    (*stl_items.offset(curitem as isize)).start = out_p;
                    (*stl_items.offset(curitem as isize)).minwid = minwid_0;
                    (*stl_items.offset(curitem as isize)).maxwid = maxwid_0;
                    fmt_p = fmt_p.offset(1);
                    curitem += 1;
                } else if *fmt_p as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                    && evaldepth > 0 as ::core::ffi::c_int
                {
                    fmt_p = fmt_p.offset(1);
                    evaldepth -= 1;
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
                        *fmt_p as uint8_t as ::core::ffi::c_int,
                    )
                    .is_null()
                    {
                        if *fmt_p as ::core::ffi::c_int == NUL {
                            break;
                        } else {
                            fmt_p = fmt_p.offset(1);
                        }
                    } else {
                        let c2rust_fresh15 = fmt_p;
                        fmt_p = fmt_p.offset(1);
                        let mut opt: ::core::ffi::c_char = *c2rust_fresh15;
                        let mut base: NumberBase = kNumBaseDecimal;
                        let mut itemisflag: bool = false_0 != 0;
                        let mut fillable: bool = true_0 != 0;
                        let mut num: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut str: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        's_1848: {
                            '_stcsign: {
                                's_1418: {
                                    match opt as ::core::ffi::c_int {
                                        102 | 70 | 116 => {
                                            fillable = false_0 != 0;
                                            let mut name: *mut ::core::ffi::c_char =
                                                buf_spname((*wp).w_buffer);
                                            if !name.is_null() {
                                                xstrlcpy(
                                                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                                                    name,
                                                    MAXPATHL as size_t,
                                                );
                                            } else {
                                                let mut t_1: *mut ::core::ffi::c_char = if opt
                                                    as ::core::ffi::c_int
                                                    == STL_FULLPATH as ::core::ffi::c_int
                                                {
                                                    (*(*wp).w_buffer).b_ffname
                                                } else {
                                                    (*(*wp).w_buffer).b_fname
                                                };
                                                home_replace(
                                                    (*wp).w_buffer,
                                                    t_1,
                                                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                                                    MAXPATHL as size_t,
                                                    true_0 != 0,
                                                );
                                            }
                                            trans_characters(
                                                &raw mut NameBuff as *mut ::core::ffi::c_char,
                                                MAXPATHL,
                                            );
                                            if opt as ::core::ffi::c_int
                                                != STL_FILENAME as ::core::ffi::c_int
                                            {
                                                str = &raw mut NameBuff as *mut ::core::ffi::c_char;
                                            } else {
                                                str = path_tail(
                                                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                                                );
                                            }
                                            break 's_1848;
                                        }
                                        123 => {
                                            let mut block_start: *mut ::core::ffi::c_char =
                                                fmt_p.offset(-(1 as ::core::ffi::c_int as isize));
                                            let mut reevaluate: bool = *fmt_p as ::core::ffi::c_int
                                                == '%' as ::core::ffi::c_int;
                                            itemisflag = true_0 != 0;
                                            if reevaluate {
                                                fmt_p = fmt_p.offset(1);
                                            }
                                            let mut t_2: *mut ::core::ffi::c_char = out_p;
                                            while (*fmt_p as ::core::ffi::c_int
                                                != '}' as ::core::ffi::c_int
                                                || reevaluate as ::core::ffi::c_int != 0
                                                    && *fmt_p
                                                        .offset(-1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        != '%' as ::core::ffi::c_int)
                                                && *fmt_p as ::core::ffi::c_int != NUL
                                                && out_p < out_end_p
                                            {
                                                let c2rust_fresh16 = fmt_p;
                                                fmt_p = fmt_p.offset(1);
                                                let c2rust_fresh17 = out_p;
                                                out_p = out_p.offset(1);
                                                *c2rust_fresh17 = *c2rust_fresh16;
                                            }
                                            if *fmt_p as ::core::ffi::c_int
                                                != '}' as ::core::ffi::c_int
                                            {
                                                break 's_1848;
                                            } else {
                                                fmt_p = fmt_p.offset(1);
                                                if reevaluate as ::core::ffi::c_int != 0
                                                    && out_p > out
                                                {
                                                    *out_p.offset(
                                                        -1 as ::core::ffi::c_int as isize,
                                                    ) = NUL as ::core::ffi::c_char;
                                                } else {
                                                    *out_p = NUL as ::core::ffi::c_char;
                                                }
                                                out_p = t_2;
                                                vim_snprintf(
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 70]>(
                                                    ),
                                                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                                                    (*curbuf).handle,
                                                );
                                                set_internal_string_var(
                                                    b"g:actual_curbuf\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                );
                                                vim_snprintf(
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 70]>(
                                                    ),
                                                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                                                    (*curwin).handle,
                                                );
                                                set_internal_string_var(
                                                    b"g:actual_curwin\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                );
                                                let save_curbuf: *mut buf_T = curbuf;
                                                let save_curwin: *mut win_T = curwin;
                                                let save_VIsual_active: ::core::ffi::c_int =
                                                    VIsual_active as ::core::ffi::c_int;
                                                curwin = wp;
                                                curbuf = (*wp).w_buffer;
                                                if curwin != save_curwin {
                                                    VIsual_active = false_0 != 0;
                                                }
                                                str = eval_to_string_safe(
                                                    out_p,
                                                    use_sandbox,
                                                    false_0 != 0,
                                                );
                                                curwin = save_curwin;
                                                curbuf = save_curbuf;
                                                VIsual_active = save_VIsual_active != 0;
                                                do_unlet(
                                                    b"g:actual_curbuf\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 16]>()
                                                        .wrapping_sub(1 as size_t),
                                                    true_0 != 0,
                                                );
                                                do_unlet(
                                                    b"g:actual_curwin\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 16]>()
                                                        .wrapping_sub(1 as size_t),
                                                    true_0 != 0,
                                                );
                                                if !str.is_null()
                                                    && *str as ::core::ffi::c_int != NUL
                                                {
                                                    if *skipdigits(str) as ::core::ffi::c_int == NUL
                                                    {
                                                        num = atoi(str);
                                                        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut str
                                                            as *mut *mut ::core::ffi::c_void;
                                                        xfree(*ptr_);
                                                        *ptr_ = NULL;
                                                        *ptr_;
                                                        itemisflag = false_0 != 0;
                                                    }
                                                }
                                                if reevaluate as ::core::ffi::c_int != 0
                                                    && !str.is_null()
                                                    && *str as ::core::ffi::c_int != NUL
                                                    && !strchr(str, '%' as ::core::ffi::c_int)
                                                        .is_null()
                                                    && evaldepth < MAX_STL_EVAL_DEPTH
                                                {
                                                    let mut parsed_usefmt: size_t =
                                                        block_start.offset_from(usefmt) as size_t;
                                                    let mut str_length: size_t = strlen(str);
                                                    let mut fmt_length: size_t = strlen(fmt_p);
                                                    let mut new_fmt_len: size_t = parsed_usefmt
                                                        .wrapping_add(str_length)
                                                        .wrapping_add(fmt_length)
                                                        .wrapping_add(3 as size_t);
                                                    let mut new_fmt: *mut ::core::ffi::c_char =
                                                        xmalloc(new_fmt_len.wrapping_mul(
                                                            ::core::mem::size_of::<
                                                                ::core::ffi::c_char,
                                                            >(
                                                            ),
                                                        ))
                                                            as *mut ::core::ffi::c_char;
                                                    let mut new_fmt_p: *mut ::core::ffi::c_char =
                                                        new_fmt;
                                                    new_fmt_p = (memcpy(
                                                        new_fmt_p as *mut ::core::ffi::c_void,
                                                        usefmt as *const ::core::ffi::c_void,
                                                        parsed_usefmt,
                                                    )
                                                        as *mut ::core::ffi::c_char)
                                                        .offset(parsed_usefmt as isize);
                                                    new_fmt_p = (memcpy(
                                                        new_fmt_p as *mut ::core::ffi::c_void,
                                                        str as *const ::core::ffi::c_void,
                                                        str_length,
                                                    )
                                                        as *mut ::core::ffi::c_char)
                                                        .offset(str_length as isize);
                                                    new_fmt_p = (memcpy(
                                                        new_fmt_p as *mut ::core::ffi::c_void,
                                                        b"%}\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                            as *const ::core::ffi::c_void,
                                                        2 as size_t,
                                                    )
                                                        as *mut ::core::ffi::c_char)
                                                        .offset(2 as ::core::ffi::c_int as isize);
                                                    new_fmt_p = (memcpy(
                                                        new_fmt_p as *mut ::core::ffi::c_void,
                                                        fmt_p as *const ::core::ffi::c_void,
                                                        fmt_length,
                                                    )
                                                        as *mut ::core::ffi::c_char)
                                                        .offset(fmt_length as isize);
                                                    *new_fmt_p = 0 as ::core::ffi::c_char;
                                                    new_fmt_p = ::core::ptr::null_mut::<
                                                        ::core::ffi::c_char,
                                                    >(
                                                    );
                                                    if usefmt != fmt {
                                                        xfree(usefmt as *mut ::core::ffi::c_void);
                                                    }
                                                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                                        &raw mut str
                                                            as *mut *mut ::core::ffi::c_void;
                                                    xfree(*ptr__0);
                                                    *ptr__0 = NULL;
                                                    *ptr__0;
                                                    usefmt = new_fmt;
                                                    fmt_p = usefmt.offset(parsed_usefmt as isize);
                                                    evaldepth += 1;
                                                    continue 's_2297;
                                                } else {
                                                    break 's_1848;
                                                }
                                            }
                                        }
                                        108 => {
                                            if !stcp.is_null()
                                                && ((*wp).w_onebuf_opt.wo_nu != 0
                                                    || (*wp).w_onebuf_opt.wo_rnu != 0)
                                                && get_vim_var_nr(VV_VIRTNUM) == 0 as varnumber_T
                                            {
                                                if (*wp).w_maxscwidth == SCL_NUM
                                                    && (*(*stcp)
                                                        .sattrs
                                                        .offset(0 as ::core::ffi::c_int as isize))
                                                    .text
                                                        [0 as ::core::ffi::c_int as usize]
                                                        != 0
                                                {
                                                    break '_stcsign;
                                                } else {
                                                    let mut relnum: ::core::ffi::c_int =
                                                        get_vim_var_nr(VV_RELNUM)
                                                            as ::core::ffi::c_int;
                                                    num = if (*wp).w_onebuf_opt.wo_rnu == 0
                                                        || (*wp).w_onebuf_opt.wo_nu != 0
                                                            && relnum == 0 as ::core::ffi::c_int
                                                    {
                                                        get_vim_var_nr(VV_LNUM)
                                                            as ::core::ffi::c_int
                                                    } else {
                                                        relnum
                                                    };
                                                    left_align_num = (*wp).w_onebuf_opt.wo_rnu != 0
                                                        && (*wp).w_onebuf_opt.wo_nu != 0
                                                        && relnum == 0 as ::core::ffi::c_int;
                                                    if !left_align_num {
                                                        (*stl_items.offset(curitem as isize))
                                                            .type_0 = Separate;
                                                        let c2rust_fresh18 = curitem;
                                                        curitem = curitem + 1;
                                                        let c2rust_lvalue_ptr_1 =
                                                            &raw mut (*stl_items
                                                                .offset(c2rust_fresh18 as isize))
                                                            .start;
                                                        *c2rust_lvalue_ptr_1 = out_p;
                                                    }
                                                    break 's_1848;
                                                }
                                            } else {
                                                if stcp.is_null() {
                                                    num = (if (*(*wp).w_buffer).b_ml.ml_flags
                                                        & ML_EMPTY
                                                        != 0
                                                    {
                                                        0 as linenr_T
                                                    } else {
                                                        (*wp).w_cursor.lnum
                                                    })
                                                        as ::core::ffi::c_int;
                                                }
                                                break 's_1848;
                                            }
                                        }
                                        76 => {
                                            num = (*(*wp).w_buffer).b_ml.ml_line_count
                                                as ::core::ffi::c_int;
                                            break 's_1848;
                                        }
                                        99 => {
                                            num = if State & MODE_INSERT as ::core::ffi::c_int
                                                == 0 as ::core::ffi::c_int
                                                && empty_line as ::core::ffi::c_int != 0
                                            {
                                                0 as ::core::ffi::c_int
                                            } else {
                                                (*wp).w_cursor.col + 1 as ::core::ffi::c_int
                                            };
                                            break 's_1848;
                                        }
                                        118 | 86 => {
                                            let mut virtcol: colnr_T =
                                                (*wp).w_virtcol + 1 as colnr_T;
                                            if opt as ::core::ffi::c_int
                                                == STL_VIRTCOL_ALT as ::core::ffi::c_int
                                                && virtcol
                                                    == (if State & MODE_INSERT as ::core::ffi::c_int
                                                        == 0 as ::core::ffi::c_int
                                                        && empty_line as ::core::ffi::c_int != 0
                                                    {
                                                        0 as ::core::ffi::c_int
                                                    } else {
                                                        (*wp).w_cursor.col + 1 as ::core::ffi::c_int
                                                    })
                                            {
                                                break 's_1848;
                                            } else {
                                                num = virtcol as ::core::ffi::c_int;
                                                break 's_1848;
                                            }
                                        }
                                        112 => {
                                            num = calc_percentage(
                                                (*wp).w_cursor.lnum as int64_t,
                                                (*(*wp).w_buffer).b_ml.ml_line_count as int64_t,
                                            );
                                            break 's_1848;
                                        }
                                        80 => {
                                            get_rel_pos(
                                                wp,
                                                &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                TMPLEN,
                                            );
                                            str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                            break 's_1848;
                                        }
                                        83 => {
                                            if p_sc != 0
                                                && (opt_idx as ::core::ffi::c_int
                                                    == kOptInvalid as ::core::ffi::c_int
                                                    || find_option(p_sloc) as ::core::ffi::c_int
                                                        == opt_idx as ::core::ffi::c_int)
                                            {
                                                str = &raw mut showcmd_buf
                                                    as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        97 => {
                                            fillable = false_0 != 0;
                                            buf_tmp[0 as ::core::ffi::c_int as usize] =
                                                NUL as ::core::ffi::c_char;
                                            if append_arg_number(
                                                wp,
                                                &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                ::core::mem::size_of::<[::core::ffi::c_char; 70]>(),
                                            ) > 0 as ::core::ffi::c_int
                                            {
                                                str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        107 => {
                                            fillable = false_0 != 0;
                                            if get_keymap_str(
                                                wp,
                                                b"<%s>\0".as_ptr() as *const ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_char,
                                                &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                TMPLEN,
                                            ) > 0 as ::core::ffi::c_int
                                            {
                                                str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        78 => {
                                            num = 0 as ::core::ffi::c_int;
                                            break 's_1848;
                                        }
                                        110 => {
                                            num = (*(*wp).w_buffer).handle as ::core::ffi::c_int;
                                            break 's_1848;
                                        }
                                        79 => {
                                            base = kNumBaseHexadecimal;
                                            break 's_1418;
                                        }
                                        111 => {
                                            break 's_1418;
                                        }
                                        66 => {
                                            base = kNumBaseHexadecimal;
                                        }
                                        98 => {}
                                        114 | 82 => {
                                            itemisflag = true_0 != 0;
                                            if (*(*wp).w_buffer).b_p_ro != 0 {
                                                str = (if opt as ::core::ffi::c_int
                                                    == STL_ROFLAG_ALT as ::core::ffi::c_int
                                                {
                                                    b",RO\0".as_ptr() as *const ::core::ffi::c_char
                                                } else {
                                                    gettext(b"[RO]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                        as *const ::core::ffi::c_char
                                                })
                                                    as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        104 | 72 => {
                                            itemisflag = true_0 != 0;
                                            if (*(*wp).w_buffer).b_help {
                                                str = (if opt as ::core::ffi::c_int
                                                    == STL_HELPFLAG_ALT as ::core::ffi::c_int
                                                {
                                                    b",HLP\0".as_ptr() as *const ::core::ffi::c_char
                                                } else {
                                                    gettext(b"[Help]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                        as *const ::core::ffi::c_char
                                                })
                                                    as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        67 => {
                                            break '_stcsign;
                                        }
                                        115 => {
                                            break '_stcsign;
                                        }
                                        121 => {
                                            if *(*(*wp).w_buffer).b_p_ft as ::core::ffi::c_int
                                                != NUL
                                                && strlen((*(*wp).w_buffer).b_p_ft)
                                                    < (TMPLEN - 3 as ::core::ffi::c_int) as size_t
                                            {
                                                vim_snprintf(
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 70]>(
                                                    ),
                                                    b"[%s]\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    (*(*wp).w_buffer).b_p_ft,
                                                );
                                                str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        89 => {
                                            itemisflag = true_0 != 0;
                                            if *(*(*wp).w_buffer).b_p_ft as ::core::ffi::c_int
                                                != NUL
                                                && strlen((*(*wp).w_buffer).b_p_ft)
                                                    < (TMPLEN - 2 as ::core::ffi::c_int) as size_t
                                            {
                                                vim_snprintf(
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 70]>(
                                                    ),
                                                    b",%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                    (*(*wp).w_buffer).b_p_ft,
                                                );
                                                let mut t_3: *mut ::core::ffi::c_char =
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                                while *t_3 as ::core::ffi::c_int
                                                    != 0 as ::core::ffi::c_int
                                                {
                                                    *t_3 = toupper(
                                                        *t_3 as uint8_t as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_char;
                                                    t_3 = t_3.offset(1);
                                                }
                                                str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        119 | 87 => {
                                            itemisflag = true_0 != 0;
                                            if (*wp).w_onebuf_opt.wo_pvw != 0 {
                                                str = (if opt as ::core::ffi::c_int
                                                    == STL_PREVIEWFLAG_ALT as ::core::ffi::c_int
                                                {
                                                    b",PRV\0".as_ptr() as *const ::core::ffi::c_char
                                                } else {
                                                    gettext(b"[Preview]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                        as *const ::core::ffi::c_char
                                                })
                                                    as *mut ::core::ffi::c_char;
                                            }
                                            break 's_1848;
                                        }
                                        113 => {
                                            if bt_quickfix((*wp).w_buffer) {
                                                str = if !(*wp).w_llist_ref.is_null() {
                                                    gettext(msg_loclist)
                                                } else {
                                                    gettext(msg_qflist)
                                                };
                                            }
                                            break 's_1848;
                                        }
                                        109 | 77 => {
                                            itemisflag = true_0 != 0;
                                            match (opt as ::core::ffi::c_int
                                                == STL_MODIFIED_ALT as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                                + bufIsChanged((*wp).w_buffer) as ::core::ffi::c_int
                                                    * 2 as ::core::ffi::c_int
                                                + ((*(*wp).w_buffer).b_p_ma == 0)
                                                    as ::core::ffi::c_int
                                                    * 4 as ::core::ffi::c_int
                                            {
                                                2 => {
                                                    str = b"[+]\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                3 => {
                                                    str = b",+\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                4 => {
                                                    str = b"[-]\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                5 => {
                                                    str = b",-\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                6 => {
                                                    str = b"[+-]\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                7 => {
                                                    str = b",+-\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                _ => {}
                                            }
                                            break 's_1848;
                                        }
                                        36 | 35 => {
                                            let mut t_4: *mut ::core::ffi::c_char = fmt_p;
                                            while *fmt_p as ::core::ffi::c_int
                                                != opt as ::core::ffi::c_int
                                                && *fmt_p as ::core::ffi::c_int != NUL
                                            {
                                                fmt_p = fmt_p.offset(1);
                                            }
                                            if *fmt_p as ::core::ffi::c_int
                                                == opt as ::core::ffi::c_int
                                            {
                                                (*stl_items.offset(curitem as isize)).type_0 =
                                                    (if opt as ::core::ffi::c_int
                                                        == STL_HIGHLIGHT_COMB as ::core::ffi::c_int
                                                    {
                                                        HighlightCombining as ::core::ffi::c_int
                                                    } else {
                                                        Highlight as ::core::ffi::c_int
                                                    })
                                                        as C2Rust_Unnamed_15;
                                                (*stl_items.offset(curitem as isize)).start = out_p;
                                                (*stl_items.offset(curitem as isize)).minwid =
                                                    -syn_name2id_len(
                                                        t_4,
                                                        fmt_p.offset_from(t_4) as size_t,
                                                    );
                                                curitem += 1;
                                                fmt_p = fmt_p.offset(1);
                                            }
                                            continue 's_2297;
                                        }
                                        _ => {
                                            break 's_1848;
                                        }
                                    }
                                    num = byteval;
                                    if num == NL {
                                        num = 0 as ::core::ffi::c_int;
                                    } else if num == CAR
                                        && get_fileformat((*wp).w_buffer) == EOL_MAC
                                    {
                                        num = NL;
                                    }
                                    break 's_1848;
                                }
                                let mut l: ::core::ffi::c_int = ml_find_line_or_offset(
                                    (*wp).w_buffer,
                                    (*wp).w_cursor.lnum,
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    false_0 != 0,
                                );
                                num = if (*(*wp).w_buffer).b_ml.ml_flags & ML_EMPTY != 0
                                    || l < 0 as ::core::ffi::c_int
                                {
                                    0 as ::core::ffi::c_int
                                } else {
                                    l + 1 as ::core::ffi::c_int
                                        + (if State & MODE_INSERT as ::core::ffi::c_int
                                            == 0 as ::core::ffi::c_int
                                            && empty_line as ::core::ffi::c_int != 0
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            (*wp).w_cursor.col
                                        })
                                };
                                break 's_1848;
                            }
                            if !stcp.is_null() {
                                let mut fdc: ::core::ffi::c_int = if opt as ::core::ffi::c_int
                                    == STL_FOLDCOL as ::core::ffi::c_int
                                {
                                    compute_foldcolumn(wp, 0 as ::core::ffi::c_int)
                                } else {
                                    0 as ::core::ffi::c_int
                                };
                                let mut width: ::core::ffi::c_int = if opt as ::core::ffi::c_int
                                    == STL_FOLDCOL as ::core::ffi::c_int
                                {
                                    (fdc > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                                } else if opt as ::core::ffi::c_int
                                    == STL_SIGNCOL as ::core::ffi::c_int
                                {
                                    (*wp).w_scwidth
                                } else {
                                    1 as ::core::ffi::c_int
                                };
                                if width > 0 as ::core::ffi::c_int {
                                    foldsignitem = curitem;
                                    lnum = get_vim_var_nr(VV_LNUM) as linenr_T;
                                    if fdc > 0 as ::core::ffi::c_int {
                                        let mut fold_buf: [schar_T; 9] = [0; 9];
                                        fill_foldcolumn(
                                            wp,
                                            (*stcp).foldinfo,
                                            (*stcp).lnum,
                                            0 as ::core::ffi::c_int,
                                            fdc,
                                            get_vim_var_nr(VV_VIRTNUM) < 0 as varnumber_T,
                                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                            &raw mut (*stcp).fold_vcol as *mut colnr_T,
                                            &raw mut fold_buf as *mut schar_T,
                                        );
                                        (*stl_items.offset(curitem as isize)).minwid =
                                            -if use_cursor_line_highlight(wp, lnum)
                                                as ::core::ffi::c_int
                                                != 0
                                            {
                                                HLF_CLF as ::core::ffi::c_int
                                            } else {
                                                HLF_FC as ::core::ffi::c_int
                                            };
                                        let mut buflen: size_t = 0 as size_t;
                                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while i < fdc {
                                            buflen = buflen.wrapping_add(schar_get(
                                                (&raw mut buf_tmp as *mut ::core::ffi::c_char)
                                                    .offset(buflen as isize),
                                                fold_buf[i as usize],
                                            ));
                                            i += 1;
                                        }
                                    }
                                    let mut signlen: size_t = 0 as size_t;
                                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while i_0 < width {
                                        (*stl_items.offset(curitem as isize)).start =
                                            out_p.offset(signlen as isize);
                                        if fdc == 0 as ::core::ffi::c_int {
                                            let mut sattr: SignTextAttrs =
                                                *(*stcp).sattrs.offset(i_0 as isize);
                                            if sattr.text[0 as ::core::ffi::c_int as usize] != 0
                                                && get_vim_var_nr(VV_VIRTNUM) == 0 as varnumber_T
                                            {
                                                signlen = signlen.wrapping_add(describe_sign_text(
                                                    (&raw mut buf_tmp as *mut ::core::ffi::c_char)
                                                        .offset(signlen as isize),
                                                    &raw mut sattr.text as *mut schar_T,
                                                ));
                                                (*stl_items.offset(curitem as isize)).minwid =
                                                    -if (*stcp).sign_cul_id != 0 {
                                                        (*stcp).sign_cul_id
                                                    } else {
                                                        sattr.hl_id
                                                    };
                                            } else {
                                                let c2rust_fresh19 = signlen;
                                                signlen = signlen.wrapping_add(1);
                                                buf_tmp[c2rust_fresh19 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                                let c2rust_fresh20 = signlen;
                                                signlen = signlen.wrapping_add(1);
                                                buf_tmp[c2rust_fresh20 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                                buf_tmp[signlen as usize] =
                                                    NUL as ::core::ffi::c_char;
                                                (*stl_items.offset(curitem as isize)).minwid =
                                                    0 as ::core::ffi::c_int;
                                            }
                                        }
                                        let c2rust_fresh21 = curitem;
                                        curitem = curitem + 1;
                                        (*stl_items.offset(c2rust_fresh21 as isize)).type_0 =
                                            (if fdc > 0 as ::core::ffi::c_int {
                                                HighlightFold as ::core::ffi::c_int
                                            } else {
                                                HighlightSign as ::core::ffi::c_int
                                            })
                                                as C2Rust_Unnamed_15;
                                        i_0 += 1;
                                    }
                                    str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                }
                            }
                        }
                        (*stl_items.offset(curitem as isize)).start = out_p;
                        (*stl_items.offset(curitem as isize)).type_0 = Normal;
                        if !str.is_null() && *str as ::core::ffi::c_int != 0 {
                            let mut t_5: *mut ::core::ffi::c_char = str;
                            if itemisflag {
                                if *t_5.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != 0
                                    && *t_5.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != 0
                                    && (!prevchar_isitem
                                        && *t_5 as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                                        || prevchar_isflag as ::core::ffi::c_int != 0
                                            && *t_5 as ::core::ffi::c_int
                                                == ' ' as ::core::ffi::c_int)
                                {
                                    t_5 = t_5.offset(1);
                                }
                                prevchar_isflag = true_0 != 0;
                            }
                            let mut l_0: ::core::ffi::c_int = vim_strsize(t_5);
                            if l_0 > 0 as ::core::ffi::c_int {
                                prevchar_isitem = true_0 != 0;
                            }
                            if l_0 > maxwid_0 {
                                while l_0 >= maxwid_0 {
                                    l_0 -= ptr2cells(t_5);
                                    t_5 = t_5.offset(utfc_ptr2len(t_5) as isize);
                                }
                                if out_p >= out_end_p {
                                    break;
                                }
                                let c2rust_fresh22 = out_p;
                                out_p = out_p.offset(1);
                                *c2rust_fresh22 = '<' as ::core::ffi::c_char;
                            }
                            if minwid_0 > 0 as ::core::ffi::c_int {
                                while l_0 < minwid_0 && out_p < out_end_p {
                                    if l_0 + 1 as ::core::ffi::c_int == minwid_0
                                        && fillchar == '-' as schar_T
                                        && ascii_isdigit(*t_5 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        let c2rust_fresh23 = out_p;
                                        out_p = out_p.offset(1);
                                        *c2rust_fresh23 = ' ' as ::core::ffi::c_char;
                                    } else {
                                        schar_get_adv(&raw mut out_p, fillchar);
                                    }
                                    l_0 += 1;
                                }
                                minwid_0 = 0 as ::core::ffi::c_int;
                                if foldsignitem >= 0 as ::core::ffi::c_int {
                                    let mut offset: ptrdiff_t = out_p.offset_from(
                                        (*stl_items.offset(foldsignitem as isize)).start,
                                    );
                                    let mut i_1: ::core::ffi::c_int = foldsignitem;
                                    while i_1 < curitem {
                                        (*stl_items.offset(i_1 as isize)).start = (*stl_items
                                            .offset(i_1 as isize))
                                        .start
                                        .offset(offset as isize);
                                        i_1 += 1;
                                    }
                                }
                            } else {
                                minwid_0 *= -1 as ::core::ffi::c_int;
                            }
                            while *t_5 as ::core::ffi::c_int != 0 && out_p < out_end_p {
                                if fillable as ::core::ffi::c_int != 0
                                    && *t_5 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                    && (!ascii_isdigit(
                                        *t_5.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) || fillchar != '-' as schar_T)
                                {
                                    schar_get_adv(&raw mut out_p, fillchar);
                                } else {
                                    let c2rust_fresh24 = out_p;
                                    out_p = out_p.offset(1);
                                    *c2rust_fresh24 = *t_5;
                                }
                                t_5 = t_5.offset(1);
                            }
                            if foldsignitem >= 0 as ::core::ffi::c_int {
                                (*stl_items.offset(curitem as isize)).type_0 = Highlight;
                                (*stl_items.offset(curitem as isize)).start = out_p;
                                (*stl_items.offset(curitem as isize)).minwid =
                                    0 as ::core::ffi::c_int;
                            }
                            while l_0 < minwid_0 && out_p < out_end_p {
                                schar_get_adv(&raw mut out_p, fillchar);
                                l_0 += 1;
                            }
                        } else if num >= 0 as ::core::ffi::c_int {
                            if out_p.offset(20 as ::core::ffi::c_int as isize) > out_end_p {
                                break;
                            }
                            prevchar_isitem = true_0 != 0;
                            let mut nstr: [::core::ffi::c_char; 20] = [0; 20];
                            let mut t_6: *mut ::core::ffi::c_char =
                                &raw mut nstr as *mut ::core::ffi::c_char;
                            if opt as ::core::ffi::c_int == STL_VIRTCOL_ALT as ::core::ffi::c_int {
                                let c2rust_fresh25 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh25 = '-' as ::core::ffi::c_char;
                                minwid_0 -= 1;
                            }
                            let c2rust_fresh26 = t_6;
                            t_6 = t_6.offset(1);
                            *c2rust_fresh26 = '%' as ::core::ffi::c_char;
                            if zeropad {
                                let c2rust_fresh27 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh27 = '0' as ::core::ffi::c_char;
                            }
                            let c2rust_fresh28 = t_6;
                            t_6 = t_6.offset(1);
                            *c2rust_fresh28 = '*' as ::core::ffi::c_char;
                            let c2rust_fresh29 = t_6;
                            t_6 = t_6.offset(1);
                            *c2rust_fresh29 = (if base as ::core::ffi::c_uint
                                == kNumBaseHexadecimal as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                'X' as ::core::ffi::c_int
                            } else {
                                'd' as ::core::ffi::c_int
                            }) as ::core::ffi::c_char;
                            *t_6 = NUL as ::core::ffi::c_char;
                            let mut num_chars: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            let mut n_3: ::core::ffi::c_int = num;
                            while n_3 >= base as ::core::ffi::c_int {
                                num_chars += 1;
                                n_3 /= base as ::core::ffi::c_int;
                            }
                            if opt as ::core::ffi::c_int == STL_VIRTCOL_ALT as ::core::ffi::c_int {
                                num_chars += 1;
                            }
                            '_c2rust_label: {
                                if out_end_p >= out_p {
                                } else {
                                    __assert_fail(
                                        b"out_end_p >= out_p\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"/home/overlord/projects/neovim/neovim/src/nvim/statusline.c\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        1856 as ::core::ffi::c_uint,
                                        b"int build_stl_str_hl(win_T *, char *, size_t, char *, OptIndex, int, schar_T, int, stl_hlrec_t **, size_t *, StlClickRecord **, statuscol_T *)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            let mut remaining_buf_len: size_t =
                                (out_end_p.offset_from(out_p) as size_t).wrapping_add(1 as size_t);
                            if num_chars > maxwid_0 {
                                num_chars += 2 as ::core::ffi::c_int;
                                let mut n_4: ::core::ffi::c_int = num_chars - maxwid_0;
                                loop {
                                    let c2rust_fresh30 = num_chars;
                                    num_chars = num_chars - 1;
                                    if c2rust_fresh30 <= maxwid_0 {
                                        break;
                                    }
                                    num /= base as ::core::ffi::c_int;
                                }
                                let c2rust_fresh31 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh31 = '>' as ::core::ffi::c_char;
                                let c2rust_fresh32 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh32 = '%' as ::core::ffi::c_char;
                                *t_6 = *t_6.offset(-3 as ::core::ffi::c_int as isize);
                                t_6 = t_6.offset(1);
                                *t_6 = NUL as ::core::ffi::c_char;
                                out_p = out_p.offset(vim_snprintf_safelen(
                                    out_p,
                                    remaining_buf_len,
                                    &raw mut nstr as *mut ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    num,
                                    n_4,
                                ) as isize);
                            } else {
                                out_p = out_p.offset(vim_snprintf_safelen(
                                    out_p,
                                    remaining_buf_len,
                                    &raw mut nstr as *mut ::core::ffi::c_char,
                                    minwid_0,
                                    num,
                                ) as isize);
                            }
                        } else {
                            (*stl_items.offset(curitem as isize)).type_0 = Empty;
                        }
                        if num >= 0 as ::core::ffi::c_int
                            || !itemisflag && !str.is_null() && *str as ::core::ffi::c_int != 0
                        {
                            prevchar_isflag = false_0 != 0;
                        }
                        if opt as ::core::ffi::c_int == STL_VIM_EXPR as ::core::ffi::c_int {
                            let mut ptr__1: *mut *mut ::core::ffi::c_void =
                                &raw mut str as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__1);
                            *ptr__1 = NULL;
                            *ptr__1;
                        }
                        curitem += 1;
                        if left_align_num {
                            (*stl_items.offset(curitem as isize)).type_0 = Separate;
                            let c2rust_fresh33 = curitem;
                            curitem = curitem + 1;
                            let c2rust_lvalue_ptr_2 =
                                &raw mut (*stl_items.offset(c2rust_fresh33 as isize)).start;
                            *c2rust_lvalue_ptr_2 = out_p;
                        }
                    }
                }
            }
        }
    }
    *out_p = NUL as ::core::ffi::c_char;
    let mut outputlen: size_t = out_p.offset_from(out) as size_t;
    let mut itemcnt: ::core::ffi::c_int = curitem - evalstart;
    curitem = evalstart;
    if usefmt != fmt {
        xfree(usefmt as *mut ::core::ffi::c_void);
    }
    let mut width_0: ::core::ffi::c_int = vim_strsize(out);
    if maxwidth > 0 as ::core::ffi::c_int
        && width_0 > maxwidth
        && (stcp.is_null()
            || width_0
                > MAX_NUMBERWIDTH
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int)
    {
        let mut item_idx: ::core::ffi::c_int = evalstart;
        let mut trunc_p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if itemcnt == 0 as ::core::ffi::c_int {
            trunc_p = out;
        } else {
            trunc_p = (*stl_items.offset(item_idx as isize)).start;
            let mut i_2: ::core::ffi::c_int = evalstart;
            while i_2 < itemcnt + evalstart {
                if (*stl_items.offset(i_2 as isize)).type_0 as ::core::ffi::c_uint
                    == Trunc as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    trunc_p = (*stl_items.offset(i_2 as isize)).start;
                    item_idx = i_2;
                    break;
                } else {
                    i_2 += 1;
                }
            }
        }
        if width_0 - vim_strsize(trunc_p) >= maxwidth {
            trunc_p = out;
            width_0 = 0 as ::core::ffi::c_int;
            loop {
                width_0 += ptr2cells(trunc_p);
                if width_0 >= maxwidth {
                    break;
                }
                trunc_p = trunc_p.offset(utfc_ptr2len(trunc_p) as isize);
            }
            let mut i_3: ::core::ffi::c_int = evalstart;
            while i_3 < itemcnt + evalstart {
                if (*stl_items.offset(i_3 as isize)).start > trunc_p {
                    let mut j: ::core::ffi::c_int = i_3;
                    while j < itemcnt + evalstart {
                        if (*stl_items.offset(j as isize)).type_0 as ::core::ffi::c_uint
                            == ClickFunc as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let mut ptr__2: *mut *mut ::core::ffi::c_void =
                                &raw mut (*stl_items.offset(j as isize)).cmd
                                    as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__2);
                            *ptr__2 = NULL;
                            *ptr__2;
                        }
                        j += 1;
                    }
                    itemcnt = i_3;
                    break;
                } else {
                    i_3 += 1;
                }
            }
            let c2rust_fresh34 = trunc_p;
            trunc_p = trunc_p.offset(1);
            *c2rust_fresh34 = '>' as ::core::ffi::c_char;
            *trunc_p = NUL as ::core::ffi::c_char;
        } else {
            let mut end: *mut ::core::ffi::c_char = out.offset(outputlen as isize);
            let mut trunc_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while width_0 >= maxwidth {
                width_0 -= ptr2cells(trunc_p.offset(trunc_len as isize));
                trunc_len += utfc_ptr2len(trunc_p.offset(trunc_len as isize));
            }
            let mut trunc_end_p: *mut ::core::ffi::c_char = trunc_p.offset(trunc_len as isize);
            memmove(
                trunc_p.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                trunc_end_p as *const ::core::ffi::c_void,
                (end.offset_from(trunc_end_p) as size_t).wrapping_add(1 as size_t),
            );
            end = end.offset(
                -(trunc_end_p.offset_from(trunc_p.offset(1 as ::core::ffi::c_int as isize))
                    as size_t as isize),
            );
            *trunc_p = '<' as ::core::ffi::c_char;
            let mut item_offset: ::core::ffi::c_int = trunc_len - 1 as ::core::ffi::c_int;
            let mut i_4: ::core::ffi::c_int = item_idx;
            while i_4 < itemcnt + evalstart {
                if (*stl_items.offset(i_4 as isize)).start >= trunc_end_p {
                    (*stl_items.offset(i_4 as isize)).start = (*stl_items.offset(i_4 as isize))
                        .start
                        .offset(-(item_offset as isize));
                } else {
                    (*stl_items.offset(i_4 as isize)).start = trunc_p;
                }
                i_4 += 1;
            }
            if (width_0 + 1 as ::core::ffi::c_int) < maxwidth {
                trunc_p = end;
            }
            loop {
                width_0 += 1;
                if width_0 >= maxwidth {
                    break;
                }
                schar_get_adv(&raw mut trunc_p, fillchar);
                end = trunc_p;
            }
        }
        width_0 = maxwidth;
    } else if width_0 < maxwidth
        && outputlen
            .wrapping_add(((maxwidth - width_0) as size_t).wrapping_mul(schar_len(fillchar)))
            .wrapping_add(1 as size_t)
            < outlen
    {
        let mut num_separators: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_5: ::core::ffi::c_int = evalstart;
        while i_5 < itemcnt + evalstart {
            if (*stl_items.offset(i_5 as isize)).type_0 as ::core::ffi::c_uint
                == Separate as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *stl_separator_locations.offset(num_separators as isize) = i_5;
                num_separators += 1;
            }
            i_5 += 1;
        }
        if num_separators != 0 {
            let mut standard_spaces: ::core::ffi::c_int = (maxwidth - width_0) / num_separators;
            let mut final_spaces: ::core::ffi::c_int =
                maxwidth - width_0 - standard_spaces * (num_separators - 1 as ::core::ffi::c_int);
            let mut l_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while l_1 < num_separators {
                let mut dislocation: ::core::ffi::c_int =
                    if l_1 == num_separators - 1 as ::core::ffi::c_int {
                        final_spaces
                    } else {
                        standard_spaces
                    };
                dislocation *= schar_len(fillchar) as ::core::ffi::c_int;
                let mut start: *mut ::core::ffi::c_char = (*stl_items
                    .offset(*stl_separator_locations.offset(l_1 as isize) as isize))
                .start;
                let mut seploc: *mut ::core::ffi::c_char = start.offset(dislocation as isize);
                memmove(
                    seploc as *mut ::core::ffi::c_void,
                    start as *const ::core::ffi::c_void,
                    strlen(start).wrapping_add(1 as size_t),
                );
                let mut s: *mut ::core::ffi::c_char = start;
                while s < seploc {
                    schar_get_adv(&raw mut s, fillchar);
                }
                let mut item_idx_0: ::core::ffi::c_int =
                    *stl_separator_locations.offset(l_1 as isize) + 1 as ::core::ffi::c_int;
                while item_idx_0 < itemcnt + evalstart {
                    (*stl_items.offset(item_idx_0 as isize)).start = (*stl_items
                        .offset(item_idx_0 as isize))
                    .start
                    .offset(dislocation as isize);
                    item_idx_0 += 1;
                }
                l_1 += 1;
            }
            width_0 = maxwidth;
        }
    }
    if !hltab.is_null() {
        *hltab = stl_hltab;
        let mut sp: *mut stl_hlrec_t = stl_hltab;
        let mut l_2: ::core::ffi::c_int = evalstart;
        while l_2 < itemcnt + evalstart {
            if (*stl_items.offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*stl_items.offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                    == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*stl_items.offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                    == HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*stl_items.offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                    == HighlightSign as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*sp).start = (*stl_items.offset(l_2 as isize)).start;
                (*sp).userhl = (*stl_items.offset(l_2 as isize)).minwid;
                let mut type_0: ::core::ffi::c_uint =
                    (*stl_items.offset(l_2 as isize)).type_0 as ::core::ffi::c_uint;
                (*sp).item = (if type_0
                    == HighlightSign as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    STL_SIGNCOL as ::core::ffi::c_int
                } else if type_0 == HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint {
                    STL_FOLDCOL as ::core::ffi::c_int
                } else if type_0 == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    STL_HIGHLIGHT_COMB as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as StlFlag;
                sp = sp.offset(1);
            }
            l_2 += 1;
        }
        (*sp).start = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*sp).userhl = 0 as ::core::ffi::c_int;
    }
    if !hltab_len.is_null() {
        *hltab_len = itemcnt as size_t;
    }
    if !tabtab.is_null() {
        *tabtab = stl_tabtab;
        let mut cur_tab_rec: *mut StlClickRecord = stl_tabtab;
        let mut l_3: ::core::ffi::c_int = evalstart;
        while l_3 < itemcnt + evalstart {
            if (*stl_items.offset(l_3 as isize)).type_0 as ::core::ffi::c_uint
                == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*cur_tab_rec).start = (*stl_items.offset(l_3 as isize)).start;
                if (*stl_items.offset(l_3 as isize)).minwid == 0 as ::core::ffi::c_int {
                    (*cur_tab_rec).def.type_0 = kStlClickDisabled;
                    (*cur_tab_rec).def.tabnr = 0 as ::core::ffi::c_int;
                } else {
                    let mut tabnr: ::core::ffi::c_int = (*stl_items.offset(l_3 as isize)).minwid;
                    if (*stl_items.offset(l_3 as isize)).minwid > 0 as ::core::ffi::c_int {
                        (*cur_tab_rec).def.type_0 = kStlClickTabSwitch;
                    } else {
                        (*cur_tab_rec).def.type_0 = kStlClickTabClose;
                        tabnr = -tabnr;
                    }
                    (*cur_tab_rec).def.tabnr = tabnr;
                }
                (*cur_tab_rec).def.func = ::core::ptr::null_mut::<::core::ffi::c_char>();
                cur_tab_rec = cur_tab_rec.offset(1);
            } else if (*stl_items.offset(l_3 as isize)).type_0 as ::core::ffi::c_uint
                == ClickFunc as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*cur_tab_rec).start = (*stl_items.offset(l_3 as isize)).start;
                (*cur_tab_rec).def.type_0 = kStlClickFuncRun;
                (*cur_tab_rec).def.tabnr = (*stl_items.offset(l_3 as isize)).minwid;
                (*cur_tab_rec).def.func = (*stl_items.offset(l_3 as isize)).cmd;
                cur_tab_rec = cur_tab_rec.offset(1);
            }
            l_3 += 1;
        }
        (*cur_tab_rec).start = ::core::ptr::null::<::core::ffi::c_char>();
        (*cur_tab_rec).def.type_0 = kStlClickDisabled;
        (*cur_tab_rec).def.tabnr = 0 as ::core::ffi::c_int;
        (*cur_tab_rec).def.func = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    redraw_not_allowed = save_redraw_not_allowed;
    if opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int
        && did_emsg > did_emsg_before
    {
        set_option_direct(
            opt_idx,
            get_option_default(opt_idx, opt_scope),
            opt_scope,
            SID_ERROR,
        );
    }
    KeyTyped = save_KeyTyped;
    return width_0;
}
pub const TMPLEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
