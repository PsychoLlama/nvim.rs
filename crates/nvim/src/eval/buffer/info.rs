//! The dictionary `getbufinfo()` returns.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::buf_get_changedtick;
use crate::types::{VAR_DICT, kListLenMayKnow};

/// One `getbufinfo()` entry: a buffer's options, variables and attributes.
fn get_buffer_info(buffer: Buf) -> DictRef {
    // SAFETY: the caller's obligation. The dictionary is handed straight to
    // the caller's list, so it is not leaked, and it stays alive for every
    // entry the closure adds.
    let dict_held = tv_dict_alloc();
    let dict = dict_held.as_ptr();
    let nr = |key: &CStr, value: VarNumber| {
        // SAFETY: a live dictionary and a NUL-terminated key.
        let _ = unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };
    let str = |key: &CStr, value: *const c_char| {
        // SAFETY: a live dictionary, and two NUL-terminated strings.
        let _ = unsafe { tv_dict_add_str(dict, key.as_ptr(), key.count_bytes(), value) };
    };
    let list = |key: &CStr, value: Option<ListRef>| {
        // SAFETY: a live dictionary and a live list, which the dictionary
        // takes over.
        let _ = unsafe { tv_dict_add_list(dict, key.as_ptr(), key.count_bytes(), value) };
    };

    nr(c"bufnr", VarNumber::from(buffer.handle));
    str(
        c"name",
        if buffer.b_ffname.is_null() {
            c"".as_ptr()
        } else {
            buffer.b_ffname as *const c_char
        },
    );
    // The *current* buffer's line is the cursor's; any other's is the one it
    // will be entered at.
    let lnum = if buffer.raw() == Buf::current_raw() {
        // SAFETY: `curwin` is set from startup to exit.
        Win::current().w_cursor.lnum
    } else {
        buflist_findlnum(buffer)
    };
    nr(c"lnum", VarNumber::from(lnum));
    nr(c"linecount", VarNumber::from(buffer.line_count()));
    nr(c"loaded", VarNumber::from(!buffer.b_ml.ml_mfp.is_null()));
    nr(c"listed", VarNumber::from(buffer.b_p_bl));
    // SAFETY: a live buffer.
    nr(c"changed", VarNumber::from(buf_is_changed(buffer)));
    // SAFETY: a live buffer.
    nr(c"changedtick", buf_get_changedtick(buffer));
    nr(
        c"hidden",
        VarNumber::from(!buffer.b_ml.ml_mfp.is_null() && buffer.b_nwindows == 0),
    );
    nr(
        c"command",
        VarNumber::from(cmdwin_buf.get() == Some(buffer.id())),
    );
    // SAFETY: a live dictionary and the buffer's own variable dictionary.
    let vars = c"variables";
    // SAFETY: the buffer's own `b:` scope; the answer takes a reference.
    let b_vars = unsafe { DictRef::retained(buffer.b_vars) };
    let _ = unsafe { tv_dict_add_dict(dict, vars.as_ptr(), vars.count_bytes(), b_vars) };

    // The windows displaying this buffer.
    let windows = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let into = windows.as_ptr();
    let append = |handle: Handle| {
        // SAFETY: a live list.
        unsafe { tv_list_append_number(into, VarNumber::from(handle)) };
    };
    for wp in tab_windows().filter(|wp| wp.w_buffer == buffer.raw()) {
        append(wp.handle);
    }
    list(c"windows", Some(windows));

    if buf_has_signs(buffer) {
        list(c"signs", Some(get_buffer_signs(buffer)));
    }
    nr(c"lastused", buffer.b_last_used);
    dict_held
}

/// `getbufinfo([{buf}|{dict}])` — every buffer, one buffer, or the buffers a
/// filter dictionary selects.
pub fn f_getbufinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the arguments and `result` are live typvals; the list belongs to
    // `result` for the whole walk, and `tv_dict_find` hands back a live entry
    // of the dictionary the argument holds.
    let list = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let mut argbuf: *mut Buffer = ptr::null_mut();
    let mut filter = Filter::default();
    if args.first().is_some_and(|arg| arg.v_type() == VAR_DICT) {
        let sel_d = args[0].dict_or_null();
        if !sel_d.is_null() {
            let flag = |key: &CStr| {
                let di =
                    unsafe { tv_dict_find(sel_d, key.as_ptr(), key.count_bytes().cast_signed()) };
                !di.is_null() && unsafe { tv_get_number(&(*di).di_tv) } != 0
            };
            filter = Filter {
                on: true,
                buflisted: flag(c"buflisted"),
                bufloaded: flag(c"bufloaded"),
                bufmodified: flag(c"bufmodified"),
            };
        }
    } else if !args.is_empty() {
        argbuf = arg_buf_chk(args, 0).map_or(ptr::null_mut(), Buf::raw);
        if argbuf.is_null() {
            return;
        }
    }
    for buf in buffers() {
        if !argbuf.is_null() && argbuf != buf.raw() || filter.rejects(buf) {
            continue;
        }
        unsafe { tv_list_append_dict(list, Some(get_buffer_info(buf))) };
        if !argbuf.is_null() {
            return;
        }
    }
}

/// The `getbufinfo({dict})` selectors. Each is an *additional* requirement,
/// and `on` is false when no dictionary was given at all.
#[derive(Default)]
struct Filter {
    on: bool,
    buflisted: bool,
    bufloaded: bool,
    bufmodified: bool,
}

impl Filter {
    /// Whether `buffer` fails one of the selectors that is switched on.
    fn rejects(&self, buffer: Buf) -> bool {
        self.on
            && (self.bufloaded && buffer.b_ml.ml_mfp.is_null()
                || self.buflisted && buffer.b_p_bl == 0
                || self.bufmodified && buffer.b_changed == 0)
    }
}
