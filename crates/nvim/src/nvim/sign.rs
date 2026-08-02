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
use crate::src::nvim::eval::typval::tv_list_first;
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
use crate::src::nvim::mbyte::{MAX_SCHAR_SIZE, utf_ptr2cells, utfc_ptr2len, utfc_ptr2schar};
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

// The carve of the transpiled module; see each child's docs.
mod command;
pub use self::command::*;
mod complete;
pub use self::complete::*;
mod vimscript;
pub use self::vimscript::*;
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
/// Room for [`describe_sign_text`]'s answer: SIGN_WIDTH cells of up to
/// `MAX_SCHAR_SIZE` bytes each, the last of which carries the NUL
/// `schar_get` writes.
pub(crate) const SIGN_TEXT_BUF: usize = SIGN_WIDTH as usize * MAX_SCHAR_SIZE as usize;

/// [`group_get_ns`]'s answer for the group `"*"`: every namespace, the
/// global one included.
pub(crate) const ALL_GROUPS: int64_t = UINT32_MAX as int64_t;

/// Orders marks the way `:sign` reports them and removes them: by row, then
/// by [`sign_item_cmp`] — priority, then mark id, then placement serial, all
/// with the newest first.
///
/// A stable sort is provably the permutation the `qsort` upstream uses
/// produced: `buf_put_decor_sh` hands every placed sign a distinct
/// `sign_add_id`, so the comparator is a total order and no two entries tie.
///
/// # Safety
/// Every mark must carry a live sign decoration.
pub(crate) unsafe fn sort_signs(signs: &mut [MTKey]) {
    // SAFETY: the caller's marks.
    unsafe {
        signs.sort_by(|a, b| {
            if a.pos.row != b.pos.row {
                return a.pos.row.cmp(&b.pos.row);
            }
            let (sh1, sh2) = (decor_find_sign(mt_decor(*a)), decor_find_sign(mt_decor(*b)));
            assert!(!sh1.is_null() && !sh2.is_null(), "sign mark without a sign");
            sign_item_cmp(
                &SignItem { sh: sh1, id: a.id },
                &SignItem { sh: sh2, id: b.id },
            )
            .cmp(&0)
        });
    }
}

/// The definition `:sign define` recorded under `name`, or null.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn sign_find(name: *const ::core::ffi::c_char) -> *mut sign_T {
    // SAFETY: the caller's name.
    unsafe { map_get_cstr_t_ptr_t(sign_map.ptr(), name as cstr_t) as *mut sign_T }
}

/// Every defined sign, in definition order.
///
/// A snapshot rather than an iterator: `:sign list` and `sign_getdefined()`
/// both format each entry as they walk, and formatting can re-enter.
pub(crate) fn sign_defs() -> Vec<*mut sign_T> {
    // SAFETY: reading the map's dense key array, which is what the
    // transpiled `map_foreach_value` expanded to.
    unsafe {
        let map = sign_map.ptr();
        (0..(*map).set.h.n_keys)
            .map(|i| *(*map).values.offset(i as isize) as *mut sign_T)
            .collect()
    }
}

/// The names of every defined sign, in definition order.
pub(crate) fn sign_names() -> Vec<*const ::core::ffi::c_char> {
    // SAFETY: as [`sign_defs`].
    unsafe {
        let map = sign_map.ptr();
        (0..(*map).set.h.n_keys)
            .map(|i| *(*map).set.keys.offset(i as isize))
            .collect()
    }
}

pub unsafe extern "C" fn buf_has_signs(mut buf: *const buf_T) -> bool {
    return buf_meta_total(buf, kMTMetaSignHL).wrapping_add(buf_meta_total(buf, kMTMetaSignText))
        != 0;
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
        // `sign_text` holds SIGN_WIDTH cells but this walk runs to the end of
        // `text` and only tests the width afterwards, so upstream (v0.12.4)
        // overruns the array for any text wider than two cells: on the heap
        // via `:sign define x text=xxx`, on the STACK via
        // nvim_buf_set_extmark{sign_text=...}. Dropping the out-of-range
        // stores is unobservable — those paths all fail and discard it.
        let sc: schar_T = utfc_ptr2schar(s, &raw mut c);
        if cells < SIGN_WIDTH as ::core::ffi::c_int {
            *sign_text.offset(cells as isize) = sc;
        }
        if !vim_isprintc(c) {
            break;
        }
        let mut width: ::core::ffi::c_int = utf_ptr2cells(s);
        if width == 2 as ::core::ffi::c_int
            && (cells + 1 as ::core::ffi::c_int) < (SIGN_WIDTH as ::core::ffi::c_int)
        {
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
