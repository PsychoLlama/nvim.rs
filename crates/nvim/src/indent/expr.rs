//! Indent computed by running something: 'indentexpr', and the built-in
//! Lisp indenter behind 'lisp'.

#![forbid(unsafe_code)]

use crate::strings::has_char;
use crate::winlayer::Buf;

use super::*;
use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cursor::check_cursor;
use crate::eval::eval_to_number;
use crate::eval::typval::tv_get_lnum;
use crate::eval::vars::set_vim_var_nr;
use crate::ex_docmd::handle_did_throw;
use crate::ex_eval::state::{did_throw, trylevel};
use crate::guard::Lock;
use crate::indent_c::{cindent_on, do_c_expr_indent};
use crate::mbyte::{c_char_info, c_next_cluster};
use crate::memline::Lines;
use crate::option::split_option_part;
use crate::option::vars::{P_LISPWORDS, p_debug, p_paste};
use crate::option::was_set_insecurely;
use crate::optionstr::{OptString, OptStringRef};
use crate::plines::win_charsize;
use crate::pos::lt;
use crate::runtime::state::current_sctx;
use crate::search::{findmatch, linewhite};
use crate::state::MODE_INSERT;
use crate::state::mode::{State, did_ai};
use crate::winlayer::Win;

/// The indent 'indentexpr' answers for the cursor line, or the line's
/// current indent when the expression failed.
pub fn get_expr_indent() -> c_int {
    let mut win = Win::current();
    let buf = Buf::current();
    // SAFETY: the caller's contract; `curwin` and `curbuf` are the current
    // window and buffer for the whole of this call.
    let use_sandbox = was_set_insecurely(win, kOptIndentexpr, OptionSetFlags::LOCAL);
    let save_sctx = current_sctx.get();
    // Saved because the expression can move the cursor via `:normal`.
    let (save_pos, save_curswant, save_set_curswant) =
        (win.w_cursor, win.w_curswant, win.w_set_curswant);
    set_vim_var_nr(Vv::Lnum, save_pos.lnum as VarNumber);

    let mut indent = {
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        let _locked = Lock::text();
        // SAFETY: as above.
        current_sctx.set(buf.b_p_script_ctx[kBufOptIndentexpr as usize]);
        // The expression is evaluated from a copy, because 'indentexpr' can
        // be changed while it is running.
        let expression = buf.b_p_inde.get();
        eval_to_number(&expression, true) as c_int
    };
    current_sctx.set(save_sctx);

    // Restore the cursor so that 'indentexpr' does not have to. Pretend to
    // be in Insert mode, which allows the cursor past end of line for the
    // "o" command.
    let save_state = State.get();
    State.set(MODE_INSERT);
    win.w_cursor = save_pos;
    win.w_curswant = save_curswant;
    win.w_set_curswant = save_set_curswant;
    check_cursor(win);
    State.set(save_state);

    // Reset `did_throw`, unless 'debug' has "throw" and we are inside a
    // try/catch.
    // SAFETY: 'debug' is a NUL-terminated option string.
    let debug_throw = p_debug(|value| has_char(value, 't' as c_int));
    if did_throw.get() && (!debug_throw || trylevel.get() == 0) {
        handle_did_throw();
        did_throw.set(false);
    }
    if indent < 0 {
        // The expression failed; keep the indent the line already has.
        indent = get_indent();
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
fn enclosing_open() -> Option<Pos> {
    // SAFETY: the caller's contract; `findmatch` answers a pointer into
    // static storage that stays valid until the next call.
    let __v = findmatch(None, '(' as c_int);
    let Some(paren) = __v else {
        return findmatch(None, '[' as c_int);
    };
    match findmatch(None, '[' as c_int) {
        Some(square) if !lt(square, paren) => Some(square),
        _ => Some(paren),
    }
}

/// The indent of the first previous non-blank line at the same paren level
/// as the cursor, searching back no further than `open`.
///
/// Leaves the cursor on the line it answered for, which is what makes
/// `get_indent` the answer.
fn same_level_indent(open: &Pos) -> Option<c_int> {
    // SAFETY: the caller's contract; the cursor stays on a real line because
    // the walk stops at `open`, which `findmatch` answered.
    let mut win = Win::current();
    let mut parencount = 0;
    loop {
        win.w_cursor.lnum -= 1;
        if win.w_cursor.lnum < open.lnum {
            return None;
        }
        if linewhite(win.w_cursor.lnum) {
            continue;
        }
        count_parens(Lines::current().line(win.w_cursor.lnum), &mut parencount);
        if parencount == 0 {
            return Some(get_indent());
        }
    }
}

/// Advances `that` over white space in `line`, adding what it is worth on
/// screen to `amount`. Both loops in [`indent_after_open`] are this.
///
/// `csarg`/`cstype` describe `line`.
fn skip_white_measuring(
    line: &[u8],
    that: &mut usize,
    amount: &mut c_int,
    cstype: CharsizeKind,
    csarg: &mut CharsizeArg<'_>,
) {
    // Past the end reads NUL, which is not white space.
    while ascii_iswhite(c_int::from(byte_at(line, *that))) {
        let byte = byte_at(line, *that);
        *amount += win_charsize(cstype, *amount, *that, int32_t::from(byte), csarg).width;
        *that += 1;
    }
}

/// The indent for a line that opens a new form: the column just after
/// `open`, plus whatever the form's own convention adds.
fn indent_after_open(open: &Pos) -> c_int {
    let mut win = Win::current();
    win.w_cursor.lnum = open.lnum;
    win.w_cursor.col = open.col;
    // The line `open` is on, which nothing below changes.
    let mut lines = Lines::current();
    let line = lines.line(open.lnum);
    let (mut csarg, cstype) = CharsizeArg::new(win, open.lnum, line);

    // Walk to `open`'s column, measuring what is before it.
    let mut that = 0;
    let mut chr = c_char_info(line, 0);
    let mut amount = 0;
    let mut col = open.col;
    while byte_at(line, that) != 0 && col > 0 {
        amount += win_charsize(cstype, amount, that, chr.value, &mut csarg).width;
        (that, chr) = c_next_cluster(line, that, chr);
        col -= 1;
    }
    let at = |i: usize| byte_at(line, i);

    // Some keywords indent their body rather than their arguments (the
    // non-standard-Lisp ones are Scheme special forms):
    //     (let ((a 1))       instead of    (let ((a 1))
    //       (...))                              (...))
    if (at(that) == b'(' || at(that) == b'[') && lisp_match(&line[that + 1..]) {
        return amount + 2;
    }
    if at(that) != 0 {
        that += 1;
        amount += 1;
    }
    let mut firsttry = amount;
    skip_white_measuring(line, &mut that, &mut amount, cstype, &mut csarg);
    if at(that) == 0 || at(that) == b';' {
        // A comment line, or nothing after the bracket at all.
        return amount;
    }
    // Not a comment. `(` is tested for so that the first argument of a
    // `let`/`do` can span more than one line.
    if at(that) != b'(' && at(that) != b'[' {
        firsttry += 1;
    }
    amount = measure_first_argument(line, &mut that, amount, cstype, &mut csarg);
    skip_white_measuring(line, &mut that, &mut amount, cstype, &mut csarg);
    if at(that) == 0 || at(that) == b';' {
        // Nothing followed the first argument on this line, so line the
        // continuation up with the argument instead of past it.
        firsttry
    } else {
        amount
    }
}

/// Walks over the form's first argument, starting at byte `that` of `line`,
/// measuring it, and answers where that leaves `amount`.
///
/// A quoted, `#`-prefixed or numeric argument is not walked at all — those
/// are values rather than forms, and upstream lines the rest up with the
/// bracket instead.
///
/// `csarg`/`cstype` describe `line`.
fn measure_first_argument(
    line: &[u8],
    that: &mut usize,
    mut amount: c_int,
    cstype: CharsizeKind,
    csarg: &mut CharsizeArg<'_>,
) -> c_int {
    let mut ci: CharInfo = c_char_info(line, *that);
    if ci.value == '"' as int32_t
        || ci.value == '\'' as int32_t
        || ci.value == '#' as int32_t
        || ('0' as int32_t..='9' as int32_t).contains(&ci.value)
    {
        return amount;
    }
    let mut parencount = 0;
    let mut quotecount = 0;
    while byte_at(line, *that) != 0
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
        if ci.value == '\\' as int32_t && byte_at(line, *that + 1) != 0 {
            amount += win_charsize(cstype, amount, *that, ci.value, csarg).width;
            (*that, ci) = c_next_cluster(line, *that, ci);
        }
        amount += win_charsize(cstype, amount, *that, ci.value, csarg).width;
        (*that, ci) = c_next_cluster(line, *that, ci);
    }
    amount
}

/// The indent the built-in Lisp indenter answers for the cursor line.
///
/// The rule is: take the indent of the first previous non-blank line at the
/// same bracket level, and failing that, line up after the bracket that
/// encloses this one.
pub fn get_lisp_indent() -> c_int {
    // SAFETY: the caller's contract; the cursor is put back before returning
    // whichever path answers.
    let mut win = Win::current();
    let realpos = win.w_cursor;
    win.w_cursor.col = 0;
    let amount = match enclosing_open() {
        // No enclosing '(' or '[': no indent.
        None => 0,
        Some(open) => match same_level_indent(&open) {
            Some(amount) => amount,
            None => indent_after_open(&open),
        },
    };
    win.w_cursor = realpos;
    amount
}

/// Whether `p` begins with one of 'lispwords', followed by white space or
/// the end of the line.
fn lisp_match(p: &[u8]) -> bool {
    // The buffer's own 'lispwords' where it has one, else the global value.
    let buf = Buf::current();
    let lispwords = if buf.b_p_lw.first_byte() != 0 {
        buf.b_p_lw.get()
    } else {
        P_LISPWORDS.get()
    };
    let mut rest: &[u8] = &lispwords;
    while !rest.is_empty() {
        // Upstream copies each word into a 512-byte buffer.
        let (word, after) = split_option_part(rest, 512, b",");
        rest = after;
        if p.starts_with(&word) && ascii_iswhite_or_nul(c_int::from(byte_at(p, word.len()))) {
            return true;
        }
    }
    false
}

/// Re-indents the cursor line to whatever `get_the_indent` says, which is
/// one of `get_c_indent`, [`get_expr_indent`] and [`get_lisp_indent`].
pub fn fixthisline(get_the_indent: Indenter) {
    let amount = get_the_indent.amount();
    if amount < 0 {
        return;
    }
    change_indent(INDENT_SET as c_int, amount, 0, true);
    if linewhite(Win::current().w_cursor.lnum) {
        // Delete the indent again if the line stays empty.
        did_ai.set(true);
    }
}

/// Whether 'indentexpr' should be used for Lisp indenting. The caller may
/// want to check 'autoindent' as well.
pub fn use_indentexpr_for_lisp() -> bool {
    let buf = Buf::current();
    buf.b_p_lisp != 0 && buf.b_p_inde.first_byte() != 0 && buf.b_p_lop.bytes() == b"expr:1"
}

/// Fixes the cursor line's indent for 'lisp' and 'cindent'.
pub fn fix_indent() {
    if p_paste() {
        return; // no auto-indenting when 'paste' is set
    }
    // SAFETY: the caller's contract.
    let buf = Buf::current();
    if buf.b_p_lisp != 0 && buf.b_p_ai != 0 {
        if use_indentexpr_for_lisp() {
            do_c_expr_indent();
        } else {
            fixthisline(Indenter::Lisp);
        }
    } else if cindent_on() {
        do_c_expr_indent();
    }
}

/// `indent()`.
pub fn f_indent(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let lnum = tv_get_lnum(&args[0]);
    result.write_number(if (1..=Buf::current().b_ml.ml_line_count).contains(&lnum) {
        get_indent_lnum(lnum) as VarNumber
    } else {
        -1
    });
}
/// `lispindent(lnum)`.
pub fn f_lispindent(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's typvals; the cursor is moved onto the asked-for
    // line and put back.
    let mut win = Win::current();
    let pos = win.w_cursor;
    let lnum = tv_get_lnum(&args[0]);
    (*result).write_number(if (1..=Buf::current().b_ml.ml_line_count).contains(&lnum) {
        win.w_cursor.lnum = lnum;
        let amount = get_lisp_indent() as VarNumber;
        win.w_cursor = pos;
        amount
    } else {
        -1
    });
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
