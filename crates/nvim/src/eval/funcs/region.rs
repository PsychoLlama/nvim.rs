//! The text a Visual selection covers: `getregion()` and
//! `getregionpos()`.
#![forbid(unsafe_code)]

use super::{kMTBlockWise, kMTCharWise, kMTLineWise};
use crate::buffer::find_buf;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::eval::list2fpos;
use crate::eval::typval::{
    NumBuf, dict_get_bool, tv_check_for_list_arg, tv_check_for_opt_dict_arg, tv_list_alloc,
    tv_list_alloc_ret,
};
use crate::keycodes::Ctrl_V;
use crate::mbyte::{cluster_len, head_off};
use crate::memline::{Lines, ml_get_buf_len};
use crate::memory::ThinCString;
use crate::message::e_buffer_is_not_loaded;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::normal::unadjust_for_sel_inner;
use crate::ops::{block_def, charwise_block_def, reset_lbr, restore_lbr};
use crate::option::vars::P_SEL;
use crate::os::cshim::gettext;
use crate::pos::{MAXCOL, equalpos, lt};
use crate::semsg;
use crate::state::mode::virtual_op;
use crate::state::virtual_active;
use crate::types::{
    BlockDef, ColNr, EvalFuncData, LineNr, MotionType, NUL, OpArg, OpType, Pos, TypVal, VAR_DICT,
    VarNumber, kListLenMayKnow,
};
use core::ffi::{CStr, c_int};

use crate::winlayer::{Buf, Win};
/// The zeroed position every local in this module starts from.
const NOWHERE: Pos = Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// A cleared operator argument, which only the blockwise path fills in.
const NO_OPARG: OpArg = OpArg {
    op_type: OpType::Nop,
    regname: 0,
    motion_type: kMTCharWise,
    motion_force: 0,
    use_reg_one: false,
    inclusive: false,
    end_adjusted: false,
    start: NOWHERE,
    end: NOWHERE,
    cursor_start: NOWHERE,
    line_count: 0,
    empty: false,
    is_visual: false,
    start_vcol: 0,
    end_vcol: 0,
    prev_opcount: 0,
    prev_count0: 0,
    excl_tr_ws: false,
};

/// What `getregionpos` resolved the arguments to.
struct Region {
    /// The upper-left corner, zero-based, after the swap and the
    /// exclusivity adjustment.
    p1: Pos,
    /// The lower-right corner, zero-based, extended to the end of a
    /// multibyte character.
    p2: Pos,
    /// Whether `p2`'s character is part of the selection.
    inclusive: bool,
    region_type: MotionType,
    /// Only meaningful for a blockwise region.
    op: OpArg,
}

/// Restores `curbuf` and 'virtualedit' when the builtin returns.
///
/// Both entry points move the current buffer to the one the positions name
/// so that the line accessors answer for it, and both must put it back
/// however they leave.
struct BufferSwap {
    buf: Buf,
    virtual_op: Option<bool>,
}

impl BufferSwap {
    fn save() -> Self {
        BufferSwap {
            buf: Buf::current(),
            virtual_op: virtual_op.get(),
        }
    }
}

impl Drop for BufferSwap {
    fn drop(&mut self) {
        self.buf.make_current();
        // `curwin` is live for the whole of a builtin call.
        Win::current().w_buffer = self.buf;
        virtual_op.set(self.virtual_op);
    }
}

/// Resolve `getregion()`'s and `getregionpos()`'s shared arguments, leaving
/// the current buffer pointed at the one the positions name.
fn resolve(args: &[TypVal], result: &mut TypVal) -> Option<Region> {
    let mut numbuf = NumBuf::new();
    // Every line accessor below runs against `findbuf`, which is made
    // current before it is read from.
    tv_list_alloc_ret(result, kListLenMayKnow as isize);
    if tv_check_for_list_arg(args, 0).is_err()
        || tv_check_for_list_arg(args, 1).is_err()
        || tv_check_for_opt_dict_arg(args, 2).is_err()
    {
        return None;
    }
    let (mut p1, mut p2) = (NOWHERE, NOWHERE);
    let (mut fnum1, mut fnum2) = (-1, -1);
    // The second is only read when the first parsed, as upstream's
    // short-circuit has it.
    if list2fpos(&args[0], &mut p1, Some(&mut fnum1), None, false).is_err()
        || list2fpos(&args[1], &mut p2, Some(&mut fnum2), None, false).is_err()
        || fnum1 != fnum2
    {
        return None;
    }

    // 'selection' decides the default exclusivity; an option dict may
    // override it and may name the region type.
    let opts =
        (args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT)).then(|| args[2].dict_ref());
    let exclusive_by_default = P_SEL.first_byte() == b'e';
    let (is_select_exclusive, spec) = match opts {
        Some(d) => (
            dict_get_bool(d, b"exclusive", exclusive_by_default as c_int) != 0,
            numbuf.dict_string(d, b"type"),
        ),
        None => (exclusive_by_default, None),
    };
    let (region_type, block_width) = parse_type(spec.unwrap_or(c"v"))?;

    let findbuf = if fnum1 != 0 {
        find_buf(fnum1)
    } else {
        Buf::current_or_none()
    };
    let loaded = findbuf.filter(|b| !b.b_ml.ml_mfp.is_null());
    let Some(findbuf) = loaded else {
        emsg(gettext(e_buffer_is_not_loaded));
        return None;
    };
    check_corner(findbuf, &mut p1)?;
    check_corner(findbuf, &mut p2)?;

    findbuf.make_current();
    Win::current().w_buffer = findbuf;
    virtual_op.set(Some(virtual_active(Win::current())));

    // Columns are one-based on the way in and zero-based from here.
    p1.col -= 1;
    p2.col -= 1;
    if !lt(p1, p2) {
        core::mem::swap(&mut p1, &mut p2);
    }

    let mut inclusive = true;
    let mut op = NO_OPARG;
    if region_type == kMTCharWise {
        if is_select_exclusive && !equalpos(p1, p2) {
            inclusive = !unadjust_for_sel_inner(&mut p2);
        }
        // An inclusive selection ending on the line terminator does not
        // actually cover a character, unless 'virtualedit' is on.
        if inclusive && virtual_op.get() == Some(false) && byte_at(p2) == NUL as u8 {
            inclusive = false;
        }
    } else if region_type == kMTBlockWise {
        op = block_oparg(p1, p2, is_select_exclusive, block_width);
    }

    // Extend the far corner over the rest of a multibyte character.
    let l = cluster_at(p2);
    if l > 1 {
        p2.col += l - 1;
    }
    Some(Region {
        p1,
        p2,
        inclusive,
        region_type,
        op,
    })
}

/// The byte at `pos` of the current buffer, NUL at the end of its line.
fn byte_at(pos: Pos) -> u8 {
    let mut lines = Buf::current().lines();
    cstr::byte_at(lines.line(pos.lnum), usize::try_from(pos.col).unwrap_or(0))
}

/// How many bytes the grapheme cluster at `pos` takes, 0 at the end of the
/// line -- which is what `utfc_ptr2len` answered for the terminator.
fn cluster_at(pos: Pos) -> ColNr {
    let mut lines = Buf::current().lines();
    let line = lines.line(pos.lnum);
    let at = usize::try_from(pos.col).unwrap_or(0);
    if at >= line.len() {
        return 0;
    }
    ColNr::try_from(cluster_len(&line[at..])).unwrap_or(0)
}

/// The `type` option: "v", "V", or CTRL-V optionally followed by a width.
fn parse_type(spec: &CStr) -> Option<(MotionType, c_int)> {
    let bad = || {
        let (arg0, spec) = (msg_cstr(c"type"), msg_cstr(spec));
        semsg!("E475: Invalid value for argument {arg0}: {spec}");
        None
    };
    match spec.to_bytes() {
        b"v" => Some((kMTCharWise, 0)),
        b"V" => Some((kMTLineWise, 0)),
        [c, rest @ ..] if c_int::from(*c) == Ctrl_V => {
            // A bare CTRL-V means "as wide as the corners"; a width
            // must be a positive number and nothing else.
            if rest.is_empty() {
                return Some((kMTBlockWise, 0));
            }
            let mut text = spec.to_bytes_with_nul().to_vec();
            let (width, past) = getdigits_int_at(&mut text, 1, false, 0);
            if width <= 0 || past != spec.to_bytes().len() {
                return bad();
            }
            Some((kMTBlockWise, width))
        }
        _ => bad(),
    }
}

/// Validate one corner against the buffer, resolving `MAXCOL` to the end of
/// its line.
fn check_corner(buffer: Buf, p: &mut Pos) -> Option<()> {
    if p.lnum < 1 || p.lnum > buffer.b_ml.ml_line_count {
        semsg!("E966: Invalid line number: {}", p.lnum);
        return None;
    }
    let len = ml_get_buf_len(buffer, p.lnum);
    if p.col == MAXCOL as ColNr {
        p.col = len + 1;
    } else if p.col < 1 || p.col > len + 1 {
        semsg!("E964: Invalid column number: {}", p.col);
        return None;
    }
    Some(())
}

/// The operator argument a blockwise region needs, which is what
/// `block_prep` reads per line.
/// `p1` and `p2` name positions in the current buffer.
fn block_oparg(p1: Pos, p2: Pos, is_select_exclusive: bool, block_width: c_int) -> OpArg {
    // 'linebreak' is turned off around the virtual-column measurements so
    // that a wrapped line does not change where the block's edges are.
    let lbr_saved = reset_lbr();
    let (sc1, ec1) = Win::current().virtual_vcol_span_at(p1);
    let (sc2, ec2) = Win::current().virtual_vcol_span_at(p2);
    restore_lbr(lbr_saved);
    let start_vcol = sc1.min(sc2);
    OpArg {
        motion_type: kMTBlockWise,
        inclusive: true,
        op_type: OpType::Nop,
        start: p1,
        end: p2,
        start_vcol,
        end_vcol: if block_width > 0 {
            // An explicit width wins over where the corners landed.
            start_vcol + block_width - 1
        } else if is_select_exclusive && ec1 < sc2 && sc2 > 0 && ec2 > ec1 {
            // Exclusive: the far corner's own column is not covered.
            sc2 - 1
        } else {
            ec1.max(ec2)
        },
        ..NO_OPARG
    }
}

/// Where a block description's text starts in its line: its column, or the
/// line's start for a charwise block that begins past the end.
fn text_start(bd: &BlockDef, line: &[u8]) -> usize {
    usize::try_from(bd.textcol)
        .ok()
        .filter(|&at| at <= line.len())
        .unwrap_or(0)
}

/// The text a block description of `line` covers: its leading pad, its
/// bytes, then its trailing pad. The pads are what a blockwise selection
/// through a tab or a wide character turns into.
fn block_def2str(bd: &BlockDef, line: &[u8]) -> ThinCString {
    let pad = |n: c_int| core::iter::repeat_n(b' ', usize::try_from(n).unwrap_or(0));
    let at = text_start(bd, line);
    // A charwise block's first line counts its terminator, which copying
    // the C string stopped at.
    let text = &line[at..];
    let text = &text[..usize::try_from(bd.textlen).unwrap_or(0).min(text.len())];
    let mut bytes = Vec::with_capacity(text.len() + 1);
    bytes.extend(pad(bd.startspaces));
    bytes.extend_from_slice(text);
    bytes.extend(pad(bd.endspaces));
    ThinCString::from_vec(bytes)
}

/// `getregion({pos1}, {pos2} [, {opts}])` — the selected text, one String
/// per line.
pub fn f_getregion(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _swap = BufferSwap::save();
    let Some(r) = resolve(args, result) else {
        return;
    };
    for lnum in r.p1.lnum..=r.p2.lnum {
        let bd = if r.region_type == kMTBlockWise {
            Some(block_def(&r.op, lnum))
        } else if r.region_type == kMTLineWise || (r.p1.lnum < lnum && lnum < r.p2.lnum) {
            // A whole line: either the region is linewise, or this is
            // an interior line of a charwise region.
            None
        } else {
            Some(charwise_block_def(r.p1, r.p2, lnum, r.inclusive))
        };
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        let text = match bd {
            Some(bd) => block_def2str(&bd, line),
            None => ThinCString::from_bytes(line),
        };
        let list = result.list_mut().expect("the list `resolve` allocated");
        list.push(TypVal::string(Some(text)));
    }
}

/// `getregionpos({pos1}, {pos2} [, {opts}])` — the selection as a pair of
/// positions per line.
pub fn f_getregionpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _swap = BufferSwap::save();
    let Some(r) = resolve(args, result) else {
        return;
    };
    // Whether a position may sit one past the end of its line.
    let allow_eol = args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT)
        && dict_get_bool(args[2].dict_ref(), b"eol", 0) != 0;

    for lnum in r.p1.lnum..=r.p2.lnum {
        let (mut ret_p1, mut ret_p2) = line_corners(&r, lnum);
        let line_len = Lines::current().line(lnum).len() as ColNr;
        clamp_corners(&mut ret_p1, &mut ret_p2, line_len, allow_eol);
        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;
        add_regionpos_range(result, ret_p1, ret_p2);
    }
}

/// Where the region starts and ends on one line, in one-based columns with
/// a virtual offset.
/// `r` describes a region covering line `lnum` of the current buffer.
fn line_corners(r: &Region, lnum: LineNr) -> (Pos, Pos) {
    if r.region_type == kMTLineWise {
        // A linewise region always covers the whole line.
        return (
            Pos { col: 1, ..NOWHERE },
            Pos {
                col: MAXCOL as ColNr,
                ..NOWHERE
            },
        );
    }
    let mut bd = if r.region_type == kMTBlockWise {
        block_def(&r.op, lnum)
    } else {
        charwise_block_def(r.p1, r.p2, lnum, r.inclusive)
    };
    // The one-based column of the character before the block's text.
    let before_text = |bd: &BlockDef| {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        let at = text_start(bd, line);
        let prev = if at == 0 {
            0
        } else {
            at - head_off(line, at - 1) - 1
        };
        prev as ColNr + 1
    };

    let mut p1 = NOWHERE;
    if bd.is_one_char != 0 {
        if r.region_type == kMTBlockWise {
            p1.col = before_text(&bd);
            p1.coladd = bd.start_char_vcols - (bd.start_vcol - r.op.start_vcol);
        } else {
            p1.col = r.p1.col + 1;
            p1.coladd = r.p1.coladd;
        }
    } else if r.region_type == kMTBlockWise && r.op.start_vcol > bd.start_vcol {
        // The block starts inside a character that begins before it.
        p1.col = MAXCOL as ColNr;
        p1.coladd = r.op.start_vcol - bd.start_vcol;
        bd.is_one_char = 1;
    } else if bd.startspaces > 0 {
        p1.col = before_text(&bd);
        p1.coladd = bd.start_char_vcols - bd.startspaces;
    } else {
        p1.col = bd.textcol + 1;
    }

    let mut p2 = NOWHERE;
    if bd.is_one_char != 0 {
        p2.col = p1.col;
        p2.coladd = p1.coladd + bd.startspaces + bd.endspaces;
    } else if bd.endspaces > 0 {
        p2.col = bd.textcol + bd.textlen + 1;
        p2.coladd = bd.endspaces;
    } else {
        p2.col = bd.textcol + bd.textlen;
    }
    (p1, p2)
}

/// Pull both corners back onto the line. Without `eol` a corner past the
/// last byte collapses to zero — "nothing here" — rather than to the line
/// end.
fn clamp_corners(p1: &mut Pos, p2: &mut Pos, line_len: ColNr, allow_eol: bool) {
    if !allow_eol && p1.col > line_len {
        p1.col = 0;
        p1.coladd = 0;
    } else if p1.col > line_len + 1 {
        p1.col = line_len + 1;
    }
    if !allow_eol && p2.col > line_len {
        // The end follows the start into "nothing here".
        p2.col = if p1.col == 0 { 0 } else { line_len };
        p2.coladd = 0;
    } else if p2.col > line_len + 1 {
        p2.col = line_len + 1;
    }
}

/// Append one line's `[[bufnr, lnum, col, off], [bufnr, lnum, col, off]]`.
/// `result` holds the list being built, and `curbuf` is the region's own
/// buffer -- the caller's `BufferSwap` has already put it there.
fn add_regionpos_range(result: &mut TypVal, p1: Pos, p2: Pos) {
    let pair = tv_list_alloc(2);
    for p in [p1, p2] {
        let pos = tv_list_alloc(4);
        let l = pos.edit();
        l.push_number(Buf::current().handle as VarNumber);
        l.push_number(p.lnum as VarNumber);
        l.push_number(p.col as VarNumber);
        l.push_number(p.coladd as VarNumber);
        pair.edit().push_list(Some(pos));
    }
    let list = result.list_mut().expect("the list `resolve` allocated");
    list.push_list(Some(pair));
}
