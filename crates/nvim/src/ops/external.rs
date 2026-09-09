//! The two operators that hand the region to someone else.
//!
//! [`op_colon`] builds a `:` command line with the region's line range already
//! filled in and *stuffs* it into the read buffer -- it does not run anything,
//! `do_cmdline` does the rest once the main loop reads what was queued. That
//! is the whole of `:` in Visual mode, and also how `=` and `gq` reach an
//! external program, because 'equalprg'/'formatprg' turn them into a `!`
//! filter command.
//!
//! [`op_function`] is `g@`: it sets `'[`/`']` to the region and calls
//! 'operatorfunc' with `"line"`, `"char"` or `"block"`. [`OPFUNC_CB`] is the
//! parsed callback behind that option, so its setter and the garbage
//! collector's mark hook live here too.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::types::NUL;

/// `:` for a Visual region, and the `!` filter `=` and `gq` fall back to.
///
/// Queues the command line into the read buffer and returns; nothing has run
/// when it does. The range is spelled the way a human would type it (`.`,
/// `$`, `.+N`) so that `:` in the command-line history reads well and can be
/// repeated.
///
/// # Safety
/// `op` must point to a live `OpArg`.
pub(crate) unsafe fn op_colon(op: *mut OpArg) {
    // SAFETY: the caller's promise -- a live `OpArg`. Every string queued
    // below is either a literal of this file's or a NUL-terminated option.
    let op = unsafe { Op::new(op) };
    stuff_readbuf_char(':' as c_int);
    if op.is_visual {
        unsafe { stuff_readbuf(c"'<,'>".as_ptr()) };
    } else {
        // Make the range look nice, so it can be repeated.
        if op.start.lnum == Win::current().w_cursor.lnum {
            stuff_readbuf_char('.' as c_int);
        } else {
            stuff_readbuf_number(op.start.lnum as c_int);
        }

        // When using !! on a closed fold the range ".!" works best to
        // operate on: it is made the whole closed fold later.
        let end_of_start_fold = Win::current().fold_last(op.start.lnum);
        if op.end.lnum != op.start.lnum && op.end.lnum != end_of_start_fold {
            // Make it a range with the end line.
            stuff_readbuf_char(',' as c_int);
            if op.end.lnum == Win::current().w_cursor.lnum {
                stuff_readbuf_char('.' as c_int);
            } else if op.end.lnum == Buf::current().line_count() {
                stuff_readbuf_char('$' as c_int);
            } else if op.start.lnum == Win::current().w_cursor.lnum
                // Not ".+number" for a closed fold: that would count the
                // folded lines twice.
                && !Win::current().fold_span(op.end.lnum).0
            {
                unsafe { stuff_readbuf(c".+".as_ptr()) };
                stuff_readbuf_number(op.line_count as c_int - 1);
            } else {
                stuff_readbuf_number(op.end.lnum as c_int);
            }
        }
    }
    if op.op_type != OpType::Colon {
        unsafe { stuff_readbuf(c"!".as_ptr()) };
    }
    if op.op_type == OpType::Indent {
        unsafe { stuff_readbuf(get_equalprg()) };
        unsafe { stuff_readbuf(c"\n".as_ptr()) };
    } else if op.op_type == OpType::Format {
        if c_int::from(unsafe { *Buf::current().b_p_fp }) != NUL {
            unsafe { stuff_readbuf(Buf::current().b_p_fp) };
        } else if c_int::from(unsafe { *p_fp.get() }) != NUL {
            unsafe { stuff_readbuf(p_fp.get()) };
        } else {
            unsafe { stuff_readbuf(c"fmt".as_ptr()) };
        }
        // The trailing `']` puts the cursor back at the end of the range
        // once the filter has replaced it.
        unsafe { stuff_readbuf(c"\n']".as_ptr()) };
    }
}

/// The parsed callback behind 'operatorfunc'.
static OPFUNC_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);

/// The parsed `'operatorfunc'`.
///
/// The address, because every operation the tree has on a callback —
/// parsing an option into it, marking it for the collector, copying it,
/// calling it — takes a `*mut Callback`.
fn global_opfunc() -> *mut Callback {
    OPFUNC_CB.ptr()
}

/// Parse a new 'operatorfunc' value; `E474` if it names nothing callable.
///
/// # Safety
/// The option's current value must be a valid C string.
pub unsafe fn did_set_operatorfunc(_args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the caller's promise -- 'operatorfunc' is a valid C string.
    if unsafe { option_set_callback_func(p_opfunc.get(), global_opfunc()) }.is_err() {
        return Some(e_invarg);
    }
    None
}

/// Mark the 'operatorfunc' callback with `copy_id` so the collector keeps it.
///
/// # Safety
/// Called from the garbage collector, with the eval heap consistent.
pub unsafe fn set_ref_in_opfunc(copy_id: c_int) -> bool {
    let (ht, list) = (::core::ptr::null_mut(), ::core::ptr::null_mut());
    // SAFETY: the caller's promise -- the eval heap is consistent.
    unsafe { set_ref_in_callback(global_opfunc(), copy_id, ht, list) }
}

/// `g@` -- call 'operatorfunc' with the region in `'[`/`']`.
///
/// The callback runs arbitrary Vimscript, so everything it might reasonably
/// want to change is saved and restored around it: 'virtualedit' (through
/// `virtual_op`, which would otherwise pin the old value) and `finish_op`, so
/// that `mode()` answers what the user sees rather than "an operator is
/// pending". `:lockmarks` restores the marks afterwards.
///
/// # Safety
/// `op` must point to a live `OpArg`.
pub(crate) unsafe fn op_function(op: *const OpArg) {
    // SAFETY: the caller's promise -- a live `OpArg`. 'operatorfunc' is a
    // NUL-terminated option string, and `b_op_end` is a live position of the
    // current buffer.
    let op = unsafe { Op::new(op.cast_mut()) };
    let orig_start: Pos = Buf::current().b_op_start;
    let orig_end: Pos = Buf::current().b_op_end;

    if c_int::from(unsafe { *p_opfunc.get() }) == NUL {
        emsg(gettext(c"E774: 'operatorfunc' is empty"));
        return;
    }

    // Set '[ and '] to the text to be operated on.
    Buf::current().b_op_start = op.start;
    Buf::current().b_op_end = op.end;
    if op.motion_type != kMTLineWise && !op.inclusive {
        // Exclude the end position.
        unsafe { decl(&mut Buf::current().b_op_end) };
    }

    let kind = match op.motion_type {
        kMTLineWise => c"line",
        kMTBlockWise => c"block",
        _ => c"char",
    };
    let mut argv: [TypVal; 2] = [TV_INITIAL_VALUE; 2];
    argv[0].write_string(kind.as_ptr() as *mut c_char);

    // Reset virtual_op so that 'virtualedit' can be changed in the
    // function, and finish_op so that mode() returns the right value.
    let save_virtual_op: Option<bool> = virtual_op.get();
    virtual_op.set(None);
    let save_finish_op: bool = finish_op.get();
    finish_op.set(false);

    let mut rettv: TypVal = TV_INITIAL_VALUE;
    let args = (&raw mut argv).cast::<TypVal>();
    if unsafe { callback_call(global_opfunc(), 1, args, &raw mut rettv) } {
        unsafe { tv_clear(&raw mut rettv) };
    }

    virtual_op.set(save_virtual_op);
    finish_op.set(save_finish_op);
    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = orig_start;
        Buf::current().b_op_end = orig_end;
    }
}
