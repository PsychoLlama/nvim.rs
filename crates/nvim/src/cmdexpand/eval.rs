//! The Vimscript face: `getcompletion()`, `getcompletiontype()`,
//! `cmdcomplete_info()`.
//!
//! [`f_getcompletion`] runs the whole classify-then-expand pipeline against a
//! string instead of the real command line, which is what makes it the
//! completion layer's differential oracle.  All three are rows in the
//! generated eval function table.

#![forbid(unsafe_code)]

use super::*;
use crate::eval::list::{cstr_of, string_tv};
use crate::eval::typval::{
    NumBuf, tv_check_for_string_arg, tv_dict_alloc_ret, tv_get_number_chk, tv_list_alloc,
    tv_list_alloc_ret,
};
use crate::lua::executor::nlua_expand_pat;
use crate::menu::set_context_in_menu_cmd;
use crate::message::{e_invarg, emsg};
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_wic;
use crate::os::cshim::gettext;
use crate::popupmenu::pum_visible;
use crate::runtime::set_context_in_runtime_cmd;
use crate::semsg;
use crate::sign::set_context_in_sign_cmd;
use crate::types::{EvalFuncData, ExpandContext, TypVal, VAR_STRING, VarNumber, ptrdiff_t};
use crate::usercmd::{cmdcomplete_str_to_type, cmdcomplete_type_to_str};
use crate::winlayer::Cc;

/// What `getcompletion()` asks of every expansion: newline-separated so the
/// caller can split it, quiet, and with `~/` restored.
const GETCOMPLETION: WildOpts = WildOpts::SILENT
    .or(WildOpts::USE_NL)
    .or(WildOpts::ADD_SLASH)
    .or(WildOpts::NO_BEEP)
    .or(WildOpts::HOME_REPLACE);

/// `getcompletion()`: expand `{pattern}` as `{type}` and answer the matches.
pub fn f_getcompletion(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut xpc = Expand::new();
    let mut filtered = false;
    let mut options = GETCOMPLETION;

    if tv_check_for_string_arg(args, 1).is_err() {
        return;
    }
    let type_0 = cstr_of(&args[1], &mut numbuf);

    if args.len() > 2 {
        filtered = tv_get_number_chk(&args[2]).unwrap_or(-1) != 0;
    }

    if p_wic() {
        options |= WildOpts::ICASE;
    }

    // For filtered results, 'wildignore' is used.
    if !filtered {
        options |= WildOpts::KEEP_ALL;
    }

    if args[0].v_type() != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    let pattern = cstr_of(&args[0], &mut numbuf2).to_bytes();
    // Where the pattern started before a context moved it on.
    let pattern_start;

    // C's `goto theend`: the "cmdline" type takes the whole classifier and
    // skips the per-type switch entirely.
    if type_0.to_bytes() == b"cmdline" {
        let cmdline_len = as_count(pattern.len());
        set_cmd_context(&mut xpc, pattern, cmdline_len, false);
        pattern_start = xpc.pattern;
        xpc.pattern_len = xpc.pattern_text().len();
        xpc.col = cmdline_len;
    } else {
        xpc.line = owned(pattern);
        xpc.pattern = 0;
        xpc.pattern_len = pattern.len();
        pattern_start = 0;

        xpc.context = cmdcomplete_str_to_type(type_0);
        let arg_after = |prefix: &[u8]| type_0.to_bytes().strip_prefix(prefix).map(owned);
        match xpc.context {
            ExpandContext::Nothing => {
                let arg0 = msg_cstr(type_0);
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
            ExpandContext::UserDefined => {
                // Must be "custom,funcname" pattern.
                let Some(func) = arg_after(b"custom,") else {
                    let arg0 = msg_cstr(type_0);
                    semsg!("E475: Invalid argument: {arg0}");
                    return;
                };
                xpc.arg = Some(func);
            }
            ExpandContext::UserList => {
                // Must be "customlist,funcname" pattern.
                let Some(func) = arg_after(b"customlist,") else {
                    let arg0 = msg_cstr(type_0);
                    semsg!("E475: Invalid argument: {arg0}");
                    return;
                };
                xpc.arg = Some(func);
            }
            // The four generators below move the pattern forward inside
            // the string, so the length has to follow it.
            ExpandContext::Menus => {
                set_context_in_menu_cmd(&mut xpc, c"menu", 0, false);
                xpc.pattern_len -= xpc.pattern - pattern_start;
            }
            ExpandContext::Sign => {
                set_context_in_sign_cmd(&mut xpc, 0);
                xpc.pattern_len -= xpc.pattern - pattern_start;
            }
            ExpandContext::Runtime => {
                set_context_in_runtime_cmd(&mut xpc, 0);
                xpc.pattern_len -= xpc.pattern - pattern_start;
            }
            ExpandContext::ShellCmdLine => {
                let mut context = ExpandContext::ShellCmdLine;
                let text = xpc.line.clone();
                set_context_for_wildcard_arg(
                    None,
                    text.as_cstr(),
                    0,
                    false,
                    &mut xpc,
                    &mut context,
                );
                xpc.pattern_len -= xpc.pattern - pattern_start;
            }
            ExpandContext::FiletypeCmd => filetype_expand_what.set(FiletypeWhat::All),
            _ => {}
        }
    }

    if xpc.context == ExpandContext::Lua {
        xpc.col = as_count(xpc.line_cstr().count_bytes());
        nlua_expand_pat(&mut xpc);
        xpc.pattern_len -= xpc.pattern - pattern_start;
    }

    let pat = if cmdline_fuzzy_completion_supported(&xpc) {
        // When fuzzy matching, don't modify the search string.
        owned(xpc.pattern_span())
    } else {
        addstar(xpc.pattern_span(), xpc.context)
    };

    expand_one(
        &mut xpc,
        Some(pat.as_cstr()),
        None,
        options,
        WildMode::AllKeep,
    );
    let retlist = tv_list_alloc_ret(result, xpc.match_count() as ptrdiff_t);
    for name in xpc.matches() {
        retlist.push(string_tv(name.as_cstr().to_bytes()));
    }
    expand_cleanup(&mut xpc);
}

/// `getcompletiontype()`: the completion type name a command line would use.
pub fn f_getcompletiontype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);

    if tv_check_for_string_arg(args, 0).is_err() {
        return;
    }

    let pat = cstr_of(&args[0], &mut numbuf).to_bytes();
    let mut xpc = Expand::new();

    let cmdline_len = as_count(pat.len());
    set_cmd_context(&mut xpc, pat, cmdline_len, false);
    let name = cmdcomplete_type_to_str(xpc.context, xpc.arg.as_ref().map(XString::as_cstr));
    *result = name.map_or(TypVal::string(None), |name| string_tv(&name));

    expand_cleanup(&mut xpc);
}

/// `cmdcomplete_info()`: the state of the completion in progress.
pub fn f_cmdcomplete_info(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // What the command line's completion holds, copied out: building the
    // answer allocates, which is no place to hold the completion borrowed.
    let state = Cc::current().with_xpc(|xpc| {
        let xpc = xpc?;
        xpc.found_any
            .then(|| (xpc.selected, xpc.matches().to_vec()))
    });

    tv_dict_alloc_ret(result);
    let Some((selected, matches)) = state else {
        return;
    };
    let Some(retdict) = result.dict_mut() else {
        return;
    };

    // Upstream's null pointer -- nothing expanded yet -- is a null entry,
    // not an empty string, so the `None` case is spelled out.
    let orig = cmdline_orig.with(|line| {
        line.as_ref()
            .map_or(TypVal::string(None), |line| string_tv(line))
    });
    let mut ret = retdict.add_tv(b"cmdline_orig", &orig);
    if ret.is_ok() {
        ret = retdict.add_number(b"pum_visible", VarNumber::from(pum_visible()));
    }
    if ret.is_ok() {
        ret = retdict.add_number(b"selected", VarNumber::from(selected));
    }
    if ret.is_ok() {
        let mut li = tv_list_alloc(as_count(matches.len()) as ptrdiff_t);
        for name in &matches {
            li.push(string_tv(name.as_cstr().to_bytes()));
        }
        let _ = retdict.add_list(b"matches", Some(li));
    }
}
