//! Signs: the definition table, and placing them as extmarks.
//!
//! A sign has two halves. `:sign define` records a *definition* — a name,
//! up to two cells of text and up to four highlight groups — in the table
//! [`store`] owns. `:sign place` then puts a *placement* in a buffer, which
//! is an extmark carrying a [`DecorSignHighlight`] copied out of the
//! definition; the drawing code never sees the definition again, which is
//! why redefining a placed sign has to walk the decoration store and patch
//! every copy. [`place`] is that half.
//!
//! Sign *groups* are namespaces. The global group is namespace 0, a named
//! group is whatever `nvim_create_namespace` answers, and `"*"` means all
//! of them ([`ALL_GROUPS`]).
//!
//! A sign's `text=` -- turning it into cells and back -- is [`text`]. The
//! `:sign` command lives in [`command`], its completion in [`complete`] and
//! the `sign_*()` Vimscript functions in [`vimscript`]. This file is the
//! module's shared vocabulary: its constants and the `use` list every child
//! takes with `use super::*`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::api::extmark::namespace_id_for;
use crate::api::extmark::{describe_ns, nvim_create_namespace};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::buffer::find_buf;
use crate::charset::vim_isprintc;
use crate::cursor::check_cursor_lnum;
use crate::decoration::{
    DECOR_ID_INVALID, DECOR_SIGN_HIGHLIGHT_INIT, decor_put_sh, for_each_decor_item, kMTMetaSignHL,
    kMTMetaSignText, kSHIsSign, sign_item, sign_item_cmp, with_decor_item,
};
use crate::drawscreen::{UPD_NOT_VALID, redraw_buf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::funcs::get_buf_arg;
use crate::eval::typval::Unconvertible;
use crate::eval::typval::{
    dict_find, dict_get_number, dict_get_number_def, tv_check_for_nonnull_dict_arg,
    tv_check_for_opt_dict_arg, tv_check_for_string_arg, tv_dict_alloc, tv_get_lnum,
    tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::ex_docmd::do_cmdline_cmd;
use crate::extmark::{self, extmark_del_id, extmark_set};
use crate::fold::fold_open_cursor;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_D, get_highlight_name_ext, syn_check_group};
use crate::marktree::cursor::{Cursor, lookup_ns};
use crate::marktree::key::{MtFlags, mt_decor, mt_decor_sign, mt_end};
use crate::memory::ThinCString;
use crate::message::{msg_putchar, msg_str, msg_title};
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::os::cshim::gettext;
use crate::os::libc::{int32_t, int64_t, ptrdiff_t, uint32_t};
use crate::tr;
use crate::types::{
    ApiString, DecorExt, DecorInline, DecorPriority, DecorSignHighlight, Dict, EvalFuncData, ExArg,
    Expand, Failed, Integer, LineNr, List, MTKey, MarkTreeIter, ScreenChar, Sign, TypVal,
    VarNumber,
};
use crate::window::buf_jump_open_win;
use crate::winlayer::{Buf, Win, buffers, first_buffer, windows};

mod command;
pub(crate) use self::command::*;
mod complete;
pub(crate) use self::complete::*;
mod place;
pub(crate) use self::place::*;
mod store;
pub(crate) use self::store::*;
mod text;
pub(crate) use self::text::*;
#[cfg(test)]
mod tests;
mod vimscript;
pub(crate) use self::vimscript::*;

/// Why `:sign` or a `sign_*()` function refused.
///
/// The code below the command and the builtins answers one of these; the
/// command ([`ex_sign`]) or the builtin reports it, once, with
/// [`SignError::report`]. The texts are upstream's, byte for byte.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SignError {
    #[error("{}", tr!("E474: Invalid argument"))]
    InvalidArgument,
    #[error("{}", tr!("E471: Argument required"))]
    ArgumentRequired,
    #[error("{}", tr!("E715: Dictionary required"))]
    DictionaryRequired,
    #[error("{}", tr!("E714: List required"))]
    ListRequired,
    #[error("{}", tr!("E155: Unknown sign: {}", msg_bytes(.0.as_bytes())))]
    UnknownSign(ThinCString),
    #[error("{}", tr!("E156: Missing sign name"))]
    MissingName,
    #[error("{}", tr!("E157: Invalid sign ID: {}", .0))]
    InvalidId(c_int),
    #[error("{}", tr!("E158: Invalid buffer name: {}", msg_bytes(.0.as_bytes())))]
    InvalidBuffer(ThinCString),
    #[error("{}", tr!("E159: Missing sign number"))]
    MissingNumber,
    #[error("{}", tr!("E160: Unknown sign command: {}", msg_bytes(.0.as_bytes())))]
    UnknownCommand(ThinCString),
    /// A `:sign define` argument that is not one of the keys; the text
    /// runs to the end of the line, as upstream quotes it.
    #[error("{}", tr!("E475: Invalid argument: {}", msg_bytes(.0.as_bytes())))]
    BadDefineArgument(ThinCString),
    /// The text as unescaped, which is what did not fit.
    #[error("{}", tr!("E239: Invalid sign text: {}", msg_bytes(.0.as_bytes())))]
    BadText(ThinCString),
    #[error("{}", tr!("E885: Not possible to change sign {}", msg_bytes(.0.as_bytes())))]
    CannotChange(ThinCString),
    #[error("{}", tr!("E934: Cannot jump to a buffer that does not have a name"))]
    NoBufferName,
    /// Refused without a word, as upstream does: nothing matched to
    /// remove, a required key is missing, or a sign cannot be placed in
    /// the group `"*"`. The builtins answer `-1` and say nothing.
    #[error("refused without a message")]
    Quiet,
    /// What the argument checks or the buffer lookup reported themselves.
    #[error(transparent)]
    Reported(#[from] Failed),
}

// Two words, so a `Result` carrying one is no wider than it has to be.
const _: () = assert!(size_of::<SignError>() <= 16);

impl From<Unconvertible> for SignError {
    /// A value that would not read as a number: its message is out.
    fn from(_: Unconvertible) -> Self {
        Self::Reported(Failed)
    }
}

impl SignError {
    /// Puts the message out, unless it is already out or there is none.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Quiet | Self::Reported(_)) {
            emsg_text(self.to_string());
        }
    }
}

/// A builtin's answer: `ok` as it is, a refusal reported and `-1`.
pub(crate) fn answer_or_report(result: Result<c_int, SignError>) -> c_int {
    result.unwrap_or_else(|err| {
        err.report();
        -1
    })
}

/// The priority a placement gets when neither the definition nor the
/// `:sign place` / `sign_place()` call names one.
pub(crate) const SIGN_DEF_PRIO: c_int = 10;

/// [`group_get_ns`]'s answer for the group `"*"`: every namespace, the
/// global one included.
pub(crate) const ALL_GROUPS: int64_t = u32::MAX as int64_t;

/// [`group_get_ns`]'s answer for a group that names no namespace.
const NO_SUCH_GROUP: int64_t = -1;

pub(crate) const SIGNCMD_DEFINE: c_int = 0;
pub(crate) const SIGNCMD_UNDEFINE: c_int = 1;
pub(crate) const SIGNCMD_LIST: c_int = 2;
pub(crate) const SIGNCMD_PLACE: c_int = 3;
pub(crate) const SIGNCMD_UNPLACE: c_int = 4;
pub(crate) const SIGNCMD_JUMP: c_int = 5;
/// One past the last subcommand: [`sign_cmd_idx`]'s "not a subcommand".
pub(crate) const SIGNCMD_LAST: c_int = 6;

/// The `:sign` subcommands, in the order the `SIGNCMD_*` constants number
/// them. Also the completion list for `:sign <Tab>`.
pub(crate) const CMDS: [&CStr; SIGNCMD_LAST as usize] = [
    c"define",
    c"undefine",
    c"list",
    c"place",
    c"unplace",
    c"jump",
];
