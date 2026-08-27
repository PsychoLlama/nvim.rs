//! `:lua`, `:luado`, `:luafile` and sourcing a file.
//!
//! [`ex_luado`] is the one with a shape of its own: it compiles the body once
//! into a function of `(line, linenr)` and runs it over the range, replacing
//! each line by what the function returns.  [`nlua_exec_file`] is what
//! `:luafile` and the runtime loader both reach.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{get_global_lstate, nlua_error, nlua_pcall, nlua_typval_exec};
use crate::change::inserted_bytes;
use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::ex_getln::script_get;
use crate::lua::ffi::{
    LUA_TNIL, lua_getglobal, lua_isnil, lua_isstring, lua_pop, lua_pushnumber, lua_pushstring,
    lua_pushvalue, lua_tolstring, lua_type, luaL_loadbuffer,
};
use crate::main::{curbuf, e_argreq, got_int};
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_replace};
use crate::memory::{strequal, xfree, xmalloc, xmallocz, xmemdupz, xrealloc};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::fileio::{file_close, file_open_stdin};
use crate::runtime::cmd_source_buffer;
use crate::strings::vim_snprintf;
use crate::types::{
    CMD_equal, FAIL, FileDescriptor, IOSIZE, buf_T, colnr_T, exarg_T, linenr_T, lua_Number, size_t,
    typval_T,
};
use crate::undo::u_save;
use ::libc::{memcpy, strlen};

/// The wrapper `:luado`'s body is compiled inside, so each line is one call.
const DOSTART: &CStr = c"return function(line, linenr) ";
const DOEND: &CStr = c" end";

/// `:lua =expr` and `:= expr` are shorthand for this.
const PRINT_WRAPPER: &CStr = c"vim._print(true, %s)";

/// How much [`nlua_exec_file`] reads from stdin at a time.
const STDIN_CHUNK: size_t = 64;

/// `:lua {chunk}`, `:lua ={expr}` and `:={expr}`.
///
/// # Safety
/// `eap` must be a live command argument block.
pub unsafe fn ex_lua(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg == 0 {
            // `:{range}lua` with no body sources the range as Lua.
            if (*eap).addr_count > 0 {
                cmd_source_buffer(eap, true);
            } else {
                emsg(gettext(&raw const e_argreq as *const _));
            }
            return;
        }

        let mut len: size_t = 0;
        let mut code = script_get(eap, &raw mut len);
        if (*eap).skip != 0 || code.is_null() {
            xfree(code.cast::<c_void>());
            return;
        }

        if (*eap).cmdidx == CMD_equal || *code == b'=' as c_char {
            // `:=expr` has no `=` to skip; `:lua =expr` does.
            let off: size_t = if (*eap).cmdidx == CMD_equal { 0 } else { 1 };
            len += PRINT_WRAPPER.count_bytes() - 2 - off;
            let code_buf = xmallocz(len).cast::<c_char>();
            vim_snprintf(code_buf, len + 1, PRINT_WRAPPER.as_ptr(), code.add(off));
            xfree(code.cast::<c_void>());
            code = code_buf;
        }

        nlua_typval_exec(
            code,
            len,
            c":lua".as_ptr(),
            ptr::null_mut::<typval_T>(),
            0,
            false,
            ptr::null_mut::<typval_T>(),
        );
        xfree(code.cast::<c_void>());
    }
}

/// `:luado {body}`: run `body` over each line of the range as
/// `function(line, linenr)`, replacing the line with a string result.
///
/// The loop stops early if the body changed buffers or shortened the one it
/// is walking, which is the only protection against it editing under itself.
/// A NUL in the result stands for a newline, as `:s` treats it.
///
/// # Safety
/// `eap` must be a live command argument block.
pub unsafe fn ex_luado(eap: *mut exarg_T) {
    // Where the wrapped chunk is assembled when it fits; upstream shares
    // `IObuff` for it, which the loop body may overwrite.
    let mut chunk = [0 as c_char; IOSIZE as usize];
    unsafe {
        if u_save((*eap).line1 - 1, (*eap).line2 + 1) == FAIL {
            emsg(gettext(c"cannot save undo information".as_ptr()));
            return;
        }
        let cmd = (*eap).arg;
        let cmd_len = strlen(cmd);
        let lstate = get_global_lstate();

        let head = DOSTART.count_bytes();
        let tail = DOEND.count_bytes();
        let lcmd_len = cmd_len + head + tail;
        // Not `chunk_buffer`'s rule: this one allocates one byte more, which
        // is upstream's own asymmetry.
        let lcmd = if lcmd_len < IOSIZE as size_t {
            chunk.as_mut_ptr()
        } else {
            xmalloc(lcmd_len + 1).cast::<c_char>()
        };
        memcpy(lcmd.cast::<c_void>(), DOSTART.as_ptr().cast(), head);
        memcpy(lcmd.add(head).cast::<c_void>(), cmd.cast(), cmd_len);
        memcpy(
            lcmd.add(head + cmd_len).cast::<c_void>(),
            DOEND.as_ptr().cast(),
            tail,
        );

        let loaded = luaL_loadbuffer(lstate, lcmd, lcmd_len, c":luado".as_ptr());
        if lcmd_len >= IOSIZE as size_t {
            xfree(lcmd.cast::<c_void>());
        }
        if loaded != 0 {
            nlua_error(lstate, gettext(c"E5109: Lua: %.*s".as_ptr()));
            return;
        }
        if nlua_pcall(lstate, 0, 1) != 0 {
            nlua_error(lstate, gettext(c"E5110: Lua: %.*s".as_ptr()));
            return;
        }

        let was_curbuf: *mut buf_T = curbuf.get();
        let mut l: linenr_T = (*eap).line1;
        while l <= (*eap).line2 {
            if l > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            lua_pushvalue(lstate, -1);
            let old_line = ml_get_buf(curbuf.get(), l);
            let old_line_len = ml_get_buf_len(curbuf.get(), l);
            lua_pushstring(lstate, old_line);
            lua_pushnumber(lstate, l as lua_Number);
            if nlua_pcall(lstate, 2, 1) != 0 {
                nlua_error(lstate, gettext(c"E5111: Lua: %.*s".as_ptr()));
                break;
            }
            if curbuf.get() != was_curbuf || l > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            if lua_isstring(lstate, -1) != 0 {
                let mut new_line_len: size_t = 0;
                let new_line = lua_tolstring(lstate, -1, &raw mut new_line_len);
                let new_line_transformed =
                    xmemdupz(new_line.cast::<c_void>(), new_line_len).cast::<c_char>();
                for i in 0..new_line_len {
                    if *new_line_transformed.add(i) == 0 {
                        *new_line_transformed.add(i) = b'\n' as c_char;
                    }
                }
                ml_replace(l, new_line_transformed, false);
                inserted_bytes(l, 0 as colnr_T, old_line_len, new_line_len as c_int);
            }
            lua_pop(lstate, 1);
            l += 1;
        }
        lua_pop(lstate, 1);
        check_cursor(Win::current());
        redraw_curbuf_later(UPD_NOT_VALID);
    }
}

/// `:luafile {path}`.
///
/// # Safety
/// `eap` must be a live command argument block.
pub unsafe fn ex_luafile(eap: *mut exarg_T) {
    unsafe {
        nlua_exec_file((*eap).arg);
    }
}

/// Read the whole of stdin, NUL-terminated. `None` on a read failure or an
/// interrupt, with the buffer already freed.
///
/// # Safety
/// Stdin must be openable; the answer is the caller's to free.
unsafe fn read_stdin() -> Option<*mut c_char> {
    let mut chunk = [0 as c_char; STDIN_CHUNK];
    unsafe {
        let mut stdin_dup = FILE_DESCRIPTOR_INIT;
        if file_open_stdin(&raw mut stdin_dup) != 0 {
            return None;
        }
        let mut sb = StringBuf::with_capacity(STDIN_CHUNK);
        loop {
            if got_int.get() {
                file_close(&raw mut stdin_dup, false);
                sb.free();
                return None;
            }
            let read_size = file_read(&raw mut stdin_dup, chunk.as_mut_ptr(), STDIN_CHUNK);
            if read_size < 0 {
                file_close(&raw mut stdin_dup, false);
                sb.free();
                return None;
            }
            if read_size > 0 {
                sb.extend(chunk.as_ptr(), read_size as size_t);
            }
            if (read_size as size_t) < STDIN_CHUNK {
                break;
            }
        }
        sb.push(0);
        file_close(&raw mut stdin_dup, false);
        Some(sb.into_raw())
    }
}

/// Run the Lua file at `path`, or stdin for `-`.
///
/// # Safety
/// `path` must be a NUL-terminated path.
pub unsafe fn nlua_exec_file(path: *const c_char) -> bool {
    unsafe {
        let lstate = get_global_lstate();
        if !strequal(path, c"-".as_ptr()) {
            lua_getglobal(lstate, c"loadfile".as_ptr());
            lua_pushstring(lstate, path);
        } else {
            let Some(text) = read_stdin() else {
                return false;
            };
            lua_getglobal(lstate, c"loadstring".as_ptr());
            lua_pushstring(lstate, text);
            xfree(text.cast::<c_void>());
        }

        // `loadfile`/`loadstring` answer either the chunk and nil, or nil and
        // the syntax error.
        if nlua_pcall(lstate, 1, 2) != 0 {
            nlua_error(lstate, gettext(c"E5111: Lua: %.*s".as_ptr()));
            return false;
        }
        if lua_type(lstate, -2) == LUA_TNIL {
            nlua_error(lstate, gettext(c"E5112: Lua chunk: %.*s".as_ptr()));
            debug_assert!(lua_isnil(lstate, -1));
            lua_pop(lstate, 1);
            return false;
        }
        debug_assert!(lua_isnil(lstate, -1));
        lua_pop(lstate, 1);
        if nlua_pcall(lstate, 0, 0) != 0 {
            nlua_error(lstate, gettext(c"E5113: Lua chunk: %.*s".as_ptr()));
            return false;
        }
        true
    }
}

/// An unopened [`FileDescriptor`], which `file_open_stdin` fills.
const FILE_DESCRIPTOR_INIT: FileDescriptor = FileDescriptor {
    fd: 0,
    buffer: ptr::null_mut(),
    read_pos: ptr::null_mut(),
    write_pos: ptr::null_mut(),
    wr: false,
    eof: false,
    non_blocking: false,
    bytes_read: 0,
};

/// klib's `StringBuilder` as the growing byte buffer it is.
///
/// Not a `Vec`: the buffer is handed to `xfree` and its `items` field is what
/// upstream's `kv_*` macros expanded to, so the allocation has to be the
/// editor's own.
struct StringBuf(StringBuilder);

impl StringBuf {
    /// # Safety
    /// Allocates; the caller must eventually [`Self::free`] or
    /// [`Self::into_raw`].
    unsafe fn with_capacity(capacity: size_t) -> Self {
        unsafe {
            Self(StringBuilder {
                size: 0,
                capacity,
                items: xrealloc(ptr::null_mut(), capacity).cast::<c_char>(),
            })
        }
    }

    /// Append `len` bytes, growing to the next power of two that fits.
    ///
    /// # Safety
    /// `src` must point at `len` readable bytes.
    unsafe fn extend(&mut self, src: *const c_char, len: size_t) {
        unsafe {
            if len == 0 {
                return;
            }
            let sb = &mut self.0;
            if sb.capacity < sb.size + len {
                sb.capacity = (sb.size + len).next_power_of_two();
                sb.items = xrealloc(sb.items.cast::<c_void>(), sb.capacity).cast::<c_char>();
            }
            debug_assert!(!sb.items.is_null());
            memcpy(sb.items.add(sb.size).cast::<c_void>(), src.cast(), len);
            sb.size += len;
        }
    }

    /// Append one byte.
    ///
    /// # Safety
    /// As [`Self::extend`].
    unsafe fn push(&mut self, byte: c_char) {
        unsafe {
            let sb = &mut self.0;
            if sb.size == sb.capacity {
                sb.capacity = if sb.capacity != 0 {
                    sb.capacity << 1
                } else {
                    8
                };
                sb.items = xrealloc(sb.items.cast::<c_void>(), sb.capacity).cast::<c_char>();
            }
            *sb.items.add(sb.size) = byte;
            sb.size += 1;
        }
    }

    /// # Safety
    /// The buffer must not be used again.
    unsafe fn free(&mut self) {
        unsafe { xfree(self.0.items.cast::<c_void>()) };
        self.0 = StringBuilder {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        };
    }

    /// Hand the buffer to the caller, which frees it.
    fn into_raw(self) -> *mut c_char {
        self.0.items
    }
}

use crate::os::fileio::file_read;
use crate::types::StringBuilder;
use crate::winlayer::Win;
