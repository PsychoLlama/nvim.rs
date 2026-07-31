use super::lines::set_buffer_lines;
use super::*;
use crate::src::nvim::eval::typval::kCallbackNone;

/// "prompt_appendbuf({buffer}, string/list)" function
pub unsafe extern "C" fn f_prompt_appendbuf(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let did_emsg_before: c_int = did_emsg.get();
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 1;
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0));
    if buf.is_null() || !bt_prompt(buf) {
        return;
    }
    let lnum: linenr_T = ((*buf).b_prompt_start.mark.lnum - 1).max(0);
    let lines: *mut typval_T = argvars.offset(1);
    let mut did_concat: bool = false;
    if !(*buf).b_prompt_append_new_line {
        let text: *const c_char = if lnum > 0 {
            ml_get_buf(buf, lnum) as *const c_char
        } else {
            c"".as_ptr()
        };
        if (*lines).v_type == VAR_LIST {
            let mut l: *mut list_T = (*lines).vval.v_list;
            if !l.is_null() && (*l).lv_len > 0 {
                let li: *mut listitem_T = (*l).lv_first;
                let new_str = concat_str(text, tv_get_string(&raw mut (*li).li_tv));
                tv_clear(&raw mut (*li).li_tv);
                (*li).li_tv.v_type = VAR_STRING;
                (*li).li_tv.vval.v_string = new_str;
                did_concat = true;
            }
        } else if (*lines).v_type == VAR_STRING {
            let new_str = concat_str(text, tv_get_string(lines));
            tv_clear(lines);
            (*lines).v_type = VAR_STRING;
            (*lines).vval.v_string = new_str;
        }
    }
    if did_emsg.get() == did_emsg_before {
        if did_concat && (*(*lines).vval.v_list).lv_len > 1 {
            let l_0: *mut list_T = (*lines).vval.v_list;
            let li_0: *mut listitem_T = (*l_0).lv_first;
            set_buffer_lines(buf, lnum, false, &raw mut (*li_0).li_tv, rettv);
            if (*rettv).vval.v_number == 0 {
                tv_list_item_remove(l_0, li_0);
                set_buffer_lines(buf, lnum, true, lines, rettv);
            }
        } else {
            set_buffer_lines(buf, lnum, (*buf).b_prompt_append_new_line, lines, rettv);
        }
    }
    if (*rettv).vval.v_number == 0 {
        // A trailing newline on the last line appended asks the next append
        // to start a fresh line rather than extending this one.
        let ends_in_newline = |s: *const c_char| {
            let len = strlen(s);
            len > 0 && *s.add(len - 1) == b'\n' as c_char
        };
        (*buf).b_prompt_append_new_line = if (*lines).v_type == VAR_LIST {
            let l: *mut list_T = (*lines).vval.v_list;
            !l.is_null()
                && (*l).lv_len > 0
                && ends_in_newline(tv_get_string(&raw mut (*(*l).lv_last).li_tv))
        } else {
            (*lines).v_type == VAR_STRING && ends_in_newline(tv_get_string(lines))
        };
    }
}
/// "prompt_setcallback({buffer}, {callback})" function
pub unsafe extern "C" fn f_prompt_setcallback(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut prompt_callback: Callback = Callback {
        data: Callback_data {
            funcref: ptr::null_mut(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(&raw mut prompt_callback, argvars.offset(1)) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_callback);
    (*buf).b_prompt_callback = prompt_callback;
}
/// "prompt_setinterrupt({buffer}, {callback})" function
pub unsafe extern "C" fn f_prompt_setinterrupt(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut interrupt_callback: Callback = Callback {
        data: Callback_data {
            funcref: ptr::null_mut(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(&raw mut interrupt_callback, argvars.offset(1)) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_interrupt);
    (*buf).b_prompt_interrupt = interrupt_callback;
}
/// "prompt_setprompt({buffer}, {text})" function
pub unsafe extern "C" fn f_prompt_setprompt(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0), false_0);
    if buf.is_null() {
        return;
    }
    let new_prompt: *const c_char = tv_get_string(argvars.offset(1));
    let new_prompt_len: c_int = strlen(new_prompt) as c_int;
    if bt_prompt(buf) && !(*buf).b_ml.ml_mfp.is_null() {
        if (*buf).b_prompt_start.mark.lnum < 1
            || (*buf).b_prompt_start.mark.lnum > (*curbuf.get()).b_ml.ml_line_count
        {
            // MAX(1, MIN(lnum, line_count)); spelled with min-then-max
            // because an empty buffer makes the two bounds cross.
            (*buf).b_prompt_start.mark.lnum = (*buf)
                .b_prompt_start
                .mark
                .lnum
                .min((*buf).b_ml.ml_line_count)
                .max(1);
            (*curbuf.get()).b_prompt_append_new_line = true;
        }
        let prompt_lno: linenr_T = (*buf).b_prompt_start.mark.lnum;
        let old_prompt: *mut c_char = buf_prompt_text(buf);
        let old_line: *mut c_char = ml_get_buf(buf, prompt_lno);
        let old_line_len: colnr_T = ml_get_buf_len(buf, prompt_lno);
        let old_prompt_len: c_int = strlen(old_prompt) as c_int;
        let mut cursor_col: colnr_T = (*curwin.get()).w_cursor.col;
        if (*buf).b_prompt_start.mark.col < old_prompt_len
            || (*buf).b_prompt_start.mark.col > old_line_len
            || !strnequal(
                old_prompt,
                old_line
                    .offset((*buf).b_prompt_start.mark.col as isize)
                    .offset(-(old_prompt_len as isize)),
                old_prompt_len as size_t,
            )
        {
            ml_replace_buf(buf, prompt_lno, new_prompt as *mut c_char, true, false);
            extmark_splice_cols(
                buf,
                prompt_lno as c_int - 1,
                0,
                old_line_len,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col = new_prompt_len as colnr_T;
        } else {
            let new_line: *mut c_char = concat_str(
                new_prompt,
                old_line.offset((*buf).b_prompt_start.mark.col as isize),
            );
            if ml_replace_buf(buf, prompt_lno, new_line, false, false) != OK {
                xfree(new_line as *mut c_void);
            }
            extmark_splice_cols(
                buf,
                prompt_lno as c_int - 1,
                0,
                (*buf).b_prompt_start.mark.col,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col += (new_prompt_len as colnr_T - (*buf).b_prompt_start.mark.col) as c_int;
        }
        if (*curwin.get()).w_buffer == buf && (*curwin.get()).w_cursor.lnum == prompt_lno {
            (*curwin.get()).w_cursor.col = cursor_col;
            check_cursor_col(curwin.get());
        }
        changed_lines(buf, prompt_lno, 0, prompt_lno + 1, 0, true);
        u_clearallandblockfree(buf);
    }
    xfree((*buf).b_prompt_text as *mut c_void);
    (*buf).b_prompt_text = xstrdup(new_prompt);
    (*buf).b_prompt_start.mark.col = new_prompt_len;
}
