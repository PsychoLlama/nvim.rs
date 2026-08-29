//! The text a Visual selection covers: `getregion()` and
//! `getregionpos()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{check_arg, list_alloc_ret};
use super::{kMTBlockWise, kMTCharWise, kMTLineWise};
use crate::api::private::helpers::cbuf_to_string;
use crate::buffer::find_buf;
use crate::charset::getdigits_int;
use crate::eval::list2fpos;
use crate::eval::typval::{
    NumBuf, tv_check_for_list_arg, tv_check_for_opt_dict_arg, tv_dict_get_bool, tv_list_alloc,
    tv_list_append_allocated_string, tv_list_append_list, tv_list_append_number,
};
use crate::keycodes::Ctrl_V;
use crate::main::{curbuf, curwin, e_buffer_is_not_loaded, p_sel, virtual_op};
use crate::mbyte::{mb_prevptr, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_buf_len, ml_get_len, ml_get_pos};
use crate::memory::xmalloc;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::normal::unadjust_for_sel_inner;
use crate::ops::{block_prep, charwise_block_prep, reset_lbr, restore_lbr};
use crate::os::cshim::{gettext, memmove};
use crate::plines::getvvcol;
use crate::pos::{MAXCOL, equalpos, lt};
use crate::semsg;
use crate::state::virtual_active;
use crate::types::{
    EvalFuncData, FAIL, MotionType, NUL, OK, OP_NOP, String_0, VAR_DICT, block_def, buf_T, colnr_T,
    kListLenMayKnow, linenr_T, oparg_T, pos_T, typval_T, varnumber_T,
};
use ::libc::memset;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::winlayer::Win;
/// The zeroed position every local in this module starts from.
const NOWHERE: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// A cleared block description. `block_prep` and `charwise_block_prep`
/// fill it; nothing reads it before they do.
const NO_BLOCK: block_def = block_def {
    startspaces: 0,
    endspaces: 0,
    textlen: 0,
    textstart: ptr::null_mut(),
    textcol: 0,
    start_vcol: 0,
    end_vcol: 0,
    is_short: 0,
    is_MAX: 0,
    is_oneChar: 0,
    pre_whitesp: 0,
    pre_whitesp_c: 0,
    end_char_vcols: 0,
    start_char_vcols: 0,
};

/// A cleared operator argument, which only the blockwise path fills in.
const NO_OPARG: oparg_T = oparg_T {
    op_type: 0,
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
    is_VIsual: false,
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
    p1: pos_T,
    /// The lower-right corner, zero-based, extended to the end of a
    /// multibyte character.
    p2: pos_T,
    /// Whether `p2`'s character is part of the selection.
    inclusive: bool,
    region_type: MotionType,
    /// Only meaningful for a blockwise region.
    oap: oparg_T,
}

/// Restores `curbuf` and 'virtualedit' when the builtin returns.
///
/// Both entry points move the current buffer to the one the positions name
/// so that the line accessors answer for it, and both must put it back
/// however they leave.
struct BufferSwap {
    buf: *mut buf_T,
    virtual_op: Option<bool>,
}

impl BufferSwap {
    /// # Safety
    /// `curbuf` and `curwin` are live.
    unsafe fn save() -> Self {
        BufferSwap {
            buf: curbuf.get(),
            virtual_op: virtual_op.get(),
        }
    }
}

impl Drop for BufferSwap {
    fn drop(&mut self) {
        curbuf.set(self.buf);
        // SAFETY: `curwin` is live for the whole of a builtin call.
        unsafe { (*curwin.get()).w_buffer = self.buf };
        virtual_op.set(self.virtual_op);
    }
}

/// Resolve `getregion()`'s and `getregionpos()`'s shared arguments, leaving
/// the current buffer pointed at the one the positions name.
fn resolve(args: Args<'_>, rettv: &mut typval_T) -> Option<Region> {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `p1`/`p2` are locals the List parser
    // fills, and every line accessor below runs against `findbuf`, which is
    // made current before it is read from.
    list_alloc_ret(rettv, kListLenMayKnow as isize);
    if check_arg(args, 0, tv_check_for_list_arg) == FAIL
        || check_arg(args, 1, tv_check_for_list_arg) == FAIL
        || check_arg(args, 2, tv_check_for_opt_dict_arg) == FAIL
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
    if unsafe { list2fpos(args.ptr(0), out1, buf1, nul, false) } != OK
        || unsafe { list2fpos(args.ptr(1), out2, buf2, nul, false) } != OK
        || fnum1 != fnum2
    {
        return None;
    }

    // 'selection' decides the default exclusivity; an option dict may
    // override it and may name the region type.
    let opts = (args.ty(2) == VAR_DICT).then(|| unsafe { args.get(2).vval.v_dict });
    let exclusive_by_default = unsafe { *p_sel.get() } == b'e' as c_char;
    let (is_select_exclusive, spec) = match opts {
        Some(d) => (
            unsafe { tv_dict_get_bool(d, c"exclusive".as_ptr(), exclusive_by_default as c_int) }
                != 0,
            unsafe { numbuf.dict_string(d, c"type".as_ptr()) },
        ),
        None => (exclusive_by_default, ptr::null()),
    };
    let spec: *const c_char = if spec.is_null() { c"v".as_ptr() } else { spec };
    let (region_type, block_width) = unsafe { parse_type(spec) }?;

    let findbuf = if fnum1 != 0 {
        find_buf(fnum1).map_or(ptr::null_mut(), |mut b| b.raw())
    } else {
        curbuf.get()
    };
    if findbuf.is_null() || unsafe { (*findbuf).b_ml.ml_mfp }.is_null() {
        emsg(gettext(e_buffer_is_not_loaded));
        return None;
    }
    unsafe { check_corner(findbuf, &mut p1) }?;
    unsafe { check_corner(findbuf, &mut p2) }?;

    curbuf.set(findbuf);
    unsafe { (*curwin.get()).w_buffer = curbuf.get() };
    virtual_op.set(Some(virtual_active(cur_win())));

    // Columns are one-based on the way in and zero-based from here.
    p1.col -= 1;
    p2.col -= 1;
    if !lt(p1, p2) {
        core::mem::swap(&mut p1, &mut p2);
    }

    let mut inclusive = true;
    let mut oap = NO_OPARG;
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
        oap = block_oparg(p1, p2, is_select_exclusive, block_width);
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
        oap,
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
///
/// # Safety
/// `buf` is a loaded buffer.
unsafe fn check_corner(buf: *mut buf_T, p: &mut pos_T) -> Option<()> {
    // SAFETY: the caller's obligation; the line length is only read once
    // the line number has been checked.
    if p.lnum < 1 || p.lnum > unsafe { (*buf).b_ml.ml_line_count } {
        semsg!("E966: Invalid line number: {}", p.lnum);
        return None;
    }
    let len = unsafe { ml_get_buf_len(buf, p.lnum) };
    if p.col == MAXCOL as colnr_T {
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
fn block_oparg(p1: pos_T, p2: pos_T, is_select_exclusive: bool, block_width: c_int) -> oparg_T {
    // SAFETY throughout: 'linebreak' is turned off around
    // the virtual-column measurements so that a wrapped line does not
    // change where the block's edges are.
    let (mut sc1, mut ec1, mut sc2, mut ec2) = (0, 0, 0, 0);
    let lbr_saved = reset_lbr();
    let (at1, at2) = (&raw const p1 as *mut pos_T, &raw const p2 as *mut pos_T);
    let nul = ptr::null_mut();
    // SAFETY: the two positions and the four out-parameters are locals.
    unsafe { getvvcol(cur_win(), at1, &raw mut sc1, nul, &raw mut ec1) };
    unsafe { getvvcol(cur_win(), at2, &raw mut sc2, nul, &raw mut ec2) };
    restore_lbr(lbr_saved);
    let start_vcol = sc1.min(sc2);
    oparg_T {
        motion_type: kMTBlockWise,
        inclusive: true,
        op_type: OP_NOP,
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
unsafe fn block_def2str(bd: &block_def) -> String_0 {
    // SAFETY throughout: the caller's obligation. The allocation is exactly the three
    // pieces plus a terminator, and each piece is written once in order.
    let size = bd.startspaces as usize + bd.endspaces as usize + bd.textlen as usize;
    let data = unsafe { xmalloc(size + 1) }.cast::<c_char>();
    // SAFETY throughout: `data` has room for the three runs written below, which is
    // what `size` was computed from, plus the terminator.
    let space = b' ' as c_int;
    unsafe { memset(data.cast::<c_void>(), space, bd.startspaces as usize) };
    let mut at = bd.startspaces as usize;
    let (dst, src) = unsafe { (data.add(at).cast::<c_void>(), bd.textstart.cast()) };
    unsafe { memmove(dst, src, bd.textlen as usize) };
    at += bd.textlen as usize;
    let dst = unsafe { data.add(at).cast::<c_void>() };
    unsafe { memset(dst, space, bd.endspaces as usize) };
    at += bd.endspaces as usize;
    unsafe { *data.add(at) = NUL as c_char };
    String_0::from_raw_parts(data, at)
}

/// `getregion({pos1}, {pos2} [, {opts}])` — the selected text, one String
/// per line.
pub unsafe fn f_getregion(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the buffer swap
    // is undone when `_swap` drops, on every path out.
    let _swap = unsafe { BufferSwap::save() };
    let Some(r) = resolve(args, rettv) else {
        return;
    };
    for lnum in r.p1.lnum..=r.p2.lnum {
        let text = if r.region_type == kMTBlockWise {
            let mut bd = NO_BLOCK;
            unsafe { block_prep(&raw const r.oap as *mut oparg_T, &raw mut bd, lnum, false) };
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
        unsafe { tv_list_append_allocated_string(rettv.vval.v_list, text.data()) };
    }
}

/// `getregionpos({pos1}, {pos2} [, {opts}])` — the selection as a pair of
/// positions per line.
pub unsafe fn f_getregionpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the buffer swap
    // is undone when `_swap` drops, on every path out.
    let _swap = unsafe { BufferSwap::save() };
    let Some(r) = resolve(args, rettv) else {
        return;
    };
    // Whether a position may sit one past the end of its line.
    let allow_eol = args.ty(2) == VAR_DICT
        && unsafe { tv_dict_get_bool(args.get(2).vval.v_dict, c"eol".as_ptr(), 0) } != 0;

    for lnum in r.p1.lnum..=r.p2.lnum {
        let line = ml_get(lnum);
        let line_len = ml_get_len(lnum);
        let (mut ret_p1, mut ret_p2) = unsafe { line_corners(&r, lnum, line) };
        clamp_corners(&mut ret_p1, &mut ret_p2, line_len, allow_eol);
        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;
        add_regionpos_range(rettv, ret_p1, ret_p2);
    }
}

/// Where the region starts and ends on one line, in one-based columns with
/// a virtual offset.
///
/// # Safety
/// `line` is line `lnum` of the current buffer and `r` describes a region
/// covering it.
unsafe fn line_corners(r: &Region, lnum: linenr_T, line: *mut c_char) -> (pos_T, pos_T) {
    if r.region_type == kMTLineWise {
        // A linewise region always covers the whole line.
        return (
            pos_T { col: 1, ..NOWHERE },
            pos_T {
                col: MAXCOL as colnr_T,
                ..NOWHERE
            },
        );
    }
    // SAFETY throughout: the caller's obligation; `bd.textstart` points into `line`,
    // so `mb_prevptr` stays inside it.
    let mut bd = NO_BLOCK;
    if r.region_type == kMTBlockWise {
        unsafe { block_prep(&raw const r.oap as *mut oparg_T, &raw mut bd, lnum, false) };
    } else {
        unsafe { charwise_block_prep(r.p1, r.p2, &raw mut bd, lnum, r.inclusive) };
    }

    let mut p1 = NOWHERE;
    if bd.is_oneChar != 0 {
        if r.region_type == kMTBlockWise {
            p1.col = unsafe { mb_prevptr(line, bd.textstart).offset_from(line) } as colnr_T + 1;
            p1.coladd = bd.start_char_vcols - (bd.start_vcol - r.oap.start_vcol);
        } else {
            p1.col = r.p1.col + 1;
            p1.coladd = r.p1.coladd;
        }
    } else if r.region_type == kMTBlockWise && r.oap.start_vcol > bd.start_vcol {
        // The block starts inside a character that begins before it.
        p1.col = MAXCOL as colnr_T;
        p1.coladd = r.oap.start_vcol - bd.start_vcol;
        bd.is_oneChar = 1;
    } else if bd.startspaces > 0 {
        p1.col = unsafe { mb_prevptr(line, bd.textstart).offset_from(line) } as colnr_T + 1;
        p1.coladd = bd.start_char_vcols - bd.startspaces;
    } else {
        p1.col = bd.textcol + 1;
    }

    let mut p2 = NOWHERE;
    if bd.is_oneChar != 0 {
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
fn clamp_corners(p1: &mut pos_T, p2: &mut pos_T, line_len: colnr_T, allow_eol: bool) {
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
/// `rettv` holds the list being built, and `curbuf` is the region's own
/// buffer -- the caller's `BufferSwap` has already put it there.
fn add_regionpos_range(rettv: &mut typval_T, p1: pos_T, p2: pos_T) {
    // SAFETY: the caller's obligation; each list is handed to its parent
    // immediately, so none is leaked.
    let pair = unsafe { tv_list_alloc(2) };
    unsafe { tv_list_append_list(rettv.vval.v_list, pair) };
    for p in [p1, p2] {
        let l = unsafe { tv_list_alloc(4) };
        unsafe { tv_list_append_list(pair, l) };
        unsafe { tv_list_append_number(l, (*curbuf.get()).handle as varnumber_T) };
        unsafe { tv_list_append_number(l, p.lnum as varnumber_T) };
        unsafe { tv_list_append_number(l, p.col as varnumber_T) };
        unsafe { tv_list_append_number(l, p.coladd as varnumber_T) };
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
