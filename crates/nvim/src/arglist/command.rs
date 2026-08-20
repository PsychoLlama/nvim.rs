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

// ---------------------------------------------------------------------------
// The Ex commands.

/// `:args`, `:arglocal` and `:argglobal`.
pub unsafe fn ex_args(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let cmdidx = unsafe { (*eap).cmdidx } as c_int;
    if cmdidx != CMD_args as c_int {
        if arglist_is_locked() {
            return;
        }
        // SAFETY: curwin always has an argument list, and dropping the
        // reference to it is what makes room for the new one.
        unsafe {
            alist_unlink(win_alist(curwin.get()));
            if cmdidx == CMD_argglobal as c_int {
                (*curwin.get()).w_alist = global_arglist();
            } else {
                alist_new();
            }
        }
    }
    // ":args file ..": define a new argument list, handled like ":next".
    // Also for ":arglocal file .." and ":argglobal file ..".
    // SAFETY: an ex-command argument is NUL-terminated.
    if unsafe { *(*eap).arg } as c_int != NUL {
        if arglist_is_locked() {
            return;
        }
        // SAFETY: caller contract.
        unsafe { ex_next(eap) };
        return;
    }
    if cmdidx == CMD_args as c_int {
        // SAFETY: every entry of the current list has a name.
        unsafe { list_args() };
    } else if cmdidx == CMD_arglocal as c_int {
        // SAFETY: both lists are valid.
        unsafe { copy_global_arglist() };
    }
}

/// `:args` with no argument: list the arguments, the current one bracketed.
///
/// # Safety
///
/// The current window's argument list must be valid.
unsafe fn list_args() {
    if argcount() <= 0 {
        // Empty argument list.
        return;
    }
    // Overwrite the command: for a short list no scrolling and hence no
    // wait_return() is needed.
    // SAFETY: every entry's name is NUL-terminated and outlives the listing.
    unsafe {
        gotocmdline(true);
        let items: Vec<&CStr> = (0..argcount())
            .map(|i| CStr::from_ptr(arg_name(i)))
            .collect();
        list_in_columns(&items, cur_arg_idx());
    }
}

/// `:arglocal` with no argument: copy the global list into the window's own,
/// skipping entries that have lost their name.
///
/// # Safety
///
/// Both the global and the current window's list must be valid.
unsafe fn copy_global_arglist() {
    // SAFETY: `ga_grow` reserves a slot for every entry that can be copied.
    unsafe {
        let al = win_alist(curwin.get());
        let count = alist_count(global_arglist());
        ga_grow(&raw mut (*al).al_ga, count);
        for i in 0..count {
            let src = alist_arg(global_arglist(), i);
            if (*src).ae_fname.is_null() {
                continue;
            }
            let at = (*al).al_ga.ga_len;
            (*alist_arg(al, at)).ae_fname = xstrdup((*src).ae_fname);
            (*alist_arg(al, at)).ae_fnum = (*src).ae_fnum;
            (*al).al_ga.ga_len += 1;
        }
    }
}

/// `:previous`, `:sprevious`, `:Next` and `:sNext`.
pub unsafe fn ex_previous(eap: *mut exarg_T) {
    // SAFETY: caller contract; the count is the command's range.
    let back = cur_arg_idx() - unsafe { (*eap).line2 } as c_int;
    // If already past the last one, go to the last one.
    let argn = if back >= argcount() {
        argcount() - 1
    } else {
        back
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(eap, argn) };
}

/// `:rewind`, `:first`, `:sfirst` and `:srewind`.
pub unsafe fn ex_rewind(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    unsafe { do_argfile(eap, 0) };
}

/// `:last` and `:slast`.
pub unsafe fn ex_last(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    unsafe { do_argfile(eap, argcount() - 1) };
}

/// `:argument` and `:sargument`.
pub unsafe fn ex_argument(eap: *mut exarg_T) {
    // SAFETY: caller contract; the argument number is the command's range.
    let argn = unsafe {
        if (*eap).addr_count > 0 {
            (*eap).line2 as c_int - 1
        } else {
            cur_arg_idx()
        }
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(eap, argn) };
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
    if unsafe { buf_hide(curbuf.get()) } {
        // SAFETY: caller contract; `fix_fname` hands back an owned name.
        other = unsafe {
            let p = fix_fname(arg_name(argn));
            let other = otherfile(p);
            xfree(p as *mut c_void);
            other
        };
        if other {
            return true;
        }
    }
    let flags = CCGD_AW as c_int
        | CCGD_EXCMD as c_int
        | flag_if(!other, CCGD_MULTWIN)
        | flag_if(forceit, CCGD_FORCEIT);
    // SAFETY: `check_changed` only reads the buffer, and may prompt.
    !unsafe { check_changed(curbuf.get(), flags) }
}

/// Edit argument `argn`. A `:s…` command splits a window first; `:tab` opens
/// a tab page.
pub unsafe fn do_argfile(eap: *mut exarg_T, argn: c_int) {
    // SAFETY: caller contract.
    let (is_split_cmd, forceit, cmdidx) = unsafe {
        (
            *(*eap).cmd as c_int == 's' as c_int,
            (*eap).forceit != 0,
            (*eap).cmdidx as c_int,
        )
    };
    let old_arg_idx = cur_arg_idx();
    if argn < 0 || argn >= argcount() {
        report_no_such_arg(argn);
        return;
    }
    // SAFETY: `argn` is in range and curbuf is valid.
    let refused = unsafe {
        !is_split_cmd
            && (*arg(argn)).ae_fnum != (*curbuf.get()).handle
            && !check_can_set_curbuf_forceit((*eap).forceit)
    };
    if refused {
        return;
    }
    // SAFETY: plain mark bookkeeping over the current position.
    unsafe { setpcmark() };
    if is_split_cmd || cmdmod.with(|m| m.cmod_tab) != 0 {
        // Split the window, or create a new tab page, first.
        if win_split(0, 0) == FAIL {
            return;
        }
        // RESET_BINDING: the new window scrolls and cursors on its own.
        // SAFETY: curwin is the window just created.
        unsafe {
            (*curwin.get()).w_onebuf_opt.wo_scb = c_int::from(false);
            (*curwin.get()).w_onebuf_opt.wo_crb = c_int::from(false);
        }
    } else {
        // SAFETY: `argn` is in range.
        if !unsafe { can_leave_curbuf(argn, forceit) } {
            return;
        }
    }
    set_cur_arg_idx(argn);
    if argn == argcount() - 1 && win_alist(curwin.get()) == global_arglist() {
        arg_had_last.set(true);
    }
    // Edit the file, always at the last known line number.
    // SAFETY: the argument name outlives `do_ecmd`'s use of it, and `eap` is
    // the caller's own live command block.
    let opened = unsafe {
        let wp = curwin.get();
        let flags = flag_if(buf_hide((*wp).w_buffer), ECMD_HIDE) + flag_if(forceit, ECMD_FORCEIT);
        do_ecmd(
            0,
            arg_name(cur_arg_idx()),
            ptr::null_mut(),
            eap,
            ECMD_LAST as linenr_T,
            flags,
            wp,
        )
    };
    if opened == FAIL {
        // It failed (Abort for an already-edited file, say): restore the
        // argument index of whichever window is current now.
        set_cur_arg_idx(old_arg_idx);
    } else if cmdidx != CMD_argdo as c_int {
        // Like Vi: set the mark where the cursor is in the file.
        // SAFETY: sets the `'` mark at the cursor.
        unsafe { setmark('\'' as c_int) };
    }
}

/// `:next` and the commands that behave like it.
pub unsafe fn ex_next(eap: *mut exarg_T) {
    // SAFETY: caller contract; the argument is NUL-terminated.
    let (forceit, is_snext, has_arg) = unsafe {
        (
            (*eap).forceit != 0,
            (*eap).cmdidx as c_int == CMD_snext as c_int,
            *(*eap).arg as c_int != NUL,
        )
    };
    // Check for a changed buffer now: if this fails the argument list is not
    // redefined.
    // SAFETY: curbuf is valid; `check_changed` only reads it and may prompt.
    let blocked = unsafe {
        !buf_hide(curbuf.get())
            && !is_snext
            && check_changed(
                curbuf.get(),
                CCGD_AW as c_int | CCGD_EXCMD as c_int | flag_if(forceit, CCGD_FORCEIT),
            )
    };
    if blocked {
        return;
    }
    let argn = if has_arg {
        // Redefine the file list.
        // SAFETY: caller contract.
        if !unsafe { do_arglist((*eap).arg, ArgListOp::Set, 0, true) } {
            return;
        }
        0
    } else {
        // SAFETY: caller contract; the count is the command's range.
        cur_arg_idx() + unsafe { (*eap).line2 } as c_int
    };
    // SAFETY: caller contract.
    unsafe { do_argfile(eap, argn) };
}

/// `:argdedupe` — drop every later argument naming the same file.
pub unsafe fn ex_argdedupe(_eap: *mut exarg_T) {
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
            let duplicate = unsafe {
                let second = full_name_save((*arg(j)).ae_fname, false);
                let duplicate = path_fnamecmp(first, second) == 0;
                xfree(second as *mut c_void);
                duplicate
            };
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
pub unsafe fn ex_argedit(eap: *mut exarg_T) {
    // SAFETY: caller contract; the insertion point is the command's range.
    let mut argn = unsafe {
        if (*eap).addr_count != 0 {
            (*eap).line2 as c_int
        } else {
            cur_arg_idx() + 1
        }
    };
    // Whether curbuf will be reused, in which case b_ffname will be set.
    // SAFETY: reads the current buffer's state.
    let curbuf_is_reusable = unsafe { curbuf_reusable() };
    // SAFETY: caller contract; the argument is NUL-terminated.
    if !unsafe { do_arglist((*eap).arg, ArgListOp::Add, argn, true) } {
        return;
    }
    // SAFETY: rebuilds the window title from the current buffer.
    unsafe { maketitle() };
    // SAFETY: curbuf is valid.
    let empty_curbuf = unsafe {
        (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0
            && ((*curbuf.get()).b_ffname.is_null() || curbuf_is_reusable)
    };
    if cur_arg_idx() == 0 && empty_curbuf {
        argn = 0;
    }
    // Edit the argument.
    if argn < argcount() {
        // SAFETY: caller contract.
        unsafe { do_argfile(eap, argn) };
    }
}

/// `:argadd` — add the files to the list without editing them.
pub unsafe fn ex_argadd(eap: *mut exarg_T) {
    // SAFETY: caller contract; the insertion point is the command's range.
    let after = unsafe {
        if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
        } else {
            cur_arg_idx() + 1
        }
    };
    // SAFETY: caller contract; the argument is NUL-terminated.
    unsafe {
        do_arglist((*eap).arg, ArgListOp::Add, after, false);
        maketitle();
    }
}

/// `:argdelete` — by range (`:2,3argdelete`, or bare for the current entry)
/// or by file pattern.
pub unsafe fn ex_argdelete(eap: *mut exarg_T) {
    if arglist_is_locked() {
        return;
    }
    // SAFETY: caller contract; the argument is NUL-terminated.
    let by_range = unsafe { (*eap).addr_count > 0 || *(*eap).arg as c_int == NUL };
    // SAFETY: caller contract.
    unsafe {
        if by_range {
            delete_arg_range(eap);
        } else {
            do_arglist((*eap).arg, ArgListOp::Delete, 0, false);
        }
        maketitle();
    }
}

/// The range half of `:argdelete`. Without a range it deletes the current
/// entry; a range reaching past the end is clamped to it.
unsafe fn delete_arg_range(eap: *mut exarg_T) {
    // SAFETY: caller contract; the argument is NUL-terminated.
    let (addr_count, has_arg) = unsafe { ((*eap).addr_count, *(*eap).arg as c_int != NUL) };
    if addr_count == 0 {
        // ":argdel" works like ":.argdel".
        if cur_arg_idx() >= argcount() {
            crate::semsg!("E610: No argument to delete");
            return;
        }
        // SAFETY: caller contract.
        unsafe {
            (*eap).line2 = cur_arg_idx() + 1;
            (*eap).line1 = (*eap).line2;
        }
    // ":1,4argdel": delete all the arguments in the range.
    // SAFETY: caller contract.
    } else if unsafe { (*eap).line2 } > argcount() {
        // SAFETY: caller contract.
        unsafe { (*eap).line2 = argcount() };
    }
    // SAFETY: caller contract.
    let (line1, line2) = unsafe { ((*eap).line1, (*eap).line2) };
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
    // SAFETY: the range sits inside the list, and the `memmove` moves
    // exactly the tail that follows it.
    unsafe {
        for i in line1..=line2 {
            xfree((*arg(i - 1)).ae_fname as *mut c_void);
        }
        memmove(
            arg(line1 - 1) as *mut c_void,
            arg(line2) as *const c_void,
            ((argcount() - line2) as size_t).wrapping_mul(size_of::<aentry_T>()),
        );
    }
    set_argcount(argcount() - count);
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
pub fn get_arglist_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    if idx >= argcount() {
        return ptr::null_mut();
    }
    arg_name(idx)
}
