//! The prompt-buffer surface: `prompt_appendbuf()`, `prompt_setcallback()`,
//! `prompt_setinterrupt()` and `prompt_setprompt()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::lines::set_buffer_lines;
use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_NUMBER, VAR_STRING};
use core::mem::offset_of;

/// Whether `s` ends in a newline — which asks the *next* `prompt_appendbuf()`
/// to start a fresh line rather than extending this one.
///
/// # Safety
/// `s` must be a NUL-terminated string.
unsafe fn ends_in_newline(s: *const c_char) -> bool {
    // SAFETY: the caller's obligation; the index is within the string because
    // `strlen` measured it.
    let len = unsafe { cstr::bytes_at(s) }.len();
    len > 0 && unsafe { *s.add(len - 1) } == b'\n'.cast_signed()
}

/// The last item of the List `lines` holds, or NULL when it is not a non-empty
/// List.
///
/// # Safety
/// `lines` must be a live typval.
unsafe fn list_last(lines: *mut TypVal) -> *mut ListItem {
    // SAFETY: the caller's obligation; under `VAR_LIST` the union's live arm
    // is `v_list`, a live list or NULL.
    let l = unsafe { (*lines).list_or_null() };
    if l.is_null() {
        return ptr::null_mut();
    }
    let (len, last) = unsafe { ((*l).lv_len, (*l).lv_last) };
    if len == 0 { ptr::null_mut() } else { last }
}

/// `prompt_appendbuf({buf}, {string/list})` — 0 when the text went in.
///
/// Text appended while the prompt line is being edited joins onto the last
/// line rather than starting a new one, unless the previous append ended in a
/// newline.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_prompt_appendbuf(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let (args, result) = frame!(args, result);
    result.v_type = VAR_NUMBER;
    result.vval.v_number = 1;
    // SAFETY: the arguments and `result` are live typvals; every list item
    // reached below belongs to the argument's own list, and `concat_str`
    // hands back an owned string the typval takes over.
    let did_emsg_before = did_emsg.get();
    let Some(buf) = (unsafe { tv_get_buf_from_arg(args.ptr(0)) }) else {
        return;
    };
    if !buf_is_prompt(Some(buf)) {
        return;
    }
    let lnum: LineNr = (buf.b_prompt_start.mark.lnum - 1).max(0);
    let lines = args.ptr(1);
    let mut did_concat = false;
    if !buf.b_prompt_append_new_line {
        // The text so far on the prompt's last line, which the first item
        // of the new text is glued onto.
        let text: *const c_char = if lnum > 0 {
            unsafe { buf.line(lnum) }.raw()
        } else {
            c"".as_ptr()
        };
        let mut tv = unsafe { Tv::new(lines) };
        if tv.v_type == VAR_LIST {
            let l = tv.list_or_null();
            if !l.is_null() && unsafe { (*l).lv_len } > 0 {
                let mut item = unsafe { Li::new((*l).lv_first) };
                let itv = item.field_ptr(offset_of!(ListItem, li_tv));
                let joined = unsafe { concat_str(text, numbuf.string(itv)) };
                unsafe { tv_clear(itv) };
                item.li_tv.v_type = VAR_STRING;
                item.li_tv.vval.v_string = joined;
                did_concat = true;
            }
        } else if tv.v_type == VAR_STRING {
            let joined = unsafe { concat_str(text, numbuf2.string(lines)) };
            unsafe { tv_clear(lines) };
            tv.v_type = VAR_STRING;
            tv.vval.v_string = joined;
        }
    }
    let tv = unsafe { Tv::new(lines) };
    if did_emsg.get() == did_emsg_before {
        let split = did_concat && unsafe { (*tv.list_or_null()).lv_len } > 1;
        if split {
            // The joined first item replaces the prompt line; the rest is
            // appended after it, but only once the replacement worked.
            let l = tv.list_or_null();
            let li = unsafe { (*l).lv_first };
            let itv = unsafe { Li::new(li) }.field_ptr(offset_of!(ListItem, li_tv));
            unsafe { set_buffer_lines(Some(buf), lnum, false, itv, result) };
            if result.number_or_zero() == 0 {
                unsafe { tv_list_item_remove(l, li) };
                unsafe { set_buffer_lines(Some(buf), lnum, true, lines, result) };
            }
        } else {
            let fresh = buf.b_prompt_append_new_line;
            unsafe { set_buffer_lines(Some(buf), lnum, fresh, lines, result) };
        }
    }
    if result.number_or_zero() == 0 {
        let mut buf = buf;
        buf.b_prompt_append_new_line = if tv.v_type == VAR_LIST {
            let last = unsafe { list_last(lines) };
            let ltv = unsafe { Li::new(last) }.field_ptr(offset_of!(ListItem, li_tv));
            !last.is_null() && unsafe { ends_in_newline(numbuf3.string(ltv)) }
        } else {
            tv.v_type == VAR_STRING && unsafe { ends_in_newline(numbuf4.string(lines)) }
        };
    }
}

/// `prompt_setcallback({buf}, {callback})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `_result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_prompt_setcallback(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, _) = frame!(args, _result);
    // SAFETY: the arguments are live typvals, and the buffer is live.
    unsafe { set_prompt_callback(args, |buf| &raw mut buf.b_prompt_callback) };
}

/// `prompt_setinterrupt({buf}, {callback})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `_result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_prompt_setinterrupt(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, _) = frame!(args, _result);
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
unsafe fn set_prompt_callback(args: Args<'_>, slot: impl Fn(&mut Buffer) -> *mut Callback) {
    // SAFETY: the caller's obligation.
    let mut callback = Callback::None;
    if check_secure() {
        return;
    }
    let Some(mut buf) = (unsafe { tv_get_buf(args.ptr(0), 0) }) else {
        return;
    };
    if !unsafe { callback_from_typval(&raw mut callback, args.ptr(1)) } {
        return;
    }
    let slot = slot(&mut buf);
    unsafe { callback_free(slot) };
    unsafe { *slot = callback };
}

/// `prompt_setprompt({buf}, {text})`.
///
/// The prompt is stored on the buffer *and* written into the prompt line, so
/// changing it has to rewrite the line the old prompt is sitting in — unless
/// that line no longer starts with the old prompt, in which case the whole
/// line is replaced.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `_result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_prompt_setprompt(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _) = frame!(args, _result);
    // SAFETY: the arguments are live typvals; every line index below is
    // clamped into the buffer first, and `concat_str` hands back an owned
    // string which `ml_replace_buf` takes over or which is freed here.
    if check_secure() {
        return;
    }
    let Some(mut buf) = (unsafe { tv_get_buf(args.ptr(0), 0) }) else {
        return;
    };
    let new_prompt = unsafe { numbuf.string(args.ptr(1)) };
    let new_prompt_len = len_as_int(unsafe { cstr::bytes_at(new_prompt) }.len());
    if buf_is_prompt(Some(buf)) && !buf.b_ml.ml_mfp.is_null() {
        unsafe { rewrite_prompt_line(buf, new_prompt, new_prompt_len) };
    }
    unsafe { xfree(buf.b_prompt_text.cast()) };
    buf.b_prompt_text = unsafe { xstrdup(new_prompt) };
    buf.b_prompt_start.mark.col = new_prompt_len;
}

/// Put `new_prompt` in place of the old one on the buffer's prompt line.
///
/// # Safety
/// `buffer` must be a live, loaded prompt buffer and `new_prompt` a
/// NUL-terminated string of `new_prompt_len` bytes.
unsafe fn rewrite_prompt_line(mut buffer: Buf, new_prompt: *const c_char, new_prompt_len: c_int) {
    // SAFETY: the caller's obligation.
    if buffer.b_prompt_start.mark.lnum < 1
        || buffer.b_prompt_start.mark.lnum > Buf::current().line_count()
    {
        // MAX(1, MIN(lnum, line_count)); spelled with min-then-max
        // because an empty buffer makes the two bounds cross.
        buffer.b_prompt_start.mark.lnum = buffer
            .b_prompt_start
            .mark
            .lnum
            .min(buffer.line_count())
            .max(1);
        Buf::current().b_prompt_append_new_line = true;
    }
    let prompt_lno = buffer.b_prompt_start.mark.lnum;
    let old_prompt = buf_prompt_text(buffer);
    let old_line = unsafe { buffer.line(prompt_lno) }.raw();
    let old_line_len = unsafe { buffer.line_len(prompt_lno) };
    let old_prompt_len = len_as_int(unsafe { cstr::bytes_at(old_prompt) }.len());
    let mut cursor_col = Win::current().w_cursor.col;
    let prompt_col = buffer.b_prompt_start.mark.col;
    // A byte offset into `old_line`. Every use is guarded by the
    // `prompt_col >= old_prompt_len` test below — `&&` short-circuits —
    // and a prompt is never longer than the line it sits on, so no
    // conversion here can fail.
    let offset = |col: c_int| usize::try_from(col).expect("a prompt column is not negative");
    // Does the line still start with the prompt it was given? When it
    // does, only the prompt itself is swapped; when it does not — the
    // user has edited it away — the whole line goes.
    let fits = prompt_col >= old_prompt_len && prompt_col <= old_line_len;
    let at = |col: c_int| unsafe { old_line.add(offset(col)) };
    let intact = fits
        && unsafe {
            strnequal(
                old_prompt,
                at(prompt_col - old_prompt_len),
                offset(old_prompt_len),
            )
        };
    // The splice both arms report is the same shape: the whole of what was
    // there, replaced by the new prompt.
    let row = prompt_lno - 1;
    let splice = |old_len: c_int| {
        extmark_splice_cols(buffer, row, 0, old_len, new_prompt_len, kExtmarkNoUndo);
    };
    if intact {
        let new_line = unsafe { concat_str(new_prompt, at(prompt_col)) };
        if unsafe { ml_replace_buf(buffer, prompt_lno, new_line, false, false) }.is_err() {
            unsafe { xfree(new_line.cast()) };
        }
        splice(prompt_col);
        cursor_col += new_prompt_len - prompt_col;
    } else {
        let whole = new_prompt.cast_mut();
        let _ = unsafe { ml_replace_buf(buffer, prompt_lno, whole, true, false) };
        splice(old_line_len);
        cursor_col = new_prompt_len;
    }
    let mut win = Win::current();
    if win.w_buffer == buffer.raw() && win.w_cursor.lnum == prompt_lno {
        win.w_cursor.col = cursor_col;
        check_cursor_col(win);
    }
    changed_lines(buffer, prompt_lno, 0, prompt_lno + 1, 0, true);
    u_clearallandblockfree(buffer);
}
