//! `:terminal` and `:lsp`, which both hand a buffer to a process.
#![forbid(unsafe_code)]

use crate::ex_docmd::DoCmdOpts;
use crate::ex_docmd::do_cmdline;
use crate::os::shell::Argv;
use crate::strings::escaped_bytes;
use crate::usercmd::win_cmd_modifiers;
use core::ffi::CStr;

use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{cmdmod_split, cmdmod_tab, kRetNilBool};
use crate::highlight_group::HLF_E;
use crate::lua::executor::nlua_exec_cstr;
use crate::message::e_shellempty;
use crate::option::vars::p_sh;

use crate::message::{emsg, emsg_multiline};
use crate::os::cshim::gettext;

use crate::types::{Array, ExArg, Object, String_0};

/// `:terminal` — spelled as a command line, not as a call.
///
/// The whole command is built as text and handed back to `do_cmdline_cmd`,
/// because `jobstart(…, {'term': v:true})` is the only entry point that
/// attaches a terminal to a buffer, and it is a vimscript function. The
/// argument therefore has to survive being read as a vimscript string
/// literal, which is what the `"` and `\` escaping is for.
pub(crate) fn ex_terminal(excmd: &mut ExArg) {
    // With a window modifier, the modifier plus `new` makes the window;
    // without one, `enew` reuses the current window.
    let mut ex_cmd = if cmdmod_tab() > 0 || cmdmod_split() != 0 {
        let mut text = cmdmod.with(win_cmd_modifiers);
        text.extend_from_slice(b" new");
        text
    } else if excmd.forceit {
        b"enew!".to_vec()
    } else {
        b"enew".to_vec()
    };

    let escape = |word: &[u8]| escaped_bytes(word, b"\"\\").as_cstr().to_bytes().to_vec();
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        ex_cmd.extend_from_slice(b" | call jobstart(\"");
        ex_cmd.extend_from_slice(&escape(excmd.line.arg()));
        ex_cmd.extend_from_slice(b"\",{'term':v:true})");
    } else {
        if p_sh(CStr::is_empty) {
            emsg(gettext(e_shellempty));
            return;
        }
        // No argument: run 'shell', as a *list* so that the shell's own
        // arguments are not re-split.
        let argv = Argv::interactive_shell();
        let list: Vec<Vec<u8>> = argv
            .words()
            .map(|word| [&b"\""[..], &escape(word.to_bytes()), b"\""].concat())
            .collect();
        ex_cmd.extend_from_slice(b" | call jobstart([");
        ex_cmd.extend_from_slice(&list.join(&b',')[..]);
        ex_cmd.extend_from_slice(b"], {'term':v:true})");
    }
    let _ = do_cmdline(
        Some(&ex_cmd),
        None,
        DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT | DoCmdOpts::KEYTYPED,
    );
}

/// `:lsp` — a Lua entry point that takes the whole argument as one string.
pub(crate) fn ex_lsp(excmd: &mut ExArg) {
    let excmd = Array::from(vec![Object::string(String_0::from_bytes(excmd.line.arg()))]);
    const CHUNK: &CStr = c"require'vim._core.ex_cmd'.ex_lsp(...)";
    let ran = nlua_exec_cstr(CHUNK, excmd, kRetNilBool);
    if let Err(e) = ran {
        emsg_multiline(e.message_or_empty(), Some(c"lua_error"), HLF_E, true);
    }
}
