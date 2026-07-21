use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, StringBuilder, Terminal, Timestamp,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, intmax_t, intptr_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T,
    scid_T, sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4,
    syn_time_T, synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, uvarnumber_T, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn strtoimax(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> intmax_t;
    fn abort() -> !;
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
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrchrnul(
        str: *const ::core::ffi::c_char,
        c: ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static dy_flags: GlobalCell<::core::ffi::c_uint>;
    static p_isf: GlobalCell<*mut ::core::ffi::c_char>;
    static p_isi: GlobalCell<*mut ::core::ffi::c_char>;
    static p_isp: GlobalCell<*mut ::core::ffi::c_char>;
    static curbuf: GlobalCell<*mut buf_T>;
    fn utf_char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_ptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_printable(c: ::core::ffi::c_int) -> bool;
    fn utf_class_tab(c: ::core::ffi::c_int, chartab: *const uint64_t) -> ::core::ffi::c_int;
    fn mb_islower(a: ::core::ffi::c_int) -> bool;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_isupper(a: ::core::ffi::c_int) -> bool;
    static utf8len_tab: [uint8_t; 256];
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn skip_to_option_part(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_has_wildcard(p: *const ::core::ffi::c_char) -> bool;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kOptDyFlagMsgsep: C2Rust_Unnamed_13 = 8;
pub const kOptDyFlagUhex: C2Rust_Unnamed_13 = 4;
pub const kOptDyFlagTruncate: C2Rust_Unnamed_13 = 2;
pub const kOptDyFlagLastline: C2Rust_Unnamed_13 = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_14 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_14 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_14 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_14 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_14 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_14 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_14 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_14 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_14 = 0;
pub const INT32_MIN: ::core::ffi::c_int =
    -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT32_MAX: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const INTMAX_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INTMAX_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isbdigit(mut c: ::core::ffi::c_int) -> bool {
    return c == '0' as ::core::ffi::c_int || c == '1' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isodigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '7' as ::core::ffi::c_int;
}
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const UVARNUMBER_MAX: ::core::ffi::c_ulong = UINT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static chartab_initialized: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static g_chartab: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);
pub const CT_CELL_MASK: ::core::ffi::c_int = 0x7 as ::core::ffi::c_int;
pub const CT_PRINT_CHAR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CT_ID_CHAR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CT_FNAME_CHAR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn init_chartab() -> ::core::ffi::c_int {
    return buf_init_chartab(curbuf.get(), true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn buf_init_chartab(
    mut buf: *mut buf_T,
    mut global: bool,
) -> ::core::ffi::c_int {
    if global {
        let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while c < ' ' as ::core::ffi::c_int {
            let c2rust_fresh0 = c;
            c = c + 1;
            (*g_chartab.ptr())[c2rust_fresh0 as usize] = (if dy_flags.get()
                & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                4 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            }) as uint8_t;
        }
        while c <= '~' as ::core::ffi::c_int {
            let c2rust_fresh1 = c;
            c = c + 1;
            (*g_chartab.ptr())[c2rust_fresh1 as usize] =
                (1 as ::core::ffi::c_int + CT_PRINT_CHAR) as uint8_t;
        }
        while c < 256 as ::core::ffi::c_int {
            if c >= 0xa0 as ::core::ffi::c_int {
                let c2rust_fresh2 = c;
                c = c + 1;
                (*g_chartab.ptr())[c2rust_fresh2 as usize] =
                    ((CT_PRINT_CHAR | CT_FNAME_CHAR) + 1 as ::core::ffi::c_int) as uint8_t;
            } else {
                let c2rust_fresh3 = c;
                c = c + 1;
                (*g_chartab.ptr())[c2rust_fresh3 as usize] = (if dy_flags.get()
                    & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    4 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                }) as uint8_t;
            }
        }
    }
    memset(
        &raw mut (*buf).b_chartab as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint64_t; 4]>(),
    );
    if (*buf).b_p_lisp != 0 {
        (*buf).b_chartab[('-' as ::core::ffi::c_int as ::core::ffi::c_uint
            >> 6 as ::core::ffi::c_int) as usize] = ((*buf).b_chartab
            [('-' as ::core::ffi::c_int as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong)
                << ('-' as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int))
            as uint64_t;
    }
    let mut i: ::core::ffi::c_int = if global as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int
    };
    while i <= 3 as ::core::ffi::c_int {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if i == 0 as ::core::ffi::c_int {
            p = p_isi.get();
        } else if i == 1 as ::core::ffi::c_int {
            p = p_isp.get();
        } else if i == 2 as ::core::ffi::c_int {
            p = p_isf.get();
        } else {
            p = (*buf).b_p_isk;
        }
        if parse_isopt(p, buf, false_0 != 0) == FAIL {
            return FAIL;
        }
        i += 1;
    }
    chartab_initialized.set(true_0 != 0);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_isopt(mut var: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return parse_isopt(var, ::core::ptr::null_mut::<buf_T>(), true_0 != 0);
}
unsafe extern "C" fn parse_isopt(
    mut var: *const ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut only_check: bool,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = var;
    while *p != 0 {
        let mut tilde: bool = false_0 != 0;
        let mut do_isalpha: bool = false_0 != 0;
        if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            tilde = true_0 != 0;
            p = p.offset(1);
        }
        let mut c: ::core::ffi::c_int = 0;
        if ascii_isdigit(*p as ::core::ffi::c_int) {
            c = getdigits_int(
                &raw mut p as *mut *mut ::core::ffi::c_char,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            );
        } else {
            c = mb_ptr2char_adv(&raw mut p);
        }
        let mut c2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
            if ascii_isdigit(*p as ::core::ffi::c_int) {
                c2 = getdigits_int(
                    &raw mut p as *mut *mut ::core::ffi::c_char,
                    true_0 != 0,
                    0 as ::core::ffi::c_int,
                );
            } else {
                c2 = mb_ptr2char_adv(&raw mut p);
            }
        }
        if c <= 0 as ::core::ffi::c_int
            || c >= 256 as ::core::ffi::c_int
            || c2 < c && c2 != -1 as ::core::ffi::c_int
            || c2 >= 256 as ::core::ffi::c_int
            || !(*p as ::core::ffi::c_int == NUL
                || *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
        {
            return FAIL;
        }
        let mut trail_comma: bool = *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int;
        p = skip_to_option_part(p);
        if trail_comma as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        if only_check {
            continue;
        }
        if c2 == -1 as ::core::ffi::c_int {
            if c == '@' as ::core::ffi::c_int {
                do_isalpha = true_0 != 0;
                c = 1 as ::core::ffi::c_int;
                c2 = 255 as ::core::ffi::c_int;
            } else {
                c2 = c;
            }
        }
        while c <= c2 {
            if !do_isalpha
                || mb_islower(c) as ::core::ffi::c_int != 0
                || mb_isupper(c) as ::core::ffi::c_int != 0
            {
                if var == p_isi.get() as *const ::core::ffi::c_char {
                    if tilde {
                        (*g_chartab.ptr())[c as usize] = ((*g_chartab.ptr())[c as usize]
                            as ::core::ffi::c_int
                            & !CT_ID_CHAR as uint8_t as ::core::ffi::c_int)
                            as uint8_t;
                    } else {
                        (*g_chartab.ptr())[c as usize] =
                            ((*g_chartab.ptr())[c as usize] as ::core::ffi::c_int | CT_ID_CHAR)
                                as uint8_t;
                    }
                } else if var == p_isp.get() as *const ::core::ffi::c_char {
                    if c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int {
                        if tilde {
                            (*g_chartab.ptr())[c as usize] = (((*g_chartab.ptr())[c as usize]
                                as ::core::ffi::c_int
                                & !CT_CELL_MASK)
                                + (if dy_flags.get()
                                    & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != 0
                                {
                                    4 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                }))
                                as uint8_t;
                            (*g_chartab.ptr())[c as usize] = ((*g_chartab.ptr())[c as usize]
                                as ::core::ffi::c_int
                                & !CT_PRINT_CHAR as uint8_t as ::core::ffi::c_int)
                                as uint8_t;
                        } else {
                            (*g_chartab.ptr())[c as usize] = (((*g_chartab.ptr())[c as usize]
                                as ::core::ffi::c_int
                                & !CT_CELL_MASK)
                                + 1 as ::core::ffi::c_int)
                                as uint8_t;
                            (*g_chartab.ptr())[c as usize] =
                                ((*g_chartab.ptr())[c as usize] as ::core::ffi::c_int
                                    | CT_PRINT_CHAR) as uint8_t;
                        }
                    }
                } else if var == p_isf.get() as *const ::core::ffi::c_char {
                    if tilde {
                        (*g_chartab.ptr())[c as usize] = ((*g_chartab.ptr())[c as usize]
                            as ::core::ffi::c_int
                            & !CT_FNAME_CHAR as uint8_t as ::core::ffi::c_int)
                            as uint8_t;
                    } else {
                        (*g_chartab.ptr())[c as usize] =
                            ((*g_chartab.ptr())[c as usize] as ::core::ffi::c_int | CT_FNAME_CHAR)
                                as uint8_t;
                    }
                } else if tilde {
                    (*buf).b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize] = ((*buf)
                        .b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_ulonglong
                        & !((1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int)))
                        as uint64_t;
                } else {
                    (*buf).b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize] = ((*buf)
                        .b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_ulonglong
                        | (1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int))
                        as uint64_t;
                }
            }
            c += 1;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn trans_characters(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: ::core::ffi::c_int,
) {
    let mut trs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = strlen(buf) as ::core::ffi::c_int;
    let mut room: ::core::ffi::c_int = bufsize - len;
    while *buf as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut trs_len: ::core::ffi::c_int = 0;
        trs_len = utfc_ptr2len(buf);
        if trs_len > 1 as ::core::ffi::c_int {
            len -= trs_len;
        } else {
            trs = transchar_byte(*buf as uint8_t as ::core::ffi::c_int);
            trs_len = strlen(trs) as ::core::ffi::c_int;
            if trs_len > 1 as ::core::ffi::c_int {
                room -= trs_len - 1 as ::core::ffi::c_int;
                if room <= 0 as ::core::ffi::c_int {
                    return;
                }
                memmove(
                    buf.offset(trs_len as isize) as *mut ::core::ffi::c_void,
                    buf.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
            memmove(
                buf as *mut ::core::ffi::c_void,
                trs as *const ::core::ffi::c_void,
                trs_len as size_t,
            );
            len -= 1;
        }
        buf = buf.offset(trs_len as isize);
    }
}
#[no_mangle]
pub unsafe extern "C" fn transstr_len(s: *const ::core::ffi::c_char, mut untab: bool) -> size_t {
    let mut p: *const ::core::ffi::c_char = s;
    let mut len: size_t = 0 as size_t;
    while *p != 0 {
        let l: size_t = utfc_ptr2len(p) as size_t;
        if l > 1 as size_t {
            if vim_isprintc(utf_ptr2char(p)) {
                len = len.wrapping_add(l);
            } else {
                let mut off: size_t = 0 as size_t;
                while off < l {
                    let mut c: ::core::ffi::c_int = utf_ptr2char(p.offset(off as isize));
                    let mut hexbuf: [::core::ffi::c_char; 9] = [0; 9];
                    len = len.wrapping_add(transchar_hex(
                        &raw mut hexbuf as *mut ::core::ffi::c_char,
                        c,
                    ));
                    off = off.wrapping_add(utf_ptr2len(p.offset(off as isize)) as size_t);
                }
            }
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == TAB && !untab {
            len = len.wrapping_add(1 as size_t);
            p = p.offset(1);
        } else {
            let c2rust_fresh12 = p;
            p = p.offset(1);
            let b2c_l: ::core::ffi::c_int =
                byte2cells(*c2rust_fresh12 as uint8_t as ::core::ffi::c_int);
            len = len.wrapping_add(
                (if b2c_l > 0 as ::core::ffi::c_int {
                    b2c_l
                } else {
                    4 as ::core::ffi::c_int
                }) as size_t,
            );
        }
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn transstr_buf(
    s: *const ::core::ffi::c_char,
    slen: ssize_t,
    buf: *mut ::core::ffi::c_char,
    buflen: size_t,
    mut untab: bool,
) -> size_t {
    let mut p: *const ::core::ffi::c_char = s;
    let mut buf_p: *mut ::core::ffi::c_char = buf;
    let buf_e: *mut ::core::ffi::c_char = buf_p
        .offset(buflen as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    while (slen < 0 as ssize_t || p.offset_from(s) < slen as isize)
        && *p as ::core::ffi::c_int != NUL
        && buf_p < buf_e
    {
        let l: size_t = utfc_ptr2len(p) as size_t;
        if l > 1 as size_t {
            if buf_p.offset(l as isize) > buf_e {
                break;
            }
            if vim_isprintc(utf_ptr2char(p)) {
                memmove(
                    buf_p as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    l,
                );
                buf_p = buf_p.offset(l as isize);
            } else {
                let mut off: size_t = 0 as size_t;
                while off < l {
                    let mut c: ::core::ffi::c_int = utf_ptr2char(p.offset(off as isize));
                    let mut hexbuf: [::core::ffi::c_char; 9] = [0; 9];
                    let hexlen: size_t =
                        transchar_hex(&raw mut hexbuf as *mut ::core::ffi::c_char, c);
                    if buf_p.offset(hexlen as isize) > buf_e {
                        break;
                    }
                    memmove(
                        buf_p as *mut ::core::ffi::c_void,
                        &raw mut hexbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        hexlen,
                    );
                    buf_p = buf_p.offset(hexlen as isize);
                    off = off.wrapping_add(utf_ptr2len(p.offset(off as isize)) as size_t);
                }
            }
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == TAB && !untab {
            let c2rust_fresh13 = p;
            p = p.offset(1);
            let c2rust_fresh14 = buf_p;
            buf_p = buf_p.offset(1);
            *c2rust_fresh14 = *c2rust_fresh13;
        } else {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            let tb: *const ::core::ffi::c_char =
                transchar_byte(*c2rust_fresh15 as uint8_t as ::core::ffi::c_int);
            let tb_len: size_t = strlen(tb);
            if buf_p.offset(tb_len as isize) > buf_e {
                break;
            }
            memmove(
                buf_p as *mut ::core::ffi::c_void,
                tb as *const ::core::ffi::c_void,
                tb_len,
            );
            buf_p = buf_p.offset(tb_len as isize);
        }
    }
    *buf_p = NUL as ::core::ffi::c_char;
    '_c2rust_label: {
        if buf_p <= buf_e {
        } else {
            __assert_fail(
                b"buf_p <= buf_e\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/charset.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                395 as ::core::ffi::c_uint,
                b"size_t transstr_buf(const char *const, const ssize_t, char *const, const size_t, _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return buf_p.offset_from(buf) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn transstr(
    s: *const ::core::ffi::c_char,
    mut untab: bool,
) -> *mut ::core::ffi::c_char {
    let len: size_t = transstr_len(s, untab).wrapping_add(1 as size_t);
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    transstr_buf(s, -1 as ssize_t, buf, len, untab);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn kv_transstr(
    mut str: *mut StringBuilder,
    s: *const ::core::ffi::c_char,
    mut untab: bool,
) -> size_t {
    if s.is_null() {
        return 0 as size_t;
    }
    let len: size_t = transstr_len(s, untab);
    if (*str).capacity < (*str).size.wrapping_add(len).wrapping_add(1 as size_t) {
        (*str).capacity = (*str).size.wrapping_add(len).wrapping_add(1 as size_t);
        (*str).capacity = (*str).capacity.wrapping_sub(1);
        (*str).capacity |= (*str).capacity >> 1 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 2 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 4 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 8 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 16 as ::core::ffi::c_int;
        (*str).capacity = (*str).capacity.wrapping_add(1);
        (*str).capacity = (*str).capacity;
        (*str).items = xrealloc(
            (*str).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*str).capacity),
        ) as *mut ::core::ffi::c_char;
    }
    transstr_buf(
        s,
        -1 as ssize_t,
        (*str).items.offset((*str).size as isize),
        len.wrapping_add(1 as size_t),
        untab,
    );
    (*str).size = (*str).size.wrapping_add(len);
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn str_foldcase(
    mut str: *mut ::core::ffi::c_char,
    mut orglen: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut len: ::core::ffi::c_int = orglen;
    if buf.is_null() {
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        ga_grow(&raw mut ga, len + 1 as ::core::ffi::c_int);
        memmove(ga.ga_data, str as *const ::core::ffi::c_void, len as size_t);
        ga.ga_len = len;
    } else {
        if len >= buflen {
            len = buflen - 1 as ::core::ffi::c_int;
        }
        memmove(
            buf as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
    }
    if buf.is_null() {
        *(ga.ga_data as *mut ::core::ffi::c_char).offset(len as isize) = NUL as ::core::ffi::c_char;
    } else {
        *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (if buf.is_null() {
        *(ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize) as ::core::ffi::c_int
    } else {
        *buf.offset(i as isize) as ::core::ffi::c_int
    }) != NUL
    {
        let mut c: ::core::ffi::c_int = utf_ptr2char(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
        let mut olen: ::core::ffi::c_int = utf_ptr2len(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
        let mut lc: ::core::ffi::c_int = mb_tolower(c);
        if (c < 0x80 as ::core::ffi::c_int || olen > 1 as ::core::ffi::c_int) && c != lc {
            let mut nlen: ::core::ffi::c_int = utf_char2len(lc);
            if olen != nlen {
                if nlen > olen {
                    if buf.is_null() {
                        ga_grow(&raw mut ga, nlen - olen + 1 as ::core::ffi::c_int);
                    } else if len + nlen - olen >= buflen {
                        lc = c;
                        nlen = olen;
                    }
                }
                if olen != nlen {
                    if buf.is_null() {
                        memmove(
                            (ga.ga_data as *mut ::core::ffi::c_char)
                                .offset(i as isize)
                                .offset(nlen as isize)
                                as *mut ::core::ffi::c_void,
                            (ga.ga_data as *mut ::core::ffi::c_char)
                                .offset(i as isize)
                                .offset(olen as isize)
                                as *const ::core::ffi::c_void,
                            strlen(
                                (ga.ga_data as *mut ::core::ffi::c_char)
                                    .offset(i as isize)
                                    .offset(olen as isize),
                            )
                            .wrapping_add(1 as size_t),
                        );
                        ga.ga_len += nlen - olen;
                    } else {
                        memmove(
                            buf.offset(i as isize).offset(nlen as isize)
                                as *mut ::core::ffi::c_void,
                            buf.offset(i as isize).offset(olen as isize)
                                as *const ::core::ffi::c_void,
                            strlen(buf.offset(i as isize).offset(olen as isize))
                                .wrapping_add(1 as size_t),
                        );
                        len += nlen - olen;
                    }
                }
            }
            utf_char2bytes(
                lc,
                if buf.is_null() {
                    (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
                } else {
                    buf.offset(i as isize)
                },
            );
        }
        i += utfc_ptr2len(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
    }
    if buf.is_null() {
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
    return buf;
}
static transchar_charbuf: GlobalCell<[uint8_t; 11]> = GlobalCell::new([0; 11]);
#[no_mangle]
pub unsafe extern "C" fn transchar(mut c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    return transchar_buf(curbuf.get(), c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_buf(
    mut buf: *const buf_T,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c < 0 as ::core::ffi::c_int {
        (*transchar_charbuf.ptr())[0 as ::core::ffi::c_int as usize] = '~' as uint8_t;
        (*transchar_charbuf.ptr())[1 as ::core::ffi::c_int as usize] = '@' as uint8_t;
        i = 2 as ::core::ffi::c_int;
        c = if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        };
    }
    if !chartab_initialized.get()
        && (c >= ' ' as ::core::ffi::c_int && c <= '~' as ::core::ffi::c_int)
        || c <= 0xff as ::core::ffi::c_int && vim_isprintc(c) as ::core::ffi::c_int != 0
    {
        (*transchar_charbuf.ptr())[i as usize] = c as uint8_t;
        (*transchar_charbuf.ptr())[(i + 1 as ::core::ffi::c_int) as usize] = NUL as uint8_t;
    } else if c <= 0xff as ::core::ffi::c_int {
        transchar_nonprint(
            buf,
            (transchar_charbuf.ptr() as *mut uint8_t as *mut ::core::ffi::c_char)
                .offset(i as isize),
            c,
        );
    } else {
        transchar_hex(
            (transchar_charbuf.ptr() as *mut uint8_t as *mut ::core::ffi::c_char)
                .offset(i as isize),
            c,
        );
    }
    return transchar_charbuf.ptr() as *mut uint8_t as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn transchar_byte(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    return transchar_byte_buf(curbuf.get(), c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_byte_buf(
    mut buf: *const buf_T,
    c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if c >= 0x80 as ::core::ffi::c_int {
        transchar_nonprint(
            buf,
            transchar_charbuf.ptr() as *mut uint8_t as *mut ::core::ffi::c_char,
            c,
        );
        return transchar_charbuf.ptr() as *mut uint8_t as *mut ::core::ffi::c_char;
    }
    return transchar_buf(buf, c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_nonprint(
    mut buf: *const buf_T,
    mut charbuf: *mut ::core::ffi::c_char,
    mut c: ::core::ffi::c_int,
) {
    if c == NL {
        c = NUL;
    } else if !buf.is_null() && c == CAR && get_fileformat(buf) == EOL_MAC {
        c = NL;
    }
    '_c2rust_label: {
        if c <= 0xff as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"c <= 0xff\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/charset.rs\0".as_ptr() as *const ::core::ffi::c_char,
                613 as ::core::ffi::c_uint,
                b"void transchar_nonprint(const buf_T *, char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if dy_flags.get() & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || c > 0x7f as ::core::ffi::c_int
    {
        transchar_hex(charbuf, c);
    } else {
        *charbuf.offset(0 as ::core::ffi::c_int as isize) = '^' as ::core::ffi::c_char;
        *charbuf.offset(1 as ::core::ffi::c_int as isize) =
            (c ^ 0x40 as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_char;
        *charbuf.offset(2 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn transchar_hex(
    buf: *mut ::core::ffi::c_char,
    c: ::core::ffi::c_int,
) -> size_t {
    let mut i: size_t = 0 as size_t;
    let c2rust_fresh4 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh4 as isize) = '<' as ::core::ffi::c_char;
    if c > 0xff as ::core::ffi::c_int {
        if c > 0xffff as ::core::ffi::c_int {
            let c2rust_fresh5 = i;
            i = i.wrapping_add(1);
            *buf.offset(c2rust_fresh5 as isize) =
                nr2hex(c as ::core::ffi::c_uint >> 20 as ::core::ffi::c_int) as ::core::ffi::c_char;
            let c2rust_fresh6 = i;
            i = i.wrapping_add(1);
            *buf.offset(c2rust_fresh6 as isize) =
                nr2hex(c as ::core::ffi::c_uint >> 16 as ::core::ffi::c_int) as ::core::ffi::c_char;
        }
        let c2rust_fresh7 = i;
        i = i.wrapping_add(1);
        *buf.offset(c2rust_fresh7 as isize) =
            nr2hex(c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int) as ::core::ffi::c_char;
        let c2rust_fresh8 = i;
        i = i.wrapping_add(1);
        *buf.offset(c2rust_fresh8 as isize) =
            nr2hex(c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_char;
    }
    let c2rust_fresh9 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh9 as isize) =
        nr2hex(c as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int) as ::core::ffi::c_char;
    let c2rust_fresh10 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh10 as isize) = nr2hex(c as ::core::ffi::c_uint) as ::core::ffi::c_char;
    let c2rust_fresh11 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh11 as isize) = '>' as ::core::ffi::c_char;
    *buf.offset(i as isize) = NUL as ::core::ffi::c_char;
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn rl_mirror_ascii(
    mut str: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
) {
    let mut p1: *mut ::core::ffi::c_char = str;
    let mut p2: *mut ::core::ffi::c_char = (if !end.is_null() {
        end
    } else {
        str.offset(strlen(str) as isize)
    })
    .offset(-(1 as ::core::ffi::c_int as isize));
    while p1 < p2 {
        let mut t: ::core::ffi::c_char = *p1;
        *p1 = *p2;
        *p2 = t;
        p1 = p1.offset(1);
        p2 = p2.offset(-1);
    }
}
#[inline]
unsafe extern "C" fn nr2hex(mut n: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    if n & 0xf as ::core::ffi::c_uint <= 9 as ::core::ffi::c_uint {
        return (n & 0xf as ::core::ffi::c_uint).wrapping_add('0' as ::core::ffi::c_uint);
    }
    return (n & 0xf as ::core::ffi::c_uint)
        .wrapping_sub(10 as ::core::ffi::c_uint)
        .wrapping_add('a' as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn byte2cells(mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if b >= 0x80 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (*g_chartab.ptr())[b as usize] as ::core::ffi::c_int & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn char2cells(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0 as ::core::ffi::c_int {
        return char2cells(if c == K_SPECIAL {
            KS_SPECIAL
        } else {
            if c == NUL {
                KS_ZERO
            } else {
                -c & 0xff as ::core::ffi::c_int
            }
        }) + 2 as ::core::ffi::c_int;
    }
    if c >= 0x80 as ::core::ffi::c_int {
        return utf_char2cells(c);
    }
    return (*g_chartab.ptr())[(c & 0xff as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
        & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn ptr2cells(mut p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    if *p as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return utf_ptr2cells(p_in);
    }
    return (*g_chartab.ptr())[*p as usize] as ::core::ffi::c_int & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn vim_strsize(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return vim_strnsize(s, MAXCOL as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn vim_strnsize(
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !s.is_null() {
        } else {
            __assert_fail(
                b"s != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/charset.rs\0".as_ptr() as *const ::core::ffi::c_char,
                766 as ::core::ffi::c_uint,
                b"int vim_strnsize(const char *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *s as ::core::ffi::c_int != NUL && {
        len -= 1;
        len >= 0 as ::core::ffi::c_int
    } {
        let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
        size += ptr2cells(s);
        s = s.offset(l as isize);
        len -= l - 1 as ::core::ffi::c_int;
    }
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isIDc(mut c: ::core::ffi::c_int) -> bool {
    return c > 0 as ::core::ffi::c_int
        && c < 0x100 as ::core::ffi::c_int
        && (*g_chartab.ptr())[c as usize] as ::core::ffi::c_int & CT_ID_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc(c: ::core::ffi::c_int) -> bool {
    return vim_iswordc_buf(c, curbuf.get());
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc_tab(c: ::core::ffi::c_int, chartab: *const uint64_t) -> bool {
    return if c >= 0x100 as ::core::ffi::c_int {
        (utf_class_tab(c, chartab) >= 2 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        (c > 0 as ::core::ffi::c_int
            && *chartab.offset((c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_ulonglong) as ::core::ffi::c_int
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc_buf(c: ::core::ffi::c_int, buf: *mut buf_T) -> bool {
    return vim_iswordc_tab(c, &raw mut (*buf).b_chartab as *mut uint64_t);
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool {
    return vim_iswordp_buf(p, curbuf.get());
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordp_buf(p: *const ::core::ffi::c_char, buf: *mut buf_T) -> bool {
    let mut c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
    if utf8len_tab[c as usize] as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
        c = utf_ptr2char(p);
    }
    return vim_iswordc_buf(c, buf);
}
#[no_mangle]
pub unsafe extern "C" fn vim_isfilec(mut c: ::core::ffi::c_int) -> bool {
    return c >= 0x100 as ::core::ffi::c_int
        || c > 0 as ::core::ffi::c_int
            && (*g_chartab.ptr())[c as usize] as ::core::ffi::c_int & CT_FNAME_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_is_fname_char(mut c: ::core::ffi::c_int) -> bool {
    return vim_isfilec(c) as ::core::ffi::c_int != 0
        || c == ',' as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int
        || c == '@' as ::core::ffi::c_int
        || c == ':' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isfilec_or_wc(mut c: ::core::ffi::c_int) -> bool {
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return vim_isfilec(c) as ::core::ffi::c_int != 0
        || c == ']' as ::core::ffi::c_int
        || path_has_wildcard(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isprintc(mut c: ::core::ffi::c_int) -> bool {
    if c >= 0x100 as ::core::ffi::c_int {
        return utf_printable(c);
    }
    return c > 0 as ::core::ffi::c_int
        && (*g_chartab.ptr())[c as usize] as ::core::ffi::c_int & CT_PRINT_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn skipwhite(mut p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    while ascii_iswhite(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skipwhite_len(
    mut p: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    while len > 0 as size_t && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        p = p.offset(1);
        len = len.wrapping_sub(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn getwhitecols_curline() -> intptr_t {
    return getwhitecols(get_cursor_line_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn getwhitecols(mut p: *const ::core::ffi::c_char) -> intptr_t {
    return skipwhite(p).offset_from(p) as intptr_t;
}
#[no_mangle]
pub unsafe extern "C" fn skipdigits(mut q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while ascii_isdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skipbin(mut q: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while ascii_isbdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiphex(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while ascii_isxdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptodigit(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptobin(
    mut q: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isbdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptohex(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isxdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptowhite(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != NUL
    {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skiptowhite_esc(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != NUL
    {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skip_to_newline(
    p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return xstrchrnul(p, NL as ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn try_getdigits(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut nr: *mut intmax_t,
) -> bool {
    *__errno_location() = 0 as ::core::ffi::c_int;
    *nr = strtoimax(*pp, pp, 10 as ::core::ffi::c_int);
    if *__errno_location() == ERANGE
        && (*nr == INTMAX_MIN as intmax_t || *nr == INTMAX_MAX as intmax_t)
    {
        return false_0 != 0;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: intmax_t,
) -> intmax_t {
    let mut number: intmax_t = 0;
    let mut ok: ::core::ffi::c_int = try_getdigits(pp, &raw mut number) as ::core::ffi::c_int;
    if strict as ::core::ffi::c_int != 0 && ok == 0 {
        abort();
    }
    return if ok != 0 { number } else { def };
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_int(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    if strict {
        '_c2rust_label: {
            if number >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as intmax_t
                && number <= 2147483647 as intmax_t
            {
            } else {
                __assert_fail(
                    b"number >= INT_MIN && number <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/charset.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1134 as ::core::ffi::c_uint,
                    b"int getdigits_int(char **, _Bool, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    } else if !(number >= INT_MIN as intmax_t && number <= INT_MAX as intmax_t) {
        return def;
    }
    return number as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_long(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: ::core::ffi::c_long,
) -> ::core::ffi::c_long {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    return number as ::core::ffi::c_long;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_int32(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: int32_t,
) -> int32_t {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    if strict {
        '_c2rust_label: {
            if number >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as intmax_t
                && number <= 2147483647 as intmax_t
            {
            } else {
                __assert_fail(
                    b"number >= INT32_MIN && number <= INT32_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/charset.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1166 as ::core::ffi::c_uint,
                    b"int32_t getdigits_int32(char **, _Bool, int32_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    } else if !(number >= INT32_MIN as intmax_t && number <= INT32_MAX as intmax_t) {
        return def;
    }
    return number as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isblankline(mut lbuf: *mut ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = skipwhite(lbuf);
    return *p as ::core::ffi::c_int == NUL
        || *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_str2nr(
    start: *const ::core::ffi::c_char,
    prep: *mut ::core::ffi::c_int,
    len: *mut ::core::ffi::c_int,
    what: ::core::ffi::c_int,
    nptr: *mut varnumber_T,
    unptr: *mut uvarnumber_T,
    maxlen: ::core::ffi::c_int,
    strict: bool,
    overflow: *mut bool,
) {
    let mut ptr: *const ::core::ffi::c_char = start;
    let mut pre: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let negative: bool = *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '-' as ::core::ffi::c_int;
    let mut un: uvarnumber_T = 0 as uvarnumber_T;
    if !len.is_null() {
        *len = 0 as ::core::ffi::c_int;
    }
    if negative {
        ptr = ptr.offset(1);
    }
    '_vim_str2nr_proceed: {
        '_vim_str2nr_oct: {
            '_vim_str2nr_bin: {
                '_vim_str2nr_hex: {
                    '_vim_str2nr_dec: {
                        if what & STR2NR_FORCE as ::core::ffi::c_int != 0 {
                            match what
                                & !(STR2NR_FORCE as ::core::ffi::c_int
                                    | STR2NR_QUOTE as ::core::ffi::c_int)
                            {
                                4 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'x' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'X' as ::core::ffi::c_int)
                                        && ascii_isxdigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_hex;
                                }
                                1 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'b' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'B' as ::core::ffi::c_int)
                                        && ascii_isbdigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_bin;
                                }
                                2 | 8 | 10 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'o' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'O' as ::core::ffi::c_int)
                                        && ascii_isodigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_oct;
                                }
                                0 => {}
                                _ => {
                                    abort();
                                }
                            }
                        } else if what
                            & (STR2NR_HEX as ::core::ffi::c_int
                                | STR2NR_OCT as ::core::ffi::c_int
                                | STR2NR_OOCT as ::core::ffi::c_int
                                | STR2NR_BIN as ::core::ffi::c_int)
                            != 0
                            && (maxlen == 0 as ::core::ffi::c_int
                                || (ptr
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    .offset_from(start)
                                    as ::core::ffi::c_int)
                                    < maxlen)
                            && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '0' as ::core::ffi::c_int
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '8' as ::core::ffi::c_int
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '9' as ::core::ffi::c_int
                        {
                            pre = *ptr.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int;
                            if what & STR2NR_HEX as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'X' as ::core::ffi::c_int
                                    || pre == 'x' as ::core::ffi::c_int)
                                && ascii_isxdigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_hex;
                            } else if what & STR2NR_BIN as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'B' as ::core::ffi::c_int
                                    || pre == 'b' as ::core::ffi::c_int)
                                && ascii_isbdigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_bin;
                            } else if what & STR2NR_OOCT as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'O' as ::core::ffi::c_int
                                    || pre == 'o' as ::core::ffi::c_int)
                                && ascii_isodigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_oct;
                            } else {
                                pre = 0 as ::core::ffi::c_int;
                                if !(what & STR2NR_OCT as ::core::ffi::c_int == 0
                                    || !ascii_isodigit(
                                        *ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ))
                                {
                                    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                                    while (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr.offset(i as isize).offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && ascii_isdigit(
                                            *ptr.offset(i as isize) as ::core::ffi::c_int
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        if *ptr.offset(i as isize) as ::core::ffi::c_int
                                            > '7' as ::core::ffi::c_int
                                        {
                                            break '_vim_str2nr_dec;
                                        }
                                        i += 1;
                                    }
                                    pre = '0' as ::core::ffi::c_int;
                                    break '_vim_str2nr_oct;
                                }
                            }
                        }
                    }
                    let after_prefix_1: *const ::core::ffi::c_char = ptr;
                    while maxlen == 0 as ::core::ffi::c_int
                        || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
                    {
                        if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                            && ptr > after_prefix_1
                            && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                        {
                            ptr = ptr.offset(1);
                            if (maxlen == 0 as ::core::ffi::c_int
                                || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                                && ascii_isdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                continue;
                            }
                            ptr = ptr.offset(-1);
                        }
                        if !ascii_isdigit(*ptr as ::core::ffi::c_int) {
                            break;
                        }
                        let digit_1: uvarnumber_T = (*ptr as ::core::ffi::c_int
                            - '0' as ::core::ffi::c_int)
                            as uvarnumber_T;
                        if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(10 as uvarnumber_T)
                            || un
                                == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(10 as uvarnumber_T)
                                && (10 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                                    || digit_1
                                        <= (UVARNUMBER_MAX as uvarnumber_T)
                                            .wrapping_rem(10 as uvarnumber_T))
                        {
                            un = (10 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_1);
                        } else {
                            un = UVARNUMBER_MAX as uvarnumber_T;
                            if !overflow.is_null() {
                                *overflow = true_0 != 0;
                            }
                        }
                        ptr = ptr.offset(1);
                    }
                    break '_vim_str2nr_proceed;
                }
                let after_prefix_2: *const ::core::ffi::c_char = ptr;
                while maxlen == 0 as ::core::ffi::c_int
                    || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
                {
                    if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                        && ptr > after_prefix_2
                        && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                    {
                        ptr = ptr.offset(1);
                        if (maxlen == 0 as ::core::ffi::c_int
                            || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                            && ascii_isxdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            continue;
                        }
                        ptr = ptr.offset(-1);
                    }
                    if !ascii_isxdigit(*ptr as ::core::ffi::c_int) {
                        break;
                    }
                    let digit_2: uvarnumber_T = hex2nr(*ptr as ::core::ffi::c_int) as uvarnumber_T;
                    if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(16 as uvarnumber_T)
                        || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(16 as uvarnumber_T)
                            && (16 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                                || digit_2
                                    <= (UVARNUMBER_MAX as uvarnumber_T)
                                        .wrapping_rem(10 as uvarnumber_T))
                    {
                        un = (16 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_2);
                    } else {
                        un = UVARNUMBER_MAX as uvarnumber_T;
                        if !overflow.is_null() {
                            *overflow = true_0 != 0;
                        }
                    }
                    ptr = ptr.offset(1);
                }
                break '_vim_str2nr_proceed;
            }
            let after_prefix: *const ::core::ffi::c_char = ptr;
            while maxlen == 0 as ::core::ffi::c_int
                || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
            {
                if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                    && ptr > after_prefix
                    && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                {
                    ptr = ptr.offset(1);
                    if (maxlen == 0 as ::core::ffi::c_int
                        || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                        && (*ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                            || *ptr as ::core::ffi::c_int == '1' as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    ptr = ptr.offset(-1);
                }
                if !(*ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                    || *ptr as ::core::ffi::c_int == '1' as ::core::ffi::c_int)
                {
                    break;
                }
                let digit: uvarnumber_T =
                    (*ptr as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as uvarnumber_T;
                if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(2 as uvarnumber_T)
                    || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(2 as uvarnumber_T)
                        && (2 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                            || digit
                                <= (UVARNUMBER_MAX as uvarnumber_T)
                                    .wrapping_rem(10 as uvarnumber_T))
                {
                    un = (2 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit);
                } else {
                    un = UVARNUMBER_MAX as uvarnumber_T;
                    if !overflow.is_null() {
                        *overflow = true_0 != 0;
                    }
                }
                ptr = ptr.offset(1);
            }
            break '_vim_str2nr_proceed;
        }
        let after_prefix_0: *const ::core::ffi::c_char = ptr;
        while maxlen == 0 as ::core::ffi::c_int
            || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
        {
            if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                && ptr > after_prefix_0
                && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
            {
                ptr = ptr.offset(1);
                if (maxlen == 0 as ::core::ffi::c_int
                    || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                    && ascii_isodigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    continue;
                }
                ptr = ptr.offset(-1);
            }
            if !ascii_isodigit(*ptr as ::core::ffi::c_int) {
                break;
            }
            let digit_0: uvarnumber_T =
                (*ptr as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as uvarnumber_T;
            if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(8 as uvarnumber_T)
                || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(8 as uvarnumber_T)
                    && (8 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                        || digit_0
                            <= (UVARNUMBER_MAX as uvarnumber_T).wrapping_rem(10 as uvarnumber_T))
            {
                un = (8 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_0);
            } else {
                un = UVARNUMBER_MAX as uvarnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            }
            ptr = ptr.offset(1);
        }
    }
    if strict as ::core::ffi::c_int != 0
        && ptr.offset_from(start) != maxlen as isize
        && (*ptr as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *ptr as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *ptr as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *ptr as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        return;
    }
    if !prep.is_null() {
        *prep = pre;
    }
    if !len.is_null() {
        *len = ptr.offset_from(start) as ::core::ffi::c_int;
    }
    if !nptr.is_null() {
        if negative {
            if un > VARNUMBER_MAX as uvarnumber_T {
                *nptr = VARNUMBER_MIN as varnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            } else {
                *nptr = -(un as varnumber_T);
            }
        } else {
            if un > VARNUMBER_MAX as uvarnumber_T {
                un = VARNUMBER_MAX as uvarnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            }
            *nptr = un as varnumber_T;
        }
    }
    if !unptr.is_null() {
        *unptr = un;
    }
}
#[no_mangle]
pub unsafe extern "C" fn hex2nr(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int {
        return c - 'a' as ::core::ffi::c_int + 10 as ::core::ffi::c_int;
    }
    if c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int {
        return c - 'A' as ::core::ffi::c_int + 10 as ::core::ffi::c_int;
    }
    return c - '0' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hexhex2nr(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if !ascii_isxdigit(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        || !ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        return -1 as ::core::ffi::c_int;
    }
    return (hex2nr(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        << 4 as ::core::ffi::c_int)
        + hex2nr(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn rem_backslash(mut str: *const ::core::ffi::c_char) -> bool {
    return *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\\' as ::core::ffi::c_int
        && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
#[no_mangle]
pub unsafe extern "C" fn backslash_halve(mut p: *mut ::core::ffi::c_char) {
    while *p as ::core::ffi::c_int != 0 && !rem_backslash(p) {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int != NUL {
        let mut dst: *mut ::core::ffi::c_char = p;
        's_50: loop {
            let c2rust_fresh16 = dst;
            dst = dst.offset(1);
            *c2rust_fresh16 = *p.offset(1 as ::core::ffi::c_int as isize);
            p = p.offset(2 as ::core::ffi::c_int as isize);
            loop {
                if *p as ::core::ffi::c_int == NUL {
                    break 's_50;
                }
                if rem_backslash(p) {
                    break;
                }
                let c2rust_fresh17 = p;
                p = p.offset(1);
                let c2rust_fresh18 = dst;
                dst = dst.offset(1);
                *c2rust_fresh18 = *c2rust_fresh17;
            }
        }
        *dst = NUL as ::core::ffi::c_char;
    }
}
#[no_mangle]
pub unsafe extern "C" fn backslash_halve_save(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut res: *mut ::core::ffi::c_char =
        xmalloc(strlen(p).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut dst: *mut ::core::ffi::c_char = res;
    while *p as ::core::ffi::c_int != NUL {
        if rem_backslash(p) {
            let c2rust_fresh19 = dst;
            dst = dst.offset(1);
            *c2rust_fresh19 = *p.offset(1 as ::core::ffi::c_int as isize);
            p = p.offset(2 as ::core::ffi::c_int as isize);
        } else {
            let c2rust_fresh20 = p;
            p = p.offset(1);
            let c2rust_fresh21 = dst;
            dst = dst.offset(1);
            *c2rust_fresh21 = *c2rust_fresh20;
        }
    }
    *dst = NUL as ::core::ffi::c_char;
    return res;
}
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
