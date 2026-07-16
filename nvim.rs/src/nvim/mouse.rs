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
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    static mut p_ch: OptInt;
    static mut p_mousem: *mut ::core::ffi::c_char;
    static mut p_mousescroll_vert: OptInt;
    static mut p_mousescroll_hor: OptInt;
    static mut p_sel: *mut ::core::ffi::c_char;
    static mut p_smd: ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn set_leftcol(leftcol: colnr_T) -> bool;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor: bool) -> bool;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn redraw_statuslines();
    fn undisplay_dollar();
    fn start_arrow(end_insert_pos: *mut pos_T);
    fn set_can_cindent(val: bool);
    fn call_vim_function(
        func: *const ::core::ffi::c_char,
        argc: ::core::ffi::c_int,
        argv: *mut typval_T,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn eval_has_provider(feat: *const ::core::ffi::c_char, throw_if_fast: bool) -> bool;
    static mut msg_grid: ScreenGrid;
    static mut msg_grid_pos: ::core::ffi::c_int;
    fn siemsg(s: *const ::core::ffi::c_char, ...);
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_clear(tv: *mut typval_T);
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tabpage_close(forceit: ::core::ffi::c_int);
    fn tabpage_close_other(tp: *mut tabpage_T, forceit: ::core::ffi::c_int);
    fn tabpage_new();
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn closeFold(pos: pos_T, count: ::core::ffi::c_int);
    fn openFold(pos: pos_T, count: ::core::ffi::c_int);
    fn AppendCharToRedobuff(c: ::core::ffi::c_int);
    fn stuffReadbuff(s: *const ::core::ffi::c_char);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn stuffnumReadbuff(n: ::core::ffi::c_int);
    fn safe_vgetc() -> ::core::ffi::c_int;
    fn vpeekc() -> ::core::ffi::c_int;
    fn vungetc(c: ::core::ffi::c_int);
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut mod_mask: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut mode_displayed: bool;
    static mut mouse_grid: ::core::ffi::c_int;
    static mut mouse_row: ::core::ffi::c_int;
    static mut mouse_col: ::core::ffi::c_int;
    static mut mouse_past_bottom: bool;
    static mut mouse_past_eol: bool;
    static mut mouse_dragging: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut topframe: *mut frame_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut VIsual_select: bool;
    static mut VIsual_reselect: ::core::ffi::c_int;
    static mut VIsual_mode: ::core::ffi::c_int;
    static mut where_paste_started: pos_T;
    static mut State: ::core::ffi::c_int;
    static mut restart_edit: ::core::ffi::c_int;
    static mut msg_silent: ::core::ffi::c_int;
    static mut KeyStuffed: ::core::ffi::c_int;
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut cmdwin_win: *mut win_T;
    fn grid_adjust(
        grid: *mut GridView,
        row_off: *mut ::core::ffi::c_int,
        col_off: *mut ::core::ffi::c_int,
    ) -> *mut ScreenGrid;
    fn get_win_by_grid_handle(handle: handle_T) -> *mut win_T;
    fn get_mouse_button(
        code: ::core::ffi::c_int,
        is_click: *mut bool,
        is_drag: *mut bool,
    ) -> ::core::ffi::c_int;
    static utf8len_tab: [uint8_t; 256];
    fn mb_get_class(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int;
    fn inc(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn show_popupmenu();
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_col_off2(wp: *mut win_T) -> ::core::ffi::c_int;
    fn scroll_redraw(up: ::core::ffi::c_int, count: linenr_T);
    fn check_topfill(wp: *mut win_T, down: bool);
    fn pagescroll(dir: Direction, count: ::core::ffi::c_int, half: bool) -> ::core::ffi::c_int;
    fn end_visual_mode();
    fn prep_redo(
        regname: ::core::ffi::c_int,
        num: ::core::ffi::c_int,
        cmd1: ::core::ffi::c_int,
        cmd2: ::core::ffi::c_int,
        cmd3: ::core::ffi::c_int,
        cmd4: ::core::ffi::c_int,
        cmd5: ::core::ffi::c_int,
    );
    fn clearop(oap: *mut oparg_T);
    fn clearopbeep(oap: *mut oparg_T);
    fn nv_scroll_line(cap: *mut cmdarg_T);
    fn may_start_select(c: ::core::ffi::c_int);
    fn clear_oparg(oap: *mut oparg_T);
    fn get_scrolloff_value(wp: *mut win_T) -> int64_t;
    fn win_chartabsize(
        wp: *mut win_T,
        p: *mut ::core::ffi::c_char,
        col: colnr_T,
    ) -> ::core::ffi::c_int;
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
    fn getvcols(
        wp: *mut win_T,
        pos1: *mut pos_T,
        pos2: *mut pos_T,
        left: *mut colnr_T,
        right: *mut colnr_T,
    );
    fn win_may_fill(wp: *mut win_T) -> bool;
    fn win_get_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn plines_win(wp: *mut win_T, lnum: linenr_T, limit_winheight: bool) -> ::core::ffi::c_int;
    fn plines_win_nofill(
        wp: *mut win_T,
        lnum: linenr_T,
        limit_winheight: bool,
    ) -> ::core::ffi::c_int;
    static mut pum_grid: ScreenGrid;
    fn pum_visible() -> bool;
    fn yank_register_mline(regname: ::core::ffi::c_int, reg: *mut *mut yankreg_T) -> bool;
    fn insert_reg(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        literally_arg: bool,
    ) -> ::core::ffi::c_int;
    fn do_put(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    );
    fn findmatch(oap: *mut oparg_T, initc: ::core::ffi::c_int) -> *mut pos_T;
    static mut tab_page_click_defs: *mut StlClickDefinition;
    fn virtual_active(wp: *mut win_T) -> bool;
    fn stl_connected(wp: *mut win_T) -> bool;
    fn ui_flush();
    fn ui_check_mouse();
    fn ui_mouse_has(mode: ::core::ffi::c_int) -> bool;
    fn ui_cursor_shape();
    fn ui_comp_mouse_focus(row: ::core::ffi::c_int, col: ::core::ffi::c_int) -> *mut ScreenGrid;
    fn win_fdccol_count(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn find_tabpage(n: ::core::ffi::c_int) -> *mut tabpage_T;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn goto_tabpage(n: ::core::ffi::c_int);
    fn tabpage_move(nr: ::core::ffi::c_int);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_drag_status_line(dragwin_0: *mut win_T, offset: ::core::ffi::c_int);
    fn win_drag_vsep_line(dragwin_0: *mut win_T, offset: ::core::ffi::c_int);
    fn global_stl_height() -> ::core::ffi::c_int;
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type size_t = usize;
pub type time_t = __time_t;
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
pub type disptick_T = uint64_t;
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
pub type Timestamp = uint64_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_13; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_15 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_15 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_15 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_15 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_15 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_15 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_15 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_15 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_15 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_15 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_15 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_15 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_15 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_15 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub type key_extra = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdarg_T {
    pub oap: *mut oparg_T,
    pub prechar: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub nchar: ::core::ffi::c_int,
    pub nchar_composing: [::core::ffi::c_char; 32],
    pub nchar_len: ::core::ffi::c_int,
    pub extra_char: ::core::ffi::c_int,
    pub opcount: ::core::ffi::c_int,
    pub count0: ::core::ffi::c_int,
    pub count1: ::core::ffi::c_int,
    pub arg: ::core::ffi::c_int,
    pub retval: ::core::ffi::c_int,
    pub searchbuf: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MOUSE_STATUSCOL: C2Rust_Unnamed_16 = 4096;
pub const MOUSE_WINBAR: C2Rust_Unnamed_16 = 2048;
pub const MOUSE_FOLD_OPEN: C2Rust_Unnamed_16 = 1024;
pub const MOUSE_FOLD_CLOSE: C2Rust_Unnamed_16 = 512;
pub const CURSOR_MOVED: C2Rust_Unnamed_16 = 256;
pub const IN_OTHER_WIN: C2Rust_Unnamed_16 = 8;
pub const IN_SEP_LINE: C2Rust_Unnamed_16 = 4;
pub const IN_STATUS_LINE: C2Rust_Unnamed_16 = 2;
pub const IN_BUFFER: C2Rust_Unnamed_16 = 1;
pub const IN_UNKNOWN: C2Rust_Unnamed_16 = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MOUSE_RELEASED: C2Rust_Unnamed_17 = 32;
pub const MOUSE_MAY_STOP_VIS: C2Rust_Unnamed_17 = 16;
pub const MOUSE_SETPOS: C2Rust_Unnamed_17 = 8;
pub const MOUSE_DID_MOVE: C2Rust_Unnamed_17 = 4;
pub const MOUSE_MAY_VIS: C2Rust_Unnamed_17 = 2;
pub const MOUSE_FOCUS: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MOUSE_X2: C2Rust_Unnamed_18 = 1024;
pub const MOUSE_X1: C2Rust_Unnamed_18 = 768;
pub const MOUSE_RELEASE: C2Rust_Unnamed_18 = 3;
pub const MOUSE_RIGHT: C2Rust_Unnamed_18 = 2;
pub const MOUSE_MIDDLE: C2Rust_Unnamed_18 = 1;
pub const MOUSE_LEFT: C2Rust_Unnamed_18 = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_19 = -2;
pub const MSCR_LEFT: C2Rust_Unnamed_19 = -1;
pub const MSCR_UP: C2Rust_Unnamed_19 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_19 = 0;
pub const PUT_CURSEND: C2Rust_Unnamed_20 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_20 = 1;
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
pub const OP_NOP: C2Rust_Unnamed_21 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
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
pub type CSType = bool;
pub const kCharsizeFast: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_20 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_20 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_20 = 16;
pub const PUT_LINE: C2Rust_Unnamed_20 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_20 = 4;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const OP_NR_SUB: C2Rust_Unnamed_21 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_21 = 28;
pub const OP_FUNCTION: C2Rust_Unnamed_21 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_21 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_21 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_21 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_21 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_21 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_21 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_21 = 20;
pub const OP_FOLD: C2Rust_Unnamed_21 = 19;
pub const OP_APPEND: C2Rust_Unnamed_21 = 18;
pub const OP_INSERT: C2Rust_Unnamed_21 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_21 = 16;
pub const OP_ROT13: C2Rust_Unnamed_21 = 15;
pub const OP_JOIN_NS: C2Rust_Unnamed_21 = 14;
pub const OP_JOIN: C2Rust_Unnamed_21 = 13;
pub const OP_LOWER: C2Rust_Unnamed_21 = 12;
pub const OP_UPPER: C2Rust_Unnamed_21 = 11;
pub const OP_COLON: C2Rust_Unnamed_21 = 10;
pub const OP_FORMAT: C2Rust_Unnamed_21 = 9;
pub const OP_INDENT: C2Rust_Unnamed_21 = 8;
pub const OP_TILDE: C2Rust_Unnamed_21 = 7;
pub const OP_FILTER: C2Rust_Unnamed_21 = 6;
pub const OP_RSHIFT: C2Rust_Unnamed_21 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_21 = 4;
pub const OP_CHANGE: C2Rust_Unnamed_21 = 3;
pub const OP_YANK: C2Rust_Unnamed_21 = 2;
pub const OP_DELETE: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_22 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
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
pub const MOUSE_VISUAL: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MOD_MASK_META: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MOD_MASK_MULTI_CLICK: ::core::ffi::c_int =
    MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;
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
static mut orig_topline: linenr_T = 0 as linenr_T;
static mut orig_topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn get_mouse_class(mut p: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    if utf8len_tab[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
        as ::core::ffi::c_int
        > 1 as ::core::ffi::c_int
    {
        return mb_get_class(p);
    }
    let c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
    if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if vim_iswordc(c) {
        return 2 as ::core::ffi::c_int;
    }
    if c != NUL
        && !vim_strchr(b"-+*/%<>&|^!=\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
    {
        return 1 as ::core::ffi::c_int;
    }
    return c;
}
unsafe extern "C" fn find_start_of_word(mut pos: *mut pos_T) {
    let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
    let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
    while (*pos).col > 0 as ::core::ffi::c_int {
        let mut col: ::core::ffi::c_int =
            (*pos).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        col -= utf_head_off(line, line.offset(col as isize));
        if get_mouse_class(line.offset(col as isize)) != cclass {
            break;
        }
        (*pos).col = col as colnr_T;
    }
}
unsafe extern "C" fn find_end_of_word(mut pos: *mut pos_T) {
    let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
    if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && (*pos).col > 0 as ::core::ffi::c_int
    {
        (*pos).col -= 1;
        (*pos).col -= utf_head_off(line, line.offset((*pos).col as isize));
    }
    let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
    while *line.offset((*pos).col as isize) as ::core::ffi::c_int != NUL {
        let mut col: ::core::ffi::c_int =
            (*pos).col as ::core::ffi::c_int + utfc_ptr2len(line.offset((*pos).col as isize));
        if get_mouse_class(line.offset(col as isize)) != cclass {
            if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                (*pos).col = col as colnr_T;
            }
            break;
        } else {
            (*pos).col = col as colnr_T;
        }
    }
}
unsafe extern "C" fn move_tab_to_mouse() {
    let mut tabnr: ::core::ffi::c_int = (*tab_page_click_defs.offset(mouse_col as isize)).tabnr;
    if tabnr <= 0 as ::core::ffi::c_int {
        tabpage_move(9999 as ::core::ffi::c_int);
    } else if tabnr < tabpage_index(curtab) {
        tabpage_move(tabnr - 1 as ::core::ffi::c_int);
    } else {
        tabpage_move(tabnr);
    };
}
unsafe extern "C" fn mouse_tab_close(mut c1: ::core::ffi::c_int) {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if c1 == 999 as ::core::ffi::c_int {
        tp = curtab;
    } else {
        tp = find_tabpage(c1);
    }
    if tp == curtab {
        if !(*first_tabpage).tp_next.is_null() {
            tabpage_close(false_0);
        }
    } else if !tp.is_null() {
        tabpage_close_other(tp, false_0);
    }
}
static mut got_click: bool = false_0 != 0;
unsafe extern "C" fn call_click_def_func(
    mut click_defs: *mut StlClickDefinition,
    mut col: ::core::ffi::c_int,
    mut which_button: ::core::ffi::c_int,
) {
    let mut c2rust_lvalue: [::core::ffi::c_char; 5] = [
        (if mod_mask & MOD_MASK_SHIFT != 0 {
            's' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask & MOD_MASK_CTRL != 0 {
            'c' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask & MOD_MASK_ALT != 0 {
            'a' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask & MOD_MASK_META != 0 {
            'm' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ];
    let mut argv: [typval_T; 4] = [
        typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_number: (*click_defs.offset(col as isize)).tabnr as varnumber_T,
            },
        },
        typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_number: (if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                    4 as ::core::ffi::c_int
                } else if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                    3 as ::core::ffi::c_int
                } else if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as varnumber_T,
            },
        },
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_string: (if which_button == MOUSE_LEFT as ::core::ffi::c_int {
                    b"l\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
                    b"r\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
                    b"m\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_X1 as ::core::ffi::c_int {
                    b"x1\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_X2 as ::core::ffi::c_int {
                    b"x2\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"?\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            },
        },
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_string: &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
            },
        },
    ];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    call_vim_function(
        (*click_defs.offset(col as isize)).func,
        ::core::mem::size_of::<[typval_T; 4]>()
            .wrapping_div(::core::mem::size_of::<typval_T>())
            .wrapping_div(
                (::core::mem::size_of::<[typval_T; 4]>()
                    .wrapping_rem(::core::mem::size_of::<typval_T>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    tv_clear(&raw mut rettv);
    got_click = false_0 != 0;
}
unsafe extern "C" fn get_fpos_of_mouse(mut mpos: *mut pos_T) -> ::core::ffi::c_int {
    let mut grid: ::core::ffi::c_int = mouse_grid;
    let mut row: ::core::ffi::c_int = mouse_row;
    let mut col: ::core::ffi::c_int = mouse_col;
    if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
    if wp.is_null() {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    let mut winrow: ::core::ffi::c_int = row;
    let mut wincol: ::core::ffi::c_int = col;
    let mut below_buffer: bool =
        mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut (*mpos).lnum);
    if !below_buffer
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
        && (if (*wp).w_onebuf_opt.wo_rl != 0 {
            (wincol >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
        } else {
            (wincol < win_col_off(wp)) as ::core::ffi::c_int
        }) != 0
    {
        return MOUSE_STATUSCOL as ::core::ffi::c_int;
    }
    if winrow >= (*wp).w_view_height + (*wp).w_status_height {
        if mouse_grid <= 1 as ::core::ffi::c_int
            && (mouse_row as OptInt) < Rows as OptInt - p_ch
            && mouse_row as OptInt >= Rows as OptInt - p_ch - global_stl_height() as OptInt
        {
            return IN_STATUS_LINE as ::core::ffi::c_int;
        }
        return IN_UNKNOWN as ::core::ffi::c_int;
    } else if winrow >= (*wp).w_view_height {
        return IN_STATUS_LINE as ::core::ffi::c_int;
    }
    if winrow < 0 as ::core::ffi::c_int && winrow + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int
    {
        return MOUSE_WINBAR as ::core::ffi::c_int;
    }
    if wincol >= (*wp).w_view_width {
        return IN_SEP_LINE as ::core::ffi::c_int;
    }
    if wp != curwin || below_buffer as ::core::ffi::c_int != 0 {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    (*mpos).col = vcol2col(wp, (*mpos).lnum, col as colnr_T, &raw mut (*mpos).coladd);
    return IN_BUFFER as ::core::ffi::c_int;
}
unsafe extern "C" fn do_popup(
    mut which_button: ::core::ffi::c_int,
    mut m_pos_flag: ::core::ffi::c_int,
    mut m_pos: pos_T,
) -> ::core::ffi::c_int {
    let mut jump_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if strcmp(
        p_mousem,
        b"popup_setpos\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if VIsual_active {
            if m_pos_flag != IN_BUFFER as ::core::ffi::c_int {
                jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
            } else if VIsual_mode == 'V' as ::core::ffi::c_int {
                if (*curwin).w_cursor.lnum <= VIsual.lnum
                    && (m_pos.lnum < (*curwin).w_cursor.lnum || VIsual.lnum < m_pos.lnum)
                    || VIsual.lnum < (*curwin).w_cursor.lnum
                        && (m_pos.lnum < VIsual.lnum || (*curwin).w_cursor.lnum < m_pos.lnum)
                {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            } else if ltoreq((*curwin).w_cursor, VIsual) as ::core::ffi::c_int != 0
                && (lt(m_pos, (*curwin).w_cursor) as ::core::ffi::c_int != 0
                    || lt(VIsual, m_pos) as ::core::ffi::c_int != 0)
                || lt(VIsual, (*curwin).w_cursor) as ::core::ffi::c_int != 0
                    && (lt(m_pos, VIsual) as ::core::ffi::c_int != 0
                        || lt((*curwin).w_cursor, m_pos) as ::core::ffi::c_int != 0)
            {
                jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
            } else if VIsual_mode == Ctrl_V {
                let mut leftcol: colnr_T = 0;
                let mut rightcol: colnr_T = 0;
                getvcols(
                    curwin,
                    &raw mut (*curwin).w_cursor,
                    &raw mut VIsual,
                    &raw mut leftcol,
                    &raw mut rightcol,
                );
                getvcol(
                    curwin,
                    &raw mut m_pos,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut m_pos.col,
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                if m_pos.col < leftcol || m_pos.col > rightcol {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            }
        } else {
            jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
        }
    }
    if jump_flags != 0 {
        jump_flags = jump_to_mouse(jump_flags, ::core::ptr::null_mut::<bool>(), which_button);
        redraw_curbuf_later(if VIsual_active as ::core::ffi::c_int != 0 {
            UPD_INVERTED as ::core::ffi::c_int
        } else {
            UPD_VALID as ::core::ffi::c_int
        });
        update_screen();
        setcursor();
        ui_flush();
    }
    show_popupmenu();
    got_click = false_0 != 0;
    return jump_flags;
}
#[no_mangle]
pub unsafe extern "C" fn do_mouse(
    mut oap: *mut oparg_T,
    mut c: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut fixindent: bool,
) -> bool {
    let mut which_button: ::core::ffi::c_int = 0;
    let mut is_click: bool = false;
    let mut is_drag: bool = false;
    static mut in_tab_line: bool = false_0 != 0;
    static mut orig_cursor: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    loop {
        which_button = get_mouse_button(
            (-c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint)
                as ::core::ffi::c_int,
            &raw mut is_click,
            &raw mut is_drag,
        );
        if !is_drag {
            break;
        }
        if !(KeyStuffed == 0 && vpeekc() != NUL) {
            break;
        }
        let mut nc: ::core::ffi::c_int = 0;
        let mut save_mouse_grid: ::core::ffi::c_int = mouse_grid;
        let mut save_mouse_row: ::core::ffi::c_int = mouse_row;
        let mut save_mouse_col: ::core::ffi::c_int = mouse_col;
        nc = safe_vgetc();
        if c == nc {
            continue;
        }
        vungetc(nc);
        mouse_grid = save_mouse_grid;
        mouse_row = save_mouse_row;
        mouse_col = save_mouse_col;
        break;
    }
    if c == -(253 as ::core::ffi::c_int
        + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return false_0 != 0;
    }
    if is_click {
        got_click = true_0 != 0;
    } else {
        if !got_click {
            return false_0 != 0;
        }
        if !is_drag {
            got_click = false_0 != 0;
            if in_tab_line {
                in_tab_line = false_0 != 0;
                return false_0 != 0;
            }
        }
    }
    if is_click as ::core::ffi::c_int != 0
        && mod_mask & MOD_MASK_CTRL != 0
        && which_button == MOUSE_RIGHT as ::core::ffi::c_int
    {
        if State & MODE_INSERT as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(Ctrl_O);
        }
        if count > 1 as ::core::ffi::c_int {
            stuffnumReadbuff(count);
        }
        stuffcharReadbuff(Ctrl_T);
        got_click = false_0 != 0;
        return false_0 != 0;
    }
    if mod_mask & MOD_MASK_CTRL != 0 && which_button != MOUSE_LEFT as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT | MOD_MASK_META) != 0
        && (!is_click
            || mod_mask & MOD_MASK_MULTI_CLICK != 0
            || which_button == MOUSE_MIDDLE as ::core::ffi::c_int)
        && !(mod_mask & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0
            && mouse_model_popup() as ::core::ffi::c_int != 0
            && which_button == MOUSE_LEFT as ::core::ffi::c_int)
        && !(mod_mask & MOD_MASK_ALT != 0
            && !mouse_model_popup()
            && which_button == MOUSE_RIGHT as ::core::ffi::c_int)
    {
        return false_0 != 0;
    }
    if !is_click && which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut regname: ::core::ffi::c_int = if !oap.is_null() {
        (*oap).regname
    } else {
        0 as ::core::ffi::c_int
    };
    if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        if State == MODE_NORMAL as ::core::ffi::c_int {
            if !oap.is_null() && (*oap).op_type != OP_NOP as ::core::ffi::c_int {
                clearopbeep(oap);
                return false_0 != 0;
            }
            if VIsual_active {
                if VIsual_select {
                    stuffcharReadbuff(Ctrl_G);
                    stuffReadbuff(b"\"+p\0".as_ptr() as *const ::core::ffi::c_char);
                } else {
                    stuffcharReadbuff('y' as ::core::ffi::c_int);
                    stuffcharReadbuff(
                        -(253 as ::core::ffi::c_int
                            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
                    );
                }
                return false_0 != 0;
            }
        } else if State & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if State & MODE_INSERT as ::core::ffi::c_int != 0 {
            if regname == '.' as ::core::ffi::c_int {
                insert_reg(regname, ::core::ptr::null_mut::<yankreg_T>(), true_0 != 0);
            } else {
                if regname == 0 as ::core::ffi::c_int
                    && eval_has_provider(
                        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                        false_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    regname = '*' as ::core::ffi::c_int;
                }
                let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
                if State & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && !yank_register_mline(regname, &raw mut reg)
                {
                    insert_reg(regname, reg, true_0 != 0);
                } else {
                    do_put(
                        regname,
                        reg,
                        BACKWARD as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        (if fixindent as ::core::ffi::c_int != 0 {
                            PUT_FIXINDENT as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) | PUT_CURSEND as ::core::ffi::c_int,
                    );
                    AppendCharToRedobuff(Ctrl_R);
                    AppendCharToRedobuff(if fixindent as ::core::ffi::c_int != 0 {
                        Ctrl_P
                    } else {
                        Ctrl_O
                    });
                    AppendCharToRedobuff(if regname == 0 as ::core::ffi::c_int {
                        '"' as ::core::ffi::c_int
                    } else {
                        regname
                    });
                }
            }
            return false_0 != 0;
        }
    }
    let mut jump_flags: ::core::ffi::c_int = if is_click as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int
    };
    let mut old_curwin: *mut win_T = curwin;
    if !tab_page_click_defs.is_null() {
        if mouse_grid <= 1 as ::core::ffi::c_int
            && mouse_row == 0 as ::core::ffi::c_int
            && (*firstwin).w_winrow > 0 as ::core::ffi::c_int
        {
            if is_drag {
                if in_tab_line {
                    move_tab_to_mouse();
                }
                return false_0 != 0;
            }
            if is_click as ::core::ffi::c_int != 0
                && cmdwin_type == 0 as ::core::ffi::c_int
                && mouse_col < Columns
            {
                let mut tabnr: ::core::ffi::c_int =
                    (*tab_page_click_defs.offset(mouse_col as isize)).tabnr;
                in_tab_line = true_0 != 0;
                's_464: {
                    match (*tab_page_click_defs.offset(mouse_col as isize)).type_0
                        as ::core::ffi::c_uint
                    {
                        1 => {
                            if which_button != MOUSE_MIDDLE as ::core::ffi::c_int {
                                if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                                    end_visual_mode();
                                    tabpage_new();
                                    tabpage_move(if tabnr == 0 as ::core::ffi::c_int {
                                        9999 as ::core::ffi::c_int
                                    } else {
                                        tabnr - 1 as ::core::ffi::c_int
                                    });
                                } else {
                                    goto_tabpage(tabnr);
                                    if curwin != old_curwin {
                                        end_visual_mode();
                                    }
                                }
                                break 's_464;
                            }
                        }
                        2 => {}
                        3 => {
                            call_click_def_func(tab_page_click_defs, mouse_col, which_button);
                            break 's_464;
                        }
                        0 | _ => {
                            break 's_464;
                        }
                    }
                    mouse_tab_close(tabnr);
                }
            }
            return true_0 != 0;
        } else if is_drag as ::core::ffi::c_int != 0 && in_tab_line as ::core::ffi::c_int != 0 {
            move_tab_to_mouse();
            return false_0 != 0;
        }
    }
    let mut m_pos_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut m_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    if mouse_model_popup() {
        m_pos_flag = get_fpos_of_mouse(&raw mut m_pos);
        if m_pos_flag
            & (IN_STATUS_LINE as ::core::ffi::c_int
                | MOUSE_WINBAR as ::core::ffi::c_int
                | MOUSE_STATUSCOL as ::core::ffi::c_int)
            == 0
            && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            && mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
        {
            if !is_click {
                return false_0 != 0;
            }
            return do_popup(which_button, m_pos_flag, m_pos) & CURSOR_MOVED as ::core::ffi::c_int
                != 0;
        }
        if m_pos_flag
            & (IN_STATUS_LINE as ::core::ffi::c_int
                | MOUSE_WINBAR as ::core::ffi::c_int
                | MOUSE_STATUSCOL as ::core::ffi::c_int)
            == 0
            && (which_button == MOUSE_LEFT as ::core::ffi::c_int
                && mod_mask & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0)
        {
            which_button = MOUSE_RIGHT as ::core::ffi::c_int;
            mod_mask &= !MOD_MASK_SHIFT;
        }
    }
    let mut end_visual: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    let mut start_visual: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    let mut mouse_can_visual: bool = ui_mouse_has(MOUSE_VISUAL);
    if State & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int) != 0
        && mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
    {
        if which_button == MOUSE_LEFT as ::core::ffi::c_int
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click {
                if VIsual_active {
                    jump_flags |= MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            } else {
                jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
            }
        } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click as ::core::ffi::c_int != 0 && VIsual_active as ::core::ffi::c_int != 0 {
                if lt((*curwin).w_cursor, VIsual) {
                    start_visual = (*curwin).w_cursor;
                    end_visual = VIsual;
                } else {
                    start_visual = VIsual;
                    end_visual = (*curwin).w_cursor;
                }
            }
            jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
            jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
        } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
            jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
        }
    }
    if !is_drag && !oap.is_null() && (*oap).op_type != OP_NOP as ::core::ffi::c_int {
        got_click = false_0 != 0;
        (*oap).motion_type = kMTCharWise;
    }
    if !is_click && !is_drag {
        jump_flags |= MOUSE_RELEASED as ::core::ffi::c_int;
    }
    let mut old_active: ::core::ffi::c_int = VIsual_active as ::core::ffi::c_int;
    let mut save_cursor: pos_T = (*curwin).w_cursor;
    if !VIsual_active || mouse_can_visual as ::core::ffi::c_int != 0 {
        jump_flags = jump_to_mouse(
            jump_flags,
            if oap.is_null() {
                ::core::ptr::null_mut::<bool>()
            } else {
                &raw mut (*oap).inclusive
            },
            which_button,
        );
    }
    let mut moved: bool = jump_flags & CURSOR_MOVED as ::core::ffi::c_int != 0;
    let mut in_winbar: bool = jump_flags & MOUSE_WINBAR as ::core::ffi::c_int != 0;
    let mut in_statuscol: bool = jump_flags & MOUSE_STATUSCOL as ::core::ffi::c_int != 0;
    let mut in_status_line: bool = jump_flags & IN_STATUS_LINE as ::core::ffi::c_int != 0;
    let mut in_global_statusline: bool =
        in_status_line as ::core::ffi::c_int != 0 && global_stl_height() > 0 as ::core::ffi::c_int;
    let mut in_sep_line: bool = jump_flags & IN_SEP_LINE as ::core::ffi::c_int != 0;
    if (in_winbar as ::core::ffi::c_int != 0
        || in_status_line as ::core::ffi::c_int != 0
        || in_statuscol as ::core::ffi::c_int != 0)
        && is_click as ::core::ffi::c_int != 0
    {
        let mut click_grid: ::core::ffi::c_int = mouse_grid;
        let mut click_row: ::core::ffi::c_int = mouse_row;
        let mut click_col: ::core::ffi::c_int = mouse_col;
        let mut wp: *mut win_T =
            mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col);
        if wp.is_null() {
            return false_0 != 0;
        }
        let mut click_defs: *mut StlClickDefinition = if in_status_line as ::core::ffi::c_int != 0 {
            (*wp).w_status_click_defs
        } else if in_winbar as ::core::ffi::c_int != 0 {
            (*wp).w_winbar_click_defs
        } else {
            (*wp).w_statuscol_click_defs
        };
        if in_global_statusline {
            click_defs = (*curwin).w_status_click_defs;
            click_col = mouse_col;
        }
        if in_statuscol as ::core::ffi::c_int != 0 && (*wp).w_onebuf_opt.wo_rl != 0 {
            click_col = (*wp).w_view_width - click_col - 1 as ::core::ffi::c_int;
        }
        if in_statuscol as ::core::ffi::c_int != 0
            && click_col >= (*wp).w_statuscol_click_defs_size as ::core::ffi::c_int
            || in_status_line as ::core::ffi::c_int != 0
                && click_col
                    >= (*(if in_global_statusline as ::core::ffi::c_int != 0 {
                        curwin
                    } else {
                        wp
                    }))
                    .w_status_click_defs_size as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        if !click_defs.is_null() {
            match (*click_defs.offset(click_col as isize)).type_0 as ::core::ffi::c_uint {
                0 => {
                    if in_statuscol as ::core::ffi::c_int != 0
                        && mouse_model_popup() as ::core::ffi::c_int != 0
                        && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                        && mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
                    {
                        do_popup(which_button, m_pos_flag, m_pos);
                    }
                }
                3 => {
                    call_click_def_func(click_defs, click_col, which_button);
                }
                _ => {
                    '_c2rust_label: {
                        if false
                            && !(b"winbar, statusline and statuscolumn only support %@ for clicks\0"
                                .as_ptr()
                                as *const ::core::ffi::c_char)
                                .is_null()
                        {
                        } else {
                            __assert_fail(
                                b"false && \"winbar, statusline and statuscolumn only support %@ for clicks\"\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/mouse.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                716 as ::core::ffi::c_uint,
                                b"_Bool do_mouse(oparg_T *, int, int, int, _Bool)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            }
        }
        if !(in_statuscol as ::core::ffi::c_int != 0
            && jump_flags
                & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
                != 0)
        {
            return false_0 != 0;
        }
    } else if in_winbar as ::core::ffi::c_int != 0 || in_statuscol as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    if curwin != old_curwin && !oap.is_null() && (*oap).op_type != OP_NOP as ::core::ffi::c_int {
        clearop(oap);
    }
    if mod_mask == 0 as ::core::ffi::c_int
        && !is_drag
        && jump_flags
            & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
            != 0
        && which_button == MOUSE_LEFT as ::core::ffi::c_int
    {
        if jump_flags & MOUSE_FOLD_OPEN as ::core::ffi::c_int != 0 {
            openFold((*curwin).w_cursor, 1 as ::core::ffi::c_int);
        } else {
            closeFold((*curwin).w_cursor, 1 as ::core::ffi::c_int);
        }
        if curwin == old_curwin {
            (*curwin).w_cursor = save_cursor;
        }
    }
    if VIsual_active as ::core::ffi::c_int != 0
        && is_drag as ::core::ffi::c_int != 0
        && get_scrolloff_value(curwin) != 0
    {
        if mouse_row == 0 as ::core::ffi::c_int {
            mouse_dragging = 2 as ::core::ffi::c_int;
        } else {
            mouse_dragging = 1 as ::core::ffi::c_int;
        }
    }
    if is_drag as ::core::ffi::c_int != 0 && mouse_row < 0 as ::core::ffi::c_int && !in_status_line
    {
        scroll_redraw(false_0, 1 as linenr_T);
        mouse_row = 0 as ::core::ffi::c_int;
    }
    let mut old_mode: ::core::ffi::c_int = VIsual_mode;
    if start_visual.lnum != 0 {
        let mut diff: linenr_T = 0;
        if mod_mask & MOD_MASK_ALT != 0 {
            VIsual_mode = Ctrl_V;
        }
        if VIsual_mode == Ctrl_V {
            let mut leftcol: colnr_T = 0;
            let mut rightcol: colnr_T = 0;
            getvcols(
                curwin,
                &raw mut start_visual,
                &raw mut end_visual,
                &raw mut leftcol,
                &raw mut rightcol,
            );
            if (*curwin).w_curswant
                > (leftcol as ::core::ffi::c_int + rightcol as ::core::ffi::c_int)
                    / 2 as ::core::ffi::c_int
            {
                end_visual.col = leftcol;
            } else {
                end_visual.col = rightcol;
            }
            if (*curwin).w_cursor.lnum >= (start_visual.lnum + end_visual.lnum) / 2 as linenr_T {
                end_visual.lnum = start_visual.lnum;
            }
            start_visual = (*curwin).w_cursor;
            (*curwin).w_cursor = end_visual;
            coladvance(curwin, end_visual.col);
            VIsual = (*curwin).w_cursor;
            (*curwin).w_cursor = start_visual;
        } else if lt((*curwin).w_cursor, start_visual) {
            VIsual = end_visual;
        } else if lt(end_visual, (*curwin).w_cursor) {
            VIsual = start_visual;
        } else if end_visual.lnum == start_visual.lnum {
            if (*curwin).w_cursor.col - start_visual.col > end_visual.col - (*curwin).w_cursor.col {
                VIsual = start_visual;
            } else {
                VIsual = end_visual;
            }
        } else {
            diff = (*curwin).w_cursor.lnum
                - start_visual.lnum
                - (end_visual.lnum - (*curwin).w_cursor.lnum);
            if diff > 0 as linenr_T {
                VIsual = start_visual;
            } else if diff < 0 as linenr_T {
                VIsual = end_visual;
            } else if (*curwin).w_cursor.col
                < (start_visual.col as ::core::ffi::c_int + end_visual.col as ::core::ffi::c_int)
                    / 2 as ::core::ffi::c_int
            {
                VIsual = end_visual;
            } else {
                VIsual = start_visual;
            }
        }
    } else if State & MODE_INSERT as ::core::ffi::c_int != 0
        && VIsual_active as ::core::ffi::c_int != 0
    {
        stuffcharReadbuff(Ctrl_O);
    }
    if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        if regname == 0 as ::core::ffi::c_int
            && eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            ) as ::core::ffi::c_int
                != 0
        {
            regname = '*' as ::core::ffi::c_int;
        }
        let mut reg_0: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        if yank_register_mline(regname, &raw mut reg_0) {
            if mouse_past_bottom {
                dir = FORWARD as ::core::ffi::c_int;
            }
        } else if mouse_past_eol {
            dir = FORWARD as ::core::ffi::c_int;
        }
        let mut c1: ::core::ffi::c_int = 0;
        let mut c2: ::core::ffi::c_int = 0;
        if fixindent {
            c1 = if dir == BACKWARD as ::core::ffi::c_int {
                '[' as ::core::ffi::c_int
            } else {
                ']' as ::core::ffi::c_int
            };
            c2 = 'p' as ::core::ffi::c_int;
        } else {
            c1 = if dir == FORWARD as ::core::ffi::c_int {
                'p' as ::core::ffi::c_int
            } else {
                'P' as ::core::ffi::c_int
            };
            c2 = NUL;
        }
        prep_redo(regname, count, NUL, c1, NUL, c2, NUL);
        if restart_edit != 0 as ::core::ffi::c_int {
            where_paste_started = (*curwin).w_cursor;
        }
        do_put(
            regname,
            reg_0,
            dir,
            count,
            (if fixindent as ::core::ffi::c_int != 0 {
                PUT_FIXINDENT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | PUT_CURSEND as ::core::ffi::c_int,
        );
    } else if (mod_mask & MOD_MASK_CTRL != 0 || mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK)
        && bt_quickfix(curbuf) as ::core::ffi::c_int != 0
    {
        if (*curwin).w_llist_ref.is_null() {
            do_cmdline_cmd(b".cc\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            do_cmdline_cmd(b".ll\0".as_ptr() as *const ::core::ffi::c_char);
        }
        got_click = false_0 != 0;
    } else if mod_mask & MOD_MASK_CTRL != 0
        || (*curbuf).b_help as ::core::ffi::c_int != 0
            && mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK
    {
        if State & MODE_INSERT as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(Ctrl_O);
        }
        stuffcharReadbuff(Ctrl_RSB);
        got_click = false_0 != 0;
    } else if mod_mask & MOD_MASK_SHIFT != 0 {
        if State & MODE_INSERT as ::core::ffi::c_int != 0
            || VIsual_active as ::core::ffi::c_int != 0 && VIsual_select as ::core::ffi::c_int != 0
        {
            stuffcharReadbuff(Ctrl_O);
        }
        if which_button == MOUSE_LEFT as ::core::ffi::c_int {
            stuffcharReadbuff('*' as ::core::ffi::c_int);
        } else {
            stuffcharReadbuff('#' as ::core::ffi::c_int);
        }
    } else if !(in_status_line as ::core::ffi::c_int != 0 || in_sep_line as ::core::ffi::c_int != 0)
    {
        if mod_mask & MOD_MASK_MULTI_CLICK != 0
            && State & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int) != 0
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click as ::core::ffi::c_int != 0 || !VIsual_active {
                if VIsual_active {
                    orig_cursor = VIsual;
                } else {
                    VIsual = (*curwin).w_cursor;
                    orig_cursor = VIsual;
                    VIsual_active = true_0 != 0;
                    VIsual_reselect = true_0;
                    may_start_select('o' as ::core::ffi::c_int);
                    setmouse();
                }
                if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                    if mod_mask & MOD_MASK_ALT != 0 {
                        VIsual_mode = Ctrl_V;
                    } else {
                        VIsual_mode = 'v' as ::core::ffi::c_int;
                    }
                } else if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                    VIsual_mode = 'V' as ::core::ffi::c_int;
                } else if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                    VIsual_mode = Ctrl_V;
                }
            }
            if mod_mask & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
                if is_click {
                    end_visual = (*curwin).w_cursor;
                    let mut gc: ::core::ffi::c_int = 0;
                    loop {
                        gc = gchar_pos(&raw mut end_visual);
                        if !ascii_iswhite(gc) {
                            break;
                        }
                        inc(&raw mut end_visual);
                    }
                    if !oap.is_null() {
                        (*oap).motion_type = kMTCharWise;
                    }
                    if !oap.is_null()
                        && VIsual_mode == 'v' as ::core::ffi::c_int
                        && !vim_iswordc(gchar_pos(&raw mut end_visual))
                        && equalpos((*curwin).w_cursor, VIsual) as ::core::ffi::c_int != 0
                        && {
                            pos = findmatch(oap, NUL);
                            !pos.is_null()
                        }
                    {
                        (*curwin).w_cursor = *pos;
                        if (*oap).motion_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int
                        {
                            VIsual_mode = 'V' as ::core::ffi::c_int;
                        } else if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                            if lt((*curwin).w_cursor, VIsual) {
                                VIsual.col += 1;
                            } else {
                                (*curwin).w_cursor.col += 1;
                            }
                        }
                    }
                }
                if pos.is_null()
                    && (is_click as ::core::ffi::c_int != 0 || is_drag as ::core::ffi::c_int != 0)
                {
                    if lt((*curwin).w_cursor, orig_cursor) {
                        find_start_of_word(&raw mut (*curwin).w_cursor);
                        find_end_of_word(&raw mut VIsual);
                    } else {
                        find_start_of_word(&raw mut VIsual);
                        if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                            && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL
                        {
                            (*curwin).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                        }
                        find_end_of_word(&raw mut (*curwin).w_cursor);
                    }
                }
                (*curwin).w_set_curswant = true_0;
            }
            if is_click {
                redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            }
        } else if VIsual_active as ::core::ffi::c_int != 0 && old_active == 0 {
            if mod_mask & MOD_MASK_ALT != 0 {
                VIsual_mode = Ctrl_V;
            } else {
                VIsual_mode = 'v' as ::core::ffi::c_int;
            }
        }
    }
    if !VIsual_active && old_active != 0 && mode_displayed as ::core::ffi::c_int != 0
        || VIsual_active as ::core::ffi::c_int != 0
            && p_smd != 0
            && msg_silent == 0 as ::core::ffi::c_int
            && (old_active == 0 || VIsual_mode != old_mode)
    {
        redraw_cmdline = true_0 != 0;
    }
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn ins_mouse(mut c: ::core::ffi::c_int) {
    let mut old_curwin: *mut win_T = curwin;
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if do_mouse(
        ::core::ptr::null_mut::<oparg_T>(),
        c,
        BACKWARD as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        false,
    ) {
        let mut new_curwin: *mut win_T = curwin;
        if curwin != old_curwin && win_valid(old_curwin) as ::core::ffi::c_int != 0 {
            curwin = old_curwin;
            curbuf = (*curwin).w_buffer;
            if bt_prompt(curbuf) {
                (*curbuf).b_prompt_insert = 'A' as ::core::ffi::c_int;
            }
        }
        start_arrow(if curwin == old_curwin {
            &raw mut tpos
        } else {
            ::core::ptr::null_mut::<pos_T>()
        });
        if curwin != new_curwin && win_valid(new_curwin) as ::core::ffi::c_int != 0 {
            curwin = new_curwin;
            curbuf = (*curwin).w_buffer;
        }
        set_can_cindent(true_0 != 0);
    }
    redraw_statuslines();
}
#[no_mangle]
pub unsafe extern "C" fn do_mousescroll(mut cap: *mut cmdarg_T) {
    let mut shift_or_ctrl: bool = mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0;
    if (*cap).arg == MSCR_UP as ::core::ffi::c_int || (*cap).arg == MSCR_DOWN as ::core::ffi::c_int
    {
        if State & MODE_NORMAL as ::core::ffi::c_int != 0
            && shift_or_ctrl as ::core::ffi::c_int != 0
        {
            pagescroll(
                (if (*cap).arg != 0 {
                    FORWARD as ::core::ffi::c_int
                } else {
                    BACKWARD as ::core::ffi::c_int
                }) as Direction,
                1 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            if shift_or_ctrl {
                (*cap).count1 = ((*curwin).w_botline - (*curwin).w_topline) as ::core::ffi::c_int;
            } else {
                (*cap).count1 = p_mousescroll_vert as ::core::ffi::c_int;
            }
            if (*cap).count1 > 0 as ::core::ffi::c_int {
                (*cap).count0 = (*cap).count1;
                nv_scroll_line(cap);
            }
        }
    } else {
        let mut step: ::core::ffi::c_int = if shift_or_ctrl as ::core::ffi::c_int != 0 {
            (*curwin).w_view_width
        } else {
            p_mousescroll_hor as ::core::ffi::c_int
        };
        let mut leftcol: colnr_T = (*curwin).w_leftcol
            + (if (*cap).arg == MSCR_RIGHT as ::core::ffi::c_int {
                -(step as colnr_T)
            } else {
                step as colnr_T
            });
        leftcol = (if leftcol > 0 as ::core::ffi::c_int {
            leftcol as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as colnr_T;
        do_mousescroll_horiz(leftcol);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ins_mousescroll(mut dir: ::core::ffi::c_int) {
    let mut cap: cmdarg_T = cmdarg_T {
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
    memset(
        &raw mut cap as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    clear_oparg(&raw mut oa);
    cap.oap = &raw mut oa;
    cap.arg = dir;
    match dir {
        1 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        0 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        -1 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        -2 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        _ => {
            siemsg(
                b"Invalid ins_mousescroll() argument: %d\0".as_ptr() as *const ::core::ffi::c_char,
                dir,
            );
        }
    }
    let mut old_curwin: *mut win_T = curwin;
    if mouse_row >= 0 as ::core::ffi::c_int && mouse_col >= 0 as ::core::ffi::c_int {
        let mut grid: ::core::ffi::c_int = mouse_grid;
        let mut row: ::core::ffi::c_int = mouse_row;
        let mut col: ::core::ffi::c_int = mouse_col;
        curwin = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
        if curwin.is_null() {
            curwin = old_curwin;
            return;
        }
        curbuf = (*curwin).w_buffer;
    }
    if curwin == old_curwin {
        if pum_visible() {
            return;
        }
        undisplay_dollar();
    }
    let mut orig_cursor: pos_T = (*curwin).w_cursor;
    do_mousescroll(&raw mut cap);
    (*curwin).w_redr_status = true_0 != 0;
    curwin = old_curwin;
    curbuf = (*curwin).w_buffer;
    if !equalpos((*curwin).w_cursor, orig_cursor) {
        start_arrow(&raw mut orig_cursor);
        set_can_cindent(true_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn is_mouse_key(mut c: ::core::ffi::c_int) -> bool {
    return c
        == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
}
unsafe extern "C" fn mouse_model_popup() -> bool {
    return *p_mousem.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'p' as ::core::ffi::c_int;
}
static mut dragwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub unsafe extern "C" fn reset_dragwin() {
    dragwin = ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn jump_to_mouse(
    mut flags: ::core::ffi::c_int,
    mut inclusive: *mut bool,
    mut which_button: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static mut status_line_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    static mut sep_line_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    static mut on_status_line: bool = false_0 != 0;
    static mut on_sep_line: bool = false_0 != 0;
    static mut on_winbar: bool = false_0 != 0;
    static mut on_statuscol: bool = false_0 != 0;
    static mut prev_row: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    static mut prev_col: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    static mut did_drag: ::core::ffi::c_int = false_0;
    let mut count: ::core::ffi::c_int = 0;
    let mut first: bool = false;
    let mut row: ::core::ffi::c_int = mouse_row;
    let mut col: ::core::ffi::c_int = mouse_col;
    let mut grid: ::core::ffi::c_int = mouse_grid;
    let mut fdc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keep_focus: bool = flags & MOUSE_FOCUS as ::core::ffi::c_int != 0;
    mouse_past_bottom = false_0 != 0;
    mouse_past_eol = false_0 != 0;
    if flags & MOUSE_RELEASED as ::core::ffi::c_int != 0 {
        if !dragwin.is_null() && did_drag == 0 {
            flags &= !(MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int);
        }
        dragwin = ::core::ptr::null_mut::<win_T>();
        did_drag = false_0;
    }
    if !(flags & MOUSE_DID_MOVE as ::core::ffi::c_int != 0
        && prev_row == mouse_row
        && prev_col == mouse_col)
    {
        prev_row = mouse_row;
        prev_col = mouse_col;
        if flags & MOUSE_SETPOS as ::core::ffi::c_int == 0 {
            if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
                return IN_UNKNOWN as ::core::ffi::c_int;
            }
            let mut wp: *mut win_T =
                mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
            if wp.is_null() {
                return IN_UNKNOWN as ::core::ffi::c_int;
            }
            let mut below_window: bool =
                grid == DEFAULT_GRID_HANDLE && row + (*wp).w_winbar_height >= (*wp).w_height;
            on_status_line = below_window as ::core::ffi::c_int != 0
                && row + (*wp).w_winbar_height - (*wp).w_height + 1 as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int;
            on_sep_line = grid == DEFAULT_GRID_HANDLE
                && col >= (*wp).w_width
                && col - (*wp).w_width + 1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
            on_winbar = row < 0 as ::core::ffi::c_int
                && row + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int;
            on_statuscol = !below_window
                && !on_status_line
                && !on_sep_line
                && !on_winbar
                && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
                && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                    (col >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
                } else {
                    (col < win_col_off(wp)) as ::core::ffi::c_int
                }) != 0;
            if on_status_line as ::core::ffi::c_int != 0 && on_sep_line as ::core::ffi::c_int != 0 {
                if stl_connected(wp) {
                    on_sep_line = false_0 != 0;
                } else {
                    on_status_line = false_0 != 0;
                }
            }
            if keep_focus {
                row = mouse_row;
                col = mouse_col;
                grid = mouse_grid;
            }
            let mut old_curwin: *mut win_T = curwin;
            let mut old_cursor: pos_T = (*curwin).w_cursor;
            if !keep_focus {
                if on_winbar {
                    return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
                }
                if !on_statuscol {
                    fdc = win_fdccol_count(wp);
                    dragwin = ::core::ptr::null_mut::<win_T>();
                    if below_window {
                        status_line_offset =
                            row + (*wp).w_winbar_height - (*wp).w_height + 1 as ::core::ffi::c_int;
                        dragwin = wp;
                    } else {
                        status_line_offset = 0 as ::core::ffi::c_int;
                    }
                    if grid == DEFAULT_GRID_HANDLE && col >= (*wp).w_width {
                        sep_line_offset = col - (*wp).w_width + 1 as ::core::ffi::c_int;
                        dragwin = wp;
                    } else {
                        sep_line_offset = 0 as ::core::ffi::c_int;
                    }
                    if status_line_offset != 0 && sep_line_offset != 0 {
                        if stl_connected(wp) {
                            sep_line_offset = 0 as ::core::ffi::c_int;
                        } else {
                            status_line_offset = 0 as ::core::ffi::c_int;
                        }
                    }
                    if VIsual_active as ::core::ffi::c_int != 0
                        && ((*wp).w_buffer != (*curwin).w_buffer
                            || status_line_offset == 0
                                && sep_line_offset == 0
                                && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                                    (col < (*wp).w_view_width - fdc) as ::core::ffi::c_int
                                } else {
                                    (col >= fdc
                                        + (if wp != cmdwin_win {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            1 as ::core::ffi::c_int
                                        }))
                                        as ::core::ffi::c_int
                                }) != 0
                                && flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0)
                    {
                        end_visual_mode();
                        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                    }
                    if cmdwin_type != 0 as ::core::ffi::c_int && wp != cmdwin_win {
                        sep_line_offset = 0 as ::core::ffi::c_int;
                        row = 0 as ::core::ffi::c_int;
                        col += (*wp).w_wincol;
                        wp = cmdwin_win;
                    }
                    if dragwin.is_null() || flags & MOUSE_RELEASED as ::core::ffi::c_int != 0 {
                        win_enter(wp, true_0 != 0);
                    }
                    if curwin != old_curwin {
                        set_mouse_topline(curwin);
                    }
                    if status_line_offset != 0 {
                        if curwin == old_curwin {
                            return IN_STATUS_LINE as ::core::ffi::c_int;
                        }
                        return IN_STATUS_LINE as ::core::ffi::c_int
                            | CURSOR_MOVED as ::core::ffi::c_int;
                    }
                    if sep_line_offset != 0 {
                        if curwin == old_curwin {
                            return IN_SEP_LINE as ::core::ffi::c_int;
                        }
                        return IN_SEP_LINE as ::core::ffi::c_int
                            | CURSOR_MOVED as ::core::ffi::c_int;
                    }
                    (*curwin).w_cursor.lnum = (*curwin).w_topline;
                }
            } else if status_line_offset != 0 {
                if which_button == MOUSE_LEFT as ::core::ffi::c_int && !dragwin.is_null() {
                    count = row - (*dragwin).w_winrow - (*dragwin).w_height
                        + 1 as ::core::ffi::c_int
                        - status_line_offset;
                    win_drag_status_line(dragwin, count);
                    did_drag |= count;
                }
                return IN_STATUS_LINE as ::core::ffi::c_int;
            } else if sep_line_offset != 0 && which_button == MOUSE_LEFT as ::core::ffi::c_int {
                if !dragwin.is_null() {
                    count = col - (*dragwin).w_wincol - (*dragwin).w_width
                        + 1 as ::core::ffi::c_int
                        - sep_line_offset;
                    win_drag_vsep_line(dragwin, count);
                    did_drag |= count;
                }
                return IN_SEP_LINE as ::core::ffi::c_int;
            } else if on_status_line as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_STATUS_LINE as ::core::ffi::c_int;
            } else if on_winbar as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
            } else if on_statuscol as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int;
            } else {
                if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
                    end_visual_mode();
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                }
                if grid == 0 as ::core::ffi::c_int {
                    row -= (*curwin).w_grid_alloc.comp_row + (*curwin).w_grid.row_offset;
                    col -= (*curwin).w_grid_alloc.comp_col + (*curwin).w_grid.col_offset;
                } else if grid != DEFAULT_GRID_HANDLE {
                    row -= (*curwin).w_grid.row_offset;
                    col -= (*curwin).w_grid.col_offset;
                }
                if row < 0 as ::core::ffi::c_int {
                    count = 0 as ::core::ffi::c_int;
                    first = true_0 != 0;
                    while (*curwin).w_topline > 1 as linenr_T {
                        if (*curwin).w_topfill < win_get_fill(curwin, (*curwin).w_topline) {
                            count += 1;
                        } else {
                            count += plines_win(
                                curwin,
                                (*curwin).w_topline - 1 as linenr_T,
                                true_0 != 0,
                            );
                        }
                        if !first && count > -row {
                            break;
                        }
                        first = false_0 != 0;
                        hasFolding(
                            curwin,
                            (*curwin).w_topline,
                            &raw mut (*curwin).w_topline,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        if (*curwin).w_topfill < win_get_fill(curwin, (*curwin).w_topline) {
                            (*curwin).w_topfill += 1;
                        } else {
                            (*curwin).w_topline -= 1;
                            (*curwin).w_topfill = 0 as ::core::ffi::c_int;
                        }
                    }
                    check_topfill(curwin, false_0 != 0);
                    (*curwin).w_valid &=
                        !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                    redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
                    row = 0 as ::core::ffi::c_int;
                } else if row >= (*curwin).w_view_height {
                    count = 0 as ::core::ffi::c_int;
                    first = true_0 != 0;
                    while (*curwin).w_topline < (*curbuf).b_ml.ml_line_count {
                        if (*curwin).w_topfill > 0 as ::core::ffi::c_int {
                            count += 1;
                        } else {
                            count += plines_win(curwin, (*curwin).w_topline, true_0 != 0);
                        }
                        if !first && count > row - (*curwin).w_view_height + 1 as ::core::ffi::c_int
                        {
                            break;
                        }
                        first = false_0 != 0;
                        if (*curwin).w_topfill > 0 as ::core::ffi::c_int {
                            (*curwin).w_topfill -= 1;
                        } else {
                            if hasFolding(
                                curwin,
                                (*curwin).w_topline,
                                ::core::ptr::null_mut::<linenr_T>(),
                                &raw mut (*curwin).w_topline,
                            ) as ::core::ffi::c_int
                                != 0
                                && (*curwin).w_topline == (*curbuf).b_ml.ml_line_count
                            {
                                break;
                            }
                            (*curwin).w_topline += 1;
                            (*curwin).w_topfill = win_get_fill(curwin, (*curwin).w_topline);
                        }
                    }
                    check_topfill(curwin, false_0 != 0);
                    redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
                    (*curwin).w_valid &=
                        !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                    row = (*curwin).w_view_height - 1 as ::core::ffi::c_int;
                } else if row == 0 as ::core::ffi::c_int {
                    if mouse_dragging > 0 as ::core::ffi::c_int
                        && (*curwin).w_cursor.lnum == (*(*curwin).w_buffer).b_ml.ml_line_count
                        && (*curwin).w_cursor.lnum == (*curwin).w_topline
                    {
                        (*curwin).w_valid &= !VALID_TOPLINE;
                    }
                }
            }
            let mut col_from_screen: colnr_T = -1 as colnr_T;
            let mut mouse_fold_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            mouse_check_grid(&raw mut col_from_screen, &raw mut mouse_fold_flags);
            if mouse_comp_pos(
                curwin,
                &raw mut row,
                &raw mut col,
                &raw mut (*curwin).w_cursor.lnum,
            ) {
                mouse_past_bottom = true_0 != 0;
            }
            if flags & MOUSE_MAY_VIS as ::core::ffi::c_int != 0 && !VIsual_active {
                VIsual = old_cursor;
                VIsual_active = true_0 != 0;
                VIsual_reselect = true_0;
                may_start_select('o' as ::core::ffi::c_int);
                setmouse();
                if p_smd != 0 && msg_silent == 0 as ::core::ffi::c_int {
                    redraw_cmdline = true_0 != 0;
                }
            }
            if col_from_screen >= 0 as ::core::ffi::c_int {
                col = col_from_screen as ::core::ffi::c_int;
            }
            (*curwin).w_curswant = col as colnr_T;
            (*curwin).w_set_curswant = false_0;
            if coladvance(curwin, col as colnr_T) == FAIL {
                if !inclusive.is_null() {
                    *inclusive = true_0 != 0;
                }
                mouse_past_eol = true_0 != 0;
            } else if !inclusive.is_null() {
                *inclusive = false_0 != 0;
            }
            count = if on_statuscol as ::core::ffi::c_int != 0 {
                IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int
            } else {
                IN_BUFFER as ::core::ffi::c_int
            };
            if curwin != old_curwin
                || (*curwin).w_cursor.lnum != old_cursor.lnum
                || (*curwin).w_cursor.col != old_cursor.col
            {
                count |= CURSOR_MOVED as ::core::ffi::c_int;
            }
            count |= mouse_fold_flags;
            return count;
        }
    }
    if status_line_offset != 0 {
        return IN_STATUS_LINE as ::core::ffi::c_int;
    }
    if sep_line_offset != 0 {
        return IN_SEP_LINE as ::core::ffi::c_int;
    }
    if on_winbar {
        return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
    }
    if on_statuscol {
        return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int;
    }
    if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    return IN_BUFFER as ::core::ffi::c_int;
}
unsafe extern "C" fn do_mousescroll_horiz(mut leftcol: colnr_T) -> bool {
    if (*curwin).w_onebuf_opt.wo_wrap != 0 {
        return false_0 != 0;
    }
    if (*curwin).w_leftcol == leftcol {
        return false_0 != 0;
    }
    if !virtual_active(curwin) && leftcol > scroll_line_len((*curwin).w_cursor.lnum) {
        (*curwin).w_cursor.lnum = find_longest_lnum();
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return set_leftcol(leftcol);
}
#[no_mangle]
pub unsafe extern "C" fn nv_mousescroll(mut cap: *mut cmdarg_T) {
    let old_curwin: *mut win_T = curwin;
    if mouse_row >= 0 as ::core::ffi::c_int && mouse_col >= 0 as ::core::ffi::c_int {
        let mut grid: ::core::ffi::c_int = mouse_grid;
        let mut row: ::core::ffi::c_int = mouse_row;
        let mut col: ::core::ffi::c_int = mouse_col;
        curwin = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
        if curwin.is_null() {
            curwin = old_curwin;
            return;
        }
        curbuf = (*curwin).w_buffer;
    }
    do_mousescroll(cap);
    (*curwin).w_redr_status = true_0 != 0;
    curwin = old_curwin;
    curbuf = (*curwin).w_buffer;
}
#[no_mangle]
pub unsafe extern "C" fn nv_mouse(mut cap: *mut cmdarg_T) {
    do_mouse(
        (*cap).oap,
        (*cap).cmdchar,
        BACKWARD as ::core::ffi::c_int,
        (*cap).count1,
        false,
    );
}
#[no_mangle]
pub unsafe extern "C" fn mouse_comp_pos(
    mut win: *mut win_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
    mut lnump: *mut linenr_T,
) -> bool {
    let mut col: ::core::ffi::c_int = *colp;
    let mut row: ::core::ffi::c_int = *rowp;
    let mut retval: bool = false_0 != 0;
    let mut count: ::core::ffi::c_int = 0;
    if (*win).w_onebuf_opt.wo_rl != 0 {
        col = (*win).w_view_width - 1 as ::core::ffi::c_int - col;
    }
    let mut lnum: linenr_T = (*win).w_topline;
    while row > 0 as ::core::ffi::c_int {
        if win_may_fill(win) {
            row -= if lnum == (*win).w_topline {
                (*win).w_topfill
            } else {
                win_get_fill(win, lnum)
            };
            count = plines_win_nofill(win, lnum, false_0 != 0);
        } else {
            count = plines_win(win, lnum, false_0 != 0);
        }
        if (*win).w_skipcol > 0 as ::core::ffi::c_int && lnum == (*win).w_topline {
            let mut width1: ::core::ffi::c_int = (*win).w_view_width - win_col_off(win);
            if width1 > 0 as ::core::ffi::c_int {
                let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if (*win).w_skipcol > width1 {
                    skip_lines = ((*win).w_skipcol as ::core::ffi::c_int - width1)
                        / (width1 + win_col_off2(win))
                        + 1 as ::core::ffi::c_int;
                } else if (*win).w_skipcol > 0 as ::core::ffi::c_int {
                    skip_lines = 1 as ::core::ffi::c_int;
                }
                count -= skip_lines;
            }
        }
        if count > row {
            break;
        }
        hasFolding(
            win,
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut lnum,
        );
        if lnum == (*(*win).w_buffer).b_ml.ml_line_count {
            retval = true_0 != 0;
            break;
        } else {
            row -= count;
            lnum += 1;
        }
    }
    while lnum < (*(*win).w_buffer).b_ml.ml_line_count
        && decor_conceal_line(
            win,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        lnum += 1;
        hasFolding(
            win,
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut lnum,
        );
    }
    if !retval {
        let mut off: ::core::ffi::c_int = win_col_off(win) - win_col_off2(win);
        col = if col > off { col } else { off };
        col += row * ((*win).w_view_width - off);
        if lnum == (*win).w_topline {
            col += (*win).w_skipcol as ::core::ffi::c_int;
        }
    }
    if (*win).w_onebuf_opt.wo_wrap == 0 {
        col += (*win).w_leftcol as ::core::ffi::c_int;
    }
    col -= win_col_off(win);
    col = if col > 0 as ::core::ffi::c_int {
        col
    } else {
        0 as ::core::ffi::c_int
    };
    *colp = col;
    *rowp = row;
    *lnump = lnum;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_find_win_inner(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    let mut wp_grid: *mut win_T = mouse_find_grid_win(gridp, rowp, colp);
    if !wp_grid.is_null() {
        return wp_grid;
    } else if *gridp > 1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<win_T>();
    }
    let mut fp: *mut frame_T = topframe;
    *rowp -= (*firstwin).w_winrow;
    while (*fp).fr_layout as ::core::ffi::c_int != FR_LEAF {
        if (*fp).fr_layout as ::core::ffi::c_int == FR_ROW {
            fp = (*fp).fr_child;
            while !(*fp).fr_next.is_null() {
                if *colp < (*fp).fr_width {
                    break;
                }
                *colp -= (*fp).fr_width;
                fp = (*fp).fr_next;
            }
        } else {
            fp = (*fp).fr_child;
            while !(*fp).fr_next.is_null() {
                if *rowp < (*fp).fr_height {
                    break;
                }
                *rowp -= (*fp).fr_height;
                fp = (*fp).fr_next;
            }
        }
    }
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        if wp == (*fp).fr_win {
            *rowp -= (*wp).w_winbar_height;
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn mouse_find_win_outer(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    let mut wp: *mut win_T = mouse_find_win_inner(gridp, rowp, colp);
    if !wp.is_null() {
        *rowp += (*wp).w_winrow_off;
        *colp += (*wp).w_wincol_off;
    }
    return wp;
}
unsafe extern "C" fn mouse_find_grid_win(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    if *gridp == msg_grid.handle {
        *rowp += msg_grid_pos;
        *gridp = DEFAULT_GRID_HANDLE;
    } else if *gridp > 1 as ::core::ffi::c_int {
        let mut wp: *mut win_T = get_win_by_grid_handle(*gridp as handle_T);
        if !wp.is_null()
            && !(*wp).w_grid_alloc.chars.is_null()
            && !((*wp).w_floating as ::core::ffi::c_int != 0 && !(*wp).w_config.mouse)
        {
            *rowp = if *rowp - (*wp).w_grid.row_offset
                < (*wp).w_view_height - 1 as ::core::ffi::c_int
            {
                *rowp - (*wp).w_grid.row_offset
            } else {
                (*wp).w_view_height - 1 as ::core::ffi::c_int
            };
            *colp =
                if *colp - (*wp).w_grid.col_offset < (*wp).w_view_width - 1 as ::core::ffi::c_int {
                    *colp - (*wp).w_grid.col_offset
                } else {
                    (*wp).w_view_width - 1 as ::core::ffi::c_int
                };
            return wp;
        }
    } else if *gridp == 0 as ::core::ffi::c_int {
        let mut grid: *mut ScreenGrid = ui_comp_mouse_focus(*rowp, *colp);
        if grid == &raw mut pum_grid {
            *gridp = (*grid).handle as ::core::ffi::c_int;
            *rowp -= (*grid).comp_row;
            *colp -= (*grid).comp_col;
            return ::core::ptr::null_mut::<win_T>();
        } else {
            let mut wp_0: *mut win_T = if curtab == curtab {
                firstwin
            } else {
                (*curtab).tp_firstwin
            };
            while !wp_0.is_null() {
                if &raw mut (*wp_0).w_grid_alloc != grid {
                    wp_0 = (*wp_0).w_next;
                } else {
                    *gridp = (*grid).handle as ::core::ffi::c_int;
                    *rowp -= (*wp_0).w_winrow + (*wp_0).w_grid.row_offset;
                    *colp -= (*wp_0).w_wincol + (*wp_0).w_grid.col_offset;
                    return wp_0;
                }
            }
        }
        *gridp = DEFAULT_GRID_HANDLE;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn vcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: colnr_T,
    mut coladdp: *mut colnr_T,
) -> colnr_T {
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
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
            s: [C2Rust_Unnamed_13 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    let mut cur_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while cur_vcol < vcol && *ci.ptr as ::core::ffi::c_int != NUL {
        let mut next_vcol: ::core::ffi::c_int =
            cur_vcol + win_charsize(cstype, cur_vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
        if next_vcol > vcol {
            break;
        }
        cur_vcol = next_vcol;
        ci = utfc_next(ci);
    }
    if !coladdp.is_null() {
        *coladdp = (vcol as ::core::ffi::c_int - cur_vcol) as colnr_T;
    }
    return ci.ptr.offset_from(line) as colnr_T;
}
#[no_mangle]
pub unsafe extern "C" fn setmouse() {
    ui_cursor_shape();
    ui_check_mouse();
}
unsafe extern "C" fn set_mouse_topline(mut wp: *mut win_T) {
    orig_topline = (*wp).w_topline;
    orig_topfill = (*wp).w_topfill;
}
unsafe extern "C" fn scroll_line_len(mut lnum: linenr_T) -> colnr_T {
    let mut col: colnr_T = 0 as colnr_T;
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    if *line as ::core::ffi::c_int != NUL {
        loop {
            let mut numchar: ::core::ffi::c_int = win_chartabsize(curwin, line, col);
            line = line.offset(utfc_ptr2len(line) as isize);
            if *line as ::core::ffi::c_int == NUL {
                break;
            }
            col += numchar;
        }
    }
    return col;
}
unsafe extern "C" fn find_longest_lnum() -> linenr_T {
    let mut ret: linenr_T = 0 as linenr_T;
    if (*curwin).w_topline <= (*curwin).w_cursor.lnum
        && (*curwin).w_botline > (*curwin).w_cursor.lnum
        && (*curwin).w_botline <= (*curbuf).b_ml.ml_line_count + 1 as linenr_T
    {
        let mut max: colnr_T = 0 as colnr_T;
        let mut lnum: linenr_T = (*curwin).w_topline;
        while lnum < (*curwin).w_botline {
            let mut len: colnr_T = scroll_line_len(lnum);
            if len > max {
                max = len;
                ret = lnum;
            } else if len == max
                && abs(lnum as ::core::ffi::c_int - (*curwin).w_cursor.lnum as ::core::ffi::c_int)
                    < abs(ret as ::core::ffi::c_int - (*curwin).w_cursor.lnum as ::core::ffi::c_int)
            {
                ret = lnum;
            }
            lnum += 1;
        }
    } else {
        ret = (*curwin).w_cursor.lnum;
    }
    return ret;
}
unsafe extern "C" fn mouse_check_grid(
    mut vcolp: *mut colnr_T,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    let mut click_grid: ::core::ffi::c_int = mouse_grid;
    let mut click_row: ::core::ffi::c_int = mouse_row;
    let mut click_col: ::core::ffi::c_int = mouse_col;
    if mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col) != curwin
        || (*curwin).w_redr_type != 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut start_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut gp: *mut ScreenGrid = grid_adjust(
        &raw mut (*curwin).w_grid,
        &raw mut start_row,
        &raw mut start_col,
    );
    if (*gp).handle != click_grid || (*gp).chars.is_null() {
        return;
    }
    click_row += start_row;
    click_col += start_col;
    if click_row < 0 as ::core::ffi::c_int
        || click_row >= (*gp).rows
        || click_col < 0 as ::core::ffi::c_int
        || click_col >= (*gp).cols
    {
        return;
    }
    let off: size_t =
        (*(*gp).line_offset.offset(click_row as isize)).wrapping_add(click_col as size_t);
    let mut col_from_screen: colnr_T = *(*gp).vcols.offset(off as isize);
    if col_from_screen >= 0 as ::core::ffi::c_int {
        *vcolp = col_from_screen;
    }
    if col_from_screen == -2 as ::core::ffi::c_int {
        *flagsp |= MOUSE_FOLD_OPEN as ::core::ffi::c_int;
    } else if col_from_screen == -3 as ::core::ffi::c_int {
        *flagsp |= MOUSE_FOLD_CLOSE as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getmousepos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = mouse_row;
    let mut col: ::core::ffi::c_int = mouse_col;
    let mut grid: ::core::ffi::c_int = mouse_grid;
    let mut winid: varnumber_T = 0 as varnumber_T;
    let mut winrow: varnumber_T = 0 as varnumber_T;
    let mut wincol: varnumber_T = 0 as varnumber_T;
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut column: varnumber_T = 0 as varnumber_T;
    let mut coladd: colnr_T = 0 as colnr_T;
    tv_dict_alloc_ret(rettv);
    let mut d: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        d,
        b"screenrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        mouse_row as varnumber_T + 1 as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"screencol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        mouse_col as varnumber_T + 1 as varnumber_T,
    );
    let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
    if !wp.is_null() {
        let mut height: ::core::ffi::c_int =
            (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
        if row < height + (*wp).w_border_adj[2 as ::core::ffi::c_int as usize] {
            winid = (*wp).handle as varnumber_T;
            winrow = (row + 1 as ::core::ffi::c_int + (*wp).w_winrow_off) as varnumber_T;
            wincol = (col + 1 as ::core::ffi::c_int + (*wp).w_wincol_off) as varnumber_T;
            if row >= 0 as ::core::ffi::c_int
                && row < (*wp).w_height
                && col >= 0 as ::core::ffi::c_int
                && col < (*wp).w_width
            {
                mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut lnum);
                col = vcol2col(wp, lnum, col as colnr_T, &raw mut coladd) as ::core::ffi::c_int;
                column = (col + 1 as ::core::ffi::c_int) as varnumber_T;
            }
        }
    }
    tv_dict_add_nr(
        d,
        b"winid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        winid,
    );
    tv_dict_add_nr(
        d,
        b"winrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        winrow,
    );
    tv_dict_add_nr(
        d,
        b"wincol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        wincol,
    );
    tv_dict_add_nr(
        d,
        b"line\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        lnum as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"column\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        column,
    );
    tv_dict_add_nr(
        d,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        coladd as varnumber_T,
    );
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
