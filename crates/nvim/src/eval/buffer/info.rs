//! The dictionary `getbufinfo()` returns.

#![forbid(unsafe_code)]
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
    let dict = tv_dict_alloc();
    let nr = |key: &CStr, value: VarNumber| {
        let _ = dict.edit().add_number(key.to_bytes(), value);
    };
    let list = |key: &CStr, value: Option<ListRef>| {
        let _ = dict.edit().add_list(key.to_bytes(), value);
    };

    nr(c"bufnr", VarNumber::from(buffer.handle));
    let name = buffer.name.full().unwrap_or(c"");
    let _ = dict.edit().add_str(b"name", Some(name));
    // The *current* buffer's line is the cursor's; any other's is the one it
    // will be entered at.
    let lnum = if buffer.is_current() {
        Win::current().w_cursor.lnum
    } else {
        buflist_findlnum(buffer)
    };
    nr(c"lnum", VarNumber::from(lnum));
    nr(c"linecount", VarNumber::from(buffer.line_count()));
    nr(c"loaded", VarNumber::from(!buffer.b_ml.ml_mfp.is_null()));
    nr(c"listed", VarNumber::from(buffer.b_p_bl));
    nr(c"changed", VarNumber::from(buf_is_changed(buffer)));
    nr(c"changedtick", buf_get_changedtick(buffer));
    nr(
        c"hidden",
        VarNumber::from(!buffer.b_ml.ml_mfp.is_null() && buffer.b_nwindows == 0),
    );
    nr(
        c"command",
        VarNumber::from(cmdwin_buf.get() == Some(buffer.id())),
    );
    // The buffer's own `b:` scope; the answer takes a reference.
    let vars = buffer.b_bufvar.di_tv.dict_handle();
    let _ = dict.edit().add_dict(b"variables", vars);

    // The windows displaying this buffer.
    let windows = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for wp in tab_windows().filter(|wp| wp.w_buffer == buffer) {
        windows.edit().push_number(VarNumber::from(wp.handle));
    }
    list(c"windows", Some(windows));

    if buf_has_signs(buffer) {
        list(c"signs", Some(get_buffer_signs(buffer)));
    }
    nr(c"lastused", buffer.b_last_used);
    dict
}

/// `getbufinfo([{buf}|{dict}])` — every buffer, one buffer, or the buffers a
/// filter dictionary selects.
pub fn f_getbufinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let list = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let mut argbuf: Option<Buf> = None;
    let mut filter = Filter::default();
    if args.first().is_some_and(|arg| arg.v_type() == VAR_DICT) {
        let sel_d = args[0].dict_ref();
        if sel_d.is_some() {
            let flag = |key: &CStr| {
                dict_find(sel_d, key.to_bytes()).is_some_and(|di| tv_get_number(&di.di_tv) != 0)
            };
            filter = Filter {
                on: true,
                buflisted: flag(c"buflisted"),
                bufloaded: flag(c"bufloaded"),
                bufmodified: flag(c"bufmodified"),
            };
        }
    } else if !args.is_empty() {
        argbuf = arg_buf_chk(args, 0);
        if argbuf.is_none() {
            return;
        }
    }
    for buf in buffers() {
        if argbuf.is_some_and(|wanted| wanted != buf) || filter.rejects(buf) {
            continue;
        }
        list.push_dict(Some(get_buffer_info(buf)));
        if argbuf.is_some() {
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
