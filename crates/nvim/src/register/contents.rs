//! A register as text: the API's way in and out.
//!
//! [`get_reg_contents`] and the `write_reg_contents*` family are what
//! `getreg()`, `setreg()`, `nvim_paste`, the clipboard provider and shada all
//! use, so these are the functions that turn a `YankReg` into lines or one
//! string and back.
//!
//! [`str_to_reg`] is the interesting direction. It splits an incoming string
//! on newlines; an embedded **NUL is a newline** the way the rest of the
//! editor spells one, so it is translated on the way in; a charwise register
//! written to again *appends to its last line* rather than starting a new
//! one; and a blockwise write has to measure the widest line in **cells**
//! rather than bytes.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::charset::getdigits_int_at;
use crate::charset::str2nr::atoi;
use crate::cstr;
use crate::mbyte::{c_partial_cells_in, c_string_cells, promised_char_len};
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::semsg;
use crate::types::{ListRef, RegLines, TypVal};
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use super::*;
use crate::types::NUL;

/// The motion type of register `regname`, and its width if blockwise (0
/// otherwise).
///
/// `kMTUnknown` for an invalid or empty register. May run the clipboard
/// provider.
pub fn get_reg_type(regname: c_int) -> (MotionType, ColNr) {
    // Every computed register reads as charwise.
    match regname {
        Ctrl_F | Ctrl_P | Ctrl_W | Ctrl_A => return (kMTCharWise, 0),
        c if c == '#' as c_int
            || c == '=' as c_int
            || c == ':' as c_int
            || c == '/' as c_int
            || c == '.' as c_int
            || c == '%' as c_int
            || c == '_' as c_int =>
        {
            return (kMTCharWise, 0);
        }
        _ => {}
    }

    if regname != NUL && !valid_yank_reg(regname, false) {
        return (kMTUnknown, 0);
    }
    get_yank_register(regname, YREG_PASTE).read(|reg| {
        if !reg.lines.is_set() {
            return (kMTUnknown, 0);
        }
        let width = if reg.kind == kMTBlockWise {
            reg.width
        } else {
            0
        };
        (reg.kind, width)
    })
}

/// What a register read answers: one string, or -- with `kGRegList` -- its
/// lines.
enum RegContents {
    Text(XString),
    Lines(ListRef),
}

/// The contents of register `regname`, as one string or -- with
/// `kGRegList` -- a `List` of lines.
///
/// `kGRegNoExpr` refuses `"=` outright and `kGRegExprSrc` answers its source
/// rather than evaluating it, which is what `getreg('=', 1, ...)` wants.
/// `"=` may run arbitrary Vimscript, and `"*`/`"+` the clipboard provider.
fn get_reg_contents(regname: c_int, flags: c_int) -> Option<RegContents> {
    let as_list = flags & kGRegList as c_int != 0;
    // A computed register's one string, as a one-item list when lines were
    // asked for -- where even a missing string is an item.
    let one_line = |text: Option<XString>| {
        if !as_list {
            return text.map(RegContents::Text);
        }
        let list = tv_list_alloc(1);
        list.edit()
            .push(TypVal::string(text.map(ThinCString::from)));
        Some(RegContents::Lines(list))
    };
    let mut regname = regname;
    if regname == '=' as c_int {
        if flags & kGRegNoExpr as c_int != 0 {
            return None;
        }
        if flags & kGRegExprSrc as c_int != 0 {
            return one_line(get_expr_line_src());
        }
        // Evaluating the expression may run Vimscript, which this
        // function's own caller already allows for.
        return one_line(get_expr_line());
    }
    if regname == '@' as c_int {
        regname = '"' as c_int; // `getreg('@')` means the unnamed register
    }
    if regname != NUL && !valid_yank_reg(regname, false) {
        return None;
    }

    if let Some(value) = get_spec_reg(regname, false) {
        return value.and_then(|text| one_line(Some(text)));
    }

    get_yank_register(regname, YREG_PUT).read(|reg| {
        if !reg.lines.is_set() {
            return None;
        }
        let lines = reg.lines.as_slice();
        if as_list {
            let list = tv_list_alloc(lines.len() as ptrdiff_t);
            for line in lines {
                list.edit().push_bytes(Some(line));
            }
            return Some(RegContents::Lines(list));
        }

        // One string, with a newline between lines and after the last one if
        // the register is linewise.
        let needs_nl = |i: usize| reg.kind == kMTLineWise || i < lines.len().wrapping_sub(1);
        let mut text = XString::with_capacity(lines.iter().map(|line| line.len() + 1).sum());
        for (i, line) in lines.iter().enumerate() {
            text.push_bytes(line);
            if needs_nl(i) {
                text.push_byte(b'\n');
            }
        }
        Some(RegContents::Text(text))
    })
}

/// [`get_reg_contents`] as one owned string, `None` when the register is
/// empty or cannot be read. `flags` must not ask for a list.
pub(crate) fn get_reg_contents_owned(regname: c_int, flags: c_int) -> Option<XString> {
    debug_assert!(flags & kGRegList as c_int == 0, "a list is not a string");
    match get_reg_contents(regname, flags)? {
        RegContents::Text(text) => Some(text),
        RegContents::Lines(_) => unreachable!("lines were not asked for"),
    }
}

/// [`get_reg_contents`] as a List of lines, `None` when the register is
/// empty or cannot be read.
pub(crate) fn get_reg_contents_list(regname: c_int, flags: c_int) -> Option<ListRef> {
    match get_reg_contents(regname, flags | kGRegList as c_int)? {
        RegContents::Lines(list) => Some(list),
        RegContents::Text(_) => unreachable!("lines were asked for"),
    }
}

/// Prepare register `name` to be written: check the name, remember `""`, and
/// empty the register unless the write is an append.
///
/// Answers `None` for an invalid name, having given E354.
fn init_write_reg(
    name: c_int,
    old_y_previous: &mut Option<c_int>,
    must_append: bool,
) -> Option<Reg> {
    if !valid_yank_reg(name, true) {
        emsg_invreg(name);
        return None;
    }
    // `get_yank_register` moves `""`, which a write to a *named* register
    // must not do; `finish_write_reg` puts it back.
    *old_y_previous = y_previous.get();
    let reg = get_yank_register(name, YREG_YANK);
    if !is_append_register(name) && !must_append {
        reg.write(YankReg::clear);
    }
    Some(reg)
}

/// What [`str_to_reg`] splits into lines: text with newlines in it, or
/// lines already split.
#[derive(Clone, Copy)]
enum RegText<'a> {
    Text(&'a [u8]),
    Lines(&'a [&'a CStr]),
}

/// Put `text` in `reg`, appending to whatever is already there.
///
/// `yank_type` may be `kMTUnknown`, in which case it is worked out from the
/// text: a trailing newline (or carriage return) means linewise. `blocklen`
/// is the block width, or -1 to measure it.
fn str_to_reg(reg: &mut YankReg, mut yank_type: MotionType, text: RegText<'_>, blocklen: ColNr) {
    if yank_type == kMTUnknown {
        let ends_line = match text {
            RegText::Lines(_) => true,
            RegText::Text(text) => text
                .last()
                .is_some_and(|&last| c_int::from(last) == NL || c_int::from(last) == CAR),
        };
        yank_type = if ends_line { kMTLineWise } else { kMTCharWise };
    }

    // How many lines `text` becomes, and whether the first of them joins the
    // register's current last line; `extraline` is the line a string not
    // ending in a newline still contributes.
    let y_size = reg.lines.len();
    let (newlines, extraline, mut append) = match text {
        RegText::Lines(lines) => (lines.len(), false, false),
        RegText::Text(text) => {
            let mut newlines = text.iter().filter(|&&b| b == b'\n').count();
            let extraline =
                yank_type == kMTCharWise || text.last().is_none_or(|&last| last != b'\n');
            if extraline {
                newlines += 1;
            }
            // A charwise register's last line has no newline of its own, so
            // the first new line continues it.
            let append = y_size > 0 && reg.kind == kMTCharWise;
            if append {
                newlines -= 1;
            }
            (newlines, extraline, append)
        }
    };
    if y_size + newlines == 0 {
        reg.lines = RegLines::NONE;
        return;
    }

    let lines = reg.lines.lines_mut();
    lines.reserve(newlines);
    let mut maxlen: usize = 0;
    match text {
        RegText::Lines(new) => {
            for line in new {
                if yank_type == kMTBlockWise {
                    maxlen = maxlen.max(c_string_cells(line.to_bytes()));
                }
                lines.push(XString::from_cstr(line));
            }
        }
        RegText::Text(text) => {
            let end = text.len();
            // `+ extraline` so that a string with no trailing newline still
            // produces its last (possibly empty) line.
            let limit = end + usize::from(extraline);
            let mut start = 0;
            while start < limit {
                let mut charlen: c_int = 0;
                let mut line_end = start;
                while line_end < end {
                    let byte = text[line_end];
                    if byte == b'\n' {
                        break;
                    }
                    if yank_type == kMTBlockWise {
                        charlen += c_partial_cells_in(text, line_end, end);
                    }
                    // A NUL is one byte and is translated below, so it must
                    // not be handed to the UTF-8 length.
                    if c_int::from(byte) == NUL {
                        line_end += 1;
                    } else {
                        // A sequence `text`'s end cuts short is as long as the
                        // lead byte promised, so the step stops at `end`:
                        // upstream's walked past it and copied the bytes after.
                        let left = end - line_end;
                        line_end += promised_char_len(&text[line_end..]).min(left);
                    }
                }
                maxlen = maxlen.max(usize::try_from(charlen).unwrap_or(0));

                // Appending continues the register's current last line.
                let mut joined = Vec::new();
                if append {
                    append = false;
                    if let Some(last) = lines.pop() {
                        joined = last.into_vec();
                        joined.pop(); // its terminator
                    }
                }
                joined.extend_from_slice(&text[start..line_end]);
                // A NUL in the text is how the editor spells a newline.
                for byte in &mut joined {
                    if c_int::from(*byte) == NUL {
                        *byte = b'\n';
                    }
                }
                lines.push(XString::from_vec(joined));

                // Past the newline `line_end` stopped on.
                start = line_end + 1;
            }
        }
    }

    reg.width = if yank_type == kMTBlockWise {
        if blocklen == -1 {
            c_int::try_from(maxlen).unwrap_or(c_int::MAX) - 1
        } else {
            blocklen
        }
    } else {
        0
    };
    // The extra data described the text that has just been replaced.
    reg.kind = yank_type;
    reg.extra = None;
    reg.timestamp = os_time();
}

/// Push the write out to the clipboard and put `""` back where it was.
fn finish_write_reg(name: c_int, reg: Reg, old_y_previous: Option<c_int>) {
    clipboard::set_clipboard(name, reg);
    // Only an explicit write to `""` moves it.
    if name != '"' as c_int {
        y_previous.set(old_y_previous);
    }
}

/// Write `text` to register `name`, working out the motion type from it.
pub fn write_reg_contents(name: c_int, text: &[u8], must_append: bool) {
    write_reg_contents_ex(name, text, must_append, kMTUnknown, 0);
}

/// Write `lines` to register `name`, one line apiece.
///
/// Writing a `"*`/`"+` register runs the clipboard provider.
pub fn write_reg_contents_lst(
    name: c_int,
    lines: &[&CStr],
    must_append: bool,
    yank_type: MotionType,
    block_len: ColNr,
) {
    if name == '/' as c_int || name == '=' as c_int {
        // Neither register can hold more than one line.
        let line = match lines {
            [] => c"",
            [line] => line,
            _ => {
                emsg(gettext(
                    e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines,
                ));
                return;
            }
        };
        write_reg_contents_cstr(name, line, must_append, yank_type, block_len);
        return;
    }
    if name == '_' as c_int {
        return; // black hole
    }

    let mut old_y_previous = None;
    let Some(reg) = init_write_reg(name, &mut old_y_previous, must_append) else {
        return;
    };
    reg.write(|reg| str_to_reg(reg, yank_type, RegText::Lines(lines), block_len));
    finish_write_reg(name, reg, old_y_previous);
}

/// [`write_reg_contents_ex`] of `text`.
pub(crate) fn write_reg_contents_cstr(
    name: c_int,
    text: &CStr,
    must_append: bool,
    yank_type: MotionType,
    block_len: ColNr,
) {
    write_reg_contents_ex(name, text.to_bytes(), must_append, yank_type, block_len);
}

/// Write `text` to register `name` as `yank_type`.
///
/// `"/`, `"#` and `"=` are not real registers and are handled here: the first
/// sets the last search pattern, the second the alternate file, and the third
/// the expression source. The first two read `text` as the C string it was,
/// up to a NUL.
pub fn write_reg_contents_ex(
    name: c_int,
    text: &[u8],
    must_append: bool,
    yank_type: MotionType,
    block_len: ColNr,
) {
    if name == '/' as c_int {
        // The pattern is copied, not kept.
        set_last_search_pat(text, RE_SEARCH as c_int, true, true);
        return;
    }

    if name == '#' as c_int {
        // The alternate file, by number or by name.
        let text = cstr::until_nul(text);
        let buf = if text.first().is_some_and(|&b| ascii_isdigit(c_int::from(b))) {
            let num = atoi(text);
            let buf = find_buf(num);
            if buf.is_none() {
                semsg!("E86: Buffer {} does not exist", int64_t::from(num));
            }
            buf
        } else {
            let pattern = XString::from_bytes(text);
            find_buf(buflist_findpat(
                pattern.as_cstr(),
                Some(text.len()),
                true,
                false,
                false,
            ))
        };
        if let Some(buf) = buf {
            Win::current().w_alt_fnum = buf.handle;
        }
        return;
    }

    if name == '=' as c_int {
        // The expression register keeps its source, not a yankreg.
        expr_line.with_mut(|line| {
            if !must_append {
                *line = None;
            }
            line.get_or_insert_with(XString::new).push_bytes(text);
        });
        return;
    }

    if name == '_' as c_int {
        return; // black hole
    }

    let mut old_y_previous = None;
    let Some(reg) = init_write_reg(name, &mut old_y_previous, must_append) else {
        return;
    };
    reg.write(|reg| str_to_reg(reg, yank_type, RegText::Text(text), block_len));
    finish_write_reg(name, reg, old_y_previous);
}

/// Set `reg`'s type from an API `regtype` string (`""`, `v`/`c`, `V`/`l`,
/// `b`/CTRL-V, optionally followed by a block width).
///
/// Answers false for a `regtype` that does not parse.
pub fn prepare_yankreg_from_object(reg: &mut YankReg, regtype: &[u8]) -> bool {
    let kind = match regtype.first().map_or(NUL, |&b| c_int::from(b)) {
        0 => kMTUnknown, // "" means "work it out from the text"
        c if c == 'v' as c_int || c == 'c' as c_int => kMTCharWise,
        c if c == 'V' as c_int || c == 'l' as c_int => kMTLineWise,
        c if c == 'b' as c_int || c == Ctrl_V => kMTBlockWise,
        _ => return false,
    };
    // As upstream, both fields are settled before the width is parsed, so a
    // `regtype` that fails below still leaves the type behind.
    reg.kind = kind;
    reg.width = 0;

    if regtype.len() > 1 {
        // A width only means something for a block.
        if kind != kMTBlockWise {
            return false;
        }
        if !regtype[1].is_ascii_digit() {
            return false;
        }
        let (width, past) = getdigits_int_at(regtype, 1, false, 1);
        reg.width = width - 1;
        // Nothing may follow the width.
        if regtype.len() > past {
            return false;
        }
    }
    reg.extra = None;
    reg.timestamp = 0;
    true
}

/// Settle a register built from an API object: drop the empty last line a
/// linewise write ends with, and decide the type when it was left unknown.
///
/// `clipboard_adjust` is for the clipboard provider, whose lists always carry
/// that trailing empty line.
pub fn finish_yankreg_from_object(reg: &mut YankReg, clipboard_adjust: bool) {
    let ends_empty = reg.lines.last().is_some_and(|last| last.is_empty());
    if ends_empty {
        if reg.kind != kMTCharWise {
            if reg.kind == kMTUnknown || clipboard_adjust {
                reg.lines.lines_mut().pop();
            }
            if reg.kind == kMTUnknown {
                reg.kind = kMTLineWise;
            }
        }
    } else if reg.kind == kMTUnknown {
        reg.kind = kMTCharWise;
    }
    reg.update_width();
}
