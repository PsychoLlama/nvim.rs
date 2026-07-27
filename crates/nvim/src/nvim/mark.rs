//! Setting marks and jumping to them.
//!
//! A named file mark is valid when its `lnum` is non-zero. A non-zero `fnum`
//! means it names a live buffer; otherwise it came from the shada file and
//! `namedfm[n].fname` is the file name. The global set is `'A`-`'Z`, which
//! the user sets, plus `'0`-`'9`, which are written when shada is saved.
//!
//! The stores split by concern: [`adjust`] rewrites every mark's line and
//! column when the buffer's lines move (`mark_adjust` is on the path of
//! every `:d`, `:m` and undo), [`jumplist`] owns the jumplist and the
//! changelist, [`lookup`] resolves a mark's name and moves the cursor to it,
//! [`show`] is `:marks`/`:delmarks`, [`shada`] is the iterator surface the
//! shada writer walks, and [`builtins`] is `getmarklist()`.

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{aucmd_defer, has_event};
use crate::src::nvim::buffer::{bt_prompt, buflist_findnr, buflist_new};
use crate::src::nvim::charset::{ptr2cells, vim_isprintc};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    IObuff, NameBuff, curbuf, curtab, curwin, e_markinval, e_marknotset, e_umark, firstwin, namedfm,
};
use crate::src::nvim::mbyte::{utf_head_off, utf_ptr2char};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xfree, xstrlcpy};
use crate::src::nvim::r#move::set_topline;
use crate::src::nvim::os::env::expand_env;
use crate::src::nvim::os::fs::os_dirname;
use crate::src::nvim::os::libc::{gettext, memmove};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{path_fnamecmp, path_shorten_fname, vim_ispathsep_nocolon};
use crate::src::nvim::plines::linetabsize_eol;
use crate::src::nvim::tag::tagstack_clear_entry;
pub use crate::src::nvim::types::*;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

/// `pos.h`'s ordering on a buffer position.
fn lt(a: pos_T, b: pos_T) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    }
}

fn equalpos(a: pos_T, b: pos_T) -> bool {
    a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd
}

fn ascii_isdigit(c: c_int) -> bool {
    c >= '0' as c_int && c <= '9' as c_int
}

mod adjust;
mod builtins;
mod jumplist;
mod lookup;
mod shada;
mod show;

pub use adjust::{mark_adjust, mark_adjust_buf, mark_adjust_nofold, mark_col_adjust};
pub use builtins::{get_buf_local_marks, get_global_marks};
pub use jumplist::{
    checkpcmark, cleanup_jumplist, copy_jumplist, ex_changes, ex_clearjumps, ex_jumps,
    free_jumplist, get_changelist, get_jumplist, mark_jumplist_forget_file, mark_jumplist_iter,
    setpcmark,
};
pub use lookup::{
    getnextmark, mark_get, mark_get_global, mark_get_local, mark_get_motion, mark_get_visual,
    mark_move_to,
};
pub use shada::{mark_buffer_iter, mark_global_iter, mark_set_global, mark_set_local};
pub use show::{ex_delmarks, ex_marks, fm_getname};

pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NUL: c_int = 0;
pub const TAB: c_int = '\t' as c_int;
pub const MAXCOL: c_uint = 2147483647;
pub const MAXLNUM: c_uint = 2147483647;
pub const MAXPATHL: c_int = 4096;
pub const IOSIZE: c_int = 1024 + 1;
pub const FORWARD: Direction = 1;
pub const BACKWARD: Direction = -1;
pub const HLF_D: c_uint = 5;
pub const BL_WHITE: c_uint = 1;
pub const BL_FIX: c_uint = 4;
pub const GETF_SETMARK: getf_values = 1;
pub const AUGROUP_ALL: c_int = -3;
pub const EVENT_MARKSET: auto_event = 82;
pub const CMOD_KEEPJUMPS: c_int = 1024;
pub const CMOD_LOCKMARKS: c_int = 2048;
pub const BUF_HAS_QF_ENTRY: c_int = 1;
pub const BUF_HAS_LL_ENTRY: c_int = 2;
pub const kOptJopFlagStack: c_uint = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kMTCharWise: MotionType = 0;
pub const kObjectTypeNil: ObjectType = 0;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeDict: ObjectType = 6;
pub const ARRAY_DICT_INIT: Dict = Dict {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};

/// How `mark_get` is allowed to resolve a mark's name.
pub const kMarkBufLocal: MarkGet = 0;
pub const kMarkAllNoResolve: MarkGet = 2;

/// What `mark_move_to` should do beyond putting the cursor there.
pub const kMarkBeginLine: MarkMove = 1;
pub const kMarkContext: MarkMove = 2;
pub const kMarkSetView: MarkMove = 8;
pub const kMarkJumpList: MarkMove = 16;

/// What `mark_move_to` did.
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkChangedCol: MarkMoveRes = 8;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkChangedCursor: MarkMoveRes = 32;

/// Which of the mark stores `mark_adjust_buf` should touch.
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;

/// `'a`..`'z` are the buffer-local marks; `'0`..`'9` extend the global set
/// with the shada file's previously-edited-file marks.
pub const NMARKS: c_int = 'z' as c_int - 'a' as c_int + 1;
pub const EXTRA_MARKS: c_int = '9' as c_int - '0' as c_int + 1;
pub const NGLOBALMARKS: c_int = NMARKS + EXTRA_MARKS;
/// The highest byte a buffer-local mark name may be.
pub const NMARK_LOCAL_MAX: c_int = 126;
/// How many positions a window's jumplist remembers.
pub const JUMPLISTSIZE: c_int = 100;

unsafe extern "C" {
    fn qf_mark_adjust(
        buf: *mut buf_T,
        wp: *mut win_T,
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
    ) -> bool;
}
/// Set named mark "c" at current cursor position.
/// Returns OK on success, FAIL if bad name given.
pub unsafe extern "C" fn setmark(mut c: c_int) -> c_int {
    let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
    return setmark_pos(
        c,
        &raw mut (*curwin.get()).w_cursor,
        (*curbuf.get()).handle as c_int,
        &raw mut view,
    );
}
/// Free fmark_T item
pub unsafe extern "C" fn free_fmark(mut fm: fmark_T) {
    xfree(fm.additional_data as *mut c_void);
}
/// Free xfmark_T item
pub unsafe extern "C" fn free_xfmark(mut fm: xfmark_T) {
    xfree(fm.fname as *mut c_void);
    free_fmark(fm.fmark);
}
/// Free and clear fmark_T item.
///
/// Does not trigger "MarkSet" event.
pub unsafe extern "C" fn clear_fmark(fm: *mut fmark_T, timestamp: Timestamp) {
    free_fmark(*fm);
    *fm = fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as c_int,
            skipcol: 0,
        },
        additional_data: ptr::null_mut(),
    };
    (*fm).timestamp = timestamp;
}
/// Schedules "MarkSet" event.
///
/// `c` — The name of the mark, e.g., 'a'.
/// `pos` — Position of the mark in the buffer.
/// `buf` — The buffer of the mark.
unsafe extern "C" fn do_markset_autocmd(mut c: c_char, mut pos: *mut pos_T, mut buf: *mut buf_T) {
    if !has_event(EVENT_MARKSET) {
        return;
    }
    let mut data: Dict = ARRAY_DICT_INIT;
    let mut data__items: [KeyValuePair; 3] = [KeyValuePair {
        key: String_0 {
            data: ptr::null_mut(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: object_data { boolean: false },
        },
    }; 3];
    data.capacity = 3;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let mut mark_str: [c_char; 2] = [c, '\0' as c_char];
    let c2rust_fresh0 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.add(c2rust_fresh0) = key_value_pair {
        key: cstr_as_string(c"name".as_ptr()),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: String_0 {
                    data: &raw mut mark_str as *mut c_char,
                    size: 1,
                },
            },
        },
    };
    let c2rust_fresh1 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.add(c2rust_fresh1) = key_value_pair {
        key: cstr_as_string(c"line".as_ptr()),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: (*pos).lnum as Integer,
            },
        },
    };
    let c2rust_fresh2 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.add(c2rust_fresh2) = key_value_pair {
        key: cstr_as_string(c"col".as_ptr()),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: (*pos).col as Integer,
            },
        },
    };
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: object_data { dict: data },
    };
    aucmd_defer(
        EVENT_MARKSET,
        &raw mut mark_str as *mut c_char,
        ptr::null_mut(),
        AUGROUP_ALL as c_int,
        buf,
        ptr::null_mut(),
        &raw mut c2rust_lvalue,
    );
}
/// Set named mark "c" to position "pos".
/// When "c" is upper case use file "fnum".
/// Returns OK on success, FAIL if bad name given.
pub unsafe extern "C" fn setmark_pos(
    mut c: c_int,
    mut pos: *mut pos_T,
    mut fnum: c_int,
    mut view_pt: *mut fmarkv_T,
) -> c_int {
    let mut i: c_int = 0;
    let mut view: fmarkv_T = if !view_pt.is_null() {
        *view_pt
    } else {
        fmarkv_T {
            topline_offset: MAXLNUM as c_int,
            skipcol: 0,
        }
    };
    if c < 0 {
        return FAIL;
    }
    if c == '\'' as c_int || c == '`' as c_int {
        if pos == &raw mut (*curwin.get()).w_cursor {
            setpcmark();
            (*curwin.get()).w_prev_pcmark = (*curwin.get()).w_pcmark;
        } else {
            (*curwin.get()).w_pcmark = *pos;
        }
        return OK;
    }
    let mut buf: *mut buf_T = buflist_findnr(fnum);
    if buf.is_null() {
        return FAIL;
    }
    if c == '"' as c_int {
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_last_cursor;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = *pos;
        (*fmarkp__).fnum = (*buf).handle as c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = view;
        (*fmarkp__).additional_data = ptr::null_mut();
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    if c == '[' as c_int {
        (*buf).b_op_start = *pos;
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    if c == ']' as c_int {
        (*buf).b_op_end = *pos;
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    if c == '<' as c_int || c == '>' as c_int {
        if c == '<' as c_int {
            (*buf).b_visual.vi_start = *pos;
        } else {
            (*buf).b_visual.vi_end = *pos;
        }
        if (*buf).b_visual.vi_mode == NUL {
            (*buf).b_visual.vi_mode = 'v' as c_int;
        }
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    if c == ':' as c_int && bt_prompt(buf) {
        let fmarkp____0: *mut fmark_T = &raw mut (*buf).b_prompt_start;
        free_fmark(*fmarkp____0);
        let fmarkp___0: *mut fmark_T = fmarkp____0;
        (*fmarkp___0).mark = *pos;
        (*fmarkp___0).fnum = (*buf).handle as c_int;
        (*fmarkp___0).timestamp = os_time();
        (*fmarkp___0).view = view;
        (*fmarkp___0).additional_data = ptr::null_mut();
        return OK;
    }
    if c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint {
        i = c - 'a' as c_int;
        let fmarkp____1: *mut fmark_T =
            (&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize);
        free_fmark(*fmarkp____1);
        let fmarkp___1: *mut fmark_T = fmarkp____1;
        (*fmarkp___1).mark = *pos;
        (*fmarkp___1).fnum = fnum;
        (*fmarkp___1).timestamp = os_time();
        (*fmarkp___1).view = view;
        (*fmarkp___1).additional_data = ptr::null_mut();
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    if c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint || ascii_isdigit(c) {
        if ascii_isdigit(c) {
            i = c - '0' as c_int + NMARKS;
        } else {
            i = c - 'A' as c_int;
        }
        let xfmarkp__: *mut xfmark_T = (namedfm.ptr() as *mut xfmark_T).offset(i as isize);
        free_xfmark(*xfmarkp__);
        (*xfmarkp__).fname = ptr::null_mut();
        let fmarkp___2: *mut fmark_T = &raw mut (*xfmarkp__).fmark;
        (*fmarkp___2).mark = *pos;
        (*fmarkp___2).fnum = fnum;
        (*fmarkp___2).timestamp = os_time();
        (*fmarkp___2).view = view;
        (*fmarkp___2).additional_data = ptr::null_mut();
        do_markset_autocmd(c as c_char, pos, buf);
        return OK;
    }
    return FAIL;
}
/// Delete every entry referring to file "fnum" from both the jumplist and the
/// tag stack.
pub unsafe extern "C" fn mark_forget_file(mut wp: *mut win_T, mut fnum: c_int) {
    mark_jumplist_forget_file(wp, fnum);
    let mut i: c_int = (*wp).w_tagstacklen - 1;
    while i >= 0 {
        if (*wp).w_tagstack[i as usize].fmark.fnum == fnum {
            tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
            if (*wp).w_tagstackidx > i {
                (*wp).w_tagstackidx -= 1;
            }
            (*wp).w_tagstacklen -= 1;
            memmove(
                (&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize) as *mut c_void,
                (&raw mut (*wp).w_tagstack as *mut taggy_T).offset((i + 1) as isize)
                    as *const c_void,
                (((*wp).w_tagstacklen - i) as size_t).wrapping_mul(size_of::<taggy_T>()),
            );
        }
        i -= 1;
    }
}
/// Wrap a pos_T into an fmark_T, used to abstract marks handling.
///
/// Pass an fmp if multiple c
/// @note  view fields are set to 0.
/// `buf` — for fmark->fnum.
/// `pos` — for fmark->mark.
/// `fmp` — pointer to save the mark.
///
/// @return[static] Mark with the given information.
pub unsafe extern "C" fn pos_to_mark(
    mut buf: *mut buf_T,
    mut fmp: *mut fmark_T,
    mut pos: pos_T,
) -> *mut fmark_T {
    static fms: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as c_int,
            skipcol: 0,
        },
        additional_data: ptr::null_mut(),
    });
    let mut fm: *mut fmark_T = if fmp.is_null() { fms.ptr() } else { fmp };
    (*fm).fnum = (*buf).handle as c_int;
    (*fm).mark = pos;
    return fm;
}
/// Restore the mark view.
/// By remembering the offset between topline and mark lnum at the time of
/// definition, this function restores the "view".
/// @note  Assumes the mark has been checked, is valid.
/// `fm` — the named mark.
pub unsafe extern "C" fn mark_view_restore(mut fm: *mut fmark_T) {
    if !fm.is_null() && (*fm).view.topline_offset >= 0 {
        let mut topline: linenr_T = (*fm).mark.lnum - (*fm).view.topline_offset;
        if topline >= 1 {
            set_topline(curwin.get(), topline);
            (*curwin.get()).w_skipcol = (if (*fm).view.skipcol > 0
                && !hasFolding(curwin.get(), topline, ptr::null_mut(), ptr::null_mut())
                && (*fm).view.skipcol < linetabsize_eol(curwin.get(), topline)
            {
                (*fm).view.skipcol as c_int
            } else {
                0
            }) as colnr_T;
        }
    }
}
pub unsafe extern "C" fn mark_view_make(mut wp: *const win_T, mut pos: pos_T) -> fmarkv_T {
    return fmarkv_T {
        topline_offset: pos.lnum - (*wp).w_topline,
        skipcol: (*wp).w_skipcol,
    };
}
/// For an xtended filemark: set the fnum from the fname.
/// This is used for marks obtained from the .shada file.  It's postponed
/// until the mark is used to avoid a long startup delay.
unsafe extern "C" fn fname2fnum(mut fm: *mut xfmark_T) {
    if (*fm).fname.is_null() {
        return;
    }
    if *(*fm).fname.offset(0) as c_int == '~' as c_int
        && vim_ispathsep_nocolon(*(*fm).fname.offset(1) as c_int) as c_int != 0
    {
        let mut len: size_t = expand_env(
            c"~/".as_ptr() as *mut c_char,
            NameBuff.ptr() as *mut c_char,
            MAXPATHL,
        );
        xstrlcpy(
            (NameBuff.ptr() as *mut c_char).add(len),
            (*fm).fname.offset(2),
            (MAXPATHL as size_t).wrapping_sub(len),
        );
    } else {
        xstrlcpy(
            NameBuff.ptr() as *mut c_char,
            (*fm).fname,
            MAXPATHL as size_t,
        );
    }
    os_dirname(IObuff.ptr() as *mut c_char, IOSIZE as size_t);
    let mut p: *mut c_char =
        path_shorten_fname(NameBuff.ptr() as *mut c_char, IObuff.ptr() as *mut c_char);
    buflist_new(NameBuff.ptr() as *mut c_char, p, 1, 0);
}
/// Check all file marks for a name that matches the file name in buf.
/// May replace the name with an fnum.
/// Used for marks that come from the .shada file.
pub unsafe extern "C" fn fmarks_check_names(mut buf: *mut buf_T) {
    let mut name: *mut c_char = (*buf).b_ffname;
    if (*buf).b_ffname.is_null() {
        return;
    }
    let mut i: c_int = 0;
    while i < NGLOBALMARKS {
        fmarks_check_one(
            (namedfm.ptr() as *mut xfmark_T).offset(i as isize),
            name,
            buf,
        );
        i += 1;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut i_0: c_int = 0;
        while i_0 < (*wp).w_jumplistlen {
            fmarks_check_one(
                (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i_0 as isize),
                name,
                buf,
            );
            i_0 += 1;
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn fmarks_check_one(
    mut fm: *mut xfmark_T,
    mut name: *mut c_char,
    mut buf: *mut buf_T,
) {
    if (*fm).fmark.fnum == 0 && !(*fm).fname.is_null() && path_fnamecmp(name, (*fm).fname) == 0 {
        (*fm).fmark.fnum = (*buf).handle as c_int;
        let mut ptr_: *mut *mut c_void = &raw mut (*fm).fname as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = ptr::null_mut();
        let _ = *ptr_;
    }
}
/// Check the position in @a fm is valid.
///
/// Checks for:
/// - NULL raising unknown mark error.
/// - Line number <= 0 raising mark not set.
/// - Line number > buffer line count, raising invalid mark.
///
/// `fm[in]` — File mark to check.
/// `errormsg[out]` — Error message, if any.
///
/// Returns true if the mark passes all the above checks, else false.
pub unsafe extern "C" fn mark_check(
    mut fm: *mut fmark_T,
    mut errormsg: *mut *const c_char,
) -> bool {
    if fm.is_null() {
        *errormsg = gettext(&raw const e_umark as *const c_char);
        return false;
    } else if (*fm).mark.lnum <= 0 {
        if (*fm).mark.lnum == 0 {
            *errormsg = gettext(&raw const e_marknotset as *const c_char);
        }
        return false;
    }
    if (*fm).fnum == (*curbuf.get()).handle && !mark_check_line_bounds(curbuf.get(), fm, errormsg) {
        return false;
    }
    return true;
}
/// Check if a mark line number is greater than the buffer line count, and set e_markinval.
///
/// @note  Should be done after the buffer is loaded into memory.
/// `buf` — Buffer where the mark is set.
/// `fm` — Mark to check.
/// `errormsg[out]` — Error message, if any.
/// Returns true if below line count else false.
pub unsafe extern "C" fn mark_check_line_bounds(
    mut buf: *mut buf_T,
    mut fm: *mut fmark_T,
    mut errormsg: *mut *const c_char,
) -> bool {
    if !buf.is_null() && (*fm).mark.lnum > (*buf).b_ml.ml_line_count {
        *errormsg = gettext(&raw const e_markinval as *const c_char);
        return false;
    }
    return true;
}
/// Clear all marks and change list in the given buffer
///
/// Used mainly when trashing the entire buffer during ":e" type commands.
///
/// Does not trigger "MarkSet" event.
///
/// `buf` — Buffer to clear marks in.
pub unsafe extern "C" fn clrallmarks(buf: *mut buf_T, timestamp: Timestamp) {
    let mut i: size_t = 0;
    while i < NMARKS as size_t {
        clear_fmark((&raw mut (*buf).b_namedm as *mut fmark_T).add(i), timestamp);
        i = i.wrapping_add(1);
    }
    clear_fmark(&raw mut (*buf).b_last_cursor, timestamp);
    (*buf).b_last_cursor.mark.lnum = 1;
    clear_fmark(&raw mut (*buf).b_last_insert, timestamp);
    clear_fmark(&raw mut (*buf).b_last_change, timestamp);
    (*buf).b_op_start.lnum = 0;
    (*buf).b_op_end.lnum = 0;
    let mut i_0: c_int = 0;
    while i_0 < (*buf).b_changelistlen {
        clear_fmark(
            (&raw mut (*buf).b_changelist as *mut fmark_T).offset(i_0 as isize),
            timestamp,
        );
        i_0 += 1;
    }
    (*buf).b_changelistlen = 0;
}
pub unsafe extern "C" fn set_last_cursor(mut win: *mut win_T) {
    if !(*win).w_buffer.is_null() {
        let fmarkp___: *mut fmark_T = &raw mut (*(*win).w_buffer).b_last_cursor;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = (*win).w_cursor;
        (*fmarkp__).fnum = 0;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = fmarkv_T {
            topline_offset: MAXLNUM as c_int,
            skipcol: 0,
        };
        (*fmarkp__).additional_data = ptr::null_mut();
    }
}
/// Adjust position to point to the first byte of a multi-byte character
///
/// If it points to a tail byte it is move backwards to the head byte.
///
/// `buf` — Buffer to adjust position in.
/// `lp` — Position to adjust.
pub unsafe extern "C" fn mark_mb_adjustpos(mut buf: *mut buf_T, mut lp: *mut pos_T) {
    if (*lp).col > 0 || (*lp).coladd > 1 {
        let p: *const c_char = ml_get_buf(buf, (*lp).lnum);
        if *p as c_int == NUL || ml_get_buf_len(buf, (*lp).lnum) < (*lp).col {
            (*lp).col = 0;
        } else {
            (*lp).col -= utf_head_off(p, p.offset((*lp).col as isize));
        }
        if (*lp).coladd == 1
            && *p.offset((*lp).col as isize) as c_int != TAB
            && vim_isprintc(utf_ptr2char(p.offset((*lp).col as isize)))
            && ptr2cells(p.offset((*lp).col as isize)) > 1
        {
            (*lp).coladd = 0;
        }
    }
}
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
