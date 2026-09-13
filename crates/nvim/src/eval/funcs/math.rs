//! Numbers: arithmetic, the bitwise operators and the random-number
//! generator.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::VARNUMBER_MAX;
use super::wrappers::{arg_number_chk, arg_string, list_alloc_ret, tv_get_float_chk};
use crate::charset::skipwhite;
use crate::eval::string2float;
use crate::eval::typval::{NumBuf, tv_list_append_number, tv_list_find, tv_list_len};
use crate::event::libuv::uv_random;
use crate::global_cell::GlobalCell;
use crate::message_fmt::c_str;
use crate::os::env::os_get_pid;
use crate::os::time::os_hrtime;
use crate::semsg;
use crate::types::{EvalFuncData, Float, TypVal, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VarNumber};
use core::ffi::{c_char, c_double, c_int, c_void};
use core::ptr;

/// `abs({expr})` — magnitude, as a Float for a Float and as a Number
/// otherwise. A value that is not coercible to a number reports through
/// `tv_get_number_chk` and yields -1, as upstream does.
pub fn f_abs(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() == VAR_FLOAT {
        result.write_float(args[0].float_or_zero().abs());
        return;
    }
    let mut error = false;
    let n = arg_number_chk(&args[0], Some(&mut error));
    result.write_number(if error {
        -1
    } else if n > 0 {
        n
    } else {
        // Not `-n`: `wrapping_neg` keeps the C's two's-complement answer for
        // the one value whose negation does not fit.
        n.wrapping_neg()
    });
}

/// The bitwise operators. Each coerces both arguments with a null error
/// pointer, so a non-coercible argument reports its own message and
/// contributes zero.
pub fn f_and(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(number(args, 0) & number(args, 1));
}

pub fn f_or(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(number(args, 0) | number(args, 1));
}

pub fn f_xor(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(number(args, 0) ^ number(args, 1));
}

pub fn f_invert(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(!number(args, 0));
}

/// Argument `i` as a Number, reporting its own error and reading as 0 when
/// it cannot be coerced. The bitwise builtins' shared coercion.
fn number(args: &[TypVal], i: usize) -> VarNumber {
    arg_number_chk(&args[i], None)
}

/// The two-argument float builtins. Both arguments are read left to right
/// and the second is only read once the first succeeded, so a pair of bad
/// arguments reports E808 once.
pub fn f_atan2(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    float2(args, result, |x, y| x.atan2(y));
}

pub fn f_fmod(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // Rust's `%` on floats is C's `fmod`.
    float2(args, result, |x, y| x % y);
}

pub fn f_pow(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    float2(args, result, c_double::powf);
}

/// Apply `op` to the first two arguments coerced to Float, or return 0.0
/// having reported E808.
fn float2(args: &[TypVal], result: &mut TypVal, op: impl FnOnce(c_double, c_double) -> c_double) {
    result.write_float(match (float_arg(args, 0), float_arg(args, 1)) {
        (Some(x), Some(y)) => op(x, y),
        _ => 0.0,
    });
}

// -- The one-argument float builtins ---------------------------------------
//
// The generated table stores one of these in the row's payload and reaches
// it through `float_op_wrapper`, so each needs a name of its own rather than
// a closure. Every body is `f64`'s inherent method, which lowers to the same
// libm call the C made; the two-argument builtins above already read that
// way. The methods make no promise about `errno` or the rounding mode, which
// costs nothing here: vim reads the value and nothing else, and reports a
// domain error as the NaN the value already is.

pub fn acos(x: Float) -> Float {
    x.acos()
}
pub fn asin(x: Float) -> Float {
    x.asin()
}
pub fn atan(x: Float) -> Float {
    x.atan()
}
pub fn ceil(x: Float) -> Float {
    x.ceil()
}
pub fn cos(x: Float) -> Float {
    x.cos()
}
pub fn cosh(x: Float) -> Float {
    x.cosh()
}
pub fn exp(x: Float) -> Float {
    x.exp()
}
pub fn floor(x: Float) -> Float {
    x.floor()
}
/// C's `log` is the natural logarithm, which Rust spells `ln`.
pub fn log(x: Float) -> Float {
    x.ln()
}
pub fn log10(x: Float) -> Float {
    x.log10()
}
/// Half away from zero, matching C's `round` rather than Rust's `round_ties_even`.
pub fn round(x: Float) -> Float {
    x.round()
}
pub fn sin(x: Float) -> Float {
    x.sin()
}
pub fn sinh(x: Float) -> Float {
    x.sinh()
}
pub fn sqrt(x: Float) -> Float {
    x.sqrt()
}
pub fn tan(x: Float) -> Float {
    x.tan()
}
pub fn tanh(x: Float) -> Float {
    x.tanh()
}
pub fn trunc(x: Float) -> Float {
    x.trunc()
}

/// Argument `i` coerced to Float, reporting E808 if it is neither a Float
/// nor a Number.
fn float_arg(args: &[TypVal], i: usize) -> Option<Float> {
    tv_get_float_chk(&args[i]).ok()
}

/// `float2nr({expr})` — truncation towards zero, saturating at the Number
/// range rather than invoking the undefined behaviour C's cast would.
pub fn f_float2nr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let Some(f) = float_arg(args, 0) else {
        return;
    };
    // The epsilon nudge is upstream's: it keeps a value that rounds to the
    // limit on the saturating side of the cast.
    result.write_number(if f <= -(VARNUMBER_MAX as c_double) + c_double::EPSILON {
        -(VARNUMBER_MAX as VarNumber)
    } else if f >= VARNUMBER_MAX as c_double - c_double::EPSILON {
        VARNUMBER_MAX as VarNumber
    } else {
        f as VarNumber
    });
}

/// `isinf({expr})` — 1, -1, or (for anything that is not an infinite Float)
/// the return value left as it was, which is 0.
pub fn f_isinf(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if let Some(f) = args[0].as_float()
        && f.is_infinite()
    {
        result.write_number(if f > 0.0 { 1 } else { -1 });
    }
}

/// `isnan({expr})`.
pub fn f_isnan(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(args[0].as_float().is_some_and(c_double::is_nan) as VarNumber);
}

/// Draw 32 bits of entropy for the generator's seed. Falls back to the
/// clock mixed with the process id when the OS source is unavailable.
fn init_srand() -> u32 {
    let mut bytes = [0u8; 4];
    // SAFETY throughout: a synchronous `uv_random` (null loop and request) fills
    // `bytes`, whose length it is told; the callback is null because the
    // call is synchronous.
    let (out, len) = (bytes.as_mut_ptr().cast::<c_void>(), bytes.len());
    let (loop_, req) = (ptr::null_mut(), ptr::null_mut());
    let rc = unsafe { uv_random(loop_, req, out, len, 0, None) };
    if rc == 0 {
        return u32::from_ne_bytes(bytes);
    }
    (os_hrtime() as u32) ^ (os_get_pid() as u32)
}

/// The seed expander: one step of splitmix32, which turns a single 32-bit
/// seed into the four the generator wants.
fn splitmix32(x: &mut u32) -> u32 {
    *x = x.wrapping_add(0x9e37_79b9);
    let mut z = *x;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

/// One step of xoshiro128**, advancing the four-word state in place.
fn xoshiro128starstar(s: &mut [u32; 4]) -> u32 {
    let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
    let t = s[1] << 9;
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = s[3].rotate_left(11);
    result
}

/// `rand([{expr}])` — the next value of the process-wide generator, or of
/// the four-Number list handed in, which is advanced in place.
pub fn f_rand(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    /// The process-wide generator, seeded from the OS on first use.
    static STATE: GlobalCell<Option<[u32; 4]>> = GlobalCell::new(None);

    let value = if args.is_empty() {
        let mut state = STATE.get().unwrap_or_else(|| {
            let mut x = init_srand();
            [
                splitmix32(&mut x),
                splitmix32(&mut x),
                splitmix32(&mut x),
                splitmix32(&mut x),
            ]
        });
        let draw = xoshiro128starstar(&mut state);
        STATE.set(Some(state));
        draw
    } else {
        let Some(seed) = seed_list(&args[0]) else {
            // Kept on the variadic message call rather than moved to
            // `semsg!`: the argument is arbitrary user bytes, and a Rust
            // format string can only carry UTF-8.
            // SAFETY throughout: `&args[0]` is a live typval, and `tv_get_string`
            // hands back a NUL-terminated buffer that outlives the call.
            let what = arg_string(&mut numbuf, &args[0]);
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
            result.write_number(-1);
            return;
        };
        // SAFETY throughout: `seed_list` proved all four items are live Numbers.
        let mut state = [
            unsafe { (*seed[0]).number_or_zero() } as u32,
            unsafe { (*seed[1]).number_or_zero() } as u32,
            unsafe { (*seed[2]).number_or_zero() } as u32,
            unsafe { (*seed[3]).number_or_zero() } as u32,
        ];
        let draw = xoshiro128starstar(&mut state);
        for (item, word) in seed.iter().zip(state) {
            unsafe { (**item).write_number(word as VarNumber) };
        }
        draw
    };
    result.write_number(value as VarNumber);
}

/// The four state words of a seed list, or `None` if the value is not a
/// four-element List of Numbers.
fn seed_list(tv: &TypVal) -> Option<[*mut TypVal; 4]> {
    if tv.v_type() != VAR_LIST {
        return None;
    }
    // SAFETY: the kind says the value holds a list pointer, which may be
    // null for an empty list literal; `tv_list_len` answers 0 for null.
    let l = tv.list_or_null();
    // SAFETY: `l` is a list pointer or null.
    if unsafe { tv_list_len(l) } != 4 {
        return None;
    }
    let mut out = [ptr::null_mut(); 4];
    for (i, slot) in out.iter_mut().enumerate() {
        // SAFETY: the length check above proves index `i` exists, so
        // `tv_list_find` returns a live item.
        let tv = unsafe { &raw mut (*tv_list_find(l, i as c_int)).li_tv };
        // SAFETY: as above.
        if unsafe { (*tv).v_type() } != VAR_NUMBER {
            return None;
        }
        *slot = tv;
    }
    Some(out)
}

/// `srand([{expr}])` — a four-Number seed list, from the OS or from the
/// Number handed in.
pub fn f_srand(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `result` is the dispatcher's cleared return value.
    list_alloc_ret(result, 4);
    let mut x = if !args.is_empty() {
        let mut error = false;
        let n = arg_number_chk(&args[0], Some(&mut error));
        if error {
            // The list stays empty, as upstream leaves it.
            return;
        }
        n as u32
    } else {
        init_srand()
    };
    for _ in 0..4 {
        // SAFETY: the list was just allocated into `result`.
        unsafe { tv_list_append_number(result.list_or_null(), splitmix32(&mut x) as VarNumber) };
    }
}

/// `range({expr} [, {max} [, {stride}]])`.
pub fn f_range(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut error = false;
    // The errors accumulate into one flag, so every argument is still read —
    // matching upstream, which reports each bad argument in turn.
    let (mut start, end, stride) = {
        let first = arg_number_chk(&args[0], Some(&mut error));
        if args.len() <= 1 {
            (0, first.wrapping_sub(1), 1)
        } else {
            let end = arg_number_chk(&args[1], Some(&mut error));
            let stride = if args.len() > 2 {
                arg_number_chk(&args[2], Some(&mut error))
            } else {
                1
            };
            (first, end, stride)
        }
    };
    if error {
        return;
    }
    if stride == 0 {
        semsg!("E726: Stride is zero");
        return;
    }
    // Wrapping throughout: these are C `VarNumber` expressions, the
    // extremes are reachable from vimscript, and upstream wraps rather than
    // trapping.
    let past_end = if stride > 0 {
        end.wrapping_add(1) < start
    } else {
        end.wrapping_sub(1) > start
    };
    if past_end {
        semsg!("E727: Start past end");
        return;
    }
    // SAFETY throughout: `result` is the dispatcher's cleared return value. The length
    // is upstream's estimate and only preallocates.
    let hint = (end as isize).wrapping_sub(start as isize) / stride as isize;
    let list = list_alloc_ret(result, hint);
    while if stride > 0 {
        start <= end
    } else {
        start >= end
    } {
        // SAFETY: `list` was just allocated into `result`.
        unsafe { tv_list_append_number(list, start) };
        let Some(next) = start.checked_add(stride) else {
            // `i += stride` overflows here in the C and the loop's own test
            // then ends it. Stopping is the same observable outcome without
            // the undefined behaviour.
            break;
        };
        start = next;
    }
}

/// `str2float({string})` — the leading sign and any whitespace around it are
/// consumed here; `string2float` parses what is left.
pub fn f_str2float(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: `&args[0]` is a live typval; `tv_get_string` hands back a
    // NUL-terminated buffer that outlives this call, and `skipwhite` only
    // walks forward over it.
    let mut p = unsafe { skipwhite(arg_string(&mut numbuf, &args[0])) };
    // Only one sign is consumed, and the whitespace skip after it is
    // what makes `"- 1"` parse as -1.
    let negate = unsafe { *p } == b'-' as c_char;
    if unsafe { *p } == b'+' as c_char || unsafe { *p } == b'-' as c_char {
        p = unsafe { skipwhite(p.add(1)) };
    }
    // SAFETY: `p` walks inside the NUL-terminated argument string.
    let (parsed, _) = unsafe { string2float(p) };
    result.write_float(if negate { -parsed } else { parsed });
}
