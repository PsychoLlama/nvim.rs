//! Creating, initialising and freeing a `lua_State`.
//!
//! [`nlua_init_state`] is the constructor -- for a thread's state and for a
//! `-l` script's -- and `nlua_state_init` the part that only the main one
//! gets: the api functions, `vim._init_packages`, treesitter and the
//! standard library.  [`nlua_init`] is what `main()` calls.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::{
    active_lstate, global_lstate, in_script, luv_set_thread_cb, main_thread, nlua_call,
    nlua_common_vim_init, nlua_debug, nlua_exec_file, nlua_get_ref_state, nlua_in_fast_event,
    nlua_init_packages, nlua_pcall, nlua_print, nlua_ref_global, nlua_require, nlua_rpcnotify,
    nlua_rpcrequest, nlua_schedule, nlua_thr_api_nvim__get_runtime, nlua_thread_acquire_vm,
    nlua_ui_attach, nlua_ui_detach, nlua_unref, nlua_wait, require_ref,
};
use crate::event::libuv::{uv_thread_equal, uv_thread_self};
use crate::lua::api_wrappers::nlua_add_api_functions;
use crate::lua::converter::nlua_init_types;
use crate::lua::ffi::{
    LUA_REGISTRYINDEX, lua_close, lua_getfield, lua_getglobal, lua_newtable, lua_pop,
    lua_pushcfunction, lua_pushinteger, lua_pushstring, lua_rawseti, lua_setfield, lua_setglobal,
    lua_tostring, luaL_newstate, luaL_openlibs,
};
use crate::lua::stdlib::nlua_state_add_stdlib;
use crate::lua::treesitter::nlua_treesitter_init;
use crate::main::{os_exit, time_fd};
use crate::os::cshim::{gettext, stderr};
use crate::runtime::runtime_search_path_validate;
use crate::types::{lua_Integer, lua_State, nlua_ref_state_t, uv_thread_t};
use ::libc::{exit, fprintf};

/// Populate the global `arg` table from the command line, with `arg[0]` the
/// script's own name.
///
/// # Safety
/// `argv` must hold `argc` NUL-terminated strings.
unsafe fn nlua_init_argv(
    lstate: *mut lua_State,
    argv: *mut *mut c_char,
    argc: c_int,
    lua_arg0: c_int,
) -> c_int {
    unsafe {
        let mut i: c_int = 0;
        lua_newtable(lstate);
        if lua_arg0 > 0 {
            lua_pushstring(lstate, *argv.offset((lua_arg0 - 1) as isize));
            lua_rawseti(lstate, -2, 0);
            while i + lua_arg0 < argc {
                lua_pushstring(lstate, *argv.offset((i + lua_arg0) as isize));
                lua_rawseti(lstate, -2, i + 1);
                i += 1;
            }
        }
        lua_setglobal(lstate, c"arg".as_ptr());
        i
    }
}

/// Everything the *main* state gets: the replaced `print` and `debug.debug`,
/// the api, the `vim.*` C functions, treesitter, the standard library, and
/// the embedded runtime modules.
///
/// # Safety
/// `lstate` must be a freshly opened Lua state.
unsafe fn nlua_state_init(lstate: *mut lua_State) -> bool {
    unsafe {
        lua_pushcfunction(lstate, nlua_print);
        lua_setglobal(lstate, c"print".as_ptr());

        lua_getglobal(lstate, c"debug".as_ptr());
        lua_pushcfunction(lstate, nlua_debug);
        lua_setfield(lstate, -2, c"debug".as_ptr());
        lua_pop(lstate, 1);

        // The `vim` table, built on the stack and stored last.
        lua_newtable(lstate);
        nlua_add_api_functions(lstate);
        nlua_init_types(lstate);

        let set = |name: &CStr, f: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int| {
            lua_pushcfunction(lstate, f);
            lua_setfield(lstate, -2, name.as_ptr());
        };
        set(c"schedule", nlua_schedule);
        set(c"in_fast_event", nlua_in_fast_event);
        set(c"call", nlua_call);
        set(c"rpcrequest", nlua_rpcrequest);
        set(c"rpcnotify", nlua_rpcnotify);
        set(c"wait", nlua_wait);
        set(c"ui_attach", nlua_ui_attach);
        set(c"ui_detach", nlua_ui_detach);

        nlua_common_vim_init(lstate, false, false);

        // Only `--startuptime` needs `require` wrapped, and the wrapper needs
        // the original to delegate to.
        if !(*time_fd.ptr()).is_null() {
            lua_getglobal(lstate, c"require".as_ptr());
            require_ref.set(nlua_ref_global(lstate, -1));
            lua_pop(lstate, 1);
            lua_pushcfunction(lstate, nlua_require);
            lua_setglobal(lstate, c"require".as_ptr());
        }

        nlua_treesitter_init(lstate);
        nlua_state_add_stdlib(lstate, false);
        lua_setglobal(lstate, c"vim".as_ptr());

        nlua_init_packages(lstate, false)
    }
}

/// Open the editor's Lua state. Fatal if it cannot be built.
///
/// # Safety
/// Called once, from `main()`.
pub unsafe fn nlua_init(argv: *mut *mut c_char, argc: c_int, lua_arg0: c_int) {
    unsafe {
        let lstate = luaL_newstate();
        if lstate.is_null() {
            fprintf(
                stderr,
                gettext(c"E970: Failed to initialize Lua interpreter\n".as_ptr()),
            );
            os_exit(1);
        }
        luaL_openlibs(lstate);
        if !nlua_state_init(lstate) {
            fprintf(
                stderr,
                gettext(c"E970: Failed to initialize builtin Lua modules\n".as_ptr()),
            );
            os_exit(1);
        }
        luv_set_thread_cb(Some(nlua_thread_acquire_vm), Some(nlua_common_free_all_mem));

        global_lstate.set(lstate);
        active_lstate.set(lstate);
        main_thread.set(uv_thread_self());
        nlua_init_argv(lstate, argv, argc, lua_arg0);
    }
}

/// `nvim -l script.lua`: run the script and leave, with no editor at all.
///
/// # Safety
/// Called once, from `main()`, instead of [`nlua_init`].
pub unsafe fn nlua_run_script(argv: *mut *mut c_char, argc: c_int, lua_arg0: c_int) -> ! {
    unsafe {
        in_script.set(true);
        global_lstate.set(nlua_init_state(false));
        luv_set_thread_cb(Some(nlua_thread_acquire_vm), Some(nlua_common_free_all_mem));
        nlua_init_argv(global_lstate.get(), argv, argc, lua_arg0);
        let lua_ok = nlua_exec_file(*argv.offset((lua_arg0 - 1) as isize));
        exit(if lua_ok { 0 } else { 1 });
    }
}

/// A state for a luv thread, or for a `-l` script.
///
/// It gets the shared half of the `vim` table and the thread-safe half of the
/// standard library; a thread also gets the one api function it is allowed.
///
/// # Safety
/// Called on the thread the state will belong to.
pub(crate) unsafe fn nlua_init_state(thread: bool) -> *mut lua_State {
    unsafe {
        // The runtime path is shared, so only the main thread may rebuild it.
        let self_0: uv_thread_t = uv_thread_self();
        if !in_script.get() && uv_thread_equal(main_thread.ptr(), &raw const self_0) != 0 {
            runtime_search_path_validate();
        }

        let lstate = luaL_newstate();
        luaL_openlibs(lstate);
        if !in_script.get() {
            lua_pushcfunction(lstate, nlua_print);
            lua_setglobal(lstate, c"print".as_ptr());
        }
        lua_pushinteger(lstate, 0 as lua_Integer);
        lua_setfield(lstate, LUA_REGISTRYINDEX, c"nlua.refcount".as_ptr());

        lua_newtable(lstate);
        nlua_common_vim_init(lstate, thread, in_script.get());
        nlua_state_add_stdlib(lstate, true);
        if !in_script.get() {
            lua_newtable(lstate);
            lua_pushcfunction(lstate, nlua_thr_api_nvim__get_runtime);
            lua_setfield(lstate, -2, c"nvim__get_runtime".as_ptr());
            lua_setfield(lstate, -2, c"api".as_ptr());
        }
        lua_setglobal(lstate, c"vim".as_ptr());

        nlua_init_packages(lstate, in_script.get());

        // package.loaded.vim = vim
        lua_getglobal(lstate, c"package".as_ptr());
        lua_getfield(lstate, -1, c"loaded".as_ptr());
        lua_getglobal(lstate, c"vim".as_ptr());
        lua_setfield(lstate, -2, c"vim".as_ptr());
        lua_pop(lstate, 2);

        lstate
    }
}

/// luv's `release_vm`: release the two sentinels and close the state.
///
/// # Safety
/// `lstate` must be a state [`nlua_init_state`] built, and unused afterwards.
unsafe extern "C-unwind" fn nlua_common_free_all_mem(lstate: *mut lua_State) {
    unsafe {
        let ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
        nlua_unref(lstate, ref_state, (*ref_state).nil_ref);
        nlua_unref(lstate, ref_state, (*ref_state).empty_dict_ref);
        lua_close(lstate);
    }
}

/// `require('vim._core.defaults')`, run once the editor is far enough along
/// to have options and mappings.
///
/// # Safety
/// The main state must exist.
pub unsafe fn nlua_init_defaults() {
    unsafe {
        let lstate = global_lstate.get();
        debug_assert!(!lstate.is_null());
        lua_getglobal(lstate, c"require".as_ptr());
        lua_pushstring(lstate, c"vim._core.defaults".as_ptr());
        if nlua_pcall(lstate, 1, 0) != 0 {
            fprintf(stderr, c"%s\n".as_ptr(), lua_tostring(lstate, -1));
        }
    }
}
