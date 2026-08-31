#![deny(unsafe_op_in_unsafe_fn)]
use crate::arglist::get_arglist_name;

use crate::api::private::helpers::{api_free_object, cstr_as_string};
use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::autocmd::{expand_get_augroup_name, expand_get_event_name, set_context_in_autocmd};
use crate::buffer::expand_buf_names;
use crate::charset::{
    backslash_halve_save, ptr2cells, rem_backslash, skipdigits, skiptowhite, skipwhite, transchar,
    transchar_byte, vim_is_ident_char, vim_isfilec_or_wc, vim_strsize,
};
use crate::cmdhist::get_history_arg;
use crate::drawscreen::{redraw_statuslines, update_screen, win_redraw_last_status};
use crate::eval::funcs::{get_expr_name, get_function_name};
use crate::eval::typval::{
    tv_check_for_string_arg, tv_clear, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc_ret, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_string,
    tv_list_unref,
};
use crate::eval::userfunc::get_user_func_name;
use crate::eval::vars::get_user_var_name;
use crate::eval::{call_func_retlist, call_func_retstr, set_context_for_expression};
use crate::ex_cmds::skip_vimgrep_pat;
use crate::ex_docmd::{
    ends_excmd, excmd_get_argt, excmd_get_cmdidx, expand_argopt, expand_findfunc, find_nextcmd,
    get_command_name, set_no_hlsearch, skip_cmd_arg, skip_range,
};
use crate::ex_getln::{
    cmd_screencol, cursorcmd, escape_fname, get_cmdline_last_prompt_id, parse_pattern_and_range,
    put_on_cmdline, realloc_cmdbuff, redrawcmd, tilde_replace, vim_strsave_fnameescape,
};
use crate::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::garray::{ga_append, ga_clear_strings, ga_concat_len, ga_grow, ga_init};
use crate::getchar::{beep_flush, char_avail, vpeekc};
use crate::global_cell::GlobalCell;
use crate::grid::{grid_line_fill, grid_line_flush, grid_line_puts, grid_line_start};
use crate::hashtab::{hash_add_item, hash_hash, hash_lookup};
use crate::help::{cleanup_help_tags, find_help_tags};
use crate::highlight::win_hl_attr;
use crate::highlight_group::{
    HLF_D, HLF_NONE, HLF_T, HLF_WM, get_highlight_name, set_context_in_highlight_cmd,
};
use crate::insexpand::find_word_end;
use crate::keycodes::{K_DOWN, K_KENTER, K_LEFT, K_RIGHT, K_UP};
use crate::lua::executor::{
    nlua_call_user_expand_func, nlua_exec, nlua_expand_get_matches, nlua_expand_pat,
};
use crate::main::{
    Columns, KeyTyped, Rows, cmd_silent, cmdline_row, cmdline_win, curbuf, current_sctx, curwin,
    e_invarg, e_toomany, got_int, hl_attr_active, msg_col, msg_didany, msg_row, msg_scrolled,
    p_fic, p_ic, p_ls, p_scs, p_wc, p_wic, p_wmh, p_wmnu, pum_want, save_p_ls, save_p_wmh,
    search_first_line, search_last_line, topframe, wild_menu_showing, wop_flags,
};
use crate::mapping::{expand_mappings, set_context_in_map_cmd};
use crate::mbyte::{mb_tolower, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_len};
use crate::memory::{xfree, xmalloc, xmemcpyz, xmemdupz, xstpcpy, xstrdup};
use crate::menu::{get_menu_name, get_menu_names, menu_is_separator, set_context_in_menu_cmd};
use crate::message::{
    emsg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_grid_view, msg_outtrans,
    msg_outtrans_long, msg_putchar, msg_puts, msg_puts_hl, msg_scroll_up, msg_start,
};
use crate::option::{
    copy_option_part, csh_like_shell, expand_old_setting, expand_setting_subtract, expand_settings,
    expand_string_setting, get_findfunc, magic_isset, set_context_in_set_cmd,
};
use crate::options::{
    kOptBoFlagWildmode, kOptWopFlagExacttext, kOptWopFlagFuzzy, kOptWopFlagPum, kOptWopFlagTagfile,
};
use crate::os::cshim::{gettext, snprintf, strchr};
use crate::os::env::{expand_env_save_opt, get_env_name, home_replace, vim_getenv};
use crate::os::fs::os_isdir;
use crate::os::lang::{get_lang_arg, get_locales};
use crate::os::users::{UserMatch, get_users, match_user};
use crate::path::{
    after_pathsep, expand_wildcards, expand_wildcards_eval, free_wild, match_suffix,
    path_is_absolute, path_tail, vim_ispathsep,
};
use crate::popupmenu::{pum_clear, pum_display, pum_get_height, pum_undisplay, pum_visible};
use crate::pos::ltoreq;
use crate::profile::{get_profile_name, set_context_in_profile_cmd};
use crate::regexp::{
    RE_LAST, RE_MAGIC, RE_STRING, skip_regexp, vim_regcomp, vim_regexec, vim_regexec_nl,
    vim_regfree,
};
use crate::runtime::{
    RuntimeOpts, expand_packadd_dir, expand_runtime_cmd, expand_runtime_dir, script_id_valid,
    script_item, set_context_in_runtime_cmd,
};
use crate::search::{
    BACKWARD, FORWARD, SEARCH_NFMSG, SEARCH_NOOF, SEARCH_OPT, SEARCH_PEEK, SEARCH_START,
    ignorecase, pat_has_uppercase, searchit,
};
use crate::sign::{get_sign_name, set_context_in_sign_cmd};
use crate::statusline::fillchar_status;
use crate::strings::{sort_strings, strcase_save, vim_strchr, vim_strsave_escaped, xstrnsave};
use crate::syntax::{
    get_syntax_name, get_syntime_arg, reset_expand_highlight, set_context_in_echohl_cmd,
    set_context_in_syntax_cmd,
};
use crate::tag::expand_tags;
use crate::types::ui::{kUICmdline, kUIMessages, kUIPopupmenu, kUIWildmenu};
use crate::types::{
    Arena, Array, CmdAddr, CompleteListItemGetter, Direction, Error, EvalFuncData, LuaRetMode,
    Object, OptInt, buf_T, cmdidx_T, colnr_T, dict_T, exarg_T, expand_T, fuzmatch_str_T, garray_T,
    hashtab_T, hlf_T, kObjectTypeArray, kObjectTypeString, list_T, listitem_T, pos_T, ptrdiff_t,
    pumitem_T, regmatch_T, size_t, ssize_t, typval_T, typval_vval_union, varnumber_T, xp_prefix_T,
};
use crate::ui::{ui_flush, ui_has, vim_beep};
use crate::usercmd::{
    cmdcomplete_str_to_type, cmdcomplete_type_to_str, find_ucmd, get_user_cmd_addr_type,
    get_user_cmd_complete, get_user_cmd_flags, get_user_cmd_nargs, get_user_commands,
    set_context_in_user_cmd, set_context_in_user_cmdarg,
};
use crate::window::{global_stl_height, last_status};
use crate::winlayer::{Cc, Live};
use ::libc::{qsort, strcpy, strncpy};
use core::ffi::CStr;

// The carve of the transpiled module; see each child's docs.
mod escape;
pub use self::escape::*;
mod expandone;
pub use self::expandone::*;
mod pum;
pub use self::pum::*;
mod showmatch;
pub use self::showmatch::*;
mod context;
pub use self::context::*;
mod cmdname;
pub use self::cmdname::*;
mod generate;
pub(crate) use self::generate::*;
mod fromcontext;
pub use self::fromcontext::*;
mod userfunc;
pub use self::userfunc::*;
mod wildkey;
pub(crate) use self::wildkey::*;
mod eval;
pub use self::eval::*;
mod bufpat;
pub(crate) use self::bufpat::*;
/// The completion context an expansion is running in, whose caller has
/// promised it outlives the value.
///
/// The promise is discharged by the frame that owns the `expand_T`: the
/// command line's own `xpc`, or a caller's local. Wrapping is the unsafe
/// step, once per entry point, and every `(*xp).field` after it is ordinary
/// checked code -- which also stops the 1 KiB struct being *copied* every
/// time a field is read, as `unsafe { (*xp).xp_context }` does.
///
/// Two addresses may not be taken off one [`Deref`](core::ops::Deref) -- the
/// second borrow pops the first -- so a caller wanting `&raw mut` on a field
/// takes it off [`Live::field_ptr`] instead.
pub(crate) type Xp = Live<expand_T>;

pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
/// Not a `WILD_*` at all — `buffer.h`'s, and `expand_buf_names` reads it out
/// of the same `options` word, so it is spelled as one of them.
pub const BUF_DIFF_FILTER: WildOpts = WildOpts::from_bits(8192);

/// What [`expand_one`] should do — upstream's `WILD_*` *modes*, which are an
/// enumeration and not a flag set: exactly one is passed, and the value space
/// (1..=13) collides with [`WildOpts`]'s bits one for one.
///
/// c2rust gave both families the same `c_int`, so `expand_one(xp, s, o,
/// WILD_ALL, WILD_SILENT)` — arguments swapped — compiled. As an enum the
/// swap does not, and [`next_match`](expandone) can match exhaustively
/// instead of leaning on a `_` arm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WildMode {
    /// Just release the previously expanded matches.
    Free,
    /// Expand, and do not keep the matches.
    ExpandFree,
    /// Expand, and keep the matches for a later `Next`/`Prev`.
    ExpandKeep,
    /// Step forward through the matches, wrapping around.
    Next,
    /// Step back through the matches, wrapping around.
    Prev,
    /// Answer every match, concatenated.
    All,
    /// Answer the longest part every match starts with.
    Longest,
    /// Answer every match and keep them.
    AllKeep,
    /// Close the popup menu and go back to the original text.
    Cancel,
    /// Apply the item selected in the popup menu.
    Apply,
    /// Move the selection a page towards the start.
    PageUp,
    /// Move the selection a page towards the end.
    PageDown,
    /// Select the item the UI named in `pum_want`.
    PumWant,
}

impl WildMode {
    /// Whether this mode only moves within a match list that already exists,
    /// rather than expanding anything.
    pub const fn navigates(self) -> bool {
        matches!(
            self,
            Self::Next | Self::Prev | Self::PageUp | Self::PageDown | Self::PumWant
        )
    }
}

crate::flag_set! {
    /// How an expansion should behave — upstream's `WILD_*` options, the
    /// `options` argument to [`expand_one`] and [`nextwild`].
    ///
    /// Distinct from the `mode` argument beside it, which is an enumeration
    /// ([`WildMode`]) whose values run 1..=13 and therefore collide with
    /// these bit values one for one. Nothing but the parameter name kept
    /// them apart while both were `int`.
    pub struct WildOpts;

    /// Answer a pattern that matched nothing as itself.
    const LIST_NOTFOUND = 1;
    /// Shorten a name under `$HOME` to `~/`.
    const HOME_REPLACE = 2;
    /// Separate the concatenated matches with newlines, not spaces.
    const USE_NL = 4;
    /// Do not beep when there is nothing to complete.
    const NO_BEEP = 8;
    /// Append a path separator to every directory answered.
    const ADD_SLASH = 16;
    /// Do not drop the matches `'wildignore'` and `'suffixes'` name.
    const KEEP_ALL = 32;
    /// Do not report a failure to the user.
    const SILENT = 64;
    /// Escape the answer for the command line it is going back into.
    const ESCAPE = 128;
    /// Match without regard to case.
    const ICASE = 256;
    /// Answer a dangling symbolic link as a match.
    const ALLLINKS = 512;
    /// Leave a trailing slash alone, whatever `'completeslash'` says.
    const IGNORE_COMPLETESLASH = 1024;
    /// Do not report a pattern that could not be expanded at all.
    const NOERROR = 2048;
    /// Order the buffer matches by when they were last used.
    const BUFLASTUSED = 4096;
    /// Leave the popup menu with nothing selected.
    const NOSELECT = 16384;
    /// `'wildmode'` says the pattern itself may be one of the answers.
    const MAY_EXPAND_PATTERN = 32768;
    /// The completion was asked for by `'wildmode'`'s function trigger.
    const FUNC_TRIGGER = 65536;
}
pub const VSE_NONE: ::core::ffi::c_int = 0;
pub const VSE_BUFFER: ::core::ffi::c_int = 2;
pub const VSE_SHELL: ::core::ffi::c_int = 1;
pub const kRetObject: LuaRetMode = 0;
pub const EXP_BREAKPT_DEL: BreakpointExpandKind = 1;
pub type BreakpointExpandKind = ::core::ffi::c_uint;
pub const EXP_PROFDEL: BreakpointExpandKind = 2;
pub const EXP_BREAKPT_ADD: BreakpointExpandKind = 0;
pub const FUZZY_SCORE_NONE: ::core::ffi::c_int = -2147483648;
pub const TAG_MANY: ::core::ffi::c_int = 300;
pub const WM_SCROLLED: ::core::ffi::c_int = 2;
pub const WM_SHOWN: ::core::ffi::c_int = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
static cmd_showtail: GlobalCell<bool> = GlobalCell::new(false);
static may_expand_pattern: GlobalCell<bool> = GlobalCell::new(false);
static pre_incsearch_pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
static compl_match_array: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
static compl_match_arraysize: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static compl_startcol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static compl_selected: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static cmdline_orig: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
/// How much of `:filetype` has already been typed, and so which of its
/// arguments are still worth offering -- upstream's `EXP_FILETYPECMD_*`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum FiletypeWhat {
    /// Nothing after `:filetype`.
    All,
    /// `indent` named; `plugin` is still on offer.
    Plugin,
    /// `plugin` named; `indent` is still on offer.
    Indent,
    /// Both named, so only `on`/`off` remain.
    OnOff,
}

static filetype_expand_what: GlobalCell<FiletypeWhat> = GlobalCell::new(FiletypeWhat::All);
static breakpt_expand_what: GlobalCell<BreakpointExpandKind> = GlobalCell::new(EXP_BREAKPT_ADD);
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
