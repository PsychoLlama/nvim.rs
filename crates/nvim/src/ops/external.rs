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

use core::ffi::{c_char, c_int};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::types::{FAIL, NUL};

/// `:` for a Visual region, and the `!` filter `=` and `gq` fall back to.
///
/// Queues the command line into the read buffer and returns; nothing has run
/// when it does. The range is spelled the way a human would type it (`.`,
/// `$`, `.+N`) so that `:` in the command-line history reads well and can be
/// repeated.
///
/// # Safety
/// `oap` must point to a live `oparg_T`.
pub(crate) unsafe fn op_colon(oap: *mut oparg_T) {
    unsafe {
        stuff_readbuf_char(':' as c_int);
        if (*oap).is_VIsual {
            stuff_readbuf(c"'<,'>".as_ptr());
        } else {
            // Make the range look nice, so it can be repeated.
            if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum {
                stuff_readbuf_char('.' as c_int);
            } else {
                stuff_readbuf_number((*oap).start.lnum as c_int);
            }

            // When using !! on a closed fold the range ".!" works best to
            // operate on: it is made the whole closed fold later.
            let mut end_of_start_fold: linenr_T = (*oap).start.lnum;
            hasFolding(
                curwin.get(),
                (*oap).start.lnum,
                ::core::ptr::null_mut(),
                &raw mut end_of_start_fold,
            );
            if (*oap).end.lnum != (*oap).start.lnum && (*oap).end.lnum != end_of_start_fold {
                // Make it a range with the end line.
                stuff_readbuf_char(',' as c_int);
                if (*oap).end.lnum == (*curwin.get()).w_cursor.lnum {
                    stuff_readbuf_char('.' as c_int);
                } else if (*oap).end.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    stuff_readbuf_char('$' as c_int);
                } else if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum
                    // Not ".+number" for a closed fold: that would count the
                    // folded lines twice.
                    && !hasFolding(
                        curwin.get(),
                        (*oap).end.lnum,
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                    )
                {
                    stuff_readbuf(c".+".as_ptr());
                    stuff_readbuf_number((*oap).line_count as c_int - 1);
                } else {
                    stuff_readbuf_number((*oap).end.lnum as c_int);
                }
            }
        }
        if (*oap).op_type != OP_COLON {
            stuff_readbuf(c"!".as_ptr());
        }
        if (*oap).op_type == OP_INDENT {
            stuff_readbuf(get_equalprg());
            stuff_readbuf(c"\n".as_ptr());
        } else if (*oap).op_type == OP_FORMAT {
            if *(*curbuf.get()).b_p_fp as c_int != NUL {
                stuff_readbuf((*curbuf.get()).b_p_fp);
            } else if *p_fp.get() as c_int != NUL {
                stuff_readbuf(p_fp.get());
            } else {
                stuff_readbuf(c"fmt".as_ptr());
            }
            // The trailing `']` puts the cursor back at the end of the range
            // once the filter has replaced it.
            stuff_readbuf(c"\n']".as_ptr());
        }
    }
}

/// The parsed callback behind 'operatorfunc'.
static OPFUNC_CB: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut(),
    },
    type_0: kCallbackNone,
});

/// Parse a new 'operatorfunc' value; `E474` if it names nothing callable.
///
/// # Safety
/// The option's current value must be a valid C string.
pub unsafe fn did_set_operatorfunc(_args: *mut optset_T) -> *const c_char {
    unsafe {
        if option_set_callback_func(p_opfunc.get(), OPFUNC_CB.ptr()) == FAIL {
            return &raw const e_invarg as *const c_char;
        }
        ::core::ptr::null()
    }
}

/// Mark the 'operatorfunc' callback with `copy_id` so the collector keeps it.
///
/// # Safety
/// Called from the garbage collector, with the eval heap consistent.
pub unsafe fn set_ref_in_opfunc(copy_id: c_int) -> bool {
    unsafe {
        set_ref_in_callback(
            OPFUNC_CB.ptr(),
            copy_id,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        )
    }
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
/// `oap` must point to a live `oparg_T`.
pub(crate) unsafe fn op_function(oap: *const oparg_T) {
    unsafe {
        let orig_start: pos_T = (*curbuf.get()).b_op_start;
        let orig_end: pos_T = (*curbuf.get()).b_op_end;

        if *p_opfunc.get() as c_int == NUL {
            emsg(gettext(c"E774: 'operatorfunc' is empty".as_ptr()));
            return;
        }

        // Set '[ and '] to the text to be operated on.
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
        if (*oap).motion_type != kMTLineWise && !(*oap).inclusive {
            // Exclude the end position.
            decl(&raw mut (*curbuf.get()).b_op_end);
        }

        let kind = match (*oap).motion_type {
            kMTLineWise => c"line",
            kMTBlockWise => c"block",
            _ => c"char",
        };
        let mut argv: [typval_T; 2] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 2];
        argv[0].v_type = VAR_STRING;
        argv[0].vval.v_string = kind.as_ptr() as *mut c_char;

        // Reset virtual_op so that 'virtualedit' can be changed in the
        // function, and finish_op so that mode() returns the right value.
        let save_virtual_op: Option<bool> = virtual_op.get();
        virtual_op.set(None);
        let save_finish_op: bool = finish_op.get();
        finish_op.set(false);

        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if callback_call(
            OPFUNC_CB.ptr(),
            1,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        ) {
            tv_clear(&raw mut rettv);
        }

        virtual_op.set(save_virtual_op);
        finish_op.set(save_finish_op);
        if cmdmod_has(CmdModFlags::LOCKMARKS) {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
    }
}
