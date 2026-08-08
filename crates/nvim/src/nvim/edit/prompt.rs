//! The prompt buffer: an Insert mode with a read-only prefix.
//!
//! A 'buftype' of "prompt" makes the last line a prompt the user types
//! after and cannot back over.  `init_prompt` is what runs on entering
//! Insert mode in such a buffer: make sure the last line exists and starts
//! with the prompt text, and put the cursor after it.  `buf_prompt_text`
//! resolves 'b:prompt_text' against the default, and `prompt_curpos_editable`
//! is the guard `ins_bs` and the cursor motions ask before moving left.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn buf_prompt_text(buf: *const buf_T) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*buf).b_prompt_text.is_null() {
            return b"% \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        return (*buf).b_prompt_text;
    }
}

pub unsafe extern "C" fn prompt_text() -> *mut ::core::ffi::c_char {
    unsafe {
        return buf_prompt_text(curbuf.get());
    }
}

pub(crate) unsafe extern "C" fn init_prompt(mut cmdchar_todo: ::core::ffi::c_int) {
    unsafe {
        let mut prompt: *mut ::core::ffi::c_char = prompt_text();
        let mut prompt_len: ::core::ffi::c_int = strlen(prompt) as ::core::ffi::c_int;
        if (*curbuf.get()).b_prompt_start.mark.lnum < 1 as linenr_T
            || (*curbuf.get()).b_prompt_start.mark.lnum > (*curbuf.get()).b_ml.ml_line_count
        {
            (*curbuf.get()).b_prompt_start.mark.lnum = if 1 as linenr_T
                > (if (*curbuf.get()).b_prompt_start.mark.lnum < (*curbuf.get()).b_ml.ml_line_count
                {
                    (*curbuf.get()).b_prompt_start.mark.lnum
                } else {
                    (*curbuf.get()).b_ml.ml_line_count
                }) {
                1 as linenr_T
            } else if (*curbuf.get()).b_prompt_start.mark.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                (*curbuf.get()).b_prompt_start.mark.lnum
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
        }
        (*curwin.get()).w_cursor.lnum =
            if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_prompt_start.mark.lnum {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*curbuf.get()).b_prompt_start.mark.lnum
            };
        let mut text: *mut ::core::ffi::c_char = ml_get((*curbuf.get()).b_prompt_start.mark.lnum);
        let mut text_len: colnr_T = ml_get_len((*curbuf.get()).b_prompt_start.mark.lnum);
        if (*curbuf.get()).b_prompt_start.mark.lnum == (*curwin.get()).w_cursor.lnum
            && ((*curbuf.get()).b_prompt_start.mark.col < prompt_len
                || (*curbuf.get()).b_prompt_start.mark.col > text_len
                || !strnequal(
                    text.offset((*curbuf.get()).b_prompt_start.mark.col as isize)
                        .offset(-(prompt_len as isize)),
                    prompt,
                    prompt_len as size_t,
                ))
        {
            if *text as ::core::ffi::c_int == NUL {
                ml_replace(
                    (*curbuf.get()).b_prompt_start.mark.lnum,
                    prompt,
                    true_0 != 0,
                );
                inserted_bytes(
                    (*curbuf.get()).b_prompt_start.mark.lnum,
                    0 as colnr_T,
                    0 as ::core::ffi::c_int,
                    prompt_len,
                );
            } else {
                let lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
                ml_append(lnum, prompt, 0 as colnr_T, false_0 != 0);
                appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
                (*curbuf.get()).b_prompt_start.mark.lnum = (*curbuf.get()).b_ml.ml_line_count;
                (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
                u_clearallandblockfree(curbuf.get());
            }
            (*curbuf.get()).b_prompt_start.mark.col = prompt_len as colnr_T;
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        if (*Insstart_orig.ptr()).lnum != (*curbuf.get()).b_prompt_start.mark.lnum
            || (*Insstart_orig.ptr()).col != (*curbuf.get()).b_prompt_start.mark.col
        {
            (*Insstart.ptr()).lnum = (*curbuf.get()).b_prompt_start.mark.lnum;
            (*Insstart.ptr()).col = (*curbuf.get()).b_prompt_start.mark.col;
            Insstart_orig.set(Insstart.get());
            Insstart_textlen.set((*Insstart.ptr()).col);
            Insstart_blank_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
            arrow_used.set(false_0 != 0);
        }
        if cmdchar_todo == 'A' as ::core::ffi::c_int {
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        if (*curbuf.get()).b_prompt_start.mark.lnum == (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.col =
                if (*curwin.get()).w_cursor.col > (*curbuf.get()).b_prompt_start.mark.col {
                    (*curwin.get()).w_cursor.col
                } else {
                    (*curbuf.get()).b_prompt_start.mark.col
                };
        }
        check_cursor(curwin.get());
    }
}

pub unsafe extern "C" fn prompt_curpos_editable() -> bool {
    unsafe {
        return (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_prompt_start.mark.lnum
            || (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum
                && (*curwin.get()).w_cursor.col >= (*curbuf.get()).b_prompt_start.mark.col;
    }
}
