//! Setting marks and jumping to them.
//!
//! A named file mark is valid when its `lnum` is non-zero. A non-zero `fnum`
//! means it names a live buffer; otherwise it came from the shada file and
//! `namedfm[n].fname` is the file name. The global set is `'A`-`'Z`, which
//! the user sets, plus `'0`-`'9`, which are written when shada is saved.
//!
//! The stores split by concern: [`store`] names the stores and builds their
//! records, [`adjust`] rewrites every mark's line
//! and column when the buffer's lines move (`mark_adjust` is on the path of
//! every `:d`, `:m` and undo), [`jumplist`] owns the jumplist and the
//! changelist, [`lookup`] resolves a mark's name and moves the cursor to it,
//! [`show`] is `:marks`/`:delmarks`, [`shada`] is the iterator surface the
//! shada writer walks, and [`builtins`] is `getmarklist()`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::ascii::{ascii_isdigit, ascii_islower, ascii_isupper};
use crate::autocmd::{aucmd_defer, has_event};
use crate::buffer::{buf_is_prompt, buflist_new, find_buf};
use crate::charset::{ptr2cells_at, vim_isprintc};
use crate::cstr;
use crate::ex_docmd::ex_msg;
use crate::fold::has_folding;
use crate::mbyte::{char_at, head_off};
use crate::memory::XString;
use crate::message::{e_markinval, e_marknotset, e_umark};
use crate::r#move::set_topline;
use crate::options::kOptJopFlagStack;
use crate::os::env::expand_env_esc_of;
use crate::os::fs::dirname_within;
use crate::path::{path_fnamecmp, path_shorten_fname, vim_ispathsep_nocolon};
use crate::plines::linetabsize_eol;
use crate::tag::tagstack_clear_entry;
use crate::types::Failed;
use crate::types::TAB;
use crate::types::*;
use crate::types::{AutoEvent, FileMark, FileMarkView, LineNr, Pos, XFileMark};
use crate::winlayer::{Buf, Win, windows};
use core::ffi::{CStr, c_char, c_int};
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
    free_jumplist, get_changelist, get_jumplist, mark_jumplist_forget_file, setpcmark,
};
pub use lookup::{
    getnextmark, mark_get, mark_get_global, mark_get_local, mark_get_motion, mark_get_visual,
    mark_move_to,
};
pub(crate) use shada::{
    ShadaMark, global_mark_timestamp, shada_buffer_marks, shada_changes, shada_global_marks,
    shada_insert_change, shada_insert_jump, shada_jumps, shada_local_mark_timestamp,
    shada_restore_last_cursor, shada_set_global_mark, shada_set_local_mark,
};
pub use show::{ex_delmarks, ex_marks, fm_getname};

pub(crate) use store::fresh_mark;
use store::{NO_VIEW, NUL_BYTE, mark_name, slot};

pub const GETF_SETMARK: GetFileFlags = 1;
pub const AUGROUP_ALL: c_int = -3;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMTCharWise: MotionType = 0;
pub const ARRAY_DICT_INIT: ApiDict = ApiDict::EMPTY;

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

use crate::global_cell::GlobalCell;
use crate::quickfix::{BUF_HAS_LL_ENTRY, BUF_HAS_QF_ENTRY};

/// The global marks: `'A`-`'Z` at `0..NMARKS`, then `'0`-`'9`.
pub(crate) static namedfm: GlobalCell<[XFileMark; 36]> = GlobalCell::new([XFileMark::UNSET; 36]);

/// Set named mark "c" at current cursor position.
/// Answers `Ok` on success, `Err` if bad name given.
pub fn setmark(c: c_int) -> Result<(), Failed> {
    let (win, buf) = (Win::current(), Buf::current());
    let view = mark_view_make_at(win, win.w_cursor);
    setmark_pos(c, win.w_cursor, true, buf.handle, view)
}

/// Schedules "MarkSet" event.
///
/// `c` — The name of the mark, e.g., 'a'.
/// `pos` — Position of the mark in the buffer.
/// `buffer` — The buffer of the mark.
fn do_markset_autocmd(c: c_char, pos: Pos, buffer: Buf) {
    if !has_event(AutoEvent::MarkSet) {
        return;
    }
    let mark_str: [c_char; 2] = [c, NUL_BYTE];
    let items: Vec<KeyValuePair> = vec![
        key_value_pair {
            key: c"name".into(),
            value: Object::string(String_0::from_bytes(&[mark_str[0].cast_unsigned()])),
        },
        key_value_pair {
            key: c"line".into(),
            value: Object::Integer(Integer::from(pos.lnum)),
        },
        key_value_pair {
            key: c"col".into(),
            value: Object::Integer(Integer::from(pos.col)),
        },
    ];
    let payload: Object = Object::dict(ApiDict::from(items));
    // `aucmd_defer` copies the name and the payload before it returns.
    aucmd_defer(
        AutoEvent::MarkSet,
        Some(cstr::in_chars(&mark_str)),
        None,
        AUGROUP_ALL,
        buffer,
        Some(&payload),
    );
}

/// Set named mark "c" to position "pos".
/// When "c" is upper case use file "fnum".
/// Answers `Ok` on success, `Err` if bad name given.
///
/// `from_cursor` says `pos` is the current window's cursor itself, which is
/// what makes setting `''` push a jump list entry rather than just move the
/// context mark (upstream compares the pointer against `&curwin->w_cursor`).
fn setmark_pos(
    c: c_int,
    at: Pos,
    from_cursor: bool,
    fnum: c_int,
    view: FileMarkView,
) -> Result<(), Failed> {
    if c < 0 {
        return Err(Failed);
    }
    // `''` and `` '` `` are the same slot, and it is the window's rather than
    // the buffer's: setting it from the cursor pushes a jumplist entry, while
    // setting it from anywhere else just moves it.
    if c == '\'' as c_int || c == '`' as c_int {
        let mut win = Win::current();
        if from_cursor {
            setpcmark();
            win.w_prev_pcmark = win.w_pcmark;
        } else {
            win.w_pcmark = at;
        }
        return Ok(());
    }
    let Some(mut buf) = find_buf(fnum) else {
        return Err(Failed);
    };
    let handle = buf.handle;

    // The tick family and the Visual range are not adjusted and not saved, so
    // they are stored as bare positions rather than as marks.
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
        buf.b_last_cursor = fresh_mark(at, handle, view);
    } else if c == ':' as c_int {
        if !buf_is_prompt(Some(buf)) {
            return Err(Failed);
        }
        buf.b_prompt_start = fresh_mark(at, handle, view);
        // The prompt mark is the one store that does NOT announce itself:
        // it moves on every prompt redraw, and a MarkSet per keystroke is
        // not what the event is for.
        return Ok(());
    } else if ascii_islower(c) {
        // A buffer-local mark keeps the *caller's* `fnum` rather than the
        // buffer's own handle. The two agree everywhere but `nvim_buf_set_mark`.
        buf.b_namedm[slot(c - 'a' as c_int)] = fresh_mark(at, fnum, view);
    } else if ascii_isupper(c) || ascii_isdigit(c) {
        let idx = slot(lookup::mark_global_index(mark_name(c)));
        store::set_global_mark(
            idx,
            XFileMark {
                fmark: fresh_mark(at, fnum, view),
                fname: None,
            },
        );
    } else {
        return Err(Failed);
    }
    do_markset_autocmd(mark_name(c), at, buf);
    Ok(())
}

/// [`setmark_pos`] of a position held by value, with no view: set mark `c`
/// to `pos`, in file `fnum` for a file mark.
///
/// A position held by value is never the cursor itself, so `''` set this way
/// just moves the context mark rather than pushing a jumplist entry.
pub(crate) fn setmark_at(c: c_int, pos: Pos, fnum: c_int) -> Result<(), Failed> {
    setmark_pos(c, pos, false, fnum, NO_VIEW)
}

/// Delete every entry referring to file "fnum" from both the jumplist and the
/// tag stack.
pub fn mark_forget_file(mut window: Win, fnum: c_int) {
    mark_jumplist_forget_file(window, fnum);
    // Backwards, so removing an entry cannot skip the one after it.
    for i in (0..window.w_tagstacklen).rev() {
        let at = slot(i);
        if window.w_tagstack[at].fmark.fnum != fnum {
            continue;
        }
        let len = slot(window.w_tagstacklen);
        if window.w_tagstackidx > i {
            window.w_tagstackidx -= 1;
        }
        window.w_tagstacklen -= 1;
        // Close the gap: the entries above move down one, and the cleared
        // one goes to the top, past the end of what is in use.
        let stack = &mut window.w_tagstack[at..len];
        tagstack_clear_entry(&mut stack[0]);
        stack.rotate_left(1);
    }
}

/// Wrap a `Pos` into a `FileMark`, used to abstract marks handling: the
/// mark at `pos` in `buffer`, unset in every other field.
///
/// The C answered a `static` here that every caller without a record of its
/// own shared, so two motion-mark lookups invalidated each other; the
/// answer is the caller's value now.
pub fn pos_to_mark(buffer: Buf, pos: Pos) -> FileMark {
    FileMark {
        mark: pos,
        fnum: buffer.handle,
        ..FileMark::UNSET
    }
}

/// Restore the mark view.
/// By remembering the offset between topline and mark lnum at the time of
/// definition, this function restores the "view".
/// @note  Assumes the mark has been checked, is valid.
/// `fm` — the named mark.
///
/// `None` restores nothing.
pub fn mark_view_restore(fm: Option<&FileMark>) {
    let Some(fm) = fm else {
        return;
    };
    if fm.view.topline_offset < 0 {
        return;
    }
    let topline = fm.mark.lnum - fm.view.topline_offset;
    if topline < 1 {
        return;
    }
    let mut win = Win::current();
    set_topline(win, topline);
    // A remembered `skipcol` is dropped when the line it names is now folded
    // away or has become too short to reach — restoring it would scroll the
    // window sideways past the end of the line.
    // `skipcol` is re-read after the scroll rather than snapshotted with
    // `topline_offset` above, because that is the order upstream reads the
    // two fields in.
    let skipcol = fm.view.skipcol;
    let keep = skipcol > 0
        && !has_folding(win, topline, None, None)
        && skipcol < linetabsize_eol(win, topline);
    win.w_skipcol = if keep { skipcol } else { 0 };
}

pub fn mark_view_make(window: Win, pos: Pos) -> FileMarkView {
    mark_view_make_at(window, pos)
}

/// The view [`mark_view_make`] records: how far below the window's topline the
/// position sits, and where the window was scrolled to sideways.
fn mark_view_make_at(window: Win, pos: Pos) -> FileMarkView {
    FileMarkView {
        topline_offset: pos.lnum - window.w_topline,
        skipcol: window.w_skipcol,
    }
}

/// For an xtended filemark: set the fnum from the fname.
/// This is used for marks obtained from the .shada file.  It's postponed
/// until the mark is used to avoid a long startup delay.
///
/// `fname` is a copy of the mark's name: loading the buffer runs
/// autocommands, and the new buffer's `fmarks_check_names` is what turns
/// every mark naming this file -- this one included -- into a number.
pub(super) fn fname2fnum(fname: &CStr) {
    let bytes = fname.to_bytes();
    let room = usize::try_from(MAXPATHL).map_or(0, |n| n - 1);
    // `~/` is expanded here rather than by `buflist_new`, because the
    // shada file stores the tilde form and two spellings of one path
    // would open two buffers.
    let mut name = if bytes.first() == Some(&b'~')
        && vim_ispathsep_nocolon(c_int::from(bytes.get(1).copied().unwrap_or(0)))
    {
        let mut home = expand_env_esc_of(c"~/", false, false, None)
            .as_cstr()
            .to_bytes()
            .to_vec();
        home.truncate(room);
        home.extend_from_slice(&bytes[2..]);
        home
    } else {
        bytes.to_vec()
    };
    // `MAXPATHL` bytes with the NUL, as upstream's `NameBuff` copy.
    name.truncate(room);
    let name = XString::from_vec(name);
    // Upstream asks into `IObuff` and goes on with whatever is there when
    // the system will not say -- the error message.
    let room = usize::try_from(IOSIZE).unwrap_or(0);
    let dir = dirname_within(room).unwrap_or_else(|message| message);
    let short = path_shorten_fname(name.as_cstr(), &dir);
    buflist_new(Some(name.as_cstr()), short, 1, 0);
}

/// Check all file marks for a name that matches the file name in buf.
/// May replace the name with an fnum.
/// Used for marks that come from the .shada file.
pub fn fmarks_check_names(buffer: Buf) {
    let Some(name) = buffer.name.full() else {
        return;
    };
    let (name, handle) = (name.to_bytes(), buffer.handle);
    namedfm.with_mut(|table| {
        for mark in table.iter_mut() {
            fmarks_check_one(mark, name, handle);
        }
    });
    // The current tab page's windows only, as upstream: a mark in another
    // tab page's jumplist keeps its file name until that window is used.
    for mut win in windows() {
        let len = slot(win.w_jumplistlen);
        for jump in &mut win.w_jumplist[..len] {
            fmarks_check_one(jump, name, handle);
        }
    }
}

fn fmarks_check_one(fm: &mut XFileMark, name: &[u8], handle: c_int) {
    if fm.fmark.fnum != 0 {
        return;
    }
    match fm.file_name() {
        Some(fname) if path_fnamecmp(name, fname.to_bytes()) == 0 => {}
        _ => return,
    }
    fm.fmark.fnum = handle;
    fm.fname = None;
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
pub(crate) fn mark_check(fm: Option<&FileMark>, errormsg: &mut Option<CString>) -> bool {
    let Some(fm) = fm else {
        *errormsg = Some(ex_msg(e_umark));
        return false;
    };
    if fm.mark.lnum <= 0 {
        // A negative line number is a mark the shada file mangled; it is not
        // "not set", so it gets no message at all.
        if fm.mark.lnum == 0 {
            *errormsg = Some(ex_msg(e_marknotset));
        }
        return false;
    }
    let buf = Buf::current();
    fm.fnum != buf.handle || mark_check_line_bounds(Some(buf), fm, errormsg)
}

/// Check if a mark line number is greater than the buffer line count, and set e_markinval.
///
/// @note  Should be done after the buffer is loaded into memory.
/// `buf` — Buffer where the mark is set.
/// `fm` — Mark to check.
/// `errormsg[out]` — Error message, if any.
/// Returns true if below line count else false.
pub(crate) fn mark_check_line_bounds(
    buffer: Option<Buf>,
    fm: &FileMark,
    errormsg: &mut Option<CString>,
) -> bool {
    let Some(buf) = buffer else {
        return true;
    };
    if fm.mark.lnum <= buf.b_ml.ml_line_count {
        return true;
    }
    *errormsg = Some(ex_msg(e_markinval));
    false
}

/// Clear all marks and change list in the given buffer
///
/// Used mainly when trashing the entire buffer during ":e" type commands.
///
/// Does not trigger "MarkSet" event.
///
/// `buffer` — Buffer to clear marks in.
pub fn clrallmarks(mut buffer: Buf, timestamp: Timestamp) {
    let b: &mut Buffer = &mut buffer;
    for mark in &mut b.b_namedm {
        mark.clear(timestamp);
    }
    b.b_last_cursor.clear(timestamp);
    // `'"` is the one store that is cleared to line 1 rather than to 0: the
    // whole point of it is where to put the cursor when the file is opened
    // again, and "the top" is a better answer than "nowhere".
    b.b_last_cursor.mark.lnum = 1;
    b.b_last_insert.clear(timestamp);
    b.b_last_change.clear(timestamp);
    b.b_op_start.lnum = 0;
    b.b_op_end.lnum = 0;
    let len = slot(b.b_changelistlen);
    for change in &mut b.b_changelist[..len] {
        change.clear(timestamp);
    }
    b.b_changelistlen = 0;
}

pub fn set_last_cursor(win: Win) {
    let Some(mut buf) = win.buffer_or_none() else {
        return;
    };
    // `fnum` 0, not the buffer's handle: this mark is written when the buffer
    // is left, and the shada writer is what fills the file name in.
    buf.b_last_cursor = fresh_mark(win.w_cursor, 0, NO_VIEW);
}

/// Adjust position to point to the first byte of a multi-byte character
///
/// If it points to a tail byte it is move backwards to the head byte.
///
/// `buffer` — Buffer to adjust position in.
/// `pos` — Position to adjust, naming a line of `buffer`.
pub fn mark_mb_adjustpos(buffer: Buf, pos: &mut Pos) {
    if pos.col <= 0 && pos.coladd <= 1 {
        return;
    }
    let mut lines = buffer.lines();
    let line = lines.line(pos.lnum);
    let col = usize::try_from(pos.col).unwrap_or(0);
    let col = if line.is_empty() || line.len() < col {
        0
    } else {
        col - head_off(line, col)
    };
    pos.col = ColNr::try_from(col).unwrap_or(ColNr::MAX);
    let at = &line[col..];
    if pos.coladd == 1
        && c_int::from(cstr::byte_at(at, 0)) != TAB
        && vim_isprintc(char_at(at))
        && ptr2cells_at(at) > 1
    {
        pos.coladd = 0;
    }
}
