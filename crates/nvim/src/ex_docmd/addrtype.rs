//! What kind of thing a command's range counts, and the default range
//! a command given none is handed.
//!
//! `:wincmd` decides its own from the window command it was given;
//! every other command declares one in the command table. What the
//! rest of `address` reads is the `addr_type` these leave behind.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::address::{head, qf_get_cur_idx, qf_get_cur_valid_idx, qf_get_valid_size, tail, ubyte};
use crate::charset::skipwhite;

use crate::ex_docmd::is_user_cmd;
use crate::types::CmdIdx;
use crate::winlayer::TabPage;

use core::ffi::{c_char, c_int};

use crate::buffer::{buf_is_quickfix, current_buf};

use crate::ex_docmd::cmdnames;
use crate::ex_docmd::window::{current_tab_nr, current_win_nr};

use crate::message::iemsg;
use crate::os::cshim::gettext;

use crate::types::{CmdAddr, ExArg, LineNr};
use crate::winlayer::{Buf, Win};

/// `:wincmd`'s address kind depends on the window command it is given: `w`
/// counts windows, `^` counts buffers, most of the tree counts something
/// the window code names itself, and the rest take no address at all.
///
/// Upstream spells the four sets as one `switch` with 68 labels. They are
/// four tables here, which is the same thing said once.
#[rustfmt::skip]
const WINCMD_OTHER: &[u8] = b"SsnjkTrRKJ+-_|]gvhlHL><}fFid\x13\x0e\x0a\x0b\x12\x1f\x1d\x07\x16\x08\x0c\x06\x09\x04";
const WINCMD_BUFFERS: &[u8] = b"^\x1e";
const WINCMD_WINDOWS: &[u8] = b"qcowWx\x11\x03\x0f\x17\x18";
const WINCMD_NONE: &[u8] = b"zPtbp=\x1a\x14\x02\x10\x0d";

/// # Safety
///
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn get_wincmd_addr_type(arg: *const c_char, excmd: &mut ExArg) {
    let c = ubyte(arg);
    excmd.addr_type = if WINCMD_OTHER.contains(&c) {
        CmdAddr::Other
    } else if WINCMD_BUFFERS.contains(&c) {
        CmdAddr::Buffers
    } else if WINCMD_WINDOWS.contains(&c) {
        CmdAddr::Windows
    } else if WINCMD_NONE.contains(&c) {
        CmdAddr::NoRange
    } else {
        // Anything else keeps whatever the command table said.
        return;
    };
}

/// Take the address kind from the command table, with the three exceptions
/// the table cannot express.
///
/// # Safety
///
/// `excmd` must point at the command's `ExArg`, unaliased for the call. `p`
/// must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn set_cmd_addr_type(excmd: &mut ExArg, p: *mut c_char) {
    if is_user_cmd(excmd.cmdidx) {
        return;
    }
    excmd.addr_type = if excmd.cmdidx != CmdIdx::SIZE {
        cmdnames[excmd.cmdidx.index()].cmd_addr_type
    } else {
        CmdAddr::Lines
    };
    if excmd.cmdidx == CmdIdx::wincmd && !p.is_null() {
        unsafe { get_wincmd_addr_type(skipwhite(p), excmd) };
    }
    // `:cc`/`:ll` in a quickfix window address the window's entries.
    if (excmd.cmdidx == CmdIdx::cc || excmd.cmdidx == CmdIdx::ll) && buf_is_quickfix(current_buf())
    {
        excmd.addr_type = CmdAddr::Other;
    }
}

/// The address `.` stands for, which is also what a bare `+N`/`-N` counts
/// from.
pub fn get_cmd_default_range(excmd: &mut ExArg) -> LineNr {
    match excmd.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            // Not the cursor line but the *last* line when the cursor is
            // past it, which a buffer shrinking under a command allows.
            Win::current()
                .w_cursor
                .lnum
                .min(Buf::current().b_ml.ml_line_count)
        }
        CmdAddr::Windows => current_win_nr(Win::current_or_none()) as LineNr,
        CmdAddr::Arguments => {
            let len = arglist_len();
            if Win::current().w_arg_idx + 1 < len {
                Win::current().w_arg_idx as LineNr + 1
            } else {
                len as LineNr
            }
        }
        CmdAddr::LoadedBuffers | CmdAddr::Buffers => Buf::current().handle as LineNr,
        CmdAddr::Tabs => current_tab_nr(TabPage::current_or_none()) as LineNr,
        CmdAddr::TabsRelative | CmdAddr::Unsigned => 1,
        CmdAddr::Quickfix => qf_get_cur_idx(excmd) as LineNr,
        CmdAddr::QuickfixValid => qf_get_cur_valid_idx(excmd) as LineNr,
        _ => 0,
    }
}

/// The range an `ExArgt::DFLALL` command means by "no range": everything.
pub fn set_cmd_dflall_range(excmd: &mut ExArg) {
    excmd.line1 = 1;
    match excmd.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            excmd.line2 = Buf::current().b_ml.ml_line_count;
        }
        CmdAddr::LoadedBuffers => {
            let (first, last) = loaded_buffer_range();
            excmd.line1 = first;
            excmd.line2 = last;
        }
        CmdAddr::Buffers => {
            excmd.line1 = head().handle as LineNr;
            excmd.line2 = tail().handle as LineNr;
        }
        CmdAddr::Windows => {
            excmd.line2 = current_win_nr(None) as LineNr;
        }
        CmdAddr::Tabs => {
            excmd.line2 = current_tab_nr(None) as LineNr;
        }
        CmdAddr::TabsRelative => excmd.line2 = 1,
        CmdAddr::Arguments => {
            let len = arglist_len();
            if len == 0 {
                excmd.line2 = 0;
                excmd.line1 = 0;
            } else {
                excmd.line2 = len as LineNr;
            }
        }
        CmdAddr::QuickfixValid => {
            excmd.line2 = qf_get_valid_size(excmd) as LineNr;
            if excmd.line2 == 0 {
                excmd.line2 = 1;
            }
        }
        t if t == CmdAddr::NoRange || t == CmdAddr::Unsigned || t == CmdAddr::Quickfix => {
            iemsg(gettext(c"INTERNAL: Cannot use ExArgt::DFLALL with CmdAddr::NoRange, CmdAddr::Unsigned or CmdAddr::Quickfix"));
        }
        _ => {}
    }
}

/// How many files are in the current window's argument list.
pub(super) fn arglist_len() -> c_int {
    unsafe { (*Win::current().w_alist).al_ga.len() as c_int }
}

/// The handles of the first and last *loaded* buffers.
pub(super) fn loaded_buffer_range() -> (LineNr, LineNr) {
    let mut buf = head();
    while buf.b_ml.ml_mfp.is_null() {
        let Some(next) = buf.next() else { break };
        buf = next;
    }
    let first = buf.handle as LineNr;
    let mut buf = tail();
    while buf.b_ml.ml_mfp.is_null() {
        let Some(prev) = buf.prev() else { break };
        buf = prev;
    }
    (first, buf.handle as LineNr)
}
