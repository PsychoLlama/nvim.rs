//! `print()`, `require()` and `vim.debug()`.
//!
//! Neovim replaces Lua's `print` so its output goes through the editor's
//! message path rather than stdout, and defers it when called from a fast
//! callback or a luv thread. [`require`] is the wrapper that `--startuptime`
//! times each module through, and [`debug`] is the `vim.debug()`
//! read-eval-print loop.

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

use super::{in_fast_callback, nlua_error, nlua_pcall, require_ref, until_nul};
use crate::eval::typval::{TV_INITIAL_VALUE, tv_clear};
use crate::event::r#loop::schedule_deferred_job;
use crate::event::multiqueue::put_job_on_main_loop;
use crate::ex_getln::{get_user_input, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave};
use crate::lua::stack::{At, CFunction, Function, Raise, Stack};
use crate::lua_functions;
use crate::message::{msg_multihl, msg_putchar};
use crate::os::cshim::gettext;
use crate::profile::startup_timing;
use crate::profile::{time_msg, time_pop, time_push};
use crate::types::ui::kUICmdline;
use crate::types::{HlMessageChunk, IOSIZE, Object, String_0, TypVal};
use crate::ui::ui_has;

lua_functions! {
    /// The functions that replace or wrap Lua's own: `print`,
    /// `debug.debug` and `require`.
    pub(crate) enum Runtime in RUNTIME_FUNCTIONS {
        0 => c"print": print,
        1 => c"debug": debug,
        2 => c"require": require,
    }
}

/// `print`, for a state of any kind.
pub(crate) fn print_function() -> CFunction {
    RUNTIME_FUNCTIONS[0].entry()
}

/// The `--startuptime` wrapper of `require`.
pub(crate) fn require_function() -> CFunction {
    RUNTIME_FUNCTIONS[2].entry()
}

/// Replace the main state's `print` and `debug.debug`.
pub(crate) fn register_runtime(stack: &mut Stack) {
    let [print, debug, _]: &[Function; 3] = RUNTIME_FUNCTIONS.try_into().expect("three functions");
    stack.push_function(print.entry());
    stack.set_global(print.name());

    stack.get_global(c"debug");
    stack.push_function(debug.entry());
    stack.set_field(At::from_top(2), debug.name());
    stack.pop(1);
}

/// The message kind `print()`'s output is reported under.
const LUA_PRINT_KIND: &CStr = c"lua_print";

/// Show one `print()`'s worth of text.
fn show_printed(text: Vec<u8>) {
    let chunk = HlMessageChunk {
        text: String_0::from_bytes(&text),
        hl_id: 0,
    };
    let mut needs_clear = false;
    msg_multihl(
        Object::Nil,
        &[chunk],
        Some(LUA_PRINT_KIND),
        true,
        false,
        None,
        &mut needs_clear,
    );
}

/// What `print()` reports when `tostring` answers something that is not a
/// string at all.
const NULL_TOSTRING: &[u8] = b"<Unknown error: lua_tolstring returned NULL for tostring result>";

/// Neovim's `print()`: every argument through `tostring`, space-separated,
/// through the message path.
///
/// A thread state and a fast callback both defer the message onto the main
/// loop; everything else shows it directly.
fn print(stack: &mut Stack) -> Result<c_int, Raise> {
    let nargs = stack.top().index();
    stack.get_global(c"tostring");

    let mut text = Vec::<u8>::new();
    for arg in 1..=nargs {
        stack.push_value(At::TOP); // tostring
        stack.push_value(At::arg(arg));
        if stack.pcall(1, 1, None).is_err() {
            let error = stack
                .to_bytes(At::TOP)
                .map(|error| until_nul(error).to_vec());
            return Err(conversion_failed(stack, arg, error.as_deref()));
        }
        let Some(shown) = stack.to_bytes(At::TOP) else {
            return Err(conversion_failed(stack, arg, Some(NULL_TOSTRING)));
        };
        text.extend_from_slice(shown);
        if arg < nargs {
            text.push(b' ');
        }
        stack.pop(1);
    }

    stack.get_field(At::REGISTRY, c"nvim.thread");
    let is_thread = stack.to_boolean(At::TOP);
    stack.pop(1);

    if is_thread {
        schedule_deferred_job(Box::new(move || show_printed(text)));
    } else if in_fast_callback.get() != 0 {
        put_job_on_main_loop(Box::new(move || show_printed(text)));
    } else {
        show_printed(text);
    }
    Ok(0)
}

/// `print()` could not convert argument `arg`: nothing is shown, and the
/// failure is thrown at the Lua caller instead. `error` is why, as the C
/// string it was (`None` for no string at all).
///
/// The message is what upstream's `IOSIZE` buffer holds, not what the
/// message would have been: upstream pushes the would-be length, and reads
/// past the buffer when the error is longer than it.
fn conversion_failed(stack: &mut Stack, arg: c_int, error: Option<&[u8]>) -> Raise {
    let template = gettext(c"E5114: Converting print argument #%i: %.*s").to_bytes();
    let error_len = c_int::try_from(error.map_or(0, <[u8]>::len)).unwrap_or(c_int::MAX);
    let message = match error {
        Some(error) => crate::formatted!(IOSIZE as usize, template, arg, error_len, error),
        None => crate::formatted!(IOSIZE as usize, template, arg, error_len, None::<&CStr>),
    };
    stack.push_bytes(&message);
    Raise::top_value()
}

/// The startuptime label for loading `module`: `require('module')`.
fn require_label(module: &CStr) -> Vec<u8> {
    let mut label = b"require('".to_vec();
    label.extend_from_slice(module.to_bytes());
    label.extend_from_slice(b"')");
    label
}

/// Neovim's `require()`: the stock one, wrapped so that `--startuptime`
/// records how long each module took.
///
/// A module already in `package.loaded` short-circuits, and once timing is
/// over the wrapper puts the stock `require` back as the global and hands
/// straight through to it.
fn require(stack: &mut Stack) -> Result<c_int, Raise> {
    // `luaL_checkstring`'s C string: up to the first NUL.
    let name = until_nul(stack.check_bytes(At::arg(1))?).to_vec();
    let name = CString::new(name).expect("cut at the first NUL");
    stack.set_top(At::arg(1));
    stack.get_field(At::REGISTRY, c"_LOADED");
    stack.get_field(At::arg(2), &name);
    if stack.to_boolean(At::TOP) {
        return Ok(1);
    }
    stack.pop(2);

    stack.push_reference(require_ref.get());
    stack.insert(At::arg(1));

    if !startup_timing() {
        // Not profiling: hand straight through, restoring the global
        // `require` if it is still this wrapper.
        stack.get_global(c"require");
        if stack.is_c_function(At::TOP, require_function()) {
            stack.push_value(At::arg(1));
            stack.set_global(c"require");
        }
        stack.pop(1);
        return Err(Raise::tail_call(1, 1));
    }

    let (rel_time, start_time) = time_push();
    let loaded = stack.pcall(1, 1, None).is_ok();
    if loaded {
        time_msg(&require_label(&name), Some(start_time));
    }
    time_pop(rel_time);
    if loaded {
        Ok(1)
    } else {
        Err(Raise::top_value())
    }
}

/// The prompt `vim.debug()` reads each line with.
const DEBUG_PROMPT: &[u8] = b"lua_debug> ";

/// `vim.debug()`: read a line, run it, repeat until `cont` or an empty line.
fn debug(stack: &mut Stack) -> Result<c_int, Raise> {
    // The prompt is a copy the array owns and releases.
    let input_args = [TypVal::string_from(DEBUG_PROMPT)];
    loop {
        stack.set_top(At::arg(0));
        let mut input = TV_INITIAL_VALUE;
        get_user_input(&input_args, &mut input, false, false);

        if ui_has(kUICmdline) {
            // `snprintf(line, IOSIZE, "lua_debug> %s", input)`: a null
            // string as the C library prints one.
            let shown = input
                .string_ref()
                .map_or(&b"(null)"[..], |text| text.as_bytes());
            let mut line = [DEBUG_PROMPT, until_nul(shown)].concat();
            line.truncate(IOSIZE as usize - 1);
            ui_ext_cmdline_block_append(0, &line);
        } else {
            msg_putchar(c_int::from(b'\n'));
        }

        // Done on anything but a String with text other than "cont".
        let Some(text) = input
            .string_ref()
            .filter(|s| !s.is_empty() && s.as_bytes() != b"cont")
        else {
            tv_clear(&mut input);
            if ui_has(kUICmdline) {
                ui_ext_cmdline_block_leave();
            }
            return Ok(0);
        };

        if stack
            .load(text.as_bytes(), Some(c"=(debug command)"))
            .is_err()
        {
            nlua_error(stack, c"E5115: Loading Lua debug string: %.*s");
        } else if nlua_pcall(stack, 0, 0).is_err() {
            nlua_error(stack, c"E5116: Calling Lua debug string: %.*s");
        }
        tv_clear(&mut input);
    }
}
