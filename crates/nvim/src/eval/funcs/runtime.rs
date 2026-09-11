//! What the editor is right now: `has()`, `mode()`, `state()` and the rest
//! of the feature and status queries.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{
    arg_lnum, arg_number, arg_number_chk, arg_string, list_alloc_ret, non_zero_arg,
};
use super::{MENU_ALL_MODES, kRetNilBool};
use crate::api::private::helpers::api_metadata;
use crate::ascii::ascii_isdigit;
use crate::autocmd::state::autocmd_busy;
use crate::charset::skipwhite;
use crate::cmdexpand::cmdline_pum_active;
use crate::cmdexpand::state::wild_menu_showing;
use crate::cstr;
use crate::eval::typval::{NumBuf, tv_dict_alloc_ret, tv_list_append_number};
use crate::eval::vars::{get_vim_var_nr, set_vim_var_nr};
use crate::eval::{eval_has_provider, get_callback_depth};
use crate::getchar::state::vgetc_busy;
use crate::getchar::{stuff_empty, typeahead, using_script};
use crate::global_cell::GlobalCell;
use crate::indent::{get_sw_value, get_sw_value_col};
use crate::insexpand::ins_compl_active;
use crate::lua::executor::nlua_exec;
use crate::memline::ml_get;
use crate::memory::handoff::owned_cstr;
use crate::memory::xstrdup;
use crate::menu::{get_menu_cmd_modes, menu_get};
use crate::message::state::msg_scrolled;
use crate::normal::op_pending;
use crate::ops::cursor_pos_info;
use crate::os::cshim::strncasecmp;
use crate::os::env::{os_get_hostname, os_get_pid};
use crate::os::state::windowsVersion;
use crate::popupmenu::{pum_set_event_info, pum_visible};
use crate::startup::{starting, stdin_isatty, stdout_isatty};
use crate::state::mode::State;
use crate::state::{MODE_CMDLINE, get_mode, get_was_safe_state};
use crate::strings::has_char;
use crate::syntax::syntax_present;
use crate::types::{
    Arena, Array, ColNr, Error, EvalFuncData, NUL, Object, String_0, TypVal, VAR_STRING, VarNumber,
    Vv, kListLenMayKnow,
};
use crate::ui::ui_gui_attached;
use crate::version::{has_nvim_version, has_vim_patch};
use crate::window::find_tabpage;
use crate::winlayer::{Buf, Win};
use crate::winlayer::{TabPage, windows_in_tab};
use ::libc::{atoi, strcasecmp, strtoul};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

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

/// Case-insensitive comparison, deliberately through libc.
///
/// `strcasecmp` folds with the process locale's table, and nvim calls
/// `setlocale(LC_ALL, "")` at startup -- under a Turkish `LC_CTYPE` it
/// really does refuse to fold `I` onto `i`. `eq_ignore_ascii_case` would be
/// a behaviour change, small but real, so this stays as the C wrote it.
///
/// # Safety
/// `name` is a NUL-terminated string.
unsafe fn same_name(name: *const c_char, want: &CStr) -> bool {
    // SAFETY: both arguments are NUL-terminated; `strcasecmp` reads no
    // further.
    unsafe { strcasecmp(name as *mut c_char, want.as_ptr() as *mut c_char) == 0 }
}

/// Whether `name` starts with `prefix`, case-insensitively.
///
/// # Safety
/// `name` is a NUL-terminated string at least as long as it claims.
unsafe fn starts_with(name: *const c_char, prefix: &CStr) -> bool {
    let (a, b) = (name as *mut c_char, prefix.as_ptr() as *mut c_char);
    // SAFETY: `strncasecmp` stops at the terminator of either operand.
    unsafe { strncasecmp(a, b, prefix.count_bytes()) == 0 }
}

/// `has("patch…")` — the two spellings, `patch-M.m.PPPP` for a Vim version
/// and `patchNNNN` for a bare Vim patch number.
///
/// # Safety
/// `name` is a NUL-terminated string beginning with `patch`.
unsafe fn has_patch(name: *const c_char) -> bool {
    // SAFETY: the caller's obligation puts `name[5]` at or before the
    // terminator, and the length test below covers `name[6]`.
    if unsafe { *name.add(5) } as u8 == b'-'
        && unsafe { cstr::bytes_at(name) }.len() >= 11
        && (b'1'..=b'9').contains(&(unsafe { *name.add(6) } as u8))
    {
        // patch-M.m.PPPP, with exactly one minor digit -- which is
        // what the `end[2] == '.'` test below insists on.
        let mut end = ptr::null_mut::<c_char>();
        let major = unsafe { strtoul(name.add(6), &raw mut end, 10) } as c_int;
        if unsafe { *end } as u8 == b'.'
            && ascii_isdigit(unsafe { *end.add(1) } as c_int)
            && unsafe { *end.add(2) } as u8 == b'.'
            && ascii_isdigit(unsafe { *end.add(3) } as c_int)
        {
            let minor = unsafe { atoi(end.add(1)) };
            return has_vim_patch(unsafe { atoi(end.add(3)) }, major * 100 + minor);
        }
        return false;
    }
    if ascii_isdigit(unsafe { *name.add(5) } as c_int) {
        return has_vim_patch(unsafe { atoi(name.add(5)) }, 0);
    }
    false
}

/// The features answered before the list is consulted.
///
/// `Some` means the name was recognised, whatever the answer; `None` sends
/// the caller on to the list and then to the providers.
///
/// # Safety
/// `name` is a NUL-terminated string.
unsafe fn special_feature(name: *const c_char) -> Option<bool> {
    // SAFETY: the caller's obligation.
    if unsafe { starts_with(name, c"patch") } {
        return Some(unsafe { has_patch(name) });
    }
    // Note the five: the trailing `-` is compared too.
    if unsafe { starts_with(name, c"nvim-") } {
        return Some(unsafe { has_nvim_version(name.add(5)) });
    }
    Some(match () {
        _ if unsafe { same_name(name, c"vim_starting") } => starting.get() != 0,
        _ if unsafe { same_name(name, c"ttyin") } => stdin_isatty.get(),
        _ if unsafe { same_name(name, c"ttyout") } => stdout_isatty.get(),
        _ if unsafe { same_name(name, c"multi_byte_encoding") } => true,
        _ if unsafe { same_name(name, c"gui_running") } => ui_gui_attached(),
        _ if unsafe { same_name(name, c"syntax_items") } => syntax_present(Win::current()),
        _ if unsafe { same_name(name, c"wsl") } => has_wsl(),
        _ => return None,
    })
}

/// Whether this is a WSL kernel, asked once and remembered.
fn has_wsl() -> bool {
    static ANSWER: GlobalCell<Option<bool>> = GlobalCell::new(None);
    if ANSWER.get().is_none() {
        let mut err = Error::none();
        const PROBE: &str = "return vim.uv.os_uname()['release']:lower():match('microsoft')";
        // SAFETY throughout: `PROBE` outlives the call (it is a `'static`), the
        // argument list is empty, and `err` is a live out-parameter.
        let code = String_0::from(PROBE);
        let no_args = Array::EMPTY;
        let arena = ptr::null_mut::<Arena>();
        let o: Object = match unsafe { nlua_exec(&code, ptr::null(), no_args, kRetNilBool, arena) }
        {
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
    // SAFETY throughout: the frame is live and `name` is the string an argument owns.
    let name = arg_string(&mut numbuf, &args[0]);
    let known = unsafe { special_feature(name) }.or_else(|| {
        FEATURES
            .iter()
            .any(|f| unsafe { same_name(name, f) })
            .then_some(true)
    });

    result.write_number(match known {
        Some(answer) => answer,
        None => {
            // The provider probes run vimscript, which sets
            // `v:shell_error`; the caller's value goes back afterwards.
            let saved = get_vim_var_nr(Vv::ShellError);
            let answer = if unsafe { same_name(name, c"clipboard_working") }
                || unsafe { same_name(name, c"unnamedplus") }
            {
                unsafe { eval_has_provider(c"clipboard".as_ptr(), true) }
            } else if unsafe { same_name(name, c"pythonx") } {
                unsafe { eval_has_provider(c"python3".as_ptr(), true) }
            } else {
                unsafe { eval_has_provider(name, true) }
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
    // SAFETY: `curbuf` is live and `result` is the cleared return value.
    result.write_number(Buf::current().b_did_filetype as VarNumber);
}

/// `eventhandler()` — whether we are inside a `vgetc()` from an event.
pub fn f_eventhandler(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value.
    result.write_number(vgetc_busy.get() as VarNumber);
}

/// `foreground()` — a no-op; nvim has no window to raise.
pub fn f_foreground(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {}

/// `getfontname()` — always empty; nvim has no font.
pub fn f_getfontname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value.
    result.write_string(ptr::null_mut());
}

/// `getpid()`
pub fn f_getpid(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value.
    result.write_number(os_get_pid() as VarNumber);
}

/// `hostname()`
pub fn f_hostname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut hostname = [0 as c_char; 256];
    // SAFETY: `os_get_hostname` writes at most the length it is given,
    // NUL-terminated; `result` then owns the duplicate.
    unsafe { os_get_hostname(hostname.as_mut_ptr(), hostname.len()) };
    unsafe { (*result).write_string(xstrdup(hostname.as_ptr())) };
}

/// `menu_get({path} [, {modes}])`
pub fn f_menu_get(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY throughout: the frame is live and `result` is the cleared return value.
    let list = list_alloc_ret(result, kListLenMayKnow as isize);
    // A non-String second argument is not an error: it just leaves the
    // mode set at "all".
    let modes = if args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING) {
        let which = arg_string(&mut numbuf, &args[1]);
        let noremap = ptr::null_mut();
        let unmenu = ptr::null_mut();
        // SAFETY: `which` is the NUL-terminated argument.
        unsafe { get_menu_cmd_modes(which, false, noremap, unmenu) }
    } else {
        MENU_ALL_MODES as c_int
    };
    let path = arg_string(&mut numbuf2, &args[0]) as *mut c_char;
    // SAFETY: `path` is the NUL-terminated argument and `list` the list
    // allocated into `result`.
    unsafe { menu_get(path, modes, list) };
}

/// `mode([{expr}])` — one character, or the full mode string when `{expr}`
/// is non-zero.
pub fn f_mode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = get_mode();
    if !args.first().is_some_and(non_zero_arg) {
        buf[1] = NUL as c_char;
    }
    result.write_string(unsafe { xstrdup(buf.as_ptr()) });
}

/// `state([{what}])` — the letters for whatever is currently in the way of
/// a `:sleep`, filtered by `{what}` if it was given.
pub fn f_state(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY (this body): the frame is live, and `result` adopts the buffer
    // at the end.
    let mut flags = Vec::<u8>::new();
    let include = if !args.is_empty() {
        arg_string(&mut numbuf, &args[0])
    } else {
        ptr::null()
    };
    let mut add = |c: u8| {
        if include.is_null() || has_char(unsafe { cstr::at(include) }, c as c_int) {
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
        ptr::null_mut()
    } else {
        owned_cstr(flags)
    });
}

/// `nextnonblank({lnum})` — the first line at or after `{lnum}` that is not
/// blank, or 0.
pub fn f_nextnonblank(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the frame is live and `curbuf` is live for the whole call; the
    // loop only reads lines it has range-checked.
    let mut lnum = arg_lnum(&args[0]);
    loop {
        if lnum < 0 || lnum > Buf::current().b_ml.ml_line_count {
            lnum = 0;
            break;
        }
        if unsafe { *skipwhite(ml_get(lnum)) } as c_int != NUL {
            break;
        }
        lnum += 1;
    }
    result.write_number(lnum as VarNumber);
}

/// `prevnonblank({lnum})` — the last line at or before `{lnum}` that is not
/// blank, or 0.
pub fn f_prevnonblank(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: as `f_nextnonblank`.
    let mut lnum = arg_lnum(&args[0]);
    if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count {
        lnum = 0;
    } else {
        while lnum >= 1 && unsafe { *skipwhite(ml_get(lnum)) } as c_int == NUL {
            lnum -= 1;
        }
    }
    result.write_number(lnum as VarNumber);
}

/// `pum_getpos()` — where the popup menu is, or an empty dict.
pub fn f_pum_getpos(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);
    // SAFETY: the dictionary just allocated into the return value.
    unsafe { pum_set_event_info((*result).dict_or_null()) };
}

/// `pumvisible()`
pub fn f_pumvisible(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `result` is the cleared return value.
    if pum_visible() {
        result.write_number(1);
    }
}

/// `shiftwidth([{col}])` — the effective 'shiftwidth', which follows
/// 'tabstop' when the option is zero and 'vartabstop' makes it depend on
/// the column.
pub fn f_shiftwidth(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    // SAFETY throughout: the frame is live and `curbuf` is live for the call.
    if !args.is_empty() {
        let col = arg_number_chk(&args[0], None) as ColNr;
        // A coercion failure answers 0, which passes; a negative column
        // leaves the 0 already in place.
        if col < 0 {
            return;
        }
        result.write_number(unsafe { get_sw_value_col(Buf::current(), col, false) } as VarNumber);
        return;
    }
    result.write_number(unsafe { get_sw_value(Buf::current()) } as VarNumber);
}

/// `tabpagebuflist([{tabnr}])` — the buffer of every window in the tab, in
/// window order.
pub fn f_tabpagebuflist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the frame is live; the window chain walked below belongs to a
    // tab page that is live for the whole call.
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
    let list = list_alloc_ret(result, kListLenMayKnow as isize);
    // `windows_in_tab` knows that the current tab's window list lives in
    // `firstwin` rather than in the tab page record, which is only
    // updated on the way out.
    for wp in windows_in_tab(tab) {
        unsafe { tv_list_append_number(list, (*wp.w_buffer).handle as VarNumber) };
    }
}

/// `visualmode([{expr}])` — the last Visual mode, cleared when `{expr}` is
/// non-zero.
pub fn f_visualmode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mode = [Buf::current().b_visual_mode_eval as c_char, NUL as c_char];
    result.write_string(unsafe { xstrdup(mode.as_ptr()) });
    if args.first().is_some_and(non_zero_arg) {
        Buf::current().b_visual_mode_eval = NUL;
    }
}

/// `wildmenumode()`
pub fn f_wildmenumode(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `result` is the cleared return value.
    if wild_menu_showing.get() != 0 || (State.get() & MODE_CMDLINE != 0 && cmdline_pum_active()) {
        result.write_number(1);
    }
}

/// `windowsversion()` — always empty here; kept for scripts that ask.
pub fn f_windowsversion(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `windowsVersion` is a live NUL-terminated buffer and `result`
    // owns the duplicate.
    unsafe { (*result).write_string(xstrdup(windowsVersion.as_ptr())) };
}

/// `wordcount()`
pub fn f_wordcount(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);
    // SAFETY: the dictionary just allocated into the return value.
    unsafe { cursor_pos_info((*result).dict_or_null()) };
}
