//! The quickfix and location list stacks.
//!
//! A `qf_info_T` is a stack of up to ten `qf_list_T`s; a list is a chain of
//! `qfline_T` entries. There is one quickfix stack for the editor and one
//! location list stack per window. This file holds the glue — the constant
//! families and the shared statics — and the children hold the work:
//!
//! - Building a list: `efm` compiles `'errorformat'`, `parse` applies it to
//!   a line, `read` drives the whole read, `list` and `entry` own the
//!   entries, `stack` owns the stacks.
//! - The commands that build one: `cmds` (`:cfile`/`:cbuffer`/`:cexpr`),
//!   `make` (`:make`/`:grep`), `vimgrep` with `dummy`, and `helpgrep`.
//! - Using a list: `navigate` picks an entry, `jump` and `switchbuf` go
//!   there, `display` and `window` with `fill` show it.
//! - Vimscript: `getprops`, `setprops` and the `eval` bridges.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::cstr_as_string;
use crate::arglist::get_arglist_exp;
use crate::ascii::ascii_iswhite;
use crate::autocmd::{
    apply_autocmds, au_event_disable, au_event_restore, aucmd_prepbuf, aucmd_restbuf,
    block_autocmds, unblock_autocmds,
};
use crate::buffer::{
    buf_is_help, buf_is_normal, buf_is_quickfix, buf_valid, buflist_findname_exp, buflist_getfile,
    buflist_new, close_buffer, current_buf, do_modelines, find_buf, no_write_message, setfname,
    wipe_buffer,
};
use crate::change::changed_lines;
use crate::charset::{skipdigits, skipwhite, vim_isprintc};
use crate::cursor::{check_cursor, coladvance};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_later, redraw_curbuf_later, update_screen,
};
use crate::edit::beginline;
use crate::eval::typval::{
    KeyTaken, callback_copy, callback_free, callback_put, tv_clear, tv_copy, tv_dict_add,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_add_tv, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number,
    tv_dict_get_string_alloc, tv_dict_get_tv, tv_dict_item_alloc_len, tv_dict_item_free,
    tv_dict_unref, tv_free, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_first, tv_list_len, tv_list_ref,
};
use crate::eval::vars::set_internal_string_var;
use crate::eval::window::{find_win_by_nr_or_id, win_by_id};
use crate::eval::{
    callback_call, callback_from_typval, eval_expr, set_ref_in_callback, set_ref_in_item,
};
use crate::ex_cmds::{append_redir, check_secure, do_ecmd, do_shell, skip_vimgrep_pat};
use crate::ex_cmds2::{autowrite_all, can_abandon};
use crate::ex_docmd::{do_cmdline_cmd, ex_cd, is_loclist_cmd};
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::ex_getln::get_list_range;
use crate::extmark::extmark_splice;
use crate::fileio::{readfile, shorten_buf_fname, shorten_fnames, vim_fgets, vim_tempname};
use crate::fold::{fold_open_cursor, fold_update_all};
use crate::fuzzy::fuzzy_match;
use crate::global_cell::GlobalCell;
use crate::help::check_help_lang;
use crate::highlight_group::syn_name2id;
use crate::main::{
    Columns, KeyTyped, cmdline_row, cmdmod, curbuf, curtab, curwin, e_au_recursive,
    e_buffer_is_not_loaded, e_dictreq, e_invalpat, e_invarg, e_invrange, e_listreq, e_loclist,
    e_no_errors, e_nomatch, e_noprevre, e_notmp, e_readerrf, e_string_required,
    e_winfixbuf_cannot_go_to_buffer, fdo_flags, firstwin, got_int, lastwin, msg_col, msg_didout,
    msg_nowait, msg_scroll, msg_scrolled, must_redraw, p_ch, p_chi, p_cpo, p_ef, p_efm, p_enc,
    p_gefm, p_gp, p_hh, p_ic, p_mef, p_menc, p_mls, p_qftf, p_rtp, p_shq, p_sp, p_swb, prevwin,
    restart_edit, swb_flags,
};
use crate::mark::setpcmark;
use crate::mbyte::{convert_setup, remove_bom, string_convert};
use crate::memfile::mf_fname;
use crate::memline::{
    check_need_swap, ml_append_buf, ml_delete, ml_get_buf, ml_get_buf_len, ml_open,
};
use crate::memory::{
    strequal, xcalloc, xfree, xmalloc, xmallocz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::message::{
    emsg, internal_error, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_keep,
    msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl, msg_start, msg_strtrunc,
    trunc_string,
};
use crate::r#move::update_topline;
use crate::normal::reset_VIsual_and_resel;
use crate::ops::get_region_bytecount;
use crate::option::{
    buf_copy_options, copy_option_part, option_set_callback_func, set_option_direct,
    set_option_value_give_err, shortmess, skip_to_option_part,
};
use crate::options::{
    kOptBufhidden, kOptBuftype, kOptCpoptions, kOptErrorfile, kOptFdoFlagQuickfix, kOptFiletype,
    kOptFoldmethod, kOptSwapfile, kOptSwbFlagUselast, kOptSwbFlagUsetab,
};
use crate::optionstr::free_string_option;
use crate::os::cshim::{gettext, snprintf, strncasecmp};
use crate::os::env::{expand_env, os_get_pid};
use crate::os::fs::{
    os_dirname, os_fileinfo_link, os_fopen, os_isdir, os_open_stdin_fd, os_path_exists, os_remove,
};
use crate::os::input::{line_breakcheck, os_breakcheck};
use crate::path::{
    PATHSEP, after_pathsep, concat_fnames, fix_fname, free_wild, gen_expand_wildcards,
    path_fnamecmp, path_is_absolute, path_tail, path_try_shorten_fname, vim_is_abs_name,
};
use crate::pos::MAXLNUM;
use crate::regexp::{vim_regcomp, vim_regexec, vim_regexec_multi, vim_regfree};
use crate::search::{BACKWARD, BACKWARD_FILE, FORWARD, FORWARD_FILE, do_search, last_search_pat};
use crate::strings::{has_non_ascii, vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::types::AutoEvent;
use crate::types::TAB;
use crate::types::builders::static_cstring;
use crate::types::{
    Callback, DirStack, Direction, EvalFuncData, ExtmarkOp, FILE, FileInfo, OptInt, OptVal,
    QFLT_INTERNAL, QFLT_LOCATION, QFLT_QUICKFIX, VarType, aco_save_T, bln_values, buf_T, cleanup_T,
    colnr_T, dict_T, dictitem_T, dobuf_action_values, exarg_T, getf_values, linenr_T, list_T,
    listitem_T, optset_T, pos_T, ptrdiff_t, qf_info_T, qf_list_T, qfline_T, qfltype_T, regmatch_T,
    regmmatch_T, regprog_T, scid_T, size_t, time_t, typval_T, typval_vval_union, varnumber_T,
    vimconv_T,
};
use crate::ui::ui_flush;
use crate::undo::u_clearallandblockfree;
use crate::window::{
    check_can_set_curbuf_forceit, check_lnums, goto_tabpage_win, win_close, win_enter, win_goto,
    win_setheight, win_split, win_valid,
};
use ::libc::{__errno_location, abort, abs, atoi, atol, fclose, fdopen, ferror, fgets, time};
use core::ffi::{CStr, c_int, c_uint};

// The carve of the transpiled module; see each child's docs.
mod efm;
pub(crate) use self::efm::*;
mod parse;
pub(crate) use self::parse::*;
mod read;
pub use self::read::*;
mod stack;
pub use self::stack::*;
pub(crate) use crate::winlayer::{Buf, Ea, Win};
mod list;
pub use self::list::*;
mod entry;
pub use self::entry::*;
mod switchbuf;
pub(crate) use self::switchbuf::*;
mod jump;
pub use self::jump::*;
mod display;
pub use self::display::*;
mod window;
pub use self::window::*;
mod fill;
pub(crate) use self::fill::*;
mod navigate;
pub use self::navigate::*;
mod vimgrep;
pub use self::vimgrep::*;
mod dummy;
pub(crate) use self::dummy::*;
mod make;
pub use self::make::*;
mod helpgrep;
pub use self::helpgrep::*;
mod cmds;
pub use self::cmds::*;
mod getprops;
pub(crate) use self::getprops::*;
mod setprops;
pub use self::setprops::*;
mod eval;
pub use self::eval::*;

/// Why a `getqflist()`/`setqflist()` request could not be carried out.
///
/// Vimscript sees one answer for all of these — the function returns `-1` and
/// says nothing more — which is why upstream never distinguished them and why
/// this type has no `Display`. The variants are for the code: `qf_*` used to
/// be a chain of functions whose only vocabulary was `FAIL`, so the reason a
/// request was refused stopped existing the moment it was returned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QfError {
    /// The request named a list that is not on the stack: an `nr` or `id`
    /// that matches nothing, or no stack at all.
    NoSuchList,
    /// A key of the `what` dictionary was given a value of a type it cannot
    /// take — an `idx` that is not a Number, a `title` that is not a String,
    /// `lines` that is not a List.
    BadValue,
    /// `lines` could not be turned into entries with the `'errorformat'` in
    /// force.
    Unparsable,
    /// The `what` dictionary named nothing this understands, so there was
    /// nothing to do. `setqflist()` reports that as a failure.
    NothingToSet,
    /// `'secure'` or the sandbox forbids setting a callback. The message has
    /// already been reported.
    Forbidden,
    /// A key could not be written into the answer dictionary. Unreachable in
    /// practice — the answer is built fresh and each key is written once —
    /// but the dictionary layer is entitled to refuse and this is where its
    /// refusal lands.
    KeyTaken,
}

impl From<KeyTaken> for QfError {
    fn from(_: KeyTaken) -> Self {
        QfError::KeyTaken
    }
}

pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_DUMMY: bln_values = 4;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const READ_DUMMY: c_uint = 16;
pub const READ_NEW: c_uint = 1;
pub const FUZZY_MATCH_MAX_LEN: c_uint = 1024;
pub const BCO_NOHELP: c_uint = 4;
pub const BCO_ENTER: c_uint = 1;
pub const VGR_FUZZY: c_uint = 4;
pub const VGR_NOJUMP: c_uint = 2;
pub const VGR_GLOBAL: c_uint = 1;
pub const QF_WINHEIGHT: c_uint = 10;
crate::flag_set! {
    /// Which properties `getqflist({what})` was asked for -- one bit per key
    /// of the `what` dictionary, and the same set of keys in the same order
    /// on the way out.
    pub struct GetListProps;

    const TITLE = 1;
    const ITEMS = 2;
    const NR = 4;
    const WINID = 8;
    const CONTEXT = 16;
    const ID = 32;
    const IDX = 64;
    const SIZE = 128;
    const TICK = 256;
    /// A location list's owning window. Answered for a location list only,
    /// which is why [`Self::ALL`] has to have it taken back out again for a
    /// quickfix list.
    const FILEWINID = 512;
    const QFBUFNR = 1024;
    const QFTF = 2048;

    /// What `{'all': 1}` asks for: every bit above.
    const ALL = 4095;
}
pub const CMDBUFFSIZE: c_int = 1024;
pub const INVALID_QFIDX: c_int = -1;
pub const INVALID_QFBUFNR: c_int = 0;
/// Messages more than one child reports.
pub(crate) const E_NO_MORE_ITEMS: &CStr = c"E553: No more items";
pub(crate) const E_QUICKFIX_LIST_CHANGED: &CStr = c"E925: Current quickfix list was changed";
pub(crate) const E_LOCATION_LIST_CHANGED: &CStr = c"E926: Current location list was changed";
static qftf_cb: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static qfFile_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
static qfSep_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
static qfLine_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
pub const BUF_HAS_QF_ENTRY: c_int = 1;
pub const BUF_HAS_LL_ENTRY: c_int = 2;
pub const EINTR: c_int = 4;
pub const INT_MAX: c_int = 2147483647;
