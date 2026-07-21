use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BorderTextType, BufUpdateCallbacks, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, CharBoundsOff, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative,
    GridView, Integer, Intersection, LuaRef, MHPutStatus, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_glyph, Set_int64_t, Set_uint32_t,
    Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal, Timestamp, UIExtension,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, int8_t, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T,
    size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strnlen(__string: *const ::core::ffi::c_char, __maxlen: size_t) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn mh_clear(h: *mut MapHash);
    fn mh_put_glyph(set: *mut Set_glyph, key: String_0, new: *mut MHPutStatus) -> uint32_t;
    fn decor_check_invalid_glyphs();
    fn next_virt_text_chunk(
        vt: VirtText,
        pos: *mut size_t,
        attr: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static firstwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static full_screen: GlobalCell<bool>;
    static exmode_active: GlobalCell<bool>;
    static default_grid: GlobalCell<ScreenGrid>;
    static resizing_screen: GlobalCell<bool>;
    static linebuf_char: GlobalCell<*mut schar_T>;
    static linebuf_attr: GlobalCell<*mut sattr_T>;
    static linebuf_vcol: GlobalCell<*mut colnr_T>;
    static linebuf_scratch: GlobalCell<*mut ::core::ffi::c_char>;
    static p_arshape: GlobalCell<::core::ffi::c_int>;
    static rdb_flags: GlobalCell<::core::ffi::c_uint>;
    static p_tbidi: GlobalCell<::core::ffi::c_int>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn hl_apply_winblend(winbl: ::core::ffi::c_int, attr: ::core::ffi::c_int)
        -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2cells_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptrlen2schar(
        p: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        firstc: *mut ::core::ffi::c_int,
    ) -> schar_T;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_cp_bounds(
        base: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> CharBoundsOff;
    fn ui_line(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        invalid_row: bool,
        startcol: ::core::ffi::c_int,
        endcol: ::core::ffi::c_int,
        clearcol: ::core::ffi::c_int,
        clearattr: ::core::ffi::c_int,
        wrap: bool,
    );
    fn ui_grid_cursor_goto(
        grid_handle: handle_T,
        new_row: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn ui_check_cursor_grid(grid_handle: handle_T);
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn check_chars_options() -> *const ::core::ffi::c_char;
}
pub type sscratch_T = int32_t;
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
pub const HLF_COUNT: C2Rust_Unnamed_12 = 76;
pub const HLF_PRE: C2Rust_Unnamed_12 = 75;
pub const HLF_OK: C2Rust_Unnamed_12 = 74;
pub const HLF_SO: C2Rust_Unnamed_12 = 73;
pub const HLF_SE: C2Rust_Unnamed_12 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_12 = 71;
pub const HLF_TS: C2Rust_Unnamed_12 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_12 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_12 = 68;
pub const HLF_CU: C2Rust_Unnamed_12 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_12 = 66;
pub const HLF_WBR: C2Rust_Unnamed_12 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_12 = 64;
pub const HLF_MSG: C2Rust_Unnamed_12 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_12 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_12 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_12 = 60;
pub const HLF_0: C2Rust_Unnamed_12 = 59;
pub const HLF_QFL: C2Rust_Unnamed_12 = 58;
pub const HLF_MC: C2Rust_Unnamed_12 = 57;
pub const HLF_CUL: C2Rust_Unnamed_12 = 56;
pub const HLF_CUC: C2Rust_Unnamed_12 = 55;
pub const HLF_TPF: C2Rust_Unnamed_12 = 54;
pub const HLF_TPS: C2Rust_Unnamed_12 = 53;
pub const HLF_TP: C2Rust_Unnamed_12 = 52;
pub const HLF_PBR: C2Rust_Unnamed_12 = 51;
pub const HLF_PST: C2Rust_Unnamed_12 = 50;
pub const HLF_PSB: C2Rust_Unnamed_12 = 49;
pub const HLF_PSX: C2Rust_Unnamed_12 = 48;
pub const HLF_PNX: C2Rust_Unnamed_12 = 47;
pub const HLF_PSK: C2Rust_Unnamed_12 = 46;
pub const HLF_PNK: C2Rust_Unnamed_12 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_12 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_12 = 43;
pub const HLF_PSI: C2Rust_Unnamed_12 = 42;
pub const HLF_PNI: C2Rust_Unnamed_12 = 41;
pub const HLF_SPL: C2Rust_Unnamed_12 = 40;
pub const HLF_SPR: C2Rust_Unnamed_12 = 39;
pub const HLF_SPC: C2Rust_Unnamed_12 = 38;
pub const HLF_SPB: C2Rust_Unnamed_12 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_12 = 36;
pub const HLF_SC: C2Rust_Unnamed_12 = 35;
pub const HLF_TXA: C2Rust_Unnamed_12 = 34;
pub const HLF_TXD: C2Rust_Unnamed_12 = 33;
pub const HLF_DED: C2Rust_Unnamed_12 = 32;
pub const HLF_CHD: C2Rust_Unnamed_12 = 31;
pub const HLF_ADD: C2Rust_Unnamed_12 = 30;
pub const HLF_FC: C2Rust_Unnamed_12 = 29;
pub const HLF_FL: C2Rust_Unnamed_12 = 28;
pub const HLF_WM: C2Rust_Unnamed_12 = 27;
pub const HLF_W: C2Rust_Unnamed_12 = 26;
pub const HLF_VNC: C2Rust_Unnamed_12 = 25;
pub const HLF_V: C2Rust_Unnamed_12 = 24;
pub const HLF_T: C2Rust_Unnamed_12 = 23;
pub const HLF_VSP: C2Rust_Unnamed_12 = 22;
pub const HLF_C: C2Rust_Unnamed_12 = 21;
pub const HLF_SNC: C2Rust_Unnamed_12 = 20;
pub const HLF_S: C2Rust_Unnamed_12 = 19;
pub const HLF_R: C2Rust_Unnamed_12 = 18;
pub const HLF_CLF: C2Rust_Unnamed_12 = 17;
pub const HLF_CLS: C2Rust_Unnamed_12 = 16;
pub const HLF_CLN: C2Rust_Unnamed_12 = 15;
pub const HLF_LNB: C2Rust_Unnamed_12 = 14;
pub const HLF_LNA: C2Rust_Unnamed_12 = 13;
pub const HLF_N: C2Rust_Unnamed_12 = 12;
pub const HLF_CM: C2Rust_Unnamed_12 = 11;
pub const HLF_M: C2Rust_Unnamed_12 = 10;
pub const HLF_LC: C2Rust_Unnamed_12 = 9;
pub const HLF_L: C2Rust_Unnamed_12 = 8;
pub const HLF_I: C2Rust_Unnamed_12 = 7;
pub const HLF_E: C2Rust_Unnamed_12 = 6;
pub const HLF_D: C2Rust_Unnamed_12 = 5;
pub const HLF_AT: C2Rust_Unnamed_12 = 4;
pub const HLF_TERM: C2Rust_Unnamed_12 = 3;
pub const HLF_EOB: C2Rust_Unnamed_12 = 2;
pub const HLF_8: C2Rust_Unnamed_12 = 1;
pub const HLF_NONE: C2Rust_Unnamed_12 = 0;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const SLF_INC_VCOL: C2Rust_Unnamed_13 = 4;
pub const SLF_WRAP: C2Rust_Unnamed_13 = 2;
pub const SLF_RIGHTLEFT: C2Rust_Unnamed_13 = 1;
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_14 = 4;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_14 = 8;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kOptRdbFlagFlush: C2Rust_Unnamed_14 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_14 = 16;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_14 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_14 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_glyph = Set_glyph {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static linebuf_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static glyph_cache: GlobalCell<Set_glyph> = GlobalCell::new(SET_INIT);
#[no_mangle]
pub unsafe extern "C" fn grid_adjust(
    mut grid: *mut GridView,
    mut row_off: *mut ::core::ffi::c_int,
    mut col_off: *mut ::core::ffi::c_int,
) -> *mut ScreenGrid {
    *row_off += (*grid).row_offset;
    *col_off += (*grid).col_offset;
    return (*grid).target;
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_str(mut str: *const ::core::ffi::c_char) -> schar_T {
    if str.is_null() {
        return 0 as schar_T;
    }
    return schar_from_buf(str, strlen(str));
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_buf(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) -> schar_T {
    '_c2rust_label: {
        if len < 32 as size_t {
        } else {
            __assert_fail(
                b"len < MAX_SCHAR_SIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                85 as ::core::ffi::c_uint,
                b"schar_T schar_from_buf(const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if len <= 4 as size_t {
        let mut sc: schar_T = 0 as schar_T;
        memcpy(
            &raw mut sc as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            len,
        );
        return sc;
    } else {
        let mut str: String_0 = String_0 {
            data: buf as *mut ::core::ffi::c_char,
            size: len,
        };
        let mut status: MHPutStatus = kMHExisting;
        let mut idx: uint32_t = mh_put_glyph(glyph_cache.ptr(), str, &raw mut status);
        '_c2rust_label_0: {
            if idx < 0xffffff as uint32_t {
            } else {
                __assert_fail(
                    b"idx < 0xFFFFFF\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    95 as ::core::ffi::c_uint,
                    b"schar_T schar_from_buf(const char *, size_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return (0xff as schar_T).wrapping_add((idx as schar_T) << 8 as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn schar_cache_clear_if_full() -> bool {
    if (*glyph_cache.ptr()).h.n_keys
        > ((1 as ::core::ffi::c_int) << 21 as ::core::ffi::c_int) as uint32_t
    {
        schar_cache_clear();
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn schar_cache_clear() {
    decor_check_invalid_glyphs();
    mh_clear(&raw mut (*glyph_cache.ptr()).h);
    if !check_chars_options().is_null() {
        abort();
    }
}
#[no_mangle]
pub unsafe extern "C" fn schar_high(mut sc: schar_T) -> bool {
    return sc & 0xff as schar_T == 0xff as schar_T;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get(
    mut buf_out: *mut ::core::ffi::c_char,
    mut sc: schar_T,
) -> size_t {
    let mut len: size_t = schar_get_adv(&raw mut buf_out, sc);
    *buf_out = NUL as ::core::ffi::c_char;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_adv(
    mut buf_out: *mut *mut ::core::ffi::c_char,
    mut sc: schar_T,
) -> size_t {
    let mut len: size_t = 0;
    if schar_high(sc) {
        let mut idx: uint32_t = sc as uint32_t >> 8 as ::core::ffi::c_int;
        '_c2rust_label: {
            if idx < (*glyph_cache.ptr()).h.n_keys {
            } else {
                __assert_fail(
                    b"idx < glyph_cache.h.n_keys\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    163 as ::core::ffi::c_uint,
                    b"size_t schar_get_adv(char **, schar_T)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        len = strlen((*glyph_cache.ptr()).keys.offset(idx as isize));
        memcpy(
            *buf_out as *mut ::core::ffi::c_void,
            (*glyph_cache.ptr()).keys.offset(idx as isize) as *const ::core::ffi::c_void,
            len,
        );
    } else {
        len = strnlen(&raw mut sc as *mut ::core::ffi::c_char, 4 as size_t);
        memcpy(
            *buf_out as *mut ::core::ffi::c_void,
            &raw mut sc as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            len,
        );
    }
    *buf_out = (*buf_out).offset(len as isize);
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn schar_len(mut sc: schar_T) -> size_t {
    if schar_high(sc) {
        let mut idx: uint32_t = sc as uint32_t >> 8 as ::core::ffi::c_int;
        '_c2rust_label: {
            if idx < (*glyph_cache.ptr()).h.n_keys {
            } else {
                __assert_fail(
                    b"idx < glyph_cache.h.n_keys\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    178 as ::core::ffi::c_uint,
                    b"size_t schar_len(schar_T)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return strlen((*glyph_cache.ptr()).keys.offset(idx as isize));
    } else {
        return strnlen(&raw mut sc as *mut ::core::ffi::c_char, 4 as size_t);
    };
}
#[no_mangle]
pub unsafe extern "C" fn schar_cells(mut sc: schar_T) -> ::core::ffi::c_int {
    if sc < 0x80 as schar_T {
        return 1 as ::core::ffi::c_int;
    }
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    return utf_ptr2cells(&raw mut sc_buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn schar_get_first_byte(mut sc: schar_T) -> ::core::ffi::c_char {
    '_c2rust_label: {
        if !(schar_high(sc) as ::core::ffi::c_int != 0
            && sc >> 8 as ::core::ffi::c_int >= (*glyph_cache.ptr()).h.n_keys)
        {
        } else {
            __assert_fail(
                b"!(schar_high(sc) && schar_idx(sc) >= glyph_cache.h.n_keys)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                206 as ::core::ffi::c_uint,
                b"char schar_get_first_byte(schar_T)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return (if schar_high(sc) as ::core::ffi::c_int != 0 {
        *(*glyph_cache.ptr())
            .keys
            .offset((sc >> 8 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
    } else {
        *(&raw mut sc as *mut ::core::ffi::c_char) as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_first_codepoint(mut sc: schar_T) -> ::core::ffi::c_int {
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    return utf_ptr2char(&raw mut sc_buf as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_ascii(mut sc: schar_T) -> ::core::ffi::c_char {
    return (if sc < 0x80 as schar_T {
        sc as ::core::ffi::c_char as ::core::ffi::c_int
    } else {
        NUL
    }) as ::core::ffi::c_char;
}
unsafe extern "C" fn schar_in_arabic_block(mut sc: schar_T) -> bool {
    let mut first_byte: ::core::ffi::c_char = schar_get_first_byte(sc);
    return first_byte as uint8_t as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int
        == 0xd8 as ::core::ffi::c_int;
}
unsafe extern "C" fn schar_get_first_two_codepoints(
    mut sc: schar_T,
    mut c0: *mut ::core::ffi::c_int,
    mut c1: *mut ::core::ffi::c_int,
) {
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    *c0 = utf_ptr2char(&raw mut sc_buf as *mut ::core::ffi::c_char);
    let mut len: ::core::ffi::c_int = utf_ptr2len(&raw mut sc_buf as *mut ::core::ffi::c_char);
    if *c0 == NUL {
        *c1 = NUL;
    } else {
        *c1 = utf_ptr2char((&raw mut sc_buf as *mut ::core::ffi::c_char).offset(len as isize));
    };
}
#[no_mangle]
pub unsafe extern "C" fn line_do_arabic_shape(mut buf: *mut schar_T, mut cols: ::core::ffi::c_int) {
    let mut c1new: ::core::ffi::c_int = 0;
    let mut c0new: ::core::ffi::c_int = 0;
    let mut scbuf: [::core::ffi::c_char; 32] = [0; 32];
    let mut scbuf_new: [::core::ffi::c_char; 32] = [0; 32];
    let mut len: size_t = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut rest: size_t = 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < cols {
        if schar_in_arabic_block(*buf.offset(i as isize)) {
            break;
        }
        i += 1;
    }
    if i == cols {
        return;
    }
    let mut c0prev: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut c0: ::core::ffi::c_int = 0;
    let mut c1: ::core::ffi::c_int = 0;
    schar_get_first_two_codepoints(*buf.offset(i as isize), &raw mut c0, &raw mut c1);
    while i < cols {
        let mut c0next: ::core::ffi::c_int = 0;
        let mut c1next: ::core::ffi::c_int = 0;
        schar_get_first_two_codepoints(
            if (i + 1 as ::core::ffi::c_int) < cols {
                *buf.offset((i + 1 as ::core::ffi::c_int) as isize)
            } else {
                0 as schar_T
            },
            &raw mut c0next,
            &raw mut c1next,
        );
        if c0 & 0xff00 as ::core::ffi::c_int == 0x600 as ::core::ffi::c_int {
            c1new = c1;
            c0new = crate::src::nvim::arabic::arabic_shape(c0, &mut c1new, c0next, c1next, c0prev);
            if !(c0new == c0 && c1new == c1) {
                scbuf = [0; 32];
                schar_get(
                    &raw mut scbuf as *mut ::core::ffi::c_char,
                    *buf.offset(i as isize),
                );
                scbuf_new = [0; 32];
                len =
                    utf_char2bytes(c0new, &raw mut scbuf_new as *mut ::core::ffi::c_char) as size_t;
                if c1new != 0 {
                    len = len.wrapping_add(utf_char2bytes(
                        c1new,
                        (&raw mut scbuf_new as *mut ::core::ffi::c_char).offset(len as isize),
                    ) as size_t);
                }
                off = utf_char2len(c0)
                    + (if c1 != 0 {
                        utf_char2len(c1)
                    } else {
                        0 as ::core::ffi::c_int
                    });
                rest = strlen((&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize));
                if rest.wrapping_add(len).wrapping_add(1 as size_t) > MAX_SCHAR_SIZE as size_t {
                    rest = rest.wrapping_sub(
                        (utf_cp_bounds(
                            (&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize),
                            (&raw mut scbuf as *mut ::core::ffi::c_char)
                                .offset(off as isize)
                                .offset(rest as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        )
                        .begin_off as size_t)
                            .wrapping_add(1 as size_t),
                    );
                }
                memcpy(
                    (&raw mut scbuf_new as *mut ::core::ffi::c_char).offset(len as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize)
                        as *const ::core::ffi::c_void,
                    rest,
                );
                *buf.offset(i as isize) = schar_from_buf(
                    &raw mut scbuf_new as *mut ::core::ffi::c_char,
                    len.wrapping_add(rest),
                );
            }
        }
        c0prev = c0;
        c0 = c0next;
        c1 = c1next;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear_line(
    mut grid: *mut ScreenGrid,
    mut off: size_t,
    mut width: ::core::ffi::c_int,
    mut valid: bool,
) {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < width {
        *(*grid)
            .chars
            .offset(off.wrapping_add(col as size_t) as isize) =
            ' ' as ::core::ffi::c_int as schar_T;
        col += 1;
    }
    let mut fill: ::core::ffi::c_int = if valid as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    memset(
        (*grid).attrs.offset(off as isize) as *mut ::core::ffi::c_void,
        fill,
        (width as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    memset(
        (*grid).vcols.offset(off as isize) as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        (width as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_invalidate(mut grid: *mut ScreenGrid) {
    memset(
        (*grid).attrs as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        ::core::mem::size_of::<sattr_T>()
            .wrapping_mul((*grid).rows as size_t)
            .wrapping_mul((*grid).cols as size_t),
    );
}
unsafe extern "C" fn grid_invalid_row(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
) -> bool {
    return *(*grid)
        .attrs
        .offset(*(*grid).line_offset.offset(row as isize) as isize)
        < 0 as sattr_T;
}
#[no_mangle]
pub unsafe extern "C" fn grid_getchar(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut attrp: *mut ::core::ffi::c_int,
) -> schar_T {
    if (*grid).chars.is_null() || row >= (*grid).rows || col >= (*grid).cols {
        return NUL as schar_T;
    }
    let mut off: size_t = (*(*grid).line_offset.offset(row as isize)).wrapping_add(col as size_t);
    if !attrp.is_null() {
        *attrp = *(*grid).attrs.offset(off as isize) as ::core::ffi::c_int;
    }
    return *(*grid).chars.offset(off as isize);
}
static grid_line_grid: GlobalCell<*mut ScreenGrid> =
    GlobalCell::new(::core::ptr::null_mut::<ScreenGrid>());
static grid_line_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static grid_line_coloff: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_maxcol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_first: GlobalCell<::core::ffi::c_int> = GlobalCell::new(INT_MAX);
static grid_line_last: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_clear_to: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_bg_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_clear_attr: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static grid_line_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn grid_line_start(mut view: *mut GridView, mut row: ::core::ffi::c_int) {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = grid_adjust(view, &raw mut row, &raw mut col);
    screengrid_line_start(grid, row, col);
}
#[no_mangle]
pub unsafe extern "C" fn screengrid_line_start(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    grid_line_maxcol.set((*grid).cols);
    '_c2rust_label: {
        if (*grid_line_grid.ptr()).is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                373 as ::core::ffi::c_uint,
                b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    grid_line_row.set(row);
    grid_line_grid.set(grid);
    grid_line_coloff.set(col);
    grid_line_first.set(linebuf_size.get() as ::core::ffi::c_int);
    grid_line_maxcol.set(
        if grid_line_maxcol.get() < (*grid).cols - grid_line_coloff.get() {
            grid_line_maxcol.get()
        } else {
            (*grid).cols - grid_line_coloff.get()
        },
    );
    grid_line_last.set(0 as ::core::ffi::c_int);
    grid_line_clear_to.set(0 as ::core::ffi::c_int);
    grid_line_bg_attr.set(0 as ::core::ffi::c_int);
    grid_line_clear_attr.set(0 as ::core::ffi::c_int);
    grid_line_flags.set(0 as ::core::ffi::c_int);
    '_c2rust_label_0: {
        if grid_line_maxcol.get() as size_t <= linebuf_size.get() {
        } else {
            __assert_fail(
                b"(size_t)grid_line_maxcol <= linebuf_size\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                385 as ::core::ffi::c_uint,
                b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if full_screen.get() as ::core::ffi::c_int != 0
        && rdb_flags.get() & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        '_c2rust_label_1: {
            if !(*linebuf_char.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"linebuf_char\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    388 as ::core::ffi::c_uint,
                    b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memset(
            linebuf_char.get() as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<schar_T>().wrapping_mul(linebuf_size.get()),
        );
        memset(
            linebuf_attr.get() as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<sattr_T>().wrapping_mul(linebuf_size.get()),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_getchar(
    mut col: ::core::ffi::c_int,
    mut attr: *mut ::core::ffi::c_int,
) -> schar_T {
    if col < grid_line_maxcol.get() {
        col += grid_line_coloff.get();
        let mut off: size_t = (*(*grid_line_grid.get())
            .line_offset
            .offset(grid_line_row.get() as isize))
        .wrapping_add(col as size_t);
        if !attr.is_null() {
            *attr = *(*grid_line_grid.get()).attrs.offset(off as isize) as ::core::ffi::c_int;
        }
        return *(*grid_line_grid.get()).chars.offset(off as isize);
    } else {
        return ' ' as ::core::ffi::c_int as schar_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_put_schar(
    mut col: ::core::ffi::c_int,
    mut schar: schar_T,
    mut attr: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if !(*grid_line_grid.ptr()).is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                418 as ::core::ffi::c_uint,
                b"void grid_line_put_schar(int, schar_T, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if col >= grid_line_maxcol.get() {
        return;
    }
    *(*linebuf_char.ptr()).offset(col as isize) = schar;
    *(*linebuf_attr.ptr()).offset(col as isize) = attr as sattr_T;
    grid_line_first.set(if grid_line_first.get() < col {
        grid_line_first.get()
    } else {
        col
    });
    grid_line_last.set(if grid_line_last.get() > col + 1 as ::core::ffi::c_int {
        grid_line_last.get()
    } else {
        col + 1 as ::core::ffi::c_int
    });
    *(*linebuf_vcol.ptr()).offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_puts(
    mut col: ::core::ffi::c_int,
    mut text: *const ::core::ffi::c_char,
    mut textlen: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ptr: *const ::core::ffi::c_char = text;
    let mut len: ::core::ffi::c_int = textlen;
    '_c2rust_label: {
        if !(*grid_line_grid.ptr()).is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                444 as ::core::ffi::c_uint,
                b"int grid_line_puts(int, const char *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut start_col: ::core::ffi::c_int = col;
    let max_col: ::core::ffi::c_int = grid_line_maxcol.get();
    while col < max_col
        && (len < 0 as ::core::ffi::c_int || (ptr.offset_from(text) as ::core::ffi::c_int) < len)
        && *ptr as ::core::ffi::c_int != NUL
    {
        let mut mbyte_blen: ::core::ffi::c_int = 0;
        if len >= 0 as ::core::ffi::c_int {
            let mut maxlen: ::core::ffi::c_int =
                text.offset(len as isize).offset_from(ptr) as ::core::ffi::c_int;
            mbyte_blen = utfc_ptr2len_len(ptr, maxlen);
            if mbyte_blen > maxlen {
                mbyte_blen = 1 as ::core::ffi::c_int;
            }
        } else {
            mbyte_blen = utfc_ptr2len(ptr);
        }
        let mut firstc: ::core::ffi::c_int = 0;
        let mut schar: schar_T = utfc_ptrlen2schar(ptr, mbyte_blen, &raw mut firstc);
        let mut mbyte_cells: ::core::ffi::c_int = utf_ptr2cells_len(ptr, mbyte_blen);
        if mbyte_cells > 2 as ::core::ffi::c_int || schar == 0 as schar_T {
            mbyte_cells = 1 as ::core::ffi::c_int;
            schar = schar_from_char(0xfffd as ::core::ffi::c_int);
        }
        if col + mbyte_cells > max_col {
            schar = '>' as ::core::ffi::c_int as schar_T;
            mbyte_cells = 1 as ::core::ffi::c_int;
        }
        if ptr == text
            && col > grid_line_first.get()
            && col < grid_line_last.get()
            && *(*linebuf_char.ptr()).offset(col as isize) == 0 as schar_T
        {
            *(*linebuf_char.ptr()).offset((col - 1 as ::core::ffi::c_int) as isize) =
                '>' as ::core::ffi::c_int as schar_T;
        }
        *(*linebuf_char.ptr()).offset(col as isize) = schar;
        *(*linebuf_attr.ptr()).offset(col as isize) = attr as sattr_T;
        *(*linebuf_vcol.ptr()).offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
        if mbyte_cells == 2 as ::core::ffi::c_int {
            *(*linebuf_char.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize) = 0 as schar_T;
            *(*linebuf_attr.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize) =
                attr as sattr_T;
            *(*linebuf_vcol.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize) =
                -1 as ::core::ffi::c_int as colnr_T;
        }
        col += mbyte_cells;
        ptr = ptr.offset(mbyte_blen as isize);
    }
    if col > start_col {
        grid_line_first.set(if grid_line_first.get() < start_col {
            grid_line_first.get()
        } else {
            start_col
        });
        grid_line_last.set(if grid_line_last.get() > col {
            grid_line_last.get()
        } else {
            col
        });
    }
    return col - start_col;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_fill(
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut sc: schar_T,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    end_col = if end_col < grid_line_maxcol.get() {
        end_col
    } else {
        grid_line_maxcol.get()
    };
    if start_col >= end_col {
        return end_col;
    }
    let mut col: ::core::ffi::c_int = start_col;
    while col < end_col {
        *(*linebuf_char.ptr()).offset(col as isize) = sc;
        *(*linebuf_attr.ptr()).offset(col as isize) = attr as sattr_T;
        *(*linebuf_vcol.ptr()).offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
        col += 1;
    }
    grid_line_first.set(if grid_line_first.get() < start_col {
        grid_line_first.get()
    } else {
        start_col
    });
    grid_line_last.set(if grid_line_last.get() > end_col {
        grid_line_last.get()
    } else {
        end_col
    });
    return end_col;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_clear_end(
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut bg_attr: ::core::ffi::c_int,
    mut clear_attr: ::core::ffi::c_int,
) {
    if grid_line_first.get() > start_col {
        grid_line_first.set(start_col);
        grid_line_last.set(start_col);
    }
    grid_line_clear_to.set(end_col);
    grid_line_bg_attr.set(bg_attr);
    grid_line_clear_attr.set(clear_attr);
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_cursor_goto(mut col: ::core::ffi::c_int) {
    ui_grid_cursor_goto((*grid_line_grid.get()).handle, grid_line_row.get(), col);
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_mirror(mut width: ::core::ffi::c_int) {
    grid_line_clear_to.set(if grid_line_last.get() > grid_line_clear_to.get() {
        grid_line_last.get()
    } else {
        grid_line_clear_to.get()
    });
    if grid_line_first.get() >= grid_line_clear_to.get() {
        return;
    }
    linebuf_mirror(
        grid_line_first.ptr(),
        grid_line_last.ptr(),
        grid_line_clear_to.ptr(),
        width,
    );
    (*grid_line_flags.ptr()) |= SLF_RIGHTLEFT as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn linebuf_mirror(
    mut firstp: *mut ::core::ffi::c_int,
    mut lastp: *mut ::core::ffi::c_int,
    mut clearp: *mut ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut first: ::core::ffi::c_int = *firstp;
    let mut last: ::core::ffi::c_int = *lastp;
    let mut n: size_t = (last - first) as size_t;
    let mut mirror: ::core::ffi::c_int = width - 1 as ::core::ffi::c_int;
    let mut scratch_char: *mut schar_T = linebuf_scratch.get() as *mut schar_T;
    memcpy(
        scratch_char.offset(first as isize) as *mut ::core::ffi::c_void,
        (*linebuf_char.ptr()).offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    let mut col: ::core::ffi::c_int = first;
    while col < last {
        let mut rev: ::core::ffi::c_int = mirror - col;
        if (col + 1 as ::core::ffi::c_int) < last
            && *scratch_char.offset((col + 1 as ::core::ffi::c_int) as isize) == 0 as schar_T
        {
            *(*linebuf_char.ptr()).offset((rev - 1 as ::core::ffi::c_int) as isize) =
                *scratch_char.offset(col as isize);
            *(*linebuf_char.ptr()).offset(rev as isize) = 0 as schar_T;
            col += 1;
        } else {
            *(*linebuf_char.ptr()).offset(rev as isize) = *scratch_char.offset(col as isize);
        }
        col += 1;
    }
    let mut scratch_attr: *mut sattr_T = linebuf_scratch.get() as *mut sattr_T;
    memcpy(
        scratch_attr.offset(first as isize) as *mut ::core::ffi::c_void,
        (*linebuf_attr.ptr()).offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    let mut col_0: ::core::ffi::c_int = first;
    while col_0 < last {
        *(*linebuf_attr.ptr()).offset((mirror - col_0) as isize) =
            *scratch_attr.offset(col_0 as isize);
        col_0 += 1;
    }
    let mut scratch_vcol: *mut colnr_T = linebuf_scratch.get() as *mut colnr_T;
    memcpy(
        scratch_vcol.offset(first as isize) as *mut ::core::ffi::c_void,
        (*linebuf_vcol.ptr()).offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
    let mut col_1: ::core::ffi::c_int = first;
    while col_1 < last {
        *(*linebuf_vcol.ptr()).offset((mirror - col_1) as isize) =
            *scratch_vcol.offset(col_1 as isize);
        col_1 += 1;
    }
    *firstp = width - *clearp;
    *clearp = width - first;
    *lastp = width - last;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_flush() {
    let mut grid: *mut ScreenGrid = grid_line_grid.get();
    grid_line_grid.set(::core::ptr::null_mut::<ScreenGrid>());
    grid_line_clear_to.set(if grid_line_last.get() > grid_line_clear_to.get() {
        grid_line_last.get()
    } else {
        grid_line_clear_to.get()
    });
    '_c2rust_label: {
        if grid_line_clear_to.get() <= grid_line_maxcol.get() {
        } else {
            __assert_fail(
                b"grid_line_clear_to <= grid_line_maxcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                595 as ::core::ffi::c_uint,
                b"void grid_line_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if grid_line_first.get() >= grid_line_clear_to.get() {
        return;
    }
    grid_put_linebuf(
        grid,
        grid_line_row.get(),
        grid_line_coloff.get(),
        grid_line_first.get(),
        grid_line_last.get(),
        grid_line_clear_to.get(),
        grid_line_bg_attr.get(),
        grid_line_clear_attr.get(),
        -1 as colnr_T,
        grid_line_flags.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_flush_if_valid_row() {
    if grid_line_row.get() < 0 as ::core::ffi::c_int
        || grid_line_row.get() >= (*grid_line_grid.get()).rows
    {
        if rdb_flags.get() & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            abort();
        } else {
            grid_line_grid.set(::core::ptr::null_mut::<ScreenGrid>());
            return;
        }
    }
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear(
    mut grid: *mut GridView,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) {
    let mut row: ::core::ffi::c_int = start_row;
    while row < end_row {
        grid_line_start(grid, row);
        end_col = if end_col < grid_line_maxcol.get() {
            end_col
        } else {
            grid_line_maxcol.get()
        };
        if grid_line_row.get() >= (*grid_line_grid.get()).rows || start_col >= end_col {
            grid_line_grid.set(::core::ptr::null_mut::<ScreenGrid>());
            return;
        }
        grid_line_clear_end(start_col, end_col, attr, 0 as ::core::ffi::c_int);
        grid_line_flush();
        row += 1;
    }
}
unsafe extern "C" fn grid_char_needs_redraw(
    mut grid: *mut ScreenGrid,
    mut col: ::core::ffi::c_int,
    mut off_to: size_t,
    mut cols: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return (cols > 0 as ::core::ffi::c_int
        && (*(*linebuf_char.ptr()).offset(col as isize) != *(*grid).chars.offset(off_to as isize)
            || *(*linebuf_attr.ptr()).offset(col as isize)
                != *(*grid).attrs.offset(off_to as isize)
            || cols > 1 as ::core::ffi::c_int
                && *(*linebuf_char.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize)
                    == 0 as schar_T
                && *(*linebuf_char.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize)
                    != *(*grid)
                        .chars
                        .offset(off_to.wrapping_add(1 as size_t) as isize)
            || exmode_active.get() as ::core::ffi::c_int != 0
            || rdb_flags.get() & kOptRdbFlagNodelta as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_put_linebuf(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut coloff: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut endcol: ::core::ffi::c_int,
    mut clear_width: ::core::ffi::c_int,
    mut bg_attr: ::core::ffi::c_int,
    mut clear_attr: ::core::ffi::c_int,
    mut last_vcol: colnr_T,
    mut flags: ::core::ffi::c_int,
) {
    let mut redraw_next: bool = false;
    let mut clear_next: bool = false_0 != 0;
    '_c2rust_label: {
        if 0 as ::core::ffi::c_int <= row && row < (*grid).rows {
        } else {
            __assert_fail(
                b"0 <= row && row < grid->rows\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                672 as ::core::ffi::c_uint,
                b"void grid_put_linebuf(ScreenGrid *, int, int, int, int, int, int, int, colnr_T, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if endcol > (*grid).cols {
        endcol = (*grid).cols;
    }
    if (*grid).chars.is_null() || row >= (*grid).rows || coloff >= (*grid).cols {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"grid_put_linebuf\0".as_ptr() as *const ::core::ffi::c_char,
            681 as ::core::ffi::c_int,
            true_0 != 0,
            b"invalid state, skipped\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut invalid_row: bool = grid != default_grid.ptr()
        && grid_invalid_row(grid, row) as ::core::ffi::c_int != 0
        && col == 0 as ::core::ffi::c_int;
    let mut off_to: size_t =
        (*(*grid).line_offset.offset(row as isize)).wrapping_add(coloff as size_t);
    let max_off_to: size_t =
        (*(*grid).line_offset.offset(row as isize)).wrapping_add((*grid).cols as size_t);
    if col > 0 as ::core::ffi::c_int
        && *(*grid)
            .chars
            .offset(off_to.wrapping_add(col as size_t) as isize)
            == 0 as schar_T
    {
        *(*linebuf_char.ptr()).offset((col - 1 as ::core::ffi::c_int) as isize) =
            '>' as ::core::ffi::c_int as schar_T;
        *(*linebuf_attr.ptr()).offset((col - 1 as ::core::ffi::c_int) as isize) = *(*grid)
            .attrs
            .offset(off_to.wrapping_add(col as size_t).wrapping_sub(1 as size_t) as isize);
        col -= 1;
    }
    let mut clear_start: ::core::ffi::c_int = endcol;
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0 {
        clear_start = col;
        col = endcol;
        endcol = clear_width;
        clear_width = col;
    }
    if p_arshape.get() != 0 && p_tbidi.get() == 0 && endcol > col {
        line_do_arabic_shape((*linebuf_char.ptr()).offset(col as isize), endcol - col);
    }
    if bg_attr != 0 {
        let mut c: ::core::ffi::c_int = col;
        while c < endcol {
            *(*linebuf_attr.ptr()).offset(c as isize) = hl_combine_attr(
                bg_attr,
                *(*linebuf_attr.ptr()).offset(c as isize) as ::core::ffi::c_int,
            ) as sattr_T;
            c += 1;
        }
    }
    redraw_next =
        grid_char_needs_redraw(grid, col, off_to.wrapping_add(col as size_t), endcol - col) != 0;
    let mut start_dirty: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut end_dirty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < endcol {
        let mut char_cells: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if (col + 1 as ::core::ffi::c_int) < endcol
            && *(*linebuf_char.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize)
                == 0 as schar_T
        {
            char_cells = 2 as ::core::ffi::c_int;
        }
        let mut redraw_this: bool = redraw_next;
        let mut off: size_t = off_to.wrapping_add(col as size_t);
        redraw_next = grid_char_needs_redraw(
            grid,
            col + char_cells,
            off.wrapping_add(char_cells as size_t),
            endcol - col - char_cells,
        ) != 0;
        if redraw_this {
            if start_dirty == -1 as ::core::ffi::c_int {
                start_dirty = col;
            }
            end_dirty = col + char_cells;
            if col + char_cells == endcol
                && off.wrapping_add(char_cells as size_t) < max_off_to
                && *(*grid)
                    .chars
                    .offset(off.wrapping_add(char_cells as size_t) as isize)
                    == NUL as schar_T
            {
                clear_next = true_0 != 0;
            }
            *(*grid).chars.offset(off as isize) = *(*linebuf_char.ptr()).offset(col as isize);
            if char_cells == 2 as ::core::ffi::c_int {
                *(*grid).chars.offset(off.wrapping_add(1 as size_t) as isize) =
                    *(*linebuf_char.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize);
            }
            *(*grid).attrs.offset(off as isize) = *(*linebuf_attr.ptr()).offset(col as isize);
            if char_cells == 2 as ::core::ffi::c_int {
                *(*grid).attrs.offset(off.wrapping_add(1 as size_t) as isize) =
                    *(*linebuf_attr.ptr()).offset(col as isize);
            }
        }
        *(*grid).vcols.offset(off as isize) = *(*linebuf_vcol.ptr()).offset(col as isize);
        if char_cells == 2 as ::core::ffi::c_int {
            *(*grid).vcols.offset(off.wrapping_add(1 as size_t) as isize) =
                *(*linebuf_vcol.ptr()).offset((col + 1 as ::core::ffi::c_int) as isize);
        }
        col += char_cells;
    }
    if clear_next {
        *(*grid)
            .chars
            .offset(off_to.wrapping_add(col as size_t) as isize) =
            ' ' as ::core::ffi::c_int as schar_T;
        end_dirty += 1;
    }
    if off_to.wrapping_add(clear_width as size_t) < max_off_to
        && *(*grid)
            .chars
            .offset(off_to.wrapping_add(clear_width as size_t) as isize)
            == 0 as schar_T
    {
        clear_width += 1;
    }
    let mut clear_dirty_start: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut clear_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0 {
        col = clear_width - 1 as ::core::ffi::c_int;
        while col >= clear_start {
            let mut off_0: size_t = off_to.wrapping_add(col as size_t);
            *(*grid).vcols.offset(off_0 as isize) =
                if flags & SLF_INC_VCOL as ::core::ffi::c_int != 0 {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
            col -= 1;
        }
    }
    clear_attr = hl_combine_attr(bg_attr, clear_attr);
    col = clear_start;
    while col < clear_width {
        let mut off_1: size_t = off_to.wrapping_add(col as size_t);
        if *(*grid).chars.offset(off_1 as isize) != ' ' as ::core::ffi::c_int as schar_T
            || *(*grid).attrs.offset(off_1 as isize) != clear_attr as sattr_T
            || rdb_flags.get() & kOptRdbFlagNodelta as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
        {
            *(*grid).chars.offset(off_1 as isize) = ' ' as ::core::ffi::c_int as schar_T;
            *(*grid).attrs.offset(off_1 as isize) = clear_attr as sattr_T;
            if clear_dirty_start == -1 as ::core::ffi::c_int {
                clear_dirty_start = col;
            }
            clear_end = col + 1 as ::core::ffi::c_int;
        }
        if flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0 {
            *(*grid).vcols.offset(off_1 as isize) =
                if flags & SLF_INC_VCOL as ::core::ffi::c_int != 0 {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
        }
        col += 1;
    }
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0
        && start_dirty != -1 as ::core::ffi::c_int
        && clear_dirty_start != -1 as ::core::ffi::c_int
    {
        if (*grid).throttled as ::core::ffi::c_int != 0
            || clear_dirty_start >= start_dirty - 5 as ::core::ffi::c_int
        {
            start_dirty = clear_dirty_start;
        } else {
            ui_line(
                grid,
                row,
                invalid_row,
                coloff + clear_dirty_start,
                coloff + clear_dirty_start,
                coloff + clear_end,
                clear_attr,
                flags & SLF_WRAP as ::core::ffi::c_int != 0,
            );
        }
        clear_end = end_dirty;
    } else if start_dirty == -1 as ::core::ffi::c_int {
        start_dirty = clear_dirty_start;
        end_dirty = clear_dirty_start;
    } else if clear_end < end_dirty {
        clear_end = end_dirty;
    } else {
        end_dirty = endcol;
    }
    if clear_end > start_dirty {
        if !(*grid).throttled {
            ui_line(
                grid,
                row,
                invalid_row,
                coloff + start_dirty,
                coloff + end_dirty,
                coloff + clear_end,
                clear_attr,
                flags & SLF_WRAP as ::core::ffi::c_int != 0,
            );
        } else if !(*grid).dirty_col.is_null() {
            if clear_end > *(*grid).dirty_col.offset(row as isize) {
                *(*grid).dirty_col.offset(row as isize) = clear_end;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_alloc(
    mut grid: *mut ScreenGrid,
    mut rows: ::core::ffi::c_int,
    mut columns: ::core::ffi::c_int,
    mut copy: bool,
    mut valid: bool,
) {
    let mut new_row: ::core::ffi::c_int = 0;
    let mut ngrid: ScreenGrid = *grid;
    '_c2rust_label: {
        if rows >= 0 as ::core::ffi::c_int && columns >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"rows >= 0 && columns >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                846 as ::core::ffi::c_uint,
                b"void grid_alloc(ScreenGrid *, int, int, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut ncells: size_t = (rows as size_t).wrapping_mul(columns as size_t);
    ngrid.chars = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<schar_T>())) as *mut schar_T;
    ngrid.attrs = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<sattr_T>())) as *mut sattr_T;
    ngrid.vcols = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<colnr_T>())) as *mut colnr_T;
    memset(
        ngrid.vcols as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        ncells.wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
    ngrid.line_offset =
        xmalloc((rows as size_t).wrapping_mul(::core::mem::size_of::<size_t>())) as *mut size_t;
    ngrid.rows = rows;
    ngrid.cols = columns;
    new_row = 0 as ::core::ffi::c_int;
    while new_row < ngrid.rows {
        *ngrid.line_offset.offset(new_row as isize) =
            (new_row as size_t).wrapping_mul(ngrid.cols as size_t);
        grid_clear_line(
            &raw mut ngrid,
            *ngrid.line_offset.offset(new_row as isize),
            columns,
            valid,
        );
        if copy {
            if new_row < (*grid).rows && !(*grid).chars.is_null() {
                let mut len: ::core::ffi::c_int = if (*grid).cols < ngrid.cols {
                    (*grid).cols
                } else {
                    ngrid.cols
                };
                memmove(
                    ngrid
                        .chars
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .chars
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
                );
                memmove(
                    ngrid
                        .attrs
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .attrs
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
                );
                memmove(
                    ngrid
                        .vcols
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .vcols
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
                );
            }
        }
        new_row += 1;
    }
    grid_free(grid);
    *grid = ngrid;
    if linebuf_size.get() < columns as size_t {
        xfree(linebuf_char.get() as *mut ::core::ffi::c_void);
        xfree(linebuf_attr.get() as *mut ::core::ffi::c_void);
        xfree(linebuf_vcol.get() as *mut ::core::ffi::c_void);
        xfree(linebuf_scratch.get() as *mut ::core::ffi::c_void);
        linebuf_char.set(xmalloc(
            (columns as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
        ) as *mut schar_T);
        linebuf_attr.set(xmalloc(
            (columns as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
        ) as *mut sattr_T);
        linebuf_vcol.set(xmalloc(
            (columns as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
        ) as *mut colnr_T);
        linebuf_scratch.set(xmalloc(
            (columns as size_t).wrapping_mul(::core::mem::size_of::<sscratch_T>()),
        ) as *mut ::core::ffi::c_char);
        linebuf_size.set(columns as size_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_free(mut grid: *mut ScreenGrid) {
    xfree((*grid).chars as *mut ::core::ffi::c_void);
    xfree((*grid).attrs as *mut ::core::ffi::c_void);
    xfree((*grid).vcols as *mut ::core::ffi::c_void);
    xfree((*grid).line_offset as *mut ::core::ffi::c_void);
    (*grid).chars = ::core::ptr::null_mut::<schar_T>();
    (*grid).attrs = ::core::ptr::null_mut::<sattr_T>();
    (*grid).vcols = ::core::ptr::null_mut::<colnr_T>();
    (*grid).line_offset = ::core::ptr::null_mut::<size_t>();
}
#[no_mangle]
pub unsafe extern "C" fn win_grid_alloc(mut wp: *mut win_T) {
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let mut grid_allocated: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
    let mut total_rows: ::core::ffi::c_int = (*wp).w_height_outer;
    let mut total_cols: ::core::ffi::c_int = (*wp).w_width_outer;
    let mut want_allocation: bool = ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        || (*wp).w_floating as ::core::ffi::c_int != 0;
    let mut has_allocation: bool = !(*grid_allocated).chars.is_null();
    if (*wp).w_view_height > (*wp).w_lines_size {
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        xfree((*wp).w_lines as *mut ::core::ffi::c_void);
        (*wp).w_lines = xcalloc(
            ((*wp).w_view_height as size_t).wrapping_add(1 as size_t),
            ::core::mem::size_of::<wline_T>(),
        ) as *mut wline_T;
        (*wp).w_lines_size = (*wp).w_view_height;
    }
    let mut was_resized: bool = false_0 != 0;
    if want_allocation as ::core::ffi::c_int != 0
        && (!has_allocation
            || (*grid_allocated).rows != total_rows
            || (*grid_allocated).cols != total_cols)
    {
        grid_alloc(
            grid_allocated,
            total_rows,
            total_cols,
            (*wp).w_grid_alloc.valid,
            false_0 != 0,
        );
        (*grid_allocated).valid = true_0 != 0;
        if (*wp).w_floating as ::core::ffi::c_int != 0
            && (*wp).w_config.border as ::core::ffi::c_int != 0
        {
            (*wp).w_redr_border = true_0 != 0;
        }
        was_resized = true_0 != 0;
    } else if !want_allocation && has_allocation as ::core::ffi::c_int != 0 {
        grid_free(grid_allocated);
        (*grid_allocated).valid = false_0 != 0;
        was_resized = true_0 != 0;
    } else if want_allocation as ::core::ffi::c_int != 0
        && has_allocation as ::core::ffi::c_int != 0
        && !(*wp).w_grid_alloc.valid
    {
        grid_invalidate(grid_allocated);
        (*grid_allocated).valid = true_0 != 0;
    }
    if want_allocation {
        (*grid).target = grid_allocated;
        (*grid).row_offset = (*wp).w_winrow_off;
        (*grid).col_offset = (*wp).w_wincol_off;
    } else {
        (*grid).target = default_grid.ptr();
        (*grid).row_offset = (*wp).w_winrow + (*wp).w_winrow_off;
        (*grid).col_offset = (*wp).w_wincol + (*wp).w_wincol_off;
    }
    if (resizing_screen.get() as ::core::ffi::c_int != 0 || was_resized as ::core::ffi::c_int != 0)
        && want_allocation as ::core::ffi::c_int != 0
    {
        ui_call_grid_resize(
            (*grid_allocated).handle as Integer,
            (*grid_allocated).cols as Integer,
            (*grid_allocated).rows as Integer,
        );
        ui_check_cursor_grid((*grid_allocated).handle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_assign_handle(mut grid: *mut ScreenGrid) {
    static last_grid_handle: GlobalCell<::core::ffi::c_int> = GlobalCell::new(DEFAULT_GRID_HANDLE);
    if (*grid).handle == 0 as ::core::ffi::c_int {
        (*last_grid_handle.ptr()) += 1;
        (*grid).handle = last_grid_handle.get() as handle_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_ins_lines(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
    mut end: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut j: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_uint = 0;
    if line_count <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < line_count {
        if width != (*grid).cols {
            j = end - 1 as ::core::ffi::c_int - i;
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                linecopy(grid, j + line_count, j, col, width);
            }
            j += line_count;
            grid_clear_line(
                grid,
                (*(*grid).line_offset.offset(j as isize)).wrapping_add(col as size_t),
                width,
                false_0 != 0,
            );
        } else {
            j = end - 1 as ::core::ffi::c_int - i;
            temp = *(*grid).line_offset.offset(j as isize) as ::core::ffi::c_uint;
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                *(*grid).line_offset.offset((j + line_count) as isize) =
                    *(*grid).line_offset.offset(j as isize);
            }
            *(*grid).line_offset.offset((j + line_count) as isize) = temp as size_t;
            grid_clear_line(grid, temp as size_t, (*grid).cols, false_0 != 0);
        }
        i += 1;
    }
    if !(*grid).throttled {
        ui_call_grid_scroll(
            (*grid).handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            -line_count as Integer,
            0 as Integer,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_del_lines(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
    mut end: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut j: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_uint = 0;
    if line_count <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < line_count {
        if width != (*grid).cols {
            j = row + i;
            loop {
                j += line_count;
                if j > end - 1 as ::core::ffi::c_int {
                    break;
                }
                linecopy(grid, j - line_count, j, col, width);
            }
            j -= line_count;
            grid_clear_line(
                grid,
                (*(*grid).line_offset.offset(j as isize)).wrapping_add(col as size_t),
                width,
                false_0 != 0,
            );
        } else {
            j = row + i;
            temp = *(*grid).line_offset.offset(j as isize) as ::core::ffi::c_uint;
            loop {
                j += line_count;
                if j > end - 1 as ::core::ffi::c_int {
                    break;
                }
                *(*grid).line_offset.offset((j - line_count) as isize) =
                    *(*grid).line_offset.offset(j as isize);
            }
            *(*grid).line_offset.offset((j - line_count) as isize) = temp as size_t;
            grid_clear_line(grid, temp as size_t, (*grid).cols, false_0 != 0);
        }
        i += 1;
    }
    if !(*grid).throttled {
        ui_call_grid_scroll(
            (*grid).handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            line_count as Integer,
            0 as Integer,
        );
    }
}
unsafe extern "C" fn grid_draw_bordertext(
    mut vt: VirtText,
    mut col: ::core::ffi::c_int,
    mut winbl: ::core::ffi::c_int,
    mut hl_attr: *const ::core::ffi::c_int,
    mut bt: BorderTextType,
    mut overflow: ::core::ffi::c_int,
) {
    let mut default_attr: ::core::ffi::c_int = *hl_attr.offset(
        (if bt as ::core::ffi::c_uint
            == kBorderTextTitle as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            HLF_BTITLE as ::core::ffi::c_int
        } else {
            HLF_BFOOTER as ::core::ffi::c_int
        }) as isize,
    );
    if overflow > 0 as ::core::ffi::c_int {
        grid_line_puts(
            1 as ::core::ffi::c_int,
            b"<\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            hl_apply_winblend(winbl, default_attr),
        );
        col += 1 as ::core::ffi::c_int;
        overflow += 1 as ::core::ffi::c_int;
    }
    let mut i: size_t = 0 as size_t;
    while i < vt.size {
        let mut attr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut text: *mut ::core::ffi::c_char =
            next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
        if text.is_null() {
            break;
        }
        if attr == -1 as ::core::ffi::c_int {
            attr = default_attr;
        }
        if overflow > 0 as ::core::ffi::c_int {
            let mut cells: ::core::ffi::c_int = mb_string2cells(text) as ::core::ffi::c_int;
            if overflow >= cells {
                overflow -= cells;
                continue;
            } else {
                let mut p: *mut ::core::ffi::c_char = text;
                while *p as ::core::ffi::c_int != 0 && overflow > 0 as ::core::ffi::c_int {
                    overflow -= utf_ptr2cells(p);
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                text = p;
            }
        }
        attr = hl_apply_winblend(winbl, attr);
        col += grid_line_puts(col, text, -1 as ::core::ffi::c_int, attr);
    }
}
unsafe extern "C" fn get_bordertext_col(
    mut total_col: ::core::ffi::c_int,
    mut text_width: ::core::ffi::c_int,
    mut align: AlignTextPos,
) -> ::core::ffi::c_int {
    match align as ::core::ffi::c_uint {
        0 => return 1 as ::core::ffi::c_int,
        1 => {
            return if (total_col - text_width) / 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                > 1 as ::core::ffi::c_int
            {
                (total_col - text_width) / 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        2 => {
            return if total_col - text_width + 1 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                total_col - text_width + 1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        _ => {}
    }
    unreachable!();
}
#[no_mangle]
pub unsafe extern "C" fn grid_draw_border(
    mut grid: *mut ScreenGrid,
    mut config: *mut WinConfig,
    mut adj: *mut ::core::ffi::c_int,
    mut winbl: ::core::ffi::c_int,
    mut hl_attr: *mut ::core::ffi::c_int,
) {
    let mut attrs: *mut ::core::ffi::c_int =
        &raw mut (*config).border_attr as *mut ::core::ffi::c_int;
    let mut default_adj: [::core::ffi::c_int; 4] = [
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ];
    if adj.is_null() {
        adj = &raw mut default_adj as *mut ::core::ffi::c_int;
    }
    let mut chars: [schar_T; 8] = [0; 8];
    if hl_attr.is_null() {
        hl_attr = hl_attr_active.get();
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        chars[i as usize] = schar_from_str(
            &raw mut *(&raw mut (*config).border_chars as *mut [::core::ffi::c_char; 32])
                .offset(i as isize) as *mut ::core::ffi::c_char,
        );
        i += 1;
    }
    let mut irow: ::core::ffi::c_int = (*grid).rows
        - *adj.offset(0 as ::core::ffi::c_int as isize)
        - *adj.offset(2 as ::core::ffi::c_int as isize);
    let mut icol: ::core::ffi::c_int = (*grid).cols
        - *adj.offset(1 as ::core::ffi::c_int as isize)
        - *adj.offset(3 as ::core::ffi::c_int as isize);
    if *adj.offset(0 as ::core::ffi::c_int as isize) != 0 {
        screengrid_line_start(grid, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[0 as ::core::ffi::c_int as usize],
                *attrs.offset(0 as ::core::ffi::c_int as isize),
            );
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < icol {
            grid_line_put_schar(
                i_0 + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[1 as ::core::ffi::c_int as usize],
                *attrs.offset(1 as ::core::ffi::c_int as isize),
            );
            i_0 += 1;
        }
        if (*config).title {
            let mut title_col: ::core::ffi::c_int =
                get_bordertext_col(icol, (*config).title_width, (*config).title_pos);
            grid_draw_bordertext(
                (*config).title_chunks,
                title_col,
                winbl,
                hl_attr,
                kBorderTextTitle,
                (*config).title_width - icol,
            );
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[2 as ::core::ffi::c_int as usize],
                *attrs.offset(2 as ::core::ffi::c_int as isize),
            );
        }
        grid_line_flush();
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < irow {
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            screengrid_line_start(
                grid,
                i_1 + *adj.offset(0 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[7 as ::core::ffi::c_int as usize],
                *attrs.offset(7 as ::core::ffi::c_int as isize),
            );
            grid_line_flush();
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            let mut ic: ::core::ffi::c_int = if i_1 == 0 as ::core::ffi::c_int
                && *adj.offset(0 as ::core::ffi::c_int as isize) == 0
                && chars[2 as ::core::ffi::c_int as usize] != 0
            {
                2 as ::core::ffi::c_int
            } else {
                3 as ::core::ffi::c_int
            };
            screengrid_line_start(
                grid,
                i_1 + *adj.offset(0 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[ic as usize],
                *attrs.offset(ic as isize),
            );
            grid_line_flush();
        }
        i_1 += 1;
    }
    if *adj.offset(2 as ::core::ffi::c_int as isize) != 0 {
        screengrid_line_start(
            grid,
            irow + *adj.offset(0 as ::core::ffi::c_int as isize),
            0 as ::core::ffi::c_int,
        );
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[6 as ::core::ffi::c_int as usize],
                *attrs.offset(6 as ::core::ffi::c_int as isize),
            );
        }
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < icol {
            let mut ic_0: ::core::ffi::c_int = if i_2 == 0 as ::core::ffi::c_int
                && *adj.offset(3 as ::core::ffi::c_int as isize) == 0
                && chars[6 as ::core::ffi::c_int as usize] != 0
            {
                6 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            };
            grid_line_put_schar(
                i_2 + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[ic_0 as usize],
                *attrs.offset(ic_0 as isize),
            );
            i_2 += 1;
        }
        if (*config).footer {
            let mut footer_col: ::core::ffi::c_int =
                get_bordertext_col(icol, (*config).footer_width, (*config).footer_pos);
            grid_draw_bordertext(
                (*config).footer_chunks,
                footer_col,
                winbl,
                hl_attr,
                kBorderTextFooter,
                (*config).footer_width - icol,
            );
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[4 as ::core::ffi::c_int as usize],
                *attrs.offset(4 as ::core::ffi::c_int as isize),
            );
        }
        grid_line_flush();
    }
}
unsafe extern "C" fn linecopy(
    mut grid: *mut ScreenGrid,
    mut to: ::core::ffi::c_int,
    mut from: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut off_to: ::core::ffi::c_uint = (*(*grid).line_offset.offset(to as isize))
        .wrapping_add(col as size_t)
        as ::core::ffi::c_uint;
    let mut off_from: ::core::ffi::c_uint = (*(*grid).line_offset.offset(from as isize))
        .wrapping_add(col as size_t)
        as ::core::ffi::c_uint;
    memmove(
        (*grid).chars.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).chars.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    memmove(
        (*grid).attrs.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).attrs.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    memmove(
        (*grid).vcols.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).vcols.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_win_by_grid_handle(mut handle: handle_T) -> *mut win_T {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_grid_alloc.handle == handle {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_char(mut c: ::core::ffi::c_int) -> schar_T {
    let mut sc: schar_T = 0 as schar_T;
    if c >= 0x200000 as ::core::ffi::c_int {
        c = 0xfffd as ::core::ffi::c_int;
    }
    utf_char2bytes(c, &raw mut sc as *mut ::core::ffi::c_char);
    return sc;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
