use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, Boolean, BufUpdateCallbacks, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative,
    GridView, Integer, Intersection, LineFlags, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    OptInt, PackerBuffer, PackerBufferFlush, RemoteUI, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal, Timestamp, UIExtension,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, packer_buffer_t, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T,
    scid_T, sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3,
    syn_time_T, synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn abort() -> !;
    fn llabs(__x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static default_grid: GlobalCell<ScreenGrid>;
    fn schar_from_buf(buf: *const ::core::ffi::c_char, len: size_t) -> schar_T;
    fn schar_from_char(c: ::core::ffi::c_int) -> schar_T;
    static rdb_flags: GlobalCell<::core::ffi::c_uint>;
    static p_wd: GlobalCell<OptInt>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn hl_blend_attrs(
        back_attr: ::core::ffi::c_int,
        front_attr: ::core::ffi::c_int,
        through: *mut bool,
    ) -> ::core::ffi::c_int;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static msg_grid: GlobalCell<ScreenGrid>;
    fn os_sleep(ms: uint64_t);
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_flush();
    fn ui_composed_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_composed_call_grid_cursor_goto(grid: Integer, row: Integer, col: Integer);
    fn ui_composed_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn ui_composed_call_raw_line(
        grid: Integer,
        row: Integer,
        startcol: Integer,
        endcol: Integer,
        clearcol: Integer,
        clearattr: Integer,
        flags: LineFlags,
        chunk: *const schar_T,
        attrs: *const sattr_T,
    );
}
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kOptRdbFlagFlush: C2Rust_Unnamed_13 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_13 = 16;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_13 = 8;
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_13 = 4;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_13 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_13 = 1;
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
pub const kLineFlagInvalid: C2Rust_Unnamed_14 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_14 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ScreenGrid,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_15 = C2Rust_Unnamed_15 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<*mut ScreenGrid>(),
};
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
static composed_uis: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub static layers: GlobalCell<C2Rust_Unnamed_15> = GlobalCell::new(KV_INITIAL_VALUE);
static bufsize: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static linebuf: GlobalCell<*mut schar_T> = GlobalCell::new(::core::ptr::null_mut::<schar_T>());
static attrbuf: GlobalCell<*mut sattr_T> = GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
static chk_height: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static chk_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static curgrid: GlobalCell<*mut ScreenGrid> =
    GlobalCell::new(::core::ptr::null_mut::<ScreenGrid>());
static valid_screen: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static msg_current_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(INT_MAX);
static msg_was_scrolled: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_sep_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static msg_sep_char: GlobalCell<schar_T> = GlobalCell::new(' ' as ::core::ffi::c_int as schar_T);
static dbghl_normal: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_clear: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_composed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_recompose: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub unsafe extern "C" fn ui_comp_init() {
    if (*layers.ptr()).size == (*layers.ptr()).capacity {
        (*layers.ptr()).capacity = if (*layers.ptr()).capacity != 0 {
            (*layers.ptr()).capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*layers.ptr()).items = xrealloc(
            (*layers.ptr()).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut ScreenGrid>().wrapping_mul((*layers.ptr()).capacity),
        ) as *mut *mut ScreenGrid;
    } else {
    };
    let c2rust_fresh0 = (*layers.ptr()).size;
    (*layers.ptr()).size = (*layers.ptr()).size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *(*layers.ptr()).items.offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = default_grid.ptr();
    curgrid.set(default_grid.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_syn_init() {
    dbghl_normal.set(syn_check_group(
        b"RedrawDebugNormal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 18]>().wrapping_sub(1 as size_t),
    ));
    dbghl_clear.set(syn_check_group(
        b"RedrawDebugClear\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
    ));
    dbghl_composed.set(syn_check_group(
        b"RedrawDebugComposed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>().wrapping_sub(1 as size_t),
    ));
    dbghl_recompose.set(syn_check_group(
        b"RedrawDebugRecompose\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 21]>().wrapping_sub(1 as size_t),
    ));
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_attach(mut ui: *mut RemoteUI) {
    (*composed_uis.ptr()) += 1;
    (*ui).composed = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_detach(mut ui: *mut RemoteUI) {
    (*composed_uis.ptr()) -= 1;
    if composed_uis.get() == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            linebuf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            attrbuf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        bufsize.set(0 as size_t);
    }
    (*ui).composed = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_should_draw() -> bool {
    return composed_uis.get() != 0 as ::core::ffi::c_int
        && valid_screen.get() as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_layers_adjust(mut layer_idx: size_t, mut raise: bool) {
    let mut size: size_t = (*layers.ptr()).size;
    let mut layer: *mut ScreenGrid = *(*layers.ptr()).items.offset(layer_idx as isize);
    if raise {
        while layer_idx < size.wrapping_sub(1 as size_t)
            && (*layer).zindex
                > (**(*layers.ptr())
                    .items
                    .offset(layer_idx.wrapping_add(1 as size_t) as isize))
                .zindex
        {
            *(*layers.ptr()).items.offset(layer_idx as isize) = *(*layers.ptr())
                .items
                .offset(layer_idx.wrapping_add(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(layer_idx as isize)).comp_index = layer_idx;
            (**(*layers.ptr()).items.offset(layer_idx as isize)).pending_comp_index_update =
                true_0 != 0;
            layer_idx = layer_idx.wrapping_add(1);
        }
    } else {
        while layer_idx > 0 as size_t
            && (*layer).zindex
                < (**(*layers.ptr())
                    .items
                    .offset(layer_idx.wrapping_sub(1 as size_t) as isize))
                .zindex
        {
            *(*layers.ptr()).items.offset(layer_idx as isize) = *(*layers.ptr())
                .items
                .offset(layer_idx.wrapping_sub(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(layer_idx as isize)).comp_index = layer_idx;
            (**(*layers.ptr()).items.offset(layer_idx as isize)).pending_comp_index_update =
                true_0 != 0;
            layer_idx = layer_idx.wrapping_sub(1);
        }
    }
    *(*layers.ptr()).items.offset(layer_idx as isize) = layer;
    (*layer).comp_index = layer_idx;
    (*layer).pending_comp_index_update = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_put_grid(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
    mut valid: bool,
    mut on_top: bool,
) -> bool {
    let mut moved: bool = false;
    (*grid).pending_comp_index_update = true_0 != 0;
    if (*grid).comp_index != 0 as size_t {
        moved = row != (*grid).comp_row || col != (*grid).comp_col;
        if ui_comp_should_draw() {
            (*grid).comp_disabled = true_0 != 0;
            compose_area(
                (*grid).comp_row as Integer,
                row as Integer,
                (*grid).comp_col as Integer,
                ((*grid).comp_col + (*grid).comp_width) as Integer,
            );
            if (*grid).comp_col < col {
                compose_area(
                    (if row > (*grid).comp_row {
                        row
                    } else {
                        (*grid).comp_row
                    }) as Integer,
                    (if row + height < (*grid).comp_row + (*grid).comp_height {
                        row + height
                    } else {
                        (*grid).comp_row + (*grid).comp_height
                    }) as Integer,
                    (*grid).comp_col as Integer,
                    col as Integer,
                );
            }
            if col + width < (*grid).comp_col + (*grid).comp_width {
                compose_area(
                    (if row > (*grid).comp_row {
                        row
                    } else {
                        (*grid).comp_row
                    }) as Integer,
                    (if row + height < (*grid).comp_row + (*grid).comp_height {
                        row + height
                    } else {
                        (*grid).comp_row + (*grid).comp_height
                    }) as Integer,
                    (col + width) as Integer,
                    ((*grid).comp_col + (*grid).comp_width) as Integer,
                );
            }
            compose_area(
                (row + height) as Integer,
                ((*grid).comp_row + (*grid).comp_height) as Integer,
                (*grid).comp_col as Integer,
                ((*grid).comp_col + (*grid).comp_width) as Integer,
            );
            (*grid).comp_disabled = false_0 != 0;
        }
        (*grid).comp_row = row;
        (*grid).comp_col = col;
    } else {
        moved = true_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < (*layers.ptr()).size {
            if *(*layers.ptr()).items.offset(i as isize) == grid {
                abort();
            }
            i = i.wrapping_add(1);
        }
        let mut insert_at: size_t = (*layers.ptr()).size;
        while insert_at > 0 as size_t
            && (**(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize))
            .zindex
                > (*grid).zindex
        {
            insert_at = insert_at.wrapping_sub(1);
        }
        if !(*curwin.ptr()).is_null()
            && *(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize)
                == &raw mut (*curwin.get()).w_grid_alloc
            && (**(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize))
            .zindex
                == (*grid).zindex
            && !on_top
        {
            insert_at = insert_at.wrapping_sub(1);
        }
        if (*layers.ptr()).size == (*layers.ptr()).capacity {
            (*layers.ptr()).capacity = if (*layers.ptr()).capacity != 0 {
                (*layers.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*layers.ptr()).items = xrealloc(
                (*layers.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<*mut ScreenGrid>().wrapping_mul((*layers.ptr()).capacity),
            ) as *mut *mut ScreenGrid;
        } else {
        };
        (*layers.ptr()).size = (*layers.ptr()).size.wrapping_add(1);
        let mut i_0: size_t = (*layers.ptr()).size.wrapping_sub(1 as size_t);
        while i_0 > insert_at {
            *(*layers.ptr()).items.offset(i_0 as isize) = *(*layers.ptr())
                .items
                .offset(i_0.wrapping_sub(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(i_0 as isize)).comp_index = i_0;
            (**(*layers.ptr()).items.offset(i_0 as isize)).pending_comp_index_update = true_0 != 0;
            i_0 = i_0.wrapping_sub(1);
        }
        *(*layers.ptr()).items.offset(insert_at as isize) = grid;
        (*grid).comp_row = row;
        (*grid).comp_col = col;
        (*grid).comp_index = insert_at;
        (*grid).pending_comp_index_update = true_0 != 0;
    }
    (*grid).comp_height = height;
    (*grid).comp_width = width;
    if moved as ::core::ffi::c_int != 0
        && valid as ::core::ffi::c_int != 0
        && ui_comp_should_draw() as ::core::ffi::c_int != 0
    {
        compose_area(
            (*grid).comp_row as Integer,
            ((*grid).comp_row + (*grid).rows) as Integer,
            (*grid).comp_col as Integer,
            ((*grid).comp_col + (*grid).cols) as Integer,
        );
    }
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_remove_grid(mut grid: *mut ScreenGrid) {
    '_c2rust_label: {
        if grid != default_grid.ptr() {
        } else {
            __assert_fail(
                b"grid != &default_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                217 as ::core::ffi::c_uint,
                b"void ui_comp_remove_grid(ScreenGrid *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*grid).comp_index == 0 as size_t {
        return;
    }
    if curgrid.get() == grid {
        curgrid.set(default_grid.ptr());
    }
    let mut i: size_t = (*grid).comp_index;
    while i < (*layers.ptr()).size.wrapping_sub(1 as size_t) {
        *(*layers.ptr()).items.offset(i as isize) = *(*layers.ptr())
            .items
            .offset(i.wrapping_add(1 as size_t) as isize);
        (**(*layers.ptr()).items.offset(i as isize)).comp_index = i;
        (**(*layers.ptr()).items.offset(i as isize)).pending_comp_index_update = true_0 != 0;
        i = i.wrapping_add(1);
    }
    (*layers.ptr()).size = (*layers.ptr()).size.wrapping_sub(1);
    (*grid).comp_index = 0 as size_t;
    (*grid).pending_comp_index_update = true_0 != 0;
    ui_comp_compose_grid(grid);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_set_grid(mut handle: handle_T) -> bool {
    if (*curgrid.get()).handle == handle {
        return true_0 != 0;
    }
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut i: size_t = 0 as size_t;
    while i < (*layers.ptr()).size {
        if (**(*layers.ptr()).items.offset(i as isize)).handle == handle {
            grid = *(*layers.ptr()).items.offset(i as isize);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !grid.is_null() {
        curgrid.set(grid);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_raise_grid(mut grid: *mut ScreenGrid, mut new_index: size_t) {
    let mut old_index: size_t = (*grid).comp_index;
    let mut i: size_t = old_index;
    while i < new_index {
        *(*layers.ptr()).items.offset(i as isize) = *(*layers.ptr())
            .items
            .offset(i.wrapping_add(1 as size_t) as isize);
        (**(*layers.ptr()).items.offset(i as isize)).comp_index = i;
        (**(*layers.ptr()).items.offset(i as isize)).pending_comp_index_update = true_0 != 0;
        i = i.wrapping_add(1);
    }
    *(*layers.ptr()).items.offset(new_index as isize) = grid;
    (*grid).comp_index = new_index;
    (*grid).pending_comp_index_update = true_0 != 0;
    let mut i_0: size_t = old_index;
    while i_0 < new_index {
        let mut grid2: *mut ScreenGrid = *(*layers.ptr()).items.offset(i_0 as isize);
        let mut startcol: ::core::ffi::c_int = if (*grid).comp_col > (*grid2).comp_col {
            (*grid).comp_col
        } else {
            (*grid2).comp_col
        };
        let mut endcol: ::core::ffi::c_int =
            if (*grid).comp_col + (*grid).cols < (*grid2).comp_col + (*grid2).cols {
                (*grid).comp_col + (*grid).cols
            } else {
                (*grid2).comp_col + (*grid2).cols
            };
        compose_area(
            (if (*grid).comp_row > (*grid2).comp_row {
                (*grid).comp_row
            } else {
                (*grid2).comp_row
            }) as Integer,
            (if (*grid).comp_row + (*grid).rows < (*grid2).comp_row + (*grid2).rows {
                (*grid).comp_row + (*grid).rows
            } else {
                (*grid2).comp_row + (*grid2).rows
            }) as Integer,
            startcol as Integer,
            endcol as Integer,
        );
        i_0 = i_0.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_cursor_goto(
    mut grid_handle: Integer,
    mut r: Integer,
    mut c: Integer,
) {
    if !ui_comp_set_grid(grid_handle as handle_T) {
        return;
    }
    let mut cursor_row: ::core::ffi::c_int = (*curgrid.get()).comp_row + r as ::core::ffi::c_int;
    let mut cursor_col: ::core::ffi::c_int = (*curgrid.get()).comp_col + c as ::core::ffi::c_int;
    if curgrid.get() != default_grid.ptr() {
        let mut new_index: size_t = (*layers.ptr()).size.wrapping_sub(1 as size_t);
        while new_index > 1 as size_t
            && (**(*layers.ptr()).items.offset(new_index as isize)).zindex > (*curgrid.get()).zindex
        {
            new_index = new_index.wrapping_sub(1);
        }
        if (*curgrid.get()).comp_index < new_index {
            ui_comp_raise_grid(curgrid.get(), new_index);
        }
    }
    if cursor_col >= (*default_grid.ptr()).cols || cursor_row >= (*default_grid.ptr()).rows {
        return;
    }
    ui_composed_call_grid_cursor_goto(1 as Integer, cursor_row as Integer, cursor_col as Integer);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_mouse_focus(
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) -> *mut ScreenGrid {
    let mut i: ssize_t = (*layers.ptr()).size as ssize_t - 1 as ssize_t;
    while i > 0 as ssize_t {
        let mut grid: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
        if (*grid).mouse_enabled as ::core::ffi::c_int != 0
            && row >= (*grid).comp_row
            && row < (*grid).comp_row + (*grid).rows
            && col >= (*grid).comp_col
            && col < (*grid).comp_col + (*grid).cols
        {
            return grid;
        }
        i -= 1;
    }
    if ui_has(kUIMultigrid) {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            let mut grid_0: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
            if (*grid_0).mouse_enabled as ::core::ffi::c_int != 0
                && row >= (*wp).w_winrow
                && row < (*wp).w_winrow + (*grid_0).rows
                && col >= (*wp).w_wincol
                && col < (*wp).w_wincol + (*grid_0).cols
            {
                return grid_0;
            }
            wp = (*wp).w_next;
        }
    }
    return ::core::ptr::null_mut::<ScreenGrid>();
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_get_grid_at_coord(
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) -> *mut ScreenGrid {
    let mut i: ssize_t = (*layers.ptr()).size as ssize_t - 1 as ssize_t;
    while i > 0 as ssize_t {
        let mut grid: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
        if row >= (*grid).comp_row
            && row < (*grid).comp_row + (*grid).rows
            && col >= (*grid).comp_col
            && col < (*grid).comp_col + (*grid).cols
        {
            return grid;
        }
        i -= 1;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut grid_0: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
        if row >= (*grid_0).comp_row
            && row < (*grid_0).comp_row + (*grid_0).rows
            && col >= (*grid_0).comp_col
            && col < (*grid_0).comp_col + (*grid_0).cols
            && !(*wp).w_config.hide
        {
            return grid_0;
        }
        wp = (*wp).w_next;
    }
    return default_grid.ptr();
}
unsafe extern "C" fn compose_line(
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut flags: LineFlags,
) {
    startcol = if startcol > 0 as Integer {
        startcol
    } else {
        0 as Integer
    };
    let mut skipstart: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skipend: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if startcol > 0 as Integer
        && flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
    {
        startcol -= 1;
        skipstart = 1 as ::core::ffi::c_int;
    }
    if endcol < (*default_grid.ptr()).cols as Integer
        && flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
    {
        endcol += 1;
        skipend = 1 as ::core::ffi::c_int;
    }
    let mut col: ::core::ffi::c_int = startcol as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut bg_line: *mut schar_T = (*default_grid.ptr()).chars.offset(
        (*(*default_grid.ptr()).line_offset.offset(row as isize)).wrapping_add(startcol as size_t)
            as isize,
    );
    let mut bg_attrs: *mut sattr_T = (*default_grid.ptr()).attrs.offset(
        (*(*default_grid.ptr()).line_offset.offset(row as isize)).wrapping_add(startcol as size_t)
            as isize,
    );
    while (col as Integer) < endcol {
        let mut until: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < (*layers.ptr()).size {
            let mut g: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
            let mut grid_width: ::core::ffi::c_int = if (*g).cols < (*g).comp_width {
                (*g).cols
            } else {
                (*g).comp_width
            };
            let mut grid_height: ::core::ffi::c_int = if (*g).rows < (*g).comp_height {
                (*g).rows
            } else {
                (*g).comp_height
            };
            if !((*g).comp_row as Integer > row
                || row >= ((*g).comp_row + grid_height) as Integer
                || (*g).comp_disabled as ::core::ffi::c_int != 0)
            {
                if (*g).comp_col <= col && col < (*g).comp_col + grid_width {
                    grid = g;
                    until = (*g).comp_col + grid_width;
                } else if (*g).comp_col > col {
                    until = if until < (*g).comp_col {
                        until
                    } else {
                        (*g).comp_col
                    };
                }
            }
            i = i.wrapping_add(1);
        }
        until = if until < endcol as ::core::ffi::c_int {
            until
        } else {
            endcol as ::core::ffi::c_int
        };
        '_c2rust_label: {
            if !grid.is_null() {
            } else {
                __assert_fail(
                    b"grid != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    408 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_0: {
            if until > col {
            } else {
                __assert_fail(
                    b"until > col\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    409 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_1: {
            if until <= (*default_grid.ptr()).cols {
            } else {
                __assert_fail(
                    b"until <= default_grid.cols\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    410 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut n: size_t = (until - col) as size_t;
        if row == msg_sep_row.get() as Integer && (*grid).comp_index <= (*msg_grid.ptr()).comp_index
        {
            grid = msg_grid.ptr();
            let mut msg_sep_attr: sattr_T = *(*hl_attr_active.ptr())
                .offset(HLF_MSGSEP as ::core::ffi::c_int as isize)
                as sattr_T;
            let mut i_0: ::core::ffi::c_int = col;
            while i_0 < until {
                *(*linebuf.ptr()).offset((i_0 as Integer - startcol) as isize) = msg_sep_char.get();
                *(*attrbuf.ptr()).offset((i_0 as Integer - startcol) as isize) = msg_sep_attr;
                i_0 += 1;
            }
        } else {
            let mut off: size_t = (*(*grid)
                .line_offset
                .offset((row - (*grid).comp_row as Integer) as isize))
            .wrapping_add((col - (*grid).comp_col) as size_t);
            memcpy(
                (*linebuf.ptr()).offset((col as Integer - startcol) as isize)
                    as *mut ::core::ffi::c_void,
                (*grid).chars.offset(off as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<schar_T>()),
            );
            memcpy(
                (*attrbuf.ptr()).offset((col as Integer - startcol) as isize)
                    as *mut ::core::ffi::c_void,
                (*grid).attrs.offset(off as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<sattr_T>()),
            );
            if (*grid).comp_col + (*grid).cols > until
                && *(*grid).chars.offset(off.wrapping_add(n) as isize) == NUL as schar_T
            {
                *(*linebuf.ptr())
                    .offset(((until - 1 as ::core::ffi::c_int) as Integer - startcol) as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
                if col as Integer == startcol && n == 1 as size_t {
                    skipstart = 0 as ::core::ffi::c_int;
                }
            }
        }
        if (*grid).blending {
            let mut width: ::core::ffi::c_int = 0;
            let mut i_1: ::core::ffi::c_int = col - startcol as ::core::ffi::c_int;
            while (i_1 as Integer) < until as Integer - startcol {
                width = 1 as ::core::ffi::c_int;
                let mut thru: bool = (*(*linebuf.ptr()).offset(i_1 as isize)
                    == ' ' as ::core::ffi::c_int as schar_T
                    || *(*linebuf.ptr()).offset(i_1 as isize)
                        == schar_from_char('⠀' as ::core::ffi::c_int))
                    && *bg_line.offset(i_1 as isize) != NUL as schar_T;
                if ((i_1 + 1 as ::core::ffi::c_int) as Integer) < endcol - startcol
                    && *bg_line.offset((i_1 + 1 as ::core::ffi::c_int) as isize) == NUL as schar_T
                {
                    width = 2 as ::core::ffi::c_int;
                    thru = thru as ::core::ffi::c_int
                        & (*(*linebuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                            == ' ' as ::core::ffi::c_int as schar_T
                            || *(*linebuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                == schar_from_char('⠀' as ::core::ffi::c_int))
                            as ::core::ffi::c_int
                        != 0;
                }
                *(*attrbuf.ptr()).offset(i_1 as isize) = hl_blend_attrs(
                    *bg_attrs.offset(i_1 as isize) as ::core::ffi::c_int,
                    *(*attrbuf.ptr()).offset(i_1 as isize) as ::core::ffi::c_int,
                    &raw mut thru,
                ) as sattr_T;
                if width == 2 as ::core::ffi::c_int {
                    *(*attrbuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize) =
                        hl_blend_attrs(
                            *bg_attrs.offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                            *(*attrbuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                            &raw mut thru,
                        ) as sattr_T;
                }
                if thru {
                    memcpy(
                        (*linebuf.ptr()).offset(i_1 as isize) as *mut ::core::ffi::c_void,
                        bg_line.offset(i_1 as isize) as *const ::core::ffi::c_void,
                        (width as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
                    );
                }
                i_1 += width;
            }
        }
        if *(*linebuf.ptr()).offset((col as Integer - startcol) as isize) == NUL as schar_T {
            *(*linebuf.ptr()).offset((col as Integer - startcol) as isize) =
                ' ' as ::core::ffi::c_int as schar_T;
            if col as Integer == endcol - 1 as Integer {
                skipend = 0 as ::core::ffi::c_int;
            }
        } else if col as Integer == startcol
            && n > 1 as size_t
            && *(*linebuf.ptr()).offset(1 as ::core::ffi::c_int as isize) == NUL as schar_T
        {
            skipstart = 0 as ::core::ffi::c_int;
        }
        col = until;
    }
    if *(*linebuf.ptr()).offset((endcol - startcol - 1 as Integer) as isize) == NUL as schar_T {
        skipend = 0 as ::core::ffi::c_int;
    }
    '_c2rust_label_2: {
        if endcol <= chk_width.get() as Integer {
        } else {
            __assert_fail(
                b"endcol <= chk_width\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                477 as ::core::ffi::c_uint,
                b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_3: {
        if row < chk_height.get() as Integer {
        } else {
            __assert_fail(
                b"row < chk_height\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                478 as ::core::ffi::c_uint,
                b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !(!grid.is_null()
        && (grid == default_grid.ptr()
            || (*grid).comp_col == 0 as ::core::ffi::c_int && (*grid).cols == Columns.get()))
    {
        flags = (flags as ::core::ffi::c_int & !(kLineFlagWrap as ::core::ffi::c_int)) as LineFlags;
    }
    let mut i_2: ::core::ffi::c_int = skipstart;
    while (i_2 as Integer) < endcol - skipend as Integer - startcol {
        if *(*attrbuf.ptr()).offset(i_2 as isize) < 0 as sattr_T {
            if rdb_flags.get() & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                abort();
            } else {
                *(*attrbuf.ptr()).offset(i_2 as isize) = 0 as ::core::ffi::c_int as sattr_T;
            }
        }
        i_2 += 1;
    }
    ui_composed_call_raw_line(
        1 as Integer,
        row,
        startcol + skipstart as Integer,
        endcol - skipend as Integer,
        endcol - skipend as Integer,
        0 as Integer,
        flags,
        (linebuf.get() as *const schar_T).offset(skipstart as isize),
        (attrbuf.get() as *const sattr_T).offset(skipstart as isize),
    );
}
unsafe extern "C" fn compose_debug(
    mut startrow: Integer,
    mut endrow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut syn_id: ::core::ffi::c_int,
    mut delay: bool,
) {
    if rdb_flags.get() & kOptRdbFlagCompositor as ::core::ffi::c_int as ::core::ffi::c_uint == 0
        || startcol >= endcol
    {
        return;
    }
    endrow = if endrow < (*default_grid.ptr()).rows as Integer {
        endrow
    } else {
        (*default_grid.ptr()).rows as Integer
    };
    endcol = if endcol < (*default_grid.ptr()).cols as Integer {
        endcol
    } else {
        (*default_grid.ptr()).cols as Integer
    };
    let mut attr: ::core::ffi::c_int = syn_id2attr(syn_id);
    if delay {
        debug_delay(endrow - startrow);
    }
    let mut row: ::core::ffi::c_int = startrow as ::core::ffi::c_int;
    while (row as Integer) < endrow {
        ui_composed_call_raw_line(
            1 as Integer,
            row as Integer,
            startcol,
            startcol,
            endcol,
            attr as Integer,
            false_0,
            linebuf.get() as *const schar_T,
            attrbuf.get() as *const sattr_T,
        );
        row += 1;
    }
    if delay {
        debug_delay(endrow - startrow);
    }
}
unsafe extern "C" fn debug_delay(mut lines: Integer) {
    ui_call_flush();
    let mut wd: uint64_t = llabs(p_wd.get() as ::core::ffi::c_longlong) as uint64_t;
    let mut factor: uint64_t = (if (if lines < 5 as Integer {
        lines
    } else {
        5 as Integer
    }) > 1 as Integer
    {
        if lines < 5 as Integer {
            lines
        } else {
            5 as Integer
        }
    } else {
        1 as Integer
    }) as uint64_t;
    os_sleep(factor.wrapping_mul(wd));
}
unsafe extern "C" fn compose_area(
    mut startrow: Integer,
    mut endrow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
) {
    compose_debug(
        startrow,
        endrow,
        startcol,
        endcol,
        dbghl_recompose.get(),
        true_0 != 0,
    );
    endrow = if endrow < (*default_grid.ptr()).rows as Integer {
        endrow
    } else {
        (*default_grid.ptr()).rows as Integer
    };
    endcol = if endcol < (*default_grid.ptr()).cols as Integer {
        endcol
    } else {
        (*default_grid.ptr()).cols as Integer
    };
    if endcol <= startcol {
        return;
    }
    let mut r: ::core::ffi::c_int = startrow as ::core::ffi::c_int;
    while (r as Integer) < endrow {
        compose_line(
            r as Integer,
            startcol,
            endcol,
            kLineFlagInvalid as ::core::ffi::c_int,
        );
        r += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_compose_grid(mut grid: *mut ScreenGrid) {
    if ui_comp_should_draw() {
        compose_area(
            (*grid).comp_row as Integer,
            ((*grid).comp_row + (*grid).rows) as Integer,
            (*grid).comp_col as Integer,
            ((*grid).comp_col + (*grid).cols) as Integer,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_raw_line(
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    row += (*curgrid.get()).comp_row as Integer;
    startcol += (*curgrid.get()).comp_col as Integer;
    endcol += (*curgrid.get()).comp_col as Integer;
    clearcol += (*curgrid.get()).comp_col as Integer;
    if curgrid.get() != default_grid.ptr() {
        flags = (flags as ::core::ffi::c_int & !(kLineFlagWrap as ::core::ffi::c_int)) as LineFlags;
    }
    '_c2rust_label: {
        if endcol <= clearcol {
        } else {
            __assert_fail(
                b"endcol <= clearcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                574 as ::core::ffi::c_uint,
                b"void ui_comp_raw_line(Integer, Integer, Integer, Integer, Integer, Integer, LineFlags, const schar_T *, const sattr_T *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if row >= (*default_grid.ptr()).rows as Integer {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_comp_raw_line\0".as_ptr() as *const ::core::ffi::c_char,
            580 as ::core::ffi::c_int,
            true_0 != 0,
            b"compositor: invalid row %ld on grid %ld\0".as_ptr() as *const ::core::ffi::c_char,
            row,
            grid,
        );
        return;
    }
    if clearcol > (*default_grid.ptr()).cols as Integer {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_comp_raw_line\0".as_ptr() as *const ::core::ffi::c_char,
            585 as ::core::ffi::c_int,
            true_0 != 0,
            b"compositor: invalid last column %ld on grid %ld\0".as_ptr()
                as *const ::core::ffi::c_char,
            clearcol,
            grid,
        );
        if startcol >= (*default_grid.ptr()).cols as Integer {
            return;
        }
        clearcol = (*default_grid.ptr()).cols as Integer;
        endcol = if endcol < clearcol { endcol } else { clearcol };
    }
    let mut covered: bool = curgrid_covered_above(row as ::core::ffi::c_int);
    if flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
        || covered as ::core::ffi::c_int != 0
        || (*curgrid.get()).blending as ::core::ffi::c_int != 0
    {
        compose_debug(
            row,
            row + 1 as Integer,
            startcol,
            clearcol,
            dbghl_composed.get(),
            true_0 != 0,
        );
        compose_line(row, startcol, clearcol, flags);
    } else {
        compose_debug(
            row,
            row + 1 as Integer,
            startcol,
            endcol,
            dbghl_normal.get(),
            endcol >= clearcol,
        );
        compose_debug(
            row,
            row + 1 as Integer,
            endcol,
            clearcol,
            dbghl_clear.get(),
            true_0 != 0,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as Integer) < endcol - startcol {
            '_c2rust_label_0: {
                if *attrs.offset(i as isize) >= 0 as sattr_T {
                } else {
                    __assert_fail(
                        b"attrs[i] >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ui_compositor.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        604 as ::core::ffi::c_uint,
                        b"void ui_comp_raw_line(Integer, Integer, Integer, Integer, Integer, Integer, LineFlags, const schar_T *, const sattr_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            i += 1;
        }
        ui_composed_call_raw_line(
            1 as Integer,
            row,
            startcol,
            endcol,
            clearcol,
            clearattr,
            flags,
            chunk,
            attrs,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_set_screen_valid(mut valid: bool) -> bool {
    let mut old_val: bool = valid_screen.get();
    valid_screen.set(valid);
    if !valid {
        msg_sep_row.set(-1 as ::core::ffi::c_int);
    }
    return old_val;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_msg_set_pos(
    mut _grid: Integer,
    mut row: Integer,
    mut scrolled: Boolean,
    mut sep_char: String_0,
    mut _zindex: Integer,
    mut _compindex: Integer,
) {
    (*msg_grid.ptr()).pending_comp_index_update = true_0 != 0;
    (*msg_grid.ptr()).comp_row = row as ::core::ffi::c_int;
    if scrolled as ::core::ffi::c_int != 0 && row > 0 as Integer {
        msg_sep_row.set(row as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if !sep_char.data.is_null() {
            msg_sep_char.set(schar_from_buf(sep_char.data, sep_char.size));
        }
    } else {
        msg_sep_row.set(-1 as ::core::ffi::c_int);
    }
    if row > msg_current_row.get() as Integer && ui_comp_should_draw() as ::core::ffi::c_int != 0 {
        compose_area(
            (if msg_current_row.get() - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                msg_current_row.get() - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as Integer,
            row,
            0 as Integer,
            (*default_grid.ptr()).cols as Integer,
        );
    } else if row < msg_current_row.get() as Integer
        && ui_comp_should_draw() as ::core::ffi::c_int != 0
        && (msg_current_row.get() < Rows.get()
            || scrolled as ::core::ffi::c_int != 0 && !msg_was_scrolled.get())
    {
        let mut delta: ::core::ffi::c_int = msg_current_row.get() - row as ::core::ffi::c_int;
        if (*msg_grid.ptr()).blending {
            let mut first_row: ::core::ffi::c_int = if row as ::core::ffi::c_int
                - (if scrolled as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                > 0 as ::core::ffi::c_int
            {
                row as ::core::ffi::c_int
                    - (if scrolled as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
            } else {
                0 as ::core::ffi::c_int
            };
            compose_area(
                first_row as Integer,
                (Rows.get() - delta) as Integer,
                0 as Integer,
                Columns.get() as Integer,
            );
        } else {
            let mut first_row_0: ::core::ffi::c_int = if row as ::core::ffi::c_int
                - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                > 0 as ::core::ffi::c_int
            {
                row as ::core::ffi::c_int
                    - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
            } else {
                0 as ::core::ffi::c_int
            };
            ui_composed_call_grid_scroll(
                1 as Integer,
                first_row_0 as Integer,
                Rows.get() as Integer,
                0 as Integer,
                Columns.get() as Integer,
                delta as Integer,
                0 as Integer,
            );
            if scrolled as ::core::ffi::c_int != 0 && !msg_was_scrolled.get() && row > 0 as Integer
            {
                compose_area(
                    row - 1 as Integer,
                    row,
                    0 as Integer,
                    Columns.get() as Integer,
                );
            }
        }
    }
    msg_current_row.set(row as ::core::ffi::c_int);
    msg_was_scrolled.set(scrolled as bool);
}
unsafe extern "C" fn curgrid_covered_above(mut row: ::core::ffi::c_int) -> bool {
    let mut above_msg: bool = *(*layers.ptr())
        .items
        .offset((*layers.ptr()).size.wrapping_sub(1 as size_t) as isize)
        == msg_grid.ptr()
        && row
            < msg_current_row.get()
                - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
    return (*layers.ptr()).size.wrapping_sub(
        (if above_msg as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as size_t,
    ) > (*curgrid.get()).comp_index.wrapping_add(1 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_scroll(
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    top += (*curgrid.get()).comp_row as Integer;
    bot += (*curgrid.get()).comp_row as Integer;
    left += (*curgrid.get()).comp_col as Integer;
    right += (*curgrid.get()).comp_col as Integer;
    let mut covered: bool = curgrid_covered_above(
        (bot - (if rows > 0 as Integer {
            rows
        } else {
            0 as Integer
        })) as ::core::ffi::c_int,
    );
    if covered as ::core::ffi::c_int != 0 || (*curgrid.get()).blending as ::core::ffi::c_int != 0 {
        compose_debug(top, bot, left, right, dbghl_recompose.get(), true_0 != 0);
        let mut r: ::core::ffi::c_int = (top
            + (if -rows > 0 as Integer {
                -rows
            } else {
                0 as Integer
            })) as ::core::ffi::c_int;
        while (r as Integer)
            < bot
                - (if rows > 0 as Integer {
                    rows
                } else {
                    0 as Integer
                })
        {
            if *(*curgrid.get()).attrs.offset(
                (*(*curgrid.get())
                    .line_offset
                    .offset((r - (*curgrid.get()).comp_row) as isize))
                .wrapping_add(left as size_t)
                .wrapping_sub((*curgrid.get()).comp_col as size_t) as isize,
            ) >= 0 as sattr_T
            {
                compose_line(r as Integer, left, right, 0 as LineFlags);
            }
            r += 1;
        }
    } else {
        ui_composed_call_grid_scroll(1 as Integer, top, bot, left, right, rows, cols);
        if rdb_flags.get() & kOptRdbFlagCompositor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            debug_delay(2 as Integer);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_resize(
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    if grid == 1 as Integer {
        ui_composed_call_grid_resize(1 as Integer, width, height);
        chk_width.set(width as ::core::ffi::c_int);
        chk_height.set(height as ::core::ffi::c_int);
        let mut new_bufsize: size_t = width as size_t;
        if bufsize.get() != new_bufsize {
            xfree(linebuf.get() as *mut ::core::ffi::c_void);
            xfree(attrbuf.get() as *mut ::core::ffi::c_void);
            linebuf.set(
                xmalloc(new_bufsize.wrapping_mul(::core::mem::size_of::<schar_T>()))
                    as *mut schar_T,
            );
            attrbuf.set(
                xmalloc(new_bufsize.wrapping_mul(::core::mem::size_of::<sattr_T>()))
                    as *mut sattr_T,
            );
            bufsize.set(new_bufsize);
        }
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
