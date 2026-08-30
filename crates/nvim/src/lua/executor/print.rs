//! `print()`, `require()` and `vim.debug()`.
//!
//! Neovim replaces Lua's `print` so its output goes through the editor's
//! message path rather than stdout, and defers it when called from a fast
//! callback (`nlua_print_event`).  [`nlua_require`] is the wrapper that keeps
//! `package.loaded` and the runtime path in step, and [`nlua_debug`] is the
//! `vim.debug()` read-eval-print loop.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{in_fast_callback, nlua_error, nlua_pcall, nlua_pushref, require_ref};
use crate::eval::typval::{TV_INITIAL_VALUE, tv_clear};
use crate::event::r#loop::loop_schedule_deferred;
use crate::event::multiqueue::multiqueue_put_event;
use crate::ex_getln::{get_user_input, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave};
use crate::garray::{ga_append, ga_clear, ga_concat_len, ga_init};
use crate::lua::ffi::{
    LUA_GLOBALSINDEX, LUA_REGISTRYINDEX, lua_call, lua_error, lua_getfield, lua_getglobal,
    lua_gettop, lua_insert, lua_iscfunction, lua_pcall, lua_pop, lua_pushlstring, lua_pushvalue,
    lua_setfield, lua_settop, lua_toboolean, lua_tocfunction, lua_tolstring, luaL_checkstring,
    luaL_loadbuffer,
};
use crate::main::{main_loop, time_fd};
use crate::memory::{xfree, xmalloc, xrealloc};
use crate::message::{msg_multihl, msg_putchar};
use crate::os::cshim::{gettext, snprintf};
use crate::profile::{time_msg, time_pop, time_push};
use crate::strings::vim_snprintf;
use crate::types::ui::kUICmdline;
use crate::types::{
    Event, HlMessage, HlMessageChunk, IOSIZE, MessageData, Object, String_0, VAR_STRING, VarLock,
    intptr_t, lua_State, proftime_T, size_t, typval_T, typval_vval_union, uint8_t,
};
use crate::ui::ui_has;
use ::libc::strlen;

/// The message kind `print()`'s output is reported under.
const LUA_PRINT_KIND: &CStr = c"lua_print";
/// How far the message garray grows at a time.
const MSG_GROWSIZE: c_int = 80;

/// Show one `print()`'s worth of text.
///
/// `argv[0]` is the NUL-terminated buffer `nlua_print` built and `argv[1]`
/// its length *including* that terminator, which the chunk excludes again.
///
/// # Safety
/// `argv` must be the two-element array `nlua_print` built.
unsafe extern "C" fn nlua_print_event(argv: *mut *mut c_void) {
    unsafe {
        let chunk = HlMessageChunk {
            text: String_0::from_raw_parts(
                (*argv.add(0)).cast::<c_char>(),
                ((*argv.add(1)).expose_provenance() as intptr_t as size_t).wrapping_sub(1),
            ),
            hl_id: 0,
        };
        // One chunk, so one allocation: upstream's `kv_push` onto an empty
        // `HlMessage`.
        let mut msg = HlMessage {
            size: 1,
            capacity: 8,
            items: xrealloc(ptr::null_mut(), size_of::<HlMessageChunk>() * 8)
                .cast::<HlMessageChunk>(),
        };
        *msg.items = chunk;

        let mut needs_clear = false;
        msg_multihl(
            Object::NIL,
            msg,
            LUA_PRINT_KIND.as_ptr(),
            true,
            false,
            ptr::null_mut::<MessageData>(),
            &raw mut needs_clear,
        );
    }
}

/// Neovim's `print()`: every argument through `tostring`, space-separated,
/// through the message path.
///
/// A thread state and a fast callback both defer the message onto the main
/// loop; everything else shows it directly. Either way the buffer's
/// ownership passes to whoever shows it.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_print(lstate: *mut lua_State) -> c_int {
    unsafe {
        let nargs = lua_gettop(lstate);
        lua_getglobal(lstate, c"tostring".as_ptr());

        let mut errmsg: *const c_char = ptr::null();
        let mut errmsg_len: size_t = 0;
        let mut msg_ga = GA_EMPTY;
        ga_init(&raw mut msg_ga, 1, MSG_GROWSIZE);

        let mut curargidx: c_int = 1;
        'nlua_print_error: {
            while curargidx <= nargs {
                lua_pushvalue(lstate, -1); // tostring
                lua_pushvalue(lstate, curargidx);
                if lua_pcall(lstate, 1, 1, 0) != 0 {
                    errmsg = lua_tolstring(lstate, -1, &raw mut errmsg_len);
                    break 'nlua_print_error;
                }
                let mut len: size_t = 0;
                let s = lua_tolstring(lstate, -1, &raw mut len);
                if s.is_null() {
                    errmsg = NULL_TOSTRING.as_ptr();
                    errmsg_len = NULL_TOSTRING.count_bytes();
                    break 'nlua_print_error;
                }
                ga_concat_len(&raw mut msg_ga, s, len);
                if curargidx < nargs {
                    ga_append(&raw mut msg_ga, b' ');
                }
                lua_pop(lstate, 1);
                curargidx += 1;
            }
            ga_append(&raw mut msg_ga, 0 as uint8_t);

            lua_getfield(lstate, LUA_REGISTRYINDEX, c"nvim.thread".as_ptr());
            let is_thread = lua_toboolean(lstate, -1) != 0;
            lua_pop(lstate, 1);

            let mut args = [
                msg_ga.ga_data,
                ptr::with_exposed_provenance_mut::<c_void>(msg_ga.ga_len as intptr_t as usize),
            ];
            if is_thread {
                loop_schedule_deferred(main_loop.ptr(), Event::new(Some(nlua_print_event), args));
            } else if in_fast_callback.get() != 0 {
                multiqueue_put_event(
                    (*main_loop.ptr()).events,
                    Event::new(Some(nlua_print_event), args),
                );
            } else {
                nlua_print_event(args.as_mut_ptr());
            }
            return 0;
        }

        // The conversion failed: nothing is shown, and the failure is thrown
        // at the Lua caller instead.
        ga_clear(&raw mut msg_ga);
        let buff = xmalloc(IOSIZE as size_t).cast::<c_char>();
        let fmt = gettext(c"E5114: Converting print argument #%i: %.*s");
        let len = vim_snprintf(
            buff,
            IOSIZE as size_t,
            fmt.as_ptr(),
            curargidx,
            errmsg_len as c_int,
            errmsg,
        ) as size_t;
        lua_pushlstring(lstate, buff, len);
        xfree(buff.cast::<c_void>());
        lua_error(lstate)
    }
}

/// Neovim's `require()`: the stock one, wrapped so that `--startuptime`
/// records how long each module took.
///
/// A module already in `package.loaded` short-circuits, and the wrapper
/// re-installs itself as the global `require` if something replaced it.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_require(lstate: *mut lua_State) -> c_int {
    let mut what = [0 as c_char; IOSIZE as usize];
    unsafe {
        let name = luaL_checkstring(lstate, 1);
        lua_settop(lstate, 1);
        lua_getfield(lstate, LUA_REGISTRYINDEX, c"_LOADED".as_ptr());
        lua_getfield(lstate, 2, name);
        if lua_toboolean(lstate, -1) != 0 {
            return 1;
        }
        lua_pop(lstate, 2);

        nlua_pushref(lstate, require_ref.get());
        lua_insert(lstate, 1);

        if time_fd.get().is_null() {
            // Not profiling: hand straight through, restoring the global
            // `require` if the stock one has taken it back.
            lua_getglobal(lstate, c"require".as_ptr());
            let is_this_wrapper = lua_iscfunction(lstate, -1) != 0
                && lua_tocfunction(lstate, -1)
                    .is_some_and(|f| ptr::fn_addr_eq(f, nlua_require as CFunction));
            if is_this_wrapper {
                lua_pushvalue(lstate, 1);
                lua_setfield(lstate, LUA_GLOBALSINDEX, c"require".as_ptr());
            }
            lua_pop(lstate, 1);
            lua_call(lstate, 1, 1);
            return 1;
        }

        let (rel_time, mut start_time): (proftime_T, proftime_T) = time_push();
        let status = lua_pcall(lstate, 1, 1, 0);
        if status == 0 {
            vim_snprintf(
                what.as_mut_ptr(),
                IOSIZE as size_t,
                c"require('%s')".as_ptr(),
                name,
            );
            time_msg(what.as_ptr(), &raw mut start_time);
        }
        time_pop(rel_time);
        if status == 0 { 1 } else { lua_error(lstate) }
    }
}

/// `vim.debug()`: read a line, run it, repeat until `cont` or an empty line.
///
/// # Safety
/// `lstate` must be a live Lua state.
pub(crate) unsafe extern "C-unwind" fn nlua_debug(lstate: *mut lua_State) -> c_int {
    let mut line = [0 as c_char; IOSIZE as usize];
    unsafe {
        let input_args: [typval_T; 2] = [
            typval_T {
                v_type: VAR_STRING,
                v_lock: VarLock::Fixed,
                vval: typval_vval_union {
                    v_string: c"lua_debug> ".as_ptr().cast_mut(),
                },
            },
            TV_INITIAL_VALUE,
        ];
        loop {
            lua_settop(lstate, 0);
            let mut input = TV_INITIAL_VALUE;
            get_user_input(input_args.as_ptr(), &raw mut input, false, false);

            if ui_has(kUICmdline) {
                snprintf(
                    line.as_mut_ptr(),
                    IOSIZE as size_t,
                    c"lua_debug> %s".as_ptr(),
                    input.vval.v_string,
                );
                ui_ext_cmdline_block_append(0, line.as_ptr());
            } else {
                msg_putchar(b'\n' as c_int);
            }

            let done = input.v_type != VAR_STRING
                || input.vval.v_string.is_null()
                || *input.vval.v_string == 0
                || cstr::bytes_at(input.vval.v_string) == b"cont";
            if done {
                tv_clear(&raw mut input);
                if ui_has(kUICmdline) {
                    ui_ext_cmdline_block_leave();
                }
                return 0;
            }

            if luaL_loadbuffer(
                lstate,
                input.vval.v_string,
                strlen(input.vval.v_string),
                c"=(debug command)".as_ptr(),
            ) != 0
            {
                nlua_error(
                    lstate,
                    gettext(c"E5115: Loading Lua debug string: %.*s").as_ptr(),
                );
            } else if nlua_pcall(lstate, 0, 0) != 0 {
                nlua_error(
                    lstate,
                    gettext(c"E5116: Calling Lua debug string: %.*s").as_ptr(),
                );
            }
            tv_clear(&raw mut input);
        }
    }
}

/// What `print()` reports when `tostring` answers something that is not a
/// string at all.
const NULL_TOSTRING: &CStr = c"<Unknown error: lua_tolstring returned NULL for tostring result>";

/// A garray `ga_init` is about to fill.
const GA_EMPTY: crate::types::garray_T = crate::types::garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ptr::null_mut(),
};

/// The signature `lua_tocfunction` answers with.
type CFunction = unsafe extern "C-unwind" fn(*mut lua_State) -> c_int;
