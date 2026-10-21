//! Yanking text into a register.
//!
//! [`op_yank_reg`] is the whole of `y`: it copies the operator's region into a
//! `YankReg` line by line. The blockwise case goes through `block_prep` per
//! line, so that a short line is padded out and a tab straddling the edge of
//! the block is split into spaces; the charwise case is the same walk with
//! `charwise_block_prep`, which is how the first and last lines of a charwise
//! yank get their partial extents.
//!
//! [`format_reg_type`] renders the `v` / `V` / `CTRL-V width` string the API
//! and `:registers` both show, and [`do_autocmd_textyankpost`] builds the
//! `v:event` dictionary TextYankPost sees.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::formatted;
use crate::guard::Lock;
use crate::mbyte::head_off;
use crate::memory::{ThinCString, XString};
use crate::message_fmt::{msg_bytes, report_msg};
use crate::ops::{block_def_of, charwise_block_def};
use crate::option::vars::P_SEL;
use crate::tr_plural;
use crate::types::RegLines;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_ulong};

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, NUL};

/// One line of a block, as `bd` measured it on `line`: the text with the
/// spaces `block_prep` counted on both ends.
///
/// With `exclude_trailing_space` -- 'formatoptions' `y`'s blockwise yank --
/// the padding at the end is dropped and any white space the text itself ends
/// with is trimmed off.
fn yank_copy_line(line: &[u8], bd: &BlockDef, exclude_trailing_space: bool) -> XString {
    let endspaces = if exclude_trailing_space {
        0
    } else {
        bd.endspaces
    };
    let pad = |n: c_int| usize::try_from(n).expect("a block's padding");
    let textcol = usize::try_from(bd.textcol).expect("a column of the line");
    let text = &line[textcol..];
    let textlen = pad(bd.textlen);

    // Lay the line out: the leading padding, the text, the trailing padding.
    let mut out = Vec::with_capacity(pad(bd.startspaces) + textlen + pad(endspaces) + 1);
    out.resize(pad(bd.startspaces), b' ');
    out.extend_from_slice(&text[..textlen]);
    out.resize(out.len() + pad(endspaces), b' ');

    if exclude_trailing_space {
        // Walk back over the trailing white space, a character at a time
        // so that a multi-byte character is not cut in half.
        let mut s = textlen;
        while s > 0 && ascii_iswhite(c_int::from(text[s - 1])) {
            s -= head_off(text, s - 1) + 1;
            out.pop();
        }
    }
    XString::from_vec(out)
}

/// Move the lines of `new` onto the end of `curr`.
///
/// This is what an uppercase register name does. A charwise append joins the
/// last old line and the first new one into a single line, unless 'cpoptions'
/// has `>`. `curr` keeps its width, time and extra data, as upstream's does.
fn append_to_register(curr: &mut YankReg, new: Vec<XString>, yank_type: MotionType) {
    // Appending linewise text makes the whole register linewise.
    if yank_type == kMTLineWise {
        curr.kind = kMTLineWise;
    }
    let join = curr.kind == kMTCharWise && !cpo_has(CpoFlag::REGAPPEND);
    let lines = curr.lines.lines_mut();
    let mut new = new.into_iter();
    if join && let Some(last_old) = lines.last_mut() {
        // Join the last old line and the first new one.
        if let Some(first_new) = new.next() {
            last_old.push_bytes(&first_new);
        }
    }
    lines.extend(new);
}

/// The "N lines yanked" message.
fn report_yank(op: &OpArg, yank_type: MotionType, yanklines: size_t) {
    let regname = op.regname;
    let name = if regname == NUL {
        Vec::new()
    } else {
        formatted!(100, gettext(c" into \"%c").to_bytes(), regname)
    };

    // The message may be the first thing that scrolls, so make sure the
    // window is up to date before it is written.
    update_topline(Win::current());
    if must_redraw.get() != 0 {
        let _ = update_screen();
    }

    let (one, many) = if yank_type == kMTBlockWise {
        (
            c"block of %ld line yanked%s",
            c"block of %ld lines yanked%s",
        )
    } else {
        (c"%ld line yanked%s", c"%ld lines yanked%s")
    };
    let fmt = ngettext(one, many, yanklines as c_ulong);
    let name = msg_bytes(&name);
    let _: bool = report_msg(0, || tr_plural!(fmt, yanklines as int64_t, name));
}

/// Yank the operator's region into `reg`.
///
/// With `append`, the new text goes onto the end of whatever `reg` already
/// holds; with `message`, the "N lines yanked" report is given. `op` must
/// describe a region of the current buffer.
pub fn op_yank_reg(op: &OpArg, message: bool, reg: Reg, append: bool) {
    let region = *op;
    let mut yank_type = region.motion_type;
    let mut yanklines = region.line_count as size_t;
    let mut yankendlnum = region.end.lnum;

    // Appending yanks into a scratch register first, then merges.
    let appending = append && reg.read(|reg| reg.lines.is_set());
    if !appending {
        reg.write(YankReg::clear);
    }

    // A charwise yank that starts in column 0 and ends before column 0 of
    // a later line is really a linewise one.
    let sel_old = region.is_visual && P_SEL.first_byte() != b'o';
    if region.motion_type == kMTCharWise
        && region.start.col == 0
        && !region.inclusive
        && !sel_old
        && region.end.col == 0
        && yanklines > 1
    {
        yank_type = kMTLineWise;
        yankendlnum -= 1;
        yanklines = yanklines.wrapping_sub(1);
    }

    let mut width = 0;
    if yank_type == kMTBlockWise {
        // A `$`-extended block has no fixed width.
        let narrow = Win::current().w_curswant == MAXCOL;
        width = region.end_vcol - region.start_vcol;
        if narrow && width > 0 {
            width -= 1;
        }
    }

    let mut lines = Vec::with_capacity(yanklines);
    let mut lnum = region.start.lnum;
    while lnum <= yankendlnum {
        match yank_type {
            kMTBlockWise => {
                let mut text = Buf::current().lines();
                let line = text.line(lnum);
                let bd = block_def_of(op, lnum, line);
                lines.push(yank_copy_line(line, &bd, region.excl_tr_ws));
            }
            kMTLineWise => lines.push(XString::from_bytes(Buf::current().lines().line(lnum))),
            kMTCharWise => {
                let mut bd = charwise_block_def(region.start, region.end, lnum, region.inclusive);
                let mut text = Buf::current().lines();
                let line = text.line(lnum);
                // The region may reach past the end of a short line.
                let rest = line.len() - usize::try_from(bd.textcol).expect("a column");
                bd.textlen = bd.textlen.min(c_int::try_from(rest).unwrap_or(c_int::MAX));
                lines.push(yank_copy_line(line, &bd, false));
            }
            _ => ::std::process::abort(),
        }
        lnum += 1;
    }

    if appending {
        reg.write(|curr| append_to_register(curr, lines, yank_type));
    } else {
        let new = YankReg {
            lines: RegLines::new(lines),
            kind: yank_type,
            width,
            timestamp: os_time(),
            extra: None,
        };
        reg.write(|slot| *slot = new);
    }

    if message {
        // A single charwise line is not worth reporting.
        if yank_type == kMTCharWise && yanklines == 1 {
            yanklines = 0;
        }
        if yanklines > p_report() as size_t {
            report_yank(op, yank_type, yanklines);
        }
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = region.start;
        Buf::current().b_op_end = region.end;
        if yank_type == kMTLineWise {
            Buf::current().b_op_start.col = 0;
            Buf::current().b_op_end.col = MAXCOL;
        }
        if yank_type != kMTLineWise && !region.inclusive {
            // An exclusive region's `']` is the character *before* the end.
            // Stepped on a copy: `decl` reads the current buffer itself.
            let mut end = Buf::current().b_op_end;
            decl(&mut end);
            Buf::current().b_op_end = end;
        }
    }
}

/// Render a register's type as the string `getregtype()` and `:registers`
/// show: `v`, `V`, or `CTRL-V` followed by the block width; empty for an
/// unknown type.
pub fn format_reg_type(reg_type: MotionType, reg_width: ColNr) -> ThinCString {
    match reg_type {
        kMTLineWise => ThinCString::from_bytes(b"V"),
        kMTCharWise => ThinCString::from_bytes(b"v"),
        kMTBlockWise => ThinCString::from_vec(format!("\x16{}", reg_width + 1).into_bytes()),
        _ => ThinCString::empty(),
    }
}

/// Fire TextYankPost with the `v:event` dictionary describing the yank of
/// `op` into `reg`.
///
/// Guarded against recursion: an autocommand that yanks would otherwise fire
/// this again. Runs arbitrary autocommands, under `textlock`.
pub fn do_autocmd_textyankpost(op: &OpArg, reg: Reg) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);

    if recursive.get() || !has_event(AutoEvent::TextYankPost) {
        return;
    }
    recursive.set(true);

    let mut save_v_event = SaveVEvent {
        sve_did_save: false,
        sve_hashtab: HashTab::new(),
    };
    let dict = get_v_event(&mut save_v_event);

    // `regcontents`: the register's lines, as a locked list.
    let (list, regtype) = reg.read(|reg| {
        let list = tv_list_alloc(reg.lines.len() as ptrdiff_t);
        for line in reg.lines.iter() {
            list.edit().push_bytes(Some(line));
        }
        list_set_lock(Some(list.edit()), VarLock::Fixed);
        (list, format_reg_type(reg.kind, reg.width))
    });
    let _ = dict.edit().add_list(b"regcontents", Some(list));
    let _ = dict.edit().add_str(b"regtype", Some(&regtype));

    let name = [op.regname as u8, NUL as u8];
    let _ = dict.edit().add_str(b"regname", Some(cstr::in_bytes(&name)));

    let flag = |set| if set { kBoolVarTrue } else { kBoolVarFalse };
    let _ = dict.edit().add_bool(b"inclusive", flag(op.inclusive));

    let operator = [get_op_char(op.op_type) as u8, NUL as u8];
    let _ = dict
        .edit()
        .add_str(b"operator", Some(cstr::in_bytes(&operator)));

    let _ = dict.edit().add_bool(b"visual", flag(op.is_visual));
    dict.edit().set_keys_readonly();

    // The buffer must not change under the yank that is still in flight.
    let locked = Lock::text();
    apply_autocmds(
        AutoEvent::TextYankPost,
        None,
        None,
        false,
        Buf::current_or_none(),
    );
    drop(locked);

    restore_v_event(dict, &mut save_v_event);
    recursive.set(false);
}

/// `y`: yank the operator's region into the register it names.
///
/// Answers false for an invalid register name, having beeped. `op` must
/// describe a region of the current buffer. Runs the clipboard provider and
/// TextYankPost, and so arbitrary Lua.
pub fn op_yank(op: &OpArg, message: bool) -> bool {
    let regname = op.regname;
    if regname != 0 && !valid_yank_reg(regname, true) {
        beep_flush();
        return false;
    }
    if regname == '_' as c_int {
        return true; // black hole: nothing to do
    }

    let reg = get_yank_register(regname, YREG_YANK);
    op_yank_reg(op, message, reg, is_append_register(regname));
    // This is what runs the clipboard provider's Lua.
    clipboard::set_clipboard(regname, reg);
    do_autocmd_textyankpost(op, reg);
    true
}
