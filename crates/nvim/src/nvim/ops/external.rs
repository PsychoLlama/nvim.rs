//! The two operators that hand the region to someone else.
//!
//! `op_colon` builds a `:` command line with the region's line range already
//! filled in (and, for `!`, the 'formatprg'/'equalprg' spelling) and leaves
//! it for the user to complete.  `op_function` is `g@`: it sets `'[`/`']`
//! to the region and calls 'operatorfunc' with `"line"`, `"char"` or
//! `"block"`.  `opfunc_cb` is the parsed callback behind that option, so
//! its setter and the garbage collector's mark hook live here too.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn op_colon(mut oap: *mut oparg_T) {
    unsafe {
        stuffcharReadbuff(':' as ::core::ffi::c_int);
        if (*oap).is_VIsual {
            stuffReadbuff(b"'<,'>\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum {
                stuffcharReadbuff('.' as ::core::ffi::c_int);
            } else {
                stuffnumReadbuff((*oap).start.lnum as ::core::ffi::c_int);
            }
            let mut endOfStartFold: linenr_T = (*oap).start.lnum;
            hasFolding(
                curwin.get(),
                (*oap).start.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut endOfStartFold,
            );
            if (*oap).end.lnum != (*oap).start.lnum && (*oap).end.lnum != endOfStartFold {
                stuffcharReadbuff(',' as ::core::ffi::c_int);
                if (*oap).end.lnum == (*curwin.get()).w_cursor.lnum {
                    stuffcharReadbuff('.' as ::core::ffi::c_int);
                } else if (*oap).end.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    stuffcharReadbuff('$' as ::core::ffi::c_int);
                } else if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum
                    && !hasFolding(
                        curwin.get(),
                        (*oap).end.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    stuffReadbuff(b".+\0".as_ptr() as *const ::core::ffi::c_char);
                    stuffnumReadbuff(
                        (*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    );
                } else {
                    stuffnumReadbuff((*oap).end.lnum as ::core::ffi::c_int);
                }
            }
        }
        if (*oap).op_type != OP_COLON {
            stuffReadbuff(b"!\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if (*oap).op_type == OP_INDENT {
            stuffReadbuff(get_equalprg());
            stuffReadbuff(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        } else if (*oap).op_type == OP_FORMAT {
            if *(*curbuf.get()).b_p_fp as ::core::ffi::c_int != NUL {
                stuffReadbuff((*curbuf.get()).b_p_fp);
            } else if *p_fp.get() as ::core::ffi::c_int != NUL {
                stuffReadbuff(p_fp.get());
            } else {
                stuffReadbuff(b"fmt\0".as_ptr() as *const ::core::ffi::c_char);
            }
            stuffReadbuff(b"\n']\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
}

static opfunc_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});

pub unsafe extern "C" fn did_set_operatorfunc(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        if option_set_callback_func(p_opfunc.get(), opfunc_cb.ptr()) == FAIL {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn set_ref_in_opfunc(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        return set_ref_in_callback(
            opfunc_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn op_function(mut oap: *const oparg_T) {
    unsafe {
        let orig_start: pos_T = (*curbuf.get()).b_op_start;
        let orig_end: pos_T = (*curbuf.get()).b_op_end;
        if *p_opfunc.get() as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E774: 'operatorfunc' is empty\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
            if (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
                && !(*oap).inclusive
            {
                decl(&raw mut (*curbuf.get()).b_op_end);
            }
            let mut argv: [typval_T; 2] = [typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            }; 2];
            argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
            argv[1 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
            argv[0 as ::core::ffi::c_int as usize].vval.v_string = [
                b"char\0".as_ptr() as *const ::core::ffi::c_char,
                b"line\0".as_ptr() as *const ::core::ffi::c_char,
                b"block\0".as_ptr() as *const ::core::ffi::c_char,
            ][(*oap).motion_type as usize]
                as *mut ::core::ffi::c_char;
            let save_virtual_op: TriState = virtual_op.get();
            virtual_op.set(kNone);
            let save_finish_op: bool = finish_op.get();
            finish_op.set(false_0 != 0);
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if callback_call(
                opfunc_cb.ptr(),
                1 as ::core::ffi::c_int,
                &raw mut argv as *mut typval_T,
                &raw mut rettv,
            ) {
                tv_clear(&raw mut rettv);
            }
            virtual_op.set(save_virtual_op);
            finish_op.set(save_finish_op);
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
                (*curbuf.get()).b_op_start = orig_start;
                (*curbuf.get()).b_op_end = orig_end;
            }
        };
    }
}
