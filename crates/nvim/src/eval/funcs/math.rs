//! Numbers: arithmetic, the bitwise operators and the random-number
//! generator.
#![deny(unsafe_op_in_unsafe_fn)]

use super::VARNUMBER_MAX;
use super::args::{Args, frame};
use super::uv_random;
use super::wrappers::tv_get_float_chk;
use crate::charset::skipwhite;
use crate::eval::string2float;
use crate::eval::typval::{
    tv_get_number_chk, tv_get_string, tv_list_alloc_ret, tv_list_append_number, tv_list_find,
    tv_list_len,
};
use crate::global_cell::GlobalCell;
use crate::main::e_invarg2;
use crate::os::env::os_get_pid;
use crate::os::libc::gettext;
use crate::os::time::os_hrtime;
use crate::semsg;
use crate::semsg_c;
use crate::types::{EvalFuncData, VAR_FLOAT, VAR_LIST, VAR_NUMBER, float_T, typval_T, varnumber_T};
use core::ffi::{c_char, c_double, c_int, c_void};
use core::ptr;

/// `abs({expr})` — magnitude, as a Float for a Float and as a Number
/// otherwise. A value that is not coercible to a number reports through
/// `tv_get_number_chk` and yields -1, as upstream does.
pub unsafe extern "C" fn f_abs(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    if args.ty(0) == VAR_FLOAT {
        rettv.v_type = VAR_FLOAT;
        // SAFETY: the tag says the union holds a float. This is what
        // `float_op_wrapper` does for the `fabs` row.
        rettv.vval.v_float = unsafe { args.get(0).vval.v_float }.abs();
        return;
    }
    let mut error = false;
    // SAFETY: `args.ptr(0)` is a live typval; the callee reports through
    // `error` rather than returning a failure.
    let n = unsafe { tv_get_number_chk(args.ptr(0), &raw mut error) };
    rettv.vval.v_number = if error {
        -1
    } else if n > 0 {
        n
    } else {
        // Not `-n`: `wrapping_neg` keeps the C's two's-complement answer for
        // the one value whose negation does not fit.
        n.wrapping_neg()
    };
}

/// The bitwise operators. Each coerces both arguments with a null error
/// pointer, so a non-coercible argument reports its own message and
/// contributes zero.
pub unsafe extern "C" fn f_and(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = number(args, 0) & number(args, 1);
}

pub unsafe extern "C" fn f_or(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = number(args, 0) | number(args, 1);
}

pub unsafe extern "C" fn f_xor(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = number(args, 0) ^ number(args, 1);
}

pub unsafe extern "C" fn f_invert(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = !number(args, 0);
}

/// Argument `i` as a Number, reporting its own error and reading as 0 when
/// it cannot be coerced. The bitwise builtins' shared coercion.
fn number(args: Args<'_>, i: usize) -> varnumber_T {
    // SAFETY: `args.ptr(i)` is a live typval; a null error pointer is the
    // documented "report and return 0" mode.
    unsafe { tv_get_number_chk(args.ptr(i), ptr::null_mut()) }
}

/// The two-argument float builtins. Both arguments are read left to right
/// and the second is only read once the first succeeded, so a pair of bad
/// arguments reports E808 once.
pub unsafe extern "C" fn f_atan2(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    float2(args, rettv, |x, y| x.atan2(y));
}

pub unsafe extern "C" fn f_fmod(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // Rust's `%` on floats is C's `fmod`.
    float2(args, rettv, |x, y| x % y);
}

pub unsafe extern "C" fn f_pow(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    float2(args, rettv, c_double::powf);
}

/// Apply `op` to the first two arguments coerced to Float, or return 0.0
/// having reported E808.
fn float2(args: Args<'_>, rettv: &mut typval_T, op: impl FnOnce(c_double, c_double) -> c_double) {
    rettv.v_type = VAR_FLOAT;
    rettv.vval.v_float = match (float_arg(args, 0), float_arg(args, 1)) {
        (Some(x), Some(y)) => op(x, y),
        _ => 0.0,
    };
}

/// Argument `i` coerced to Float, reporting E808 if it is neither a Float
/// nor a Number.
fn float_arg(args: Args<'_>, i: usize) -> Option<float_T> {
    let mut f: float_T = 0.0;
    // SAFETY: `args.ptr(i)` is a live typval and `f` is a live local.
    unsafe { tv_get_float_chk(args.ptr(i), &raw mut f) }.then_some(f)
}

/// `float2nr({expr})` — truncation towards zero, saturating at the Number
/// range rather than invoking the undefined behaviour C's cast would.
pub unsafe extern "C" fn f_float2nr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let Some(f) = float_arg(args, 0) else {
        return;
    };
    // The epsilon nudge is upstream's: it keeps a value that rounds to the
    // limit on the saturating side of the cast.
    rettv.vval.v_number = if f <= -(VARNUMBER_MAX as c_double) + c_double::EPSILON {
        -(VARNUMBER_MAX as varnumber_T)
    } else if f >= VARNUMBER_MAX as c_double - c_double::EPSILON {
        VARNUMBER_MAX as varnumber_T
    } else {
        f as varnumber_T
    };
}

/// `isinf({expr})` — 1, -1, or (for anything that is not an infinite Float)
/// the return value left as it was, which is 0.
pub unsafe extern "C" fn f_isinf(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    if let Some(f) = as_float(args, 0) {
        if f.is_infinite() {
            rettv.vval.v_number = if f > 0.0 { 1 } else { -1 };
        }
    }
}

/// `isnan({expr})`.
pub unsafe extern "C" fn f_isnan(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = as_float(args, 0).is_some_and(c_double::is_nan) as varnumber_T;
}

/// Argument `i` if it is a Float. Unlike [`float_arg`] this neither coerces
/// a Number nor reports an error — `isinf`/`isnan` answer for every type.
fn as_float(args: Args<'_>, i: usize) -> Option<float_T> {
    let tv = args.get(i);
    // SAFETY: the tag says the union holds a float.
    (tv.v_type == VAR_FLOAT).then(|| unsafe { tv.vval.v_float })
}

/// Draw 32 bits of entropy for the generator's seed. Falls back to the
/// clock mixed with the process id when the OS source is unavailable.
fn init_srand() -> u32 {
    let mut bytes = [0u8; 4];
    // SAFETY: a synchronous `uv_random` (null loop and request) fills
    // `bytes`, whose length it is told; the callback is null because the
    // call is synchronous.
    let rc = unsafe {
        uv_random(
            ptr::null_mut(),
            ptr::null_mut(),
            bytes.as_mut_ptr().cast::<c_void>(),
            bytes.len(),
            0,
            None,
        )
    };
    if rc == 0 {
        return u32::from_ne_bytes(bytes);
    }
    // SAFETY: `os_get_pid` reads the process id; no arguments, no state.
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
pub unsafe extern "C" fn f_rand(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    /// The process-wide generator, seeded from the OS on first use.
    static STATE: GlobalCell<Option<[u32; 4]>> = GlobalCell::new(None);

    let (args, rettv) = frame!(argvars, rettv);
    let result = if !args.has(0) {
        let mut state = STATE.get().unwrap_or_else(|| {
            let mut x = init_srand();
            [
                splitmix32(&mut x),
                splitmix32(&mut x),
                splitmix32(&mut x),
                splitmix32(&mut x),
            ]
        });
        let result = xoshiro128starstar(&mut state);
        STATE.set(Some(state));
        result
    } else {
        let Some(seed) = seed_list(args.get(0)) else {
            // Kept on the variadic message call rather than moved to
            // `semsg!`: the argument is arbitrary user bytes, and a Rust
            // format string can only carry UTF-8.
            // SAFETY: `args.ptr(0)` is a live typval, and `tv_get_string`
            // hands back a NUL-terminated buffer that outlives the call.
            unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), tv_get_string(args.ptr(0)),) };
            rettv.v_type = VAR_NUMBER;
            rettv.vval.v_number = -1;
            return;
        };
        // SAFETY: `seed_list` proved all four items are live Numbers.
        unsafe {
            let mut state = [
                (*seed[0]).vval.v_number as u32,
                (*seed[1]).vval.v_number as u32,
                (*seed[2]).vval.v_number as u32,
                (*seed[3]).vval.v_number as u32,
            ];
            let result = xoshiro128starstar(&mut state);
            for (item, word) in seed.iter().zip(state) {
                (**item).vval.v_number = word as varnumber_T;
            }
            result
        }
    };
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = result as varnumber_T;
}

/// The four state words of a seed list, or `None` if the value is not a
/// four-element List of Numbers.
fn seed_list(tv: &typval_T) -> Option<[*mut typval_T; 4]> {
    if tv.v_type != VAR_LIST {
        return None;
    }
    // SAFETY: the tag says the union holds a list pointer, which may be
    // null for an empty list literal; `tv_list_len` answers 0 for null.
    let l = unsafe { tv.vval.v_list };
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
        if unsafe { (*tv).v_type } != VAR_NUMBER {
            return None;
        }
        *slot = tv;
    }
    Some(out)
}

/// `srand([{expr}])` — a four-Number seed list, from the OS or from the
/// Number handed in.
pub unsafe extern "C" fn f_srand(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `rettv` is the dispatcher's cleared return value.
    unsafe { tv_list_alloc_ret(rettv, 4) };
    let mut x = if args.has(0) {
        let mut error = false;
        // SAFETY: `args.ptr(0)` is a live typval.
        let n = unsafe { tv_get_number_chk(args.ptr(0), &raw mut error) };
        if error {
            // The list stays empty, as upstream leaves it.
            return;
        }
        n as u32
    } else {
        init_srand()
    };
    for _ in 0..4 {
        // SAFETY: the list was just allocated into `rettv`.
        unsafe { tv_list_append_number(rettv.vval.v_list, splitmix32(&mut x) as varnumber_T) };
    }
}

/// `range({expr} [, {max} [, {stride}]])`.
pub unsafe extern "C" fn f_range(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut error = false;
    // SAFETY: each `args.ptr(i)` is a live typval. The errors accumulate
    // into one flag, so every argument is still read — matching upstream,
    // which reports each bad argument in turn.
    let (mut start, end, stride) = unsafe {
        let first = tv_get_number_chk(args.ptr(0), &raw mut error);
        if !args.has(1) {
            (0, first.wrapping_sub(1), 1)
        } else {
            let end = tv_get_number_chk(args.ptr(1), &raw mut error);
            let stride = if args.has(2) {
                tv_get_number_chk(args.ptr(2), &raw mut error)
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
    // Wrapping throughout: these are C `varnumber_T` expressions, the
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
    // SAFETY: `rettv` is the dispatcher's cleared return value. The length
    // is upstream's estimate and only preallocates.
    let list = unsafe {
        tv_list_alloc_ret(
            rettv,
            (end as isize).wrapping_sub(start as isize) / stride as isize,
        )
    };
    while if stride > 0 {
        start <= end
    } else {
        start >= end
    } {
        // SAFETY: `list` was just allocated into `rettv`.
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
pub unsafe extern "C" fn f_str2float(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` is a live typval; `tv_get_string` hands back a
    // NUL-terminated buffer that outlives this call, and `skipwhite` only
    // walks forward over it.
    unsafe {
        let mut p = skipwhite(tv_get_string(args.ptr(0)));
        // Only one sign is consumed, and the whitespace skip after it is
        // what makes `"- 1"` parse as -1.
        let negate = *p == b'-' as c_char;
        if *p == b'+' as c_char || *p == b'-' as c_char {
            p = skipwhite(p.add(1));
        }
        string2float(p, &raw mut rettv.vval.v_float);
        if negate {
            rettv.vval.v_float = -rettv.vval.v_float;
        }
    }
    rettv.v_type = VAR_FLOAT;
}
