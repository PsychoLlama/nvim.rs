//! `system()` and `systemlist()`: the argument vector and the captured
//! output.
//!
//! Both directions swap NUL and newline. A shell's stdin is a byte stream
//! with no way to carry a NUL, so `save_tv_as_string` writes a newline for
//! every NUL a List item held and vice versa; `get_system_output_as_rettv`
//! undoes it on the way back. That is why the two halves look asymmetric:
//! one builds a buffer, the other rewrites one in place.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::{null, null_mut};

use crate::buffer::find_buf;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    NumBuf, tv_get_number, tv_list_alloc, tv_list_alloc_ret, tv_list_first, tv_list_len,
    tv_list_ref,
};
use crate::eval::vars::emsg_static;
use crate::eval::vars::set_vim_var_nr;
use crate::eval::{NL, PROF_YES, Tv};
use crate::ex_cmds::check_secure;
use crate::main::{do_profiling, e_invarg, e_invarg2, e_invargNval, e_nobufnr, p_verbose};
use crate::memline::ml_get_buf;
use crate::memory::{memchrsub, xcalloc, xfree, xmalloc, xmemdupz, xstrdup};
use crate::message::{msg_puts, verbose_enter_scroll, verbose_leave_scroll};
use crate::os::cshim::{gettext, snprintf};
use crate::os::fs::os_can_exe;
use crate::os::shell::{os_system, shell_argv_to_str, shell_build_argv, shell_free_argv};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::types::{
    EvalFuncData, IOSIZE, NUL, OptInt, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, Vv,
    kListLenMayKnow, list_T, listitem_T, proftime_T, ptrdiff_t, size_t, typval_T, varnumber_T,
};
use ::libc::strlen;

/// Build a `NULL`-terminated argument vector out of a String (through the
/// shell) or a List (directly). `cmd`, when given, comes back naming the
/// executable; `executable` is cleared when the first item is not one.
///
/// # Safety
/// `cmd_tv` must be valid; `cmd` and `executable` null or valid. `numbuf` is
/// the scratch a Number command is spelled into and must outlive `*cmd`,
/// which may point into it.
pub unsafe fn tv_to_argv(
    cmd_tv: *mut typval_T,
    cmd: *mut *const c_char,
    executable: *mut bool,
    numbuf: &mut NumBuf,
) -> *mut *mut c_char {
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    // SAFETY: the caller's promise -- the typval outlives the call.
    let tv = unsafe { Tv::new(cmd_tv) };
    if tv.v_type == VAR_STRING {
        // SAFETY: `numbuf` is the caller's scratch, which outlives `*cmd`.
        let cmd_str = unsafe { numbuf.string(cmd_tv) };
        if !cmd.is_null() {
            // SAFETY: the caller's promise -- a non-null `cmd` is valid.
            unsafe { *cmd = cmd_str };
        }
        // SAFETY: `cmd_str` is NUL-terminated.
        return unsafe { shell_build_argv(cmd_str, null::<c_char>()) };
    }
    if tv.v_type != VAR_LIST {
        let what = c"expected String or List".as_ptr();
        // SAFETY: the format takes one NUL-terminated string.
        unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), what) };
        return null_mut();
    }

    // SAFETY: `VAR_LIST` says `v_list` is the union's live member.
    let argl: *mut list_T = unsafe { tv.vval.v_list };
    // SAFETY: `argl` is a live List or null.
    let argc = unsafe { tv_list_len(argl) };
    if argc == 0 {
        // SAFETY: `e_invarg` is a shared NUL-terminated message.
        emsg_static(&e_invarg);
        return null_mut();
    }

    // The first item has to resolve to something runnable, and the
    // resolved path is what actually goes in slot 0.
    // SAFETY: a non-empty List has a first item, and `numbuf2` outlives
    // the string rendered into it.
    let arg0 = unsafe { numbuf2.string_chk(&raw mut (*tv_list_first(argl)).li_tv) };
    let mut exe_resolved: *mut c_char = null_mut();
    // SAFETY: `arg0` is NUL-terminated and `exe_resolved` is this frame's.
    let runnable = !arg0.is_null() && unsafe { os_can_exe(arg0, &raw mut exe_resolved, true) };
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
            unsafe { semsg_c!(gettext(e_invargNval.as_ptr()), what, text) };
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
        let mut arg: *const listitem_T = unsafe { (*argl).lv_first };
        while !arg.is_null() {
            // SAFETY: `arg` is one of the List's items, and `numbuf3`
            // outlives the string rendered into it.
            let a = unsafe { numbuf3.string_chk(&raw const (*arg).li_tv) };
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
            // SAFETY: `arg` is a live item.
            arg = unsafe { (*arg).li_next };
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
) -> *mut list_T {
    // A trailing newline does not start an empty last line unless the
    // caller asked to keep one.
    // SAFETY: the caller's promise -- `len` bytes are readable, so the
    // last one is.
    if !keepempty && unsafe { *str.add(len - 1) } as c_int == NL {
        len -= 1;
    }
    // SAFETY: the allocation is the encoder's own sink.
    let list = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    // SAFETY: as above; `str` has `len` readable bytes.
    unsafe { encode_list_write(list as *mut c_void, str, len) };
    list
}

/// The shared body of `system()` and `systemlist()`.
///
/// # Safety
/// `argvars` must hold the builtin's arguments; `rettv` must be valid.
pub(crate) unsafe fn get_system_output_as_rettv(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    retlist: bool,
) {
    let mut cmdbuf = NumBuf::new();
    let profiling = do_profiling.get() == PROF_YES;
    // SAFETY: the caller's promise -- `rettv` outlives the call.
    let mut ret = unsafe { Tv::new(rettv) };
    ret.v_type = VAR_STRING;
    ret.vval.v_string = null_mut();
    if check_secure() {
        return;
    }

    let mut input_len: ptrdiff_t = 0;
    // SAFETY: the builtin's vector always has a second slot, which is
    // `VAR_UNKNOWN` when the caller passed one argument.
    let input = unsafe { save_tv_as_string(argvars.add(1), &raw mut input_len, false, false) };
    if input_len < 0 {
        debug_assert!(input.is_null());
        return;
    }

    let mut executable = true;
    // SAFETY: `argvars` is the builtin's own vector, and `cmdbuf` outlives
    // the argv a Number command is spelled into.
    let argv = unsafe { tv_to_argv(argvars, null_mut(), &raw mut executable, &mut cmdbuf) };
    if argv.is_null() {
        // A command that does not exist reports -1 rather than a shell
        // exit status.
        if !executable {
            // SAFETY: setting a `v:` variable only touches the vimvars.
            unsafe { set_vim_var_nr(Vv::ShellError, -1) };
        }
        // SAFETY: the input buffer is owned here.
        unsafe { xfree(input as *mut c_void) };
        return;
    }

    if p_verbose.get() > 3 as OptInt {
        // SAFETY: `argv` is the NULL-terminated vector built above.
        let cmdstr = unsafe { shell_argv_to_str(argv) };
        // SAFETY: the scroll bracket is the message area's own.
        unsafe { verbose_enter_scroll() };
        // SAFETY: the format takes the one NUL-terminated `cmdstr`.
        unsafe { smsg_c!(0, gettext(c"Executing command: \"%s\"".as_ptr()), cmdstr) };
        // SAFETY: the literal is NUL-terminated.
        unsafe { msg_puts(c"\n\n".as_ptr()) };
        // SAFETY: this closes the bracket opened above.
        unsafe { verbose_leave_scroll() };
        // SAFETY: `cmdstr` is the owned rendering.
        unsafe { xfree(cmdstr as *mut c_void) };
    }

    let mut wait_time: proftime_T = 0;
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
    // SAFETY: setting a `v:` variable only touches the vimvars.
    unsafe { set_vim_var_nr(Vv::ShellError, status as varnumber_T) };

    if res.is_null() {
        if retlist {
            // SAFETY: `rettv` is the caller's.
            unsafe { tv_list_alloc_ret(rettv, 0 as ptrdiff_t) };
        } else {
            // SAFETY: the literal is NUL-terminated.
            ret.vval.v_string = unsafe { xstrdup(c"".as_ptr()) };
        }
        return;
    }

    if retlist {
        // The `keepempty` argument is the third, so it is only read
        // when the second was given too.
        let mut keepempty = 0;
        // SAFETY: the builtin declares three slots, and the third is only
        // reached once the second turned out to be given.
        let given = unsafe { (*argvars.add(1)).v_type } != VAR_UNKNOWN
            && unsafe { (*argvars.add(2)).v_type } != VAR_UNKNOWN;
        if given {
            // SAFETY: as above.
            keepempty = unsafe { tv_get_number(argvars.add(2)) } as c_int;
        }
        // SAFETY: `res` holds `nread` readable bytes.
        let list = unsafe { string_to_list(res, nread, keepempty != 0) };
        ret.vval.v_list = list;
        // SAFETY: the List was just built.
        unsafe { tv_list_ref(list) };
        ret.v_type = VAR_LIST;
        // SAFETY: the encoder copied what it needed.
        unsafe { xfree(res as *mut c_void) };
    } else {
        // Undo the swap in place; the buffer is handed over as it is.
        // SAFETY: `res` holds `nread` writable bytes.
        unsafe { memchrsub(res as *mut c_void, NUL as c_char, 1 as c_char, nread) };
        ret.vval.v_string = res;
    }
}

/// `system()`
///
/// # Safety
/// Called through the builtin table.
pub unsafe fn f_system(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_system_output_as_rettv(argvars, rettv, false) }
}

/// `systemlist()`
///
/// # Safety
/// Called through the builtin table.
pub unsafe fn f_systemlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_system_output_as_rettv(argvars, rettv, true) }
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
    tv: *mut typval_T,
    len: *mut ptrdiff_t,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- both outlive the call.
    let value = unsafe { Tv::new(tv) };
    // SAFETY: as above.
    unsafe { *len = 0 };
    if value.v_type == VAR_UNKNOWN {
        return null_mut();
    }
    if value.v_type != VAR_LIST && value.v_type != VAR_NUMBER {
        // SAFETY: `numbuf` outlives the string rendered into it.
        let ret = unsafe { numbuf.string_chk(tv) };
        if ret.is_null() {
            // SAFETY: the caller's promise about `len`.
            unsafe { *len = -1 };
            return null_mut();
        }
        // SAFETY: `ret` is NUL-terminated, and `len` is the caller's.
        unsafe { *len = strlen(ret) as ptrdiff_t };
        // SAFETY: `ret` has the `*len` bytes just measured.
        return unsafe { xmemdupz(ret as *const c_void, *len as size_t) as *mut c_char };
    }
    if value.v_type == VAR_NUMBER {
        // SAFETY: a `VAR_NUMBER`, which is what the callee wants.
        return unsafe { buffer_as_string(tv, len) };
    }
    // SAFETY: `VAR_LIST` says `v_list` is the union's live member.
    unsafe { list_as_string(value.vval.v_list, len, endnl, crlf) }
}

/// A Number names a buffer; its whole text is the input.
///
/// # Safety
/// `tv` must be a `VAR_NUMBER`; `len` valid.
unsafe fn buffer_as_string(tv: *mut typval_T, len: *mut ptrdiff_t) -> *mut c_char {
    // SAFETY: the caller's promise -- a `VAR_NUMBER`, so `v_number` is the
    // union's live member.
    let nr = unsafe { Tv::new(tv).vval.v_number };
    let Some(buf) = find_buf(nr as c_int) else {
        // SAFETY: the format takes one number, and `len` is the caller's.
        unsafe { semsg_c!(gettext(e_nobufnr.as_ptr()), nr) };
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
        unsafe { *len += strlen(ml_get_buf(buf.raw(), lnum)) as ptrdiff_t + 1 };
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
        end = unsafe { copy_swapping_nl(ml_get_buf(buf.raw(), lnum), end) };
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
    list: *mut list_T,
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
        let mut li: *const listitem_T = unsafe { (*list).lv_first };
        while !li.is_null() {
            // SAFETY: `li` is one of the List's items, `numbuf` outlives
            // the string rendered into it, and `len` is the caller's.
            unsafe { *len += strlen(numbuf.string(&raw const (*li).li_tv)) as ptrdiff_t + sep };
            // SAFETY: `li` is a live item.
            li = unsafe { (*li).li_next };
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
        let mut li: *const listitem_T = unsafe { (*list).lv_first };
        while !li.is_null() {
            // SAFETY: `li` is one of the List's items, `numbuf2` outlives
            // the string rendered into it, and the measurement above left
            // room for that string's bytes.
            unsafe { end = copy_swapping_nl(numbuf2.string(&raw const (*li).li_tv), end) };
            // SAFETY: `li` is a live item.
            let last = unsafe { (*li).li_next }.is_null();
            if endnl || !last {
                if crlf {
                    // SAFETY: the measurement charged every item `sep`
                    // bytes, which is two when `crlf`.
                    end = unsafe { put(end, b'\r' as c_char) };
                }
                // SAFETY: as above.
                end = unsafe { put(end, b'\n' as c_char) };
            }
            // SAFETY: `li` is a live item.
            li = unsafe { (*li).li_next };
        }
    }
    // SAFETY: the terminator's room is the separator the last item was
    // charged, or the extra `sep` bytes the allocation added.
    unsafe { *end = NUL as c_char };
    // SAFETY: both cursors are into the one allocation.
    unsafe { *len = end.offset_from(ret) as ptrdiff_t };
    ret
}
