//! Indent computed by running something: 'indentexpr', and the built-in
//! Lisp indenter behind 'lisp'.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;

use super::*;
use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cursor::{check_cursor, get_cursor_line_ptr};
use crate::eval::eval_to_number;
use crate::eval::typval::tv_get_lnum;
use crate::eval::vars::set_vim_var_nr;
use crate::ex_docmd::handle_did_throw;
use crate::guard::Lock;
use crate::indent_c::{cindent_on, do_c_expr_indent};
use crate::main::{
    State, current_sctx, did_ai, did_throw, p_debug, p_lispwords, p_paste, trylevel,
};
use crate::mbyte::{utf_ptr2char_info, utf_ptr2str_char_info, utfc_next};
use crate::memory::{xfree, xstrdup};
use crate::option::{copy_option_part, was_set_insecurely};
use crate::os::cshim::strncmp;
use crate::plines::{init_charsize_arg, win_charsize};
use crate::pos::lt;
use crate::search::{findmatch, linewhite};
use crate::state::MODE_INSERT;
use crate::strings::vim_strchr;
use ::libc::strcmp;

/// The indent 'indentexpr' answers for the cursor line, or the line's
/// current indent when the expression failed.
///
/// # Safety
/// There must be a current window and buffer.
pub unsafe fn get_expr_indent() -> c_int {
    let win = curwin.get();
    let buf = curbuf.get();
    // SAFETY: the caller's contract; `curwin` and `curbuf` are the current
    // window and buffer for the whole of this call.
    let use_sandbox = unsafe { was_set_insecurely(win, kOptIndentexpr, OptionSetFlags::LOCAL) };
    let save_sctx = current_sctx.get();
    // Saved because the expression can move the cursor via `:normal`.
    // SAFETY: as above.
    let (save_pos, save_curswant, save_set_curswant) =
        unsafe { ((*win).w_cursor, (*win).w_curswant, (*win).w_set_curswant) };
    // SAFETY: as above.
    unsafe { set_vim_var_nr(Vv::Lnum, save_pos.lnum as varnumber_T) };

    let mut indent = {
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        let _locked = Lock::text();
        // SAFETY: as above.
        current_sctx.set(unsafe { (*buf).b_p_script_ctx[kBufOptIndentexpr as usize] });
        // SAFETY: as above. The expression is evaluated from a copy, because
        // 'indentexpr' can be changed while it is running.
        unsafe {
            let inde_copy = xstrdup((*buf).b_p_inde);
            let answer = eval_to_number(inde_copy, true) as c_int;
            xfree(inde_copy.cast());
            answer
        }
    };
    current_sctx.set(save_sctx);

    // Restore the cursor so that 'indentexpr' does not have to. Pretend to
    // be in Insert mode, which allows the cursor past end of line for the
    // "o" command.
    let save_state = State.get();
    State.set(MODE_INSERT);
    // SAFETY: as above.
    unsafe {
        (*win).w_cursor = save_pos;
        (*win).w_curswant = save_curswant;
        (*win).w_set_curswant = save_set_curswant;
        check_cursor(win);
    }
    State.set(save_state);

    // Reset `did_throw`, unless 'debug' has "throw" and we are inside a
    // try/catch.
    // SAFETY: 'debug' is a NUL-terminated option string.
    let debug_throw = !unsafe { vim_strchr(p_debug.get(), 't' as c_int) }.is_null();
    if did_throw.get() && (!debug_throw || trylevel.get() == 0) {
        // SAFETY: as above.
        unsafe { handle_did_throw() };
        did_throw.set(false);
    }
    if indent < 0 {
        // The expression failed; keep the indent the line already has.
        // SAFETY: as above.
        indent = unsafe { get_indent() };
    }
    indent
}

/// Adds one line's net `(`/`[` depth to `parencount`, skipping what Lisp
/// does not read as syntax: a `;` comment to end of line, a backslash and
/// the byte after it, and the contents of a string.
///
/// An unterminated string stops the scan, which is why this walks the line
/// itself rather than counting bytes.
fn count_parens(line: &[u8], parencount: &mut c_int) {
    let mut i = 0;
    while i < line.len() {
        match line[i] {
            // The rest of the line is a comment.
            b';' => return,
            // The escaped byte is not syntax; the step below skips it.
            b'\\' => i += 1,
            b'"' if i + 1 < line.len() => {
                let mut t = i;
                loop {
                    t += 1;
                    if t >= line.len() || line[t] == b'"' {
                        break;
                    }
                    if line[t] == b'\\' {
                        t += 1;
                        if t >= line.len() {
                            break;
                        }
                        if t + 1 >= line.len() {
                            t += 1;
                            break;
                        }
                    }
                }
                if t >= line.len() {
                    // Unterminated: nothing after it can be read.
                    return;
                }
                i = t; // the closing quote, which the step below moves past
            }
            b'(' | b'[' => *parencount += 1,
            b')' | b']' => *parencount -= 1,
            _ => {}
        }
        i += 1;
    }
}

/// The `(` or `[` that encloses the cursor line, whichever starts later.
///
/// # Safety
/// There must be a current window.
unsafe fn enclosing_open() -> Option<pos_T> {
    // SAFETY: the caller's contract; `findmatch` answers a pointer into
    // static storage that stays valid until the next call.
    unsafe {
        let round = findmatch(::core::ptr::null_mut(), '(' as c_int);
        if round.is_null() {
            let square = findmatch(::core::ptr::null_mut(), '[' as c_int);
            return (!square.is_null()).then(|| *square);
        }
        let paren = *round;
        let square = findmatch(::core::ptr::null_mut(), '[' as c_int);
        if square.is_null() || lt(*square, paren) {
            Some(paren)
        } else {
            Some(*square)
        }
    }
}

/// The indent of the first previous non-blank line at the same paren level
/// as the cursor, searching back no further than `open`.
///
/// Leaves the cursor on the line it answered for, which is what makes
/// `get_indent` the answer.
///
/// # Safety
/// There must be a current window and buffer.
unsafe fn same_level_indent(open: &pos_T) -> Option<c_int> {
    // SAFETY: the caller's contract; the cursor stays on a real line because
    // the walk stops at `open`, which `findmatch` answered.
    unsafe {
        let win = curwin.get();
        let mut parencount = 0;
        loop {
            (*win).w_cursor.lnum -= 1;
            if (*win).w_cursor.lnum < open.lnum {
                return None;
            }
            if linewhite((*win).w_cursor.lnum) {
                continue;
            }
            count_parens(
                CStr::from_ptr(get_cursor_line_ptr()).to_bytes(),
                &mut parencount,
            );
            if parencount == 0 {
                return Some(get_indent());
            }
        }
    }
}

/// Advances `that` over white space, adding what it is worth on screen to
/// `amount`. Both loops in [`indent_after_open`] are this.
///
/// # Safety
/// `that` must address a NUL-terminated string, and `csarg`/`cstype` must
/// describe the line it points into.
unsafe fn skip_white_measuring(
    that: &mut *mut c_char,
    amount: &mut c_int,
    cstype: CharsizeKind,
    csarg: &mut CharsizeArg,
) {
    // SAFETY: the caller's line, walked one byte at a time and stopped by
    // the NUL, which is not white space.
    unsafe {
        while ascii_iswhite(**that as c_int) {
            *amount +=
                win_charsize(cstype, *amount, *that, **that as uint8_t as int32_t, csarg).width;
            *that = that.add(1);
        }
    }
}

/// The indent for a line that opens a new form: the column just after
/// `open`, plus whatever the form's own convention adds.
///
/// # Safety
/// `open` must be a position in the current buffer.
unsafe fn indent_after_open(open: &pos_T) -> c_int {
    // SAFETY: the caller's position; the cursor is moved onto it first, so
    // `get_cursor_line_ptr` is the line `open.col` indexes into.
    unsafe {
        let win = curwin.get();
        (*win).w_cursor.lnum = open.lnum;
        (*win).w_cursor.col = open.col;
        let line = get_cursor_line_ptr();
        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(&mut csarg, win, open.lnum, line);

        // Walk to `open`'s column, measuring what is before it.
        let mut sci: StrCharInfo = utf_ptr2str_char_info(line);
        let mut amount = 0;
        let mut col = open.col;
        while *sci.ptr != 0 && col > 0 {
            amount += win_charsize(cstype, amount, sci.ptr, sci.chr.value, &mut csarg).width;
            sci = utfc_next(sci);
            col -= 1;
        }
        let mut that = sci.ptr;

        // Some keywords indent their body rather than their arguments (the
        // non-standard-Lisp ones are Scheme special forms):
        //     (let ((a 1))       instead of    (let ((a 1))
        //       (...))                              (...))
        if (*that == b'(' as c_char || *that == b'[' as c_char) && lisp_match(that.add(1)) {
            return amount + 2;
        }
        if *that != 0 {
            that = that.add(1);
            amount += 1;
        }
        let mut firsttry = amount;
        skip_white_measuring(&mut that, &mut amount, cstype, &mut csarg);
        if *that == 0 || *that == b';' as c_char {
            // A comment line, or nothing after the bracket at all.
            return amount;
        }
        // Not a comment. `(` is tested for so that the first argument of a
        // `let`/`do` can span more than one line.
        if *that != b'(' as c_char && *that != b'[' as c_char {
            firsttry += 1;
        }
        amount = measure_first_argument(&mut that, amount, cstype, &mut csarg);
        skip_white_measuring(&mut that, &mut amount, cstype, &mut csarg);
        if *that == 0 || *that == b';' as c_char {
            // Nothing followed the first argument on this line, so line the
            // continuation up with the argument instead of past it.
            firsttry
        } else {
            amount
        }
    }
}

/// Walks over the form's first argument, measuring it, and answers where
/// that leaves `amount`.
///
/// A quoted, `#`-prefixed or numeric argument is not walked at all — those
/// are values rather than forms, and upstream lines the rest up with the
/// bracket instead.
///
/// # Safety
/// `that` must address a NUL-terminated string, and `csarg`/`cstype` must
/// describe the line it points into.
unsafe fn measure_first_argument(
    that: &mut *mut c_char,
    mut amount: c_int,
    cstype: CharsizeKind,
    csarg: &mut CharsizeArg,
) -> c_int {
    // SAFETY: the caller's line, walked one character at a time by
    // `utfc_next` and stopped by the NUL.
    unsafe {
        let mut ci: CharInfo = utf_ptr2char_info(*that);
        if ci.value == '"' as int32_t
            || ci.value == '\'' as int32_t
            || ci.value == '#' as int32_t
            || ('0' as int32_t..='9' as int32_t).contains(&ci.value)
        {
            return amount;
        }
        let mut parencount = 0;
        let mut quotecount = 0;
        while **that != 0
            && (!ascii_iswhite(ci.value as c_int) || quotecount != 0 || parencount != 0)
        {
            if ci.value == '"' as int32_t {
                quotecount = (quotecount == 0) as c_int;
            }
            if quotecount == 0 {
                if ci.value == '(' as int32_t || ci.value == '[' as int32_t {
                    parencount += 1;
                } else if ci.value == ')' as int32_t || ci.value == ']' as int32_t {
                    parencount -= 1;
                }
            }
            // A backslash and the character it escapes are one step.
            if ci.value == '\\' as int32_t && *that.add(1) != 0 {
                amount += win_charsize(cstype, amount, *that, ci.value, csarg).width;
                let next = utfc_next(StrCharInfo {
                    ptr: *that,
                    chr: ci,
                });
                *that = next.ptr;
                ci = next.chr;
            }
            amount += win_charsize(cstype, amount, *that, ci.value, csarg).width;
            let next = utfc_next(StrCharInfo {
                ptr: *that,
                chr: ci,
            });
            *that = next.ptr;
            ci = next.chr;
        }
        amount
    }
}

/// The indent the built-in Lisp indenter answers for the cursor line.
///
/// The rule is: take the indent of the first previous non-blank line at the
/// same bracket level, and failing that, line up after the bracket that
/// encloses this one.
///
/// # Safety
/// There must be a current window and buffer.
pub unsafe fn get_lisp_indent() -> c_int {
    // SAFETY: the caller's contract; the cursor is put back before returning
    // whichever path answers.
    unsafe {
        let win = curwin.get();
        let realpos = (*win).w_cursor;
        (*win).w_cursor.col = 0;
        let amount = match enclosing_open() {
            // No enclosing '(' or '[': no indent.
            None => 0,
            Some(open) => match same_level_indent(&open) {
                Some(amount) => amount,
                None => indent_after_open(&open),
            },
        };
        (*win).w_cursor = realpos;
        amount
    }
}

/// Whether `p` begins with one of 'lispwords', followed by white space or
/// the end of the line.
///
/// # Safety
/// `p` must be a NUL-terminated string.
unsafe fn lisp_match(p: *mut c_char) -> bool {
    // SAFETY: the caller's string, and `buf` is this frame's;
    // `copy_option_part` bounds its copy by the size it is given.
    unsafe {
        let mut buf: [c_char; 512] = [0; 512];
        let mut word = if *(*curbuf.get()).b_p_lw != 0 {
            (*curbuf.get()).b_p_lw
        } else {
            p_lispwords.get()
        };
        while *word != 0 {
            let len = copy_option_part(
                &raw mut word,
                buf.as_mut_ptr(),
                buf.len(),
                c",".as_ptr().cast_mut(),
            );
            if strncmp(buf.as_ptr(), p, len) == 0 && ascii_iswhite_or_nul(*p.add(len) as c_int) {
                return true;
            }
        }
        false
    }
}

/// Re-indents the cursor line to whatever `get_the_indent` says, which is
/// one of `get_c_indent`, [`get_expr_indent`] and [`get_lisp_indent`].
///
/// # Safety
/// There must be a current window and buffer, and the line must be
/// modifiable.
pub unsafe fn fixthisline(get_the_indent: IndentGetter) {
    // SAFETY: the caller's contract; `get_the_indent` is one of the three
    // indent engines, all of which read the current buffer.
    unsafe {
        let amount = get_the_indent.expect("non-null function pointer")();
        if amount < 0 {
            return;
        }
        change_indent(INDENT_SET as c_int, amount, 0, true);
        if linewhite((*curwin.get()).w_cursor.lnum) {
            // Delete the indent again if the line stays empty.
            did_ai.set(true);
        }
    }
}

/// Whether 'indentexpr' should be used for Lisp indenting. The caller may
/// want to check 'autoindent' as well.
///
/// # Safety
/// There must be a current buffer.
pub unsafe fn use_indentexpr_for_lisp() -> bool {
    // SAFETY: the caller's contract.
    unsafe {
        let buf = curbuf.get();
        (*buf).b_p_lisp != 0
            && *(*buf).b_p_inde != 0
            && strcmp((*buf).b_p_lop, c"expr:1".as_ptr()) == 0
    }
}

/// Fixes the cursor line's indent for 'lisp' and 'cindent'.
///
/// # Safety
/// There must be a current window and buffer.
pub unsafe fn fix_indent() {
    if p_paste.get() != 0 {
        return; // no auto-indenting when 'paste' is set
    }
    // SAFETY: the caller's contract.
    unsafe {
        let buf = curbuf.get();
        if (*buf).b_p_lisp != 0 && (*buf).b_p_ai != 0 {
            if use_indentexpr_for_lisp() {
                do_c_expr_indent();
            } else {
                fixthisline(Some(get_lisp_indent));
            }
        } else if cindent_on() {
            do_c_expr_indent();
        }
    }
}

/// `indent()`.
///
/// # Safety
/// The evaluator's contract: `argvars` and `rettv` are live typvals.
pub unsafe fn f_indent(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's typvals, and there is a current buffer.
    unsafe {
        let lnum = tv_get_lnum(argvars);
        (*rettv).vval.v_number = if (1..=(*curbuf.get()).b_ml.ml_line_count).contains(&lnum) {
            get_indent_lnum(lnum) as varnumber_T
        } else {
            -1
        };
    }
}

/// `lispindent(lnum)`.
///
/// # Safety
/// The evaluator's contract: `argvars` and `rettv` are live typvals.
pub unsafe fn f_lispindent(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's typvals; the cursor is moved onto the asked-for
    // line and put back.
    unsafe {
        let win = curwin.get();
        let pos = (*win).w_cursor;
        let lnum = tv_get_lnum(argvars);
        (*rettv).vval.v_number = if (1..=(*curbuf.get()).b_ml.ml_line_count).contains(&lnum) {
            (*win).w_cursor.lnum = lnum;
            let amount = get_lisp_indent() as varnumber_T;
            (*win).w_cursor = pos;
            amount
        } else {
            -1
        };
    }
}

#[cfg(test)]
mod tests {
    use super::count_parens;

    fn depth(line: &str) -> i32 {
        let mut n = 0;
        count_parens(line.as_bytes(), &mut n);
        n
    }

    #[test]
    fn count_parens_counts_both_bracket_kinds() {
        assert_eq!(depth("(a [b] c)"), 0);
        assert_eq!(depth("(let ((a 1))"), 1);
        assert_eq!(depth("))]"), -3);
    }

    #[test]
    fn count_parens_skips_comments_escapes_and_strings() {
        assert_eq!(depth("(a ; ((("), 1);
        assert_eq!(depth(r"(a \( b"), 1);
        assert_eq!(depth(r#"(a "((" b"#), 1);
        assert_eq!(depth(r#"(a "\"((" b"#), 1);
        // An unterminated string stops the scan, so the ')' is not seen.
        assert_eq!(depth("(a \"bb) "), 1);
        // A quote as the line's last byte is an ordinary character.
        assert_eq!(depth("(a\""), 1);
    }
}
