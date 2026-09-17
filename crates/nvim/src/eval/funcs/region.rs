//! The text a Visual selection covers: `getregion()` and
//! `getregionpos()`.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::list_alloc_ret;
use super::{kMTBlockWise, kMTCharWise, kMTLineWise};
use crate::api::private::helpers::cbuf_to_string;
use crate::buffer::find_buf;
use crate::charset::getdigits_int;
use crate::eval::list2fpos;
use crate::eval::typval::{
    NumBuf, dict_get_bool, tv_check_for_list_arg, tv_check_for_opt_dict_arg, tv_list_alloc,
};
use crate::keycodes::Ctrl_V;
use crate::mbyte::{mb_prevptr, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_buf_len, ml_get_len, ml_get_pos};
use crate::memory::xmalloc;
use crate::message::e_buffer_is_not_loaded;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::normal::unadjust_for_sel_inner;
use crate::ops::{block_prep, charwise_block_prep, reset_lbr, restore_lbr};
use crate::option::vars::P_SEL;
use crate::os::cshim::gettext;
use crate::plines::getvvcol;
use crate::pos::{MAXCOL, equalpos, lt};
use crate::semsg;
use crate::state::mode::virtual_op;
use crate::state::virtual_active;
use crate::types::{
    BlockDef, ColNr, EvalFuncData, LineNr, MotionType, NUL, OpArg, OpType, Pos, String_0, TypVal,
    VAR_DICT, VarNumber, kListLenMayKnow,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::winlayer::{Buf, Win};
/// The zeroed position every local in this module starts from.
const NOWHERE: Pos = Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// A cleared block description. `block_prep` and `charwise_block_prep`
/// fill it; nothing reads it before they do.
const NO_BLOCK: BlockDef = BlockDef {
    startspaces: 0,
    endspaces: 0,
    textlen: 0,
    textstart: ptr::null_mut(),
    textcol: 0,
    start_vcol: 0,
    end_vcol: 0,
    is_short: 0,
    is_max: 0,
    is_one_char: 0,
    pre_whitesp: 0,
    pre_whitesp_c: 0,
    end_char_vcols: 0,
    start_char_vcols: 0,
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
        Win::current().w_buffer = self.buf.raw();
        virtual_op.set(self.virtual_op);
    }
}

/// Resolve `getregion()`'s and `getregionpos()`'s shared arguments, leaving
/// the current buffer pointed at the one the positions name.
fn resolve(args: &[TypVal], result: &mut TypVal) -> Option<Region> {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `p1`/`p2` are locals the List parser
    // fills, and every line accessor below runs against `findbuf`, which is
    // made current before it is read from.
    list_alloc_ret(result, kListLenMayKnow as isize);
    if tv_check_for_list_arg(args, 0).is_err()
        || tv_check_for_list_arg(args, 1).is_err()
        || tv_check_for_opt_dict_arg(args, 2).is_err()
    {
        return None;
    }
    let (mut p1, mut p2) = (NOWHERE, NOWHERE);
    let (mut fnum1, mut fnum2) = (-1, -1);
    let (out1, buf1) = (&raw mut p1, &raw mut fnum1);
    let (out2, buf2) = (&raw mut p2, &raw mut fnum2);
    let nul = ptr::null_mut();
    // SAFETY: both arguments are live typvals and the four out-parameters
    // are locals. The second is only read when the first parsed, as
    // upstream's short-circuit has it.
    if unsafe { list2fpos(&args[0], out1, buf1, nul, false) }.is_err()
        || unsafe { list2fpos(&args[1], out2, buf2, nul, false) }.is_err()
        || fnum1 != fnum2
    {
        return None;
    }

    // 'selection' decides the default exclusivity; an option dict may
    // override it and may name the region type.
    let opts =
        (args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT)).then(|| args[2].dict_or_null());
    let exclusive_by_default = P_SEL.first_byte() == b'e';
    let (is_select_exclusive, spec) = match opts {
        Some(d) => {
            // SAFETY: the argument's own dictionary.
            let d = unsafe { d.as_ref() };
            (
                dict_get_bool(d, b"exclusive", exclusive_by_default as c_int) != 0,
                numbuf.dict_string(d, b"type"),
            )
        }
        None => (exclusive_by_default, ptr::null()),
    };
    let spec: *const c_char = if spec.is_null() { c"v".as_ptr() } else { spec };
    let (region_type, block_width) = unsafe { parse_type(spec) }?;

    let findbuf = if fnum1 != 0 {
        find_buf(fnum1).map_or(ptr::null_mut(), |b| b.raw())
    } else {
        Buf::current_raw()
    };
    // SAFETY: `find_buf` and `curbuf` are both a live buffer or null.
    let loaded = (unsafe { Buf::from_raw(findbuf) }).filter(|b| !b.b_ml.ml_mfp.is_null());
    let Some(findbuf) = loaded else {
        emsg(gettext(e_buffer_is_not_loaded));
        return None;
    };
    check_corner(findbuf, &mut p1)?;
    check_corner(findbuf, &mut p2)?;

    findbuf.make_current();
    Win::current().w_buffer = findbuf.raw();
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
        if inclusive
            && virtual_op.get() == Some(false)
            && unsafe { *ml_get_pos(&raw mut p2) } == NUL as c_char
        {
            inclusive = false;
        }
    } else if region_type == kMTBlockWise {
        op = block_oparg(p1, p2, is_select_exclusive, block_width);
    }

    // Extend the far corner over the rest of a multibyte character.
    let l = unsafe { utfc_ptr2len(ml_get_pos(&raw mut p2)) };
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

/// The `type` option: "v", "V", or CTRL-V optionally followed by a width.
///
/// # Safety
/// `spec` is NUL-terminated.
unsafe fn parse_type(spec: *const c_char) -> Option<(MotionType, c_int)> {
    // SAFETY throughout: the caller's obligation; `getdigits_int` only walks forward
    // over `spec` and leaves `p` on the terminator when it consumed the
    // whole width.
    let bad = || {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (arg0, spec) = unsafe { (c_str(c"type".as_ptr()), c_str(spec)) };
        semsg!("E475: Invalid value for argument {arg0}: {spec}");
        None
    };
    match unsafe { CStr::from_ptr(spec) }.to_bytes() {
        b"v" => Some((kMTCharWise, 0)),
        b"V" => Some((kMTLineWise, 0)),
        [c, ..] if *c as c_int == Ctrl_V => {
            let mut p = unsafe { spec.add(1) } as *mut c_char;
            // A bare CTRL-V means "as wide as the corners"; a width
            // must be a positive number and nothing else.
            if unsafe { *p } != NUL as c_char {
                let width = unsafe { getdigits_int(&raw mut p, false, 0) };
                if width <= 0 || unsafe { *p } != NUL as c_char {
                    return bad();
                }
                return Some((kMTBlockWise, width));
            }
            Some((kMTBlockWise, 0))
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
    // SAFETY throughout: 'linebreak' is turned off around
    // the virtual-column measurements so that a wrapped line does not
    // change where the block's edges are.
    let (mut sc1, mut ec1, mut sc2, mut ec2) = (0, 0, 0, 0);
    let lbr_saved = reset_lbr();
    let (at1, at2) = (&raw const p1 as *mut Pos, &raw const p2 as *mut Pos);
    let nul = ptr::null_mut();
    // SAFETY: the two positions and the four out-parameters are locals.
    unsafe { getvvcol(Win::current(), at1, &raw mut sc1, nul, &raw mut ec1) };
    unsafe { getvvcol(Win::current(), at2, &raw mut sc2, nul, &raw mut ec2) };
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

/// The text a block description covers: its leading pad, its bytes, then
/// its trailing pad. The pads are what a blockwise selection through a tab
/// or a wide character turns into.
///
/// # Safety
/// `bd` has been filled by one of the block-prep functions.
unsafe fn block_def2str(bd: &BlockDef) -> String_0 {
    // SAFETY throughout: the caller's obligation. The allocation is exactly the three
    // pieces plus a terminator, and each piece is written once in order.
    let size = bd.startspaces as usize + bd.endspaces as usize + bd.textlen as usize;
    let data = unsafe { xmalloc(size + 1) }.cast::<c_char>();
    // SAFETY throughout: `data` has room for the three runs written below, which is
    // what `size` was computed from, plus the terminator.
    let space = b' ' as c_int;
    let into = data.cast::<u8>();
    unsafe { into.write_bytes((space) as u8, bd.startspaces as usize) };
    let mut at = bd.startspaces as usize;
    let (dst, src) = unsafe { (data.add(at), bd.textstart) };
    unsafe { dst.cast::<u8>().copy_from(src.cast(), bd.textlen as usize) };
    at += bd.textlen as usize;
    let dst = unsafe { data.add(at).cast::<c_void>() };
    let into = dst.cast::<u8>();
    unsafe { into.write_bytes((space) as u8, bd.endspaces as usize) };
    at += bd.endspaces as usize;
    unsafe { *data.add(at) = NUL as c_char };
    // SAFETY: `data` is this function's own block, NUL-terminated above.
    unsafe { String_0::from_owned_parts(data, at) }
}

/// `getregion({pos1}, {pos2} [, {opts}])` — the selected text, one String
/// per line.
pub fn f_getregion(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _swap = BufferSwap::save();
    let Some(r) = resolve(args, result) else {
        return;
    };
    for lnum in r.p1.lnum..=r.p2.lnum {
        let text = if r.region_type == kMTBlockWise {
            let mut bd = NO_BLOCK;
            unsafe { block_prep(&raw const r.op as *mut OpArg, &raw mut bd, lnum, false) };
            unsafe { block_def2str(&bd) }
        } else if r.region_type == kMTLineWise || (r.p1.lnum < lnum && lnum < r.p2.lnum) {
            // A whole line: either the region is linewise, or this is
            // an interior line of a charwise region.
            unsafe { cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as usize) }
        } else {
            let mut bd = NO_BLOCK;
            unsafe { charwise_block_prep(r.p1, r.p2, &raw mut bd, lnum, r.inclusive) };
            unsafe { block_def2str(&bd) }
        };
        debug_assert!(!text.data().is_null());
        // The list takes the block over, so the string gives it up rather
        // than releasing it here.
        unsafe { (*result.list_or_null()).push_allocated_string(text.into_raw()) };
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
        let line = ml_get(lnum);
        let line_len = ml_get_len(lnum);
        let (mut ret_p1, mut ret_p2) = unsafe { line_corners(&r, lnum, line) };
        clamp_corners(&mut ret_p1, &mut ret_p2, line_len, allow_eol);
        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;
        add_regionpos_range(result, ret_p1, ret_p2);
    }
}

/// Where the region starts and ends on one line, in one-based columns with
/// a virtual offset.
///
/// # Safety
/// `line` is line `lnum` of the current buffer and `r` describes a region
/// covering it.
unsafe fn line_corners(r: &Region, lnum: LineNr, line: *mut c_char) -> (Pos, Pos) {
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
    // SAFETY throughout: the caller's obligation; `bd.textstart` points into `line`,
    // so `mb_prevptr` stays inside it.
    let mut bd = NO_BLOCK;
    if r.region_type == kMTBlockWise {
        unsafe { block_prep(&raw const r.op as *mut OpArg, &raw mut bd, lnum, false) };
    } else {
        unsafe { charwise_block_prep(r.p1, r.p2, &raw mut bd, lnum, r.inclusive) };
    }

    let mut p1 = NOWHERE;
    if bd.is_one_char != 0 {
        if r.region_type == kMTBlockWise {
            p1.col = unsafe { mb_prevptr(line, bd.textstart).offset_from(line) } as ColNr + 1;
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
        p1.col = unsafe { mb_prevptr(line, bd.textstart).offset_from(line) } as ColNr + 1;
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
    let into = pair.as_ptr();
    unsafe { (*result.list_or_null()).push_list(Some(pair)) };
    for p in [p1, p2] {
        let pos = tv_list_alloc(4);
        let l = pos.as_ptr();
        unsafe { (*into).push_list(Some(pos)) };
        unsafe { (*l).push_number(Buf::current().handle as VarNumber) };
        unsafe { (*l).push_number(p.lnum as VarNumber) };
        unsafe { (*l).push_number(p.col as VarNumber) };
        unsafe { (*l).push_number(p.coladd as VarNumber) };
    }
}
