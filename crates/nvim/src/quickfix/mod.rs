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
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_FILETYPE, EVENT_QUICKFIXCMDPOST,
    EVENT_QUICKFIXCMDPRE, apply_autocmds, au_event_disable, au_event_restore, aucmd_prepbuf,
    aucmd_restbuf, block_autocmds, unblock_autocmds,
};
use crate::buffer::{
    bt_help, bt_normal, bt_quickfix, buf_valid, buflist_findname_exp, buflist_findnr,
    buflist_getfile, buflist_new, bufref_valid, close_buffer, do_modelines, no_write_message,
    set_bufref, setfname, wipe_buffer,
};
use crate::change::changed_lines;
use crate::charset::{skipdigits, skipwhite, vim_isprintc};
use crate::cursor::{check_cursor, coladvance};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_later, redraw_curbuf_later, update_screen,
};
use crate::edit::beginline;
use crate::eval::typval::{
    callback_copy, callback_free, callback_put, kCallbackNone, tv_clear, tv_copy, tv_dict_add,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_add_tv, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number,
    tv_dict_get_string, tv_dict_get_tv, tv_dict_item_alloc_len, tv_dict_item_free, tv_dict_unref,
    tv_free, tv_get_number_chk, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_first, tv_list_len, tv_list_ref,
};
use crate::eval::vars::set_internal_string_var;
use crate::eval::window::{find_win_by_nr_or_id, win_id2wp};
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
use crate::fold::{foldOpenCursor, foldUpdateAll};
use crate::fuzzy::fuzzy_match;
use crate::global_cell::GlobalCell;
use crate::help::check_help_lang;
use crate::highlight_group::syn_name2id;
use crate::main::{
    Columns, IObuff, KeyTyped, NameBuff, cmdline_row, cmdmod, curbuf, curtab, curwin,
    e_au_recursive, e_buffer_is_not_loaded, e_dictreq, e_invalpat, e_invarg, e_invarg2, e_invrange,
    e_listreq, e_loclist, e_no_errors, e_nomatch, e_nomatch2, e_noprevre, e_notmp, e_openerrf,
    e_readerrf, e_string_required, e_trailing_arg, e_winfixbuf_cannot_go_to_buffer,
    empty_string_option, fdo_flags, first_tabpage, firstwin, got_int, lastwin, msg_col, msg_didout,
    msg_nowait, msg_scroll, msg_scrolled, must_redraw, p_ch, p_chi, p_cpo, p_ef, p_efm, p_enc,
    p_gefm, p_gp, p_hh, p_ic, p_mef, p_menc, p_mls, p_qftf, p_rtp, p_shq, p_sp, p_swb, prevwin,
    restart_edit, swb_flags, textlock,
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
    FreeWild, PATHSEP, after_pathsep, concat_fnames, fix_fname, gen_expand_wildcards,
    path_fnamecmp, path_is_absolute, path_tail, path_try_shorten_fname, vim_isAbsName,
};
use crate::pos::MAXLNUM;
use crate::regexp::{vim_regcomp, vim_regexec, vim_regexec_multi, vim_regfree};
use crate::search::{BACKWARD, BACKWARD_FILE, FORWARD, FORWARD_FILE, do_search, last_search_pat};
use crate::strings::{has_non_ascii, vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::types::builders::static_cstring;
use crate::types::{
    Callback, Callback_data, DirStack, Direction, EvalFuncData, ExtmarkOp, FILE, FileInfo, OptInt,
    OptVal, OptValData, OptValType, QFLT_INTERNAL, QFLT_LOCATION, QFLT_QUICKFIX, VarType,
    aco_save_T, bln_values, buf_T, bufref_T, cleanup_T, cmdidx_T, colnr_T, dict_T, dictitem_T,
    dobuf_action_values, exarg_T, getf_values, handle_T, linenr_T, list_T, listitem_T, optset_T,
    pos_T, ptrdiff_t, qf_info_T, qf_list_T, qfline_T, qfltype_T, regmatch_T, regmmatch_T,
    regprog_T, scid_T, size_t, time_t, typval_T, typval_vval_union, varnumber_T, vimconv_T, win_T,
};
use crate::ui::ui_flush;
use crate::undo::u_clearallandblockfree;
use crate::window::{
    check_can_set_curbuf_forceit, check_lnums, goto_tabpage_win, win_close, win_enter, win_goto,
    win_setheight, win_split, win_valid,
};
use ::libc::{
    __errno_location, abort, abs, atoi, atol, fclose, fdopen, ferror, fgets, strcmp, strlen, time,
};
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
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_DUMMY: bln_values = 4;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const SHM_OVERALL: c_uint = 79;
pub const BL_FIX: c_uint = 4;
pub const BL_WHITE: c_uint = 1;
pub const ECMD_NOWINENTER: c_uint = 64;
pub const ECMD_OLDBUF: c_uint = 4;
pub const ECMD_SET_HELP: c_uint = 2;
pub const ECMD_HIDE: c_uint = 1;
pub const ECMD_ONE: c_int = 1;
pub const READ_DUMMY: c_uint = 16;
pub const READ_NEW: c_uint = 1;
pub const FUZZY_MATCH_MAX_LEN: c_uint = 1024;
pub const BCO_NOHELP: c_uint = 4;
pub const BCO_ENTER: c_uint = 1;
pub const OPT_NOWIN: c_uint = 16;
pub const OPT_LOCAL: c_uint = 2;
pub const VGR_FUZZY: c_uint = 4;
pub const VGR_NOJUMP: c_uint = 2;
pub const VGR_GLOBAL: c_uint = 1;
pub const QF_WINHEIGHT: c_uint = 10;
pub const QF_GETLIST_QFTF: c_uint = 2048;
pub const QF_GETLIST_NONE: c_uint = 0;
pub const QF_GETLIST_QFBUFNR: c_uint = 1024;
pub const QF_GETLIST_FILEWINID: c_uint = 512;
pub const QF_GETLIST_TICK: c_uint = 256;
pub const QF_GETLIST_SIZE: c_uint = 128;
pub const QF_GETLIST_IDX: c_uint = 64;
pub const QF_GETLIST_ITEMS: c_uint = 2;
pub const QF_GETLIST_ID: c_uint = 32;
pub const QF_GETLIST_CONTEXT: c_uint = 16;
pub const QF_GETLIST_WINID: c_uint = 8;
pub const QF_GETLIST_NR: c_uint = 4;
pub const QF_GETLIST_TITLE: c_uint = 1;
pub const QF_GETLIST_ALL: c_uint = 4095;
pub const MAXPATHL: c_int = 4096;
pub const CMDBUFFSIZE: c_int = 1024;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const ML_EMPTY: c_int = 0x1;
pub const INVALID_QFIDX: c_int = -1;
pub const INVALID_QFBUFNR: c_int = 0;
/// Messages more than one child reports.
pub(crate) const E_NO_MORE_ITEMS: &CStr = c"E553: No more items";
pub(crate) const E_QUICKFIX_LIST_CHANGED: &CStr = c"E925: Current quickfix list was changed";
pub(crate) const E_LOCATION_LIST_CHANGED: &CStr = c"E926: Current location list was changed";
static qftf_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: core::ptr::null_mut(),
    },
    type_0: kCallbackNone,
});
static qfFile_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
static qfSep_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
static qfLine_hl_id: GlobalCell<c_int> = GlobalCell::new(0);
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const BF_NEW: c_int = 0x10;
pub const BF_DUMMY: c_int = 0x80;
pub const BUF_HAS_QF_ENTRY: c_int = 1;
pub const BUF_HAS_LL_ENTRY: c_int = 2;
pub const IOSIZE: c_int = 1024 + 1;
pub const EINTR: c_int = 4;
pub const INT_MAX: c_int = 2147483647;
