//! The dictionary `getbufinfo()` returns.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::buf_get_changedtick;
use crate::types::{VAR_DICT, VAR_UNKNOWN, kListLenMayKnow};

/// One `getbufinfo()` entry: a buffer's options, variables and attributes.
///
/// # Safety
/// `buf` must be a live buffer.
unsafe fn get_buffer_info(buf: Buf) -> *mut dict_T {
    // SAFETY: the caller's obligation. The dictionary is handed straight to
    // the caller's list, so it is not leaked, and it stays alive for every
    // entry the closure adds.
    let dict = unsafe { tv_dict_alloc() };
    let nr = |key: &CStr, value: varnumber_T| {
        // SAFETY: a live dictionary and a NUL-terminated key.
        unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };
    let str = |key: &CStr, value: *const c_char| {
        // SAFETY: a live dictionary, and two NUL-terminated strings.
        unsafe { tv_dict_add_str(dict, key.as_ptr(), key.count_bytes(), value) };
    };
    let list = |key: &CStr, value: *mut list_T| {
        // SAFETY: a live dictionary and a live list, which the dictionary
        // takes over.
        unsafe { tv_dict_add_list(dict, key.as_ptr(), key.count_bytes(), value) };
    };

    nr(c"bufnr", varnumber_T::from(buf.handle));
    str(
        c"name",
        if buf.b_ffname.is_null() {
            c"".as_ptr()
        } else {
            buf.b_ffname as *const c_char
        },
    );
    // The *current* buffer's line is the cursor's; any other's is the one it
    // will be entered at.
    let lnum = if buf.raw() == curbuf.get() {
        // SAFETY: `curwin` is set from startup to exit.
        unsafe { Win::current() }.w_cursor.lnum
    } else {
        // SAFETY: a live buffer.
        unsafe { buflist_findlnum(buf.raw()) }
    };
    nr(c"lnum", varnumber_T::from(lnum));
    nr(c"linecount", varnumber_T::from(buf.line_count()));
    nr(c"loaded", varnumber_T::from(!buf.b_ml.ml_mfp.is_null()));
    nr(c"listed", varnumber_T::from(buf.b_p_bl));
    // SAFETY: a live buffer.
    nr(
        c"changed",
        varnumber_T::from(unsafe { buf_is_changed(buf.raw()) }),
    );
    // SAFETY: a live buffer.
    nr(c"changedtick", unsafe { buf_get_changedtick(buf.raw()) });
    nr(
        c"hidden",
        varnumber_T::from(!buf.b_ml.ml_mfp.is_null() && buf.b_nwindows == 0),
    );
    nr(c"command", varnumber_T::from(buf.raw() == cmdwin_buf.get()));
    // SAFETY: a live dictionary and the buffer's own variable dictionary.
    unsafe {
        tv_dict_add_dict(
            dict,
            c"variables".as_ptr(),
            c"variables".count_bytes(),
            buf.b_vars,
        );
    }

    // The windows displaying this buffer.
    // SAFETY: the list is handed to the dictionary below, so it is not leaked.
    let windows = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    let append = |handle: handle_T| {
        // SAFETY: a live list.
        unsafe { tv_list_append_number(windows, varnumber_T::from(handle)) };
    };
    for wp in tab_windows().filter(|wp| wp.w_buffer == buf.raw()) {
        append(wp.handle);
    }
    list(c"windows", windows);

    // SAFETY: a live buffer; `get_buffer_signs` hands back a fresh list the
    // dictionary takes over.
    if unsafe { buf_has_signs(buf.raw()) } {
        list(c"signs", unsafe { get_buffer_signs(buf.raw()) });
    }
    nr(c"lastused", buf.b_last_used);
    dict
}

/// `getbufinfo([{buf}|{dict}])` — every buffer, one buffer, or the buffers a
/// filter dictionary selects.
pub unsafe fn f_getbufinfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the list belongs to
    // `rettv` for the whole walk, and `tv_dict_find` hands back a live entry
    // of the dictionary the argument holds.
    unsafe {
        let list = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        let mut argbuf: *mut buf_T = ptr::null_mut();
        let mut filter = Filter::default();
        if args.ty(0) == VAR_DICT {
            let sel_d = args.get(0).vval.v_dict;
            if !sel_d.is_null() {
                let flag = |key: &CStr| {
                    let di = tv_dict_find(sel_d, key.as_ptr(), key.count_bytes().cast_signed());
                    !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0
                };
                filter = Filter {
                    on: true,
                    buflisted: flag(c"buflisted"),
                    bufloaded: flag(c"bufloaded"),
                    bufmodified: flag(c"bufmodified"),
                };
            }
        } else if args.ty(0) != VAR_UNKNOWN {
            argbuf = tv_get_buf_from_arg(args.ptr(0));
            if argbuf.is_null() {
                return;
            }
        }
        for buf in buffers() {
            if !argbuf.is_null() && argbuf != buf.raw() || filter.rejects(buf) {
                continue;
            }
            tv_list_append_dict(list, get_buffer_info(buf));
            if !argbuf.is_null() {
                return;
            }
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
    /// Whether `buf` fails one of the selectors that is switched on.
    fn rejects(&self, buf: Buf) -> bool {
        self.on
            && (self.bufloaded && buf.b_ml.ml_mfp.is_null()
                || self.buflisted && buf.b_p_bl == 0
                || self.bufmodified && buf.b_changed == 0)
    }
}
