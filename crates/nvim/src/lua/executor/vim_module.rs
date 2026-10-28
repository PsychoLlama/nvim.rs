//! Building the `vim` table itself.
//!
//! [`VIM_FUNCTIONS`] is the half of `vim.*` the executor implements;
//! [`add_common`] is the part every state gets -- `vim.is_thread` and the
//! `vim.NIL` and `vim.empty_dict()` sentinels -- and [`init_packages`] runs
//! `vim._init_packages` over the embedded Lua modules. [`ui_attach`] and
//! [`ui_detach`] are `vim.ui_attach()`, whose callbacks the compositor calls
//! back into, and [`thread_api`] the one api function a luv thread gets.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::io::Write;

use super::{
    BUILTIN_MODULES, call, in_fast_event, nlua_pcall, rpcnotify, rpcrequest, schedule, until_nul,
    wait,
};
use crate::api::extmark::ns_initialized;
use crate::lua::converter::{pop_array, push_array};
use crate::lua::stack::{At, Function, Kind, Raise, Stack};
use crate::lua::state::{MAIN_REFS, nlua_disable_preload};
use crate::lua_functions;
use crate::runtime::runtime_get_named_thread;
use crate::types::ui::kUILinegrid;
use crate::ui::state::{ui_ext_names, ui_refresh_cmdheight};
use crate::ui::{ui_add_cb, ui_remove_cb};

lua_functions! {
    /// The `vim.*` functions the executor implements, in the order they are
    /// set on the main state's `vim` table.
    pub(crate) enum VimFunctions in VIM_FUNCTIONS {
        0 => c"schedule": schedule,
        1 => c"in_fast_event": in_fast_event,
        2 => c"call": call,
        3 => c"rpcrequest": rpcrequest,
        4 => c"rpcnotify": rpcnotify,
        5 => c"wait": wait,
        6 => c"ui_attach": ui_attach,
        7 => c"ui_detach": ui_detach,
    }
}

lua_functions! {
    /// What every state's `vim` table and sentinels carry.
    enum Common in COMMON_FUNCTIONS {
        0 => c"is_thread": is_thread,
        1 => c"__tostring": nil_tostring,
        2 => c"__index": nil_index,
        3 => c"__tostring": empty_dict_tostring,
        4 => c"preload": module_preloader,
    }
}

lua_functions! {
    /// A luv thread's `vim.api`.
    enum ThreadApi in THREAD_API {
        0 => c"nvim__get_runtime": thread_get_runtime,
    }
}

/// A luv thread's `vim.api` functions.
pub(crate) fn thread_api() -> &'static [Function] {
    THREAD_API
}

/// The name the module whose preload the `--luamod-dev` flag stops at.
const FIRST_DEV_MODULE: &CStr = c"vim.inspect";

/// `vim.is_thread()`.
fn is_thread(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.get_field(At::REGISTRY, c"nvim.thread");
    Ok(1)
}

/// `vim.NIL`'s `__tostring`.
fn nil_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"vim.NIL");
    Ok(1)
}

/// `vim.NIL`'s `__index` and `__newindex`: indexing a JSON `null` names
/// what was indexed, rather than Lua's "attempt to index a userdata value".
fn nil_index(_: &mut Stack) -> Result<c_int, Raise> {
    Err(Raise::message("attempt to index vim.NIL"))
}

/// `vim.empty_dict()`'s `__tostring`.
fn empty_dict_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"vim.empty_dict()");
    Ok(1)
}

/// The half of the `vim` table that a thread state gets as well as the main
/// one: `vim.is_thread` and the two identity sentinels. On the main state
/// the sentinels are references, which is how the converter pushes them.
///
/// Expects the `vim` table on top; leaves it there.
pub(crate) fn add_common(stack: &mut Stack, is_thread: bool) {
    let [
        is_thread_function,
        nil_tostring,
        nil_index,
        empty_dict_tostring,
        _,
    ]: &[Function; 5] = COMMON_FUNCTIONS.try_into().expect("five functions");

    stack.push_boolean(is_thread);
    stack.set_field(At::REGISTRY, c"nvim.thread");
    stack.push_function(is_thread_function.entry());
    stack.set_field(At::from_top(2), c"is_thread");

    // vim.NIL: a zero-length userdatum, so it is unique and compares by
    // identity, with a metatable that renders it.
    stack.new_empty_userdata();
    stack.new_table(0, 0);
    stack.push_function(nil_tostring.entry());
    stack.set_field(At::from_top(2), c"__tostring");
    stack.push_function(nil_index.entry());
    stack.set_field(At::from_top(2), c"__index");
    stack.push_function(nil_index.entry());
    stack.set_field(At::from_top(2), c"__newindex");
    stack.set_metatable(At::from_top(2));
    if !is_thread {
        let nil_ref = stack.reference(At::TOP);
        MAIN_REFS.with_mut(|refs| refs.nil_ref = nil_ref);
    }
    stack.push_value(At::TOP);
    stack.set_field(At::REGISTRY, c"mpack.NIL");
    stack.set_field(At::from_top(2), c"NIL");

    // vim.empty_dict(): the *metatable* is the marker, so an empty table
    // carrying it converts as a dictionary rather than a list.
    stack.new_table(0, 0);
    stack.push_function(empty_dict_tostring.entry());
    stack.set_field(At::from_top(2), c"__tostring");
    if !is_thread {
        let empty_dict_ref = stack.reference(At::TOP);
        MAIN_REFS.with_mut(|refs| refs.empty_dict_ref = empty_dict_ref);
    }
    stack.push_value(At::TOP);
    stack.set_field(At::REGISTRY, c"mpack.empty_dict");
    stack.set_field(At::from_top(2), c"_empty_dict_mt");
}

/// The `package.preload` entry for one embedded module: load its bytecode and
/// run it. Its upvalue is the module's index in [`BUILTIN_MODULES`].
fn module_preloader(stack: &mut Stack) -> Result<c_int, Raise> {
    let index = usize::try_from(stack.to_integer(At::upvalue(1))).unwrap_or(usize::MAX);
    let chunk = BUILTIN_MODULES[index].chunk();
    if stack.load(chunk, None).is_err() {
        return Err(Raise::top_value());
    }
    Err(Raise::tail_call(0, 1))
}

/// Register every embedded module in `package.preload`, then `require`
/// `vim._init_packages`, which is what wires up the rest of the runtime.
///
/// `nlua_disable_preload` (`--luamod-dev`) stops the registration at
/// `vim.inspect`, so everything after it is loaded from the runtime
/// directory instead.
pub(crate) fn init_packages(stack: &mut Stack, is_standalone: bool) -> bool {
    let preloader = COMMON_FUNCTIONS[4].entry();
    stack.get_global(c"package");
    stack.get_field(At::TOP, c"preload");
    for (index, def) in (0..).zip(BUILTIN_MODULES) {
        stack.push_integer(index);
        stack.push_closure(preloader, 1);
        stack.set_field(At::from_top(2), def.name);
        if nlua_disable_preload.get() && !is_standalone && def.name == FIRST_DEV_MODULE {
            break;
        }
    }
    stack.pop(2);

    stack.get_global(c"require");
    stack.push_bytes(b"vim._init_packages");
    if nlua_pcall(stack, 1, 0).is_err() {
        // `fprintf(stderr, "%s\n", lua_tostring(L, -1))`.
        let error = stack.to_bytes(At::TOP).map_or(&b"(null)"[..], until_nul);
        let _ = std::io::stderr().write_all(&[error, b"\n"].concat());
        return false;
    }
    true
}

/// `vim.ui_attach(ns_id, opts, callback)`.
///
/// `opts` names the UI extensions the callback wants; at least one must be
/// true, or the attach is refused.
fn ui_attach(stack: &mut Stack) -> Result<c_int, Raise> {
    let ns_id = ns_argument(stack)?;
    if stack.kind(At::arg(2)) != Kind::Table {
        return Err(Raise::message("opts must be a table"));
    }
    if stack.kind(At::arg(3)) != Kind::Function {
        return Err(Raise::message("callback must be a Lua function"));
    }

    let mut ext_widgets = [false; kUILinegrid as usize];
    let mut tbl_has_true_val = false;
    stack.push_value(At::arg(2));
    stack.push_nil();
    while stack.next(At::from_top(2)) {
        let value = stack.to_boolean(At::TOP);
        // `lua_tolstring` of the key: a number key is turned into its
        // string in place, as upstream does, and the name is a C string.
        let key = stack.to_bytes(At::from_top(2)).map(until_nul);
        if key == Some(b"set_cmdheight".as_slice()) {
            ui_refresh_cmdheight.set(value);
        } else {
            let found = key.and_then(|key| {
                ui_ext_names[..kUILinegrid as usize]
                    .iter()
                    .position(|name| name.to_bytes() == key)
            });
            match found {
                Some(i) => {
                    if value {
                        tbl_has_true_val = true;
                    }
                    ext_widgets[i] = value;
                }
                None => {
                    let key = key
                        .map(<[u8]>::to_vec)
                        .unwrap_or_else(|| b"(null)".to_vec());
                    return Err(Raise::message(
                        [b"Unexpected key: ".as_slice(), &key].concat(),
                    ));
                }
            }
        }
        stack.pop(1);
    }
    if !tbl_has_true_val {
        return Err(Raise::message(
            "opts table must contain at least one 'true' ext_widget",
        ));
    }

    let ui_event_cb = stack.reference(At::arg(3));
    ui_add_cb(ns_id, ui_event_cb, ext_widgets);
    ui_refresh_cmdheight.set(true);
    Ok(0)
}

/// `vim.ui_detach(ns_id)`.
fn ui_detach(stack: &mut Stack) -> Result<c_int, Raise> {
    let ns_id = ns_argument(stack)?;
    ui_remove_cb(ns_id, false);
    Ok(0)
}

/// The namespace at slot 1, as `(uint32_t)luaL_checkinteger(L, 1)` reads it,
/// refused unless it has been created.
fn ns_argument(stack: &mut Stack) -> Result<u32, Raise> {
    // The C cast keeps the low 32 bits.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ns_id = stack.check_integer(At::arg(1))? as u32;
    if !ns_initialized(ns_id) {
        return Err(Raise::message("invalid ns_id"));
    }
    Ok(ns_id)
}

/// `nvim__get_runtime()` as a thread state sees it: the one api function a
/// luv thread is allowed, because it reads only the runtime path.
fn thread_get_runtime(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 3 {
        return Err(Raise::message("Expected 3 arguments"));
    }

    stack.check_kind(At::TOP, Kind::Table)?;
    stack.get_field(At::TOP, c"is_lua");
    if stack.kind(At::TOP) != Kind::Boolean {
        return Err(Raise::message("is_lua is not a boolean"));
    }
    let is_lua = stack.to_boolean(At::TOP);
    stack.pop(2);

    stack.check_kind(At::TOP, Kind::Boolean)?;
    let all = stack.to_boolean(At::TOP);
    stack.pop(1);

    let pat = match pop_array(stack) {
        Ok(pat) => pat,
        Err(e) => return Err(Raise::message(e.message_or_empty().to_bytes())),
    };

    let ret = runtime_get_named_thread(is_lua, &pat, all);
    push_array(stack, &ret, true);
    Ok(1)
}
