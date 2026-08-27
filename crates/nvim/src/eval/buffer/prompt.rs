//! The prompt-buffer surface: `prompt_appendbuf()`, `prompt_setcallback()`,
//! `prompt_setinterrupt()` and `prompt_setprompt()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::lines::set_buffer_lines;
use super::*;
use crate::eval::typval::{NumBuf, kCallbackNone};
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_NUMBER, VAR_STRING};

/// Whether `s` ends in a newline — which asks the *next* `prompt_appendbuf()`
/// to start a fresh line rather than extending this one.
///
/// # Safety
/// `s` must be a NUL-terminated string.
unsafe fn ends_in_newline(s: *const c_char) -> bool {
    // SAFETY: the caller's obligation; the index is within the string because
    // `strlen` measured it.
    unsafe {
        let len = strlen(s);
        len > 0 && *s.add(len - 1) == b'\n'.cast_signed()
    }
}

/// The last item of the List `lines` holds, or NULL when it is not a non-empty
/// List.
///
/// # Safety
/// `lines` must be a live typval.
unsafe fn list_last(lines: *mut typval_T) -> *mut listitem_T {
    // SAFETY: the caller's obligation; under `VAR_LIST` the union's live arm
    // is `v_list`, a live list or NULL.
    unsafe {
        let l = (*lines).vval.v_list;
        if l.is_null() || (*l).lv_len == 0 {
            ptr::null_mut()
        } else {
            (*l).lv_last
        }
    }
}

/// `prompt_appendbuf({buf}, {string/list})` — 0 when the text went in.
///
/// Text appended while the prompt line is being edited joins onto the last
/// line rather than starting a new one, unless the previous append ended in a
/// newline.
pub unsafe fn f_prompt_appendbuf(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 1;
    // SAFETY: the arguments and `rettv` are live typvals; every list item
    // reached below belongs to the argument's own list, and `concat_str`
    // hands back an owned string the typval takes over.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let Some(buf) = Buf::from_raw(tv_get_buf_from_arg(args.ptr(0))) else {
            return;
        };
        if !bt_prompt(buf.raw()) {
            return;
        }
        let lnum: linenr_T = (buf.b_prompt_start.mark.lnum - 1).max(0);
        let lines = args.ptr(1);
        let mut did_concat = false;
        if !buf.b_prompt_append_new_line {
            // The text so far on the prompt's last line, which the first item
            // of the new text is glued onto.
            let text: *const c_char = if lnum > 0 {
                buf.line(lnum).raw()
            } else {
                c"".as_ptr()
            };
            if (*lines).v_type == VAR_LIST {
                let l = (*lines).vval.v_list;
                if !l.is_null() && (*l).lv_len > 0 {
                    let li = (*l).lv_first;
                    let joined = concat_str(text, numbuf.string(&raw mut (*li).li_tv));
                    tv_clear(&raw mut (*li).li_tv);
                    (*li).li_tv.v_type = VAR_STRING;
                    (*li).li_tv.vval.v_string = joined;
                    did_concat = true;
                }
            } else if (*lines).v_type == VAR_STRING {
                let joined = concat_str(text, numbuf2.string(lines));
                tv_clear(lines);
                (*lines).v_type = VAR_STRING;
                (*lines).vval.v_string = joined;
            }
        }
        if did_emsg.get() == did_emsg_before {
            if did_concat && (*(*lines).vval.v_list).lv_len > 1 {
                // The joined first item replaces the prompt line; the rest is
                // appended after it, but only once the replacement worked.
                let l = (*lines).vval.v_list;
                let li = (*l).lv_first;
                set_buffer_lines(buf.raw(), lnum, false, &raw mut (*li).li_tv, rettv);
                if rettv.vval.v_number == 0 {
                    tv_list_item_remove(l, li);
                    set_buffer_lines(buf.raw(), lnum, true, lines, rettv);
                }
            } else {
                set_buffer_lines(buf.raw(), lnum, buf.b_prompt_append_new_line, lines, rettv);
            }
        }
        if rettv.vval.v_number == 0 {
            let mut buf = buf;
            buf.b_prompt_append_new_line = if (*lines).v_type == VAR_LIST {
                let last = list_last(lines);
                !last.is_null() && ends_in_newline(numbuf3.string(&raw mut (*last).li_tv))
            } else {
                (*lines).v_type == VAR_STRING && ends_in_newline(numbuf4.string(lines))
            };
        }
    }
}

/// `prompt_setcallback({buf}, {callback})`.
pub unsafe fn f_prompt_setcallback(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _) = frame!(argvars, _rettv);
    // SAFETY: the arguments are live typvals, and the buffer is live.
    unsafe { set_prompt_callback(args, |buf| &raw mut buf.b_prompt_callback) };
}

/// `prompt_setinterrupt({buf}, {callback})`.
pub unsafe fn f_prompt_setinterrupt(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _) = frame!(argvars, _rettv);
    // SAFETY: the arguments are live typvals, and the buffer is live.
    unsafe { set_prompt_callback(args, |buf| &raw mut buf.b_prompt_interrupt) };
}

/// The half `prompt_setcallback()` and `prompt_setinterrupt()` share: resolve
/// the buffer, build the callback, then free the one `slot` held.
///
/// Nothing is freed until the new callback has been built, so a bad second
/// argument leaves the old one in place.
///
/// # Safety
/// The arguments must be live typvals, and `slot` must answer a field of the
/// buffer it is handed.
unsafe fn set_prompt_callback(args: Args<'_>, slot: impl Fn(&mut buf_T) -> *mut Callback) {
    // SAFETY: the caller's obligation.
    unsafe {
        let mut callback = Callback {
            data: Callback_data {
                funcref: ptr::null_mut(),
            },
            type_0: kCallbackNone,
        };
        if check_secure() {
            return;
        }
        let Some(mut buf) = Buf::from_raw(tv_get_buf(args.ptr(0), 0)) else {
            return;
        };
        if !callback_from_typval(&raw mut callback, args.ptr(1)) {
            return;
        }
        let slot = slot(&mut buf);
        callback_free(slot);
        *slot = callback;
    }
}

/// `prompt_setprompt({buf}, {text})`.
///
/// The prompt is stored on the buffer *and* written into the prompt line, so
/// changing it has to rewrite the line the old prompt is sitting in — unless
/// that line no longer starts with the old prompt, in which case the whole
/// line is replaced.
pub unsafe fn f_prompt_setprompt(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let (args, _) = frame!(argvars, _rettv);
    // SAFETY: the arguments are live typvals; every line index below is
    // clamped into the buffer first, and `concat_str` hands back an owned
    // string which `ml_replace_buf` takes over or which is freed here.
    unsafe {
        if check_secure() {
            return;
        }
        let Some(mut buf) = Buf::from_raw(tv_get_buf(args.ptr(0), 0)) else {
            return;
        };
        let new_prompt = numbuf.string(args.ptr(1));
        let new_prompt_len = len_as_int(strlen(new_prompt));
        if bt_prompt(buf.raw()) && !buf.b_ml.ml_mfp.is_null() {
            rewrite_prompt_line(buf, new_prompt, new_prompt_len);
        }
        xfree(buf.b_prompt_text.cast());
        buf.b_prompt_text = xstrdup(new_prompt);
        buf.b_prompt_start.mark.col = new_prompt_len;
    }
}

/// Put `new_prompt` in place of the old one on the buffer's prompt line.
///
/// # Safety
/// `buf` must be a live, loaded prompt buffer and `new_prompt` a
/// NUL-terminated string of `new_prompt_len` bytes.
unsafe fn rewrite_prompt_line(mut buf: Buf, new_prompt: *const c_char, new_prompt_len: c_int) {
    // SAFETY: the caller's obligation.
    unsafe {
        if buf.b_prompt_start.mark.lnum < 1
            || buf.b_prompt_start.mark.lnum > Buf::current().line_count()
        {
            // MAX(1, MIN(lnum, line_count)); spelled with min-then-max
            // because an empty buffer makes the two bounds cross.
            buf.b_prompt_start.mark.lnum =
                buf.b_prompt_start.mark.lnum.min(buf.line_count()).max(1);
            Buf::current().b_prompt_append_new_line = true;
        }
        let prompt_lno = buf.b_prompt_start.mark.lnum;
        let old_prompt = buf_prompt_text(buf);
        let old_line = buf.line(prompt_lno).raw();
        let old_line_len = buf.line_len(prompt_lno);
        let old_prompt_len = len_as_int(strlen(old_prompt));
        let mut cursor_col = Win::current().w_cursor.col;
        let prompt_col = buf.b_prompt_start.mark.col;
        // A byte offset into `old_line`. Every use is guarded by the
        // `prompt_col >= old_prompt_len` test below — `&&` short-circuits —
        // and a prompt is never longer than the line it sits on, so no
        // conversion here can fail.
        let offset = |col: c_int| usize::try_from(col).expect("a prompt column is not negative");
        // Does the line still start with the prompt it was given? When it
        // does, only the prompt itself is swapped; when it does not — the
        // user has edited it away — the whole line goes.
        let intact = prompt_col >= old_prompt_len
            && prompt_col <= old_line_len
            && strnequal(
                old_prompt,
                old_line.add(offset(prompt_col - old_prompt_len)),
                offset(old_prompt_len),
            );
        if intact {
            let new_line = concat_str(new_prompt, old_line.add(offset(prompt_col)));
            if ml_replace_buf(buf.raw(), prompt_lno, new_line, false, false) != OK {
                xfree(new_line.cast());
            }
            extmark_splice_cols(
                buf.raw(),
                prompt_lno - 1,
                0,
                prompt_col,
                new_prompt_len,
                kExtmarkNoUndo,
            );
            cursor_col += new_prompt_len - prompt_col;
        } else {
            ml_replace_buf(
                buf.raw(),
                prompt_lno,
                new_prompt as *mut c_char,
                true,
                false,
            );
            extmark_splice_cols(
                buf.raw(),
                prompt_lno - 1,
                0,
                old_line_len,
                new_prompt_len,
                kExtmarkNoUndo,
            );
            cursor_col = new_prompt_len;
        }
        let mut win = Win::current();
        if win.w_buffer == buf.raw() && win.w_cursor.lnum == prompt_lno {
            win.w_cursor.col = cursor_col;
            check_cursor_col(win);
        }
        changed_lines(buf, prompt_lno, 0, prompt_lno + 1, 0, true);
        u_clearallandblockfree(buf);
    }
}
