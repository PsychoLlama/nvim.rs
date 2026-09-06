//! Searching.
//!
//! This module is the root of the family and holds nothing but the
//! namespace its children share: [`pattern`] remembers the patterns,
//! [`find`] runs one over the buffer, [`command`] is `/` and `?`,
//! [`charsearch`] is `f`/`t`, [`select`] is `gn`, [`stat`] is the
//! `[1/15]` count, [`matchpair`] and [`comment`] are `%`, and
//! [`includes`] with [`incline`] is `[i`/`:checkpath`.

#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::ascii::ascii_isdigit;
use crate::autocmd::apply_autocmds;
use crate::buffer::buf_get_changedtick;
use crate::change::get_leader_len;
use crate::charset::{skipwhite, vim_isfilec, vim_iswordc, vim_iswordp};
use crate::cmdhist::add_to_history;
use crate::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, inc_cursor,
};
use crate::drawscreen::state::{dollar_vcol, sc_col};
use crate::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later, redraw_later,
    setcursor, show_cursor_info_later, showmode, update_screen,
};
use crate::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_dict_find,
    tv_get_number_chk, tv_list_find, tv_list_len,
};
use crate::eval::vars::set_vim_var_nr;
use crate::ex_cmds::{getfile, prepare_tagpreview};
use crate::ex_docmd::set_no_hlsearch;
use crate::ex_getln::gotocmdline;
use crate::file_search::{file_name_in_line, find_file_name_in_path};
use crate::fileio::vim_fgets;
use crate::fold::{fold_open_cursor, has_folding};
use crate::getchar::char_avail;
use crate::getchar::state::{KeyStuffed, KeyTyped, got_int};
use crate::global_cell::GlobalCell;
use crate::indent_c::is_pos_in_string;
use crate::insexpand::{
    compl_status_adding, compl_status_sol, ctrl_x_mode_not_default, find_word_end, find_word_start,
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted, ins_compl_len,
};
use crate::mark::setpcmark;
use crate::mbyte::{
    mb_isupper, mb_strcmp_ic, mb_strnicmp, utf_char2bytes, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utfc_ptr2len,
};
use crate::memline::{decl, inc, incl, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::memory::{xfree, xmalloc, xmemdupz, xstrlcpy};
use crate::message::state::{
    bot_top_msg, called_emsg, cmd_silent, msg_ext_overwrite, msg_hist_off, msg_nowait, msg_row,
    msg_scrolled, msg_silent, top_bot_msg,
};
use crate::message::{e_interr, e_nopresub, e_noprevre, e_patnotf2};
use crate::message::{
    emsg, give_warning, iemsg, messaging, msg, msg_check, msg_clr_eos, msg_end, msg_ext_set_kind,
    msg_home_replace, msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, msg_start, msg_strtrunc, msg_trunc, verbose_enter, verbose_leave,
};
use crate::mouse::setmouse;
use crate::r#move::validate_cursor;
use crate::normal::may_start_select;
use crate::option::vars::{
    fdo_flags, p_def, p_hls, p_ic, p_inc, p_js, p_mat, p_msc, p_ri, p_scs, p_sel, p_verbose, p_ws,
};
use crate::option::{magic_isset, shortmess};
use crate::options::{kOptBoFlagShowmatch, kOptFdoFlagSearch};
use crate::os::cshim::{gettext, snprintf, strstr};
use crate::os::fs::os_fopen;
use crate::os::input::{fast_breakcheck, line_breakcheck};
use crate::os::time::{os_delay, os_time};
use crate::path::path_full_compare;
use crate::plines::getvcol;
use crate::pos::{clearpos, equalpos, lt, ltoreq};
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{skip_regexp_ex, vim_regcomp, vim_regexec, vim_regexec_multi, vim_regfree};
use crate::search::state::{
    no_hlsearch, no_smartcase, search_match_endcol, search_match_lines, searchcmdlen,
};
use crate::state::MODE_SHOWMATCH;
use crate::state::mode::State;
use crate::strings::{reverse_text, vim_snprintf, vim_strchr, xstrnsave};
use crate::tag::state::g_do_tagpreview;
use crate::types::AutoEvent;
use crate::types::TAB;
use crate::types::ui::kUIMessages;
use crate::types::{
    CmdArg, ColNr, Dict, Direction, EvalFuncData, FILE, FileComparison, LPos, LineNr, List, Magic,
    MotionType, OpArg, Pos, ProfTime, RegMMatch, RegMatch, SearchItArg, SearchOffset,
    SearchPattern, TypVal, VarNumber, int64_t, ptrdiff_t, size_t,
};
use crate::ui::state::{Columns, Rows};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush, ui_has, vim_beep};
use crate::window::{win_enter, win_split};
use ::libc::{atol, fclose, strpbrk};
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
pub const MAGIC_ALL: Magic = 4;
pub const MAGIC_ON: Magic = 3;
pub const HIST_SEARCH: ::core::ffi::c_int = 1;
pub const kMTLineWise: MotionType = 1;
pub const kEqualFiles: FileComparison = 1;
pub const CHECK_PATH: ::core::ffi::c_uint = 3;
pub const FIND_DEFINE: ::core::ffi::c_uint = 2;
pub const ACTION_EXPAND: ::core::ffi::c_uint = 5;
pub const ACTION_SHOW_ALL: ::core::ffi::c_uint = 4;
pub const ACTION_SPLIT: ::core::ffi::c_uint = 3;
pub const ACTION_SHOW: ::core::ffi::c_uint = 1;
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
pub const FM_BLOCKSTOP: ::core::ffi::c_uint = 4;
pub const FM_FORWARD: ::core::ffi::c_uint = 2;
pub const FM_BACKWARD: ::core::ffi::c_uint = 1;
pub const SEARCH_STAT_DEF_TIMEOUT: ::core::ffi::c_int = 40;
pub const SEARCH_STAT_BUF_LEN: ::core::ffi::c_int = 16;
pub const LSIZE: ::core::ffi::c_uint = 512;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub(crate) mod state;
