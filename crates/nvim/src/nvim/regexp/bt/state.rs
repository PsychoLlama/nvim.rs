//! Saving and restoring where the engine is.
//!
//! Two shapes of position, because a match runs over either a string or a
//! range of buffer lines: a `regsave_T` holds a pointer for the first and a
//! line/column pair for the second, and which one is live is `rex.reg_match`
//! being null. Everything here comes in that pair.
//!
//! `regstack` is the saved-state stack the matcher pushes decisions onto, and
//! `backpos` the record of where each loop back-edge has already been — a
//! `regsave_T`'s `rs_len` is the `backpos` length to truncate to, so undoing a
//! decision also forgets the loop positions discovered after it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::main::p_mmp;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::regexp::{
    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, NSUBEXP, reg_getline, regbehind_T, regitem_T,
    regsave_T, regstack, regstate_T, rex, save_se_T,
};
use crate::src::nvim::types::{colnr_T, garray_T, lpos_T, uint8_t};

/// Record the current input position, and how much of `gap` — always
/// `backpos` — belongs to it.
pub(crate) fn reg_save(save: *mut regsave_T, gap: *mut garray_T) {
    // SAFETY: `save` is a slot in a live frame and `gap` the backpos garray.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            (*save).rs_u.pos.col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
            (*save).rs_u.pos.lnum = (*rex.ptr()).lnum;
        } else {
            (*save).rs_u.ptr = (*rex.ptr()).input;
        }
        (*save).rs_len = (*gap).ga_len;
    }
}

/// Put the input position back to what [`reg_save`] recorded, refetching the
/// line if the match has moved off it since.
pub(crate) fn reg_restore(save: *mut regsave_T, gap: *mut garray_T) {
    // SAFETY: as `reg_save`.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            if (*rex.ptr()).lnum != (*save).rs_u.pos.lnum {
                (*rex.ptr()).lnum = (*save).rs_u.pos.lnum;
                (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum).cast();
            }
            (*rex.ptr()).input = (*rex.ptr()).line.add((*save).rs_u.pos.col as usize);
        } else {
            (*rex.ptr()).input = (*save).rs_u.ptr;
        }
        (*gap).ga_len = (*save).rs_len;
    }
}

/// Is the input exactly where `save` recorded? The `backpos` length is not
/// part of the comparison.
pub(crate) fn reg_save_equal(save: *const regsave_T) -> bool {
    // SAFETY: as `reg_save`.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).lnum == (*save).rs_u.pos.lnum
                && (*rex.ptr()).input == (*rex.ptr()).line.add((*save).rs_u.pos.col as usize)
        } else {
            (*rex.ptr()).input == (*save).rs_u.ptr
        }
    }
}

/// Move the current position into the capture slot `posp`, keeping what was
/// there in `savep`. The multi-line half of the pair.
pub(crate) fn save_se_multi(savep: *mut save_se_T, posp: *mut lpos_T) {
    // SAFETY: `savep` is a slot in a live frame, `posp` a capture slot.
    unsafe {
        (*savep).se_u.pos = *posp;
        (*posp).lnum = (*rex.ptr()).lnum;
        (*posp).col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
    }
}

/// [`save_se_multi`] for a string match, where a capture is a pointer.
pub(crate) fn save_se_one(savep: *mut save_se_T, pp: *mut *mut uint8_t) {
    // SAFETY: as `save_se_multi`.
    unsafe {
        (*savep).se_u.ptr = *pp;
        *pp = (*rex.ptr()).input;
    }
}

/// Push a frame for `state` and hand it back for the caller to fill in.
///
/// Null when 'maxmempattern' has been reached — the only bound on how deep a
/// pattern may backtrack, and the reason a runaway match ends in E363 rather
/// than in a dead editor.
pub(crate) fn regstack_push(state: regstate_T, scan: *mut uint8_t) -> *mut regitem_T {
    // SAFETY: `regstack` is live for the duration of a match; `ga_grow`
    // reserves the frame before it is written.
    unsafe {
        if (((*regstack.ptr()).ga_len as u32) >> 10) as i64 >= p_mmp.get() {
            emsg(gettext(
                E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN.as_ptr(),
            ));
            return core::ptr::null_mut();
        }
        ga_grow(regstack.ptr(), size_of::<regitem_T>() as c_int);
        let rp = (*regstack.ptr())
            .ga_data
            .cast::<u8>()
            .add((*regstack.ptr()).ga_len as usize)
            .cast::<regitem_T>();
        (*rp).rs_state = state;
        (*rp).rs_scan = scan;
        (*regstack.ptr()).ga_len += size_of::<regitem_T>() as c_int;
        rp
    }
}

/// Pop the top frame, resuming at the node it was pushed for.
pub(crate) fn regstack_pop(scan: &mut *mut uint8_t) {
    // SAFETY: the caller only pops frames it pushed.
    unsafe {
        let rp = (*regstack.ptr())
            .ga_data
            .cast::<u8>()
            .add((*regstack.ptr()).ga_len as usize)
            .cast::<regitem_T>()
            .sub(1);
        *scan = (*rp).rs_scan;
        (*regstack.ptr()).ga_len -= size_of::<regitem_T>() as c_int;
    }
}

/// Copy every `\1`..`\9` capture into `bp`, so that a look-behind attempt can
/// be undone whole.
///
/// `need_clear_subexpr` means the captures have not been touched yet this
/// match, and then there is nothing to copy — the flag alone restores them.
pub(crate) fn save_subexpr(bp: *mut regbehind_T) {
    // SAFETY: `bp` is the record in front of a live `RS_BEHIND1` frame; the
    // capture arrays hold `NSUBEXP` slots.
    unsafe {
        (*bp).save_need_clear_subexpr = (*rex.ptr()).need_clear_subexpr;
        if (*rex.ptr()).need_clear_subexpr != 0 {
            return;
        }
        for i in 0..NSUBEXP as usize {
            if (*rex.ptr()).reg_match.is_null() {
                (*bp).save_start[i].se_u.pos = *(*rex.ptr()).reg_startpos.add(i);
                (*bp).save_end[i].se_u.pos = *(*rex.ptr()).reg_endpos.add(i);
            } else {
                (*bp).save_start[i].se_u.ptr = *(*rex.ptr()).reg_startp.add(i);
                (*bp).save_end[i].se_u.ptr = *(*rex.ptr()).reg_endp.add(i);
            }
        }
    }
}

/// Undo [`save_subexpr`].
pub(crate) fn restore_subexpr(bp: *mut regbehind_T) {
    // SAFETY: as `save_subexpr`.
    unsafe {
        (*rex.ptr()).need_clear_subexpr = (*bp).save_need_clear_subexpr;
        if (*rex.ptr()).need_clear_subexpr != 0 {
            return;
        }
        for i in 0..NSUBEXP as usize {
            if (*rex.ptr()).reg_match.is_null() {
                *(*rex.ptr()).reg_startpos.add(i) = (*bp).save_start[i].se_u.pos;
                *(*rex.ptr()).reg_endpos.add(i) = (*bp).save_end[i].se_u.pos;
            } else {
                *(*rex.ptr()).reg_startp.add(i) = (*bp).save_start[i].se_u.ptr;
                *(*rex.ptr()).reg_endp.add(i) = (*bp).save_end[i].se_u.ptr;
            }
        }
    }
}
