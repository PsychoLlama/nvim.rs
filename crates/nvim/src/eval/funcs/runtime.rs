//! What the editor is right now: `has()`, `mode()`, `state()` and the rest
//! of the feature and status queries.
#![forbid(unsafe_code)]

use super::wrappers::{arg_lnum, arg_number, arg_number_chk, non_zero_arg};
use super::{MENU_ALL_MODES, kRetNilBool};
use crate::api::private::helpers::api_metadata;
use crate::ascii::ascii_iswhite;
use crate::autocmd::state::autocmd_busy;
use crate::cmdexpand::cmdline_pum_active;
use crate::cmdexpand::state::wild_menu_showing;
use crate::eval::typval::{NumBuf, tv_dict_alloc_ret, tv_list_alloc_ret};
use crate::eval::vars::{get_vim_var_nr, set_vim_var_nr};
use crate::eval::{eval_has_provider, get_callback_depth};
use crate::getchar::state::vgetc_busy;
use crate::getchar::{stuff_empty, typeahead, using_script};
use crate::global_cell::GlobalCell;
use crate::indent::{get_sw_value, get_sw_value_col};
use crate::insexpand::ins_compl_active;
use crate::lua::executor::nlua_exec_cstr;
use crate::memline::Lines;
use crate::memory::ThinCString;
use crate::menu::{cmd_modes, menu_get};
use crate::message::state::msg_scrolled;
use crate::normal::op_pending;
use crate::ops::cursor_pos_info;
use crate::os::cshim::cstr_ncasecmp;
use crate::os::env::{os_get_pid, os_hostname_into};
use crate::os::state::windowsVersion;
use crate::popupmenu::{pum_set_event_info, pum_visible};
use crate::startup::{starting, stdin_isatty, stdout_isatty};
use crate::state::mode::State;
use crate::state::{MODE_CMDLINE, get_mode, get_was_safe_state};
use crate::strings::has_char;
use crate::syntax::syntax_present;
use crate::types::{
    Array, ColNr, Error, EvalFuncData, NUL, Object, TypVal, VAR_STRING, VarNumber, Vv,
    kListLenMayKnow,
};
use crate::ui::ui_gui_attached;
use crate::version::{has_nvim_version, has_vim_patch};
use crate::window::find_tabpage;
use crate::winlayer::{Buf, Win};
use crate::winlayer::{TabPage, windows_in_tab};
use core::ffi::{CStr, c_char, c_int};

/// The features `has()` answers yes to unconditionally.
///
/// Kept in the C's order, which is neither alphabetical nor meaningful --
/// it is a linear scan, so the order is not observable.
const FEATURES: [&CStr; 90] = [
    c"linux",
    c"unix",
    c"fname_case",
    c"acl",
    c"autochdir",
    c"arabic",
    c"autocmd",
    c"browsefilter",
    c"byte_offset",
    c"cindent",
    c"cmdline_compl",
    c"cmdline_hist",
    c"cmdwin",
    c"comments",
    c"conceal",
    c"cursorbind",
    c"cursorshape",
    c"dialog_con",
    c"diff",
    c"digraphs",
    c"eval",
    c"ex_extra",
    c"extra_search",
    c"file_in_path",
    c"filterpipe",
    c"find_in_path",
    c"float",
    c"folding",
    c"fork",
    c"gettext",
    c"iconv",
    c"insert_expand",
    c"jumplist",
    c"keymap",
    c"lambda",
    c"langmap",
    c"libcall",
    c"linebreak",
    c"lispindent",
    c"listcmds",
    c"localmap",
    c"menu",
    c"mksession",
    c"modify_fname",
    c"mouse",
    c"multi_byte",
    c"multi_lang",
    c"nanotime",
    c"num64",
    c"packages",
    c"path_extra",
    c"persistent_undo",
    c"profile",
    c"reltime",
    c"quickfix",
    c"rightleft",
    c"scrollbind",
    c"showcmd",
    c"cmdline_info",
    c"shada",
    c"signs",
    c"smartindent",
    c"startuptime",
    c"statusline",
    c"spell",
    c"syntax",
    c"tablineat",
    c"tag_binary",
    c"termguicolors",
    c"terminfo",
    c"termresponse",
    c"textobjects",
    c"timers",
    c"title",
    c"user-commands",
    c"user_commands",
    c"vartabs",
    c"vertsplit",
    c"vimscript-1",
    c"virtualedit",
    c"visual",
    c"visualextra",
    c"vreplace",
    c"wildignore",
    c"wildmenu",
    c"windows",
    c"winaltkeys",
    c"writebackup",
    c"xattr",
    c"nvim",
];

/// Case-insensitive equality, deliberately through libc.
///
/// `strncasecmp` folds with the process locale's table, and nvim calls
/// `setlocale(LC_ALL, "")` at startup -- under a Turkish `LC_CTYPE` it
/// really does refuse to fold `I` onto `i`. `eq_ignore_ascii_case` would be
/// a behaviour change, small but real, so this stays as the C wrote it.
/// Comparing `want`'s NUL as well makes it the C's `strcasecmp(..) == 0`.
fn same_name(name: &CStr, want: &CStr) -> bool {
    cstr_ncasecmp(name, want, want.count_bytes() + 1) == 0
}

/// Whether `name` starts with `prefix`, case-insensitively.
fn starts_with(name: &CStr, prefix: &CStr) -> bool {
    cstr_ncasecmp(name, prefix, prefix.count_bytes()) == 0
}

/// The run of ASCII digits `text` starts with, as the C library reads it:
/// the value, saturated at `max` the way `strtoul` (`u64::MAX`) and `atoi`
/// (`i64::MAX`) saturate, and how many bytes it took.
fn leading_number(text: &[u8], max: u64) -> (u64, usize) {
    let len = text.iter().take_while(|b| b.is_ascii_digit()).count();
    let value = text[..len]
        .iter()
        .try_fold(0u64, |n, &d| {
            n.checked_mul(10)?.checked_add(u64::from(d - b'0'))
        })
        .filter(|&n| n <= max)
        .unwrap_or(max);
    (value, len)
}

/// `atoi` on the digits `text` starts with, truncated to an `int` as the
/// C's `(int)strtol(..)` is.
fn atoi_digits(text: &[u8]) -> c_int {
    leading_number(text, i64::MAX as u64).0 as c_int
}

/// `has("patch…")` — the two spellings, `patch-M.m.PPPP` for a Vim version
/// and `patchNNNN` for a bare Vim patch number. `name` begins with `patch`.
fn has_patch(name: &[u8]) -> bool {
    let at = |i: usize| name.get(i).copied().unwrap_or(NUL as u8);
    if at(5) == b'-' && name.len() >= 11 && (b'1'..=b'9').contains(&at(6)) {
        // patch-M.m.PPPP, with exactly one minor digit -- which is
        // what the `end + 2 == '.'` test below insists on.
        let (major, digits) = leading_number(&name[6..], u64::MAX);
        let end = 6 + digits;
        if at(end) == b'.'
            && at(end + 1).is_ascii_digit()
            && at(end + 2) == b'.'
            && at(end + 3).is_ascii_digit()
        {
            let minor = atoi_digits(&name[end + 1..]);
            let version = (major as c_int).wrapping_mul(100).wrapping_add(minor);
            return has_vim_patch(atoi_digits(&name[end + 3..]), version);
        }
        return false;
    }
    if at(5).is_ascii_digit() {
        return has_vim_patch(atoi_digits(&name[5..]), 0);
    }
    false
}

/// The features answered before the list is consulted.
///
/// `Some` means the name was recognised, whatever the answer; `None` sends
/// the caller on to the list and then to the providers.
fn special_feature(name: &CStr) -> Option<bool> {
    if starts_with(name, c"patch") {
        return Some(has_patch(name.to_bytes()));
    }
    // Note the five: the trailing `-` is compared too.
    if starts_with(name, c"nvim-") {
        return Some(has_nvim_version(&name.to_bytes()[5..]));
    }
    Some(match () {
        _ if same_name(name, c"vim_starting") => starting.get() != 0,
        _ if same_name(name, c"ttyin") => stdin_isatty.get(),
        _ if same_name(name, c"ttyout") => stdout_isatty.get(),
        _ if same_name(name, c"multi_byte_encoding") => true,
        _ if same_name(name, c"gui_running") => ui_gui_attached(),
        _ if same_name(name, c"syntax_items") => syntax_present(Win::current()),
        _ if same_name(name, c"wsl") => has_wsl(),
        _ => return None,
    })
}

/// Whether this is a WSL kernel, asked once and remembered.
fn has_wsl() -> bool {
    static ANSWER: GlobalCell<Option<bool>> = GlobalCell::new(None);
    if ANSWER.get().is_none() {
        let mut err = Error::none();
        const PROBE: &CStr = c"return vim.uv.os_uname()['release']:lower():match('microsoft')";
        let o: Object = match nlua_exec_cstr(PROBE, Array::EMPTY, kRetNilBool) {
            Ok(value) => value,
            Err(e) => {
                err = e;
                Object::Nil
            }
        };
        debug_assert!(!err.is_set());
        let yes = o.as_boolean() == Some(true);
        ANSWER.set(Some(yes));
    }
    ANSWER.get() == Some(true)
}

/// `has({feature})`
pub fn f_has(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let name = numbuf.string(&args[0]);
    let known = special_feature(name)
        .or_else(|| FEATURES.iter().any(|f| same_name(name, f)).then_some(true));

    result.write_number(match known {
        Some(answer) => answer,
        None => {
            // The provider probes run vimscript, which sets
            // `v:shell_error`; the caller's value goes back afterwards.
            let saved = get_vim_var_nr(Vv::ShellError);
            let answer = if same_name(name, c"clipboard_working") || same_name(name, c"unnamedplus")
            {
                eval_has_provider(c"clipboard", true)
            } else if same_name(name, c"pythonx") {
                eval_has_provider(c"python3", true)
            } else {
                eval_has_provider(name, true)
            };
            set_vim_var_nr(Vv::ShellError, saved);
            answer
        }
    } as VarNumber);
}

/// `api_info()` — the whole API metadata dict.
pub fn f_api_info(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // `api_metadata` answers a copy, so the conversion may take it.
    *result = TypVal::from(api_metadata());
}

/// `did_filetype()` — whether a FileType autocommand has fired for this
/// buffer since it was last loaded.
pub fn f_did_filetype(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(Buf::current().b_did_filetype as VarNumber);
}

/// `eventhandler()` — whether we are inside a `vgetc()` from an event.
pub fn f_eventhandler(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(vgetc_busy.get() as VarNumber);
}

/// `foreground()` — a no-op; nvim has no window to raise.
pub fn f_foreground(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {}

/// `getfontname()` — always empty; nvim has no font.
pub fn f_getfontname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
}

/// `getpid()`
pub fn f_getpid(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(os_get_pid() as VarNumber);
}

/// `hostname()`
pub fn f_hostname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut hostname = [0u8; 256];
    os_hostname_into(&mut hostname);
    let name = CStr::from_bytes_until_nul(&hostname).unwrap_or(c"");
    result.write_string(Some(ThinCString::from_cstr(name)));
}

/// `menu_get({path} [, {modes}])`
pub fn f_menu_get(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let list = tv_list_alloc_ret(result, kListLenMayKnow as isize);
    // A non-String second argument is not an error: it just leaves the
    // mode set at "all".
    let modes = if args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING) {
        cmd_modes(numbuf.string(&args[1]).to_bytes(), false).0
    } else {
        MENU_ALL_MODES as c_int
    };
    menu_get(numbuf2.string(&args[0]), modes, list);
}

/// `mode([{expr}])` — one character, or the full mode string when `{expr}`
/// is non-zero.
pub fn f_mode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = get_mode();
    if !args.first().is_some_and(non_zero_arg) {
        buf[1] = NUL as c_char;
    }
    result.write_string(Some(c_chars_until_nul(&buf)));
}

/// `state([{what}])` — the letters for whatever is currently in the way of
/// a `:sleep`, filtered by `{what}` if it was given.
pub fn f_state(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut flags = Vec::<u8>::new();
    let include = args.first().map(|arg| numbuf.string(arg));
    let mut add = |c: u8| {
        if include.is_none_or(|include| has_char(include, c as c_int)) {
            flags.push(c);
        }
    };

    if !(stuff_empty() && typeahead().is_empty() && using_script() == 0) {
        add(b'm');
    }
    if op_pending() {
        add(b'o');
    }
    if autocmd_busy.get() {
        add(b'x');
    }
    if ins_compl_active() {
        add(b'a');
    }
    if !get_was_safe_state() {
        add(b'S');
    }
    // One `c` per nested callback, capped at three.
    for _ in 0..get_callback_depth().min(3) {
        add(b'c');
    }
    if msg_scrolled.get() > 0 {
        add(b's');
    }

    // No flag at all left the garray unallocated, so `state()` answered the
    // *null* string rather than an empty one. Keep that.
    result.write_string(if flags.is_empty() {
        None
    } else {
        Some(flags.into())
    });
}

/// `nextnonblank({lnum})` — the first line at or after `{lnum}` that is not
/// blank, or 0.
pub fn f_nextnonblank(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut lnum = arg_lnum(&args[0]);
    let mut lines = Lines::current();
    loop {
        if lnum < 0 || lnum > lines.count() {
            lnum = 0;
            break;
        }
        if !is_blank(lines.line(lnum)) {
            break;
        }
        lnum += 1;
    }
    result.write_number(lnum as VarNumber);
}

/// `prevnonblank({lnum})` — the last line at or before `{lnum}` that is not
/// blank, or 0.
pub fn f_prevnonblank(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut lnum = arg_lnum(&args[0]);
    let mut lines = Lines::current();
    if lnum < 1 || lnum > lines.count() {
        lnum = 0;
    } else {
        while lnum >= 1 && is_blank(lines.line(lnum)) {
            lnum -= 1;
        }
    }
    result.write_number(lnum as VarNumber);
}

/// Whether `line` is nothing but spaces and tabs.
fn is_blank(line: &[u8]) -> bool {
    line.iter().all(|&b| ascii_iswhite(c_int::from(b)))
}

/// `pum_getpos()` — where the popup menu is, or an empty dict.
pub fn f_pum_getpos(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);
    pum_set_event_info(result.dict_mut().expect("the dict just stored"));
}

/// `pumvisible()`
pub fn f_pumvisible(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if pum_visible() {
        result.write_number(1);
    }
}

/// `shiftwidth([{col}])` — the effective 'shiftwidth', which follows
/// 'tabstop' when the option is zero and 'vartabstop' makes it depend on
/// the column.
pub fn f_shiftwidth(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if !args.is_empty() {
        let col = arg_number_chk(&args[0], None) as ColNr;
        // A coercion failure answers 0, which passes; a negative column
        // leaves the 0 already in place.
        if col < 0 {
            return;
        }
        result.write_number(get_sw_value_col(Buf::current(), col, false) as VarNumber);
        return;
    }
    result.write_number(get_sw_value(Buf::current()) as VarNumber);
}

/// `tabpagebuflist([{tabnr}])` — the buffer of every window in the tab, in
/// window order.
pub fn f_tabpagebuflist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let tab = if !args.is_empty() {
        find_tabpage(arg_number(&args[0]) as c_int)
    } else {
        Some(TabPage::current())
    };
    // A bad tab number answers 0, not an empty List. Every live tab page
    // has at least one window, so this is the only way out.
    let Some(tab) = tab else {
        return;
    };
    let list = tv_list_alloc_ret(result, kListLenMayKnow as isize);
    // `windows_in_tab` knows that the current tab's window list lives in
    // `firstwin` rather than in the tab page record, which is only
    // updated on the way out.
    for wp in windows_in_tab(tab) {
        list.push_number(wp.w_buffer.handle() as VarNumber);
    }
}

/// `visualmode([{expr}])` — the last Visual mode, cleared when `{expr}` is
/// non-zero.
pub fn f_visualmode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mode = [Buf::current().b_visual_mode_eval as c_char];
    result.write_string(Some(c_chars_until_nul(&mode)));
    if args.first().is_some_and(non_zero_arg) {
        Buf::current().b_visual_mode_eval = NUL;
    }
}

/// `wildmenumode()`
pub(crate) fn f_wildmenumode(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if wild_menu_showing.get() != 0 || (State.get() & MODE_CMDLINE != 0 && cmdline_pum_active()) {
        result.write_number(1);
    }
}

/// `windowsversion()` — always empty here; kept for scripts that ask.
pub fn f_windowsversion(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(Some(c_chars_until_nul(&windowsVersion)));
}

/// A copy of the C characters in `chars` up to the first NUL, or all of
/// them when there is none.
fn c_chars_until_nul(chars: &[c_char]) -> ThinCString {
    let bytes: Vec<u8> = chars
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    bytes.into()
}

/// `wordcount()`
pub fn f_wordcount(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);
    cursor_pos_info(result.dict_mut());
}
