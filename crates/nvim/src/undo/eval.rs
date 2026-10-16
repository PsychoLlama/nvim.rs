//! `:undolist` and the `undofile()`/`undotree()` builtins.
//!
//! The read-only view of the undo tree: three ways of asking what is in it
//! and one of asking where it would be stored.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::ffi::CString;

use super::file::u_get_undo_file_name;
use super::store::Marks;
use super::*;
use crate::eval::typval::NumBuf;
use crate::highlight_group::HLF_T;
use crate::memory::ThinCString;
use crate::types::{VAR_STRING, kListLenMayKnow};
use crate::winlayer::Buf;

/// One `:undolist` row: the header's sequence number, how many changes deep
/// in the tree it sits, when it was made, and — if it was ever written out —
/// which file write it belongs to.
fn undolist_row(uh: &UndoHeader, changes: c_int) -> CString {
    let mut row = format!("{:>6} {:>7}  ", uh.seq, changes).into_bytes();
    // What a 64-byte scratch buffer held.
    row.extend_from_slice(&undo_time_text(uh.time, 64));
    if uh.save_nr > 0 {
        // The "saved" column starts at 33, however long the time took.
        row.resize(row.len().max(33), b' ');
        row.extend_from_slice(format!("  {:>3}", uh.save_nr).as_bytes());
    }
    CString::new(row).expect("a row is digits, spaces and a time with no NUL")
}

/// `:undolist` — every leaf of the undo tree, oldest first.
pub fn ex_undolist(_excmd: &mut ExArg) {
    let buf = Buf::current();
    // A leaf is a header nothing branches off downwards, and the whole tree
    // has to be walked to find them all.
    let mut rows: Vec<CString> = Vec::new();
    for visit in buf.tree_walk(buf.b_u_oldhead, Marks::next()) {
        if let Some(uh) = buf
            .header(visit.link)
            .filter(|uh| visit.first && uh.prev.is_none())
        {
            rows.push(undolist_row(uh, visit.depth));
        }
    }

    msg_ext_set_kind(c"list_cmd");
    if rows.is_empty() {
        msg(gettext(c"Nothing to undo"), 0);
        return;
    }
    // The walk reaches the branches in tree order, not in sequence order.
    rows.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    msg_start();
    let heading = gettext(c"number changes  when               saved");
    msg_str_hl(heading, HLF_T, false);
    for row in &rows {
        if got_int.get() {
            break;
        }
        msg_putchar('\n' as c_int);
        if got_int.get() {
            break;
        }
        msg_str(row);
    }
    msg_end();
}

/// One branch of the tree as `undotree()` reports it: a list of dictionaries,
/// newest change first, each carrying its own alternate branch under `alt`.
///
/// Every dictionary gets its keys in one fixed order — `seq`, `time`, then
/// `newhead`/`curhead`/`save`/`alt` where they apply — which is the order
/// upstream adds them in.
fn eval_tree(buffer: Buf, first: UndoLink) -> ListRef {
    let mut list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let mut link = first;
    while let Some(uh) = buffer.header(link) {
        let (seq, time, save_nr, alt_next, prev) =
            (uh.seq, uh.time, uh.save_nr, uh.alt_next, uh.prev);
        let mut dict = tv_dict_alloc();
        let _ = dict.add_number(b"seq", VarNumber::from(seq));
        let _ = dict.add_number(b"time", time);
        if link == buffer.b_u_newhead {
            let _ = dict.add_number(b"newhead", 1);
        }
        if link == buffer.b_u_curhead {
            let _ = dict.add_number(b"curhead", 1);
        }
        if save_nr > 0 {
            let _ = dict.add_number(b"save", VarNumber::from(save_nr));
        }
        if alt_next.is_some() {
            let _ = dict.add_list(b"alt", Some(eval_tree(buffer, alt_next)));
        }
        list.push_dict(Some(dict));
        link = prev;
    }
    list
}

/// `undofile({name})` — where the undo file for `{name}` would be written.
pub fn f_undofile(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let fname = numbuf.string(&args[0]);
    if fname.is_empty() {
        result.write_string(None);
        return;
    }
    let ffname = full_name_save(fname, true);
    let undo_file = u_get_undo_file_name(Some(ffname.as_cstr()), false);
    result.write_string(undo_file.map(|name| ThinCString::from_bytes(&name)));
}

/// `undotree([{buf}])` — the whole tree, plus where in it the buffer sits.
pub fn f_undotree(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);
    let buf = match args.first() {
        Some(tv) => get_buf_arg(tv),
        None => Buf::current_or_none(),
    };
    let Some(buf) = buf else { return };
    let entries = eval_tree(buf, buf.b_u_oldhead);
    let Some(dict) = result.dict_mut() else {
        return;
    };
    let _ = dict.add_number(b"synced", VarNumber::from(buf.b_u_synced));
    let _ = dict.add_number(b"seq_last", VarNumber::from(buf.b_u_seq_last));
    let _ = dict.add_number(b"save_last", VarNumber::from(buf.b_u_save_nr_last));
    let _ = dict.add_number(b"seq_cur", VarNumber::from(buf.b_u_seq_cur));
    let _ = dict.add_number(b"time_cur", buf.b_u_time_cur);
    let _ = dict.add_number(b"save_cur", VarNumber::from(buf.b_u_save_nr_cur));
    let _ = dict.add_list(b"entries", Some(entries));
}

/// The header a change to `buffer` would be recorded against, making one if
/// the buffer has none yet. `None` when undo is off for the buffer.
///
/// `extmark` hangs its undo records off the answer, looked up through the
/// buffer as it needs it.
pub fn u_force_get_undo_header(buffer: Buf) -> Option<UndoLink> {
    let current = |b: Buf| {
        [b.b_u_curhead, b.b_u_newhead]
            .into_iter()
            .find(|&link| b.has_header(link))
    };
    if let Some(link) = current(buffer) {
        return Some(link);
    }
    // Nothing to hang it on: force an undo header, even for an empty change.
    let _ = u_savecommon(buffer, 0, 1, 1, true);
    let link = current(buffer);
    assert!(
        link.is_some() || get_undolevel(buffer) <= 0,
        "u_savecommon made no undo header while undo was enabled"
    );
    link
}
