//! `:undolist` and the `undofile()`/`undotree()` builtins.
//!
//! The read-only view of the undo tree: three ways of asking what is in it
//! and one of asking where it would be stored.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;
use std::ffi::CString;

use super::file::u_get_undo_file_name;
use super::store::Marks;
use super::*;
use crate::eval::typval::NumBuf;
use crate::highlight_group::HLF_T;
use crate::types::{VAR_STRING, VAR_UNKNOWN, kListLenMayKnow};
use crate::winlayer::Buf;

/// `tv_dict_add_nr` for a literal key, whose length the `CStr` already knows.
///
/// Module-private on purpose: a *public* safe fn taking a raw pointer trips
/// `clippy::not_unsafe_ptr_arg_deref`, which is denied tree-wide.
fn dict_add_nr(dict: *mut dict_T, key: &CStr, val: varnumber_T) {
    // SAFETY: a dictionary this module just allocated, and a NUL-terminated
    // key with its own length.
    unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), val) };
}

/// [`dict_add_nr`] for a list value, which the dictionary takes over.
fn dict_add_list(dict: *mut dict_T, key: &CStr, val: *mut list_T) {
    // SAFETY: as [`dict_add_nr`], plus a list this module just built.
    unsafe { tv_dict_add_list(dict, key.as_ptr(), key.count_bytes(), val) };
}

/// One `:undolist` row: the header's sequence number, how many changes deep
/// in the tree it sits, when it was made, and — if it was ever written out —
/// which file write it belongs to.
fn undolist_row(uh: &u_header_T, changes: c_int) -> CString {
    let mut row = format!("{:>6} {:>7}  ", uh.uh_seq, changes).into_bytes();
    let mut when = [0 as c_char; 64];
    // SAFETY: a buffer of exactly the length passed; `undo_fmt_time` leaves a
    // NUL-terminated string in it and reads nothing else.
    unsafe { undo_fmt_time(when.as_mut_ptr(), when.len(), uh.uh_time) };
    // SAFETY: NUL-terminated, and the NUL is inside the array.
    row.extend_from_slice(unsafe { CStr::from_ptr(when.as_ptr()) }.to_bytes());
    if uh.uh_save_nr > 0 {
        // The "saved" column starts at 33, however long the time took.
        row.resize(row.len().max(33), b' ');
        row.extend_from_slice(format!("  {:>3}", uh.uh_save_nr).as_bytes());
    }
    CString::new(row).expect("a row is digits, spaces and one NUL-terminated time")
}

/// `:undolist` — every leaf of the undo tree, oldest first.
///
/// # Safety
///
/// A live current buffer.
pub unsafe fn ex_undolist(_eap: *mut exarg_T) {
    // SAFETY: a live current buffer, by the contract above.
    let buf = unsafe { Buf::current() };
    // A leaf is a header nothing branches off downwards, and the whole tree
    // has to be walked to find them all.
    let rows: Vec<CString> = buf
        .tree_walk(buf.b_u_oldhead, Marks::next())
        .filter(|visit| visit.first && visit.header.uh_prev.is_none())
        .map(|visit| undolist_row(&visit.header, visit.depth))
        .collect();

    // SAFETY: a NUL-terminated literal.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    if rows.is_empty() {
        msg(gettext(c"Nothing to undo"), 0);
        return;
    }
    let mut rows = rows;
    // The walk reaches the branches in tree order, not in sequence order.
    rows.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    // SAFETY: the editor's own message state.
    unsafe { msg_start() };
    let heading = gettext(c"number changes  when               saved");
    // SAFETY: the string `gettext` just answered, NUL-terminated.
    unsafe { msg_puts_hl(heading.as_ptr(), HLF_T, false) };
    for row in &rows {
        if got_int.get() {
            break;
        }
        // SAFETY: the editor's own message state.
        unsafe { msg_putchar('\n' as c_int) };
        if got_int.get() {
            break;
        }
        // SAFETY: a NUL-terminated string this function owns.
        unsafe { msg_puts(row.as_ptr()) };
    }
    // SAFETY: the editor's own message state.
    unsafe { msg_end() };
}

/// One branch of the tree as `undotree()` reports it: a list of dictionaries,
/// newest change first, each carrying its own alternate branch under `alt`.
fn eval_tree(buf: Buf, first: UndoLink) -> *mut list_T {
    // SAFETY: an empty list, whose length is not known up front.
    let list: *mut list_T = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    let mut link = first;
    while let Some(uh) = buf.header(link) {
        // SAFETY: a fresh dictionary.
        let dict: *mut dict_T = unsafe { tv_dict_alloc() };
        dict_add_nr(dict, c"seq", varnumber_T::from(uh.uh_seq));
        dict_add_nr(dict, c"time", uh.uh_time);
        if uh.link() == buf.b_u_newhead {
            dict_add_nr(dict, c"newhead", 1);
        }
        if uh.link() == buf.b_u_curhead {
            dict_add_nr(dict, c"curhead", 1);
        }
        if uh.uh_save_nr > 0 {
            dict_add_nr(dict, c"save", varnumber_T::from(uh.uh_save_nr));
        }
        if uh.uh_alt_next.is_some() {
            dict_add_list(dict, c"alt", eval_tree(buf, uh.uh_alt_next));
        }
        // SAFETY: a list and a dictionary this function owns.
        unsafe { tv_list_append_dict(list, dict) };
        link = uh.uh_prev;
    }
    list
}

/// `undofile({name})` — where the undo file for `{name}` would be written.
///
/// # Safety
///
/// The eval-function contract: one argument and a return value to fill in.
pub unsafe fn f_undofile(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the eval-function contract, by the contract above.
    unsafe { (*rettv).v_type = VAR_STRING };
    // SAFETY: as above.
    let fname: *const c_char = unsafe { numbuf.string(argvars) };
    // SAFETY: a NUL-terminated name.
    if unsafe { *fname } == NUL as c_char {
        // SAFETY: the return value to fill in.
        unsafe { (*rettv).vval.v_string = ptr::null_mut() };
        return;
    }
    // SAFETY: a NUL-terminated name.
    let ffname: *mut c_char = unsafe { full_name_save(fname, true) };
    if !ffname.is_null() {
        // SAFETY: a NUL-terminated absolute path, and the return value.
        unsafe { (*rettv).vval.v_string = u_get_undo_file_name(ffname, false) };
    }
    // SAFETY: NULL, or `full_name_save`'s allocation.
    unsafe { xfree(ffname.cast()) };
}

/// `undotree([{buf}])` — the whole tree, plus where in it the buffer sits.
///
/// # Safety
///
/// The eval-function contract, and a live current buffer.
pub unsafe fn f_undotree(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the eval-function contract, by the contract above.
    unsafe { tv_dict_alloc_ret(rettv) };
    let tv: *mut typval_T = argvars;
    // SAFETY: as above.
    let raw = if unsafe { (*tv).v_type } == VAR_UNKNOWN {
        curbuf.get()
    } else {
        // SAFETY: as above.
        unsafe { get_buf_arg(tv) }
    };
    // SAFETY: the return value the contract gives us.
    let dict = unsafe { (*rettv).vval.v_dict };
    // SAFETY: `curbuf` and `get_buf_arg` both answer a live buffer or NULL.
    let buf = unsafe { Buf::from_raw(raw) };
    let Some(buf) = buf else { return };

    dict_add_nr(dict, c"synced", varnumber_T::from(buf.b_u_synced));
    dict_add_nr(dict, c"seq_last", varnumber_T::from(buf.b_u_seq_last));
    dict_add_nr(dict, c"save_last", varnumber_T::from(buf.b_u_save_nr_last));
    dict_add_nr(dict, c"seq_cur", varnumber_T::from(buf.b_u_seq_cur));
    dict_add_nr(dict, c"time_cur", buf.b_u_time_cur);
    dict_add_nr(dict, c"save_cur", varnumber_T::from(buf.b_u_save_nr_cur));
    dict_add_list(dict, c"entries", eval_tree(buf, buf.b_u_oldhead));
}

/// The header a change to `buf` would be recorded against, making one if the
/// buffer has none yet.
///
/// The address is what the caller wants — `extmark`'s undo list hangs off
/// `uh_extmark` — and the store's allocations are stable, so handing one out
/// is sound.
///
/// # Safety
///
/// `buf` points at a live buffer, and a live current window.
pub unsafe fn u_force_get_undo_header(buf: *mut buf_T) -> *mut u_header_T {
    // SAFETY: a live buffer, by the contract above.
    let mut b = unsafe { Buf::new(buf) };
    if let Some(uh) = b.header(b.b_u_curhead).or_else(|| b.header(b.b_u_newhead)) {
        return uh.raw();
    }
    // Nothing to hang it on: force an undo header, even for an empty change.
    u_savecommon(b, 0, 1, 1, true);
    // SAFETY: `u_savecommon` may have reloaded the buffer under us.
    b = unsafe { Buf::new(buf) };
    match b.header(b.b_u_curhead).or_else(|| b.header(b.b_u_newhead)) {
        Some(uh) => uh.raw(),
        None => {
            assert!(
                get_undolevel(b) <= 0,
                "u_savecommon made no undo header while undo was enabled"
            );
            ptr::null_mut()
        }
    }
}
