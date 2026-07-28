//! What is on the screen: the `screen*()` cell queries, the `syn*()` syntax
//! queries and the highlight-group lookups.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{NUL, VAR_STRING, false_0, kListLenMayKnow, true_0};
use crate::src::nvim::eval::typval::{
    tv_get_lnum, tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf,
    tv_list_alloc_ret, tv_list_append_number, tv_list_append_string, tv_list_set_ret,
};
use crate::src::nvim::grid::{
    MAX_SCHAR_SIZE, grid_getchar, schar_from_char, schar_get, schar_get_first_codepoint,
};
use crate::src::nvim::highlight_group::{
    HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DIM, HL_INVERSE, HL_ITALIC, HL_NOCOMBINE, HL_OVERLINE,
    HL_STANDOUT, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDASHED, HL_UNDERDOTTED, HL_UNDERDOUBLE,
    HL_UNDERLINE, get_highlight_name_ext, highlight_color, highlight_exists, highlight_has_attr,
    syn_get_final_id, syn_name2id,
};
use crate::src::nvim::main::{curbuf, curwin};
use crate::src::nvim::mbyte::{utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memline::ml_get_len;
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::message::msg_scroll_flush;
use crate::src::nvim::syntax::{
    HL_CONCEAL, get_syntax_info, syn_get_id, syn_get_stack_item, syn_get_sub_char,
};
use crate::src::nvim::types::{EvalFuncData, ScreenGrid, colnr_T, schar_T, typval_T, varnumber_T};
use crate::src::nvim::ui::{ui_current_col, ui_current_row, ui_rgb_attached};
use crate::src::nvim::ui_compositor::ui_comp_get_grid_at_coord;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The size of a `tv_get_string_buf` scratch buffer. `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

/// One screen cell, resolved from the `{row}`, `{col}` pair every
/// `screen*()` query starts with.
///
/// The coordinates vimscript uses are one-based and name a point on the
/// *composed* screen; the grid they land on has its own origin, so they are
/// rebased onto it here. A pair that lands nowhere sensible comes out of
/// bounds rather than as an error, which is what these builtins report with
/// -1 or an empty answer.
struct Cell {
    grid: *mut ScreenGrid,
    row: c_int,
    col: c_int,
}

impl Cell {
    /// Resolve arguments 0 and 1.
    ///
    /// # Safety
    /// `args` is a live call frame.
    unsafe fn at(args: Args) -> Cell {
        // SAFETY: the caller's obligation; the compositor always answers
        // with a live grid.
        unsafe {
            // A coercion failure answers 0, which the -1 turns into an
            // out-of-range coordinate. The subtraction wraps because the C's
            // does: a `{row}` of INT_MIN is a silly argument, not a crash.
            let mut row =
                (tv_get_number_chk(args.ptr(0), ptr::null_mut()) as c_int).wrapping_sub(1);
            let mut col =
                (tv_get_number_chk(args.ptr(1), ptr::null_mut()) as c_int).wrapping_sub(1);
            // Legacy tests read printed messages back with screenchar(), so
            // the pending message scroll has to reach the grid first.
            msg_scroll_flush();
            let grid = ui_comp_get_grid_at_coord(row, col);
            row -= (*grid).comp_row;
            col -= (*grid).comp_col;
            Cell { grid, row, col }
        }
    }

    /// Whether the cell is on the grid it was rebased onto.
    ///
    /// # Safety
    /// `self.grid` is live.
    unsafe fn on_grid(&self) -> bool {
        // SAFETY: the constructor's grid is live.
        unsafe {
            self.row >= 0
                && self.row < (*self.grid).rows
                && self.col >= 0
                && self.col < (*self.grid).cols
        }
    }

    /// The cell's character, as the grid's packed representation.
    ///
    /// # Safety
    /// `self` is on the grid.
    unsafe fn schar(&self) -> schar_T {
        // SAFETY: the caller has checked the bounds.
        unsafe { grid_getchar(self.grid, self.row, self.col, ptr::null_mut()) }
    }

    /// The cell's character, spelled out as UTF-8 and NUL-terminated.
    ///
    /// # Safety
    /// `self` is on the grid.
    unsafe fn text(&self) -> [c_char; NUMBUFLEN] {
        let mut buf = [0 as c_char; NUMBUFLEN];
        debug_assert!(NUMBUFLEN > MAX_SCHAR_SIZE as usize);
        // SAFETY: the caller has checked the bounds; `schar_get` writes at
        // most `MAX_SCHAR_SIZE` bytes plus a terminator.
        unsafe { schar_get(buf.as_mut_ptr(), self.schar()) };
        buf
    }
}

/// `screenattr({row}, {col})` — the cell's highlight attribute, or -1 off
/// the grid.
pub unsafe extern "C" fn f_screenattr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; the attribute row is as long as the grid is
    // wide, which the bounds check has established.
    unsafe {
        let cell = Cell::at(args);
        rettv.vval.v_number = if cell.on_grid() {
            let offset = *(*cell.grid).line_offset.add(cell.row as usize) + cell.col as usize;
            *(*cell.grid).attrs.add(offset) as c_int
        } else {
            -1
        } as varnumber_T;
    }
}

/// `screenchar({row}, {col})` — the first codepoint in the cell, or -1 off
/// the grid.
pub unsafe extern "C" fn f_screenchar(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe {
        let cell = Cell::at(args);
        rettv.vval.v_number = if cell.on_grid() {
            schar_get_first_codepoint(cell.schar())
        } else {
            -1
        } as varnumber_T;
    }
}

/// `screenchars({row}, {col})` — every codepoint in the cell, including the
/// combining ones `screenchar()` drops.
pub unsafe extern "C" fn f_screenchars(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `rettv` is the cleared return value.
    unsafe {
        let cell = Cell::at(args);
        let list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        if !cell.on_grid() {
            return;
        }
        let buf = cell.text();
        // The C walks with a do-while, so a cell whose text is empty still
        // reports one codepoint.
        let mut i = 0usize;
        loop {
            tv_list_append_number(list, utf_ptr2char(buf.as_ptr().add(i)) as varnumber_T);
            i += utf_ptr2len(buf.as_ptr().add(i)) as usize;
            if buf[i] as c_int == NUL {
                break;
            }
        }
    }
}

/// `screencol()` — the cursor's screen column, one-based.
pub unsafe extern "C" fn f_screencol(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = (ui_current_col() + 1) as varnumber_T };
}

/// `screenrow()` — the cursor's screen row, one-based.
pub unsafe extern "C" fn f_screenrow(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = (ui_current_row() + 1) as varnumber_T };
}

/// `screenstring({row}, {col})` — the cell's whole text, or "" off the grid.
pub unsafe extern "C" fn f_screenstring(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the frame is live and `rettv` now owns the duplicated string.
    unsafe {
        let cell = Cell::at(args);
        if cell.on_grid() {
            rettv.vval.v_string = xstrdup(cell.text().as_ptr());
        }
    }
}

/// `hlID({name})` — the highlight group's id, or 0.
pub unsafe extern "C" fn f_hlID(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    rettv.vval.v_number = unsafe { syn_name2id(tv_get_string(args.ptr(0))) } as varnumber_T;
}

/// `hlexists({name})` — whether the group is defined.
pub unsafe extern "C" fn f_hlexists(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    rettv.vval.v_number = unsafe { highlight_exists(tv_get_string(args.ptr(0))) } as varnumber_T;
}

/// What a `synIDattr()` `{what}` argument selects.
enum Attr {
    /// A colour or a font name. `highlight_color` re-reads `{what}` itself,
    /// because the trailing `#` is part of the request.
    Color,
    /// The group's own name.
    Name,
    /// Whether one attribute bit is set.
    Bit(c_int),
}

/// Resolve `{what}`.
///
/// The C is a `switch` on the first letter with nested tests on as few more
/// as it takes to tell the spellings apart — `bg`/`blink`/`bold`,
/// `inverse`/`italic`, `nocombine`/`name`, `sp`/`strikethrough`/`standout`,
/// and the five underline styles. Every test is case-insensitive, and every
/// index it reads is inside the string because the byte after the last one
/// is the terminator: reading `what[i]` past the end here answers 0, which
/// takes the same branch the NUL did.
fn attr_selector(what: &[u8]) -> Option<Attr> {
    let at = |i: usize| what.get(i).copied().unwrap_or(0).to_ascii_lowercase();
    Some(match at(0) {
        b'b' => match at(1) {
            b'g' => Attr::Color,
            b'l' => Attr::Bit(HL_BLINK as c_int),
            _ => Attr::Bit(HL_BOLD as c_int),
        },
        b'c' => Attr::Bit(HL_CONCEALED as c_int),
        b'd' => Attr::Bit(HL_DIM as c_int),
        b'o' => Attr::Bit(HL_OVERLINE as c_int),
        b'f' => Attr::Color,
        b'i' if at(1) == b'n' => Attr::Bit(HL_INVERSE as c_int),
        b'i' => Attr::Bit(HL_ITALIC as c_int),
        b'n' if at(1) == b'o' => Attr::Bit(HL_NOCOMBINE as c_int),
        b'n' => Attr::Name,
        b'r' => Attr::Bit(HL_INVERSE as c_int),
        b's' => match at(1) {
            b'p' => Attr::Color,
            b't' if at(2) == b'r' => Attr::Bit(HL_STRIKETHROUGH as c_int),
            _ => Attr::Bit(HL_STANDOUT as c_int),
        },
        // `ul` is the underline *colour*; every other `u` spelling is one
        // of the five underline styles, and those are only told apart from
        // the sixth byte on — which is why the length guard comes first.
        b'u' if what.len() < 9 => Attr::Color,
        b'u' => match (at(5), at(6), at(7)) {
            (b'l', _, _) => Attr::Bit(HL_UNDERLINE as c_int),
            (c, _, _) if c != b'd' => Attr::Bit(HL_UNDERCURL as c_int),
            (_, c, _) if c != b'o' => Attr::Bit(HL_UNDERDASHED as c_int),
            (_, _, b'u') => Attr::Bit(HL_UNDERDOUBLE as c_int),
            _ => Attr::Bit(HL_UNDERDOTTED as c_int),
        },
        _ => return None,
    })
}

/// `synIDattr({id}, {what} [, {mode}])`
pub unsafe extern "C" fn f_synIDattr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; `what` is the string an argument owns and
    // outlives the `highlight_color` call, and `modebuf` outlives the string
    // `tv_get_string_buf` may park in it.
    unsafe {
        let id = tv_get_number(args.ptr(0)) as c_int;
        let what = tv_get_string(args.ptr(1));

        // "cterm" or "gui"; anything else, including an absent argument,
        // means whatever the attached UI is.
        let modec = if args.has(2) {
            let mut modebuf = [0 as c_char; NUMBUFLEN];
            let mode = tv_get_string_buf(args.ptr(2), modebuf.as_mut_ptr());
            match (*mode as u8).to_ascii_lowercase() {
                c @ (b'c' | b'g') => c as c_int,
                _ => 0,
            }
        } else if ui_rgb_attached() {
            'g' as c_int
        } else {
            'c' as c_int
        };

        let p = match attr_selector(CStr::from_ptr(what).to_bytes()) {
            Some(Attr::Color) => highlight_color(id, what, modec),
            Some(Attr::Name) => get_highlight_name_ext(ptr::null_mut(), id - 1, false),
            Some(Attr::Bit(bit)) => highlight_has_attr(id, bit, modec),
            None => ptr::null(),
        };
        rettv.v_type = VAR_STRING;
        rettv.vval.v_string = if p.is_null() {
            ptr::null_mut()
        } else {
            xstrdup(p)
        };
    }
}

/// `synID({lnum}, {col}, {trans})` — the syntax id at a position, 0 off the
/// buffer or when the `{trans}` argument does not coerce.
pub unsafe extern "C" fn f_synID(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live, and `curbuf`/`curwin` are live for the
    // whole call.
    unsafe {
        let lnum = tv_get_lnum(args.ptr(0));
        // Wraps because the C's does; `col` is only used as a range test.
        let col = (tv_get_number(args.ptr(1)) as colnr_T).wrapping_sub(1);
        let mut transerr = false;
        let trans = tv_get_number_chk(args.ptr(2), &raw mut transerr) as c_int;

        let mut id = 0;
        if !transerr
            && lnum >= 1
            && lnum <= (*curbuf.get()).b_ml.ml_line_count
            && col >= 0
            && col < ml_get_len(lnum)
        {
            id = syn_get_id(curwin.get(), lnum, col, trans, ptr::null_mut(), false_0);
        }
        rettv.vval.v_number = id as varnumber_T;
    }
}

/// `synIDtrans({id})` — the id the group's `:hi link` chain ends at.
pub unsafe extern "C" fn f_synIDtrans(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe {
        let id = tv_get_number(args.ptr(0)) as c_int;
        rettv.vval.v_number = if id > 0 { syn_get_final_id(id) } else { 0 } as varnumber_T;
    }
}

/// `synconcealed({lnum}, {col})` — `[concealed, replacement, group]`.
pub unsafe extern "C" fn f_synconcealed(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut syntax_flags = 0;
    let mut matchid = 0;
    let mut text = [0 as c_char; NUMBUFLEN];
    // SAFETY: the frame is live; `curbuf`/`curwin` are live for the whole
    // call and `text` outlives the list it is copied into.
    unsafe {
        // Cleared first: an out-of-range position answers an empty List,
        // not a three-item one.
        tv_list_set_ret(rettv, ptr::null_mut());
        let lnum = tv_get_lnum(args.ptr(0));
        // Wraps because the C's does.
        let col = (tv_get_number(args.ptr(1)) as colnr_T).wrapping_sub(1);

        // Note the `<=`: unlike synID(), the position one past the end of
        // the line is in range here.
        if lnum >= 1
            && lnum <= (*curbuf.get()).b_ml.ml_line_count
            && col >= 0
            && col <= ml_get_len(lnum)
            && (*curwin.get()).w_onebuf_opt.wo_cole > 0
        {
            // Run the syntax engine for its side effect: `get_syntax_info`
            // reports on the position it last looked at.
            syn_get_id(curwin.get(), lnum, col, false_0, ptr::null_mut(), false_0);
            syntax_flags = get_syntax_info(&raw mut matchid);
            if syntax_flags & HL_CONCEAL as c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_cole < 3 {
                let mut cchar = schar_from_char(syn_get_sub_char());
                // At 'conceallevel' 1 a group with no `cchar` falls back to
                // 'listchars' "conceal", and to a space if that is unset.
                if cchar == NUL as schar_T && (*curwin.get()).w_onebuf_opt.wo_cole == 1 {
                    cchar = match (*curwin.get()).w_p_lcs_chars.conceal {
                        c if c == NUL as schar_T => ' ' as schar_T,
                        c => c,
                    };
                }
                if cchar != NUL as schar_T {
                    schar_get(text.as_mut_ptr(), cchar);
                }
            }
        }

        let list = tv_list_alloc_ret(rettv, 3);
        tv_list_append_number(
            list,
            (syntax_flags & HL_CONCEAL as c_int != 0) as c_int as varnumber_T,
        );
        tv_list_append_string(list, text.as_ptr(), -1);
        tv_list_append_number(list, matchid as varnumber_T);
    }
}

/// `synstack({lnum}, {col})` — every syntax id in effect at a position,
/// outermost first.
pub unsafe extern "C" fn f_synstack(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; `curbuf`/`curwin` are live for the whole
    // call.
    unsafe {
        // An out-of-range position answers an empty List, not a List of no
        // items.
        tv_list_set_ret(rettv, ptr::null_mut());
        let lnum = tv_get_lnum(args.ptr(0));
        // Wraps because the C's does.
        let col = (tv_get_number(args.ptr(1)) as colnr_T).wrapping_sub(1);

        if lnum >= 1
            && lnum <= (*curbuf.get()).b_ml.ml_line_count
            && col >= 0
            && col <= ml_get_len(lnum)
        {
            let list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
            // Run the syntax engine, keeping the stack this time.
            syn_get_id(curwin.get(), lnum, col, false_0, ptr::null_mut(), true_0);
            for i in 0.. {
                let id = syn_get_stack_item(i);
                if id < 0 {
                    break;
                }
                tv_list_append_number(list, id as varnumber_T);
            }
        }
    }
}
