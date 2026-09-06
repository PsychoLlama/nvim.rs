//! The Ex commands that drive the argument list: `:args`, `:arglocal` and
//! `:argglobal`; `:argadd`, `:argedit`, `:argdelete` and `:argdedupe`; and
//! the `:next`/`:previous`/`:first`/`:last`/`:argument` family that walks
//! it, each in its window-splitting `:s…` form too.
//!
//! Every entry point here takes the live command block `ex_docmd` hands it.
//! That is their whole safety contract, and it is stated here rather than
//! repeated on each of them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::memline::MlFlags;
use crate::types::CmdIdx;
use crate::winlayer::{Buf, Win};

// ---------------------------------------------------------------------------
// The Ex commands.

/// `:args`, `:arglocal` and `:argglobal`.
pub unsafe fn ex_args(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let eap = unsafe { Ea::new(args) };
    // SAFETY: caller contract.
    let cmdidx = eap.cmdidx;
    if cmdidx != CmdIdx::args {
        if arglist_is_locked() {
            return;
        }
        // SAFETY: curwin always has an argument list, and dropping the
        // reference to it is what makes room for the new one.
        unsafe { alist_unlink(win_alist(Win::current())) };
        if cmdidx == CmdIdx::argglobal {
            Win::current().w_alist = global_arglist();
        } else {
            alist_new();
        }
    }
    // ":args file ..": define a new argument list, handled like ":next".
    // Also for ":arglocal file .." and ":argglobal file ..".
    // SAFETY: an ex-command argument is NUL-terminated.
    if unsafe { *eap.arg } as c_int != NUL {
        if arglist_is_locked() {
            return;
        }
        // SAFETY: caller contract.
        unsafe { ex_next(eap.raw()) };
        return;
    }
    if cmdidx == CmdIdx::args {
        // SAFETY: every entry of the current list has a name.
        list_args();
    } else if cmdidx == CmdIdx::arglocal {
        // SAFETY: both lists are valid.
        copy_global_arglist();
    }
}

/// `:args` with no argument: list the arguments, the current one bracketed.
///
/// Safe: every window always has an argument list.
fn list_args() {
    if argcount() <= 0 {
        // Empty argument list.
        return;
    }
    // Overwrite the command: for a short list no scrolling and hence no
    // wait_return() is needed.
    // SAFETY: every entry's name is NUL-terminated and outlives the listing.
    unsafe { gotocmdline(true) };
    let items: Vec<&CStr> = (0..argcount())
        .map(|i| unsafe { CStr::from_ptr(arg_name(i)) })
        .collect();
    unsafe { list_in_columns(&items, cur_arg_idx()) };
}

/// `:arglocal` with no argument: copy the global list into the window's own,
/// skipping entries that have lost their name.
///
/// Safe: both lists always exist -- every window has one, and the global
/// list lives from startup to exit.
fn copy_global_arglist() {
    let al = win_alist(Win::current());
    // SAFETY: both lists are live, and each name is copied into an
    // allocation the new entry owns. The copies are collected before any of
    // them joins the window's list, which is never the global one here.
    let copies: Vec<ArgEntry> = unsafe { &(*global_arglist()).al_ga }
        .iter()
        .filter(|entry| !entry.ae_fname.is_null())
        .map(|entry| ArgEntry {
            ae_fname: unsafe { xstrdup(entry.ae_fname) },
            ae_fnum: entry.ae_fnum,
        })
        .collect();
    unsafe { (*al).al_ga.extend(copies) };
}

/// `:previous`, `:sprevious`, `:Next` and `:sNext`.
pub unsafe fn ex_previous(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the count is the command's range.
    let back = cur_arg_idx() - args.line2 as c_int;
    // If already past the last one, go to the last one.
    let argn = if back >= argcount() {
        argcount() - 1
    } else {
        back
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(args.raw(), argn) };
}

/// `:rewind`, `:first`, `:sfirst` and `:srewind`.
pub unsafe fn ex_rewind(args: *mut ExArg) {
    // SAFETY: caller contract.
    unsafe { do_argfile(args, 0) };
}

/// `:last` and `:slast`.
pub unsafe fn ex_last(args: *mut ExArg) {
    // SAFETY: caller contract.
    unsafe { do_argfile(args, argcount() - 1) };
}

/// `:argument` and `:sargument`.
pub unsafe fn ex_argument(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the argument number is the command's range.
    let argn = if args.addr_count > 0 {
        args.line2 as c_int - 1
    } else {
        cur_arg_idx()
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(args.raw(), argn) };
}

/// Why argument `argn` cannot be reached.
fn report_no_such_arg(argn: c_int) {
    if argcount() <= 1 {
        crate::semsg!("E163: There is only one file to edit");
    } else if argn < 0 {
        crate::semsg!("E164: Cannot go before first file");
    } else {
        crate::semsg!("E165: Cannot go beyond last file");
    }
}

/// May the current buffer be left to edit argument `argn`? With 'hidden' it
/// may, unless this is a re-edit of the same file; otherwise the buffer must
/// be unchanged, written, or abandoned by force.
///
/// # Safety
///
/// `argn` must be a valid argument index.
unsafe fn can_leave_curbuf(argn: c_int, forceit: bool) -> bool {
    let mut other = true;
    // SAFETY: reads the current buffer's 'hidden' state.
    if unsafe { buf_hide(Buf::current()) } {
        // SAFETY: caller contract; `fix_fname` hands back an owned name.
        // SAFETY: caller contract; `fix_fname` hands back an owned name,
        // which is freed once `otherfile` has read it.
        let full = unsafe { fix_fname(arg_name(argn)) };
        other = unsafe { otherfile(full) };
        unsafe { xfree(full.cast()) };
        if other {
            return true;
        }
    }
    let flags = CCGD_AW as c_int
        | CCGD_EXCMD as c_int
        | flag_if(!other, CCGD_MULTWIN)
        | flag_if(forceit, CCGD_FORCEIT);
    // SAFETY: `check_changed` only reads the buffer, and may prompt.
    !unsafe { check_changed(Buf::current_raw(), flags) }
}

/// Edit argument `argn`. A `:s…` command splits a window first; `:tab` opens
/// a tab page.
pub unsafe fn do_argfile(args: *mut ExArg, argn: c_int) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract.
    // SAFETY: `cmd` points at the command's own text, which is not empty.
    let is_split_cmd = unsafe { *args.cmd } as c_int == 's' as c_int;
    let forceit = args.forceit != 0;
    let cmdidx = args.cmdidx;
    let old_arg_idx = cur_arg_idx();
    if argn < 0 || argn >= argcount() {
        report_no_such_arg(argn);
        return;
    }
    // SAFETY: `argn` is in range and curbuf is valid.
    // SAFETY: `argn` is in range, checked a statement ago; reading the
    // entry outside the chain is safe because that check, not the
    // short-circuit, is what bounds it.
    let entry_fnum = unsafe { (*arg(argn)).ae_fnum };
    let refused = !is_split_cmd
        && entry_fnum != Buf::current().handle
        && !check_can_set_curbuf_forceit(args.forceit);
    if refused {
        return;
    }
    // SAFETY: plain mark bookkeeping over the current position.
    setpcmark();
    if is_split_cmd || cmdmod.with(|m| m.cmod_tab) != 0 {
        // Split the window, or create a new tab page, first.
        if win_split(0, 0).is_err() {
            return;
        }
        // RESET_BINDING: the new window scrolls and cursors on its own.
        // SAFETY: curwin is the window just created.
        Win::current().w_onebuf_opt.wo_scb = c_int::from(false);
        Win::current().w_onebuf_opt.wo_crb = c_int::from(false);
    } else {
        // SAFETY: `argn` is in range.
        if !unsafe { can_leave_curbuf(argn, forceit) } {
            return;
        }
    }
    set_cur_arg_idx(argn);
    if argn == argcount() - 1 && win_alist(Win::current()) == global_arglist() {
        arg_had_last.set(true);
    }
    // Edit the file, always at the last known line number.
    // SAFETY: the argument name outlives `do_ecmd`'s use of it, and `args` is
    // the caller's own live command block.
    let wp = Win::current();
    // SAFETY: `curwin` is live, so is its buffer.
    let hidden = unsafe { buf_hide(wp.buffer()) };
    let flags = EcmdFlags::HIDE.when(hidden) | EcmdFlags::FORCEIT.when(forceit);
    let name = arg_name(cur_arg_idx());
    let last = newlnum::LAST as LineNr;
    let none = ptr::null_mut();
    // SAFETY: as above; `do_ecmd` may fire autocommands, and nothing here
    // is held across it.
    let opened = unsafe { do_ecmd(0, name, none, args.raw(), last, flags, Some(wp.id())) };
    if opened.is_err() {
        // It failed (Abort for an already-edited file, say): restore the
        // argument index of whichever window is current now.
        set_cur_arg_idx(old_arg_idx);
    } else if cmdidx != CmdIdx::argdo {
        // Like Vi: set the mark where the cursor is in the file.
        // SAFETY: sets the `'` mark at the cursor.
        let _ = unsafe { setmark('\'' as c_int) };
    }
}

/// `:next` and the commands that behave like it.
pub unsafe fn ex_next(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the argument is NUL-terminated.
    let forceit = args.forceit != 0;
    let is_snext = args.cmdidx == CmdIdx::snext;
    // SAFETY: `arg` points at the command's own text.
    let has_arg = unsafe { *args.arg } as c_int != NUL;
    // Check for a changed buffer now: if this fails the argument list is not
    // redefined.
    // SAFETY: curbuf is valid; `check_changed` only reads it and may prompt.
    let flags = CCGD_AW as c_int | CCGD_EXCMD as c_int | flag_if(forceit, CCGD_FORCEIT);
    let buffer = Buf::current_raw();
    let blocked =
        unsafe { !buf_hide(Buf::new(buffer)) && !is_snext && check_changed(buffer, flags) };
    if blocked {
        return;
    }
    let argn = if has_arg {
        // Redefine the file list.
        // SAFETY: caller contract.
        if !unsafe { do_arglist(args.arg, ArgListOp::Set, 0, true) } {
            return;
        }
        0
    } else {
        // SAFETY: caller contract; the count is the command's range.
        cur_arg_idx() + args.line2 as c_int
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(args.raw(), argn) };
}

/// `:argdedupe` — drop every later argument naming the same file.
pub unsafe fn ex_argdedupe(_args: *mut ExArg) {
    let mut i = 0;
    while i < argcount() {
        // Expand each argument to a full path, to catch different paths
        // leading to the same file.
        // SAFETY: `i` is in range; `full_name_save` hands back an owned name.
        let first = unsafe { full_name_save((*arg(i)).ae_fname, false) };
        let mut j = i + 1;
        while j < argcount() {
            // SAFETY: `j` is in range, and the second name is freed as soon
            // as the comparison is done with it.
            // SAFETY: `j` is in range; `full_name_save` hands back an
            // owned name, freed once it has been compared.
            let second = unsafe { full_name_save((*arg(j)).ae_fname, false) };
            let duplicate = unsafe { path_fnamecmp(first, second) } == 0;
            unsafe { xfree(second.cast()) };
            if !duplicate {
                j += 1;
                continue;
            }
            // SAFETY: `j` is in range.
            unsafe { remove_arg(j) };
            let idx = cur_arg_idx();
            if idx == j {
                set_cur_arg_idx(i);
            } else if idx > j {
                set_cur_arg_idx(idx - 1);
            }
        }
        // SAFETY: `first` is ours to free and nothing refers to it now.
        unsafe { xfree(first as *mut c_void) };
        i += 1;
    }
}

/// `:argedit` — add the file to the list and edit it.
pub unsafe fn ex_argedit(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the insertion point is the command's range.
    let mut argn = if args.addr_count != 0 {
        args.line2 as c_int
    } else {
        cur_arg_idx() + 1
    };
    // Whether curbuf will be reused, in which case b_ffname will be set.
    // SAFETY: reads the current buffer's state.
    let curbuf_is_reusable = unsafe { curbuf_reusable() };
    // SAFETY: caller contract; the argument is NUL-terminated.
    if !unsafe { do_arglist(args.arg, ArgListOp::Add, argn, true) } {
        return;
    }
    // SAFETY: rebuilds the window title from the current buffer.
    unsafe { maketitle() };
    // SAFETY: curbuf is valid.
    let empty_curbuf = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY)
        && (Buf::current().b_ffname.is_null() || curbuf_is_reusable);
    if cur_arg_idx() == 0 && empty_curbuf {
        argn = 0;
    }
    // Edit the argument.
    if argn < argcount() {
        // SAFETY: caller contract.
        unsafe { do_argfile(args.raw(), argn) };
    }
}

/// `:argadd` — add the files to the list without editing them.
pub unsafe fn ex_argadd(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the insertion point is the command's range.
    let after = if args.addr_count > 0 {
        args.line2 as c_int
    } else {
        cur_arg_idx() + 1
    };
    // SAFETY: caller contract; the argument is NUL-terminated.
    unsafe { do_arglist(args.arg, ArgListOp::Add, after, false) };
    unsafe { maketitle() };
}

/// `:argdelete` — by range (`:2,3argdelete`, or bare for the current entry)
/// or by file pattern.
pub unsafe fn ex_argdelete(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    if arglist_is_locked() {
        return;
    }
    // SAFETY: caller contract; the argument is NUL-terminated.
    let by_range = unsafe { args.addr_count > 0 || *args.arg as c_int == NUL };
    // SAFETY: caller contract.
    if by_range {
        unsafe { delete_arg_range(args.raw()) };
    } else {
        unsafe { do_arglist(args.arg, ArgListOp::Delete, 0, false) };
    }
    unsafe { maketitle() };
}

/// The range half of `:argdelete`. Without a range it deletes the current
/// entry; a range reaching past the end is clamped to it.
unsafe fn delete_arg_range(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let mut args = unsafe { Ea::new(args) };
    // SAFETY: caller contract; the argument is NUL-terminated.
    let (addr_count, has_arg) = unsafe { (args.addr_count, *args.arg as c_int != NUL) };
    if addr_count == 0 {
        // ":argdel" works like ":.argdel".
        if cur_arg_idx() >= argcount() {
            crate::semsg!("E610: No argument to delete");
            return;
        }
        // SAFETY: caller contract.
        args.line2 = cur_arg_idx() + 1;
        args.line1 = args.line2;
    // ":1,4argdel": delete all the arguments in the range.
    // SAFETY: caller contract.
    } else if args.line2 > argcount() {
        // SAFETY: caller contract.
        args.line2 = argcount();
    }
    // SAFETY: caller contract.
    let (line1, line2) = (args.line1, args.line2);
    let count = line2 - line1 + 1;
    if has_arg {
        // Can't have both a range and an argument.
        crate::semsg!("E474: Invalid argument");
        return;
    }
    if count <= 0 {
        // Don't complain about ":%argdel" on an empty list.
        if line1 != 1 || line2 != 0 {
            crate::semsg!("E16: Invalid range");
        }
        return;
    }
    // SAFETY: the range sits inside the list, and every entry in it owns
    // its name.
    for gone in unsafe { cur_arglist() }.drain(as_count(line1 - 1)..as_count(line2)) {
        unsafe { xfree(gone.ae_fname.cast()) };
    }
    let idx = cur_arg_idx();
    if idx >= line2 {
        set_cur_arg_idx(idx - count);
    } else if idx > line1 {
        set_cur_arg_idx(line1);
    }
    if argcount() == 0 {
        set_cur_arg_idx(0);
    } else if cur_arg_idx() >= argcount() {
        set_cur_arg_idx(argcount() - 1);
    }
}

/// Completion source for `:argedit` and `:argdelete`: the argument names.
pub fn get_arglist_name(_expand: *mut Expand, idx: c_int) -> *mut c_char {
    if idx >= argcount() {
        return ptr::null_mut();
    }
    arg_name(idx)
}
