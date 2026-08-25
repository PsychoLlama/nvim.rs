//! Per-context match generators for the contexts with no module of their own.
//!
//! [`expand_other`] is the table of `(context, generator)` pairs that
//! [`super::fromcontext::expand_from_context`] dispatches through, plus the
//! generators small enough to live next to it — `:breakadd`, `:scriptnames`,
//! `:retab`, `:messages`, `:mapclear`, `:filetype`, `:checkhealth` and the
//! LSP list.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::path::ExpandFlags;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::types::{
    ArrayBuf, BackslashEscape, ExpandContext, FAIL, MAXPATHL, OK, kErrorTypeNone, static_cstring,
};

/// Expand a file or directory pattern.
///
/// For `":set path="` and `":set tags="` the escaped spaces have to be
/// un-escaped first, which is what `xp_backslash` records — and that has to
/// happen on a copy, because the caller still owns `pat`.
/// A context that wants directories and not plain files.
const fn dirs_only(flags: ExpandFlags) -> ExpandFlags {
    flags.without(ExpandFlags::FILE).or(ExpandFlags::DIR)
}

pub(crate) unsafe fn expand_files_and_dirs(
    xp: *mut expand_T,
    pat: *mut c_char,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    flags: ExpandFlags,
    options: WildOpts,
) -> c_int {
    unsafe {
        let mut pat = pat;
        let mut flags = flags;
        let free_pat = (*xp).xp_backslash != BackslashEscape::NONE;
        if free_pat {
            // Halve the backslashes of an escaped space (or comma).
            let pat_len = strlen(pat);
            pat = xstrnsave(pat, pat_len);
            let mut pat_end = pat.add(pat_len);
            let mut p = pat;
            while *p != 0 {
                if *p == b'\\' as c_char {
                    // How many bytes of escaping to drop, if any.  Each arm
                    // is a distinct `xp_backslash` mode; upstream's
                    // BACKSLASH_IN_FILENAME arm of the comma case is not
                    // compiled on any platform this port builds for.
                    let drop = if (*xp).xp_backslash.has(BackslashEscape::THREE)
                        && *p.add(1) == b'\\' as c_char
                        && *p.add(2) == b'\\' as c_char
                        && *p.add(3) == b' ' as c_char
                    {
                        3
                    } else if (*xp).xp_backslash.has(BackslashEscape::ONE)
                        && *p.add(1) == b' ' as c_char
                    {
                        1
                    } else if (*xp).xp_backslash.has(BackslashEscape::COMMA)
                        && *p.add(1) == b'\\' as c_char
                        && *p.add(2) == b',' as c_char
                    {
                        2
                    } else {
                        0
                    };
                    if drop > 0 {
                        let from = p.add(drop);
                        // +1 for the NUL.
                        ptr::copy(from, p, pat_end.offset_from(from) as usize + 1);
                        pat_end = pat_end.sub(drop);
                    }
                }
                p = p.add(1);
            }
        }

        let ret = if (*xp).xp_context == ExpandContext::Findfunc {
            expand_findfunc(pat, matches, numMatches)
        } else {
            flags = match (*xp).xp_context {
                ExpandContext::Files => flags | ExpandFlags::FILE,
                ExpandContext::FilesInPath => flags | ExpandFlags::FILE | ExpandFlags::PATH,
                ExpandContext::DirsInCdpath => dirs_only(flags) | ExpandFlags::CDPATH,
                _ => dirs_only(flags),
            };
            if options.has(WildOpts::ICASE) {
                flags |= ExpandFlags::ICASE;
            }
            // Expand wildcards, supporting %:h and the like.
            expand_wildcards_eval(&raw mut pat, numMatches, matches, flags)
        };

        if free_pat {
            xfree(pat as *mut c_void);
        }
        ret
    }
}

/// Answer `list[idx]` as a C string, or NULL when `idx` is out of range.
///
/// Every generator below is called with rising indices until it answers NULL,
/// so "past the end" is how the loop terminates.
fn nth_option(list: &[&'static CStr], idx: c_int) -> *mut c_char {
    match usize::try_from(idx).ok().and_then(|i| list.get(i)) {
        Some(text) => text.as_ptr().cast_mut(),
        None => ptr::null_mut(),
    }
}

/// The possible arguments of the `":filetype {plugin,indent}"` command.
///
/// Which of them apply depends on how much of the command has been typed,
/// which `set_context_in_filetype_cmd` recorded in `filetype_expand_what`.
pub(crate) fn get_filetypecmd_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    nth_option(
        match filetype_expand_what.get() {
            FiletypeWhat::All => &[c"indent", c"plugin", c"on", c"off"],
            FiletypeWhat::Plugin => &[c"plugin", c"on", c"off"],
            FiletypeWhat::Indent => &[c"indent", c"on", c"off"],
            FiletypeWhat::OnOff => &[c"on", c"off"],
        },
        idx,
    )
}

/// The possible arguments of `":breakadd"`, `":breakdel"` and `":profdel"`.
///
/// The three share the tail of one list: `:breakadd` takes all four,
/// `:breakdel` everything but "expr", and `:profdel` only the two that name
/// something already being profiled.
pub(crate) fn get_breakadd_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    const OPTS: [&CStr; 4] = [c"expr", c"file", c"func", c"here"];
    nth_option(
        match breakpt_expand_what.get() {
            EXP_BREAKPT_ADD => &OPTS,
            EXP_BREAKPT_DEL => &OPTS[1..4],
            _ => &OPTS[1..3],
        },
        idx,
    )
}

/// The sourced scripts, for `":scriptnames"`.
///
/// Answers a pointer into the shared `NameBuff`, so the caller must copy it
/// before asking for the next one — which `expand_generic` does.
pub(crate) unsafe fn get_scriptnames_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    unsafe {
        let sid = idx + 1;
        if !script_id_valid(sid) {
            return ptr::null_mut();
        }
        let si = script_item(sid);
        home_replace(
            ptr::null::<buf_T>(),
            (*si).sn_name,
            NameBuff.ptr() as *mut c_char,
            MAXPATHL as size_t,
            true,
        );
        NameBuff.ptr() as *mut c_char
    }
}

/// The possible arguments of the `":retab {-indentonly}"` option.
pub(crate) fn get_retab_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    nth_option(&[c"-indentonly"], idx)
}

/// The possible arguments of the `":messages {clear}"` command.
pub(crate) fn get_messages_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    nth_option(&[c"clear"], idx)
}

/// The possible arguments of the `":mapclear"` command.
pub(crate) fn get_mapclear_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    nth_option(&[c"<buffer>"], idx)
}

/// The `idx`th entry of a cached Lua answer, when it is a string.
///
/// Both Lua-backed generators cache one [`Object`] across the whole
/// completion and index into it per call; this is the indexing half.
unsafe fn nth_lua_string(names: &GlobalCell<Object>, idx: c_int) -> *mut c_char {
    unsafe {
        let names = names.get();
        if names.type_0 != kObjectTypeArray || idx < 0 || idx >= names.data.array.size as c_int {
            return ptr::null_mut();
        }
        let item = &*names.data.array.items.add(idx as usize);
        if item.type_0 != kObjectTypeString {
            return ptr::null_mut();
        }
        item.data.string.data()
    }
}

/// Replace the cached answer with a fresh one, dropping the old.
unsafe fn cache_lua_answer(names: &GlobalCell<Object>, script: &'static CStr, args: Array) {
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let res = nlua_exec(
            static_cstring(script),
            ptr::null(),
            args,
            kRetObject,
            ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        api_free_object(names.get());
        names.set(res);
    }
}

/// Completion for `:checkhealth`: the available healthcheck names.
///
/// Asked of Lua once per command line — `get_cmdline_last_prompt_id` changes
/// when a new one is opened — and cached for the rest of it.
pub(crate) unsafe fn get_healthcheck_names(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    unsafe {
        static names: GlobalCell<Object> = GlobalCell::new(Object::NIL);
        static last_gen: GlobalCell<c_uint> = GlobalCell::new(0);
        if last_gen.get() != get_cmdline_last_prompt_id() || last_gen.get() == 0 {
            cache_lua_answer(&names, c"return vim.health._complete()", ARRAY_DICT_INIT);
            last_gen.set(get_cmdline_last_prompt_id());
        }
        nth_lua_string(&names, idx)
    }
}

/// Completion for `:lsp`.
///
/// Unlike `:checkhealth` the answer depends on the whole command line, so the
/// cache is keyed on that as well as on the prompt id.
pub(crate) unsafe fn get_lsp_arg(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    unsafe {
        static names: GlobalCell<Object> = GlobalCell::new(Object::NIL);
        static last_xp_line: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
        static last_gen: GlobalCell<c_uint> = GlobalCell::new(0);
        if last_xp_line.get().is_null()
            || strcmp(last_xp_line.get(), (*xp).xp_line) != 0
            || last_gen.get() != get_cmdline_last_prompt_id()
        {
            xfree(last_xp_line.get() as *mut c_void);
            last_xp_line.set(xstrdup((*xp).xp_line));
            // The current command line, as the Lua function's one argument.
            let mut args = ArrayBuf::<1>::new();
            args.push(Object::string(cstr_as_string((*xp).xp_line)));
            cache_lua_answer(
                &names,
                c"return require'vim._core.ex_cmd'.lsp_complete(...)",
                args.array(),
            );
            last_gen.set(get_cmdline_last_prompt_id());
        }
        nth_lua_string(&names, idx)
    }
}

/// `(context, generator, match case-insensitively, escape the matches)`.
///
/// The contexts whose matches come from walking a list one item at a time.
/// Everything else has a generator of its own in
/// [`super::fromcontext::expand_from_context`].
const GENERATORS: [(ExpandContext, ItemGetter, bool, bool); 33] = [
    (ExpandContext::Commands, get_command_name, false, true),
    (ExpandContext::FiletypeCmd, get_filetypecmd_arg, true, true),
    (ExpandContext::Mapclear, get_mapclear_arg, true, true),
    (ExpandContext::Messages, get_messages_arg, true, true),
    (ExpandContext::History, get_history_arg, true, true),
    (ExpandContext::UserCommands, get_user_commands, false, true),
    (
        ExpandContext::UserAddrType,
        get_user_cmd_addr_type,
        false,
        true,
    ),
    (ExpandContext::UserCmdFlags, get_user_cmd_flags, false, true),
    (ExpandContext::UserNargs, get_user_cmd_nargs, false, true),
    (
        ExpandContext::UserComplete,
        get_user_cmd_complete,
        false,
        true,
    ),
    (ExpandContext::UserVars, get_user_var_name, false, true),
    (ExpandContext::Functions, get_function_name, false, true),
    (ExpandContext::UserFunc, get_user_func_name, false, true),
    (ExpandContext::Expression, get_expr_name, false, true),
    (ExpandContext::Menus, get_menu_name, false, true),
    (ExpandContext::Menunames, get_menu_names, false, true),
    (ExpandContext::Syntax, get_syntax_name, true, true),
    (ExpandContext::Syntime, get_syntime_arg, true, true),
    (ExpandContext::Highlight, get_highlight_name, true, false),
    (ExpandContext::Events, expand_get_event_name, true, false),
    (ExpandContext::Augroup, expand_get_augroup_name, true, false),
    (ExpandContext::Sign, get_sign_name, true, true),
    (ExpandContext::Profile, get_profile_name, true, true),
    (ExpandContext::Language, get_lang_arg, true, false),
    (ExpandContext::Locales, get_locales, true, false),
    (ExpandContext::EnvVars, get_env_name, true, true),
    (ExpandContext::User, get_users, true, false),
    (ExpandContext::Arglist, get_arglist_name, true, false),
    (ExpandContext::Breakpoint, get_breakadd_arg, true, true),
    (ExpandContext::Scriptnames, get_scriptnames_arg, true, false),
    (ExpandContext::Retab, get_retab_arg, true, true),
    (
        ExpandContext::Checkhealth,
        get_healthcheck_names,
        true,
        false,
    ),
    (ExpandContext::Lsp, get_lsp_arg, true, false),
];

/// Do the expansion based on `xp->xp_context` and `rmp`.
///
/// Answers `FAIL` for a context that is not in the table, which is how
/// [`super::fromcontext::expand_from_context`] reports "nothing to complete".
pub(crate) unsafe fn expand_other(
    pat: *mut c_char,
    xp: *mut expand_T,
    rmp: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> c_int {
    unsafe {
        // Find the context in the table and call expand_generic() with the
        // right function to do the expansion.
        let Some(&(_, func, ic, escaped)) = GENERATORS
            .iter()
            .find(|&&(context, ..)| context == (*xp).xp_context)
        else {
            return FAIL;
        };
        if ic {
            (*rmp).rm_ic = true;
        }
        expand_generic(pat, xp, rmp, matches, numMatches, Some(func), escaped);
        OK
    }
}
