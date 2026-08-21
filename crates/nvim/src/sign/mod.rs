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

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int};
use core::ops::{Deref, DerefMut};
use core::slice;
use std::ffi::CString;

use crate::api::extmark::{describe_ns, nvim_create_namespace};
use crate::api::private::helpers::cstr_as_string;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::buffer::{buflist_findname_exp, buflist_findnr};
use crate::charset::{
    backslash_halve, getdigits_int, skiptowhite, skiptowhite_esc, skipwhite, vim_isprintc,
};
use crate::cursor::check_cursor_lnum;
use crate::decoration::{
    DECOR_SIGN_HIGHLIGHT_INIT, SIGN_WIDTH, Sh, decor_find_sign, decor_items, decor_put_sh,
    kMTMetaSignHL, kMTMetaSignText, kSHIsSign, sign_item_cmp,
};
use crate::drawscreen::{UPD_NOT_VALID, redraw_buf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::funcs::get_buf_arg;
use crate::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg, tv_check_for_string_arg,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_number_def, tv_dict_get_string, tv_get_lnum, tv_get_number_chk,
    tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_number, tv_list_first,
};
use crate::ex_docmd::do_cmdline_cmd;
use crate::extmark::{extmark_del, extmark_del_id, extmark_set};
use crate::fold::fold_open_cursor;
use crate::global_cell::GlobalCell;
use crate::grid::schar_get;
use crate::highlight_group::{HLF_D, get_highlight_name_ext, syn_check_group};
use crate::main::{
    curwin, e_argreq, e_dictreq, e_invalid_buffer_name_str, e_invarg, e_invarg2, e_listreq,
    e_trailing_arg, firstbuf, got_int, namespace_ids,
};
use crate::map::mh_get_string;
use crate::marktree::cursor::{Cursor, lookup_ns, tree_of};
use crate::marktree::key::{
    MT_FLAG_DECOR_SIGNHL, MT_FLAG_DECOR_SIGNTEXT, mt_decor, mt_decor_sign, mt_end,
};
use crate::marktree::{marktree_itr_current, marktree_itr_next};
use crate::mbyte::{MAX_SCHAR_SIZE, utf_ptr2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::memory::{xfree, xstrdup};
use crate::message::{emsg, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title};
use crate::os::cshim::{gettext, snprintf, strncmp};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    DecorExt, DecorInline, DecorInlineData, DecorPriority, DecorSignHighlight, DecorVirtText,
    Error, EvalFuncData, FAIL, Integer, MTKey, MarkTreeIter, OK, SignItem, buf_T, dict_T,
    dictitem_T, exarg_T, expand_T, int32_t, int64_t, linenr_T, list_T, ptrdiff_t, schar_T, sign_T,
    size_t, typval_T, uint32_t, varnumber_T,
};
use crate::window::buf_jump_open_win;
use crate::winlayer::{Buf, Win, buffers, windows};
use ::libc::{atoi, strcmp, strlen};

mod command;
pub use self::command::*;
mod complete;
pub use self::complete::*;
mod place;
pub use self::place::*;
mod store;
pub use self::store::*;
mod text;
pub use self::text::*;
mod vimscript;
pub use self::vimscript::*;

/// The priority a placement gets when neither the definition nor the
/// `:sign place` / `sign_place()` call names one.
pub const SIGN_DEF_PRIO: c_int = 10;

pub const MSG_BUF_LEN: c_int = 480;

/// [`group_get_ns`]'s answer for the group `"*"`: every namespace, the
/// global one included.
pub(crate) const ALL_GROUPS: int64_t = u32::MAX as int64_t;

/// [`group_get_ns`]'s answer for a group that names no namespace.
const NO_SUCH_GROUP: int64_t = -1;

pub const SIGNCMD_DEFINE: c_int = 0;
pub const SIGNCMD_UNDEFINE: c_int = 1;
pub const SIGNCMD_LIST: c_int = 2;
pub const SIGNCMD_PLACE: c_int = 3;
pub const SIGNCMD_UNPLACE: c_int = 4;
pub const SIGNCMD_JUMP: c_int = 5;
/// One past the last subcommand: [`sign_cmd_idx`]'s "not a subcommand".
pub const SIGNCMD_LAST: c_int = 6;

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
