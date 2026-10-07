//! The prompt-buffer surface: `prompt_appendbuf()`, `prompt_setcallback()`,
//! `prompt_setinterrupt()` and `prompt_setprompt()`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::lines::set_buffer_lines;
use super::*;
use crate::edit::{buf_prompt_text_owned, set_buf_prompt_text};
use crate::eval::typval::{NumBuf, list_items, list_len, tv_copy};
use crate::memline::{Lines, ml_replace_buf_text};
use crate::memory::ThinCString;
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_STRING};

/// Whether `s` ends in a newline — which asks the *next* `prompt_appendbuf()`
/// to start a fresh line rather than extending this one.
fn ends_in_newline(s: &[u8]) -> bool {
    s.last() == Some(&b'\n')
}

/// `prompt_appendbuf({buf}, {string/list})` — 0 when the text went in.
///
/// Text appended while the prompt line is being edited joins onto the last
/// line rather than starting a new one, unless the previous append ended in a
/// newline.
pub fn f_prompt_appendbuf(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    result.write_number(1);
    let did_emsg_before = did_emsg.get();
    let Some(buf) = tv_get_buf_from_arg(&args[0]) else {
        return;
    };
    if !buf_is_prompt(Some(buf)) {
        return;
    }
    let lnum: LineNr = (buf.b_prompt_start.mark.lnum - 1).max(0);
    // A String argument that is glued onto the prompt line is replaced by
    // the joined text, which this local owns; a List argument is joined in
    // place, in the caller's own list.
    let joined_string;
    let mut lines = &args[1];
    let list = lines.list_handle();
    let mut did_concat = false;
    if !buf.b_prompt_append_new_line {
        // The text so far on the prompt's last line, which the first item
        // of the new text is glued onto.
        let text = if lnum > 0 {
            Lines::in_buffer(buf).line(lnum).to_vec()
        } else {
            Vec::new()
        };
        let glue = |tail: &CStr| ThinCString::from_vec([text.as_slice(), tail.to_bytes()].concat());
        if lines.v_type() == VAR_LIST {
            if let Some(list) = &list
                && let Some(item) = list.edit().items_mut().first_mut()
            {
                let joined = glue(numbuf.string(&item.li_tv));
                tv_clear(&mut item.li_tv);
                item.li_tv.write_string(Some(joined));
                did_concat = true;
            }
        } else if lines.v_type() == VAR_STRING {
            joined_string = TypVal::string(Some(glue(numbuf2.string(lines))));
            lines = &joined_string;
        }
    }
    if did_emsg.get() == did_emsg_before {
        let split = did_concat && list_len(list.as_deref()) > 1;
        if let (true, Some(list)) = (split, &list) {
            // The joined first item replaces the prompt line; the rest is
            // appended after it, but only once the replacement worked. The
            // first item is copied out: the replacement runs autocommands,
            // which may edit the list.
            let mut first = TypVal::Number(0);
            tv_copy(&list.items()[0].li_tv, &mut first);
            set_buffer_lines(Some(buf), lnum, false, &first, result);
            drop(first);
            if result.number_or_zero() == 0 {
                drop(list.edit().take_range(0, 0));
                set_buffer_lines(Some(buf), lnum, true, lines, result);
            }
        } else {
            let fresh = buf.b_prompt_append_new_line;
            set_buffer_lines(Some(buf), lnum, fresh, lines, result);
        }
    }
    if result.number_or_zero() == 0 {
        let mut buf = buf;
        buf.b_prompt_append_new_line = if lines.v_type() == VAR_LIST {
            match list_items(lines.list_ref()).last() {
                Some(last) => ends_in_newline(numbuf3.bytes(&last.li_tv)),
                None => false,
            }
        } else {
            lines.v_type() == VAR_STRING && ends_in_newline(numbuf4.bytes(lines))
        };
    }
}

/// `prompt_setcallback({buf}, {callback})`.
pub fn f_prompt_setcallback(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    set_prompt_callback(args, PromptSlot::Callback);
}

/// `prompt_setinterrupt({buf}, {callback})`.
pub fn f_prompt_setinterrupt(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    set_prompt_callback(args, PromptSlot::Interrupt);
}

/// Which of a prompt buffer's two callbacks to set.
enum PromptSlot {
    Callback,
    Interrupt,
}

/// The half `prompt_setcallback()` and `prompt_setinterrupt()` share: resolve
/// the buffer, build the callback, then free the one `slot` held.
///
/// Nothing is freed until the new callback has been built, so a bad second
/// argument leaves the old one in place.
fn set_prompt_callback(args: &[TypVal], slot: PromptSlot) {
    let mut callback = Callback::None;
    if check_secure() {
        return;
    }
    let Some(mut buf) = tv_get_buf(&args[0], 0) else {
        return;
    };
    if !callback_from_typval(&mut callback, &args[1]) {
        return;
    }
    let slot = match slot {
        PromptSlot::Callback => &mut buf.b_prompt_callback,
        PromptSlot::Interrupt => &mut buf.b_prompt_interrupt,
    };
    callback_free(slot);
    *slot = callback;
}

/// `prompt_setprompt({buf}, {text})`.
///
/// The prompt is stored on the buffer *and* written into the prompt line, so
/// changing it has to rewrite the line the old prompt is sitting in — unless
/// that line no longer starts with the old prompt, in which case the whole
/// line is replaced.
pub fn f_prompt_setprompt(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if check_secure() {
        return;
    }
    let Some(mut buf) = tv_get_buf(&args[0], 0) else {
        return;
    };
    let new_prompt = ThinCString::from_cstr(numbuf.string(&args[1]));
    let new_prompt_len = len_as_int(new_prompt.as_bytes().len());
    if buf_is_prompt(Some(buf)) && !buf.b_ml.ml_mfp.is_null() {
        rewrite_prompt_line(buf, new_prompt.as_bytes());
    }
    set_buf_prompt_text(buf, new_prompt);
    buf.b_prompt_start.mark.col = new_prompt_len;
}

/// Put `new_prompt` in place of the old one on the buffer's prompt line;
/// `buffer` is a loaded prompt buffer.
fn rewrite_prompt_line(mut buffer: Buf, new_prompt: &[u8]) {
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
    let new_prompt_len = len_as_int(new_prompt.len());
    let prompt_lno = buffer.b_prompt_start.mark.lnum;
    let old_prompt = buf_prompt_text_owned(buffer);
    let old_prompt = old_prompt.as_bytes();
    let old_line = Lines::in_buffer(buffer).line(prompt_lno).to_vec();
    let old_line_len = len_as_int(old_line.len());
    let old_prompt_len = len_as_int(old_prompt.len());
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
    let intact = fits && {
        let from = offset(prompt_col - old_prompt_len);
        old_line[from..from + old_prompt.len()] == *old_prompt
    };
    // The splice both arms report is the same shape: the whole of what was
    // there, replaced by the new prompt.
    let row = prompt_lno - 1;
    let splice = |old_len: c_int| {
        extmark_splice_cols(buffer, row, 0, old_len, new_prompt_len, kExtmarkNoUndo);
    };
    if intact {
        let new_line = [new_prompt, &old_line[offset(prompt_col)..]].concat();
        let _ = ml_replace_buf_text(buffer, prompt_lno, &new_line);
        splice(prompt_col);
        cursor_col += new_prompt_len - prompt_col;
    } else {
        let _ = ml_replace_buf_text(buffer, prompt_lno, new_prompt);
        splice(old_line_len);
        cursor_col = new_prompt_len;
    }
    let mut win = Win::current();
    if win.w_buffer == buffer && win.w_cursor.lnum == prompt_lno {
        win.w_cursor.col = cursor_col;
        check_cursor_col(win);
    }
    changed_lines(buffer, prompt_lno, 0, prompt_lno + 1, 0, true);
    u_clearallandblockfree(buffer);
}
