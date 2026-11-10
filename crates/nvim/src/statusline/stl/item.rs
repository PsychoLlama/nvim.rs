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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::memline::MlFlags;
use crate::types::ColNr;
use core::ffi::c_int;

use super::{
    Env, Fill, Kind, MAX_STL_EVAL_DEPTH, NUM_BASE_DECIMAL, NUM_BASE_HEXADECIMAL, NumberBase,
    StlItem, StlScratch, TMPLEN, as_number, cells_at, char_len_at, dup_cstring, group,
    in_insert_mode, parse, put_number, strsize_at, syntax_id, tr, upper, vim_var, with_scratch,
};
use crate::cstr;
use crate::decoration::SCL_NUM;
use crate::statusline::defs::StlOpt;
use crate::types::Vv;

/// What carries across items while the format is walked.
#[derive(Debug)]
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
#[derive(Debug)]
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
            base: NUM_BASE_DECIMAL,
            itemisflag: false,
            fillable: true,
            foldsignitem: None,
            left_align_num: false,
        }
    }
}

/// What a `%{}` item did.
#[derive(Debug)]
enum Expr {
    /// It produced a value; the text, if any, is in the scratch buffer.
    Value { num: c_int, itemisflag: bool },
    /// `%{%...%}` whose result has items of its own: splice it into the
    /// format at `block_start` and read the format again.
    Reparse { block_start: usize, result: Vec<u8> },
}

/// Whether the walk goes on to the next item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Next,
    Stop,
}

// The item letters a `match` reads, as patterns.
const SEPARATE: u8 = StlOpt::Separate.letter();
const TRUNC_MARK: u8 = StlOpt::TruncMark.letter();
const USER_HL: u8 = StlOpt::UserHl.letter();
const TAB_PAGE_NR: u8 = StlOpt::TabPageNr.letter();
const TAB_CLOSE_NR: u8 = StlOpt::TabCloseNr.letter();
const CLICK_FUNC: u8 = StlOpt::ClickFunc.letter();

/// One walk over a format: where the reader and the writer are, and what
/// every item reads.
struct Walk<'a> {
    env: &'a Env<'a>,
    out: &'a mut [u8],
    fill: &'a Fill,
    discard_clicks: bool,
    /// The last byte of `out` is reserved for the NUL, so every visible
    /// character must land before this.
    end: usize,
    /// The format, which a `%{%...%}` block can rewrite.
    fmt: Vec<u8>,
    /// Where the format is read.
    p: usize,
    /// Where `out` is written.
    pos: usize,
    /// An item's text, reused from item to item.
    text: Vec<u8>,
    st: State,
}

/// Walk `usefmt`, answering where in `out` the text ends.
pub(super) fn run(
    env: &Env,
    out: &mut [u8],
    usefmt: Vec<u8>,
    fill: &Fill,
    discard_clicks: bool,
) -> usize {
    let end = out.len() - 1;
    let mut walk = Walk {
        env,
        out,
        fill,
        discard_clicks,
        end,
        fmt: usefmt,
        p: 0,
        pos: 0,
        text: Vec::new(),
        st: State {
            groupdepth: 0,
            evaldepth: 0,
            prevchar_isflag: true,
            prevchar_isitem: false,
        },
    };
    while walk.p < walk.fmt.len() && walk.step() == Flow::Next {}
    walk.pos
}

impl Walk<'_> {
    /// Copy literal text up to the next `%` and read the item it starts.
    fn step(&mut self) -> Flow {
        if !self.copy_to_item() {
            return Flow::Stop;
        }
        match self.fmt[self.p] {
            // Two `%` in a row print one.
            b'%' => {
                self.out[self.pos] = b'%';
                self.pos += 1;
                self.p += 1;
                self.st.prevchar_isflag = false;
                self.st.prevchar_isitem = false;
            }
            // `%=`: where leftover width is spread. Ignored inside a group.
            SEPARATE => {
                self.p += 1;
                if self.st.groupdepth <= 0 {
                    let pos = self.pos;
                    with_scratch(|s| s.push(Kind::Separate, pos));
                }
            }
            // `%<`: where to start cutting when the line is too long.
            TRUNC_MARK => {
                self.p += 1;
                let pos = self.pos;
                with_scratch(|s| s.push(Kind::Trunc, pos));
            }
            // `%)`: the end of a group. Ignored when none is open.
            b')' => {
                self.p += 1;
                if self.st.groupdepth >= 1 {
                    let Walk {
                        out,
                        pos,
                        end,
                        st,
                        fill,
                        ..
                    } = self;
                    *pos = with_scratch(|s| {
                        group::close(s, out, *pos, *end, &mut st.groupdepth, fill)
                    });
                }
            }
            _ => return self.item(),
        }
        Flow::Next
    }

    /// Copy the format verbatim until the next item, the end of the format,
    /// or the end of the output buffer, and step over the `%`. Answers
    /// whether there is an item to read: a `%` at the very end of the format
    /// is ignored. Upstream steps a byte at a time; the run is the same.
    fn copy_to_item(&mut self) -> bool {
        let (fmt, p) = (&self.fmt, self.p);
        if fmt[p] != b'%' {
            self.st.prevchar_isflag = false;
            self.st.prevchar_isitem = false;
        }
        let run = fmt[p..]
            .iter()
            .position(|&byte| byte == b'%')
            .unwrap_or(fmt.len() - p)
            .min(self.end - self.pos);
        self.out[self.pos..self.pos + run].copy_from_slice(&fmt[p..p + run]);
        self.pos += run;
        self.p += run;
        if self.p >= self.fmt.len() || self.pos >= self.end {
            return false;
        }
        self.p += 1;
        self.p < self.fmt.len()
    }

    /// An item with a width field: the three that overload it with an
    /// argument, then the ones that take a maximum width too.
    fn item(&mut self) -> Flow {
        let mut spec = parse::Spec::read(&self.fmt, &mut self.p);
        if self.p >= self.fmt.len() {
            return Flow::Stop;
        }
        match self.fmt[self.p] {
            USER_HL => {
                self.user_highlight(&spec);
                return Flow::Next;
            }
            letter @ (TAB_PAGE_NR | TAB_CLOSE_NR) => {
                self.tab_region(letter == TAB_CLOSE_NR, spec.minwid);
                return Flow::Next;
            }
            CLICK_FUNC => return self.click_region(spec.minwid),
            _ => {}
        }

        spec.finish(&self.fmt, &mut self.p);
        if self.p >= self.fmt.len() {
            return Flow::Stop;
        }
        match self.fmt[self.p] {
            b'(' => self.open_group(&spec),
            // `%}`: the end of a block a `%{%...%}` was expanded into.
            b'}' if self.st.evaldepth > 0 => {
                self.p += 1;
                self.st.evaldepth -= 1;
            }
            byte => {
                // Anything the alphabet does not name is skipped.
                self.p += 1;
                match StlOpt::from_byte(byte) {
                    Some(opt @ (StlOpt::Highlight | StlOpt::HighlightComb)) => {
                        self.highlight_name(opt);
                    }
                    Some(opt) => return self.printed(opt, &spec),
                    None => {}
                }
            }
        }
        Flow::Next
    }

    /// `%N*`: a user highlight group, whose number is the width field.
    fn user_highlight(&mut self, spec: &parse::Spec) {
        let minwid = if spec.minwid > 9 { 1 } else { spec.minwid };
        let start = self.pos;
        with_scratch(|s| {
            s.push_item(StlItem {
                start,
                minwid,
                kind: Kind::Highlight,
                ..StlItem::default()
            });
        });
        self.p += 1;
    }

    /// `%NT` and `%NX`: a region that switches to or closes tab page N.
    /// `tabline=%1Ttab\ one%X` switches to tab 1; `%1X` closes it.
    fn tab_region(&mut self, closes: bool, number: c_int) {
        let mut minwid = number;
        if closes {
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
        let start = self.pos;
        with_scratch(|s| {
            s.push_item(StlItem {
                start,
                minwid,
                kind: Kind::TabPage,
                ..StlItem::default()
            });
        });
        self.p += 1;
    }

    /// `%@Func@`: the region a mouse click runs `Func` for. An unterminated
    /// name ends the walk.
    fn click_region(&mut self, minwid: c_int) -> Flow {
        self.p += 1;
        let from = self.p;
        while self.p < self.fmt.len() && self.fmt[self.p] != CLICK_FUNC {
            self.p += 1;
        }
        if self.p >= self.fmt.len() {
            return Flow::Stop;
        }
        // The name is only copied when the caller asked for the click
        // records; otherwise the region is recorded without one.
        let name = (!self.discard_clicks).then(|| dup_cstring(&self.fmt[from..self.p]));
        let start = self.pos;
        with_scratch(|s| {
            let cmd = name.map_or(0, |name| s.add_cmd(name));
            s.push_item(StlItem {
                start,
                cmd,
                minwid,
                kind: Kind::ClickFunc,
                ..StlItem::default()
            });
        });
        self.p += 1;
        Flow::Next
    }

    /// `%(`: the start of a group.
    fn open_group(&mut self, spec: &parse::Spec) {
        let (start, depth) = (self.pos, self.st.groupdepth);
        with_scratch(|s| {
            // The group stack is as long as the item arena, so the item
            // that grows it must be made room for first.
            s.grow();
            s.groupitems[usize::try_from(depth).unwrap_or(usize::MAX)] = s.curitem;
            s.push_item(StlItem {
                start,
                minwid: spec.minwid,
                maxwid: spec.maxwid,
                kind: Kind::Group,
                ..StlItem::default()
            });
        });
        self.st.groupdepth += 1;
        self.p += 1;
    }

    /// `%#name#` and `%$name$` name a highlight group rather than printing
    /// anything, so they record and move on. An unterminated name is
    /// dropped.
    fn highlight_name(&mut self, opt: StlOpt) {
        let from = self.p;
        while self.p < self.fmt.len() && self.fmt[self.p] != opt.letter() {
            self.p += 1;
        }
        if self.p < self.fmt.len() {
            let kind = if opt == StlOpt::HighlightComb {
                Kind::HighlightCombining
            } else {
                Kind::Highlight
            };
            let minwid = -syntax_id(&self.fmt[from..self.p]);
            let start = self.pos;
            with_scratch(|s| {
                s.push_item(StlItem {
                    start,
                    minwid,
                    kind,
                    ..StlItem::default()
                });
            });
            self.p += 1;
        }
    }

    /// An item that prints: evaluate it, then lay it out. Stops the walk
    /// when the output is full.
    fn printed(&mut self, opt: StlOpt, spec: &parse::Spec) -> Flow {
        self.text.clear();
        let mut value = Value::default();
        if opt == StlOpt::VimExpr {
            // Evaluating re-enters the editor, so this is the one arm that
            // must not run under a scratch borrow.
            value.itemisflag = true;
            let Walk {
                env,
                out,
                pos,
                end,
                fmt,
                p,
                text,
                st,
                ..
            } = self;
            match vim_expr(env, out, pos, *end, fmt, p, text, st) {
                Expr::Value { num, itemisflag } => {
                    value.num = num;
                    value.itemisflag = itemisflag;
                }
                Expr::Reparse {
                    block_start,
                    result,
                } => {
                    self.splice(block_start, &result);
                    return Flow::Next;
                }
            }
        } else {
            let (env, pos, text) = (self.env, self.pos, &mut self.text);
            value = with_scratch(|s| value_of(env, s, opt, pos, text));
        }
        // The width helpers measure C strings.
        self.text.push(0);

        let Walk {
            out,
            pos,
            end,
            text,
            st,
            fill,
            ..
        } = self;
        let kept = with_scratch(|s| emit(s, out, pos, *end, opt, spec, &value, text, st, fill));
        if kept { Flow::Next } else { Flow::Stop }
    }

    /// Splice a `%{%...%}` block's result in where the block was, close it
    /// with a `%}` so the depth comes back down, and read the format again
    /// from there.
    fn splice(&mut self, block_start: usize, result: &[u8]) {
        let fmt = &self.fmt;
        let mut next = Vec::with_capacity(block_start + result.len() + fmt.len() - self.p + 2);
        next.extend_from_slice(&fmt[..block_start]);
        next.extend_from_slice(result);
        next.extend_from_slice(b"%}");
        next.extend_from_slice(&fmt[self.p..]);
        self.fmt = next;
        self.p = block_start;
        self.st.evaldepth += 1;
    }
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
fn value_of(env: &Env, s: &mut StlScratch, opt: StlOpt, pos: usize, text: &mut Vec<u8>) -> Value {
    let mut v = Value::default();
    match opt {
        // The file name, in full, relative, or just its last component.
        // Blanks in it are never replaced by the fill character.
        StlOpt::FilePath | StlOpt::FullPath | StlOpt::FileName => {
            v.fillable = false;
            env.file_name(opt == StlOpt::FullPath, opt == StlOpt::FileName, text);
        }
        // The line number, which `'statuscolumn'` overloads with `v:lnum`
        // and `v:relnum` -- and with a sign, when 'signcolumn' is "number".
        StlOpt::Line => {
            if env.is_statuscol()
                && (env.win.w_onebuf_opt.wo_nu != 0 || env.win.w_onebuf_opt.wo_rnu != 0)
                && vim_var(Vv::Virtnum) == 0
            {
                if env.win.w_maxscwidth == SCL_NUM && env.number_column_has_sign() {
                    return statuscol_sign(env, s, opt, pos, text, v);
                }
                let relnum = c_int::try_from(vim_var(Vv::Relnum)).unwrap_or(c_int::MAX);
                let nu = env.win.w_onebuf_opt.wo_nu != 0;
                let rnu = env.win.w_onebuf_opt.wo_rnu != 0;
                v.num = if !rnu || (nu && relnum == 0) {
                    c_int::try_from(vim_var(Vv::Lnum)).unwrap_or(c_int::MAX)
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
                    env.win.w_cursor.lnum.get()
                };
            }
        }
        StlOpt::NumLines => v.num = env.buf.b_ml.ml_line_count.get(),
        StlOpt::Column => {
            v.num = if !in_insert_mode() && env.empty_line {
                0
            } else {
                (env.win.w_cursor.col + 1).get()
            };
        }
        StlOpt::VirtCol | StlOpt::VirtColAlt => {
            let virtcol = env.win.w_virtcol + 1;
            let col = if !in_insert_mode() && env.empty_line {
                0
            } else {
                (env.win.w_cursor.col + 1).get()
            };
            // `%V` is not shown when it says the same as `%c`.
            if !(opt == StlOpt::VirtColAlt && virtcol == ColNr::new(col)) {
                v.num = virtcol.get();
            }
        }
        StlOpt::Percentage => v.num = env.percentage(),
        // Not a number: `get_rel_pos()` can answer a name like "Top".
        StlOpt::AltPercent => env.rel_pos(text),
        StlOpt::ShowCmd => env.showcmd(text),
        StlOpt::ArgListStat => {
            v.fillable = false;
            env.arg_number(text);
        }
        StlOpt::Keymap => {
            v.fillable = false;
            env.keymap(text);
        }
        // The page number, which only printing ever sets.
        StlOpt::PageNum => v.num = 0,
        StlOpt::BufNo => v.num = env.buf.handle.get(),
        StlOpt::Offset | StlOpt::OffsetX => {
            if opt == StlOpt::OffsetX {
                v.base = NUM_BASE_HEXADECIMAL;
            }
            let offset = env.line_offset().filter(|&l| l >= 0);
            v.num = match offset {
                Some(l) if !env.buf.b_ml.ml_flags.has(MlFlags::EMPTY) => {
                    l + 1
                        + if !in_insert_mode() && env.empty_line {
                            0
                        } else {
                            env.win.w_cursor.col.get()
                        }
                }
                _ => 0,
            };
        }
        StlOpt::ByteVal | StlOpt::ByteValX => {
            if opt == StlOpt::ByteValX {
                v.base = NUM_BASE_HEXADECIMAL;
            }
            v.num = env.byte_value();
        }
        StlOpt::RoFlag | StlOpt::RoFlagAlt => {
            v.itemisflag = true;
            if env.buf.b_p_ro != 0 {
                if opt == StlOpt::RoFlagAlt {
                    text.extend_from_slice(b",RO");
                } else {
                    text.extend_from_slice(tr(c"[RO]"));
                }
            }
        }
        StlOpt::HelpFlag | StlOpt::HelpFlagAlt => {
            v.itemisflag = true;
            if env.buf.b_help {
                if opt == StlOpt::HelpFlagAlt {
                    text.extend_from_slice(b",HLP");
                } else {
                    text.extend_from_slice(tr(c"[Help]"));
                }
            }
        }
        // The 'statuscolumn' fold and sign columns.
        StlOpt::FoldCol | StlOpt::SignCol => return statuscol_sign(env, s, opt, pos, text, v),
        StlOpt::FileType => env.with_filetype(|ft| {
            // Bracket it only when it fits the scratch buffer, brackets and
            // terminator included.
            if !ft.is_empty() && ft.len() < TMPLEN as usize - 3 {
                text.push(b'[');
                text.extend_from_slice(ft);
                text.push(b']');
            }
        }),
        StlOpt::FileTypeAlt => {
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
        StlOpt::PreviewFlag | StlOpt::PreviewFlagAlt => {
            v.itemisflag = true;
            if env.win.w_onebuf_opt.wo_pvw != 0 {
                if opt == StlOpt::PreviewFlagAlt {
                    text.extend_from_slice(b",PRV");
                } else {
                    text.extend_from_slice(tr(c"[Preview]"));
                }
            }
        }
        StlOpt::Quickfix => env.quickfix_title(text),
        StlOpt::Modified | StlOpt::ModifiedAlt => {
            v.itemisflag = true;
            let alt = c_int::from(opt == StlOpt::ModifiedAlt);
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
        // The items with no value of their own: the highlight, group,
        // separator, truncation, tab-page and click letters, each handled
        // by the walk above before it gets here.
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
    opt: StlOpt,
    pos: usize,
    text: &mut Vec<u8>,
    mut v: Value,
) -> Value {
    if !env.is_statuscol() {
        return v;
    }
    let fdc = if opt == StlOpt::FoldCol {
        env.fold_column_width()
    } else {
        0
    };
    // A fold column is one item wide; a sign column is as wide as the signs
    // need; `%l`'s sign is a single column.
    let width = if opt == StlOpt::FoldCol {
        c_int::from(fdc > 0)
    } else if opt == StlOpt::SignCol {
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
    for i in 0..usize::try_from(width).unwrap_or(0) {
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
    opt: StlOpt,
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
                let (to, from) = (*pos, s.items[fs].start);
                for i in fs..s.curitem {
                    s.items[i].start = s.items[i].start.wrapping_add(to).wrapping_sub(from);
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
            opt == StlOpt::VirtColAlt,
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
