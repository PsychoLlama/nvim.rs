//! What kind of thing a command's range counts, and the default range
//! a command given none is handed.
//!
//! `:wincmd` decides its own from the window command it was given;
//! every other command declares one in the command table. What the
//! rest of `address` reads is the `addr_type` these leave behind.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::address::{head, tail};
use crate::narrow::len_as_int;
use crate::quickfix::{qf_get_cur_idx, qf_get_cur_valid_idx, qf_get_valid_size};

use crate::ex_docmd::is_user_cmd;
use crate::types::CmdIdx;
use crate::winlayer::TabPage;

use core::ffi::c_int;

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

/// `c` is the first byte of the window command, white space skipped.
pub(crate) fn get_wincmd_addr_type(c: u8, excmd: &mut ExArg) {
    let kind = if WINCMD_OTHER.contains(&c) {
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
    excmd.set_kind_unread(kind);
}

impl ExArg<'_> {
    /// Take the address kind from the command table, with the three exceptions
    /// the table cannot express.
    ///
    /// `first` is the first byte of the command's argument, white space
    /// skipped, which only `:wincmd` consults; `None` is "there is no
    /// argument", upstream's null `p`.
    pub fn set_addr_type(&mut self, first: Option<u8>) {
        if is_user_cmd(self.cmdidx) {
            return;
        }
        self.set_kind_unread(if self.cmdidx != CmdIdx::SIZE {
            cmdnames[self.cmdidx.index()].cmd_addr_type
        } else {
            CmdAddr::Lines
        });
        if self.cmdidx == CmdIdx::wincmd
            && let Some(first) = first
        {
            get_wincmd_addr_type(first, self);
        }
        // `:cc`/`:ll` in a quickfix window address the window's entries.
        if (self.cmdidx == CmdIdx::cc || self.cmdidx == CmdIdx::ll)
            && buf_is_quickfix(current_buf())
        {
            self.set_kind_unread(CmdAddr::Other);
        }
    }
}

/// The address `.` stands for, which is also what a bare `+N`/`-N` counts
/// from, as the plain number [`ExArg::set_range_numbers`] takes.
pub fn get_cmd_default_range(excmd: &mut ExArg) -> c_int {
    match excmd.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            // Not the cursor line but the *last* line when the cursor is
            // past it, which a buffer shrinking under a command allows.
            Win::current()
                .w_cursor
                .lnum
                .min(Buf::current().b_ml.ml_line_count)
                .get()
        }
        CmdAddr::Windows => current_win_nr(Win::current_or_none()),
        CmdAddr::Arguments => {
            let len = arglist_len();
            if Win::current().w_arg_idx + 1 < len {
                Win::current().w_arg_idx + 1
            } else {
                len
            }
        }
        CmdAddr::LoadedBuffers | CmdAddr::Buffers => Buf::current().handle.get(),
        CmdAddr::Tabs => current_tab_nr(TabPage::current_or_none()),
        CmdAddr::TabsRelative | CmdAddr::Unsigned => 1,
        CmdAddr::Quickfix => len_as_int(qf_get_cur_idx(excmd.cmdidx)),
        CmdAddr::QuickfixValid => qf_get_cur_valid_idx(excmd.cmdidx),
        _ => 0,
    }
}

/// The range an `ExArgt::DFLALL` command means by "no range": everything.
pub fn set_cmd_dflall_range(excmd: &mut ExArg) {
    if excmd.counts_lines() {
        excmd.line1 = LineNr::ONE;
    } else {
        excmd.number1 = 1;
    }
    match excmd.addr_type {
        CmdAddr::Lines => {
            excmd.line2 = Buf::current().b_ml.ml_line_count;
        }
        // An "other" kind's whole range is still every line.
        CmdAddr::Other => {
            excmd.number2 = Buf::current().b_ml.ml_line_count.get();
        }
        CmdAddr::LoadedBuffers => {
            (excmd.number1, excmd.number2) = loaded_buffer_range();
        }
        CmdAddr::Buffers => {
            excmd.number1 = head().handle.get();
            excmd.number2 = tail().handle.get();
        }
        CmdAddr::Windows => {
            excmd.number2 = current_win_nr(None);
        }
        CmdAddr::Tabs => {
            excmd.number2 = current_tab_nr(None);
        }
        CmdAddr::TabsRelative => excmd.number2 = 1,
        CmdAddr::Arguments => {
            let len = arglist_len();
            excmd.number1 = c_int::from(len != 0);
            excmd.number2 = len;
        }
        CmdAddr::QuickfixValid => {
            excmd.number2 = len_as_int(qf_get_valid_size(excmd.cmdidx)).max(1);
        }
        t if t == CmdAddr::NoRange || t == CmdAddr::Unsigned || t == CmdAddr::Quickfix => {
            iemsg(gettext(c"INTERNAL: Cannot use ExArgt::DFLALL with CmdAddr::NoRange, CmdAddr::Unsigned or CmdAddr::Quickfix"));
        }
        _ => {}
    }
}

/// How many files are in the current window's argument list.
pub(super) fn arglist_len() -> c_int {
    crate::arglist::argcount()
}

/// The numbers of the first and last *loaded* buffers.
pub(super) fn loaded_buffer_range() -> (c_int, c_int) {
    let mut buf = head();
    while buf.b_ml.ml_mfp.is_none() {
        let Some(next) = buf.next() else { break };
        buf = next;
    }
    let first = buf.handle.get();
    let mut buf = tail();
    while buf.b_ml.ml_mfp.is_none() {
        let Some(prev) = buf.prev() else { break };
        buf = prev;
    }
    (first, buf.handle.get())
}
