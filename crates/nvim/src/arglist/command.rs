//! The Ex commands that drive the argument list: `:args`, `:arglocal` and
//! `:argglobal`; `:argadd`, `:argedit`, `:argdelete` and `:argdedupe`; and
//! the `:next`/`:previous`/`:first`/`:last`/`:argument` family that walks
//! it, each in its window-splitting `:s…` form too.
//!
//! A command that changes the list hands [`do_arglist`] a copy of its
//! argument: the autocommands adding a file runs must not find the command
//! line borrowed.

#![forbid(unsafe_code)]

use super::*;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::memline::MlFlags;
use crate::types::CmdIdx;
use crate::winlayer::{Buf, Win};

// ---------------------------------------------------------------------------
// The Ex commands.

/// `:args`, `:arglocal` and `:argglobal`.
pub fn ex_args(excmd: &mut ExArg) {
    let cmdidx = excmd.cmdidx;
    if cmdidx != CmdIdx::args {
        if arglist_is_locked() {
            return;
        }
        // Dropping the window's reference is what makes room for the new
        // list.
        alist_unlink(cur_alist());
        if cmdidx == CmdIdx::argglobal {
            Win::current().w_alist = ArgListId::GLOBAL;
        } else {
            alist_new();
        }
    }
    // ":args file ..": define a new argument list, handled like ":next".
    // Also for ":arglocal file .." and ":argglobal file ..".
    if c_int::from(excmd.line.byte_at(excmd.line.arg)) != NUL {
        if arglist_is_locked() {
            return;
        }
        ex_next(excmd);
        return;
    }
    if cmdidx == CmdIdx::args {
        list_args();
    } else if cmdidx == CmdIdx::arglocal {
        copy_global_arglist();
    }
}

/// `:args` with no argument: list the arguments, the current one bracketed.
fn list_args() {
    if argcount() <= 0 {
        // Empty argument list.
        return;
    }
    // Overwrite the command: for a short list no scrolling and hence no
    // wait_return() is needed.
    gotocmdline(true);
    // Copies: a more-prompt may wait in the middle of the listing.
    let names = arg_names(cur_alist());
    let items: Vec<&CStr> = names.iter().map(XString::as_cstr).collect();
    list_in_columns(&items, cur_arg_idx());
}

/// `:arglocal` with no argument: copy the global list into the window's own,
/// which is never the global one here.
fn copy_global_arglist() {
    let copies = with_list(ArgListId::GLOBAL, |al| al.entries.clone()).unwrap_or_default();
    with_list(cur_alist(), |al| al.entries.extend(copies));
}

/// `:previous`, `:sprevious`, `:Next` and `:sNext`.
pub fn ex_previous(excmd: &mut ExArg) {
    let back = cur_arg_idx() - excmd.line2 as c_int;
    // If already past the last one, go to the last one.
    let argn = if back >= argcount() {
        argcount() - 1
    } else {
        back
    };
    do_argfile(excmd, argn);
}

/// `:rewind`, `:first`, `:sfirst` and `:srewind`.
pub fn ex_rewind(excmd: &mut ExArg) {
    do_argfile(excmd, 0);
}

/// `:last` and `:slast`.
pub fn ex_last(excmd: &mut ExArg) {
    do_argfile(excmd, argcount() - 1);
}

/// `:argument` and `:sargument`.
pub fn ex_argument(excmd: &mut ExArg) {
    let argn = if excmd.addr_count > 0 {
        excmd.line2 as c_int - 1
    } else {
        cur_arg_idx()
    };
    do_argfile(excmd, argn);
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
fn can_leave_curbuf(argn: c_int, forceit: bool) -> bool {
    let mut other = true;
    if buf_hide(Buf::current()) {
        // A copy: the name is resolved and compared, and the buffer may go.
        if let Some(name) = arg_name(cur_alist(), argn) {
            let full = fix_fname(name.as_cstr());
            other = otherfile(full.as_cstr());
        }
        if other {
            return true;
        }
    }
    let flags = CCGD_AW as c_int
        | CCGD_EXCMD as c_int
        | flag_if(!other, CCGD_MULTWIN)
        | flag_if(forceit, CCGD_FORCEIT);
    !check_changed(Buf::current(), flags)
}

/// Edit argument `argn`. A `:s…` command splits a window first; `:tab` opens
/// a tab page.
pub fn do_argfile(excmd: &mut ExArg, argn: c_int) {
    let is_split_cmd = c_int::from(excmd.line.byte_at(excmd.line.cmd)) == 's' as c_int;
    let forceit = excmd.forceit;
    let cmdidx = excmd.cmdidx;
    let old_arg_idx = cur_arg_idx();
    if argn < 0 || argn >= argcount() {
        report_no_such_arg(argn);
        return;
    }
    let entry_fnum = entry_fnum(cur_alist(), argn).unwrap_or(0);
    let refused = !is_split_cmd
        && entry_fnum != Buf::current().handle
        && !check_can_set_curbuf_forceit(c_int::from(excmd.forceit));
    if refused {
        return;
    }
    setpcmark();
    if is_split_cmd || cmdmod.with(|m| m.cmod_tab) != 0 {
        // Split the window, or create a new tab page, first.
        if win_split(0, 0).is_err() {
            return;
        }
        // RESET_BINDING: the new window scrolls and cursors on its own.
        Win::current().w_onebuf_opt.wo_scb = c_int::from(false);
        Win::current().w_onebuf_opt.wo_crb = c_int::from(false);
    } else if !can_leave_curbuf(argn, forceit) {
        return;
    }
    set_cur_arg_idx(argn);
    if argn == argcount() - 1 && cur_alist() == ArgListId::GLOBAL {
        ARG_HAD_LAST.set(true);
    }
    // Edit the file, always at the last known line number.
    let wp = Win::current();
    let hidden = buf_hide(wp.buffer());
    let flags = EcmdFlags::HIDE.when(hidden) | EcmdFlags::FORCEIT.when(forceit);
    // A copy: `do_ecmd` may fire autocommands that change the list.
    let name = arg_name(cur_alist(), cur_arg_idx());
    let last = newlnum::LAST as LineNr;
    let opened = do_ecmd(
        0,
        name.as_ref().map(XString::as_cstr),
        None,
        Some(excmd),
        last,
        flags,
        Some(wp.id()),
    );
    if opened.is_err() {
        // It failed (Abort for an already-edited file, say): restore the
        // argument index of whichever window is current now.
        set_cur_arg_idx(old_arg_idx);
    } else if cmdidx != CmdIdx::argdo {
        // Like Vi: set the mark where the cursor is in the file.
        let _ = setmark('\'' as c_int);
    }
}

/// `:next` and the commands that behave like it.
pub fn ex_next(excmd: &mut ExArg) {
    let forceit = excmd.forceit;
    let is_snext = excmd.cmdidx == CmdIdx::snext;
    let has_arg = c_int::from(excmd.line.byte_at(excmd.line.arg)) != NUL;
    // Check for a changed buffer now: if this fails the argument list is not
    // redefined.
    let flags = CCGD_AW as c_int | CCGD_EXCMD as c_int | flag_if(forceit, CCGD_FORCEIT);
    let buffer = Buf::current();
    let blocked = !buf_hide(buffer) && !is_snext && check_changed(buffer, flags);
    if blocked {
        return;
    }
    let argn = if has_arg {
        // Redefine the file list.
        let arg = excmd.line.arg().to_vec();
        if !do_arglist(&arg, ArgListOp::Set, 0, true) {
            return;
        }
        0
    } else {
        cur_arg_idx() + excmd.line2 as c_int
    };
    do_argfile(excmd, argn);
}

/// `:argdedupe` — drop every later argument naming the same file.
pub fn ex_argdedupe(_excmd: &mut ExArg) {
    // Each argument's own name (not its buffer's) expanded to a full path,
    // to catch different paths leading to the same file. Nothing here runs
    // user code, so the names are resolved once rather than per comparison.
    let mut full: Vec<XString> = with_list(cur_alist(), |al| {
        al.entries
            .iter()
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
    .iter()
    .map(|name| full_name_save(name.as_cstr(), false))
    .collect();
    let mut i = 0;
    while i < full.len() {
        let mut j = i + 1;
        while j < full.len() {
            if path_fnamecmp(&full[i], &full[j]) != 0 {
                j += 1;
                continue;
            }
            full.remove(j);
            let j = count_of(j);
            remove_arg(j);
            let idx = cur_arg_idx();
            if idx == j {
                set_cur_arg_idx(count_of(i));
            } else if idx > j {
                set_cur_arg_idx(idx - 1);
            }
        }
        i += 1;
    }
}

/// `:argedit` — add the file to the list and edit it.
pub fn ex_argedit(excmd: &mut ExArg) {
    let mut argn = if excmd.addr_count != 0 {
        excmd.line2 as c_int
    } else {
        cur_arg_idx() + 1
    };
    // Whether curbuf will be reused, in which case b_ffname will be set.
    let curbuf_is_reusable = curbuf_reusable();
    let arg = excmd.line.arg().to_vec();
    if !do_arglist(&arg, ArgListOp::Add, argn, true) {
        return;
    }
    maketitle();
    let empty_curbuf = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY)
        && (Buf::current().name.full().is_none() || curbuf_is_reusable);
    if cur_arg_idx() == 0 && empty_curbuf {
        argn = 0;
    }
    // Edit the argument.
    if argn < argcount() {
        do_argfile(excmd, argn);
    }
}

/// `:argadd` — add the files to the list without editing them.
pub fn ex_argadd(excmd: &mut ExArg) {
    let after = if excmd.addr_count > 0 {
        excmd.line2 as c_int
    } else {
        cur_arg_idx() + 1
    };
    let arg = excmd.line.arg().to_vec();
    do_arglist(&arg, ArgListOp::Add, after, false);
    maketitle();
}

/// `:argdelete` — by range (`:2,3argdelete`, or bare for the current entry)
/// or by file pattern.
pub fn ex_argdelete(excmd: &mut ExArg) {
    if arglist_is_locked() {
        return;
    }
    let by_range = excmd.addr_count > 0 || excmd.line.byte_at(excmd.line.arg) == 0;
    if by_range {
        delete_arg_range(excmd);
    } else {
        let arg = excmd.line.arg().to_vec();
        do_arglist(&arg, ArgListOp::Delete, 0, false);
    }
    maketitle();
}

/// The range half of `:argdelete`. Without a range it deletes the current
/// entry; a range reaching past the end is clamped to it.
fn delete_arg_range(excmd: &mut ExArg) {
    let (addr_count, has_arg) = (excmd.addr_count, excmd.line.byte_at(excmd.line.arg) != 0);
    if addr_count == 0 {
        // ":argdel" works like ":.argdel".
        if cur_arg_idx() >= argcount() {
            crate::semsg!("E610: No argument to delete");
            return;
        }
        excmd.line2 = cur_arg_idx() + 1;
        excmd.line1 = excmd.line2;
    // ":1,4argdel": delete all the arguments in the range.
    } else if excmd.line2 > argcount() {
        excmd.line2 = argcount();
    }
    let (line1, line2) = (excmd.line1, excmd.line2);
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
    let range = as_count(line1 - 1)..as_count(line2);
    with_list(cur_alist(), |al| al.entries.drain(range).for_each(drop));
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
pub fn get_arglist_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let idx = c_int::try_from(idx).ok()?;
    arg_name(cur_alist(), idx).map(|name| Candidate::Owned(name.as_cstr().to_owned()))
}
