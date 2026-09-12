//! `system()` and `systemlist()`: the argument vector and the captured
//! output.
//!
//! Both directions swap NUL and newline. A shell's stdin is a byte stream
//! with no way to carry a NUL, so `save_tv_as_string` writes a newline for
//! every NUL a List item held and vice versa; `get_system_output_as_rettv`
//! undoes it on the way back. That is why the two halves look asymmetric:
//! one builds a buffer, the other rewrites one in place.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::semsg;
use crate::smsg;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::{null, null_mut};

use crate::buffer::find_buf;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    ListRef, NumBuf, tv_get_number, tv_list_alloc, tv_list_alloc_ret, tv_list_first, tv_list_iter,
    tv_list_len,
};
use crate::eval::vars::emsg_static;
use crate::eval::vars::set_vim_var_nr;
use crate::eval::{NL, PROF_YES, Tv};
use crate::ex_cmds::check_secure;
use crate::memline::ml_get_buf;
use crate::memory::{memchrsub, xcalloc, xfree, xmalloc, xmemdupz, xstrdup};
use crate::message::e_invarg;
use crate::message::{msg_str, verbose_enter_scroll, verbose_leave_scroll};
use crate::message_fmt::c_str;
use crate::option::vars::p_verbose;
use crate::os::cshim::snprintf;
use crate::os::fs::os_can_exe;
use crate::os::shell::{os_system, shell_argv_to_str, shell_build_argv, shell_free_argv};
use crate::profile::do_profiling;
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::types::{
    EvalFuncData, IOSIZE, List, NUL, OptInt, ProfTime, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarNumber, Vv, kListLenMayKnow, ptrdiff_t, size_t,
};

/// Build a `NULL`-terminated argument vector out of a String (through the
/// shell) or a List (directly). `cmd`, when given, comes back naming the
/// executable; `executable` is cleared when the first item is not one.
///
/// # Safety
/// `cmd_tv` must be valid; `cmd` and `executable` null or valid. `numbuf` is
/// the scratch a Number command is spelled into and must outlive `*cmd`,
/// which may point into it.
pub unsafe fn tv_to_argv(
    cmd_tv: &TypVal,
    cmd: *mut *const c_char,
    executable: *mut bool,
    numbuf: &mut NumBuf,
) -> *mut *mut c_char {
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    // SAFETY: the caller's promise -- the typval outlives the call.
    let tv = cmd_tv;
    if tv.v_type() == VAR_STRING {
        // SAFETY: `numbuf` is the caller's scratch, which outlives `*cmd`.
        let cmd_str = unsafe { numbuf.string(cmd_tv) };
        if !cmd.is_null() {
            // SAFETY: the caller's promise -- a non-null `cmd` is valid.
            unsafe { *cmd = cmd_str };
        }
        // SAFETY: `cmd_str` is NUL-terminated.
        return unsafe { shell_build_argv(cmd_str, null::<c_char>()) };
    }
    if tv.v_type() != VAR_LIST {
        let what = c"expected String or List".as_ptr();
        // SAFETY: the format takes one NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return null_mut();
    }

    let argl: *mut List = tv.list_or_null();
    // SAFETY: `argl` is a live List or null.
    let argc = unsafe { tv_list_len(argl) };
    if argc == 0 {
        // SAFETY: `e_invarg` is a shared NUL-terminated message.
        emsg_static(e_invarg);
        return null_mut();
    }

    // The first item has to resolve to something runnable, and the
    // resolved path is what actually goes in slot 0.
    // SAFETY: a non-empty List has a first item, and `numbuf2` outlives
    // the string rendered into it.
    let arg0 = unsafe { numbuf2.string_chk(&(*tv_list_first(argl)).li_tv) };
    let mut exe_resolved: *mut c_char = null_mut();
    // SAFETY: `arg0` is NUL-terminated and `exe_resolved` is this frame's.
    let runnable =
        !arg0.is_null() && unsafe { os_can_exe(cstr::at(arg0), &raw mut exe_resolved, true) };
    if !runnable {
        if !arg0.is_null() && !executable.is_null() {
            let mut buf: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];
            let size = size_of::<[c_char; IOSIZE as usize]>();
            let fmt = c"'%s' is not executable".as_ptr();
            // SAFETY: `buf` is this frame's and `size` is its length; the
            // format takes the one NUL-terminated string `arg0`.
            unsafe { snprintf(buf.as_mut_ptr(), size, fmt, arg0) };
            let (what, text) = (c"cmd".as_ptr(), buf.as_mut_ptr());
            // SAFETY: the format takes two NUL-terminated strings.
            let (what, text) = unsafe { (c_str(what), c_str(text)) };
            semsg!("E475: Invalid value for argument {what}: {text}");
            // SAFETY: the caller's promise -- a non-null `executable`.
            unsafe { *executable = false };
        }
        return null_mut();
    }
    if !cmd.is_null() {
        // SAFETY: the caller's promise -- a non-null `cmd` is valid.
        unsafe { *cmd = exe_resolved };
    }

    let slots = argc as size_t + 1;
    // SAFETY: `xcalloc` never answers NULL, and the last of the `argc + 1`
    // zeroed slots stays the vector's NULL terminator.
    let argv = unsafe { xcalloc(slots, size_of::<*mut c_char>()) } as *mut *mut c_char;
    let mut i = 0;
    if !argl.is_null() {
        // SAFETY: `argl` is a live List.
        for arg in tv_list_iter(unsafe { argl.as_ref() }) {
            // SAFETY: `arg` is one of the List's items, and `numbuf3`
            // outlives the string rendered into it.
            let a = unsafe { numbuf3.string_chk(&arg.li_tv) };
            if a.is_null() {
                // SAFETY: `argv` holds `i` owned strings and a NULL tail.
                unsafe { shell_free_argv(argv) };
                // SAFETY: `exe_resolved` is the owned path from above.
                unsafe { xfree(exe_resolved as *mut c_void) };
                return null_mut();
            }
            // SAFETY: the List has `argc` items, so slot `i` is inside the
            // vector; `a` is NUL-terminated.
            unsafe { *argv.offset(i) = xstrdup(a) };
            i += 1;
        }
    }
    // Slot 0 holds the item's own spelling; swap in the resolved path.
    // SAFETY: slot 0 was written above, and nothing else owns it.
    unsafe { xfree(*argv as *mut c_void) };
    // SAFETY: as above.
    unsafe { *argv = exe_resolved };
    argv
}

/// Split captured output into a List of lines, undoing the NUL/newline
/// swap on the way.
///
/// # Safety
/// `str` must hold `len` readable bytes.
pub(crate) unsafe fn string_to_list(
    str: *const c_char,
    mut len: size_t,
    keepempty: bool,
) -> ListRef {
    // A trailing newline does not start an empty last line unless the
    // caller asked to keep one.
    // SAFETY: the caller's promise -- `len` bytes are readable, so the
    // last one is.
    if !keepempty && unsafe { *str.add(len - 1) } as c_int == NL {
        len -= 1;
    }
    let list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    // SAFETY: as above; `str` has `len` readable bytes.
    unsafe { encode_list_write(list.as_ptr() as *mut c_void, str, len) };
    list
}

/// The shared body of `system()` and `systemlist()`.
pub(crate) fn get_system_output_as_rettv(args: &[TypVal], result: &mut TypVal, retlist: bool) {
    let mut cmdbuf = NumBuf::new();
    let profiling = do_profiling.get() == PROF_YES;
    // SAFETY: the caller's promise -- `result` outlives the call.
    let mut ret = unsafe { Tv::new(result) };
    ret.write_string(null_mut());
    if check_secure() {
        return;
    }

    let mut input_len: ptrdiff_t = 0;
    // SAFETY: an argument is a live typval; with no input argument there is
    // nothing to feed the command.
    let input = match args.get(1) {
        Some(tv) => unsafe { save_tv_as_string(tv, &raw mut input_len, false, false) },
        None => null_mut(),
    };
    if input_len < 0 {
        debug_assert!(input.is_null());
        return;
    }

    let mut executable = true;
    // SAFETY: `args` is the builtin's own vector, and `cmdbuf` outlives
    // the argv a Number command is spelled into.
    let argv = unsafe { tv_to_argv(&args[0], null_mut(), &raw mut executable, &mut cmdbuf) };
    if argv.is_null() {
        // A command that does not exist reports -1 rather than a shell
        // exit status.
        if !executable {
            set_vim_var_nr(Vv::ShellError, -1);
        }
        // SAFETY: the input buffer is owned here.
        unsafe { xfree(input as *mut c_void) };
        return;
    }

    if p_verbose.get() > 3 as OptInt {
        // SAFETY: `argv` is the NULL-terminated vector built above.
        let cmdstr = unsafe { shell_argv_to_str(argv) };
        verbose_enter_scroll();
        // SAFETY: the format takes the one NUL-terminated `cmdstr`.
        let shown = unsafe { c_str(cmdstr) };
        smsg!(0, "Executing command: \"{shown}\"");
        // SAFETY: the literal is NUL-terminated.
        msg_str(c"\n\n");
        verbose_leave_scroll();
        // SAFETY: `cmdstr` is the owned rendering.
        unsafe { xfree(cmdstr as *mut c_void) };
    }

    let mut wait_time: ProfTime = 0;
    if profiling {
        // SAFETY: the profile clock is the editor's own.
        wait_time = unsafe { prof_child_enter() };
    }
    let mut nread: size_t = 0;
    let mut res: *mut c_char = null_mut();
    let ilen = input_len as size_t;
    // SAFETY: `argv` is the vector built above, `input` its `ilen` bytes of
    // standard input, and the two out-parameters are this frame's.
    let status = unsafe { os_system(argv, input, ilen, &raw mut res, &raw mut nread) };
    if profiling {
        // SAFETY: paired with the `prof_child_enter` above.
        unsafe { prof_child_exit(wait_time) };
    }
    // SAFETY: the child has read it, and the buffer is owned here.
    unsafe { xfree(input as *mut c_void) };
    set_vim_var_nr(Vv::ShellError, status as VarNumber);

    if res.is_null() {
        if retlist {
            // SAFETY: `result` is the caller's.
            tv_list_alloc_ret(result, 0 as ptrdiff_t);
        } else {
            // SAFETY: the literal is NUL-terminated.
            ret.write_string(unsafe { xstrdup(c"".as_ptr()) });
        }
        return;
    }

    if retlist {
        // The `keepempty` argument is the third, so it is only read
        // when the second was given too.
        let mut keepempty = 0;
        if args.len() > 2 {
            keepempty = tv_get_number(&args[2]) as c_int;
        }
        // SAFETY: `res` holds `nread` readable bytes.
        ret.write_list(Some(unsafe { string_to_list(res, nread, keepempty != 0) }));
        // SAFETY: the encoder copied what it needed.
        unsafe { xfree(res as *mut c_void) };
    } else {
        // Undo the swap in place; the buffer is handed over as it is.
        // SAFETY: `res` holds `nread` writable bytes.
        unsafe { memchrsub(res as *mut c_void, NUL as c_char, 1 as c_char, nread) };
        ret.write_string(res);
    }
}

/// `system()`
pub fn f_system(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_system_output_as_rettv(args, result, false)
}

/// `systemlist()`
pub fn f_systemlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_system_output_as_rettv(args, result, true)
}

/// Write `c` at `dest` and answer the byte after it.
///
/// # Safety
/// `dest` must have room for one more byte.
#[inline(always)]
unsafe fn put(dest: *mut c_char, c: c_char) -> *mut c_char {
    // SAFETY: the caller's promise -- one writable byte at `dest`.
    unsafe {
        *dest = c;
        dest.add(1)
    }
}

/// Copy the NUL-terminated string at `src` to `dest`, writing a NUL for
/// every newline it holds, and answer the end of what was written.
///
/// The swap is the module's convention: a child's standard input is a byte
/// stream with no way to carry a NUL, so the two trade places on the way
/// out and the reading half puts them back.
///
/// # Safety
/// `src` must be NUL-terminated, and `dest` must have room for its bytes.
unsafe fn copy_swapping_nl(src: *const c_char, dest: *mut c_char) -> *mut c_char {
    let (mut src, mut dest) = (src, dest);
    loop {
        // SAFETY: the caller's promise -- the walk stops at the terminator,
        // so it never leaves the string.
        let c = unsafe { *src };
        if c as c_int == NUL {
            return dest;
        }
        let out = if c == b'\n' as c_char {
            NUL as c_char
        } else {
            c
        };
        // SAFETY: the caller's promise -- one byte written per byte read.
        dest = unsafe { put(dest, out) };
        // SAFETY: `c` was not the terminator, so the next byte is inside.
        src = unsafe { src.add(1) };
    }
}

/// Render a typval as the byte stream a child process's stdin wants: a
/// String as it is, a Number as that buffer's whole text, a List one item
/// per line. `len` comes back -1 for a coercion that failed.
///
/// Newlines in the text become NULs and the line separators are newlines,
/// which is the convention the reading half undoes.
///
/// # Safety
/// `tv` and `len` must be valid.
pub unsafe fn save_tv_as_string(
    tv: &TypVal,
    len: *mut ptrdiff_t,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- both outlive the call.
    let value = unsafe { Tv::new(::core::ptr::from_ref(tv).cast_mut()) };
    // SAFETY: as above.
    unsafe { *len = 0 };
    if value.v_type() == VAR_UNKNOWN {
        return null_mut();
    }
    if value.v_type() != VAR_LIST && value.v_type() != VAR_NUMBER {
        // SAFETY: `numbuf` outlives the string rendered into it.
        let ret = unsafe { numbuf.string_chk(tv) };
        if ret.is_null() {
            // SAFETY: the caller's promise about `len`.
            unsafe { *len = -1 };
            return null_mut();
        }
        // SAFETY: `ret` is NUL-terminated, and `len` is the caller's.
        unsafe { *len = cstr::bytes_at(ret).len() as ptrdiff_t };
        // SAFETY: `ret` has the `*len` bytes just measured.
        return unsafe { xmemdupz(ret as *const c_void, *len as size_t) as *mut c_char };
    }
    if value.v_type() == VAR_NUMBER {
        // SAFETY: a `VAR_NUMBER`, which is what the callee wants.
        return unsafe { buffer_as_string(tv, len) };
    }
    // SAFETY: `VAR_LIST` says the value holds a List.
    unsafe { list_as_string(value.list_or_null(), len, endnl, crlf) }
}

/// A Number names a buffer; its whole text is the input.
///
/// # Safety
/// `tv` must be a `VAR_NUMBER`; `len` valid.
unsafe fn buffer_as_string(tv: &TypVal, len: *mut ptrdiff_t) -> *mut c_char {
    // A `VAR_NUMBER`, so the value holds a buffer number.
    let nr = tv.number_or_zero();
    let Some(buf) = find_buf(nr as c_int) else {
        semsg!("E86: Buffer {} does not exist", nr);
        // SAFETY: the caller's promise about `len`.
        unsafe { *len = -1 };
        return null_mut();
    };

    // Measure first: every line's bytes plus its terminator. The walk
    // is `strlen` on purpose — upstream counts bytes up to the NUL,
    // not whatever the memline records as the line's length.
    for lnum in 1..=buf.line_count() {
        // SAFETY: `lnum` is a line of the buffer, and a line is
        // NUL-terminated; `len` is the caller's.
        unsafe { *len += cstr::bytes_at(ml_get_buf(buf, lnum)).len() as ptrdiff_t + 1 };
    }
    // SAFETY: the caller's promise about `len`.
    if unsafe { *len } == 0 {
        return null_mut();
    }

    // SAFETY: `xmalloc` never answers NULL, and the block holds every
    // line's bytes, its separator and the final terminator.
    let ret = unsafe { xmalloc(*len as size_t + 1) as *mut c_char };
    let mut end = ret;
    for lnum in 1..=buf.line_count() {
        // SAFETY: `lnum` is a line of the buffer, and the measurement above
        // left room for its bytes and one separator.
        end = unsafe { copy_swapping_nl(ml_get_buf(buf, lnum), end) };
        // SAFETY: as above -- the separator's byte was measured in.
        end = unsafe { put(end, b'\n' as c_char) };
    }
    // SAFETY: the terminator is the one byte the allocation added.
    unsafe { *end = NUL as c_char };
    // SAFETY: both cursors are into the one allocation.
    unsafe { *len = end.offset_from(ret) as ptrdiff_t };
    ret
}

/// A List is one line per item.
///
/// # Safety
/// `list` must be null or valid; `len` valid.
unsafe fn list_as_string(
    list: *mut List,
    len: *mut ptrdiff_t,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let sep = if crlf { 2 } else { 1 };

    // Measure first, charging every item a separator.
    if !list.is_null() {
        // SAFETY: the caller's promise -- a live List.
        for li in tv_list_iter(unsafe { list.as_ref() }) {
            // SAFETY: `numbuf` outlives the string rendered into it, and
            // `len` is the caller's.
            let tv_len = unsafe { cstr::bytes_at(numbuf.string(&li.li_tv)) }.len();
            unsafe { *len += tv_len as ptrdiff_t + sep };
        }
    }
    // SAFETY: the caller's promise about `len`.
    if unsafe { *len } == 0 {
        return null_mut();
    }

    // The last item's separator is only written when `endnl`, so the
    // measured length already covers the terminator when it is not.
    // SAFETY: `xmalloc` never answers NULL, and `len` is the caller's.
    let ret = unsafe { xmalloc((*len + if endnl { sep } else { 0 }) as size_t) as *mut c_char };
    let mut end = ret;
    if !list.is_null() {
        // SAFETY: the caller's promise -- a live List.
        let count = unsafe { tv_list_len(list) } as usize;
        for (at, li) in tv_list_iter(unsafe { list.as_ref() }).enumerate() {
            // SAFETY: `numbuf2` outlives the string rendered into it, and
            // the measurement above left room for that string's bytes.
            unsafe { end = copy_swapping_nl(numbuf2.string(&li.li_tv), end) };
            let last = at + 1 == count;
            if endnl || !last {
                if crlf {
                    // SAFETY: the measurement charged every item `sep`
                    // bytes, which is two when `crlf`.
                    end = unsafe { put(end, b'\r' as c_char) };
                }
                // SAFETY: as above.
                end = unsafe { put(end, b'\n' as c_char) };
            }
        }
    }
    // SAFETY: the terminator's room is the separator the last item was
    // charged, or the extra `sep` bytes the allocation added.
    unsafe { *end = NUL as c_char };
    // SAFETY: both cursors are into the one allocation.
    unsafe { *len = end.offset_from(ret) as ptrdiff_t };
    ret
}
