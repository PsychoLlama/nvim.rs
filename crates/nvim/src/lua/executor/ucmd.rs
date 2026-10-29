//! A user command whose implementation is a Lua function.
//!
//! [`nlua_do_ucmd`] builds the command's argument table -- name, args,
//! fargs, bang, line1/line2, range, count, reg, mods and the parsed `smods`
//! -- and calls the registered `LuaRef` with it.  [`nlua_set_sctx`] is what
//! makes an error inside that function report the Lua file and line it was
//! defined at rather than the command's.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::{nlua_error, nlua_pcall, until_nul, with_active_stack, with_main_stack};
use crate::ex_docmd::cmdmod_report;
use crate::ex_docmd::state::cmdmod;
use crate::ex_getln::{cmdpreview_get_bufnr, cmdpreview_get_ns};
use crate::lua::stack::{At, Stack};
use crate::option::vars::p_verbose;
use crate::path::fix_fname;
use crate::runtime::{find_script_by_name, new_script_item, script_is_lua, with_script_item};
use crate::types::{CmdModFlags, ExArg, ExArgt, Handle, OptInt, ScriptCtx, ScriptId, lua_Integer};
use crate::usercmd::{LuaCommand, uc_mods, uc_split_args_iter};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};

/// The `smods` flags that are just a bit and a name.
const MOD_FLAGS: [(CmdModFlags, &CStr); 9] = [
    (CmdModFlags::BROWSE, c"browse"),
    (CmdModFlags::CONFIRM, c"confirm"),
    (CmdModFlags::HIDE, c"hide"),
    (CmdModFlags::KEEPALT, c"keepalt"),
    (CmdModFlags::KEEPJUMPS, c"keepjumps"),
    (CmdModFlags::KEEPMARKS, c"keepmarks"),
    (CmdModFlags::KEEPPATTERNS, c"keeppatterns"),
    (CmdModFlags::LOCKMARKS, c"lockmarks"),
    (CmdModFlags::NOSWAPFILE, c"noswapfile"),
];

/// Point `current` at the Lua source position the running function was
/// defined at, so `:verbose` and an error message name the `.lua` file.
///
/// Only the *innermost Lua* frame counts: C frames and chunks that came from
/// a string rather than a file are skipped. Nothing happens below
/// `'verbose'` 1, because walking the Lua stack is not free.
pub fn nlua_set_sctx(current: &mut ScriptCtx) {
    if !script_is_lua(current.sc_sid) {
        return;
    }
    current.sc_lnum = 0;
    if p_verbose() <= 0 as OptInt {
        return;
    }
    let found = with_active_stack(|stack| {
        let mut level: c_int = 1;
        loop {
            let frame = stack.frame(level)?;
            if !frame.is_c
                && let Some(path) = frame.source.strip_prefix(b"@")
            {
                return Some((path.to_vec(), frame.current_line));
            }
            level += 1;
        }
    });
    let Some(Some((source, line))) = found else {
        return;
    };

    let source = CString::new(source).expect("a chunk name holds no NUL");
    let source_path = fix_fname(&source);
    let mut sid = find_script_by_name(source_path.as_cstr());
    if sid <= 0 {
        sid = new_script_item(Some(source_path));
        with_script_item(sid, |si| si.sn_lua = true);
    }
    current.sc_sid = sid as ScriptId;
    current.sc_seq = -1;
    current.sc_lnum = line;
}

/// Store the value on top under `name` in the table below it.
fn set(stack: &mut Stack, name: &CStr) {
    stack.set_field(At::from_top(2), name);
}

/// Call a Lua-implemented user command, or its `preview` half.
///
/// The preview callback takes two extra arguments — the preview namespace
/// and buffer — and answers how much of a preview it produced (0 to 2);
/// anything else is 0.
///
/// `cmd` is what the run needs of the command, copied out of its table:
/// the callback may redefine or delete the command while it runs.
pub(crate) fn nlua_do_ucmd(cmd: &LuaCommand, excmd: &ExArg, preview: bool) -> c_int {
    with_main_stack(|stack| {
        stack.push_reference(cmd.luaref);

        // The one argument is a table describing the invocation.
        stack.new_table(0, 0);

        stack.push_bytes(cmd.name.to_bytes());
        set(stack, c"name");
        stack.push_boolean(excmd.forceit);
        set(stack, c"bang");
        stack.push_integer(excmd.line1 as lua_Integer);
        set(stack, c"line1");
        stack.push_integer(excmd.line2 as lua_Integer);
        set(stack, c"line2");

        // `excmd` is the raw argument text; `fargs` the split one. The raw
        // string is pushed once and stored twice.
        stack.new_table(0, 0);
        stack.push_bytes(excmd.line.cstr_from(excmd.line.arg).to_bytes());
        stack.push_value(At::TOP);
        stack.set_field(At::from_top(4), c"args");
        if cmd.argt.has(ExArgt::NOSPC) {
            // At most one argument: `fargs` is the whole of it, or empty.
            if cmd.argt.has(ExArgt::NEEDARG) || !excmd.line.arg().is_empty() {
                stack.raw_set_index(At::from_top(2), 1);
            } else {
                stack.pop(1);
            }
        } else if excmd.line.args.is_empty() {
            stack.pop(1);
            // Not pre-split (`:command` rather than `nvim_cmd`): split here,
            // honouring backslash escapes.
            let mut end = 0;
            let mut i: c_int = 1;
            let mut word = Vec::with_capacity(excmd.line.arg().len());
            let mut done = false;
            while !done {
                done = uc_split_args_iter(excmd.line.arg(), &mut end, &mut word);
                if !word.is_empty() {
                    stack.push_bytes(&word);
                    stack.raw_set_index(At::from_top(2), i);
                    i += 1;
                }
            }
        } else {
            stack.pop(1);
            for (i, &(at, len)) in (1..).zip(excmd.line.args.iter()) {
                stack.push_bytes(&excmd.line.tail(at)[..len]);
                stack.raw_set_index(At::from_top(2), i);
            }
        }
        set(stack, c"fargs");

        // The register as a one-letter string; none is the empty string.
        let reg = u8::try_from(excmd.regname).unwrap_or_default();
        let reg = [reg];
        stack.push_bytes(until_nul(&reg));
        set(stack, c"reg");
        stack.push_integer(excmd.addr_count as lua_Integer);
        set(stack, c"range");
        if excmd.addr_count > 0 {
            stack.push_integer(excmd.line2 as lua_Integer);
        } else {
            stack
                .push_integer(lua_Integer::try_from(cmd.default_count).unwrap_or(lua_Integer::MAX));
        }
        set(stack, c"count");

        // `nargs` as `:command -nargs=` spells it.
        stack.push_bytes(&[nargs_char(cmd.argt)]);
        set(stack, c"nargs");

        let mods = cmdmod.with(|cmod| uc_mods(cmod, false));
        stack.push_bytes(&mods);
        set(stack, c"mods");

        // `smods`: the same modifiers, parsed.
        stack.new_table(0, 0);
        let (tab, verbose, split, flags) = cmdmod_report();
        stack.push_integer((tab - 1) as lua_Integer);
        set(stack, c"tab");
        stack.push_integer((verbose - 1) as lua_Integer);
        set(stack, c"verbose");

        let has = |flag: u32| split & c_int::try_from(flag).unwrap_or_default() != 0;
        let split_name: &[u8] = if has(WSP_ABOVE) {
            b"aboveleft"
        } else if has(WSP_BELOW) {
            b"belowright"
        } else if has(WSP_TOP) {
            b"topleft"
        } else if has(WSP_BOT) {
            b"botright"
        } else {
            b""
        };
        stack.push_bytes(split_name);
        set(stack, c"split");

        stack.push_boolean(has(WSP_VERT));
        set(stack, c"vertical");
        stack.push_boolean(has(WSP_HOR));
        set(stack, c"horizontal");

        stack.push_boolean(flags.has(CmdModFlags::SILENT));
        set(stack, c"silent");
        stack.push_boolean(flags.has(CmdModFlags::ERRSILENT));
        set(stack, c"emsg_silent");
        stack.push_boolean(flags.has(CmdModFlags::UNSILENT));
        set(stack, c"unsilent");
        stack.push_boolean(flags.has(CmdModFlags::SANDBOX));
        set(stack, c"sandbox");
        stack.push_boolean(flags.has(CmdModFlags::NOAUTOCMD));
        set(stack, c"noautocmd");
        for (flag, name) in MOD_FLAGS {
            stack.push_boolean(flags.has(flag));
            set(stack, name);
        }
        set(stack, c"smods");

        if preview {
            stack.push_integer(cmdpreview_get_ns() as lua_Integer);
            let cmdpreview_bufnr: Handle = cmdpreview_get_bufnr();
            if cmdpreview_bufnr != 0 {
                stack.push_integer(cmdpreview_bufnr as lua_Integer);
            } else {
                stack.push_nil();
            }
        }

        let (nargs_in, nresults) = if preview { (3, 1) } else { (1, 0) };
        if nlua_pcall(stack, nargs_in, nresults).is_err() {
            nlua_error(stack, c"Lua :command callback: %.*s");
            return 0;
        }

        if !preview {
            return 0;
        }
        // Anything but a number from 0 to 2 is 0, and stays on the stack, as
        // upstream leaves it.
        let answer = stack
            .is_number(At::TOP)
            .then(|| narrow_preview(stack.to_integer(At::TOP)));
        match answer {
            Some(retv) if (0..=2).contains(&retv) => {
                stack.pop(1);
                retv
            }
            _ => 0,
        }
    })
}

/// A preview callback's answer as the `int` upstream cast it to.
fn narrow_preview(answer: lua_Integer) -> c_int {
    #[allow(clippy::cast_possible_truncation)]
    let narrowed = answer as c_int;
    narrowed
}

/// The `-nargs=` letter this command's argument flags mean.
fn nargs_char(argt: ExArgt) -> u8 {
    if !argt.has(ExArgt::EXTRA) {
        return b'0';
    }
    let needarg = argt.has(ExArgt::NEEDARG);
    if argt.has(ExArgt::NOSPC) {
        if needarg { b'1' } else { b'?' }
    } else if needarg {
        b'+'
    } else {
        b'*'
    }
}
