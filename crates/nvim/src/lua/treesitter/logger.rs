//! The parser's optional log callback.
//!
//! `parser:_set_logger(lex, parse, fn)` installs a [`TSLogger`] whose
//! payload is a [`Logger`]: which of lex/parse to report, and -- while a
//! parse runs -- where the callback sits on that parse's stack. The
//! callback itself is index 1 of the parser userdatum's environment table,
//! so the collector owns it: nothing to release when the parser dies or a
//! second logger replaces the first (upstream leaked the first's options
//! and its registry reference).
//!
//! The callback runs protected. A failure is not raised from inside
//! tree-sitter's frames, as upstream's `luaL_error` was: it is recorded,
//! the rest of the parse logs nothing, and the parse raises once
//! tree-sitter has returned ([`LogScope::finish`]).

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::Cell;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::NonNull;
use std::rc::Rc;

use super::parser::{Parser, check_parser};
use super::*;
use crate::lua::stack::{Kind, Raise};
use crate::types::lua_State;

/// One installed logger: the payload tree-sitter hands back to
/// [`logger_cb`].
pub(crate) struct Logger {
    lex: bool,
    parse: bool,
    /// While a parse runs: the state it runs on and the slot its callback
    /// was pushed to.
    call: Cell<Option<(NonNull<lua_State>, At)>>,
    /// The callback failed during the running parse.
    failed: Cell<bool>,
}

/// What `logger_cb` is told, as tree-sitter's C type.
type LogFunction = unsafe extern "C-unwind" fn(*mut c_void, TSLogType, *const c_char);

/// Tree-sitter's log callback: call the Lua callback with the kind and the
/// message, unless the kind is filtered out, no parse of ours is running,
/// or the callback already failed in this one.
///
/// # Safety
/// `payload` is the [`Logger`] this logger was installed with, kept alive
/// by whoever installed it (see [`LogScope`]); `message` is NUL-terminated.
/// Tree-sitter's callback contract, which hands back what it was given.
unsafe extern "C-unwind" fn logger_cb(
    payload: *mut c_void,
    kind: TSLogType,
    message: *const c_char,
) {
    // SAFETY: the caller's live `Logger`, only ever read through shared
    // references (its state is in `Cell`s).
    let logger = unsafe { &*payload.cast_const().cast::<Logger>() };
    let wanted = if kind == LOG_TYPE_LEX {
        logger.lex
    } else {
        logger.parse
    };
    let Some((state, callback)) = logger.call.get() else {
        return;
    };
    if !wanted || logger.failed.get() {
        return;
    }
    // SAFETY: the state the running parse is on, inside the C function that
    // started it; this view owns nothing below what it pushes, and the
    // parse's own view is not used until tree-sitter returns.
    let mut stack = unsafe { Stack::above_top(state.as_ptr()) };
    stack.push_value(callback);
    stack.push_bytes(if kind == LOG_TYPE_PARSE {
        b"parse"
    } else {
        b"lex"
    });
    // SAFETY: the caller's NUL-terminated message.
    stack.push_bytes(unsafe { c_bytes(message) });
    if stack.pcall(2, 0, None).is_err() {
        stack.set_top(stack.base());
        logger.failed.set(true);
    }
}

/// `parser:_set_logger(lex, parse, fn)`.
pub(crate) fn parser_set_logger(stack: &mut Stack) -> Result<c_int, Raise> {
    let parser = check_parser(stack, At::arg(1))?;
    for slot in [2, 3] {
        if stack.kind(At::arg(slot)) != Kind::Boolean {
            return Err(Raise::argument(slot, "boolean expected"));
        }
    }
    if stack.kind(At::arg(4)) != Kind::Function {
        return Err(Raise::argument(4, "function expected"));
    }
    let logger = Rc::new(Logger {
        lex: stack.to_boolean(At::arg(2)),
        parse: stack.to_boolean(At::arg(3)),
        call: Cell::new(None),
        failed: Cell::new(false),
    });
    stack.new_table(1, 0);
    stack.push_value(At::arg(4));
    stack.raw_set_index(At::from_top(2), 1);
    stack.set_environment(At::arg(1));
    let installed = TSLogger {
        payload: Rc::as_ptr(&logger).cast_mut().cast::<c_void>(),
        log: Some(logger_cb as LogFunction),
    };
    // SAFETY: the parser the userdatum owns; the payload is kept alive by
    // the userdatum, which outlives the parser's use of it (the parser is
    // deleted first), or by the parse running when it is replaced.
    unsafe { ts_parser_set_logger(parser, installed) };
    if let Some(userdatum) = stack.to_userdata::<Parser>(At::arg(1)) {
        userdatum.logger = Some(logger);
    }
    Ok(0)
}

/// `parser:_logger()`: the callback, or nil when none is installed.
pub(crate) fn parser_get_logger(stack: &mut Stack) -> Result<c_int, Raise> {
    check_parser(stack, At::arg(1))?;
    let installed = stack
        .to_userdata::<Parser>(At::arg(1))
        .is_some_and(|userdatum| userdatum.logger.is_some());
    if installed {
        stack.push_environment(At::arg(1));
        stack.raw_get_index(At::TOP, 1);
    } else {
        stack.push_nil();
    }
    Ok(1)
}

/// The logger of one parse, armed for its length.
pub(crate) struct LogScope {
    logger: Rc<Logger>,
    /// What an outer parse of the same parser had armed.
    outer: (Option<(NonNull<lua_State>, At)>, bool),
}

impl LogScope {
    /// Arm the logger of the parser userdatum at `parser`, if it has one:
    /// push its callback and tell the logger where it is.
    pub(crate) fn begin(stack: &mut Stack, parser: At) -> Option<LogScope> {
        let logger = stack.to_userdata::<Parser>(parser)?.logger.clone()?;
        stack.push_environment(parser);
        stack.raw_get_index(At::TOP, 1);
        let callback = stack.top();
        let state = NonNull::new(stack.lua_state()).expect("a live state");
        let outer = (
            logger.call.replace(Some((state, callback))),
            logger.failed.replace(false),
        );
        Some(LogScope { logger, outer })
    }

    /// Disarm the logger and drop the two slots `begin` pushed; `true` when
    /// the callback failed during the parse.
    pub(crate) fn finish(self, stack: &mut Stack) -> bool {
        let (call, failed) = self.outer;
        self.logger.call.set(call);
        stack.pop(2);
        self.logger.failed.replace(failed)
    }
}
