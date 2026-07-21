use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, EvalFuncData,
    ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection,
    KeyValuePair, ListLenSpecials, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MsgpackRpcRequestHandler, Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, VarLockStatus,
    VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __compar_fn_t, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictitem_T, dictvar_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, fuzmatch_str_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
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
    fn ceil(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn floor(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static p_ws: GlobalCell<::core::ffi::c_int>;
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T);
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T;
    fn callback_free(callback: *mut Callback);
    fn tv_dict_unref(d: *mut dict_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_has_key(d: *const dict_T, key: *const ::core::ffi::c_char) -> bool;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_dict_get_callback(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: ptrdiff_t,
        result: *mut Callback,
    ) -> bool;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_nonnull_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static curbuf: GlobalCell<*mut buf_T>;
    fn ctrl_x_mode_whole_line() -> bool;
    fn find_word_start(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_word_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_line_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_toupper(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_islower(a: ::core::ffi::c_int) -> bool;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_isupper(a: ::core::ffi::c_int) -> bool;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargval: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_listarg: [::core::ffi::c_char; 0];
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_13 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_13 = 3;
pub const BACKWARD: C2Rust_Unnamed_13 = -1;
pub const FORWARD: C2Rust_Unnamed_13 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FUZZY_MATCH_MAX_LEN: C2Rust_Unnamed_14 = 1024;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_15 = -2147483648;
pub type score_t = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_struct {
    pub needle_len: ::core::ffi::c_int,
    pub haystack_len: ::core::ffi::c_int,
    pub lower_needle: [::core::ffi::c_int; 1024],
    pub lower_haystack: [::core::ffi::c_int; 1024],
    pub match_bonus: [score_t; 1024],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzzyItem_T {
    pub idx: ::core::ffi::c_int,
    pub item: *mut listitem_T,
    pub score: ::core::ffi::c_int,
    pub lmatchpos: *mut list_T,
    pub pat: *mut ::core::ffi::c_char,
    pub itemstr: *mut ::core::ffi::c_char,
    pub itemstr_allocated: bool,
    pub startpos: ::core::ffi::c_int,
}
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
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
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
pub const SCORE_SCALE: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match(
    str: *mut ::core::ffi::c_char,
    pat_arg: *const ::core::ffi::c_char,
    matchseq: bool,
    outScore: *mut ::core::ffi::c_int,
    matches: *mut uint32_t,
    maxMatches: ::core::ffi::c_int,
) -> bool {
    let mut complete: bool = false_0 != 0;
    let mut numMatches: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    *outScore = 0 as ::core::ffi::c_int;
    let save_pat: *mut ::core::ffi::c_char = xstrdup(pat_arg);
    let mut pat: *mut ::core::ffi::c_char = save_pat;
    let mut p: *mut ::core::ffi::c_char = pat;
    loop {
        if matchseq {
            complete = true_0 != 0;
        } else {
            p = skipwhite(p);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            pat = p;
            while *p as ::core::ffi::c_int != NUL && !ascii_iswhite(utf_ptr2char(p)) {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            if *p as ::core::ffi::c_int == NUL {
                complete = true_0 != 0;
            }
            *p = NUL as ::core::ffi::c_char;
        }
        let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
        if has_match(pat, str) != 0 {
            let mut fzy_score: score_t =
                match_positions(pat, str, matches.offset(numMatches as isize));
            score = if fzy_score == -::core::f32::INFINITY as score_t {
                INT_MIN + 1 as ::core::ffi::c_int
            } else if fzy_score == ::core::f32::INFINITY as score_t {
                INT_MAX
            } else if fzy_score < 0 as ::core::ffi::c_int as score_t {
                ceil(
                    fzy_score as ::core::ffi::c_double * SCORE_SCALE as ::core::ffi::c_double
                        - 0.5f64,
                ) as ::core::ffi::c_int
            } else {
                floor(
                    fzy_score as ::core::ffi::c_double * SCORE_SCALE as ::core::ffi::c_double
                        + 0.5f64,
                ) as ::core::ffi::c_int
            };
        }
        if score == FUZZY_SCORE_NONE as ::core::ffi::c_int {
            numMatches = 0 as ::core::ffi::c_int;
            *outScore = FUZZY_SCORE_NONE as ::core::ffi::c_int;
            break;
        } else {
            if score > 0 as ::core::ffi::c_int && *outScore > INT_MAX - score {
                *outScore = INT_MAX;
            } else if score < 0 as ::core::ffi::c_int
                && *outScore < INT_MIN + 1 as ::core::ffi::c_int - score
            {
                *outScore = INT_MIN + 1 as ::core::ffi::c_int;
            } else {
                *outScore += score;
            }
            numMatches += mb_charlen(pat);
            if complete as ::core::ffi::c_int != 0 || numMatches >= maxMatches {
                break;
            }
            p = p.offset(1);
        }
    }
    xfree(save_pat as *mut ::core::ffi::c_void);
    return numMatches != 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_match_item_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *const fuzzyItem_T)).score;
    if v1 == v2 {
        let pat: *const ::core::ffi::c_char = (*(s1 as *const fuzzyItem_T)).pat;
        let patlen: size_t = strlen(pat);
        let mut startpos: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).startpos;
        let exact_match1: bool = startpos >= 0 as ::core::ffi::c_int
            && strncmp(
                pat,
                (*(s1 as *mut fuzzyItem_T))
                    .itemstr
                    .offset(startpos as isize),
                patlen,
            ) == 0 as ::core::ffi::c_int;
        startpos = (*(s2 as *const fuzzyItem_T)).startpos;
        let exact_match2: bool = startpos >= 0 as ::core::ffi::c_int
            && strncmp(
                pat,
                (*(s2 as *mut fuzzyItem_T))
                    .itemstr
                    .offset(startpos as isize),
                patlen,
            ) == 0 as ::core::ffi::c_int;
        if exact_match1 as ::core::ffi::c_int == exact_match2 as ::core::ffi::c_int {
            let idx1: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).idx;
            let idx2: ::core::ffi::c_int = (*(s2 as *const fuzzyItem_T)).idx;
            return if idx1 == idx2 {
                0 as ::core::ffi::c_int
            } else if idx1 > idx2 {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else if exact_match2 {
            return 1 as ::core::ffi::c_int;
        }
        return -1 as ::core::ffi::c_int;
    } else {
        return if v1 > v2 {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    };
}
unsafe extern "C" fn fuzzy_match_in_list(
    l: *mut list_T,
    str: *mut ::core::ffi::c_char,
    matchseq: bool,
    key: *const ::core::ffi::c_char,
    item_cb: *mut Callback,
    retmatchpos: bool,
    fmatchlist: *mut list_T,
    max_matches: ::core::ffi::c_int,
) {
    let mut len: ::core::ffi::c_int = tv_list_len(l);
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    if max_matches > 0 as ::core::ffi::c_int && len > max_matches {
        len = max_matches;
    }
    let items: *mut fuzzyItem_T =
        xcalloc(len as size_t, ::core::mem::size_of::<fuzzyItem_T>()) as *mut fuzzyItem_T;
    let mut match_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matches: [uint32_t; 1024] = [0; 1024];
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if max_matches > 0 as ::core::ffi::c_int && match_count >= max_matches {
                break;
            }
            let mut itemstr: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut itemstr_allocate: bool = false;
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            rettv.v_type = VAR_UNKNOWN;
            let tv: *const typval_T = &raw mut (*li).li_tv;
            if (*tv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                itemstr = (*tv).vval.v_string;
            } else if (*tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && (!key.is_null()
                    || (*item_cb).type_0 as ::core::ffi::c_uint
                        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                if !key.is_null() {
                    itemstr = tv_dict_get_string((*tv).vval.v_dict, key, false);
                } else {
                    let mut argv: [typval_T; 2] = [typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    }; 2];
                    (*(*tv).vval.v_dict).dv_refcount += 1;
                    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
                    argv[0 as ::core::ffi::c_int as usize].vval.v_dict = (*tv).vval.v_dict;
                    argv[1 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
                    if callback_call(
                        item_cb,
                        1 as ::core::ffi::c_int,
                        &raw mut argv as *mut typval_T,
                        &raw mut rettv,
                    ) {
                        if rettv.v_type as ::core::ffi::c_uint
                            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            itemstr = rettv.vval.v_string;
                            itemstr_allocate = true;
                        }
                    }
                    tv_dict_unref((*tv).vval.v_dict);
                }
            }
            let mut score: ::core::ffi::c_int = 0;
            if !itemstr.is_null()
                && fuzzy_match(
                    itemstr,
                    str,
                    matchseq,
                    &raw mut score,
                    &raw mut matches as *mut uint32_t,
                    FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                let mut itemstr_copy: *mut ::core::ffi::c_char =
                    if itemstr_allocate as ::core::ffi::c_int != 0 {
                        xstrdup(itemstr)
                    } else {
                        itemstr
                    };
                let mut match_positions_0: *mut list_T = ::core::ptr::null_mut::<list_T>();
                if retmatchpos {
                    match_positions_0 =
                        tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
                    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut p: *const ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && j < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int
                    {
                        if !ascii_iswhite(utf_ptr2char(p)) || matchseq as ::core::ffi::c_int != 0 {
                            tv_list_append_number(
                                match_positions_0,
                                matches[j as usize] as varnumber_T,
                            );
                            j += 1;
                        }
                        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                    }
                }
                (*items.offset(match_count as isize)).idx = match_count;
                (*items.offset(match_count as isize)).item = li;
                (*items.offset(match_count as isize)).score = score;
                (*items.offset(match_count as isize)).pat = str;
                (*items.offset(match_count as isize)).startpos =
                    matches[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                (*items.offset(match_count as isize)).itemstr = itemstr_copy;
                (*items.offset(match_count as isize)).itemstr_allocated = itemstr_allocate;
                (*items.offset(match_count as isize)).lmatchpos = match_positions_0;
                match_count += 1;
            }
            tv_clear(&raw mut rettv);
            li = (*li).li_next;
        }
    }
    if match_count > 0 as ::core::ffi::c_int {
        qsort(
            items as *mut ::core::ffi::c_void,
            match_count as size_t,
            ::core::mem::size_of::<fuzzyItem_T>(),
            Some(
                fuzzy_match_item_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut retlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
        if retmatchpos {
            let li_0: *const listitem_T = tv_list_find(fmatchlist, 0 as ::core::ffi::c_int);
            '_c2rust_label: {
                if !li_0.is_null() && !(*li_0).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/fuzzy.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        293 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_0).li_tv.vval.v_list;
        } else {
            retlist = fmatchlist;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < match_count {
            tv_list_append_tv(retlist, &raw mut (*(*items.offset(i as isize)).item).li_tv);
            i += 1;
        }
        if retmatchpos {
            let mut li_1: *const listitem_T = tv_list_find(fmatchlist, -2 as ::core::ffi::c_int);
            '_c2rust_label_0: {
                if !li_1.is_null() && !(*li_1).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/fuzzy.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_1).li_tv.vval.v_list;
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < match_count {
                '_c2rust_label_1: {
                    if !(*items.offset(i_0 as isize)).lmatchpos.is_null() {
                    } else {
                        __assert_fail(
                            b"items[i].lmatchpos != NULL\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/fuzzy.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            311 as ::core::ffi::c_uint,
                            b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                tv_list_append_list(retlist, (*items.offset(i_0 as isize)).lmatchpos);
                (*items.offset(i_0 as isize)).lmatchpos = ::core::ptr::null_mut::<list_T>();
                i_0 += 1;
            }
            li_1 = tv_list_find(fmatchlist, -1 as ::core::ffi::c_int);
            '_c2rust_label_2: {
                if !li_1.is_null() && !(*li_1).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/fuzzy.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        318 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_1).li_tv.vval.v_list;
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < match_count {
                tv_list_append_number(retlist, (*items.offset(i_1 as isize)).score as varnumber_T);
                i_1 += 1;
            }
        }
    }
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < match_count {
        if (*items.offset(i_2 as isize)).itemstr_allocated {
            xfree((*items.offset(i_2 as isize)).itemstr as *mut ::core::ffi::c_void);
        }
        '_c2rust_label_3: {
            if (*items.offset(i_2 as isize)).lmatchpos.is_null() {
            } else {
                __assert_fail(
                    b"items[i].lmatchpos == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/fuzzy.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    330 as ::core::ffi::c_uint,
                    b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        i_2 += 1;
    }
    xfree(items as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn do_fuzzymatch(
    argvars: *const typval_T,
    rettv: *mut typval_T,
    retmatchpos: bool,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list
            .is_null()
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            if retmatchpos as ::core::ffi::c_int != 0 {
                b"matchfuzzypos()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"matchfuzzy()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        );
        return;
    }
    let mut cb: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut matchseq: bool = false_0 != 0;
    let mut max_matches: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let d: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        let mut di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
        di = tv_dict_find(
            d,
            b"key\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*di).di_tv.vval.v_string.is_null()
                || *(*di).di_tv.vval.v_string as ::core::ffi::c_int == NUL
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"key\0".as_ptr() as *const ::core::ffi::c_char,
                    tv_get_string(&raw const (*di).di_tv),
                );
                return;
            }
            key = tv_get_string(&raw const (*di).di_tv);
        } else if !tv_dict_get_callback(
            d,
            b"text_cb\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
            &raw mut cb,
        ) {
            semsg(
                gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                b"text_cb\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        di = tv_dict_find(
            d,
            b"limit\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(
                    gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                    b"limit\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
            max_matches = tv_get_number_chk(&raw const (*di).di_tv, ::core::ptr::null_mut::<bool>())
                as ::core::ffi::c_int;
        }
        if tv_dict_has_key(d, b"matchseq\0".as_ptr() as *const ::core::ffi::c_char) {
            matchseq = true_0 != 0;
        }
    }
    tv_list_alloc_ret(
        rettv,
        (if retmatchpos as ::core::ffi::c_int != 0 {
            3 as ::core::ffi::c_int
        } else {
            kListLenUnknown as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if retmatchpos {
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
    }
    fuzzy_match_in_list(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list,
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        matchseq,
        key,
        &raw mut cb,
        retmatchpos,
        (*rettv).vval.v_list,
        max_matches,
    );
    callback_free(&raw mut cb);
}
#[no_mangle]
pub unsafe extern "C" fn f_matchfuzzy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    do_fuzzymatch(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_matchfuzzypos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    do_fuzzymatch(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn fuzzy_match_str_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).score;
    let idx1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).idx;
    let idx2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).idx;
    if v1 == v2 {
        return if idx1 == idx2 {
            0 as ::core::ffi::c_int
        } else if idx1 > idx2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        return if v1 > v2 {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    };
}
unsafe extern "C" fn fuzzy_match_str_sort(fm: *mut fuzmatch_str_T, sz: ::core::ffi::c_int) {
    qsort(
        fm as *mut ::core::ffi::c_void,
        sz as size_t,
        ::core::mem::size_of::<fuzmatch_str_T>(),
        Some(
            fuzzy_match_str_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
}
unsafe extern "C" fn fuzzy_match_func_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).score;
    let idx1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).idx;
    let idx2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).idx;
    let str1: *const ::core::ffi::c_char = (*(s1 as *mut fuzmatch_str_T)).str;
    let str2: *const ::core::ffi::c_char = (*(s2 as *mut fuzmatch_str_T)).str;
    if *str1 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        && *str2 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    if *str1 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        && *str2 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if v1 == v2 {
        return if idx1 == idx2 {
            0 as ::core::ffi::c_int
        } else if idx1 > idx2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return if v1 > v2 {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn fuzzy_match_func_sort(fm: *mut fuzmatch_str_T, sz: ::core::ffi::c_int) {
    qsort(
        fm as *mut ::core::ffi::c_void,
        sz as size_t,
        ::core::mem::size_of::<fuzmatch_str_T>(),
        Some(
            fuzzy_match_func_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str(
    str: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if str.is_null() || pat.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
    let mut matchpos: [uint32_t; 1024] = [0; 1024];
    fuzzy_match(
        str,
        pat,
        true_0 != 0,
        &raw mut score,
        &raw mut matchpos as *mut uint32_t,
        ::core::mem::size_of::<[uint32_t; 1024]>()
            .wrapping_div(::core::mem::size_of::<uint32_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint32_t; 1024]>()
                    .wrapping_rem(::core::mem::size_of::<uint32_t>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int,
    );
    return score;
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str_with_pos(
    str: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
) -> *mut garray_T {
    if str.is_null() || pat.is_null() {
        return ::core::ptr::null_mut::<garray_T>();
    }
    let mut match_positions_0: *mut garray_T =
        xmalloc(::core::mem::size_of::<garray_T>()) as *mut garray_T;
    ga_init(
        match_positions_0,
        ::core::mem::size_of::<uint32_t>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
    let mut matches: [uint32_t; 1024] = [0; 1024];
    if !fuzzy_match(
        str,
        pat,
        false_0 != 0,
        &raw mut score,
        &raw mut matches as *mut uint32_t,
        FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int,
    ) || score == FUZZY_SCORE_NONE as ::core::ffi::c_int
    {
        ga_clear(match_positions_0);
        xfree(match_positions_0 as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<garray_T>();
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = pat;
    while *p as ::core::ffi::c_int != NUL {
        if !ascii_iswhite(utf_ptr2char(p)) {
            ga_grow(match_positions_0, 1 as ::core::ffi::c_int);
            *((*match_positions_0).ga_data as *mut uint32_t)
                .offset((*match_positions_0).ga_len as isize) = matches[j as usize];
            (*match_positions_0).ga_len += 1;
            j += 1;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return match_positions_0;
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str_in_line(
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut len: *mut ::core::ffi::c_int,
    mut current_pos: *mut pos_T,
    mut score: *mut ::core::ffi::c_int,
) -> bool {
    let mut str: *mut ::core::ffi::c_char = *ptr;
    let mut strBegin: *mut ::core::ffi::c_char = str;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found: bool = false_0 != 0;
    if str.is_null() || pat.is_null() {
        return found;
    }
    let mut line_end: *mut ::core::ffi::c_char = find_line_end(str);
    while str < line_end {
        start = find_word_start(str);
        if *start as ::core::ffi::c_int == NUL {
            break;
        }
        end = find_word_end(start);
        let mut save_end: ::core::ffi::c_char = *end;
        *end = NUL as ::core::ffi::c_char;
        *score = fuzzy_match_str(start, pat);
        *end = save_end;
        if *score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
            *len = end.offset_from(start) as ::core::ffi::c_int;
            found = true_0 != 0;
            *ptr = start;
            if !current_pos.is_null() {
                (*current_pos).col += end.offset_from(strBegin) as ::core::ffi::c_int;
            }
            break;
        } else {
            str = end;
            while *str as ::core::ffi::c_int != NUL && !vim_iswordp(str) {
                str = str.offset(utfc_ptr2len(str) as isize);
            }
        }
    }
    if !found {
        *ptr = line_end;
    }
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn search_for_fuzzy_match(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut pattern: *mut ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut start_pos: *mut pos_T,
    mut len: *mut ::core::ffi::c_int,
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut score: *mut ::core::ffi::c_int,
) -> bool {
    let mut current_pos: pos_T = *pos;
    let mut circly_end: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_new_match: bool = false_0 != 0;
    let mut looped_around: bool = false_0 != 0;
    let mut whole_line: bool = ctrl_x_mode_whole_line();
    if buf == curbuf.get() {
        circly_end = *start_pos;
    } else {
        circly_end.lnum = (*buf).b_ml.ml_line_count;
        circly_end.col = 0 as ::core::ffi::c_int as colnr_T;
        circly_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if whole_line as ::core::ffi::c_int != 0 && (*start_pos).lnum != (*pos).lnum {
        current_pos.lnum = (current_pos.lnum as ::core::ffi::c_int + dir) as linenr_T;
    }
    while !(looped_around as ::core::ffi::c_int != 0
        && (if whole_line as ::core::ffi::c_int != 0 {
            (current_pos.lnum == circly_end.lnum) as ::core::ffi::c_int
        } else {
            equalpos(current_pos, circly_end) as ::core::ffi::c_int
        }) != 0)
    {
        if current_pos.lnum >= 1 as linenr_T && current_pos.lnum <= (*buf).b_ml.ml_line_count {
            *ptr = ml_get_buf(buf, current_pos.lnum);
            if !whole_line {
                *ptr = (*ptr).offset(current_pos.col as isize);
            }
            if !(*ptr).is_null() && **ptr as ::core::ffi::c_int != NUL {
                if !whole_line {
                    found_new_match =
                        fuzzy_match_str_in_line(ptr, pattern, len, &raw mut current_pos, score);
                    if found_new_match {
                        *pos = current_pos;
                        break;
                    } else if looped_around as ::core::ffi::c_int != 0
                        && current_pos.lnum == circly_end.lnum
                    {
                        break;
                    }
                } else if fuzzy_match_str(*ptr, pattern) != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                    found_new_match = true_0 != 0;
                    *pos = current_pos;
                    *len = ml_get_buf_len(buf, current_pos.lnum) as ::core::ffi::c_int;
                    break;
                }
            }
        }
        if dir == FORWARD as ::core::ffi::c_int {
            current_pos.lnum += 1;
            if current_pos.lnum > (*buf).b_ml.ml_line_count {
                if p_ws.get() == 0 {
                    break;
                }
                current_pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                looped_around = true_0 != 0;
            }
        } else {
            current_pos.lnum -= 1;
            if current_pos.lnum < 1 as linenr_T {
                if p_ws.get() == 0 {
                    break;
                }
                current_pos.lnum = (*buf).b_ml.ml_line_count;
                looped_around = true_0 != 0;
            }
        }
        current_pos.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return found_new_match;
}
#[no_mangle]
pub unsafe extern "C" fn fuzmatch_str_free(
    fuzmatch: *mut fuzmatch_str_T,
    mut count: ::core::ffi::c_int,
) {
    if fuzmatch.is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        xfree((*fuzmatch.offset(count as isize)).str as *mut ::core::ffi::c_void);
        i += 1;
    }
    xfree(fuzmatch as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn fuzzymatches_to_strmatches(
    fuzmatch: *mut fuzmatch_str_T,
    matches: *mut *mut *mut ::core::ffi::c_char,
    count: ::core::ffi::c_int,
    funcsort: bool,
) {
    if count > 0 as ::core::ffi::c_int {
        *matches = xmalloc(
            (count as size_t).wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        ) as *mut *mut ::core::ffi::c_char;
        if funcsort {
            fuzzy_match_func_sort(fuzmatch, count);
        } else {
            fuzzy_match_str_sort(fuzmatch, count);
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            *(*matches).offset(i as isize) = (*fuzmatch.offset(i as isize)).str;
            i += 1;
        }
    }
    xfree(fuzmatch as *mut ::core::ffi::c_void);
}
pub const SCORE_GAP_LEADING: ::core::ffi::c_double = -0.005f64;
pub const SCORE_GAP_TRAILING: ::core::ffi::c_double = -0.005f64;
pub const SCORE_GAP_INNER: ::core::ffi::c_double = -0.01f64;
pub const SCORE_MATCH_CONSECUTIVE: ::core::ffi::c_double = 1.0f64;
pub const SCORE_MATCH_SLASH: ::core::ffi::c_double = 0.9f64;
pub const SCORE_MATCH_WORD: ::core::ffi::c_double = 0.8f64;
pub const SCORE_MATCH_CAPITAL: ::core::ffi::c_double = 0.7f64;
pub const SCORE_MATCH_DOT: ::core::ffi::c_double = 0.6f64;
unsafe extern "C" fn has_match(
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if needle.is_null() || haystack.is_null() || *needle == 0 {
        return FAIL;
    }
    let mut n_ptr: *const ::core::ffi::c_char = needle;
    let mut h_ptr: *const ::core::ffi::c_char = haystack;
    while *n_ptr != 0 {
        let n_char: ::core::ffi::c_int = utf_ptr2char(n_ptr);
        let mut found: bool = false_0 != 0;
        while *h_ptr != 0 {
            let h_char: ::core::ffi::c_int = utf_ptr2char(h_ptr);
            if n_char == h_char || mb_toupper(n_char) == h_char {
                found = true_0 != 0;
                h_ptr = h_ptr.offset(utfc_ptr2len(h_ptr) as isize);
                break;
            } else {
                h_ptr = h_ptr.offset(utfc_ptr2len(h_ptr) as isize);
            }
        }
        if !found {
            return FAIL;
        }
        n_ptr = n_ptr.offset(utfc_ptr2len(n_ptr) as isize);
    }
    return OK;
}
unsafe extern "C" fn compute_bonus_codepoint(
    mut last_c: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> score_t {
    if c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || vim_iswordc(c) as ::core::ffi::c_int != 0
    {
        if last_c == '/' as ::core::ffi::c_int {
            return SCORE_MATCH_SLASH;
        }
        if last_c == '-' as ::core::ffi::c_int
            || last_c == '_' as ::core::ffi::c_int
            || last_c == ' ' as ::core::ffi::c_int
        {
            return SCORE_MATCH_WORD;
        }
        if last_c == '.' as ::core::ffi::c_int {
            return SCORE_MATCH_DOT;
        }
        if mb_isupper(c) as ::core::ffi::c_int != 0 && mb_islower(last_c) as ::core::ffi::c_int != 0
        {
            return SCORE_MATCH_CAPITAL;
        }
    }
    return 0 as ::core::ffi::c_int as score_t;
}
unsafe extern "C" fn setup_match_struct(
    match_0: *mut match_struct,
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = needle;
    while *p as ::core::ffi::c_int != NUL && i < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int {
        let c: ::core::ffi::c_int = utf_ptr2char(p);
        let c2rust_fresh1 = i;
        i = i + 1;
        (*match_0).lower_needle[c2rust_fresh1 as usize] = mb_tolower(c);
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    (*match_0).needle_len = i;
    i = 0 as ::core::ffi::c_int;
    p = haystack;
    let mut prev_c: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL && i < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int {
        let c_0: ::core::ffi::c_int = utf_ptr2char(p);
        (*match_0).lower_haystack[i as usize] = mb_tolower(c_0);
        (*match_0).match_bonus[i as usize] = compute_bonus_codepoint(prev_c, c_0);
        prev_c = c_0;
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        i += 1;
    }
    (*match_0).haystack_len = i;
}
#[inline]
unsafe extern "C" fn match_row(
    mut match_0: *const match_struct,
    mut row: ::core::ffi::c_int,
    mut curr_D: *mut score_t,
    mut curr_M: *mut score_t,
    mut last_D: *const score_t,
    mut last_M: *const score_t,
) {
    let mut n: ::core::ffi::c_int = (*match_0).needle_len;
    let mut m: ::core::ffi::c_int = (*match_0).haystack_len;
    let mut i: ::core::ffi::c_int = row;
    let mut lower_needle: *const ::core::ffi::c_int =
        &raw const (*match_0).lower_needle as *const ::core::ffi::c_int;
    let mut lower_haystack: *const ::core::ffi::c_int =
        &raw const (*match_0).lower_haystack as *const ::core::ffi::c_int;
    let mut match_bonus: *const score_t = &raw const (*match_0).match_bonus as *const score_t;
    let mut prev_score: score_t = -::core::f32::INFINITY as score_t;
    let mut gap_score: score_t = if i == n - 1 as ::core::ffi::c_int {
        SCORE_GAP_TRAILING
    } else {
        SCORE_GAP_INNER
    };
    let mut prev_M: score_t = -::core::f32::INFINITY as score_t;
    let mut prev_D: score_t = -::core::f32::INFINITY as score_t;
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < m {
        if *lower_needle.offset(i as isize) == *lower_haystack.offset(j as isize) {
            let mut score: score_t = -::core::f32::INFINITY as score_t;
            if i == 0 {
                score = j as score_t * SCORE_GAP_LEADING + *match_bonus.offset(j as isize);
            } else if j != 0 {
                score = (if prev_M + *match_bonus.offset(j as isize)
                    > prev_D as ::core::ffi::c_double + 1.0f64
                {
                    prev_M as ::core::ffi::c_double
                        + *match_bonus.offset(j as isize) as ::core::ffi::c_double
                } else {
                    prev_D as ::core::ffi::c_double + 1.0f64
                }) as score_t;
            }
            prev_D = *last_D.offset(j as isize);
            prev_M = *last_M.offset(j as isize);
            *curr_D.offset(j as isize) = score;
            prev_score = if score > prev_score + gap_score {
                score
            } else {
                prev_score + gap_score
            };
            *curr_M.offset(j as isize) = prev_score;
        } else {
            prev_D = *last_D.offset(j as isize);
            prev_M = *last_M.offset(j as isize);
            *curr_D.offset(j as isize) = -::core::f32::INFINITY as score_t;
            prev_score = prev_score + gap_score;
            *curr_M.offset(j as isize) = prev_score;
        }
        j += 1;
    }
}
unsafe extern "C" fn match_positions(
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
    positions: *mut uint32_t,
) -> score_t {
    if needle.is_null() || haystack.is_null() || *needle == 0 {
        return -::core::f32::INFINITY as score_t;
    }
    let mut match_0: match_struct = match_struct {
        needle_len: 0,
        haystack_len: 0,
        lower_needle: [0; 1024],
        lower_haystack: [0; 1024],
        match_bonus: [0.; 1024],
    };
    setup_match_struct(&raw mut match_0, needle, haystack);
    let mut n: ::core::ffi::c_int = match_0.needle_len;
    let mut m: ::core::ffi::c_int = match_0.haystack_len;
    if m > FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int || n > m {
        return -::core::f32::INFINITY as score_t;
    } else if n == m {
        if !positions.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < n {
                *positions.offset(i as isize) = i as uint32_t;
                i += 1;
            }
        }
        return ::core::f32::INFINITY as score_t;
    }
    if n as size_t
        > (SIZE_MAX as usize)
            .wrapping_div(::core::mem::size_of::<score_t>())
            .wrapping_div(FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as usize)
            .wrapping_div(2 as usize)
    {
        return -::core::f32::INFINITY as score_t;
    }
    let mut block: *mut score_t = xmalloc(
        ::core::mem::size_of::<score_t>()
            .wrapping_mul(FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t)
            .wrapping_mul(n as size_t)
            .wrapping_mul(2 as size_t),
    ) as *mut score_t;
    let mut D: *mut [score_t; 1024] = block as *mut [score_t; 1024];
    let mut M: *mut [score_t; 1024] = block.offset(
        (FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t).wrapping_mul(n as size_t) as isize,
    ) as *mut [score_t; 1024];
    match_row(
        &raw mut match_0,
        0 as ::core::ffi::c_int,
        &raw mut *D.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *M.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *D.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *M.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
    );
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < n {
        match_row(
            &raw mut match_0,
            i_0,
            &raw mut *D.offset(i_0 as isize) as *mut score_t,
            &raw mut *M.offset(i_0 as isize) as *mut score_t,
            &raw mut *D.offset((i_0 - 1 as ::core::ffi::c_int) as isize) as *mut score_t,
            &raw mut *M.offset((i_0 - 1 as ::core::ffi::c_int) as isize) as *mut score_t,
        );
        i_0 += 1;
    }
    if !positions.is_null() {
        let mut match_required: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_1: ::core::ffi::c_int = n - 1 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = m - 1 as ::core::ffi::c_int;
        while i_1 >= 0 as ::core::ffi::c_int {
            while j >= 0 as ::core::ffi::c_int {
                if (*D.offset(i_1 as isize))[j as usize] != -::core::f32::INFINITY as score_t
                    && (match_required != 0
                        || (*D.offset(i_1 as isize))[j as usize]
                            == (*M.offset(i_1 as isize))[j as usize])
                {
                    match_required = (i_1 != 0
                        && j != 0
                        && (*M.offset(i_1 as isize))[j as usize]
                            == (*D.offset((i_1 - 1 as ::core::ffi::c_int) as isize))
                                [(j - 1 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_double
                                + SCORE_MATCH_CONSECUTIVE)
                        as ::core::ffi::c_int;
                    let c2rust_fresh0 = j;
                    j = j - 1;
                    *positions.offset(i_1 as isize) = c2rust_fresh0 as uint32_t;
                    break;
                } else {
                    j -= 1;
                }
            }
            i_1 -= 1;
        }
    }
    let mut result: score_t =
        (*M.offset((n - 1 as ::core::ffi::c_int) as isize))[(m - 1 as ::core::ffi::c_int) as usize];
    xfree(block as *mut ::core::ffi::c_void);
    return result;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
