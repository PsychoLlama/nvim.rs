#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! `:trust`, the ex-command face of `vim.secure.trust`.

use crate::lua::executor::{nlua_error, nlua_pcall, until_nul, with_main_stack};
use crate::lua::stack::At;
use crate::types::ExArg;
use core::ffi::CStr;

/// What `:trust` asks `vim.secure.trust` to do with the path.
#[derive(Clone, Copy)]
enum TrustAction {
    Allow,
    Deny,
    Remove,
}

impl TrustAction {
    /// The `action` field `vim.secure.trust` expects.
    fn name(self) -> &'static CStr {
        match self {
            TrustAction::Allow => c"allow",
            TrustAction::Deny => c"deny",
            TrustAction::Remove => c"remove",
        }
    }
}

/// Call `vim.secure.trust{ action = ..., path|bufnr = ... }` and report
/// what it says. `path` of `None` means the current buffer.
fn nlua_trust(action: TrustAction, path: Option<&CStr>) -> bool {
    with_main_stack(|stack| {
        // What every path below unwinds the stack to.
        let top = stack.top();

        // `vim.secure.trust` and its one argument table, which `nlua_pcall`
        // consumes.
        stack.get_global(c"vim");
        stack.get_field(At::TOP, c"secure");
        stack.get_field(At::TOP, c"trust");

        stack.new_table(0, 0);
        stack.push_bytes(b"action");
        stack.push_bytes(action.name().to_bytes());
        stack.set_table(At::from_top(3));
        match path {
            Some(path) => {
                stack.push_bytes(b"path");
                stack.push_bytes(path.to_bytes());
            }
            None => {
                stack.push_bytes(b"bufnr");
                stack.push_number(0.0);
            }
        }
        stack.set_table(At::from_top(3));

        if nlua_pcall(stack, 1, 2).is_err() {
            nlua_error(stack, c"vim.secure.trust: %.*s");
            stack.set_top(top);
            return false;
        }

        let success = stack.to_boolean(At::from_top(2));
        // The second result is the path it acted on, or why it could not.
        if let Some(msg) = stack.to_bytes(At::TOP) {
            let msg = String::from_utf8_lossy(until_nul(msg));
            match (success, action) {
                (true, TrustAction::Allow) => {
                    crate::smsg!(0, "Allowed in trust database: \"{msg}\"");
                }
                (true, TrustAction::Deny) => {
                    crate::smsg!(0, "Denied in trust database: \"{msg}\"");
                }
                (true, TrustAction::Remove) => {
                    crate::smsg!(0, "Removed from trust database: \"{msg}\"");
                }
                (false, _) => {
                    crate::semsg!("E5570: Cannot update trust file: {msg}");
                }
            }
        }

        stack.set_top(top);
        success
    })
}

/// `:trust [++deny|++remove] [path]`. Without a path it acts on the
/// current buffer.
pub fn ex_trust(excmd: &mut ExArg) {
    let arg = excmd.line.arg;
    let rest = excmd.line.skip_to_white(arg);
    let word = excmd.line.slice_at(arg, rest - arg);
    let action = match word {
        b"++deny" => TrustAction::Deny,
        b"++remove" => TrustAction::Remove,
        b"" => TrustAction::Allow,
        _ => {
            crate::semsg!("E475: Invalid argument: {}", String::from_utf8_lossy(word));
            return;
        }
    };
    let path = excmd.line.skip_white(rest);
    let path = (excmd.line.byte_at(path) != 0).then(|| excmd.line.cstr_from(path));
    nlua_trust(action, path);
}
