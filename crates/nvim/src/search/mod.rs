//! Searching.
//!
//! This module is the root of the family and holds nothing but the
//! namespace its children share: [`pattern`] remembers the patterns,
//! [`find`] runs one over the buffer, [`command`] is `/` and `?`,
//! [`charsearch`] is `f`/`t`, [`select`] is `gn`, [`stat`] is the
//! `[1/15]` count, [`matchpair`] and [`comment`] are `%`, and
//! [`includes`] with [`incline`] is `[i`/`:checkpath`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::ascii_isdigit;
use crate::autocmd::{EVENT_SEARCHWRAPPED, apply_autocmds};
use crate::buffer::buf_get_changedtick;
use crate::change::get_leader_len;
use crate::charset::{skipwhite, vim_isfilec, vim_iswordc, vim_iswordp};
use crate::cmdhist::add_to_history;
use crate::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, inc_cursor,
};
use crate::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later, redraw_later,
    setcursor, show_cursor_info_later, showmode, update_screen,
};
use crate::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_dict_find,
    tv_get_number_chk, tv_get_string_chk, tv_list_find, tv_list_len,
};
use crate::eval::vars::set_vim_var_nr;
use crate::ex_cmds::{getfile, prepare_tagpreview};
use crate::ex_docmd::set_no_hlsearch;
use crate::ex_getln::gotocmdline;
use crate::file_search::{file_name_in_line, find_file_name_in_path};
use crate::fileio::vim_fgets;
use crate::fold::{fold_open_cursor, has_folding};
use crate::getchar::char_avail;
use crate::global_cell::GlobalCell;
use crate::indent_c::is_pos_in_string;
use crate::insexpand::{
    compl_status_adding, compl_status_sol, ctrl_x_mode_not_default, find_word_end, find_word_start,
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted, ins_compl_len,
};
use crate::main::{
    Columns, IObuff, KeyStuffed, KeyTyped, Rows, State, VIsual, VIsual_active, VIsual_mode,
    bot_top_msg, called_emsg, cmd_silent, curbuf, curwin, dollar_vcol, e_interr, e_invarg2,
    e_nopresub, e_noprevre, e_patnotf2, fdo_flags, g_do_tagpreview, got_int, msg_ext_overwrite,
    msg_hist_off, msg_nowait, msg_row, msg_scrolled, msg_silent, no_hlsearch, no_smartcase, p_def,
    p_hls, p_ic, p_inc, p_js, p_mat, p_msc, p_ri, p_scs, p_sel, p_siso, p_so, p_verbose, p_ws,
    rc_did_emsg, sc_col, search_match_endcol, search_match_lines, searchcmdlen, top_bot_msg,
};
use crate::mark::setpcmark;
use crate::mbyte::{
    mb_isupper, mb_strcmp_ic, mb_strnicmp, utf_char2bytes, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utfc_ptr2len,
};
use crate::memline::{decl, inc, incl, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::memory::{xfree, xmalloc, xmemdupz, xstrlcpy};
use crate::message::{
    emsg, give_warning, iemsg, messaging, msg, msg_check, msg_clr_eos, msg_end, msg_ext_set_kind,
    msg_home_replace, msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, msg_start, msg_strtrunc, msg_trunc, verbose_enter, verbose_leave,
};
use crate::mouse::setmouse;
use crate::r#move::validate_cursor;
use crate::normal::may_start_select;
use crate::option::{magic_isset, shortmess};
use crate::options::{kOptBoFlagShowmatch, kOptFdoFlagSearch};
use crate::os::cshim::{gettext, snprintf, strncmp, strstr};
use crate::os::fs::os_fopen;
use crate::os::input::{fast_breakcheck, line_breakcheck};
use crate::os::time::{os_delay, os_time};
use crate::path::path_full_compare;
use crate::plines::getvcol;
use crate::pos::{clearpos, equalpos, lt, ltoreq};
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::{skip_regexp_ex, vim_regcomp, vim_regexec, vim_regexec_multi, vim_regfree};
use crate::state::MODE_SHOWMATCH;
use crate::strings::{reverse_text, vim_snprintf, vim_strchr, xstrnsave};
use crate::types::ui::kUIMessages;
use crate::types::{
    Direction, EvalFuncData, FILE, MotionType, OptInt, SearchOffset, SearchPattern, buf_T,
    cmdarg_T, colnr_T, dict_T, file_comparison, int64_t, linenr_T, list_T, lpos_T, magic_T,
    oparg_T, pos_T, proftime_T, ptrdiff_t, regmatch_T, regmmatch_T, searchit_arg_T, size_t,
    typval_T, varnumber_T, win_T,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush, ui_has, vim_beep};
use crate::window::{win_enter, win_split, win_valid};
use ::libc::{atol, fclose, strlen, strpbrk};
/// `searchit`/`do_search` flags plus the search-stat sizing constants.
pub const SEARCH_HL_PRIORITY: ::core::ffi::c_int = 0;
pub const SEARCH_NFMSG: ::core::ffi::c_int = 8;

// The carve of the transpiled module; see each child's docs.
mod pattern;
pub use self::pattern::*;
mod find;
pub use self::find::*;
mod charsearch;
pub use self::charsearch::*;
mod command;
pub use self::command::*;
mod incline;
pub(crate) use self::incline::*;
mod includes;
pub use self::includes::*;
mod comment;
pub use self::comment::*;
mod select;
pub use self::select::*;
mod matchpair;
pub use self::matchpair::*;
mod stat;
pub use self::stat::*;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kMTLineWise: MotionType = 1;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_22 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_22 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_23 = 5;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_23 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_23 = 3;
pub const ACTION_SHOW: C2Rust_Unnamed_23 = 1;
pub const SEARCH_COL: ::core::ffi::c_int = 4096;
pub const SEARCH_PEEK: ::core::ffi::c_int = 2048;
pub const SEARCH_KEEP: ::core::ffi::c_int = 1024;
pub const SEARCH_MARK: ::core::ffi::c_int = 512;
pub const SEARCH_START: ::core::ffi::c_int = 256;
pub const SEARCH_NOOF: ::core::ffi::c_int = 128;
pub const SEARCH_END: ::core::ffi::c_int = 64;
pub const SEARCH_HIS: ::core::ffi::c_int = 32;
pub const SEARCH_OPT: ::core::ffi::c_int = 16;
pub const SEARCH_MSG: ::core::ffi::c_int = 12;
pub const SEARCH_ECHO: ::core::ffi::c_int = 2;
pub const SEARCH_REV: ::core::ffi::c_int = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_25 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_25 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_25 = 1;
pub const SEARCH_STAT_DEF_TIMEOUT: ::core::ffi::c_int = 40;
pub const SEARCH_STAT_BUF_LEN: ::core::ffi::c_int = 16;
pub const LSIZE: C2Rust_Unnamed_29 = 512;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
