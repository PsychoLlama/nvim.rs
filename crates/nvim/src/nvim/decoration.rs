use crate::src::nvim::api::extmark::virt_text_to_array;
use crate::src::nvim::api::private::helpers::{arena_array, arena_string, cstr_as_string};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::change::changed_lines_invalidate_buf;
use crate::src::nvim::decoration_provider::decor_providers_invoke_conceal_line;
use crate::src::nvim::drawscreen::{
    conceal_cursor_line, redraw_buf_line_later, redraw_buf_range_later,
};
use crate::src::nvim::extmark::extmark_set;
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_from_char, schar_get, schar_get_first_codepoint, schar_high};
use crate::src::nvim::highlight::{hl_add_url, hl_combine_attr};
use crate::src::nvim::highlight_group::{syn_id2attr, syn_id2name};
use crate::src::nvim::main::{
    curtab, curwin, decor_state, first_tabpage, firstwin, hl_mode_str, namespace_localscope,
    virt_text_pos_str,
};
use crate::src::nvim::map::set_has_uint32_t;
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_EXT, MT_FLAG_DECOR_HL, MT_FLAG_DECOR_SIGNTEXT, kMTFilterSelect, mt_conceal_lines,
    mt_decor, mt_decor_any, mt_decor_sign, mt_end, mt_invalid,
};
use crate::src::nvim::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get, marktree_itr_get_filter,
    marktree_itr_get_overlap, marktree_itr_next, marktree_itr_next_filter,
    marktree_itr_step_out_filter, marktree_itr_step_overlap,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xrealloc};
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memmove, qsort};
use crate::src::nvim::sign::{buf_has_signs, describe_sign_text};
use crate::src::nvim::types::{
    Arena, Array, DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority,
    DecorPriorityInternal, DecorRange, DecorRange_data as C2Rust_Unnamed_22, DecorRangeKind,
    DecorRangeSlot, DecorSignHighlight, DecorState, DecorVirtText, Dict, Error, Integer, MTKey,
    MTNode, MTPair, MTPos, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_19, MetaFilter,
    MetaIndex, Object, OptInt, SignItem, SignTextAttrs, TriState, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, buf_T, colnr_T, int32_t, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeInteger, kObjectTypeString, key_value_pair, linenr_T, lpos_T, object,
    object_data as C2Rust_Unnamed_14, sattr_T, schar_T, size_t, tabpage_T, uint16_t, uint32_t,
    uint64_t, virt_line, win_T,
};

// The carve of the transpiled module; see each child's docs.
mod dict;
pub use self::dict::*;
mod signs;
pub use self::signs::*;
mod query;
pub use self::query::*;
mod state;
pub use self::state::*;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_16 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_17 = 128;
pub const kSHConceal: C2Rust_Unnamed_17 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_17 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_17 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_17 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_17 = 4;
pub const kSHHlEol: C2Rust_Unnamed_17 = 2;
pub const kSHIsSign: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_18 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_18 = 4;
pub const kVTHide: C2Rust_Unnamed_18 = 2;
pub const kVTIsLines: C2Rust_Unnamed_18 = 1;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaLines: MetaIndex = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_20 = 9;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kDecorKindUIWatched: C2Rust_Unnamed_21 = 4;
pub const kDecorKindVirtLines: C2Rust_Unnamed_21 = 3;
pub const kDecorKindVirtText: C2Rust_Unnamed_21 = 2;
pub const kDecorKindHighlight: C2Rust_Unnamed_21 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_27 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SignItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub name: *mut ::core::ffi::c_char,
    pub val: ::core::ffi::c_int,
}
pub const kExtmarkHighlight: C2Rust_Unnamed_29 = 32;
pub const kExtmarkSign: C2Rust_Unnamed_29 = 2;
pub const kExtmarkNone: C2Rust_Unnamed_29 = 1;
pub const kExtmarkVirtText: C2Rust_Unnamed_29 = 8;
pub const kExtmarkVirtLines: C2Rust_Unnamed_29 = 16;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
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
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
/// Whether marks in namespace `ns_id` are visible in `wp`. A namespace that is
/// not window-local is visible in every window; a window-local one only in the
/// windows that opted in.
///
/// # Safety
/// `wp` must point to a live window.
#[inline]
pub unsafe fn ns_in_win(ns_id: uint32_t, wp: *mut win_T) -> bool {
    unsafe {
        if !set_has_uint32_t(namespace_localscope.ptr(), ns_id) {
            return true;
        }
        set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id)
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
/// Every `DecorSignHighlight` that any extmark anywhere points at, in one
/// array. An extmark stores the *index* of the first item of its decoration
/// and each item the index of the next, so one decoration is a chain through
/// this store and `DECOR_ID_INVALID` ends it.
///
/// Entries are never removed. An index handed to a marktree entry has to stay
/// valid until that entry is deleted, so a freed item goes on a freelist
/// threaded through the same `next` field ([`DECOR_FREELIST`]) and is reused
/// in place.
static DECOR_ITEMS: GlobalCell<Vec<DecorSignHighlight>> = GlobalCell::new(Vec::new());

/// Index of the first free slot of [`DECOR_ITEMS`], or `DECOR_ID_INVALID`.
static DECOR_FREELIST: GlobalCell<uint32_t> = GlobalCell::new(DECOR_ID_INVALID);

/// A pointer to decoration item `idx`.
///
/// A pointer and not a borrow, because upstream hands these out and then
/// keeps writing through them across calls that can add another item — see
/// [`decor_put_sh`], which is what invalidates them. Panics on an index no
/// [`decor_put_sh`] handed out, where upstream would read past the array.
pub(crate) fn decor_item(idx: uint32_t) -> *mut DecorSignHighlight {
    DECOR_ITEMS.with(|items| {
        let items = &items[..];
        assert!((idx as usize) < items.len(), "decoration item out of range");
        items.as_ptr().cast_mut().wrapping_add(idx as usize)
    })
}

/// How many slots [`DECOR_ITEMS`] has handed out, free ones included.
pub(crate) fn decor_item_count() -> usize {
    DECOR_ITEMS.with(Vec::len)
}
pub static to_free_virt: GlobalCell<*mut DecorVirtText> =
    GlobalCell::new(::core::ptr::null_mut::<DecorVirtText>());
pub static to_free_sh: GlobalCell<uint32_t> = GlobalCell::new(UINT32_MAX as uint32_t);
pub unsafe extern "C" fn bufhl_add_hl_pos_offset(
    mut buf: *mut buf_T,
    mut src_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut pos_start: lpos_T,
    mut pos_end: lpos_T,
    mut offset: colnr_T,
) {
    let mut hl_start: colnr_T = 0 as colnr_T;
    let mut hl_end: colnr_T = 0 as colnr_T;
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    let mut lnum: linenr_T = pos_start.lnum;
    while lnum <= pos_end.lnum {
        let mut end_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pos_start.lnum < lnum && lnum < pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if lnum == pos_start.lnum && lnum < pos_end.lnum {
            hl_start = pos_start.col + offset;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if pos_start.lnum < lnum && lnum == pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            hl_end = pos_end.col + offset;
        } else if pos_start.lnum == lnum && pos_end.lnum == lnum {
            hl_start = pos_start.col + offset;
            hl_end = pos_end.col + offset;
        }
        extmark_set(
            buf,
            src_id as uint32_t,
            ::core::ptr::null_mut::<uint32_t>(),
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            hl_start,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int + end_off,
            hl_end,
            decor,
            MT_FLAG_DECOR_HL as uint16_t,
            true_0 != 0,
            false_0 != 0,
            true_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<Error>(),
        );
        lnum += 1;
    }
}
pub unsafe extern "C" fn decor_redraw(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
) {
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            let mut below: bool =
                (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                    && (*vt).flags as ::core::ffi::c_int & kVTLinesAbove as ::core::ffi::c_int == 0;
            let mut vt_lnum: linenr_T = row1 as linenr_T + 1 as linenr_T + below as linenr_T;
            redraw_buf_line_later(buf, vt_lnum, true_0 != 0);
            if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                || (*vt).pos as ::core::ffi::c_uint
                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut vt_col: colnr_T =
                    if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                        0 as colnr_T
                    } else {
                        col1 as colnr_T
                    };
                changed_lines_invalidate_buf(
                    buf,
                    vt_lnum,
                    vt_col,
                    vt_lnum + 1 as linenr_T,
                    0 as linenr_T,
                );
            }
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_item(idx);
            decor_redraw_sh(buf, row1, row2, *sh);
            idx = (*sh).next;
        }
    } else {
        decor_redraw_sh(buf, row1, row2, decor_sh_from_inline(decor.data.hl));
    };
}
pub unsafe extern "C" fn decor_redraw_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: DecorSignHighlight,
) {
    if sh.hl_id != 0
        || !sh.url.is_null()
        || sh.flags as ::core::ffi::c_int
            & (kSHIsSign as ::core::ffi::c_int
                | kSHSpellOn as ::core::ffi::c_int
                | kSHSpellOff as ::core::ffi::c_int
                | kSHConceal as ::core::ffi::c_int)
            != 0
    {
        if row2 >= row1 {
            redraw_buf_range_later(
                buf,
                row1 as linenr_T + 1 as linenr_T,
                row2 as linenr_T + 1 as linenr_T,
            );
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                changed_window_setting(wp);
            }
            wp = (*wp).w_next;
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        redraw_buf_line_later(buf, row1 as linenr_T + 1 as linenr_T, false_0 != 0);
    }
}
/// Stores `item` and answers the index other code refers to it by, reusing a
/// freed slot when there is one.
///
/// Invalidates every pointer [`decor_item`] has handed out.
pub fn decor_put_sh(item: DecorSignHighlight) -> uint32_t {
    let free = DECOR_FREELIST.get();
    if free != DECOR_ID_INVALID {
        // SAFETY: a freelist index is one this store handed out.
        DECOR_FREELIST.set(unsafe { (*decor_item(free)).next });
        DECOR_ITEMS.with_mut(|items| items[free as usize] = item);
        return free;
    }
    DECOR_ITEMS.with_mut(|items| {
        items.push(item);
        (items.len() - 1) as uint32_t
    })
}
pub unsafe extern "C" fn decor_put_vt(
    mut vt: DecorVirtText,
    mut next: *mut DecorVirtText,
) -> *mut DecorVirtText {
    let mut decor_alloc: *mut DecorVirtText =
        xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
    *decor_alloc = vt;
    (*decor_alloc).next = next;
    return decor_alloc;
}
pub unsafe extern "C" fn decor_sh_from_inline(
    mut item: DecorHighlightInline,
) -> DecorSignHighlight {
    '_c2rust_label: {
        if item.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"!(item.flags & kSHIsSign)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                166 as ::core::ffi::c_uint,
                b"DecorSignHighlight decor_sh_from_inline(DecorHighlightInline)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut conv: DecorSignHighlight = DecorSignHighlight {
        flags: item.flags,
        priority: item.priority,
        hl_id: item.hl_id,
        text: [item.conceal_char, 0],
        sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sign_add_id: 0,
        number_hl_id: 0 as ::core::ffi::c_int,
        line_hl_id: 0 as ::core::ffi::c_int,
        cursorline_hl_id: 0 as ::core::ffi::c_int,
        next: DECOR_ID_INVALID as uint32_t,
        url: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    return conv;
}
pub unsafe extern "C" fn buf_put_decor(
    mut buf: *mut buf_T,
    mut decor: DecorInline,
    mut row: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    if decor.ext as ::core::ffi::c_int != 0 && (row as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_item(idx);
            buf_put_decor_sh(buf, sh, row, row2);
            idx = (*sh).next;
        }
    }
}
pub unsafe extern "C" fn buf_decor_remove(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut free: bool,
) {
    decor_redraw(buf, row1, row2, col1, decor);
    if decor.ext as ::core::ffi::c_int != 0 && (row1 as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_item(idx);
            buf_remove_decor_sh(buf, row1, row2, sh);
            idx = (*sh).next;
        }
    }
    if free {
        decor_free(decor);
    }
}
pub unsafe extern "C" fn decor_free(mut decor: DecorInline) {
    if !decor.ext {
        return;
    }
    let mut vt: *mut DecorVirtText = decor.data.ext.vt;
    let mut idx: uint32_t = decor.data.ext.sh_idx;
    if (*decor_state.ptr()).running_decor_provider {
        while !vt.is_null() {
            if (*vt).next.is_null() {
                (*vt).next = to_free_virt.get();
                to_free_virt.set(decor.data.ext.vt);
                break;
            } else {
                vt = (*vt).next;
            }
        }
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_item(idx);
            if (*sh).next == DECOR_ID_INVALID as uint32_t {
                (*sh).next = to_free_sh.get();
                to_free_sh.set(decor.data.ext.sh_idx);
                break;
            } else {
                idx = (*sh).next;
            }
        }
    } else {
        decor_free_inner(vt, idx);
    };
}
unsafe extern "C" fn decor_free_inner(mut vt: *mut DecorVirtText, mut first_idx: uint32_t) {
    while !vt.is_null() {
        if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
            clear_virtlines(&raw mut (*vt).data.virt_lines);
        } else {
            clear_virttext(&raw mut (*vt).data.virt_text);
        }
        let mut tofree: *mut DecorVirtText = vt;
        vt = (*vt).next;
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    let mut idx: uint32_t = first_idx;
    while idx != DECOR_ID_INVALID as uint32_t {
        let mut sh: *mut DecorSignHighlight = decor_item(idx);
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).sign_name as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        (*sh).flags = 0 as uint16_t;
        if !(*sh).url.is_null() {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).url as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
        }
        if (*sh).next == DECOR_ID_INVALID as uint32_t {
            (*sh).next = DECOR_FREELIST.get();
            DECOR_FREELIST.set(first_idx);
            break;
        } else {
            idx = (*sh).next;
        }
    }
}
pub unsafe extern "C" fn decor_check_to_be_deleted() {
    '_c2rust_label: {
        if !(*decor_state.ptr()).running_decor_provider {
        } else {
            __assert_fail(
                b"!decor_state.running_decor_provider\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                330 as ::core::ffi::c_uint,
                b"void decor_check_to_be_deleted(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    decor_free_inner(to_free_virt.get(), to_free_sh.get());
    to_free_virt.set(::core::ptr::null_mut::<DecorVirtText>());
    to_free_sh.set(DECOR_ID_INVALID as uint32_t);
    (*decor_state.ptr()).win = ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn clear_virttext(mut text: *mut VirtText) {
    let mut i: size_t = 0 as size_t;
    while i < (*text).size {
        xfree((*(*text).items.offset(i as isize)).text as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree((*text).items as *mut ::core::ffi::c_void);
    (*text).capacity = 0 as size_t;
    (*text).size = (*text).capacity;
    (*text).items = ::core::ptr::null_mut::<VirtTextChunk>();
    *text = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
}
pub unsafe extern "C" fn clear_virtlines(mut lines: *mut VirtLines) {
    let mut i: size_t = 0 as size_t;
    while i < (*lines).size {
        clear_virttext(&raw mut (*(*lines).items.offset(i as isize)).line);
        i = i.wrapping_add(1);
    }
    xfree((*lines).items as *mut ::core::ffi::c_void);
    (*lines).capacity = 0 as size_t;
    (*lines).size = (*lines).capacity;
    (*lines).items = ::core::ptr::null_mut::<virt_line>();
    *lines = VirtLines {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<virt_line>(),
    };
}
pub unsafe extern "C" fn decor_check_invalid_glyphs() {
    let mut i: size_t = 0 as size_t;
    while i < decor_item_count() {
        let mut it: *mut DecorSignHighlight = decor_item(i as uint32_t);
        let mut width: ::core::ffi::c_int =
            if (*it).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                SIGN_WIDTH as ::core::ffi::c_int
            } else if (*it).flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < width {
            if schar_high((*it).text[j as usize]) {
                (*it).text[j as usize] =
                    schar_from_char(schar_get_first_codepoint((*it).text[j as usize]));
            }
            j += 1;
        }
        i = i.wrapping_add(1);
    }
}
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_decor_virt(mut mark: MTKey) -> *mut DecorVirtText {
    return if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0 {
        mark.decor_data.ext.vt
    } else {
        ::core::ptr::null_mut::<DecorVirtText>()
    };
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
