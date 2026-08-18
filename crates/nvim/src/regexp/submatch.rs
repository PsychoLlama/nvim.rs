//! What the last match captured, for the code that runs *inside* a
//! substitution: `submatch()` and `submatch(n, 1)`, and the list a `\=`
//! expression's function is handed as its argument.
//!
//! These read `rsm`, not `rex`. A `\=` expression may run a search or a
//! substitution of its own, which takes `rex` over; `rsm` is the snapshot
//! [`super::substitute`] takes of the outermost match before evaluating, so
//! that `submatch()` keeps answering about the substitution the user wrote.
//! `can_f_submatch` is what says the snapshot is live.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{LineOrigin, NUL, Rex, can_f_submatch, reg_line, reg_line_len, rsm};
use crate::eval::typval::{
    tv_list_alloc, tv_list_append_string, tv_list_first, tv_list_init_static10, tv_list_ref,
};
use crate::memory::{xfree, xmalloc, xmemcpyz};
use crate::strings::xstrnsave;
use crate::types::{VAR_STRING, colnr_T, linenr_T, list_T, staticList10_T, typval_T, ufunc_T};
use ::libc::{strcpy, strncpy};

/// The text of the submatch line `lnum` lines into the match `submatch()`
/// and a `\=` expression see.
pub(crate) fn reg_getline_submatch(rex: Rex, lnum: linenr_T) -> *mut c_char {
    reg_line(rex, lnum, LineOrigin::Submatch)
}

/// Its length.
pub(crate) fn reg_getline_submatch_len(rex: Rex, lnum: linenr_T) -> colnr_T {
    reg_line_len(rex, lnum, LineOrigin::Submatch)
}

/// Fill `argv[argskip]` — a ten-item static list — with the submatches, and
/// report how many arguments the call now has.
///
/// This is the `fe_argv_func` a `\=` expression's function is called
/// through, so it runs before the function body sees its arguments. A
/// function that does not take a submatches argument gets none: the list
/// stays as the caller left it, which is what tells [`super::substitute`]
/// there is nothing to free.
pub(crate) unsafe extern "C" fn fill_submatch_list(
    _argc: c_int,
    argv: *mut typval_T,
    argskip: c_int,
    fp: *mut ufunc_T,
) -> c_int {
    // SAFETY: `argv` has at least `argskip + 1` slots and `argv[argskip]`
    // holds the `staticList10_T` the caller keeps alive across the call;
    // `rsm` describes a live string match.
    unsafe {
        let listarg = argv.offset(argskip as isize);
        if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= argskip {
            return argskip;
        }

        // Relies on `sl_list` being the first member of `staticList10_T`.
        tv_list_init_static10((*listarg).vval.v_list as *mut staticList10_T);

        // A `staticList10_T` always has exactly ten items, one per capture.
        let match_ = (*rsm.ptr()).sm_match;
        let mut li = tv_list_first((*listarg).vval.v_list);
        for i in 0..10 {
            let start = (*match_).startp[i];
            let text = if start.is_null() || (*match_).endp[i].is_null() {
                core::ptr::null_mut()
            } else {
                xstrnsave(start, (*match_).endp[i].offset_from(start) as usize)
            };
            (*li).li_tv.v_type = VAR_STRING;
            (*li).li_tv.vval.v_string = text;
            li = (*li).li_next;
        }
        argskip + 1
    }
}

/// Free the strings [`fill_submatch_list`] allocated into `sl`.
pub(crate) unsafe fn clear_submatch_list(sl: *mut staticList10_T) {
    // SAFETY: `sl` is the caller's list, whose items own their strings.
    unsafe {
        let mut li = (*sl).sl_list.lv_first;
        while !li.is_null() {
            xfree((*li).li_tv.vval.v_string.cast());
            li = (*li).li_next;
        }
    }
}

/// The text capture `no` matched, as an allocated string the caller owns.
/// Null outside a substitution and for a capture that did not participate.
///
/// A buffer match's capture can span lines, in which case the breaks come
/// back as newlines. That length is not known without walking the lines, so
/// the walk runs twice: round 1 measures and allocates, round 2 copies.
/// Both rounds must agree, so keep them in step.
pub(crate) unsafe fn reg_submatch(no: c_int) -> *mut c_char {
    // SAFETY: guarded by `can_f_submatch`, which is only set while `rsm`
    // describes a live match; `no` is bounds-checked against the ten
    // capture slots by its caller (`submatch()` rejects anything else).
    unsafe {
        if !can_f_submatch.get() || no < 0 {
            return core::ptr::null_mut();
        }
        let no = no as usize;
        // SAFETY: `can_f_submatch` says a match is live, so the context
        // still names the buffer the submatch lines are read from.
        let rex = Rex::acquire();

        // A string match has no lines to cross.
        if !(*rsm.ptr()).sm_match.is_null() {
            let match_ = (*rsm.ptr()).sm_match;
            let start = (*match_).startp[no];
            if start.is_null() || (*match_).endp[no].is_null() {
                return core::ptr::null_mut();
            }
            return xstrnsave(start, (*match_).endp[no].offset_from(start) as usize);
        }

        let mmatch = (*rsm.ptr()).sm_mmatch;
        let mut retval: *mut c_char = core::ptr::null_mut();
        for round in 1..=2 {
            let mut lnum = (*mmatch).startpos[no].lnum;
            if lnum < 0 || (*mmatch).endpos[no].lnum < 0 {
                return core::ptr::null_mut();
            }
            let line = reg_getline_submatch(rex, lnum);
            if line.is_null() {
                // Anti-crash check; cannot happen.
                break;
            }
            let scol = (*mmatch).startpos[no].col;
            let ecol = (*mmatch).endpos[no].col;
            let s = line.offset(scol as isize);

            // Counts the terminating NUL, so that it is also the size to
            // allocate at the end of round 1.
            let mut len: usize;
            if (*mmatch).endpos[no].lnum == lnum {
                // Within one line: from the start column to the end one.
                len = (ecol - scol) as usize;
                if round == 2 {
                    xmemcpyz(retval.cast(), s.cast(), len);
                }
                len += 1;
            } else {
                // The rest of the start line, then whole lines, then the
                // head of the end line. Each break travels as a newline.
                len = (reg_getline_submatch_len(rex, lnum) - scol) as usize;
                if round == 2 {
                    strcpy(retval, s);
                    *retval.add(len) = b'\n' as c_char;
                }
                len += 1;
                lnum += 1;
                while lnum < (*mmatch).endpos[no].lnum {
                    let line = reg_getline_submatch(rex, lnum);
                    if round == 2 {
                        strcpy(retval.add(len), line);
                    }
                    len += reg_getline_submatch_len(rex, lnum) as usize;
                    if round == 2 {
                        *retval.add(len) = b'\n' as c_char;
                    }
                    len += 1;
                    lnum += 1;
                }
                if round == 2 {
                    strncpy(
                        retval.add(len),
                        reg_getline_submatch(rex, lnum),
                        ecol as usize,
                    );
                }
                len += ecol as usize;
                if round == 2 {
                    *retval.add(len) = NUL as c_char;
                }
                len += 1;
            }

            if retval.is_null() {
                retval = xmalloc(len).cast();
            }
        }
        retval
    }
}

/// [`reg_submatch`] as one list item per line, which is what
/// `submatch(no, 1)` returns. Unlike [`reg_submatch`] this keeps NULs in the
/// text apart from the line breaks, because each line is its own item.
pub(crate) unsafe fn reg_submatch_list(no: c_int) -> *mut list_T {
    // SAFETY: as [`reg_submatch`].
    unsafe {
        if !can_f_submatch.get() || no < 0 {
            return core::ptr::null_mut();
        }
        let no = no as usize;
        // SAFETY: `can_f_submatch` says a match is live, so the context
        // still names the buffer the submatch lines are read from.
        let rex = Rex::acquire();

        // A string match is one item.
        if !(*rsm.ptr()).sm_match.is_null() {
            let match_ = (*rsm.ptr()).sm_match;
            let start = (*match_).startp[no];
            if start.is_null() || (*match_).endp[no].is_null() {
                return core::ptr::null_mut();
            }
            let list = tv_list_alloc(1);
            tv_list_append_string(list, start, (*match_).endp[no].offset_from(start));
            tv_list_ref(list);
            return list;
        }

        let mmatch = (*rsm.ptr()).sm_mmatch;
        let slnum = (*mmatch).startpos[no].lnum;
        let elnum = (*mmatch).endpos[no].lnum;
        if slnum < 0 || elnum < 0 {
            return core::ptr::null_mut();
        }
        let scol = (*mmatch).startpos[no].col;
        let ecol = (*mmatch).endpos[no].col;

        let list = tv_list_alloc((elnum - slnum + 1) as isize);
        let s = reg_getline_submatch(rex, slnum).offset(scol as isize);
        if slnum == elnum {
            tv_list_append_string(list, s, (ecol - scol) as isize);
        } else {
            // A negative length means "to the end of the line".
            tv_list_append_string(list, s, -1);
            for lnum in slnum + 1..elnum {
                tv_list_append_string(list, reg_getline_submatch(rex, lnum), -1);
            }
            tv_list_append_string(list, reg_getline_submatch(rex, elnum), ecol as isize);
        }
        tv_list_ref(list);
        list
    }
}
