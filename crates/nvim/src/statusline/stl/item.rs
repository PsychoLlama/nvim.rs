//! The item loop: reading one `%` item and laying its value out.
//!
//! `run()` walks the format once, copying literal text through and handing
//! each `%` to the arm that knows it. Most arms are a question for the window
//! or the buffer, asked through [`Env`]; the ones that are not are the four
//! that record something instead of printing it (`%=`, `%<`, `%(`, `%)`), the
//! three that overload the width field with an argument (`%N*`, `%NT`/`%NX`,
//! `%@Func@`), and `%{}`, which evaluates.
//!
//! What every arm produces is a [`Value`] -- a number, or text in the
//! caller's scratch buffer -- and [`emit`] is the one place that turns that
//! into output, applying the item's width, alignment and fill.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::memline::MlFlags;
use core::ffi::c_int;
use core::ptr;

use super::{
    Env, Fill, Kind, MAX_STL_EVAL_DEPTH, NumberBase, STL_BYTEVAL_X, STL_CLICK_FUNC, STL_FILENAME,
    STL_FOLDCOL, STL_FULLPATH, STL_HELPFLAG_ALT, STL_HIGHLIGHT, STL_HIGHLIGHT_COMB,
    STL_MODIFIED_ALT, STL_OFFSET_X, STL_PREVIEWFLAG_ALT, STL_ROFLAG_ALT, STL_SEPARATE, STL_SIGNCOL,
    STL_TABCLOSENR, STL_TABPAGENR, STL_TRUNCMARK, STL_USER_HL, STL_VIM_EXPR, STL_VIRTCOL_ALT,
    StlItem, StlScratch, TMPLEN, as_number, cells_at, char_len_at, dup_cstring, group,
    in_insert_mode, kNumBaseDecimal, kNumBaseHexadecimal, parse, put_number, strsize_at, syntax_id,
    tr, upper, vim_var, with_scratch,
};
use crate::cstr;
use crate::decoration::SCL_NUM;
use crate::types::Vv;

/// What carries across items while the format is walked.
struct State {
    /// How many `%(` are open.
    groupdepth: c_int,
    /// How many `%{%...%}` blocks the format has been rewritten by.
    evaldepth: c_int,
    /// Whether the last thing written was a flag item, which is what lets
    /// the next flag drop its leading blank.
    prevchar_isflag: bool,
    /// Whether the last thing written was an item at all, which is what lets
    /// a flag drop its leading comma.
    prevchar_isitem: bool,
}

/// What one item evaluated to, apart from its text.
struct Value {
    /// The number to print, or negative for "no number".
    num: c_int,
    /// Whether the number prints in hexadecimal.
    base: NumberBase,
    /// Whether this is a flag like `[+]` or `,RO`, whose leading separator
    /// is dropped depending on what came before it.
    itemisflag: bool,
    /// Whether blanks in the text are replaced by the fill character.
    fillable: bool,
    /// Where the `'statuscolumn'` sign or fold items this item recorded
    /// start, which right-padding has to shift along with the text.
    foldsignitem: Option<usize>,
    /// A `'statuscolumn'` number item that gets a `%=` of its own after it.
    left_align_num: bool,
}

impl Default for Value {
    fn default() -> Self {
        Value {
            num: -1,
            base: kNumBaseDecimal,
            itemisflag: false,
            fillable: true,
            foldsignitem: None,
            left_align_num: false,
        }
    }
}

/// What a `%{}` item did.
enum Expr {
    /// It produced a value; the text, if any, is in the scratch buffer.
    Value { num: c_int, itemisflag: bool },
    /// `%{%...%}` whose result has items of its own: splice it into the
    /// format at `block_start` and read the format again.
    Reparse { block_start: usize, result: Vec<u8> },
}

/// Walk `usefmt`, answering where in `out` the text ends.
pub(super) fn run(
    env: &Env,
    out: &mut [u8],
    usefmt: Vec<u8>,
    fill: &Fill,
    discard_clicks: bool,
) -> usize {
    // The last byte of the buffer is reserved for the NUL, so every visible
    // character must land before it.
    let end = out.len() - 1;
    let mut fmt = usefmt;
    let mut p = 0usize;
    let mut pos = 0usize;
    let mut text: Vec<u8> = Vec::new();
    let mut st = State {
        groupdepth: 0,
        evaldepth: 0,
        prevchar_isflag: true,
        prevchar_isitem: false,
    };

    while p < fmt.len() {
        if fmt[p] != b'%' {
            st.prevchar_isflag = false;
            st.prevchar_isitem = false;
        }
        // Copy the format verbatim until the next item, the end of the
        // format, or the end of the output buffer. Upstream steps a byte at
        // a time; the run is the same either way.
        let run = fmt[p..]
            .iter()
            .position(|&byte| byte == b'%')
            .unwrap_or(fmt.len() - p)
            .min(end - pos);
        out[pos..pos + run].copy_from_slice(&fmt[p..p + run]);
        pos += run;
        p += run;
        if p >= fmt.len() || pos >= end {
            break;
        }
        // Step over the `%` and read what it introduces. A `%` at the very
        // end of the format is ignored.
        p += 1;
        if p >= fmt.len() {
            break;
        }

        // Two `%` in a row print one.
        if fmt[p] == b'%' {
            out[pos] = fmt[p];
            pos += 1;
            p += 1;
            st.prevchar_isflag = false;
            st.prevchar_isitem = false;
            continue;
        }
        // `%=`: where leftover width is spread. Ignored inside a group.
        if fmt[p] == STL_SEPARATE as u8 {
            p += 1;
            if st.groupdepth > 0 {
                continue;
            }
            with_scratch(|s| s.push(Kind::Separate, pos));
            continue;
        }
        // `%<`: where to start cutting when the line is too long.
        if fmt[p] == STL_TRUNCMARK as u8 {
            p += 1;
            with_scratch(|s| s.push(Kind::Trunc, pos));
            continue;
        }
        // `%)`: the end of a group. Ignored when none is open.
        if fmt[p] == b')' {
            p += 1;
            if st.groupdepth < 1 {
                continue;
            }
            pos = with_scratch(|s| group::close(s, out, pos, end, &mut st.groupdepth, fill));
            continue;
        }

        let mut spec = parse::Spec::read(&fmt, &mut p);
        if p >= fmt.len() {
            break;
        }

        // `%N*`: a user highlight group, whose number is the width field.
        if fmt[p] == STL_USER_HL as u8 {
            let minwid = if spec.minwid > 9 { 1 } else { spec.minwid };
            with_scratch(|s| {
                s.push_item(StlItem {
                    start: pos,
                    minwid,
                    kind: Kind::Highlight,
                    ..StlItem::default()
                });
            });
            p += 1;
            continue;
        }

        // `%NT` and `%NX`: a region that switches to or closes tab page N.
        // `tabline=%1Ttab\ one%X` switches to tab 1; `%1X` closes it.
        if fmt[p] == STL_TABPAGENR as u8 || fmt[p] == STL_TABCLOSENR as u8 {
            let mut minwid = spec.minwid;
            if fmt[p] == STL_TABCLOSENR as u8 {
                if minwid == 0 {
                    // A bare `%X` ends the label, so it takes the number of
                    // the last tab label opened -- which, because the items
                    // are one shared stack, may be an outer expansion's.
                    minwid = with_scratch(|s| {
                        (0..s.curitem)
                            .rev()
                            .find(|&n| s.items[n].kind == Kind::TabPage && s.items[n].minwid >= 0)
                            .map_or(0, |n| s.items[n].minwid)
                    });
                } else {
                    // Close numbers are stored negative.
                    minwid = -minwid;
                }
            }
            with_scratch(|s| {
                s.push_item(StlItem {
                    start: pos,
                    minwid,
                    kind: Kind::TabPage,
                    ..StlItem::default()
                });
            });
            p += 1;
            continue;
        }

        // `%@Func@`: the region a mouse click runs `Func` for.
        if fmt[p] == STL_CLICK_FUNC as u8 {
            p += 1;
            let from = p;
            while p < fmt.len() && fmt[p] != STL_CLICK_FUNC as u8 {
                p += 1;
            }
            if p >= fmt.len() {
                break;
            }
            // The name is only copied when the caller asked for the click
            // records; otherwise the region is recorded without one.
            let cmd = if discard_clicks {
                ptr::null_mut()
            } else {
                dup_cstring(&fmt[from..p])
            };
            with_scratch(|s| {
                s.push_item(StlItem {
                    start: pos,
                    cmd,
                    minwid: spec.minwid,
                    kind: Kind::ClickFunc,
                    ..StlItem::default()
                });
            });
            p += 1;
            continue;
        }

        spec.finish(&fmt, &mut p);
        if p >= fmt.len() {
            break;
        }

        // `%(`: the start of a group.
        if fmt[p] == b'(' {
            with_scratch(|s| {
                // The group stack is as long as the item arena, so the item
                // that grows it must be made room for first.
                s.grow();
                s.groupitems[st.groupdepth as usize] = s.curitem;
                s.push_item(StlItem {
                    start: pos,
                    minwid: spec.minwid,
                    maxwid: spec.maxwid,
                    kind: Kind::Group,
                    ..StlItem::default()
                });
            });
            st.groupdepth += 1;
            p += 1;
            continue;
        }

        // `%}`: the end of a block a `%{%...%}` was expanded into.
        if fmt[p] == b'}' && st.evaldepth > 0 {
            p += 1;
            st.evaldepth -= 1;
            continue;
        }

        // Anything the alphabet does not name is skipped.
        if !parse::is_item_letter(fmt[p]) {
            p += 1;
            continue;
        }
        let opt = fmt[p];
        p += 1;

        // `%#name#` and `%$name$` name a highlight group rather than
        // printing anything, so they record and move on.
        if opt == STL_HIGHLIGHT as u8 || opt == STL_HIGHLIGHT_COMB as u8 {
            let from = p;
            while p < fmt.len() && fmt[p] != opt {
                p += 1;
            }
            if p < fmt.len() {
                let kind = if opt == STL_HIGHLIGHT_COMB as u8 {
                    Kind::HighlightCombining
                } else {
                    Kind::Highlight
                };
                let minwid = -syntax_id(&fmt[from..p]);
                with_scratch(|s| {
                    s.push_item(StlItem {
                        start: pos,
                        minwid,
                        kind,
                        ..StlItem::default()
                    });
                });
                p += 1;
            }
            continue;
        }

        text.clear();
        let mut value = Value::default();
        if opt == STL_VIM_EXPR as u8 {
            // Evaluating re-enters the editor, so this is the one arm that
            // must not run under a scratch borrow.
            value.itemisflag = true;
            match vim_expr(env, out, &mut pos, end, &fmt, &mut p, &mut text, &st) {
                Expr::Value { num, itemisflag } => {
                    value.num = num;
                    value.itemisflag = itemisflag;
                }
                Expr::Reparse {
                    block_start,
                    result,
                } => {
                    // Splice the result in where the block was, close it
                    // with a `%}` so the depth comes back down, and read the
                    // format again from there.
                    let mut next =
                        Vec::with_capacity(block_start + result.len() + fmt.len() - p + 2);
                    next.extend_from_slice(&fmt[..block_start]);
                    next.extend_from_slice(&result);
                    next.extend_from_slice(b"%}");
                    next.extend_from_slice(&fmt[p..]);
                    fmt = next;
                    p = block_start;
                    st.evaldepth += 1;
                    continue;
                }
            }
        } else {
            value = with_scratch(|s| value_of(env, s, opt, pos, &mut text));
        }
        // The width helpers measure C strings.
        text.push(0);

        let kept = with_scratch(|s| {
            emit(
                s, out, &mut pos, end, opt, &spec, &value, &text, &mut st, fill,
            )
        });
        if !kept {
            break;
        }
    }
    pos
}

/// `%{expr}` and `%{%expr%}`: evaluate, and decide whether the result is
/// text, a number, or more format to read.
#[allow(clippy::too_many_arguments)]
fn vim_expr(
    env: &Env,
    out: &mut [u8],
    pos: &mut usize,
    end: usize,
    fmt: &[u8],
    p: &mut usize,
    text: &mut Vec<u8>,
    st: &State,
) -> Expr {
    let block_start = *p - 1;
    let reevaluate = *p < fmt.len() && fmt[*p] == b'%';
    if reevaluate {
        *p += 1;
    }

    // Copy the expression into the output buffer, which is where it can be
    // NUL-terminated and handed to the evaluator. The text is rewound
    // afterwards, so nothing of it survives.
    let start = *pos;
    while *p < fmt.len() && !(fmt[*p] == b'}' && (!reevaluate || fmt[*p - 1] == b'%')) && *pos < end
    {
        out[*pos] = fmt[*p];
        *pos += 1;
        *p += 1;
    }
    if *p >= fmt.len() || fmt[*p] != b'}' {
        // Missing `}` or out of space: the item produces nothing.
        return Expr::Value {
            num: -1,
            itemisflag: true,
        };
    }
    *p += 1;
    if reevaluate && *pos > 0 {
        // Drop the `%` that ends `%{% expr %}`.
        out[*pos - 1] = 0;
    } else {
        out[*pos] = 0;
    }
    *pos = start;

    let Some(result) = env.eval(cstr::in_bytes(&out[start..])) else {
        return Expr::Value {
            num: -1,
            itemisflag: true,
        };
    };

    // A result made only of digits becomes a number, so that the item's
    // width and zero-padding apply to it.
    if let Some(num) = as_number(&result) {
        return Expr::Value {
            num,
            itemisflag: false,
        };
    }
    // A `%{%...%}` result with items of its own is read as format.
    if reevaluate
        && !result.is_empty()
        && result.contains(&b'%')
        && st.evaldepth < MAX_STL_EVAL_DEPTH
    {
        return Expr::Reparse {
            block_start,
            result,
        };
    }
    text.extend_from_slice(&result);
    Expr::Value {
        num: -1,
        itemisflag: true,
    }
}

/// What the item `opt` evaluates to.
fn value_of(env: &Env, s: &mut StlScratch, opt: u8, pos: usize, text: &mut Vec<u8>) -> Value {
    let mut v = Value::default();
    match opt {
        // The file name, in full, relative, or just its last component.
        // Blanks in it are never replaced by the fill character.
        b'f' | b'F' | b't' => {
            v.fillable = false;
            env.file_name(opt == STL_FULLPATH as u8, opt == STL_FILENAME as u8, text);
        }
        // The line number, which `'statuscolumn'` overloads with `v:lnum`
        // and `v:relnum` -- and with a sign, when 'signcolumn' is "number".
        b'l' => {
            if env.is_statuscol()
                && (env.win.w_onebuf_opt.wo_nu != 0 || env.win.w_onebuf_opt.wo_rnu != 0)
                && vim_var(Vv::Virtnum) == 0
            {
                if env.win.w_maxscwidth == SCL_NUM && env.number_column_has_sign() {
                    return statuscol_sign(env, s, opt, pos, text, v);
                }
                let relnum = vim_var(Vv::Relnum) as c_int;
                let nu = env.win.w_onebuf_opt.wo_nu != 0;
                let rnu = env.win.w_onebuf_opt.wo_rnu != 0;
                v.num = if !rnu || (nu && relnum == 0) {
                    vim_var(Vv::Lnum) as c_int
                } else {
                    relnum
                };
                // With both 'number' and 'relativenumber' the cursor line's
                // own number is left-aligned, which is a `%=` after it
                // rather than before it.
                v.left_align_num = rnu && nu && relnum == 0;
                if !v.left_align_num {
                    s.push(Kind::Separate, pos);
                }
            } else if !env.is_statuscol() {
                v.num = if env.buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
                    0
                } else {
                    env.win.w_cursor.lnum as c_int
                };
            }
        }
        b'L' => v.num = env.buf.b_ml.ml_line_count as c_int,
        b'c' => {
            v.num = if !in_insert_mode() && env.empty_line {
                0
            } else {
                env.win.w_cursor.col + 1
            };
        }
        b'v' | b'V' => {
            let virtcol = env.win.w_virtcol + 1;
            let col = if !in_insert_mode() && env.empty_line {
                0
            } else {
                env.win.w_cursor.col + 1
            };
            // `%V` is not shown when it says the same as `%c`.
            if !(opt == STL_VIRTCOL_ALT as u8 && virtcol == col) {
                v.num = virtcol;
            }
        }
        b'p' => v.num = env.percentage(),
        // Not a number: `get_rel_pos()` can answer a name like "Top".
        b'P' => env.rel_pos(text),
        b'S' => env.showcmd(text),
        b'a' => {
            v.fillable = false;
            env.arg_number(text);
        }
        b'k' => {
            v.fillable = false;
            env.keymap(text);
        }
        // The page number, which only printing ever sets.
        b'N' => v.num = 0,
        b'n' => v.num = env.buf.handle,
        b'o' | b'O' => {
            if opt == STL_OFFSET_X as u8 {
                v.base = kNumBaseHexadecimal;
            }
            let l = env.line_offset();
            v.num = if env.buf.b_ml.ml_flags.has(MlFlags::EMPTY) || l < 0 {
                0
            } else {
                l + 1
                    + if !in_insert_mode() && env.empty_line {
                        0
                    } else {
                        env.win.w_cursor.col
                    }
            };
        }
        b'b' | b'B' => {
            if opt == STL_BYTEVAL_X as u8 {
                v.base = kNumBaseHexadecimal;
            }
            v.num = env.byte_value();
        }
        b'r' | b'R' => {
            v.itemisflag = true;
            if env.buf.b_p_ro != 0 {
                if opt == STL_ROFLAG_ALT as u8 {
                    text.extend_from_slice(b",RO");
                } else {
                    text.extend_from_slice(tr(c"[RO]"));
                }
            }
        }
        b'h' | b'H' => {
            v.itemisflag = true;
            if env.buf.b_help {
                if opt == STL_HELPFLAG_ALT as u8 {
                    text.extend_from_slice(b",HLP");
                } else {
                    text.extend_from_slice(tr(c"[Help]"));
                }
            }
        }
        // The 'statuscolumn' fold and sign columns.
        b'C' | b's' => return statuscol_sign(env, s, opt, pos, text, v),
        b'y' => env.with_filetype(|ft| {
            // Bracket it only when it fits the scratch buffer, brackets and
            // terminator included.
            if !ft.is_empty() && ft.len() < TMPLEN as usize - 3 {
                text.push(b'[');
                text.extend_from_slice(ft);
                text.push(b']');
            }
        }),
        b'Y' => {
            v.itemisflag = true;
            env.with_filetype(|ft| {
                if !ft.is_empty() && ft.len() < TMPLEN as usize - 2 {
                    text.push(b',');
                    text.extend_from_slice(ft);
                    // The comma is upper-cased too; it has no upper case.
                    for byte in text.iter_mut() {
                        *byte = upper(*byte);
                    }
                }
            });
        }
        b'w' | b'W' => {
            v.itemisflag = true;
            if env.win.w_onebuf_opt.wo_pvw != 0 {
                if opt == STL_PREVIEWFLAG_ALT as u8 {
                    text.extend_from_slice(b",PRV");
                } else {
                    text.extend_from_slice(tr(c"[Preview]"));
                }
            }
        }
        b'q' => env.quickfix_title(text),
        b'm' | b'M' => {
            v.itemisflag = true;
            let alt = c_int::from(opt == STL_MODIFIED_ALT as u8);
            let modified = c_int::from(env.is_changed()) * 2;
            let readonly = c_int::from(env.buf.b_p_ma == 0) * 4;
            let flag: &[u8] = match alt + modified + readonly {
                2 => b"[+]",
                3 => b",+",
                4 => b"[-]",
                5 => b",-",
                6 => b"[+-]",
                7 => b",+-",
                _ => b"",
            };
            text.extend_from_slice(flag);
        }
        _ => {}
    }
    v
}

/// The `'statuscolumn'` fold column (`%C`) and sign column (`%s`), and the
/// sign `%l` draws in place of the number when `'signcolumn'` is "number".
///
/// Each column is its own item, because each carries its own highlight.
fn statuscol_sign(
    env: &Env,
    s: &mut StlScratch,
    opt: u8,
    pos: usize,
    text: &mut Vec<u8>,
    mut v: Value,
) -> Value {
    if !env.is_statuscol() {
        return v;
    }
    let fdc = if opt == STL_FOLDCOL as u8 {
        env.fold_column_width()
    } else {
        0
    };
    // A fold column is one item wide; a sign column is as wide as the signs
    // need; `%l`'s sign is a single column.
    let width = if opt == STL_FOLDCOL as u8 {
        c_int::from(fdc > 0)
    } else if opt == STL_SIGNCOL as u8 {
        env.win.w_scwidth
    } else {
        1
    };
    if width <= 0 {
        return v;
    }
    v.foldsignitem = Some(s.curitem);

    let mut minwid = 0;
    if fdc > 0 {
        minwid = env.fold_glyphs(fdc, text);
    }
    let mut signlen = 0usize;
    for i in 0..width as usize {
        let start = pos + signlen;
        if fdc == 0 {
            let before = text.len();
            // No sign here draws two blanks in the default highlight.
            minwid = env.sign_text(i, text).unwrap_or(0);
            signlen += text.len() - before;
        }
        s.push_item(StlItem {
            start,
            minwid,
            kind: if fdc > 0 {
                Kind::HighlightFold
            } else {
                Kind::HighlightSign
            },
            ..StlItem::default()
        });
    }
    v
}

/// Write one item's value into `out`, applying its width, alignment and fill.
///
/// Answers whether the walk can go on: it stops when the item did not fit.
#[allow(clippy::too_many_arguments)]
fn emit(
    s: &mut StlScratch,
    out: &mut [u8],
    pos: &mut usize,
    end: usize,
    opt: u8,
    spec: &parse::Spec,
    v: &Value,
    text: &[u8],
    st: &mut State,
    fill: &Fill,
) -> bool {
    // The item is normal until something below says otherwise, and starts
    // where the write cursor is.
    s.grow();
    let at = s.curitem;
    s.items[at] = StlItem {
        start: *pos,
        kind: Kind::Normal,
        ..StlItem::default()
    };

    let mut minwid = spec.minwid;
    // `text` carries its terminator, so one byte means "nothing".
    let has_text = text.len() > 1;
    if has_text {
        let mut t = 0usize;
        // A flag drops its leading `,` after a non-item, or its leading
        // blank after another flag.
        if v.itemisflag {
            if text[0] != 0
                && text[1] != 0
                && ((!st.prevchar_isitem && text[0] == b',')
                    || (st.prevchar_isflag && text[0] == b' '))
            {
                t = 1;
            }
            st.prevchar_isflag = true;
        }

        let mut l = strsize_at(text, t);
        if l > 0 {
            st.prevchar_isitem = true;
        }
        // Too wide: cut it from the front and mark the cut.
        if l > spec.maxwid {
            while l >= spec.maxwid {
                l -= cells_at(text, t);
                t += char_len_at(text, t);
            }
            if *pos >= end {
                return false;
            }
            out[*pos] = b'<';
            *pos += 1;
        }

        if minwid > 0 {
            // Right-aligned: pad in front.
            while l < minwid && *pos < end {
                // Never put a `-` in front of a digit.
                if l + 1 == minwid && fill.is_dash() && text[t].is_ascii_digit() {
                    out[*pos] = b' ';
                    *pos += 1;
                } else {
                    *pos = fill.put(out, *pos);
                }
                l += 1;
            }
            minwid = 0;
            // The `'statuscolumn'` sign and fold items were recorded before
            // the padding, so move them along with the text.
            if let Some(fs) = v.foldsignitem {
                let offset = *pos as isize - s.items[fs].start as isize;
                for i in fs..s.curitem {
                    s.items[i].start = (s.items[i].start as isize + offset) as usize;
                }
            }
        } else {
            // A negative width says left-aligned; the padding below is what
            // makes it so.
            minwid = -minwid;
        }

        while text[t] != 0 && *pos < end {
            // A blank becomes the fill character, unless the fill is `-` and
            // a digit follows.
            if v.fillable && text[t] == b' ' && (!text[t + 1].is_ascii_digit() || !fill.is_dash()) {
                *pos = fill.put(out, *pos);
            } else {
                out[*pos] = text[t];
                *pos += 1;
            }
            t += 1;
        }

        // A `'statuscolumn'` sign or fold item ends with an item that puts
        // the highlight back.
        if v.foldsignitem.is_some() {
            s.items[at].kind = Kind::Highlight;
            s.items[at].start = *pos;
            s.items[at].minwid = 0;
        }

        // Left-aligned: pad behind.
        while l < minwid && *pos < end {
            *pos = fill.put(out, *pos);
            l += 1;
        }
    } else if v.num >= 0 {
        // A number needs room for its widest form.
        if *pos + 20 > end {
            return false;
        }
        st.prevchar_isitem = true;
        let plan = parse::number_plan(
            opt == STL_VIRTCOL_ALT as u8,
            spec.zeropad,
            v.base,
            v.num,
            minwid,
            spec.maxwid,
        );
        *pos = put_number(out, *pos, &plan);
    } else {
        s.items[at].kind = Kind::Empty;
    }

    if v.num >= 0 || (!v.itemisflag && has_text) {
        st.prevchar_isflag = false;
    }
    s.curitem = at + 1;
    // A left-aligned `'statuscolumn'` number is followed by a separator, so
    // that the slack lands after the number rather than before it.
    if v.left_align_num {
        s.push(Kind::Separate, *pos);
    }
    true
}
