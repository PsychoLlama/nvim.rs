//! What the editor is right now: `has()`, `mode()`, `state()` and the rest
//! of the feature and status queries.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::wrappers::non_zero_arg;
use super::{MENU_ALL_MODES, kRetNilBool};
use crate::api::private::converter::object_to_vim;
use crate::api::private::helpers::api_metadata;
use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::cmdexpand::cmdline_pum_active;
use crate::eval::typval::{
    tv_dict_alloc_ret, tv_get_lnum, tv_get_number, tv_get_number_chk, tv_get_string,
    tv_list_alloc_ret, tv_list_append_number,
};
use crate::eval::vars::{get_vim_var_nr, set_vim_var_nr};
use crate::eval::{eval_has_provider, get_callback_depth};
use crate::garray::{ga_append, ga_init};
use crate::getchar::{stuff_empty, using_script};
use crate::global_cell::GlobalCell;
use crate::indent::{get_sw_value, get_sw_value_col};
use crate::insexpand::ins_compl_active;
use crate::lua::executor::nlua_exec;
use crate::main::{
    State, autocmd_busy, curbuf, curtab, firstwin, msg_scrolled, starting, stdin_isatty,
    stdout_isatty, typebuf, vgetc_busy, wild_menu_showing, windowsVersion,
};
use crate::memline::ml_get;
use crate::memory::xstrdup;
use crate::menu::{get_menu_cmd_modes, menu_get};
use crate::normal::op_pending;
use crate::ops::cursor_pos_info;
use crate::os::cshim::strncasecmp;
use crate::os::env::{os_get_hostname, os_get_pid};
use crate::popupmenu::{pum_set_event_info, pum_visible};
use crate::state::{MODE_CMDLINE, get_mode, get_was_safe_state};
use crate::strings::vim_strchr;
use crate::syntax::syntax_present;
use crate::types::{
    Arena, Array, Error, EvalFuncData, NUL, Object, String_0, VAR_STRING, Vv, colnr_T, garray_T,
    kErrorTypeNone, kListLenMayKnow, kObjectTypeBoolean, tabpage_T, typval_T, uint8_t, varnumber_T,
    win_T,
};
use crate::ui::ui_gui_attached;
use crate::version::{has_nvim_version, has_vim_patch};
use crate::window::find_tabpage;
use ::libc::{atoi, strcasecmp, strlen, strtoul};
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
    // SAFETY: `strncasecmp` stops at the terminator of either operand.
    unsafe {
        strncasecmp(
            name as *mut c_char,
            prefix.as_ptr() as *mut c_char,
            prefix.count_bytes(),
        ) == 0
    }
}

/// `has("patch…")` — the two spellings, `patch-M.m.PPPP` for a Vim version
/// and `patchNNNN` for a bare Vim patch number.
///
/// # Safety
/// `name` is a NUL-terminated string beginning with `patch`.
unsafe fn has_patch(name: *const c_char) -> bool {
    // SAFETY: the caller's obligation puts `name[5]` at or before the
    // terminator, and the length test below covers `name[6]`.
    unsafe {
        if *name.add(5) as u8 == b'-'
            && strlen(name) >= 11
            && (b'1'..=b'9').contains(&(*name.add(6) as u8))
        {
            // patch-M.m.PPPP, with exactly one minor digit -- which is
            // what the `end[2] == '.'` test below insists on.
            let mut end = ptr::null_mut::<c_char>();
            let major = strtoul(name.add(6), &raw mut end, 10) as c_int;
            if *end as u8 == b'.'
                && ascii_isdigit(*end.add(1) as c_int)
                && *end.add(2) as u8 == b'.'
                && ascii_isdigit(*end.add(3) as c_int)
            {
                let minor = atoi(end.add(1));
                return has_vim_patch(atoi(end.add(3)), major * 100 + minor);
            }
            return false;
        }
        if ascii_isdigit(*name.add(5) as c_int) {
            return has_vim_patch(atoi(name.add(5)), 0);
        }
        false
    }
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
    unsafe {
        if starts_with(name, c"patch") {
            return Some(has_patch(name));
        }
        // Note the five: the trailing `-` is compared too.
        if starts_with(name, c"nvim-") {
            return Some(has_nvim_version(name.add(5)));
        }
        Some(match () {
            _ if same_name(name, c"vim_starting") => starting.get() != 0,
            _ if same_name(name, c"ttyin") => stdin_isatty.get(),
            _ if same_name(name, c"ttyout") => stdout_isatty.get(),
            _ if same_name(name, c"multi_byte_encoding") => true,
            _ if same_name(name, c"gui_running") => ui_gui_attached(),
            _ if same_name(name, c"syntax_items") => syntax_present(crate::main::curwin.get()),
            _ if same_name(name, c"wsl") => has_wsl(),
            _ => return None,
        })
    }
}

/// Whether this is a WSL kernel, asked once and remembered.
fn has_wsl() -> bool {
    static ANSWER: GlobalCell<Option<bool>> = GlobalCell::new(None);
    if ANSWER.get().is_none() {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        const PROBE: &str = "return vim.uv.os_uname()['release']:lower():match('microsoft')";
        // SAFETY: `PROBE` outlives the call (it is a `'static`), the
        // argument list is empty, and `err` is a live out-parameter.
        let o: Object = unsafe {
            nlua_exec(
                String_0 {
                    data: PROBE.as_ptr() as *mut c_char,
                    size: PROBE.len(),
                },
                ptr::null(),
                Array {
                    size: 0,
                    capacity: 0,
                    items: ptr::null_mut(),
                },
                kRetNilBool,
                ptr::null_mut::<Arena>(),
                &raw mut err,
            )
        };
        debug_assert!(err.type_0 == kErrorTypeNone);
        // SAFETY: the union member is the one the type tag names.
        let yes = o.type_0 == kObjectTypeBoolean && unsafe { o.data.boolean } as c_int == 1;
        ANSWER.set(Some(yes));
    }
    ANSWER.get() == Some(true)
}

/// `has({feature})`
pub unsafe fn f_has(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `name` is the string an argument owns.
    unsafe {
        let name = tv_get_string(args.ptr(0));
        let known = special_feature(name)
            .or_else(|| FEATURES.iter().any(|f| same_name(name, f)).then_some(true));

        rettv.vval.v_number = match known {
            Some(answer) => answer,
            None => {
                // The provider probes run vimscript, which sets
                // `v:shell_error`; the caller's value goes back afterwards.
                let saved = get_vim_var_nr(Vv::ShellError);
                let answer =
                    if same_name(name, c"clipboard_working") || same_name(name, c"unnamedplus") {
                        eval_has_provider(c"clipboard".as_ptr(), true)
                    } else if same_name(name, c"pythonx") {
                        eval_has_provider(c"python3".as_ptr(), true)
                    } else {
                        eval_has_provider(name, true)
                    };
                set_vim_var_nr(Vv::ShellError, saved);
                answer
            }
        } as varnumber_T;
    }
}

/// `api_info()` — the whole API metadata dict.
pub unsafe fn f_api_info(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value; a null `Error` out-pointer
    // is what the converter's infallible path takes.
    unsafe { object_to_vim(api_metadata(), rettv, ptr::null_mut()) };
}

/// `did_filetype()` — whether a FileType autocommand has fired for this
/// buffer since it was last loaded.
pub unsafe fn f_did_filetype(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `curbuf` is live and `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = (*curbuf.get()).b_did_filetype as varnumber_T };
}

/// `eventhandler()` — whether we are inside a `vgetc()` from an event.
pub unsafe fn f_eventhandler(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = vgetc_busy.get() as varnumber_T };
}

/// `foreground()` — a no-op; nvim has no window to raise.
pub unsafe fn f_foreground(_argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {}

/// `getfontname()` — always empty; nvim has no font.
pub unsafe fn f_getfontname(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();
    }
}

/// `getpid()`
pub unsafe fn f_getpid(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = os_get_pid() as varnumber_T };
}

/// `hostname()`
pub unsafe fn f_hostname(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut hostname = [0 as c_char; 256];
    // SAFETY: `os_get_hostname` writes at most the length it is given,
    // NUL-terminated; `rettv` then owns the duplicate.
    unsafe {
        os_get_hostname(hostname.as_mut_ptr(), hostname.len());
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xstrdup(hostname.as_ptr());
    }
}

/// `menu_get({path} [, {modes}])`
pub unsafe fn f_menu_get(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `rettv` is the cleared return value.
    unsafe {
        let list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        // A non-String second argument is not an error: it just leaves the
        // mode set at "all".
        let modes = if args.ty(1) == VAR_STRING {
            get_menu_cmd_modes(
                tv_get_string(args.ptr(1)),
                false,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        } else {
            MENU_ALL_MODES as c_int
        };
        menu_get(tv_get_string(args.ptr(0)) as *mut c_char, modes, list);
    }
}

/// `mode([{expr}])` — one character, or the full mode string when `{expr}`
/// is non-zero.
pub unsafe fn f_mode(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // `MODE_MAX_LENGTH` in the C.
    let mut buf = [0 as c_char; 4];
    // SAFETY: the frame is live; `get_mode` fills the buffer it is given and
    // `rettv` then owns the duplicate.
    unsafe {
        get_mode(buf.as_mut_ptr());
        if !non_zero_arg(args.ptr(0)) {
            buf[1] = NUL as c_char;
        }
        rettv.vval.v_string = xstrdup(buf.as_ptr());
        rettv.v_type = VAR_STRING;
    }
}

/// `state([{what}])` — the letters for whatever is currently in the way of
/// a `:sleep`, filtered by `{what}` if it was given.
pub unsafe fn f_state(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    // SAFETY: the frame is live; `ga` is a local the appends below own, and
    // `rettv` adopts its allocation at the end. `ga_grow` zero-fills what it
    // adds, so the bytes past the last append terminate the string.
    unsafe {
        ga_init(&raw mut ga, 1, 20);
        let include = if args.has(0) {
            tv_get_string(args.ptr(0))
        } else {
            ptr::null()
        };
        let mut add = |c: u8| {
            if include.is_null() || !vim_strchr(include, c as c_int).is_null() {
                ga_append(&raw mut ga, c as uint8_t);
            }
        };

        if !(stuff_empty() && (*typebuf.ptr()).tb_len == 0 && using_script() == 0) {
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

        rettv.v_type = VAR_STRING;
        rettv.vval.v_string = ga.ga_data as *mut c_char;
    }
}

/// `nextnonblank({lnum})` — the first line at or after `{lnum}` that is not
/// blank, or 0.
pub unsafe fn f_nextnonblank(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `curbuf` is live for the whole call; the
    // loop only reads lines it has range-checked.
    unsafe {
        let mut lnum = tv_get_lnum(args.ptr(0));
        loop {
            if lnum < 0 || lnum > (*curbuf.get()).b_ml.ml_line_count {
                lnum = 0;
                break;
            }
            if *skipwhite(ml_get(lnum)) as c_int != NUL {
                break;
            }
            lnum += 1;
        }
        rettv.vval.v_number = lnum as varnumber_T;
    }
}

/// `prevnonblank({lnum})` — the last line at or before `{lnum}` that is not
/// blank, or 0.
pub unsafe fn f_prevnonblank(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: as `f_nextnonblank`.
    unsafe {
        let mut lnum = tv_get_lnum(args.ptr(0));
        if lnum < 1 || lnum > (*curbuf.get()).b_ml.ml_line_count {
            lnum = 0;
        } else {
            while lnum >= 1 && *skipwhite(ml_get(lnum)) as c_int == NUL {
                lnum -= 1;
            }
        }
        rettv.vval.v_number = lnum as varnumber_T;
    }
}

/// `pum_getpos()` — where the popup menu is, or an empty dict.
pub unsafe fn f_pum_getpos(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe {
        tv_dict_alloc_ret(rettv);
        pum_set_event_info((*rettv).vval.v_dict);
    }
}

/// `pumvisible()`
pub unsafe fn f_pumvisible(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe {
        if pum_visible() {
            (*rettv).vval.v_number = 1;
        }
    }
}

/// `shiftwidth([{col}])` — the effective 'shiftwidth', which follows
/// 'tabstop' when the option is zero and 'vartabstop' makes it depend on
/// the column.
pub unsafe fn f_shiftwidth(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live and `curbuf` is live for the call.
    unsafe {
        if args.has(0) {
            let col = tv_get_number_chk(args.ptr(0), ptr::null_mut()) as colnr_T;
            // A coercion failure answers 0, which passes; a negative column
            // leaves the 0 already in place.
            if col < 0 {
                return;
            }
            rettv.vval.v_number = get_sw_value_col(curbuf.get(), col, false) as varnumber_T;
            return;
        }
        rettv.vval.v_number = get_sw_value(curbuf.get()) as varnumber_T;
    }
}

/// `tabpagebuflist([{tabnr}])` — the buffer of every window in the tab, in
/// window order.
pub unsafe fn f_tabpagebuflist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; the window chain walked below belongs to a
    // tab page that is live for the whole call.
    unsafe {
        let mut wp: *mut win_T = ptr::null_mut();
        if !args.has(0) {
            wp = firstwin.get();
        } else {
            let tp: *mut tabpage_T = find_tabpage(tv_get_number(args.ptr(0)) as c_int);
            if !tp.is_null() {
                // The current tab's window list lives in `firstwin`, not in
                // the tab page record, which is only updated on the way out.
                wp = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
            }
        }
        // A bad tab number answers 0, not an empty List.
        if wp.is_null() {
            return;
        }
        let list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        while !wp.is_null() {
            tv_list_append_number(list, (*(*wp).w_buffer).handle as varnumber_T);
            wp = (*wp).w_next;
        }
    }
}

/// `visualmode([{expr}])` — the last Visual mode, cleared when `{expr}` is
/// non-zero.
pub unsafe fn f_visualmode(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live, `curbuf` is live for the call, and `rettv`
    // owns the duplicate.
    unsafe {
        let mode = [(*curbuf.get()).b_visual_mode_eval as c_char, NUL as c_char];
        rettv.v_type = VAR_STRING;
        rettv.vval.v_string = xstrdup(mode.as_ptr());
        if non_zero_arg(args.ptr(0)) {
            (*curbuf.get()).b_visual_mode_eval = NUL;
        }
    }
}

/// `wildmenumode()`
pub unsafe fn f_wildmenumode(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe {
        if wild_menu_showing.get() != 0 || (State.get() & MODE_CMDLINE != 0 && cmdline_pum_active())
        {
            (*rettv).vval.v_number = 1;
        }
    }
}

/// `windowsversion()` — always empty here; kept for scripts that ask.
pub unsafe fn f_windowsversion(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `windowsVersion` is a live NUL-terminated buffer and `rettv`
    // owns the duplicate.
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xstrdup(windowsVersion.ptr() as *const c_char);
    }
}

/// `wordcount()`
pub unsafe fn f_wordcount(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe {
        tv_dict_alloc_ret(rettv);
        cursor_pos_info((*rettv).vval.v_dict);
    }
}
