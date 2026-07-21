use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExtmarkMove, ExtmarkOp,
    ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative,
    GridView, Integer, Intersection, KeyDict_buf_attach, KeyDict_buf_delete, KeyDict_empty,
    KeyDict_keymap, KeyValuePair, LuaRef, LuaRetMode, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t,
    MarkAdjustMode, MarkGet, MarkTree, MetaIndex, Object, ObjectType, OptInt, OptionalKeys,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0,
    Terminal, Timestamp, TryState, UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t,
    aco_save_T, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    dobuf_action_values, dobuf_start_values, except_T, except_type_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    lua_State, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed,
    partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T,
    size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, vim_exception, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn strchrsub(str: *mut ::core::ffi::c_char, c: ::core::ffi::c_char, x: ::core::ffi::c_char);
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_allocz(arena: *mut Arena, size: size_t) -> *mut ::core::ffi::c_char;
    fn arena_memdupz(
        arena: *mut Arena,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn api_err_invalid(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        val_s: *const ::core::ffi::c_char,
        val_n: int64_t,
        quote_val: bool,
    );
    fn check_string_array(
        arr: Array,
        name: *mut ::core::ffi::c_char,
        disallow_nl: bool,
        err: *mut Error,
    ) -> bool;
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn dict_get_value(
        dict: *mut dict_T,
        key: String_0,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn dict_set_var(
        dict: *mut dict_T,
        key: String_0,
        value: Object,
        del: bool,
        retval: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn normalize_index(
        buf: *mut buf_T,
        index: int64_t,
        end_exclusive: bool,
        oob: *mut bool,
    ) -> int64_t;
    fn buf_get_text(
        buf: *mut buf_T,
        lnum: int64_t,
        start_col: int64_t,
        end_col: int64_t,
        err: *mut Error,
    ) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn api_clear_error(value: *mut Error);
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn set_mark(
        buf: *mut buf_T,
        name: String_0,
        line: Integer,
        col: Integer,
        err: *mut Error,
    ) -> bool;
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn buf_ensure_loaded(buf: *mut buf_T) -> bool;
    fn do_buffer(
        action: ::core::ffi::c_int,
        start: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn buf_updates_register(
        buf: *mut buf_T,
        channel_id: uint64_t,
        cb: BufUpdateCallbacks,
        send_buffer: bool,
    ) -> bool;
    fn buf_updates_unregister(buf: *mut buf_T, channelid: uint64_t);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor_col(win: *mut win_T);
    fn check_visual_pos();
    fn rename_buffer(new_fname: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static State: GlobalCell<::core::ffi::c_int>;
    static RedrawingDisabled: GlobalCell<::core::ffi::c_int>;
    fn modify_keymap(
        channel_id: uint64_t,
        buffer: Buffer,
        is_unmap: bool,
        mode: String_0,
        lhs: String_0,
        rhs: String_0,
        opts: *mut KeyDict_keymap,
        err: *mut Error,
    );
    fn keymap_array(mode: String_0, buf: *mut buf_T, arena: *mut Arena) -> Array;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn mark_get(
        buf: *mut buf_T,
        win: *mut win_T,
        fmp: *mut fmark_T,
        flag: MarkGet,
        name: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn mark_adjust_buf(
        buf: *mut buf_T,
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
        adjust_folds: bool,
        mode: MarkAdjustMode,
        op: ExtmarkOp,
    );
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
        noalloc: bool,
    ) -> ::core::ffi::c_int;
    fn ml_delete_buf(buf: *mut buf_T, lnum: linenr_T, message: bool) -> ::core::ffi::c_int;
    fn ml_find_line_or_offset(
        buf: *mut buf_T,
        lnum: linenr_T,
        offp: *mut ::core::ffi::c_int,
        no_ff: bool,
    ) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn changed_cline_bef_curs(wp: *mut win_T);
    fn invalidate_botline_win(wp: *mut win_T);
    fn get_region_bytecount(
        buf: *mut buf_T,
        start_lnum: linenr_T,
        end_lnum: linenr_T,
        start_col: colnr_T,
        end_col: colnr_T,
    ) -> bcount_t;
    static p_acd: GlobalCell<::core::ffi::c_int>;
    fn u_save_buf(buf: *mut buf_T, top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
pub const MODE_INSERT: C2Rust_Unnamed_17 = 16;
pub const FORWARD: C2Rust_Unnamed_16 = 1;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_16 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_16 = 3;
pub const BACKWARD: C2Rust_Unnamed_16 = -1;
pub const kDirectionNotSet: C2Rust_Unnamed_16 = 0;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_17 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_17 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_17 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_17 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_17 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_17 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_17 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_17 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_17 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_17 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_17 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_17 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_17 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_17 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_17 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_17 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_17 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_17 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_17 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub const KEYSET_OPTIDX_buf_attach__on_bytes: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_lines: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_detach: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_reload: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_changedtick: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const BUF_UPDATE_CALLBACKS_INIT: BufUpdateCallbacks = BufUpdateCallbacks {
    on_lines: LUA_NOREF,
    on_bytes: LUA_NOREF,
    on_changedtick: LUA_NOREF,
    on_detach: LUA_NOREF,
    on_reload: LUA_NOREF,
    utf_sizes: false_0 != 0,
    preview: false_0 != 0,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn api_buf_ensure_loaded(mut buf: Buffer, mut err: *mut Error) -> *mut buf_T {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return ::core::ptr::null_mut::<buf_T>();
    }
    if (*b).b_ml.ml_mfp.is_null() && !buf_ensure_loaded(b) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to load buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<buf_T>();
    }
    return b;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_line_count(mut buf: Buffer, mut err: *mut Error) -> Integer {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Integer;
    }
    if (*b).b_ml.ml_mfp.is_null() {
        return 0 as Integer;
    }
    return (*b).b_ml.ml_line_count as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_attach(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut send_buffer: Boolean,
    mut opts: *mut KeyDict_buf_attach,
    mut err: *mut Error,
) -> Boolean {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return false_0 != 0;
    }
    let mut cb: BufUpdateCallbacks = BUF_UPDATE_CALLBACKS_INIT;
    if channel_id == LUA_INTERNAL_CALL {
        if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_lines
            != 0 as ::core::ffi::c_ulonglong
        {
            cb.on_lines = (*opts).on_lines;
            (*opts).on_lines = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_bytes
            != 0 as ::core::ffi::c_ulonglong
        {
            cb.on_bytes = (*opts).on_bytes;
            (*opts).on_bytes = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_changedtick
            != 0 as ::core::ffi::c_ulonglong
        {
            cb.on_changedtick = (*opts).on_changedtick;
            (*opts).on_changedtick = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_detach
            != 0 as ::core::ffi::c_ulonglong
        {
            cb.on_detach = (*opts).on_detach;
            (*opts).on_detach = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_reload
            != 0 as ::core::ffi::c_ulonglong
        {
            cb.on_reload = (*opts).on_reload;
            (*opts).on_reload = LUA_NOREF as LuaRef;
        }
        cb.utf_sizes = (*opts).utf_sizes as bool;
        cb.preview = (*opts).preview as bool;
    }
    return buf_updates_register(b, channel_id, cb, send_buffer as bool);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_detach(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut err: *mut Error,
) -> Boolean {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return false_0 != 0;
    }
    buf_updates_unregister(b, channel_id);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_lines(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut strict_indexing: Boolean,
    mut arena: *mut Arena,
    mut lstate: *mut lua_State,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if (*b).b_ml.ml_mfp.is_null() {
        return rv;
    }
    let mut oob: bool = false_0 != 0;
    start = normalize_index(b, start as int64_t, true_0 != 0, &raw mut oob) as Integer;
    end = normalize_index(b, end as int64_t, true_0 != 0, &raw mut oob) as Integer;
    if !(!strict_indexing || !oob) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if start >= end {
        return rv;
    }
    let mut size: size_t = (end - start) as size_t;
    init_line_array(lstate, &raw mut rv, size, arena);
    buf_collect_lines(
        b,
        size,
        start as linenr_T,
        0 as ::core::ffi::c_int,
        channel_id != VIML_INTERNAL_CALL,
        &raw mut rv,
        lstate,
        arena,
    );
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_lines(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut strict_indexing: Boolean,
    mut replacement: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
    if b.is_null() {
        return;
    }
    let mut oob: bool = false_0 != 0;
    start = normalize_index(b, start as int64_t, true_0 != 0, &raw mut oob) as Integer;
    end = normalize_index(b, end as int64_t, true_0 != 0, &raw mut oob) as Integer;
    if !(!strict_indexing || !oob) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if !(start <= end) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'start' is higher than 'end'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    if !check_string_array(
        replacement,
        b"replacement string\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        disallow_nl,
        err,
    ) {
        return;
    }
    let mut new_len: size_t = replacement.size;
    let mut old_len: size_t = (end - start) as size_t;
    let mut extra: ptrdiff_t = 0 as ptrdiff_t;
    let mut lines: *mut *mut ::core::ffi::c_char = (if new_len != 0 as size_t {
        arena_alloc(
            arena,
            new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
            true_0 != 0,
        )
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    let mut i: size_t = 0 as size_t;
    while i < new_len {
        let l: String_0 = (*replacement.items.offset(i as isize)).data.string;
        *lines.offset(i as isize) = arena_memdupz(arena, l.data, l.size);
        memchrsub(
            *lines.offset(i as isize) as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            l.size,
        );
        i = i.wrapping_add(1);
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    's_382: {
        if (*b).b_p_ma == 0 {
            api_set_error(
                err,
                kErrorTypeException,
                b"Buffer is not 'modifiable'\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if u_save_buf(b, (start - 1 as Integer) as linenr_T, end as linenr_T)
            == 0 as ::core::ffi::c_int
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to save undo information\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            let mut deleted_bytes: bcount_t = get_region_bytecount(
                b,
                start as linenr_T,
                end as linenr_T,
                0 as colnr_T,
                0 as colnr_T,
            );
            let mut to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_0: size_t = 0 as size_t;
            while i_0 < to_delete {
                if ml_delete_buf(b, start as linenr_T, false) == 0 as ::core::ffi::c_int {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to delete line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_382;
                } else {
                    i_0 = i_0.wrapping_add(1);
                }
            }
            if to_delete > 0 as size_t {
                extra -= to_delete as ptrdiff_t;
            }
            let mut to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut inserted_bytes: bcount_t = 0 as bcount_t;
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_replace {
                let mut lnum: int64_t = start as int64_t + i_1 as int64_t;
                if !(lnum < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_382;
                } else if ml_replace_buf(
                    b,
                    lnum as linenr_T,
                    *lines.offset(i_1 as isize),
                    false,
                    true,
                ) == 0 as ::core::ffi::c_int
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to replace line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_382;
                } else {
                    inserted_bytes +=
                        strlen(*lines.offset(i_1 as isize)) as bcount_t + 1 as bcount_t;
                    i_1 = i_1.wrapping_add(1);
                }
            }
            let mut i_2: size_t = to_replace;
            while i_2 < new_len {
                let mut lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
                if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_382;
                } else if ml_append_buf(
                    b,
                    lnum_0 as linenr_T,
                    *lines.offset(i_2 as isize),
                    0 as colnr_T,
                    false,
                ) == 0 as ::core::ffi::c_int
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to insert line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_382;
                } else {
                    inserted_bytes +=
                        strlen(*lines.offset(i_2 as isize)) as bcount_t + 1 as bcount_t;
                    extra += 1;
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let mut adjust: linenr_T = if end > start {
                MAXLNUM as ::core::ffi::c_int as linenr_T
            } else {
                0 as linenr_T
            };
            mark_adjust_buf(
                b,
                start as linenr_T,
                (end - 1 as Integer) as linenr_T,
                adjust,
                extra as linenr_T,
                true,
                kMarkAdjustApi,
                kExtmarkNOOP,
            );
            if VIsual_active.get() as ::core::ffi::c_int != 0
                && b == curbuf.get()
                && (*VIsual.ptr()).lnum >= start as linenr_T
            {
                if (*VIsual.ptr()).lnum >= end as linenr_T {
                    (*VIsual.ptr()).lnum += extra as linenr_T;
                }
                check_visual_pos();
            }
            extmark_splice(
                b,
                start as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                (end - start) as ::core::ffi::c_int,
                0 as colnr_T,
                deleted_bytes,
                new_len as ::core::ffi::c_int,
                0 as colnr_T,
                inserted_bytes,
                kExtmarkUndo,
            );
            changed_lines(
                b,
                start as linenr_T,
                0 as colnr_T,
                end as linenr_T,
                extra as linenr_T,
                true,
            );
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut win: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !win.is_null() {
                    if (*win).w_buffer == b {
                        fix_cursor(win, start as linenr_T, end as linenr_T, extra as linenr_T);
                    }
                    win = (*win).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
    }
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_text(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut start_row: Integer,
    mut start_col: Integer,
    mut end_row: Integer,
    mut end_col: Integer,
    mut replacement: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut scratch: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut scratch__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    scratch.capacity = 1 as size_t;
    scratch.items = &raw mut scratch__items as *mut Object;
    if replacement.size == 0 as size_t {
        let c2rust_fresh1 = scratch.size;
        scratch.size = scratch.size.wrapping_add(1);
        *scratch.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        };
        replacement = scratch;
    }
    let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
    if b.is_null() {
        return;
    }
    let mut oob: bool = false_0 != 0;
    start_row = normalize_index(b, start_row as int64_t, false_0 != 0, &raw mut oob) as Integer;
    if oob {
        api_err_invalid(
            err,
            b"start_row\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    end_row = normalize_index(b, end_row as int64_t, false_0 != 0, &raw mut oob) as Integer;
    if oob {
        api_err_invalid(
            err,
            b"end_row\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    let mut str_at_start: *mut ::core::ffi::c_char = ml_get_buf(b, start_row as linenr_T);
    let mut len_at_start: colnr_T = ml_get_buf_len(b, start_row as linenr_T);
    str_at_start = arena_memdupz(arena, str_at_start, len_at_start as size_t);
    start_col = if start_col < 0 as Integer {
        len_at_start as Integer + start_col + 1 as Integer
    } else {
        start_col
    };
    if !(start_col >= 0 as Integer && start_col <= len_at_start as Integer) {
        api_err_invalid(
            err,
            b"start_col\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    let mut str_at_end: *mut ::core::ffi::c_char = ml_get_buf(b, end_row as linenr_T);
    let mut len_at_end: colnr_T = ml_get_buf_len(b, end_row as linenr_T);
    str_at_end = arena_memdupz(arena, str_at_end, len_at_end as size_t);
    end_col = if end_col < 0 as Integer {
        len_at_end as Integer + end_col + 1 as Integer
    } else {
        end_col
    };
    if !(end_col >= 0 as Integer && end_col <= len_at_end as Integer) {
        api_err_invalid(
            err,
            b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    if !(start_row <= end_row && !(end_row == start_row && start_col > end_col)) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'start' is higher than 'end'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    if !check_string_array(
        replacement,
        b"replacement string\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        disallow_nl,
        err,
    ) {
        return;
    }
    let mut new_len: size_t = replacement.size;
    let mut new_byte: bcount_t = 0 as bcount_t;
    let mut old_byte: bcount_t = 0 as bcount_t;
    if start_row == end_row {
        old_byte = end_col as bcount_t - start_col as bcount_t;
    } else {
        old_byte = (old_byte as ::core::ffi::c_long
            + (len_at_start as Integer - start_col) as ::core::ffi::c_long)
            as bcount_t;
        let mut i: int64_t = 1 as int64_t;
        while i < end_row - start_row {
            let mut lnum: int64_t = start_row as int64_t + i;
            old_byte += (ml_get_buf_len(b, lnum as linenr_T) + 1 as ::core::ffi::c_int) as bcount_t;
            i += 1;
        }
        old_byte += end_col as bcount_t + 1 as bcount_t;
    }
    let mut first_item: String_0 = (*replacement.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut last_item: String_0 = (*replacement
        .items
        .offset(replacement.size.wrapping_sub(1 as size_t) as isize))
    .data
    .string;
    let mut firstlen: size_t = (start_col as size_t).wrapping_add(first_item.size);
    let mut last_part_len: size_t = (len_at_end as size_t).wrapping_sub(end_col as size_t);
    if replacement.size == 1 as size_t {
        firstlen = firstlen.wrapping_add(last_part_len);
    }
    let mut first: *mut ::core::ffi::c_char = arena_allocz(arena, firstlen);
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    memcpy(
        first as *mut ::core::ffi::c_void,
        str_at_start as *const ::core::ffi::c_void,
        start_col as size_t,
    );
    memcpy(
        first.offset(start_col as isize) as *mut ::core::ffi::c_void,
        first_item.data as *const ::core::ffi::c_void,
        first_item.size,
    );
    memchrsub(
        first.offset(start_col as isize) as *mut ::core::ffi::c_void,
        NUL as ::core::ffi::c_char,
        NL as ::core::ffi::c_char,
        first_item.size,
    );
    if replacement.size == 1 as size_t {
        memcpy(
            first
                .offset(start_col as isize)
                .offset(first_item.size as isize) as *mut ::core::ffi::c_void,
            str_at_end.offset(end_col as isize) as *const ::core::ffi::c_void,
            last_part_len,
        );
    } else {
        last = arena_allocz(arena, last_item.size.wrapping_add(last_part_len));
        memcpy(
            last as *mut ::core::ffi::c_void,
            last_item.data as *const ::core::ffi::c_void,
            last_item.size,
        );
        memchrsub(
            last as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            last_item.size,
        );
        memcpy(
            last.offset(last_item.size as isize) as *mut ::core::ffi::c_void,
            str_at_end.offset(end_col as isize) as *const ::core::ffi::c_void,
            last_part_len,
        );
    }
    let mut lines: *mut *mut ::core::ffi::c_char = arena_alloc(
        arena,
        new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        true_0 != 0,
    ) as *mut *mut ::core::ffi::c_char;
    *lines.offset(0 as ::core::ffi::c_int as isize) = first;
    new_byte += first_item.size as bcount_t;
    let mut i_0: size_t = 1 as size_t;
    while i_0 < new_len.wrapping_sub(1 as size_t) {
        let l: String_0 = (*replacement.items.offset(i_0 as isize)).data.string;
        *lines.offset(i_0 as isize) = arena_memdupz(arena, l.data, l.size);
        memchrsub(
            *lines.offset(i_0 as isize) as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            l.size,
        );
        new_byte += l.size as bcount_t + 1 as bcount_t;
        i_0 = i_0.wrapping_add(1);
    }
    if replacement.size > 1 as size_t {
        *lines.offset(replacement.size.wrapping_sub(1 as size_t) as isize) = last;
        new_byte += last_item.size as bcount_t + 1 as bcount_t;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    's_652: {
        if (*b).b_p_ma == 0 {
            api_set_error(
                err,
                kErrorTypeException,
                b"Buffer is not 'modifiable'\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if u_save_buf(
            b,
            start_row as linenr_T - 1 as linenr_T,
            end_row as linenr_T + 1 as linenr_T,
        ) == 0 as ::core::ffi::c_int
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to save undo information\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            let mut extra: ptrdiff_t = 0 as ptrdiff_t;
            let mut old_len: size_t = (end_row - start_row + 1 as Integer) as size_t;
            let mut to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_delete {
                if ml_delete_buf(b, start_row as linenr_T, false) == 0 as ::core::ffi::c_int {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to delete line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_652;
                } else {
                    i_1 = i_1.wrapping_add(1);
                }
            }
            if to_delete > 0 as size_t {
                extra -= to_delete as ptrdiff_t;
            }
            let mut to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut i_2: size_t = 0 as size_t;
            while i_2 < to_replace {
                let mut lnum_0: int64_t = start_row as int64_t + i_2 as int64_t;
                if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_652;
                } else if ml_replace_buf(
                    b,
                    lnum_0 as linenr_T,
                    *lines.offset(i_2 as isize),
                    false,
                    true,
                ) == 0 as ::core::ffi::c_int
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to replace line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_652;
                } else {
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let mut i_3: size_t = to_replace;
            while i_3 < new_len {
                let mut lnum_1: int64_t = start_row as int64_t + i_3 as int64_t - 1 as int64_t;
                if !(lnum_1 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_652;
                } else if ml_append_buf(
                    b,
                    lnum_1 as linenr_T,
                    *lines.offset(i_3 as isize),
                    0 as colnr_T,
                    false,
                ) == 0 as ::core::ffi::c_int
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to insert line\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break 's_652;
                } else {
                    extra += 1;
                    i_3 = i_3.wrapping_add(1);
                }
            }
            let mut col_extent: colnr_T = (end_col
                - (if end_row == start_row {
                    start_col
                } else {
                    0 as Integer
                })) as colnr_T;
            let mut adjust: linenr_T = if end_row >= start_row {
                MAXLNUM as ::core::ffi::c_int as linenr_T
            } else {
                0 as linenr_T
            };
            mark_adjust_buf(
                b,
                start_row as linenr_T,
                end_row as linenr_T - 1 as linenr_T,
                adjust,
                extra as linenr_T,
                true,
                kMarkAdjustApi,
                kExtmarkNOOP,
            );
            if VIsual_active.get() as ::core::ffi::c_int != 0
                && b == curbuf.get()
                && VIsual_mode.get() != 22 as ::core::ffi::c_int
            {
                fix_pos_col(
                    b,
                    VIsual.ptr(),
                    start_row as linenr_T,
                    start_col as colnr_T,
                    end_row as linenr_T,
                    end_col as colnr_T,
                    new_len as linenr_T,
                    last_item.size as colnr_T,
                    1 as colnr_T,
                );
                check_visual_pos();
            }
            extmark_splice(
                b,
                start_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                start_col as colnr_T,
                (end_row - start_row) as ::core::ffi::c_int,
                col_extent,
                old_byte,
                new_len as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                last_item.size as colnr_T,
                new_byte,
                kExtmarkUndo,
            );
            changed_lines(
                b,
                start_row as linenr_T,
                start_col as colnr_T,
                end_row as linenr_T + 1 as linenr_T,
                extra as linenr_T,
                true,
            );
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut win: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !win.is_null() {
                    if (*win).w_buffer == b {
                        if (*win).w_cursor.lnum as Integer >= start_row
                            && (*win).w_cursor.lnum as Integer <= end_row
                        {
                            fix_cursor_cols(
                                win,
                                start_row as linenr_T,
                                start_col as colnr_T,
                                end_row as linenr_T,
                                end_col as colnr_T,
                                new_len as linenr_T,
                                last_item.size as colnr_T,
                            );
                        } else {
                            fix_cursor(
                                win,
                                start_row as linenr_T,
                                end_row as linenr_T,
                                extra as linenr_T,
                            );
                        }
                    }
                    win = (*win).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
    }
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_text(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut start_row: Integer,
    mut start_col: Integer,
    mut end_row: Integer,
    mut end_col: Integer,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut lstate: *mut lua_State,
    mut err: *mut Error,
) -> Array {
    let mut str: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if (*b).b_ml.ml_mfp.is_null() {
        return rv;
    }
    let mut oob: bool = false_0 != 0;
    start_row = normalize_index(b, start_row as int64_t, false_0 != 0, &raw mut oob) as Integer;
    end_row = normalize_index(b, end_row as int64_t, false_0 != 0, &raw mut oob) as Integer;
    if oob {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if !(start_row <= end_row) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'start' is higher than 'end'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    let mut replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let mut size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
    init_line_array(lstate, &raw mut rv, size, arena);
    if start_row == end_row {
        let mut line: String_0 = buf_get_text(
            b,
            start_row as int64_t,
            start_col as int64_t,
            end_col as int64_t,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            push_linestr(
                lstate,
                &raw mut rv,
                line.data,
                line.size,
                0 as ::core::ffi::c_int,
                replace_nl,
                arena,
            );
            return rv;
        }
    } else {
        str = buf_get_text(
            b,
            start_row as int64_t,
            start_col as int64_t,
            (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            push_linestr(
                lstate,
                &raw mut rv,
                str.data,
                str.size,
                0 as ::core::ffi::c_int,
                replace_nl,
                arena,
            );
            if size > 2 as size_t {
                buf_collect_lines(
                    b,
                    size.wrapping_sub(2 as size_t),
                    start_row as linenr_T + 1 as linenr_T,
                    1 as ::core::ffi::c_int,
                    replace_nl,
                    &raw mut rv,
                    lstate,
                    arena,
                );
            }
            str = buf_get_text(b, end_row as int64_t, 0 as int64_t, end_col as int64_t, err);
            if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                push_linestr(
                    lstate,
                    &raw mut rv,
                    str.data,
                    str.size,
                    size.wrapping_sub(1 as size_t) as ::core::ffi::c_int,
                    replace_nl,
                    arena,
                );
            }
        }
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_offset(
    mut buf: Buffer,
    mut index: Integer,
    mut err: *mut Error,
) -> Integer {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Integer;
    }
    if (*b).b_ml.ml_mfp.is_null() {
        return -1 as Integer;
    }
    if !(index >= 0 as Integer && index <= (*b).b_ml.ml_line_count as Integer) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    return ml_find_line_or_offset(
        b,
        index as linenr_T + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_var(
    mut buf: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_get_value((*b).b_vars, name, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_changedtick(mut buf: Buffer, mut err: *mut Error) -> Integer {
    let b: *const buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return -1 as Integer;
    }
    return buf_get_changedtick(b);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_keymap(
    mut buf: Buffer,
    mut mode: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    return keymap_array(mode, b, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_keymap(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    modify_keymap(channel_id, buf, false_0 != 0, mode, lhs, rhs, opts, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_keymap(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut mode: String_0,
    mut lhs: String_0,
    mut err: *mut Error,
) {
    let mut rhs: String_0 = String_0 {
        data: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        size: 0 as size_t,
    };
    modify_keymap(
        channel_id,
        buf,
        true_0 != 0,
        mode,
        lhs,
        rhs,
        ::core::ptr::null_mut::<KeyDict_keymap>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_var(
    mut buf: Buffer,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    dict_set_var(
        (*b).b_vars,
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_var(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    dict_set_var(
        (*b).b_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_name(mut buf: Buffer, mut err: *mut Error) -> String_0 {
    let mut rv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() || (*b).b_ffname.is_null() {
        return rv;
    }
    return cstr_as_string((*b).b_ffname);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_name(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    let mut ren_ret: ::core::ffi::c_int = OK;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let is_curbuf: bool = b == curbuf.get();
    let save_acd: ::core::ffi::c_int = p_acd.get();
    if !is_curbuf {
        (*RedrawingDisabled.ptr()) += 1;
        p_acd.set(0 as ::core::ffi::c_int);
    }
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    aucmd_prepbuf(&raw mut aco, b);
    ren_ret = rename_buffer(name.data);
    aucmd_restbuf(&raw mut aco);
    if !is_curbuf {
        (*RedrawingDisabled.ptr()) -= 1;
        p_acd.set(save_acd);
    }
    try_leave(&raw mut tstate, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    if ren_ret == FAIL {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to rename buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_is_loaded(mut buf: Buffer) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, &raw mut stub);
    api_clear_error(&raw mut stub);
    return !b.is_null() && !(*b).b_ml.ml_mfp.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_delete(
    mut buf: Buffer,
    mut opts: *mut KeyDict_buf_delete,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut force: bool = (*opts).force as bool;
    let mut unload: bool = (*opts).unload as bool;
    let mut result: ::core::ffi::c_int = do_buffer(
        if unload as ::core::ffi::c_int != 0 {
            DOBUF_UNLOAD as ::core::ffi::c_int
        } else {
            DOBUF_WIPE as ::core::ffi::c_int
        },
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*b).handle as ::core::ffi::c_int,
        force as ::core::ffi::c_int,
    );
    if result == FAIL {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to unload buffer.\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_is_valid(mut buf: Buffer) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_buffer_by_handle(buf, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) -> Boolean {
    let mut res: bool = false_0 != 0;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return res as Boolean;
    }
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    let mut fm: *mut fmark_T = mark_get(
        b,
        curwin.get(),
        ::core::ptr::null_mut::<fmark_T>(),
        kMarkAllNoResolve,
        *name.data as ::core::ffi::c_int,
    );
    if fm.is_null() {
        api_err_invalid(
            err,
            b"mark name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    if (*fm).mark.lnum != 0 as linenr_T && (*fm).fnum == (*b).handle {
        res = set_mark(b, name, 0 as Integer, 0 as Integer, err);
    }
    return res as Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut line: Integer,
    mut col: Integer,
    mut _opts: *mut KeyDict_empty,
    mut err: *mut Error,
) -> Boolean {
    let mut res: bool = false_0 != 0;
    let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
    if b.is_null() {
        return res as Boolean;
    }
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    res = set_mark(b, name, line, col, err);
    return res as Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut mark: ::core::ffi::c_char = *name.data;
    fm = mark_get(
        b,
        curwin.get(),
        ::core::ptr::null_mut::<fmark_T>(),
        kMarkAllNoResolve,
        mark as ::core::ffi::c_int,
    );
    if fm.is_null() {
        api_err_invalid(
            err,
            b"mark name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    if (*fm).fnum != (*b).handle {
        pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        pos = (*fm).mark;
    }
    rv = arena_array(arena, 2 as size_t);
    let c2rust_fresh2 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: pos.lnum as Integer,
        },
    };
    let c2rust_fresh3 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: pos.col as Integer,
        },
    };
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_call(
    mut buf: Buffer,
    mut fun: LuaRef,
    mut err: *mut Error,
) -> Object {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut res: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    aucmd_prepbuf(&raw mut aco, b);
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    res = nlua_call_ref(
        fun,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetLuaref,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
    aucmd_restbuf(&raw mut aco);
    try_leave(&raw mut tstate, err);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__buf_stats(
    mut buf: Buffer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    let mut rv: Dict = arena_dict(arena, 7 as size_t);
    let c2rust_fresh4 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh4 as isize) = key_value_pair {
        key: cstr_as_string(b"flush_count\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*b).flush_count as Integer,
            },
        },
    };
    let c2rust_fresh5 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh5 as isize) = key_value_pair {
        key: cstr_as_string(b"current_lnum\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*b).b_ml.ml_line_lnum as Integer,
            },
        },
    };
    let c2rust_fresh6 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"line_dirty\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: (*b).b_ml.ml_flags & 0x2 as ::core::ffi::c_int != 0,
            },
        },
    };
    let c2rust_fresh7 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh7 as isize) = key_value_pair {
        key: cstr_as_string(b"dirty_bytes\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*b).deleted_bytes as Integer,
            },
        },
    };
    let c2rust_fresh8 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh8 as isize) = key_value_pair {
        key: cstr_as_string(b"dirty_bytes2\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*b).deleted_bytes2 as Integer,
            },
        },
    };
    let c2rust_fresh9 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh9 as isize) = key_value_pair {
        key: cstr_as_string(b"virt_blocks\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: buf_meta_total(b, kMTMetaLines) as Integer,
            },
        },
    };
    let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
    if !(*b).b_u_curhead.is_null() {
        uhp = (*b).b_u_curhead;
    } else if !(*b).b_u_newhead.is_null() {
        uhp = (*b).b_u_newhead;
    }
    if !uhp.is_null() {
        let c2rust_fresh10 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"uhp_extmark_size\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*uhp).uh_extmark.size as Integer,
                },
            },
        };
    }
    return rv;
}
unsafe extern "C" fn fix_cursor(
    mut win: *mut win_T,
    mut lo: linenr_T,
    mut hi: linenr_T,
    mut extra: linenr_T,
) {
    if (*win).w_cursor.lnum >= lo {
        if (*win).w_cursor.lnum >= hi {
            (*win).w_cursor.lnum += extra;
        } else if extra < 0 as linenr_T {
            check_cursor_lnum(win);
        }
        check_cursor_col(win);
        changed_cline_bef_curs(win);
        (*win).w_valid &= !VALID_BOTLINE_AP;
        update_topline(win);
    } else {
        invalidate_botline_win(win);
    };
}
unsafe extern "C" fn fix_pos_col(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut start_row: linenr_T,
    mut start_col: colnr_T,
    mut end_row: linenr_T,
    mut end_col: colnr_T,
    mut new_rows: linenr_T,
    mut new_cols_at_end_row: colnr_T,
    mut mode_col_adj: colnr_T,
) {
    if (*pos).lnum < start_row {
        return;
    }
    let mut old_rows: linenr_T = end_row - start_row + 1 as linenr_T;
    let mut lnum_shift: linenr_T = new_rows - old_rows;
    if (*pos).lnum > end_row {
        (*pos).lnum += lnum_shift;
        return;
    }
    let mut end_row_change_start: colnr_T = if new_rows == 1 as linenr_T {
        start_col
    } else {
        0 as colnr_T
    };
    let mut end_row_change_end: colnr_T = end_row_change_start + new_cols_at_end_row;
    if (*pos).lnum == end_row && (*pos).col + mode_col_adj > end_col {
        (*pos).lnum += lnum_shift;
        (*pos).col += end_row_change_end - end_col;
        return;
    }
    let mut old_coladd: colnr_T = (*pos).coladd;
    (*pos).col += (*pos).coladd;
    (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
    let mut new_end_row: linenr_T = start_row + new_rows - 1 as linenr_T;
    if (*pos).lnum > new_end_row {
        (*pos).lnum = new_end_row;
        let mut len: colnr_T = ml_get_buf_len(buf, new_end_row);
        if (*pos).col < len {
            (*pos).col = len;
        }
    }
    if (*pos).lnum == new_end_row
        && (*pos).col > end_row_change_end
        && old_coladd == 0 as ::core::ffi::c_int
    {
        (*pos).col = end_row_change_end;
        if (*pos).col - mode_col_adj >= end_row_change_start {
            (*pos).col -= mode_col_adj;
        }
    }
}
unsafe extern "C" fn fix_cursor_cols(
    mut win: *mut win_T,
    mut start_row: linenr_T,
    mut start_col: colnr_T,
    mut end_row: linenr_T,
    mut end_col: colnr_T,
    mut new_rows: linenr_T,
    mut new_cols_at_end_row: colnr_T,
) {
    let mut mode_col_adj: colnr_T =
        if win == curwin.get() && State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
            0 as colnr_T
        } else {
            1 as colnr_T
        };
    fix_pos_col(
        (*win).w_buffer,
        &raw mut (*win).w_cursor,
        start_row,
        start_col,
        end_row,
        end_col,
        new_rows,
        new_cols_at_end_row,
        mode_col_adj,
    );
    check_cursor_col(win);
    changed_cline_bef_curs(win);
    invalidate_botline_win(win);
}
#[inline]
unsafe extern "C" fn init_line_array(
    mut lstate: *mut lua_State,
    mut a: *mut Array,
    mut size: size_t,
    mut arena: *mut Arena,
) {
    if !lstate.is_null() {
        lua_createtable(lstate, size as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    } else {
        *a = arena_array(arena, size);
    };
}
unsafe extern "C" fn push_linestr(
    mut lstate: *mut lua_State,
    mut a: *mut Array,
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut idx: ::core::ffi::c_int,
    mut replace_nl: bool,
    mut arena: *mut Arena,
) {
    if !lstate.is_null() {
        if !s.is_null()
            && replace_nl as ::core::ffi::c_int != 0
            && !strchr(s, '\n' as ::core::ffi::c_int).is_null()
        {
            let mut tmp: *mut ::core::ffi::c_char =
                xmemdupz(s as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
            strchrsub(tmp, '\n' as ::core::ffi::c_char, NUL as ::core::ffi::c_char);
            lua_pushlstring(lstate, tmp, len);
            xfree(tmp as *mut ::core::ffi::c_void);
        } else {
            lua_pushlstring(lstate, s, len);
        }
        lua_rawseti(
            lstate,
            -2 as ::core::ffi::c_int,
            idx + 1 as ::core::ffi::c_int,
        );
    } else {
        let mut str: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
        if len > 0 as size_t {
            str = arena_string(
                arena,
                String_0 {
                    data: s as *mut ::core::ffi::c_char,
                    size: len,
                },
            );
            if replace_nl {
                strchrsub(
                    str.data,
                    '\n' as ::core::ffi::c_char,
                    NUL as ::core::ffi::c_char,
                );
            }
        }
        let c2rust_fresh0 = (*a).size;
        (*a).size = (*a).size.wrapping_add(1);
        *(*a).items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: str },
        };
    };
}
#[no_mangle]
pub unsafe extern "C" fn buf_collect_lines(
    mut buf: *mut buf_T,
    mut n: size_t,
    mut start: linenr_T,
    mut start_idx: ::core::ffi::c_int,
    mut replace_nl: bool,
    mut l: *mut Array,
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
) {
    let mut i: size_t = 0 as size_t;
    while i < n {
        let mut lnum: linenr_T = start + i as linenr_T;
        let mut bufstr: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
        let mut bufstrlen: size_t = ml_get_buf_len(buf, lnum) as size_t;
        push_linestr(
            lstate,
            l,
            bufstr,
            bufstrlen,
            start_idx + i as ::core::ffi::c_int,
            replace_nl,
            arena,
        );
        i = i.wrapping_add(1);
    }
}
