use crate::src::nvim::api::extmark::{describe_ns, nvim_create_namespace};
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::buffer::{buflist_findname_exp, buflist_findnr};
use crate::src::nvim::charset::{
    backslash_halve, getdigits_int, skiptowhite, skiptowhite_esc, skipwhite, vim_isprintc,
};
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::decoration::{
    decor_find_sign, decor_item, decor_item_count, decor_put_sh, sign_item_cmp,
};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_buf_later};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::funcs::get_buf_arg;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg, tv_check_for_string_arg,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_number_def, tv_dict_get_string, tv_get_lnum, tv_get_number_chk,
    tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_number,
};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::extmark::{extmark_del, extmark_del_id, extmark_set};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_get;
use crate::src::nvim::highlight_group::{HLF_D, get_highlight_name_ext, syn_check_group};
use crate::src::nvim::main::{
    curtab, curwin, e_argreq, e_dictreq, e_invalid_buffer_name_str, e_invarg, e_invarg2, e_listreq,
    e_trailing_arg, firstbuf, firstwin, got_int, namespace_ids,
};
use crate::src::nvim::map::{
    map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, mh_get_String, mh_get_cstr_t,
};
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_SIGNHL, MT_FLAG_DECOR_SIGNTEXT, mt_decor, mt_decor_sign, mt_end,
};
use crate::src::nvim::marktree::{
    marktree_itr_current, marktree_itr_get, marktree_itr_get_overlap, marktree_itr_next,
    marktree_itr_step_overlap, marktree_lookup_ns,
};
use crate::src::nvim::mbyte::{utf_ptr2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::src::nvim::memory::{xcalloc, xfree, xmallocz, xrealloc, xstrdup};
use crate::src::nvim::message::{
    emsg, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, semsg, smsg,
};
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, gettext, memcpy, memmove, qsort, snprintf, strcmp, strlen, strncmp,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    DecorExt, DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority,
    DecorSignHighlight, DecorVirtText, Error, EvalFuncData, Integer, ListLenSpecials, MTKey,
    MTNode, MTPair, MTPos, Map_String_int, Map_cstr_t_ptr_t, MapHash, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_16, MetaIndex, NS, Set_cstr_t, SignItem, String_0, VarType,
    buf_T, colnr_T, cstr_t, dict_T, dictitem_T, exarg_T, expand_T, int32_t, int64_t, linenr_T,
    list_T, listitem_T, ptr_t, ptrdiff_t, schar_T, sign_T, size_t, typval_T, uint16_t, uint32_t,
    varnumber_T, win_T,
};
use crate::src::nvim::window::buf_jump_open_win;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kSHIsSign: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_SIGN: C2Rust_Unnamed_17 = 34;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_17 = 13;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_17 = 9;
pub const EXPAND_FILES: C2Rust_Unnamed_17 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SIGN_DEF_PRIO: C2Rust_Unnamed_18 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const BL_WHITE: C2Rust_Unnamed_22 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_24 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_25 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut cstr_t,
}
pub const EXP_SIGN_GROUPS: C2Rust_Unnamed_27 = 6;
pub const EXP_SIGN_NAMES: C2Rust_Unnamed_27 = 5;
pub const EXP_UNPLACE: C2Rust_Unnamed_27 = 4;
pub const EXP_LIST: C2Rust_Unnamed_27 = 3;
pub const EXP_PLACE: C2Rust_Unnamed_27 = 2;
pub const EXP_DEFINE: C2Rust_Unnamed_27 = 1;
pub const EXP_SUBCMD: C2Rust_Unnamed_27 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"int sign_row_cmp(const void *, const void *)\0",
    )
};
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_get_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
static sign_map: GlobalCell<Map_cstr_t_ptr_t> = GlobalCell::new(MAP_INIT);
static sign_ns: GlobalCell<C2Rust_Unnamed_24> = GlobalCell::new(C2Rust_Unnamed_24 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Integer>(),
});
static cmds: GlobalCell<[*mut ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"define\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"undefine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"place\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"unplace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
pub const SIGNCMD_DEFINE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIGNCMD_UNDEFINE: ::core::ffi::c_int = 1;
pub const SIGNCMD_LIST: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGNCMD_PLACE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGNCMD_UNPLACE: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SIGNCMD_JUMP: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const SIGNCMD_LAST: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
unsafe extern "C" fn group_get_ns(mut group: *const ::core::ffi::c_char) -> int64_t {
    if group.is_null() {
        return 0 as int64_t;
    } else if strcmp(group, b"*\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return UINT32_MAX as int64_t;
    }
    let mut ns: ::core::ffi::c_int = map_get_String_int(namespace_ids.ptr(), cstr_as_string(group));
    return (if ns != 0 {
        ns
    } else {
        -1 as ::core::ffi::c_int
    }) as int64_t;
}
unsafe extern "C" fn sign_get_name(mut sh: *mut DecorSignHighlight) -> *const ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = (*sh).sign_name;
    return if name.is_null() {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    } else if set_has_cstr_t(&raw mut (*sign_map.ptr()).set, name as cstr_t) as ::core::ffi::c_int
        != 0
    {
        name as *const ::core::ffi::c_char
    } else {
        b"[Deleted]\0".as_ptr() as *const ::core::ffi::c_char
    };
}
unsafe extern "C" fn buf_set_sign(
    mut buf: *mut buf_T,
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut lnum: linenr_T,
    mut sp: *mut sign_T,
) {
    if !group.is_null() && map_get_String_int(namespace_ids.ptr(), cstr_as_string(group)) == 0 {
        if (*sign_ns.ptr()).size == (*sign_ns.ptr()).capacity {
            (*sign_ns.ptr()).capacity = if (*sign_ns.ptr()).capacity != 0 {
                (*sign_ns.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*sign_ns.ptr()).items = xrealloc(
                (*sign_ns.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Integer>().wrapping_mul((*sign_ns.ptr()).capacity),
            ) as *mut Integer;
        } else {
        };
        let c2rust_fresh3 = (*sign_ns.ptr()).size;
        (*sign_ns.ptr()).size = (*sign_ns.ptr()).size.wrapping_add(1);
        *(*sign_ns.ptr()).items.offset(c2rust_fresh3 as isize) =
            nvim_create_namespace(cstr_as_string(group));
    }
    let mut ns: uint32_t = if !group.is_null() {
        nvim_create_namespace(cstr_as_string(group)) as uint32_t
    } else {
        0 as uint32_t
    };
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    sign.flags = (sign.flags as ::core::ffi::c_int | kSHIsSign as ::core::ffi::c_int) as uint16_t;
    memcpy(
        &raw mut sign.text as *mut schar_T as *mut ::core::ffi::c_void,
        &raw mut (*sp).sn_text as *mut schar_T as *const ::core::ffi::c_void,
        (SIGN_WIDTH as ::core::ffi::c_int as size_t)
            .wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    sign.sign_name = xstrdup((*sp).sn_name);
    sign.hl_id = (*sp).sn_text_hl;
    sign.line_hl_id = (*sp).sn_line_hl;
    sign.number_hl_id = (*sp).sn_num_hl;
    sign.cursorline_hl_id = (*sp).sn_cul_hl;
    sign.priority = prio as DecorPriority;
    let mut has_hl: bool = (*sp).sn_line_hl != 0 || (*sp).sn_num_hl != 0 || (*sp).sn_cul_hl != 0;
    let mut decor_flags: uint16_t = ((if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        MT_FLAG_DECOR_SIGNTEXT
    } else {
        0 as ::core::ffi::c_int
    }) | (if has_hl as ::core::ffi::c_int != 0 {
        MT_FLAG_DECOR_SIGNHL
    } else {
        0 as ::core::ffi::c_int
    })) as uint16_t;
    let mut decor: DecorInline = DecorInline {
        ext: true_0 != 0,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: decor_put_sh(sign),
                vt: ::core::ptr::null_mut::<DecorVirtText>(),
            },
        },
    };
    extmark_set(
        buf,
        ns,
        id,
        (if (*buf).b_ml.ml_line_count < lnum {
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int
        } else {
            lnum as ::core::ffi::c_int
        }) - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        -1 as ::core::ffi::c_int,
        -1 as colnr_T,
        decor,
        decor_flags,
        true_0 != 0,
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
}
unsafe extern "C" fn buf_mod_sign(
    mut buf: *mut buf_T,
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut sp: *mut sign_T,
) -> linenr_T {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t || !group.is_null() && ns == 0 as int64_t {
        return 0 as linenr_T;
    }
    let mut mark: MTKey = marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns as uint32_t,
        *id,
        false_0 != 0,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    );
    if mark.pos.row >= 0 as int32_t {
        buf_set_sign(
            buf,
            id,
            group,
            prio,
            mark.pos.row as linenr_T + 1 as linenr_T,
            sp,
        );
    }
    return mark.pos.row as linenr_T + 1 as linenr_T;
}
unsafe extern "C" fn buf_findsign(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t || !group.is_null() && ns == 0 as int64_t {
        return 0 as ::core::ffi::c_int;
    }
    return marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns as uint32_t,
        id as uint32_t,
        false_0 != 0,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    )
    .pos
    .row as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sign_row_cmp(
    mut p1: *const ::core::ffi::c_void,
    mut p2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const MTKey = p1 as *mut MTKey;
    let mut s2: *const MTKey = p2 as *mut MTKey;
    if (*s1).pos.row != (*s2).pos.row {
        return if (*s1).pos.row > (*s2).pos.row {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    let mut sh1: *mut DecorSignHighlight = decor_find_sign(mt_decor(*s1));
    let mut sh2: *mut DecorSignHighlight = decor_find_sign(mt_decor(*s2));
    '_c2rust_label: {
        if !sh1.is_null() && !sh2.is_null() {
        } else {
            __assert_fail(
                b"sh1 && sh2\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/sign.rs\0".as_ptr() as *const ::core::ffi::c_char,
                178 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut si1: SignItem = SignItem {
        sh: sh1,
        id: (*s1).id,
    };
    let mut si2: SignItem = SignItem {
        sh: sh2,
        id: (*s2).id,
    };
    return sign_item_cmp(&si1, &si2);
}
unsafe extern "C" fn buf_delete_signs(
    mut buf: *mut buf_T,
    mut group: *mut ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t {
        return FAIL;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut row: ::core::ffi::c_int = if atlnum > 0 as linenr_T {
        atlnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut signs: C2Rust_Unnamed_23 = C2Rust_Unnamed_23 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<MTKey>(),
    };
    if atlnum > 0 as linenr_T {
        if !marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            row,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        ) {
            return FAIL;
        }
        let mut pair: MTPair = MTPair {
            start: MTKey {
                pos: MTPos { row: 0, col: 0 },
                ns: 0,
                id: 0,
                flags: 0,
                decor_data: DecorInlineData {
                    hl: DecorHighlightInline {
                        flags: 0,
                        priority: 0,
                        hl_id: 0,
                        conceal_char: 0,
                    },
                },
            },
            end_pos: MTPos { row: 0, col: 0 },
            end_right_gravity: false,
        };
        while marktree_itr_step_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            if (ns == UINT32_MAX as int64_t || ns == pair.start.ns as int64_t)
                && mt_decor_sign(pair.start) as ::core::ffi::c_int != 0
            {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh1 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh1 as isize) = pair.start;
            }
        }
    } else {
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            0 as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if row != 0 && mark.pos.row > row as int32_t {
            break;
        }
        if !mt_end(mark)
            && mt_decor_sign(mark) as ::core::ffi::c_int != 0
            && (id == 0 as ::core::ffi::c_int || mark.id as ::core::ffi::c_int == id)
            && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
        {
            if atlnum > 0 as linenr_T {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh2 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh2 as isize) = mark;
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                );
            } else {
                extmark_del(buf, &raw mut itr as *mut MarkTreeIter, mark, true_0 != 0);
            }
        } else {
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
    }
    if signs.size != 0 {
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<MTKey>(),
            Some(
                sign_row_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        extmark_del_id(
            buf,
            (*signs.items.offset(0 as ::core::ffi::c_int as isize)).ns,
            (*signs.items.offset(0 as ::core::ffi::c_int as isize)).id,
        );
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<MTKey>();
    } else if atlnum > 0 as linenr_T {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn buf_has_signs(mut buf: *const buf_T) -> bool {
    return buf_meta_total(buf, kMTMetaSignHL).wrapping_add(buf_meta_total(buf, kMTMetaSignText))
        != 0;
}
unsafe extern "C" fn sign_list_placed(mut rbuf: *mut buf_T, mut group: *mut ::core::ffi::c_char) {
    let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut namebuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut groupbuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut buf: *mut buf_T = if !rbuf.is_null() {
        rbuf
    } else {
        firstbuf.get()
    };
    let mut ns: int64_t = group_get_ns(group);
    msg_puts_title(gettext(
        b"\n--- Signs ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    while !buf.is_null() && !got_int.get() {
        if buf_has_signs(buf) {
            msg_putchar('\n' as ::core::ffi::c_int);
            vim_snprintf(
                &raw mut lbuf as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                gettext(b"Signs for %s:\0".as_ptr() as *const ::core::ffi::c_char),
                (*buf).b_fname,
            );
            msg_puts_hl(
                &raw mut lbuf as *mut ::core::ffi::c_char,
                HLF_D,
                false_0 != 0,
            );
        }
        if ns >= 0 as int64_t {
            let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1];
            let mut signs: C2Rust_Unnamed_25 = C2Rust_Unnamed_25 {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<MTKey>(),
            };
            marktree_itr_get(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                0 as int32_t,
                0 as ::core::ffi::c_int,
                &raw mut itr as *mut MarkTreeIter,
            );
            while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
                let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
                if !mt_end(mark)
                    && mt_decor_sign(mark) as ::core::ffi::c_int != 0
                    && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
                {
                    if signs.size == signs.capacity {
                        signs.capacity = if signs.capacity != 0 {
                            signs.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        signs.items = xrealloc(
                            signs.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                        ) as *mut MTKey;
                    } else {
                    };
                    let c2rust_fresh4 = signs.size;
                    signs.size = signs.size.wrapping_add(1);
                    *signs.items.offset(c2rust_fresh4 as isize) = mark;
                }
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                );
            }
            if signs.size != 0 {
                qsort(
                    signs.items.offset(0 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    signs.size,
                    ::core::mem::size_of::<MTKey>(),
                    Some(
                        sign_row_cmp
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                msg_putchar('\n' as ::core::ffi::c_int);
                let mut i: size_t = 0 as size_t;
                while i < signs.size {
                    namebuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    groupbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    let mut mark_0: MTKey = *signs.items.offset(i as isize);
                    let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark_0));
                    if !(*sh).sign_name.is_null() {
                        vim_snprintf(
                            &raw mut namebuf as *mut ::core::ffi::c_char,
                            MSG_BUF_LEN as size_t,
                            gettext(b"  name=%s\0".as_ptr() as *const ::core::ffi::c_char),
                            sign_get_name(sh),
                        );
                    }
                    if mark_0.ns != 0 as uint32_t {
                        vim_snprintf(
                            &raw mut groupbuf as *mut ::core::ffi::c_char,
                            MSG_BUF_LEN as size_t,
                            gettext(b"  group=%s\0".as_ptr() as *const ::core::ffi::c_char),
                            describe_ns(
                                mark_0.ns as NS,
                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        );
                    }
                    vim_snprintf(
                        &raw mut lbuf as *mut ::core::ffi::c_char,
                        MSG_BUF_LEN as size_t,
                        gettext(b"    line=%d  id=%u%s%s  priority=%d\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        mark_0.pos.row + 1 as int32_t,
                        mark_0.id,
                        &raw mut groupbuf as *mut ::core::ffi::c_char,
                        &raw mut namebuf as *mut ::core::ffi::c_char,
                        (*sh).priority as ::core::ffi::c_int,
                    );
                    msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
                    if i < signs.size.wrapping_sub(1 as size_t) {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    i = i.wrapping_add(1);
                }
                xfree(signs.items as *mut ::core::ffi::c_void);
                signs.capacity = 0 as size_t;
                signs.size = signs.capacity;
                signs.items = ::core::ptr::null_mut::<MTKey>();
            }
        }
        if !rbuf.is_null() {
            return;
        }
        buf = (*buf).b_next;
    }
}
unsafe extern "C" fn sign_cmd_idx(
    mut begin_cmd: *mut ::core::ffi::c_char,
    mut end_cmd: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut save: ::core::ffi::c_char = *end_cmd;
    *end_cmd = NUL as ::core::ffi::c_char;
    idx = 0 as ::core::ffi::c_int;
    while !((*cmds.ptr())[idx as usize].is_null()
        || strcmp(begin_cmd, (*cmds.ptr())[idx as usize]) == 0 as ::core::ffi::c_int)
    {
        idx += 1;
    }
    *end_cmd = save;
    return idx;
}
pub unsafe extern "C" fn describe_sign_text(
    mut buf: *mut ::core::ffi::c_char,
    mut sign_text: *mut schar_T,
) -> size_t {
    let mut p: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < SIGN_WIDTH as ::core::ffi::c_int {
        schar_get(buf.offset(p as isize), *sign_text.offset(i as isize));
        let mut len: size_t = strlen(buf.offset(p as isize));
        if len == 0 as size_t {
            break;
        }
        p = p.wrapping_add(len);
        i += 1;
    }
    return p;
}
pub unsafe extern "C" fn init_sign_text(
    mut sp: *mut sign_T,
    mut sign_text: *mut schar_T,
    mut text: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endp: *mut ::core::ffi::c_char =
        text.offset(strlen(text) as ::core::ffi::c_int as isize);
    s = if !sp.is_null() { text } else { endp };
    while s.offset(1 as ::core::ffi::c_int as isize) < endp {
        if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            memmove(
                s as *mut ::core::ffi::c_void,
                s.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(s.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            endp = endp.offset(-1);
        }
        s = s.offset(1);
    }
    let mut cells: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    s = text;
    while s < endp {
        let mut c: ::core::ffi::c_int = 0;
        *sign_text.offset(cells as isize) = utfc_ptr2schar(s, &raw mut c);
        if !vim_isprintc(c) {
            break;
        }
        let mut width: ::core::ffi::c_int = utf_ptr2cells(s);
        if width == 2 as ::core::ffi::c_int {
            *sign_text.offset((cells + 1 as ::core::ffi::c_int) as isize) = 0 as schar_T;
        }
        cells += width;
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    if s != endp || cells > SIGN_WIDTH as ::core::ffi::c_int {
        if !sp.is_null() {
            semsg(
                gettext(b"E239: Invalid sign text: %s\0".as_ptr() as *const ::core::ffi::c_char),
                text,
            );
        }
        return FAIL;
    }
    if cells < 1 as ::core::ffi::c_int {
        *sign_text.offset(0 as ::core::ffi::c_int as isize) = 0 as schar_T;
    } else if cells == 1 as ::core::ffi::c_int {
        *sign_text.offset(1 as ::core::ffi::c_int as isize) = ' ' as ::core::ffi::c_int as schar_T;
    }
    return OK;
}
unsafe extern "C" fn sign_define_by_name(
    mut name: *mut ::core::ffi::c_char,
    mut icon: *mut ::core::ffi::c_char,
    mut text: *mut ::core::ffi::c_char,
    mut linehl: *mut ::core::ffi::c_char,
    mut texthl: *mut ::core::ffi::c_char,
    mut culhl: *mut ::core::ffi::c_char,
    mut numhl: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
    let mut new_sign: bool = false_0 != 0;
    let mut sp: *mut *mut sign_T = map_put_ref_cstr_t_ptr_t(
        sign_map.ptr(),
        name as cstr_t,
        &raw mut key,
        &raw mut new_sign,
    ) as *mut *mut sign_T;
    if new_sign {
        *key = xstrdup(name) as cstr_t;
        *sp = xcalloc(1 as size_t, ::core::mem::size_of::<sign_T>()) as *mut sign_T;
        (**sp).sn_name = *key as *mut ::core::ffi::c_char;
    }
    if !icon.is_null() {
        xfree((**sp).sn_icon as *mut ::core::ffi::c_void);
        (**sp).sn_icon = xstrdup(icon);
        backslash_halve((**sp).sn_icon);
    }
    if !text.is_null() && init_sign_text(*sp, &raw mut (**sp).sn_text as *mut schar_T, text) == FAIL
    {
        return FAIL;
    }
    (**sp).sn_priority = prio;
    let mut arg: [*mut ::core::ffi::c_char; 4] = [linehl, texthl, culhl, numhl];
    let mut hl: [*mut ::core::ffi::c_int; 4] = [
        &raw mut (**sp).sn_line_hl,
        &raw mut (**sp).sn_text_hl,
        &raw mut (**sp).sn_cul_hl,
        &raw mut (**sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if !arg[i as usize].is_null() {
            *hl[i as usize] = if *arg[i as usize] as ::core::ffi::c_int != 0 {
                syn_check_group(arg[i as usize], strlen(arg[i as usize]))
            } else {
                0 as ::core::ffi::c_int
            };
        }
        i += 1;
    }
    if !new_sign {
        let mut did_redraw: bool = false_0 != 0;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < decor_item_count() {
            let mut sh: *mut DecorSignHighlight = decor_item(i_0 as uint32_t);
            if !(*sh).sign_name.is_null()
                && strcmp((*sh).sign_name, name) == 0 as ::core::ffi::c_int
            {
                memcpy(
                    &raw mut (*sh).text as *mut schar_T as *mut ::core::ffi::c_void,
                    &raw mut (**sp).sn_text as *mut schar_T as *const ::core::ffi::c_void,
                    (SIGN_WIDTH as ::core::ffi::c_int as size_t)
                        .wrapping_mul(::core::mem::size_of::<schar_T>()),
                );
                (*sh).hl_id = (**sp).sn_text_hl;
                (*sh).line_hl_id = (**sp).sn_line_hl;
                (*sh).number_hl_id = (**sp).sn_num_hl;
                (*sh).cursorline_hl_id = (**sp).sn_cul_hl;
                if !did_redraw {
                    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                        firstwin.get()
                    } else {
                        (*curtab.get()).tp_firstwin
                    };
                    while !wp.is_null() {
                        if buf_has_signs((*wp).w_buffer) {
                            redraw_buf_later((*wp).w_buffer, UPD_NOT_VALID);
                        }
                        wp = (*wp).w_next;
                    }
                    did_redraw = true_0 != 0;
                }
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    return OK;
}
unsafe extern "C" fn sign_undefine_by_name(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sp: *mut sign_T = map_del_cstr_t_ptr_t(
        sign_map.ptr(),
        name as cstr_t,
        ::core::ptr::null_mut::<cstr_t>(),
    ) as *mut sign_T;
    if sp.is_null() {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    xfree((*sp).sn_name as *mut ::core::ffi::c_void);
    xfree((*sp).sn_icon as *mut ::core::ffi::c_void);
    xfree(sp as *mut ::core::ffi::c_void);
    return OK;
}
unsafe extern "C" fn sign_list_defined(mut sp: *mut sign_T) {
    smsg(
        0 as ::core::ffi::c_int,
        b"sign %s\0".as_ptr() as *const ::core::ffi::c_char,
        (*sp).sn_name,
    );
    if !(*sp).sn_icon.is_null() {
        msg_puts(b" icon=\0".as_ptr() as *const ::core::ffi::c_char);
        msg_outtrans((*sp).sn_icon, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_puts(gettext(
            b" (not supported)\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        msg_puts(b" text=\0".as_ptr() as *const ::core::ffi::c_char);
        let mut buf: [::core::ffi::c_char; 64] = [0; 64];
        describe_sign_text(
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut (*sp).sn_text as *mut schar_T,
        );
        msg_outtrans(
            &raw mut buf as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    if (*sp).sn_priority > 0 as ::core::ffi::c_int {
        let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
        vim_snprintf(
            &raw mut lbuf as *mut ::core::ffi::c_char,
            MSG_BUF_LEN as size_t,
            b" priority=%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*sp).sn_priority,
        );
        msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
    }
    static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
        b" linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut hl: [::core::ffi::c_int; 4] = [
        (*sp).sn_line_hl,
        (*sp).sn_text_hl,
        (*sp).sn_cul_hl,
        (*sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if hl[i as usize] > 0 as ::core::ffi::c_int {
            msg_puts((*arg.ptr())[i as usize]);
            let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                ::core::ptr::null_mut::<expand_T>(),
                hl[i as usize] - 1 as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_puts(if !p.is_null() {
                p
            } else {
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char
            });
        }
        i += 1;
    }
}
unsafe extern "C" fn sign_list_by_name(mut name: *mut ::core::ffi::c_char) {
    let mut sp: *mut sign_T = map_get_cstr_t_ptr_t(sign_map.ptr(), name as cstr_t) as *mut sign_T;
    if !sp.is_null() {
        sign_list_defined(sp);
    } else {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
    };
}
unsafe extern "C" fn sign_place(
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut prio: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if !group.is_null()
        && (*group as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *group as ::core::ffi::c_int == NUL)
    {
        return FAIL;
    }
    let mut sp: *mut sign_T = map_get_cstr_t_ptr_t(sign_map.ptr(), name as cstr_t) as *mut sign_T;
    if sp.is_null() {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    if prio == -1 as ::core::ffi::c_int {
        prio = if (*sp).sn_priority != -1 as ::core::ffi::c_int {
            (*sp).sn_priority
        } else {
            SIGN_DEF_PRIO as ::core::ffi::c_int
        };
    }
    if lnum > 0 as linenr_T {
        buf_set_sign(buf, id, group, prio, lnum, sp);
    } else {
        lnum = buf_mod_sign(buf, id, group, prio, sp);
    }
    if lnum <= 0 as linenr_T {
        semsg(
            gettext(
                b"E885: Not possible to change sign %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            name,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn sign_unplace_inner(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    if !buf_has_signs(buf) {
        return FAIL;
    }
    if id == 0 as ::core::ffi::c_int
        || atlnum > 0 as linenr_T
        || !group.is_null() && *group as ::core::ffi::c_int == '*' as ::core::ffi::c_int
    {
        if buf_delete_signs(buf, group, id, atlnum) == 0 {
            return FAIL;
        }
    } else {
        let mut ns: int64_t = group_get_ns(group);
        if ns < 0 as int64_t || !extmark_del_id(buf, ns as uint32_t, id as uint32_t) {
            return FAIL;
        }
    }
    return OK;
}
unsafe extern "C" fn sign_unplace(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    if !buf.is_null() {
        return sign_unplace_inner(buf, id, group, atlnum);
    } else {
        let mut retval: ::core::ffi::c_int = OK;
        let mut cbuf: *mut buf_T = firstbuf.get();
        while !cbuf.is_null() {
            if sign_unplace_inner(cbuf, id, group, atlnum) == 0 {
                retval = FAIL;
            }
            cbuf = (*cbuf).b_next;
        }
        return retval;
    };
}
unsafe extern "C" fn sign_jump(
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) -> linenr_T {
    let mut lnum: linenr_T = buf_findsign(buf, id, group) as linenr_T;
    if lnum <= 0 as linenr_T {
        semsg(
            gettext(b"E157: Invalid sign ID: %d\0".as_ptr() as *const ::core::ffi::c_char),
            id,
        );
        return -1 as linenr_T;
    }
    if !buf_jump_open_win(buf).is_null() {
        (*curwin.get()).w_cursor.lnum = lnum;
        check_cursor_lnum(curwin.get());
        beginline(BL_WHITE as ::core::ffi::c_int);
    } else {
        if (*buf).b_fname.is_null() {
            emsg(gettext(
                b"E934: Cannot jump to a buffer that does not have a name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return -1 as linenr_T;
        }
        let mut cmdlen: size_t = strlen((*buf).b_fname).wrapping_add(24 as size_t);
        let mut cmd: *mut ::core::ffi::c_char = xmallocz(cmdlen) as *mut ::core::ffi::c_char;
        snprintf(
            cmd,
            cmdlen,
            b"e +%ld %s\0".as_ptr() as *const ::core::ffi::c_char,
            lnum as int64_t,
            (*buf).b_fname,
        );
        do_cmdline_cmd(cmd);
        xfree(cmd as *mut ::core::ffi::c_void);
    }
    foldOpenCursor();
    return lnum;
}
unsafe extern "C" fn sign_define_cmd(
    mut name: *mut ::core::ffi::c_char,
    mut cmdline: *mut ::core::ffi::c_char,
) {
    let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    loop {
        let mut arg: *mut ::core::ffi::c_char = skipwhite(cmdline);
        if *arg as ::core::ffi::c_int == NUL {
            break;
        }
        cmdline = skiptowhite_esc(arg);
        if strncmp(
            arg,
            b"icon=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            icon = arg.offset(5 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"text=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            text = arg.offset(5 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"linehl=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            linehl = arg.offset(7 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"texthl=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            texthl = arg.offset(7 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"culhl=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            culhl = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"numhl=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            numhl = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            prio = atoi(arg.offset(9 as ::core::ffi::c_int as isize));
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        if *cmdline as ::core::ffi::c_int == NUL {
            break;
        }
        let c2rust_fresh7 = cmdline;
        cmdline = cmdline.offset(1);
        *c2rust_fresh7 = NUL as ::core::ffi::c_char;
    }
    sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
}
unsafe extern "C" fn sign_place_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *mut ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
) {
    if id <= 0 as ::core::ffi::c_int {
        if lnum >= 0 as linenr_T
            || !name.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            sign_list_placed(buf, group);
        }
    } else {
        if name.is_null()
            || buf.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut uid: uint32_t = id as uint32_t;
        sign_place(&raw mut uid, group, name, buf, lnum, prio);
    };
}
unsafe extern "C" fn sign_unplace_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    if lnum >= 0 as linenr_T
        || !name.is_null()
        || !group.is_null() && *group as ::core::ffi::c_int == NUL
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if id == -1 as ::core::ffi::c_int {
        lnum = (*curwin.get()).w_cursor.lnum;
        buf = (*curwin.get()).w_buffer;
    }
    if sign_unplace(
        buf,
        if 0 as ::core::ffi::c_int > id {
            0 as ::core::ffi::c_int
        } else {
            id
        },
        group,
        lnum,
    ) == 0
        && lnum > 0 as linenr_T
    {
        emsg(gettext(
            b"E159: Missing sign number\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
unsafe extern "C" fn sign_jump_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    if name.is_null() && group.is_null() && id == -1 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        return;
    }
    if buf.is_null()
        || !group.is_null() && *group as ::core::ffi::c_int == NUL
        || lnum >= 0 as linenr_T
        || !name.is_null()
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    sign_jump(id, group, buf);
}
unsafe extern "C" fn parse_sign_cmd_args(
    mut cmd: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_int,
    mut group: *mut *mut ::core::ffi::c_char,
    mut prio: *mut ::core::ffi::c_int,
    mut buf: *mut *mut buf_T,
    mut lnum: *mut linenr_T,
) -> ::core::ffi::c_int {
    let mut arg1: *mut ::core::ffi::c_char = arg;
    let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum_arg: bool = false_0 != 0;
    if ascii_isdigit(*arg as ::core::ffi::c_int) {
        *id = getdigits_int(&raw mut arg, true_0 != 0, 0 as ::core::ffi::c_int);
        if !ascii_iswhite(*arg as ::core::ffi::c_int) && *arg as ::core::ffi::c_int != NUL {
            *id = -1 as ::core::ffi::c_int;
            arg = arg1;
        } else {
            arg = skipwhite(arg);
        }
    }
    while *arg as ::core::ffi::c_int != NUL {
        if strncmp(
            arg,
            b"line=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            *lnum = atoi(arg) as linenr_T;
            arg = skiptowhite(arg);
            lnum_arg = true_0 != 0;
        } else if strncmp(
            arg,
            b"*\0".as_ptr() as *const ::core::ffi::c_char,
            1 as size_t,
        ) == 0 as ::core::ffi::c_int
            && cmd == SIGNCMD_UNPLACE
        {
            if *id != -1 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return FAIL;
            }
            *id = -2 as ::core::ffi::c_int;
            arg = skiptowhite(arg.offset(1 as ::core::ffi::c_int as isize));
        } else if strncmp(
            arg,
            b"name=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            let mut namep: *mut ::core::ffi::c_char = arg;
            arg = skiptowhite(arg);
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh5 = arg;
                arg = arg.offset(1);
                *c2rust_fresh5 = NUL as ::core::ffi::c_char;
            }
            while *namep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                && *namep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                namep = namep.offset(1);
            }
            *name = namep;
        } else if strncmp(
            arg,
            b"group=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(6 as ::core::ffi::c_int as isize);
            *group = arg;
            arg = skiptowhite(arg);
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh6 = arg;
                arg = arg.offset(1);
                *c2rust_fresh6 = NUL as ::core::ffi::c_char;
            }
        } else if strncmp(
            arg,
            b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(9 as ::core::ffi::c_int as isize);
            *prio = atoi(arg);
            arg = skiptowhite(arg);
        } else if strncmp(
            arg,
            b"file=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            filename = arg;
            *buf = buflist_findname_exp(arg);
            break;
        } else if strncmp(
            arg,
            b"buffer=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(7 as ::core::ffi::c_int as isize);
            filename = arg;
            *buf = buflist_findnr(getdigits_int(
                &raw mut arg,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            ));
            if *skipwhite(arg) as ::core::ffi::c_int != NUL {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    arg,
                );
            }
            break;
        } else {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return FAIL;
        }
        arg = skipwhite(arg);
    }
    if !filename.is_null() && (*buf).is_null() {
        semsg(
            gettext(&raw const e_invalid_buffer_name_str as *const ::core::ffi::c_char),
            filename,
        );
        return FAIL;
    }
    if filename.is_null()
        && (cmd == SIGNCMD_PLACE && lnum_arg as ::core::ffi::c_int != 0 || cmd == SIGNCMD_JUMP)
    {
        *buf = (*curwin.get()).w_buffer;
    }
    return OK;
}
pub unsafe fn ex_sign(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
    let mut idx: ::core::ffi::c_int = sign_cmd_idx(arg, p);
    if idx == SIGNCMD_LAST {
        semsg(
            gettext(b"E160: Unknown sign command: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    arg = skipwhite(p);
    if idx <= SIGNCMD_LIST {
        if idx == SIGNCMD_LIST && *arg as ::core::ffi::c_int == NUL {
            let mut sp: *mut sign_T = ::core::ptr::null_mut::<sign_T>();
            let mut __i: uint32_t = 0;
            __i = 0 as uint32_t;
            while __i < (*sign_map.ptr()).set.h.n_keys {
                sp = *(*sign_map.ptr()).values.offset(__i as isize) as *mut sign_T;
                sign_list_defined(sp);
                __i = __i.wrapping_add(1);
            }
        } else if *arg as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E156: Missing sign name\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            p = skiptowhite(arg);
            if *p as ::core::ffi::c_int != NUL {
                let c2rust_fresh0 = p;
                p = p.offset(1);
                *c2rust_fresh0 = NUL as ::core::ffi::c_char;
            }
            while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                arg = arg.offset(1);
            }
            if idx == SIGNCMD_DEFINE {
                sign_define_cmd(arg, p);
            } else if idx == SIGNCMD_LIST {
                sign_list_by_name(arg);
            } else {
                sign_undefine_by_name(arg);
            }
            return;
        }
    } else {
        let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut lnum: linenr_T = -1 as linenr_T;
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if parse_sign_cmd_args(
            idx,
            arg,
            &raw mut name,
            &raw mut id,
            &raw mut group,
            &raw mut prio,
            &raw mut buf,
            &raw mut lnum,
        ) == FAIL
        {
            return;
        }
        if idx == SIGNCMD_PLACE {
            sign_place_cmd(buf, lnum, name, id, group, prio);
        } else if idx == SIGNCMD_UNPLACE {
            sign_unplace_cmd(buf, lnum, name, id, group);
        } else if idx == SIGNCMD_JUMP {
            sign_jump_cmd(buf, lnum, name, id, group);
        }
    };
}
unsafe extern "C" fn sign_get_info_dict(mut sp: *mut sign_T) -> *mut dict_T {
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_dict_add_str(
        d,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*sp).sn_name,
    );
    if !(*sp).sn_icon.is_null() {
        tv_dict_add_str(
            d,
            b"icon\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*sp).sn_icon,
        );
    }
    if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        let mut buf: [::core::ffi::c_char; 64] = [0; 64];
        describe_sign_text(
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut (*sp).sn_text as *mut schar_T,
        );
        tv_dict_add_str(
            d,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    }
    if (*sp).sn_priority > 0 as ::core::ffi::c_int {
        tv_dict_add_nr(
            d,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*sp).sn_priority as varnumber_T,
        );
    }
    static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
        b"linehl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"texthl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"culhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"numhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut hl: [::core::ffi::c_int; 4] = [
        (*sp).sn_line_hl,
        (*sp).sn_text_hl,
        (*sp).sn_cul_hl,
        (*sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if hl[i as usize] > 0 as ::core::ffi::c_int {
            let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                ::core::ptr::null_mut::<expand_T>(),
                hl[i as usize] - 1 as ::core::ffi::c_int,
                false_0 != 0,
            );
            tv_dict_add_str(
                d,
                (*arg.ptr())[i as usize],
                strlen((*arg.ptr())[i as usize]),
                if !p.is_null() {
                    p
                } else {
                    b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }
        i += 1;
    }
    return d;
}
unsafe extern "C" fn sign_get_placed_info_dict(mut mark: MTKey) -> *mut dict_T {
    let mut d: *mut dict_T = tv_dict_alloc();
    let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
    tv_dict_add_str(
        d,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        sign_get_name(sh),
    );
    tv_dict_add_nr(
        d,
        b"id\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        mark.id as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_str(
        d,
        b"group\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        describe_ns(mark.ns as NS, b"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    tv_dict_add_nr(
        d,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (mark.pos.row + 1 as int32_t) as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*sh).priority as varnumber_T,
    );
    return d;
}
pub unsafe extern "C" fn get_buffer_signs(mut buf: *mut buf_T) -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        0 as int32_t,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if !mt_end(mark) && mt_decor_sign(mark) as ::core::ffi::c_int != 0 {
            tv_list_append_dict(l, sign_get_placed_info_dict(mark));
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    return l;
}
unsafe extern "C" fn sign_get_placed_in_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut sign_id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(retlist, d);
    tv_dict_add_nr(
        d,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*buf).handle as varnumber_T,
    );
    let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    tv_dict_add_list(
        d,
        b"signs\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    let mut ns: int64_t = group_get_ns(group);
    if !buf_has_signs(buf) || ns < 0 as int64_t {
        return;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut signs: C2Rust_Unnamed_28 = C2Rust_Unnamed_28 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<MTKey>(),
    };
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        if lnum != 0 {
            lnum as int32_t - 1 as int32_t
        } else {
            0 as int32_t
        },
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if lnum != 0 && mark.pos.row >= lnum {
            break;
        }
        if !mt_end(mark)
            && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
            && (lnum == 0 as linenr_T && sign_id == 0 as ::core::ffi::c_int
                || sign_id == 0 as ::core::ffi::c_int && lnum == mark.pos.row + 1 as int32_t
                || lnum == 0 as linenr_T && sign_id == mark.id as ::core::ffi::c_int
                || lnum == mark.pos.row + 1 as int32_t && sign_id == mark.id as ::core::ffi::c_int)
        {
            if mt_decor_sign(mark) {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh10 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh10 as isize) = mark;
            }
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    if signs.size != 0 {
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<MTKey>(),
            Some(
                sign_row_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut i: size_t = 0 as size_t;
        while i < signs.size {
            tv_list_append_dict(
                l,
                sign_get_placed_info_dict(*signs.items.offset(i as isize)),
            );
            i = i.wrapping_add(1);
        }
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<MTKey>();
    }
}
unsafe extern "C" fn sign_get_placed(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    if !buf.is_null() {
        sign_get_placed_in_buf(buf, lnum, id, group, retlist);
    } else {
        let mut cbuf: *mut buf_T = firstbuf.get();
        while !cbuf.is_null() {
            if buf_has_signs(cbuf) {
                sign_get_placed_in_buf(cbuf, 0 as linenr_T, id, group, retlist);
            }
            cbuf = (*cbuf).b_next;
        }
    };
}
pub unsafe extern "C" fn free_signs() {
    let mut name: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
    let mut names: C2Rust_Unnamed_26 = C2Rust_Unnamed_26 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<cstr_t>(),
    };
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*sign_map.ptr()).set.h.n_keys {
        name = *(*sign_map.ptr()).set.keys.offset(__i as isize);
        if names.size == names.capacity {
            names.capacity = if names.capacity != 0 {
                names.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            names.items = xrealloc(
                names.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<cstr_t>().wrapping_mul(names.capacity),
            ) as *mut cstr_t;
        } else {
        };
        let c2rust_fresh8 = names.size;
        names.size = names.size.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *names.items.offset(c2rust_fresh8 as isize);
        *c2rust_lvalue_ptr = name;
        __i = __i.wrapping_add(1);
    }
    let mut i: size_t = 0 as size_t;
    while i < names.size {
        sign_undefine_by_name(*names.items.offset(i as isize) as *const ::core::ffi::c_char);
        i = i.wrapping_add(1);
    }
    xfree(names.items as *mut ::core::ffi::c_void);
    names.capacity = 0 as size_t;
    names.size = names.capacity;
    names.items = ::core::ptr::null_mut::<cstr_t>();
}
static expand_what: GlobalCell<C2Rust_Unnamed_27> = GlobalCell::new(EXP_SUBCMD);
unsafe extern "C" fn get_nth_sign_name(mut idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut name: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
    let mut current_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*sign_map.ptr()).set.h.n_keys {
        name = *(*sign_map.ptr()).set.keys.offset(__i as isize);
        let c2rust_fresh9 = current_idx;
        current_idx = current_idx + 1;
        if c2rust_fresh9 == idx {
            return name as *mut ::core::ffi::c_char;
        }
        __i = __i.wrapping_add(1);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_nth_sign_group_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < (*sign_ns.ptr()).size as ::core::ffi::c_int {
        return describe_ns(
            *(*sign_ns.ptr()).items.offset(idx as isize) as NS,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        ) as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_sign_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match expand_what.get() as ::core::ffi::c_uint {
        0 => return (*cmds.ptr())[idx as usize],
        1 => {
            let mut define_arg: [*mut ::core::ffi::c_char; 8] = [
                b"culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"icon=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"text=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return define_arg[idx as usize];
        }
        2 => {
            let mut place_arg: [*mut ::core::ffi::c_char; 7] = [
                b"line=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"name=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return place_arg[idx as usize];
        }
        3 => {
            let mut list_arg: [*mut ::core::ffi::c_char; 4] = [
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return list_arg[idx as usize];
        }
        4 => {
            let mut unplace_arg: [*mut ::core::ffi::c_char; 4] = [
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return unplace_arg[idx as usize];
        }
        5 => return get_nth_sign_name(idx),
        6 => return get_nth_sign_group_name(idx),
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
pub unsafe extern "C" fn set_context_in_sign_cmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_SIGN as ::core::ffi::c_int;
    expand_what.set(EXP_SUBCMD);
    (*xp).xp_pattern = arg;
    let mut end_subcmd: *mut ::core::ffi::c_char = skiptowhite(arg);
    if *end_subcmd as ::core::ffi::c_int == NUL {
        return;
    }
    let mut cmd_idx: ::core::ffi::c_int = sign_cmd_idx(arg, end_subcmd);
    let mut begin_subcmd_args: *mut ::core::ffi::c_char = skipwhite(end_subcmd);
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = begin_subcmd_args;
    loop {
        p = skipwhite(p);
        last = p;
        p = skiptowhite(p);
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
    }
    p = vim_strchr(last, '=' as ::core::ffi::c_int);
    if p.is_null() {
        (*xp).xp_pattern = last;
        match cmd_idx {
            SIGNCMD_DEFINE => {
                expand_what.set(EXP_DEFINE);
            }
            SIGNCMD_PLACE => {
                if ascii_isdigit(*begin_subcmd_args as ::core::ffi::c_int) {
                    expand_what.set(EXP_PLACE);
                } else {
                    expand_what.set(EXP_LIST);
                }
            }
            SIGNCMD_LIST | SIGNCMD_UNDEFINE => {
                expand_what.set(EXP_SIGN_NAMES);
            }
            SIGNCMD_JUMP | SIGNCMD_UNPLACE => {
                expand_what.set(EXP_UNPLACE);
            }
            _ => {
                (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
    } else {
        (*xp).xp_pattern = p.offset(1 as ::core::ffi::c_int as isize);
        match cmd_idx {
            SIGNCMD_DEFINE => {
                if strncmp(
                    last,
                    b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
                } else if strncmp(
                    last,
                    b"icon\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            SIGNCMD_PLACE => {
                if strncmp(
                    last,
                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_NAMES);
                } else if strncmp(
                    last,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_GROUPS);
                } else if strncmp(
                    last,
                    b"file\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
                if strncmp(
                    last,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_GROUPS);
                } else if strncmp(
                    last,
                    b"file\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            _ => {
                (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
    };
}
unsafe extern "C" fn sign_define_from_dict(
    mut name: *mut ::core::ffi::c_char,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    if name.is_null() {
        name = tv_dict_get_string(
            dict,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if name.is_null()
            || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if !dict.is_null() {
        icon = tv_dict_get_string(
            dict,
            b"icon\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        linehl = tv_dict_get_string(
            dict,
            b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        text = tv_dict_get_string(
            dict,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        texthl = tv_dict_get_string(
            dict,
            b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        culhl = tv_dict_get_string(
            dict,
            b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        numhl = tv_dict_get_string(
            dict,
            b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        prio = tv_dict_get_number_def(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
    }
    return sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio)
        - 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sign_define_multiple(mut l: *mut list_T, mut retlist: *mut list_T) {
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                retval = sign_define_from_dict(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number(retlist, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
pub unsafe extern "C" fn f_sign_define(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        sign_define_multiple(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            (*rettv).vval.v_list,
        );
        return;
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut name: *mut ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    if name.is_null() {
        return;
    }
    if tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict
    } else {
        ::core::ptr::null_mut::<dict_T>()
    };
    (*rettv).vval.v_number = sign_define_from_dict(name, d) as varnumber_T;
}
pub unsafe extern "C" fn f_sign_getdefined(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut sp: *mut sign_T = ::core::ptr::null_mut::<sign_T>();
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*sign_map.ptr()).set.h.n_keys {
            sp = *(*sign_map.ptr()).values.offset(__i as isize) as *mut sign_T;
            tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp));
            __i = __i.wrapping_add(1);
        }
    } else {
        let mut sp_0: *mut sign_T = map_get_cstr_t_ptr_t(
            sign_map.ptr(),
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        ) as *mut sign_T;
        if !sp_0.is_null() {
            tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp_0));
        }
    };
}
pub unsafe extern "C" fn f_sign_getplaced(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut sign_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut notanum: bool = false_0 != 0;
    tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = get_buf_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
        if buf.is_null() {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_nonnull_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
                return;
            }
            let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            let mut dict: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            di = tv_dict_find(
                dict,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                lnum = tv_get_lnum(&raw mut (*di).di_tv);
                if lnum <= 0 as linenr_T {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                sign_id =
                    tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
                if notanum {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                group = tv_get_string_chk(&raw mut (*di).di_tv);
                if group.is_null() {
                    return;
                }
                if *group as ::core::ffi::c_int == NUL {
                    group = ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
        }
    }
    sign_get_placed(buf, lnum, sign_id, group, (*rettv).vval.v_list);
}
pub unsafe extern "C" fn f_sign_jump(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut notanum: bool = false_0 != 0;
    let mut id: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut notanum,
    ) as ::core::ffi::c_int;
    if notanum {
        return;
    }
    if id <= 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut group: *mut ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    if group.is_null() {
        return;
    }
    if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        group = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut buf_T = get_buf_arg(argvars.offset(2 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    (*rettv).vval.v_number = sign_jump(id, group, buf) as varnumber_T;
}
unsafe extern "C" fn sign_place_from_dict(
    mut id_tv: *mut typval_T,
    mut group_tv: *mut typval_T,
    mut name_tv: *mut typval_T,
    mut buf_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut notanum: bool = false_0 != 0;
    if id_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            id_tv = &raw mut (*di).di_tv;
        }
    }
    if !id_tv.is_null() {
        id = tv_get_number_chk(id_tv, &raw mut notanum) as ::core::ffi::c_int;
        if notanum {
            return -1 as ::core::ffi::c_int;
        }
        if id < 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if group_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            group_tv = &raw mut (*di).di_tv;
        }
    }
    if !group_tv.is_null() {
        group = tv_get_string_chk(group_tv) as *mut ::core::ffi::c_char;
        if group.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            group = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if name_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            name_tv = &raw mut (*di).di_tv;
        }
    }
    if name_tv.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    name = tv_get_string_chk(name_tv) as *mut ::core::ffi::c_char;
    if name.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if buf_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            buf_tv = &raw mut (*di).di_tv;
        }
    }
    if buf_tv.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut buf: *mut buf_T = get_buf_arg(buf_tv);
    if buf.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut lnum: linenr_T = 0 as linenr_T;
    di = tv_dict_find(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        lnum = tv_get_lnum(&raw mut (*di).di_tv);
        if lnum <= 0 as linenr_T {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    di = tv_dict_find(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        prio = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
        if notanum {
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut uid: uint32_t = id as uint32_t;
    if sign_place(&raw mut uid, group, name, buf, lnum, prio) == OK {
        return uid as ::core::ffi::c_int;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_sign_place(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 4 as ::core::ffi::c_int) == FAIL {
            return;
        }
        dict = (*argvars.offset(4 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    (*rettv).vval.v_number = sign_place_from_dict(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        argvars.offset(2 as ::core::ffi::c_int as isize),
        argvars.offset(3 as ::core::ffi::c_int as isize),
        dict,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_sign_placelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut sign_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                sign_id = sign_place_from_dict(
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number((*rettv).vval.v_list, sign_id as varnumber_T);
            li = (*li).li_next;
        }
    }
}
unsafe extern "C" fn sign_undefine_multiple(mut l: *mut list_T, mut retlist: *mut list_T) {
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut name: *mut ::core::ffi::c_char =
                tv_get_string_chk(&raw const (*li).li_tv) as *mut ::core::ffi::c_char;
            if !name.is_null() && sign_undefine_by_name(name) == 1 as ::core::ffi::c_int {
                retval = 0 as ::core::ffi::c_int;
            }
            tv_list_append_number(retlist, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
pub unsafe extern "C" fn f_sign_undefine(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        sign_undefine_multiple(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            (*rettv).vval.v_list,
        );
        return;
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free_signs();
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else {
        let mut name: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if name.is_null() {
            return;
        }
        if sign_undefine_by_name(name) == OK {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    };
}
unsafe extern "C" fn sign_unplace_from_dict(
    mut group_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut group: *mut ::core::ffi::c_char = if !group_tv.is_null() {
        tv_get_string(group_tv) as *mut ::core::ffi::c_char
    } else {
        tv_dict_get_string(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        )
    };
    if !group.is_null()
        && *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        group = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !dict.is_null() {
        di = tv_dict_find(
            dict,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            buf = get_buf_arg(&raw mut (*di).di_tv);
            if buf.is_null() {
                return -1 as ::core::ffi::c_int;
            }
        }
        if !tv_dict_find(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        )
        .is_null()
        {
            id = tv_dict_get_number(dict, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
            if id <= 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1 as ::core::ffi::c_int;
            }
        }
    }
    return sign_unplace(buf, id, group, 0 as linenr_T) - 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_sign_unplace(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        dict = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    (*rettv).vval.v_number =
        sign_unplace_from_dict(argvars.offset(0 as ::core::ffi::c_int as isize), dict)
            as varnumber_T;
}
pub unsafe extern "C" fn f_sign_unplacelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                retval = sign_unplace_from_dict(
                    ::core::ptr::null_mut::<typval_T>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number((*rettv).vval.v_list, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
