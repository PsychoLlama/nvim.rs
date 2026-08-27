//! Setting marks and jumping to them.
//!
//! A named file mark is valid when its `lnum` is non-zero. A non-zero `fnum`
//! means it names a live buffer; otherwise it came from the shada file and
//! `namedfm[n].fname` is the file name. The global set is `'A`-`'Z`, which
//! the user sets, plus `'0`-`'9`, which are written when shada is saved.
//!
//! The stores split by concern: [`store`] is the handle layer every other
//! module here reaches a record through, [`adjust`] rewrites every mark's line
//! and column when the buffer's lines move (`mark_adjust` is on the path of
//! every `:d`, `:m` and undo), [`jumplist`] owns the jumplist and the
//! changelist, [`lookup`] resolves a mark's name and moves the cursor to it,
//! [`show`] is `:marks`/`:delmarks`, [`shada`] is the iterator surface the
//! shada writer walks, and [`builtins`] is `getmarklist()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::cstr_as_string;
use crate::ascii::{ascii_isdigit, ascii_islower, ascii_isupper};
use crate::autocmd::{EVENT_MARKSET, aucmd_defer, has_event};
use crate::buffer::{bt_prompt, buflist_new, find_buf};
use crate::charset::{ptr2cells, vim_isprintc};
use crate::ex_docmd::ex_msg;
use crate::fold::has_folding;
use crate::global_cell::GlobalCell;
use crate::main::{e_markinval, e_marknotset, e_umark};
use crate::mbyte::{utf_head_off, utf_ptr2char};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{xfree, xstrlcpy};
use crate::r#move::set_topline;
use crate::options::kOptJopFlagStack;
use crate::os::cshim::memmove;
use crate::os::env::expand_env;
use crate::os::fs::os_dirname;
use crate::path::{path_fnamecmp, path_shorten_fname, vim_ispathsep_nocolon};
use crate::plines::linetabsize_eol;
use crate::tag::tagstack_clear_entry;
use crate::types::*;
use crate::winlayer::{Buf, Win, windows};
use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CString;

mod adjust;
mod builtins;
mod jumplist;
mod lookup;
mod shada;
mod show;
mod store;

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
pub(crate) use shada::global_mark_timestamp;
pub use shada::{mark_buffer_iter, mark_global_iter, mark_set_global, mark_set_local};
pub use show::{ex_delmarks, ex_marks, fm_getname};

use store::{Fmark, GlobalMarks, NO_VIEW, NUL_BYTE, Xfmark, mark_name};

pub const TAB: c_int = '\t' as c_int;
pub const GETF_SETMARK: getf_values = 1;
pub const AUGROUP_ALL: c_int = -3;
pub const BUF_HAS_QF_ENTRY: c_int = 1;
pub const BUF_HAS_LL_ENTRY: c_int = 2;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMTCharWise: MotionType = 0;
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

use crate::quickfix::qf_mark_adjust;

/// Set named mark "c" at current cursor position.
/// Returns OK on success, FAIL if bad name given.
///
/// # Safety
/// The editor's globals must be live, which they are from startup to exit.
pub unsafe fn setmark(c: c_int) -> c_int {
    // SAFETY: `curwin`/`curbuf` are live from startup to exit.
    let (win, buf) = unsafe { (Win::current(), Buf::current()) };
    let mut view = mark_view_make_at(win, win.w_cursor);
    // SAFETY: the cursor and the view live on the stack for the call, and
    // `curbuf`'s handle names a live buffer.
    unsafe {
        setmark_pos(
            c,
            &raw mut (*win.raw()).w_cursor,
            buf.handle as c_int,
            &raw mut view,
        )
    }
}

/// Free fmark_T item
///
/// # Safety
/// `fm.additional_data` must be an owned allocation or null, and must not be
/// reachable from anywhere else afterwards.
pub unsafe fn free_fmark(fm: fmark_T) {
    // SAFETY: forwarded from the caller.
    unsafe { xfree(fm.additional_data.cast()) };
}

/// Free xfmark_T item
///
/// # Safety
/// As [`free_fmark`], plus `fm.fname` must be an owned allocation or null.
pub unsafe fn free_xfmark(fm: xfmark_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        xfree(fm.fname.cast());
        free_fmark(fm.fmark);
    }
}

/// Free and clear fmark_T item.
///
/// Does not trigger "MarkSet" event.
///
/// # Safety
/// `fm` must point at a live, writable `fmark_T` whose `additional_data` is
/// this store's to free.
pub unsafe fn clear_fmark(fm: *mut fmark_T, timestamp: Timestamp) {
    // SAFETY: forwarded from the caller.
    unsafe { Fmark::new(fm) }.clear(timestamp);
}

/// Schedules "MarkSet" event.
///
/// `c` — The name of the mark, e.g., 'a'.
/// `pos` — Position of the mark in the buffer.
/// `buf` — The buffer of the mark.
///
/// # Safety
/// `pos` must point at a live position and `buf` at a live buffer.
unsafe fn do_markset_autocmd(c: c_char, pos: *mut pos_T, buf: *mut buf_T) {
    // SAFETY: the autocommand tables are the editor's own, live from startup.
    if !has_event(EVENT_MARKSET) {
        return;
    }
    // SAFETY: the caller promised a live position.
    let pos = unsafe { *pos };
    let mut mark_str: [c_char; 2] = [c, NUL_BYTE];
    // SAFETY: the three keys are `'static` C strings, `mark_str` and `items`
    // outlive the `aucmd_defer` call, and `aucmd_defer` copies the payload
    // before it returns. `buf` is the caller's live buffer.
    unsafe {
        let mut items: [KeyValuePair; 3] = [
            key_value_pair {
                key: cstr_as_string(c"name".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: object_data {
                        string: String_0::from_raw_parts(mark_str.as_mut_ptr(), 1),
                    },
                },
            },
            key_value_pair {
                key: cstr_as_string(c"line".as_ptr()),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: object_data {
                        integer: Integer::from(pos.lnum),
                    },
                },
            },
            key_value_pair {
                key: cstr_as_string(c"col".as_ptr()),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: object_data {
                        integer: Integer::from(pos.col),
                    },
                },
            },
        ];
        let mut payload: Object = object {
            type_0: kObjectTypeDict,
            data: object_data {
                dict: Dict {
                    size: items.len(),
                    capacity: items.len(),
                    items: items.as_mut_ptr(),
                },
            },
        };
        aucmd_defer(
            EVENT_MARKSET,
            mark_str.as_mut_ptr(),
            ptr::null_mut(),
            AUGROUP_ALL,
            Buf::new(buf),
            ptr::null_mut(),
            &raw mut payload,
        );
    }
}

/// Set named mark "c" to position "pos".
/// When "c" is upper case use file "fnum".
/// Returns OK on success, FAIL if bad name given.
///
/// # Safety
/// `pos` must point at a live position; `view_pt` must be null or point at a
/// live `fmarkv_T`.
pub unsafe fn setmark_pos(c: c_int, pos: *mut pos_T, fnum: c_int, view_pt: *mut fmarkv_T) -> c_int {
    // SAFETY: the caller promised a live position, and a live view or null.
    let (at, view) = unsafe { (*pos, if view_pt.is_null() { NO_VIEW } else { *view_pt }) };
    if c < 0 {
        return FAIL;
    }
    // `''` and `` '` `` are the same slot, and it is the window's rather than
    // the buffer's: setting it from the cursor pushes a jumplist entry, while
    // setting it from anywhere else just moves it.
    if c == '\'' as c_int || c == '`' as c_int {
        // SAFETY: `curwin` is live from startup to exit.
        let mut win = unsafe { Win::current() };
        if ptr::eq(pos, &raw const win.w_cursor) {
            // SAFETY: the editor's globals are live.
            setpcmark();
            win.w_prev_pcmark = win.w_pcmark;
        } else {
            win.w_pcmark = at;
        }
        return OK;
    }
    let Some(mut buf) = find_buf(fnum) else {
        return FAIL;
    };
    let handle = buf.handle as c_int;

    // The tick family and the Visual range are not adjusted and not saved, so
    // they are stored raw rather than through `Fmark::place`.
    if c == '[' as c_int {
        buf.b_op_start = at;
    } else if c == ']' as c_int {
        buf.b_op_end = at;
    } else if c == '<' as c_int || c == '>' as c_int {
        if c == '<' as c_int {
            buf.b_visual.vi_start = at;
        } else {
            buf.b_visual.vi_end = at;
        }
        if buf.b_visual.vi_mode == NUL {
            buf.b_visual.vi_mode = 'v' as c_int;
        }
    } else if c == '"' as c_int {
        buf.last_cursor().replace(at, handle, view);
    } else if c == ':' as c_int {
        // SAFETY: `buf` is live.
        if !unsafe { bt_prompt(buf.raw()) } {
            return FAIL;
        }
        buf.prompt_start().replace(at, handle, view);
        // The prompt mark is the one store that does NOT announce itself:
        // it moves on every prompt redraw, and a MarkSet per keystroke is
        // not what the event is for.
        return OK;
    } else if ascii_islower(c) {
        // A buffer-local mark keeps the *caller's* `fnum` rather than the
        // buffer's own handle. The two agree everywhere but `nvim_buf_set_mark`.
        buf.named_mark(c - 'a' as c_int).replace(at, fnum, view);
    } else if ascii_isupper(c) || ascii_isdigit(c) {
        GlobalMarks::at(lookup::mark_global_index(mark_name(c))).replace(at, fnum, view);
    } else {
        return FAIL;
    }
    // SAFETY: `pos` and `buf` are the caller's, both live.
    unsafe { do_markset_autocmd(mark_name(c), pos, buf.raw()) };
    OK
}

/// Delete every entry referring to file "fnum" from both the jumplist and the
/// tag stack.
///
/// # Safety
/// `wp` must be a live window.
pub unsafe fn mark_forget_file(wp: *mut win_T, fnum: c_int) {
    // SAFETY: the caller promised a live window.
    let mut wp = unsafe { Win::new(wp) };
    unsafe { mark_jumplist_forget_file(wp.raw(), fnum) };
    // Backwards, so removing an entry cannot skip the one after it.
    for i in (0..wp.w_tagstacklen).rev() {
        if wp.tag_mark(i).fnum() != fnum {
            continue;
        }
        let at = usize::try_from(i).expect("tag stack index in range");
        // SAFETY: `i` is inside the tag stack, whose entries are live.
        unsafe { tagstack_clear_entry(&mut (*wp.raw()).w_tagstack[at]) };
        if wp.w_tagstackidx > i {
            wp.w_tagstackidx -= 1;
        }
        wp.w_tagstacklen -= 1;
        // SAFETY: source and destination are inside `[taggy_T; 20]` and the
        // length is what is left above `i`, so the move stays in the array.
        unsafe {
            let stack = (&raw mut (*wp.raw()).w_tagstack).cast::<taggy_T>();
            memmove(
                stack.offset(i as isize).cast(),
                stack.offset(i as isize + 1).cast(),
                size_t::try_from(wp.w_tagstacklen - i)
                    .unwrap_or(0)
                    .wrapping_mul(size_of::<taggy_T>()),
            );
        }
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
///
/// # Safety
/// `buf` must be a live buffer; `fmp` must be null or point at a live,
/// writable `fmark_T`. A null `fmp` answers a shared static, so the result is
/// only valid until the next such call.
pub unsafe fn pos_to_mark(buf: *mut buf_T, fmp: *mut fmark_T, pos: pos_T) -> *mut fmark_T {
    /// The scratch record the `fmp`-less callers share. `mark_get_local`
    /// hands its address straight back to the caller, which is why a second
    /// motion-mark lookup invalidates the first.
    static SCRATCH: GlobalCell<fmark_T> = GlobalCell::new(store::UNSET_FMARK);

    // SAFETY: `fmp` is the caller's live record, or the shared static.
    let fm = unsafe { Fmark::new(if fmp.is_null() { SCRATCH.ptr() } else { fmp }) };
    // SAFETY: the caller promised a live buffer.
    fm.set_fnum(unsafe { Buf::new(buf) }.handle as c_int);
    fm.set_pos(pos);
    fm.raw()
}

/// Restore the mark view.
/// By remembering the offset between topline and mark lnum at the time of
/// definition, this function restores the "view".
/// @note  Assumes the mark has been checked, is valid.
/// `fm` — the named mark.
///
/// # Safety
/// `fm` must be null or point at a live `fmark_T`.
pub unsafe fn mark_view_restore(fmp: *mut fmark_T) {
    if fmp.is_null() {
        return;
    }
    // SAFETY: the caller promised a live record.
    let fm = unsafe { Fmark::new(fmp) }.read();
    if fm.view.topline_offset < 0 {
        return;
    }
    let topline = fm.mark.lnum - fm.view.topline_offset;
    if topline < 1 {
        return;
    }
    // SAFETY: `curwin` is live from startup to exit.
    let mut win = unsafe { Win::current() };
    // SAFETY: as above.
    set_topline(win, topline);
    // A remembered `skipcol` is dropped when the line it names is now folded
    // away or has become too short to reach — restoring it would scroll the
    // window sideways past the end of the line.
    // `skipcol` is re-read after the scroll rather than snapshotted with
    // `topline_offset` above, because that is the order upstream reads the
    // two fields in.
    // SAFETY: `win` is live and the two calls read only it and the buffer.
    let skipcol = unsafe { Fmark::new(fmp) }.read().view.skipcol;
    // SAFETY: as above.
    let keep = unsafe {
        skipcol > 0
            && !has_folding(win, topline, None, None)
            && skipcol < linetabsize_eol(win, topline)
    };
    win.w_skipcol = if keep { skipcol } else { 0 };
}

/// # Safety
/// `wp` must be a live window.
pub unsafe fn mark_view_make(wp: *const win_T, pos: pos_T) -> fmarkv_T {
    // SAFETY: the caller promised a live window.
    mark_view_make_at(unsafe { Win::new(wp.cast_mut()) }, pos)
}

/// The view [`mark_view_make`] records: how far below the window's topline the
/// position sits, and where the window was scrolled to sideways.
fn mark_view_make_at(wp: Win, pos: pos_T) -> fmarkv_T {
    fmarkv_T {
        topline_offset: pos.lnum - wp.w_topline,
        skipcol: wp.w_skipcol,
    }
}

/// For an xtended filemark: set the fnum from the fname.
/// This is used for marks obtained from the .shada file.  It's postponed
/// until the mark is used to avoid a long startup delay.
///
/// # Safety
/// `fm` must point at a live `xfmark_T` whose `fname`, if set, is a
/// NUL-terminated string.
pub(super) unsafe fn fname2fnum(fm: *mut xfmark_T) {
    // SAFETY: the caller promised a live record.
    let fm = unsafe { Xfmark::new(fm) };
    let fname = fm.fname();
    if fname.is_null() {
        return;
    }
    let mut name = [0 as c_char; MAXPATHL as usize];
    let mut dir = [0 as c_char; IOSIZE as usize];
    let (name_buf, dir_buf) = (name.as_mut_ptr(), dir.as_mut_ptr());
    // SAFETY: `fname` is a NUL-terminated string with at least one byte, and
    // `name`/`dir` are `MAXPATHL`/`IOSIZE` bytes of this frame's own storage.
    // Upstream shares `NameBuff`/`IObuff`, which `buflist_new` runs
    // autocommands over.
    unsafe {
        // `~/` is expanded here rather than by `buflist_new`, because the
        // shada file stores the tilde form and two spellings of one path
        // would open two buffers.
        if *fname == '~' as c_char && vim_ispathsep_nocolon(c_int::from(*fname.offset(1))) {
            let len = expand_env(c"~/".as_ptr().cast_mut(), name_buf, MAXPATHL);
            xstrlcpy(
                name_buf.add(len),
                fname.offset(2),
                (MAXPATHL as size_t).wrapping_sub(len),
            );
        } else {
            xstrlcpy(name_buf, fname, MAXPATHL as size_t);
        }
        os_dirname(dir_buf, IOSIZE as size_t);
        let short = path_shorten_fname(name_buf, dir_buf);
        buflist_new(name_buf, short, 1, 0);
    }
}

/// Check all file marks for a name that matches the file name in buf.
/// May replace the name with an fnum.
/// Used for marks that come from the .shada file.
///
/// # Safety
/// `buf` must be a live buffer, and the editor's window list must be live.
pub unsafe fn fmarks_check_names(buf: *mut buf_T) {
    // SAFETY: the caller promised a live buffer.
    let buf = unsafe { Buf::new(buf) };
    let name = buf.b_ffname;
    if name.is_null() {
        return;
    }
    for mark in GlobalMarks::all() {
        // SAFETY: `name` is the buffer's own file name, live while it is.
        unsafe { fmarks_check_one(mark, name, buf) };
    }
    // The current tab page's windows only, as upstream: a mark in another
    // tab page's jumplist keeps its file name until that window is used.
    for win in windows() {
        for jump in win.jumps() {
            // SAFETY: as above.
            unsafe { fmarks_check_one(jump, name, buf) };
        }
    }
}

/// # Safety
/// `name` must be a NUL-terminated string that outlives the call.
unsafe fn fmarks_check_one(fm: Xfmark, name: *mut c_char, buf: Buf) {
    let fname = fm.fname();
    // SAFETY: both names are NUL-terminated strings.
    if fm.fmark().fnum() != 0 || fname.is_null() || unsafe { path_fnamecmp(name, fname) } != 0 {
        return;
    }
    fm.fmark().set_fnum(buf.handle as c_int);
    fm.clear_fname();
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
///
/// # Safety
/// `fm` must be null or point at a live `fmark_T`.
pub(crate) unsafe fn mark_check(fm: *mut fmark_T, errormsg: &mut Option<CString>) -> bool {
    if fm.is_null() {
        // SAFETY: a NUL-terminated message static.
        *errormsg = Some(unsafe { ex_msg((&raw const e_umark).cast::<c_char>()) });
        return false;
    }
    // SAFETY: the caller promised a live record.
    let fm = unsafe { Fmark::new(fm) };
    if fm.lnum() <= 0 {
        // A negative line number is a mark the shada file mangled; it is not
        // "not set", so it gets no message at all.
        if fm.lnum() == 0 {
            // SAFETY: as above.
            *errormsg = Some(unsafe { ex_msg((&raw const e_marknotset).cast::<c_char>()) });
        }
        return false;
    }
    // SAFETY: `curbuf` is live from startup to exit.
    let buf = unsafe { Buf::current() };
    // SAFETY: as above; the record and the out-parameter are the caller's.
    fm.fnum() != buf.handle || unsafe { mark_check_line_bounds(buf.raw(), fm.raw(), errormsg) }
}

/// Check if a mark line number is greater than the buffer line count, and set e_markinval.
///
/// @note  Should be done after the buffer is loaded into memory.
/// `buf` — Buffer where the mark is set.
/// `fm` — Mark to check.
/// `errormsg[out]` — Error message, if any.
/// Returns true if below line count else false.
///
/// # Safety
/// `buf` must be null or a live buffer, and `fm` must point at a live
/// `fmark_T`.
pub(crate) unsafe fn mark_check_line_bounds(
    buf: *mut buf_T,
    fm: *mut fmark_T,
    errormsg: &mut Option<CString>,
) -> bool {
    // SAFETY: the caller promised a live buffer or null.
    let Some(buf) = (unsafe { Buf::from_raw(buf) }) else {
        return true;
    };
    // SAFETY: the caller promised a live record.
    if unsafe { Fmark::new(fm) }.lnum() <= buf.b_ml.ml_line_count {
        return true;
    }
    // SAFETY: a NUL-terminated message static.
    *errormsg = Some(unsafe { ex_msg((&raw const e_markinval).cast::<c_char>()) });
    false
}

/// Clear all marks and change list in the given buffer
///
/// Used mainly when trashing the entire buffer during ":e" type commands.
///
/// Does not trigger "MarkSet" event.
///
/// `buf` — Buffer to clear marks in.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn clrallmarks(buf: *mut buf_T, timestamp: Timestamp) {
    // SAFETY: the caller promised a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    for mark in buf.named_marks() {
        mark.clear(timestamp);
    }
    buf.last_cursor().clear(timestamp);
    // `'"` is the one store that is cleared to line 1 rather than to 0: the
    // whole point of it is where to put the cursor when the file is opened
    // again, and "the top" is a better answer than "nowhere".
    buf.last_cursor().set_lnum(1);
    buf.last_insert().clear(timestamp);
    buf.last_change().clear(timestamp);
    buf.b_op_start.lnum = 0;
    buf.b_op_end.lnum = 0;
    for change in buf.changes() {
        change.clear(timestamp);
    }
    buf.b_changelistlen = 0;
}

/// # Safety
/// `win` must be a live window.
pub unsafe fn set_last_cursor(win: *mut win_T) {
    // SAFETY: the caller promised a live window.
    let win = unsafe { Win::new(win) };
    let Some(buf) = win.buffer_or_none() else {
        return;
    };
    // `fnum` 0, not the buffer's handle: this mark is written when the buffer
    // is left, and the shada writer is what fills the file name in.
    buf.last_cursor().replace(win.w_cursor, 0, NO_VIEW);
}

/// Adjust position to point to the first byte of a multi-byte character
///
/// If it points to a tail byte it is move backwards to the head byte.
///
/// `buf` — Buffer to adjust position in.
/// `lp` — Position to adjust.
///
/// # Safety
/// `buf` must be a live buffer and `lp` must point at a live, writable
/// position naming a line of it.
pub unsafe fn mark_mb_adjustpos(buf: *mut buf_T, lp: *mut pos_T) {
    // SAFETY: the caller promised a live position.
    let mut pos = unsafe { *lp };
    if pos.col <= 0 && pos.coladd <= 1 {
        return;
    }
    // SAFETY: the caller promised a live buffer and a line of it.
    unsafe {
        let p = ml_get_buf(buf, pos.lnum);
        if *p == NUL_BYTE || ml_get_buf_len(buf, pos.lnum) < pos.col {
            pos.col = 0;
        } else {
            pos.col -= utf_head_off(p, p.offset(pos.col as isize));
        }
        // A `coladd` of 1 on a printable wide character is the "one cell into
        // it" position virtual editing produces; the head byte has no such
        // offset, so it goes.
        let at = p.offset(pos.col as isize);
        if pos.coladd == 1
            && c_int::from(*at) != TAB
            && vim_isprintc(utf_ptr2char(at))
            && ptr2cells(at) > 1
        {
            pos.coladd = 0;
        }
        *lp = pos;
    }
}
