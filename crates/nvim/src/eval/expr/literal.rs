//! Operands that are written out: numbers, the three string forms,
//! `&option` and `$ENV`.
//!
//! The two quoted forms are each parsed twice — once to find the closing
//! quote and size the result, once to fill it — and the two passes must
//! agree byte for byte. What keeps them in step is a small correction the
//! measuring pass accumulates: `extra` in `eval_string`, `reduce` in
//! `eval_lit_string`. Both count how much longer or shorter the result is
//! than the source text it was read from.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{hex2nr, skipdigits, vim_str2nr};
use crate::eval::typval::{tv_blob_alloc, tv_blob_set_ret, tv_clear};
use crate::eval::vars::{eval_one_expr_in_str, optval_as_tv};
use crate::eval::{
    BS, CAR, ESC, FF, FSK_IN_STRING, FSK_KEYCODE, FSK_SIMPLIFY, NL, STR2NR_ALL, TAB,
    find_option_var_end, get_env_len, kOptValTypeNil,
};
use crate::eval::{Cur, Tv};
use crate::garray::{ga_append, ga_clear, ga_concat, ga_init};
use crate::keycodes::{find_special_key, trans_special};
use crate::mbyte::{mb_copy_char, utf_char2bytes, utfc_ptr2len};
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, iemsg};
use crate::message_fmt::c_str;
use crate::option::{get_option_value, get_tty_option, is_option_hidden, is_tty_option};
use crate::options::{kOptAleph, kOptInvalid};
use crate::os::cshim::{gettext, strncasecmp};
use crate::os::env::{expand_env_save, vim_getenv};
use crate::types::{
    FAIL, NUL, OK, OptIndex, OptVal, OptionSetFlags, VAR_FLOAT, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarLock, blob_T, float_T, garray_T, size_t, typval_T, typval_vval_union, uint8_t,
    varnumber_T,
};
use ::libc::{strlen, strtod, toupper};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// An empty growable array of bytes.
const UNSET_GA: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: null_mut(),
};

/// A walk over a NUL-terminated buffer: the `*mut c_char` a scan steps
/// along, with its byte reads checked once here.
///
/// Named for the two passes of a string literal, which is what it was
/// written for; the `:function` parser walks its argument lists with it too.
///
/// Construction is the one unsafe step, as [`Live<T>`](crate::winlayer::Live)
/// has it; every `byte()`/`at()` after it is ordinary checked code. It is
/// `#[repr(transparent)]` so that `(&raw mut walk).cast()` is the
/// `*mut *const c_char` that `mb_copy_char`, `find_special_key` and
/// `trans_special` take — they advance the walk in place, which is why this
/// cannot be an index.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Walk(*mut c_char);

impl Walk {
    /// # Safety
    /// `p` must point inside a NUL-terminated buffer that stays valid for as
    /// long as the walk is used.
    pub(crate) const unsafe fn new(p: *mut c_char) -> Self {
        Self(p)
    }

    /// The byte `i` past the walk.
    ///
    /// Reading past the terminating NUL would be out of bounds, so a caller
    /// asking for `i > 0` has already seen a non-NUL at every offset below.
    pub(crate) fn at(self, i: usize) -> u8 {
        // SAFETY: the constructor's promise, plus the caller's: the walk has
        // not stepped past the NUL.
        unsafe { *self.0.add(i) as u8 }
    }

    /// The byte under the walk.
    pub(crate) fn byte(self) -> u8 {
        self.at(0)
    }

    /// The byte `i` before it, which the caller has already walked past.
    pub(crate) fn behind(self, i: usize) -> u8 {
        // SAFETY: as [`Walk::at`], backwards over bytes already read.
        unsafe { *self.0.sub(i) as u8 }
    }

    /// Step it `n` bytes on.
    pub(crate) fn step(&mut self, n: usize) {
        self.0 = self.0.wrapping_add(n);
    }

    /// Step it `n` bytes back, over bytes it has already read.
    pub(crate) fn step_back(&mut self, n: usize) {
        self.0 = self.0.wrapping_sub(n);
    }

    /// The `c_char` under the walk, for the octal escape, which reads back
    /// what it wrote and needs the sign the C arithmetic has.
    pub(crate) fn chr(self) -> c_char {
        // SAFETY: as [`Walk::at`].
        unsafe { *self.0 }
    }

    /// Write `b` where the walk stands, without stepping.
    pub(crate) fn set(&mut self, b: c_char) {
        // SAFETY: the constructor's promise -- the destination was sized by
        // the measuring pass, which counted this byte.
        unsafe { *self.0 = b };
    }

    /// Write `b` where it stands and step past it, which is how the second
    /// pass fills the result.
    pub(crate) fn put(&mut self, b: c_char) {
        self.set(b);
        self.step(1);
    }

    /// The pointer back, for the callees that still take one.
    pub(crate) fn raw(self) -> *mut c_char {
        self.0
    }

    /// How many bytes it stands past `start`.
    ///
    /// # Safety
    /// `start` must be in the same allocation.
    pub(crate) unsafe fn since(self, start: *const c_char) -> isize {
        unsafe { self.0.offset_from(start) }
    }
}

/// `&option`, `&l:option`, `&g:option` or `+option`, with the cursor on the
/// `&` or the `+`. Leaves it after the option name.
///
/// A null `rettv` means "only say whether this names an option"; that is
/// `has("+option")`, which is also the only caller `working` is true for.
///
/// # Safety
/// `arg` must point at the cursor into a writable, NUL-terminated
/// expression; `rettv` must be null or valid.
pub(crate) unsafe fn eval_option(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a writable,
    // NUL-terminated expression, and `rettv` is null or valid.
    let working = unsafe { **arg } == b'+' as c_char; // has("+option")
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: OptionSetFlags = OptionSetFlags::NONE;

    // Isolate the option name and find its value.
    let (idxp, flagsp) = (&raw mut opt_idx, &raw mut opt_flags);
    let option_end = unsafe { find_option_var_end(arg, idxp, flagsp) } as *mut c_char;
    if option_end.is_null() {
        if !rettv.is_null() {
            let name = unsafe { *arg };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E112: Option name missing: {name}");
        }
        return FAIL;
    }
    if !evaluate {
        unsafe { *arg = option_end };
        return OK;
    }

    // The name is terminated in place for the lookup and put back
    // afterwards, because the error messages want the whole expression.
    // SAFETY: `option_end` is inside the expression, which is writable.
    let c = unsafe { *option_end };
    unsafe { *option_end = NUL as c_char };

    let opt_name = unsafe { CStr::from_ptr(*arg) };
    let is_tty_opt = is_tty_option(opt_name);
    let ret = if opt_idx == kOptInvalid && !is_tty_opt {
        // Only report it when the result is going to be used.
        if !rettv.is_null() {
            let name = unsafe { *arg };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E113: Unknown option: {name}");
        }
        FAIL
    } else if !rettv.is_null() {
        let value: OptVal = if is_tty_opt {
            get_tty_option(opt_name)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        debug_assert!(value.type_0 != kOptValTypeNil);
        unsafe { *rettv = optval_as_tv(value, true) };
        OK
    } else if working && !is_tty_opt && is_option_hidden(opt_idx) {
        FAIL
    } else {
        OK
    };

    unsafe { *option_end = c };
    unsafe { *arg = option_end };
    ret
}

/// A Number, a Float or a `0z` Blob literal, with the cursor on the first
/// digit. `want_string` suppresses the Float reading, so that `1.2` in a
/// context that wants a string is the Number 1 followed by `.2`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_number(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    want_string: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a
    // NUL-terminated expression and `rettv` is valid when `evaluate`.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let mut p = unsafe { Walk::new(skipdigits(cur.get().add(1))) };

    // A Float is accepted only for the exact `1.2`, `1.2e3` shapes: a
    // digit either side of the dot, and nothing alphabetic or a second
    // dot after what was read.
    let mut get_float = false;
    if !want_string && p.byte() == b'.' && ascii_isdigit(c_int::from(p.at(1))) {
        get_float = true;
        // SAFETY: `skipdigits` stops at the NUL, and the walk is still
        // inside the expression at every step below.
        p = unsafe { Walk::new(skipdigits(p.raw().add(2))) };
        if p.byte() == b'e' || p.byte() == b'E' {
            p.step(1);
            if p.byte() == b'-' || p.byte() == b'+' {
                p.step(1);
            }
            if !ascii_isdigit(c_int::from(p.byte())) {
                get_float = false;
            } else {
                p = unsafe { Walk::new(skipdigits(p.raw().add(1))) };
            }
        }
        let after = p.byte();
        if after.is_ascii_alphabetic() || after == b'.' {
            get_float = false;
        }
    }

    if get_float {
        let mut f: float_T = 0.;
        // SAFETY: the cursor is on the first digit of the literal.
        let used = unsafe { string2float(cur.get(), &raw mut f) };
        cur.bump(used as usize);
        if evaluate {
            rv.v_type = VAR_FLOAT;
            rv.vval.v_float = f;
        }
    } else if cur.byte() == b'0' && matches!(cur.at(1), b'z' | b'Z') {
        // SAFETY: a fresh Blob of this call's own, or none while skipping.
        let blob: *mut blob_T = if evaluate {
            unsafe { tv_blob_alloc() }
        } else {
            null_mut()
        };
        // SAFETY: the `0z` was just read, so the walk starts inside the
        // expression and every step below stops at a non-hex byte.
        let mut bp = unsafe { Walk::new(cur.get().add(2)) };
        while ascii_isxdigit(c_int::from(bp.byte())) {
            if !ascii_isxdigit(c_int::from(bp.at(1))) {
                if !blob.is_null() {
                    // SAFETY: a literal message, and `blob` is this call's
                    // own, unreferenced allocation.
                    let odd = c"E973: Blob literal should have an even number of hex characters";
                    emsg(gettext(odd));
                    unsafe { ga_clear(&raw mut (*blob).bv_ga) };
                    unsafe { xfree(blob.cast()) };
                }
                return FAIL;
            }
            if !blob.is_null() {
                let pair = (hex2nr(c_int::from(bp.byte())) << 4) + hex2nr(c_int::from(bp.at(1)));
                // SAFETY: as above -- `blob` is this call's own.
                unsafe { ga_append(&raw mut (*blob).bv_ga, pair as uint8_t) };
            }
            // A dot may separate byte pairs: `0z00.11.22`.
            if bp.at(2) == b'.' && ascii_isxdigit(c_int::from(bp.at(3))) {
                bp.step(1);
            }
            bp.step(2);
        }
        if !blob.is_null() {
            // SAFETY: `rettv` is valid whenever a Blob was allocated.
            unsafe { tv_blob_set_ret(rettv, blob) };
        }
        cur.set(bp.raw());
    } else {
        let mut len: c_int = 0;
        let mut n: varnumber_T = 0;
        let (text, lenp, np) = (cur.get(), &raw mut len, &raw mut n);
        let all = STR2NR_ALL as c_int;
        // SAFETY: the cursor is on the first digit and the two
        // out-parameters are this frame's locals.
        let (skip_pre, no_len, no_ov) = (null_mut(), null_mut(), null_mut());
        unsafe { vim_str2nr(text, skip_pre, lenp, all, np, no_len, 0, true, no_ov) };
        if len == 0 {
            if evaluate {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let text = unsafe { c_str(text) };
                semsg!("E15: Invalid expression: \"{text}\"");
            }
            return FAIL;
        }
        cur.bump(len as usize);
        if evaluate {
            rv.v_type = VAR_NUMBER;
            rv.vval.v_number = n;
        }
    }
    OK
}

/// A double-quoted string, with the cursor on the quote — or, when
/// `interpolate` is set, on the first character of a `$"..."` piece, which
/// ends at the closing quote or at a single `{`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a
    // NUL-terminated expression and `rettv` is valid when `evaluate`. Both
    // walks below stay inside that expression: the measuring pass stops at
    // the NUL and the filling pass repeats it byte for byte.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let arg_end = unsafe { cur.get().add(strlen(cur.get()) as usize) } as *const c_char;
    let off = if interpolate { 0 } else { 1 };
    // How much longer the result is than the text it is read from. The
    // 1 an interpolated piece starts with is the terminator it writes;
    // a doubled brace gives a byte back. It is `unsigned` upstream and
    // may go negative here for the same reason it wraps there — the
    // sum with the source length is what is used, and stays positive.
    let mut extra: isize = if interpolate { 1 } else { 0 };

    // Find the end of the string, skipping backslashed characters.
    let mut p = unsafe { Walk::new(cur.get().add(off)) };
    while p.byte() != NUL as u8 && p.byte() != b'"' {
        if p.byte() == b'\\' && p.at(1) != NUL as u8 {
            p.step(1);
            if p.byte() == b'<' {
                // A `\<x>` form is at least 4 characters and produces up
                // to 9 (6 for the character, 3 for a modifier): reserve
                // five extra.
                extra += 5;
                let mut modifiers: c_int = 0;
                let mut flags = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                if p.at(1) != b'*' {
                    flags |= FSK_SIMPLIFY as c_int;
                }
                // Skip to the `>` so a `{` inside is not read as the
                // start of an interpolated expression.
                let left = unsafe { arg_end.offset_from(p.raw()) } as size_t;
                let (walk, mods) = ((&raw mut p).cast(), &raw mut modifiers);
                // SAFETY: `walk` is this frame's own walk, which
                // `find_special_key` advances in place.
                let found = unsafe { find_special_key(walk, left, mods, flags, null_mut()) };
                if found != 0 {
                    p = unsafe { Walk::new(p.raw().sub(1)) }; // leave `p` on the `>`
                }
            }
        } else if interpolate && (p.byte() == b'{' || p.byte() == b'}') {
            if p.byte() == b'{' && p.at(1) != b'{' {
                break; // start of an expression
            }
            p.step(1);
            if p.behind(1) == b'}' && p.byte() != b'}' {
                let text = cur.get();
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let text = unsafe { c_str(text) };
                semsg!("E1278: Stray '}}' without a matching '{{': {text}");
                return FAIL;
            }
            extra -= 1; // `{{` becomes `{`, `}}` becomes `}`
        }
        p.step(unsafe { utfc_ptr2len(p.raw()) } as usize);
    }

    if p.byte() != b'"' && !(interpolate && p.byte() == b'{') {
        let text = cur.get();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let text = unsafe { c_str(text) };
        semsg!("E114: Missing quote: {text}");
        return FAIL;
    }
    if !evaluate {
        cur.set(unsafe { p.raw().add(off) });
        return OK;
    }

    // Copy the string into allocated memory, resolving the escapes.
    rv.v_type = VAR_STRING;
    let len = (unsafe { p.since(cur.get()) } + extra) as c_int;
    let buffer = unsafe { xmalloc(len as size_t) } as *mut c_char;
    rv.vval.v_string = buffer;
    let mut end = unsafe { Walk::new(buffer) };

    p = unsafe { Walk::new(cur.get().add(off)) };
    while p.byte() != NUL as u8 && p.byte() != b'"' {
        if p.byte() != b'\\' {
            if interpolate && (p.byte() == b'{' || p.byte() == b'}') {
                if p.byte() == b'{' && p.at(1) != b'{' {
                    break; // start of an expression
                }
                p.step(1); // reduce `{{` to `{` and `}}` to `}`
            }
            // SAFETY: both are this frame's own walks, which
            // `mb_copy_char` advances in place.
            unsafe { mb_copy_char((&raw mut p).cast(), (&raw mut end).cast()) };
            continue;
        }

        p.step(1);
        // Every arm that handles the escape itself leaves `handled` set;
        // the rest — including `\<` that did not name a key — fall
        // through to copying the character after the backslash.
        let mut handled = true;
        match p.byte() {
            b'b' => {
                end.put(BS as c_char);
                p.step(1);
            }
            b'e' => {
                end.put(ESC as c_char);
                p.step(1);
            }
            b'f' => {
                end.put(FF as c_char);
                p.step(1);
            }
            b'n' => {
                end.put(NL as c_char);
                p.step(1);
            }
            b'r' => {
                end.put(CAR as c_char);
                p.step(1);
            }
            b't' => {
                end.put(TAB as c_char);
                p.step(1);
            }
            // hex `\x1`/`\x12`, Unicode `\u0023`/`\U0001f600`. With no
            // hex digit after it the letter itself is copied, by the
            // next pass of the loop rather than here.
            b'X' | b'x' | b'u' | b'U' => {
                if ascii_isxdigit(c_int::from(p.at(1))) {
                    // SAFETY: `toupper` reads no memory.
                    let c = unsafe { toupper(c_int::from(p.byte())) };
                    let mut n = if c == 'X' as c_int {
                        2
                    } else if p.byte() == b'u' {
                        4
                    } else {
                        8
                    };
                    let mut nr: c_int = 0;
                    loop {
                        n -= 1;
                        if n < 0 || !ascii_isxdigit(c_int::from(p.at(1))) {
                            break;
                        }
                        p.step(1);
                        nr = (nr << 4) + hex2nr(c_int::from(p.byte()));
                    }
                    p.step(1);
                    // `\u` stores the character in the current encoding;
                    // `\x` stores the byte.
                    if c != 'X' as c_int {
                        // SAFETY: the measuring pass reserved five bytes for
                        // this escape, which is more than a character takes.
                        let written = unsafe { utf_char2bytes(nr, end.raw()) };
                        end.step(written as usize);
                    } else {
                        end.put(nr as c_char);
                    }
                }
            }
            // octal `\1`, `\12`, `\123`
            b'0'..=b'7' => {
                end.set((c_int::from(p.chr()) - '0' as c_int) as c_char);
                p.step(1);
                if p.byte() >= b'0' && p.byte() <= b'7' {
                    let digit = c_int::from(p.chr()) - '0' as c_int;
                    end.set(((c_int::from(end.chr()) << 3) + digit) as c_char);
                    p.step(1);
                    if p.byte() >= b'0' && p.byte() <= b'7' {
                        let digit = c_int::from(p.chr()) - '0' as c_int;
                        end.set(((c_int::from(end.chr()) << 3) + digit) as c_char);
                        p.step(1);
                    }
                }
                end.step(1);
            }
            // a special key, e.g. `\<C-W>`
            b'<' => {
                let mut flags = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                if p.at(1) != b'*' {
                    flags |= FSK_SIMPLIFY as c_int;
                }
                let left = unsafe { arg_end.offset_from(p.raw()) } as size_t;
                let (walk, out) = ((&raw mut p).cast(), end.raw());
                // SAFETY: `walk` is this frame's own walk, which
                // `trans_special` advances in place, and `out` has the five
                // bytes the measuring pass reserved.
                let written = unsafe { trans_special(walk, left, out, flags, false, null_mut()) };
                if written != 0 {
                    end.step(written as usize);
                    if end.raw() >= buffer.wrapping_offset(len as isize) {
                        iemsg(c"eval_string() used more space than allocated");
                    }
                } else {
                    handled = false;
                }
            }
            _ => handled = false,
        }
        if !handled {
            // SAFETY: as the copy above.
            unsafe { mb_copy_char((&raw mut p).cast(), (&raw mut end).cast()) };
        }
    }

    end.set(NUL as c_char);
    if p.byte() == b'"' && !interpolate {
        p.step(1);
    }
    cur.set(p.raw());
    OK
}

/// A single-quoted string, in which the only escape is a doubled quote —
/// or, when `interpolate` is set, a `$'...'` piece, which also reduces a
/// doubled brace and stops at a single `{`.
///
/// # Safety
/// As `eval_string`.
pub(crate) unsafe fn eval_lit_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a
    // NUL-terminated expression and `rettv` is valid when `evaluate`. Both
    // walks below stay inside that expression: the measuring pass stops at
    // the NUL and the filling pass repeats it byte for byte.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let off = if interpolate { 0 } else { 1 };
    // How much *shorter* the result is than the text: one byte per
    // doubled quote or brace, less the terminator an interpolated piece
    // writes. The sign is the opposite of `eval_string`'s `extra`.
    let mut reduce: c_int = if interpolate { -1 } else { 0 };

    // Find the end of the string, skipping `''`.
    let mut p = unsafe { Walk::new(cur.get().add(off)) };
    while p.byte() != NUL as u8 {
        if p.byte() == b'\'' {
            if p.at(1) != b'\'' {
                break;
            }
            reduce += 1;
            p.step(1);
        } else if interpolate {
            if p.byte() == b'{' {
                if p.at(1) != b'{' {
                    break; // start of an expression
                }
                p.step(1);
                reduce += 1;
            } else if p.byte() == b'}' {
                p.step(1);
                if p.byte() != b'}' {
                    let text = cur.get();
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let text = unsafe { c_str(text) };
                    semsg!("E1278: Stray '}}' without a matching '{{': {text}");
                    return FAIL;
                }
                reduce += 1;
            }
        }
        p.step(unsafe { utfc_ptr2len(p.raw()) } as usize);
    }

    if p.byte() != b'\'' && !(interpolate && p.byte() == b'{') {
        let text = cur.get();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let text = unsafe { c_str(text) };
        semsg!("E115: Missing quote: {text}");
        return FAIL;
    }
    if !evaluate {
        cur.set(unsafe { p.raw().add(off) });
        return OK;
    }

    let size = (unsafe { p.since(cur.get()) } - reduce as isize) as size_t;
    let buffer = unsafe { xmalloc(size) } as *mut c_char;
    rv.v_type = VAR_STRING;
    rv.vval.v_string = buffer;
    let mut str = unsafe { Walk::new(buffer) };
    p = unsafe { Walk::new(cur.get().add(off)) };
    while p.byte() != NUL as u8 {
        if p.byte() == b'\'' {
            if p.at(1) != b'\'' {
                break;
            }
            p.step(1);
        } else if interpolate && (p.byte() == b'{' || p.byte() == b'}') {
            if p.byte() == b'{' && p.at(1) != b'{' {
                break; // start of an expression
            }
            p.step(1);
        }
        // SAFETY: both are this frame's own walks, which `mb_copy_char`
        // advances in place.
        unsafe { mb_copy_char((&raw mut p).cast(), (&raw mut str).cast()) };
    }
    str.set(NUL as c_char);
    cur.set(unsafe { p.raw().add(off) });
    OK
}

/// `$"..."` or `$'...'`, with the cursor on the `$`: alternating literal
/// pieces and `{expr}` substitutions, joined into one String.
///
/// Answers `OK` even for a piece that failed — upstream's; `rettv` then
/// holds whatever was assembled before the error, which may be null.
///
/// # Safety
/// As `eval_string`.
pub(crate) unsafe fn eval_interp_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a
    // NUL-terminated expression and `rettv` is valid when `evaluate`. `ga`
    // is this frame's own and is initialised before anything appends to it.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let mut ret = OK;
    let mut ga = UNSET_GA;
    unsafe { ga_init(&raw mut ga, 1, 80) };

    // `*arg` is on the `$`; move it to the first string character.
    cur.bump(1);
    let quote = cur.byte();
    cur.bump(1);

    loop {
        // The piece up to the matching quote or to a single `{`; `arg`
        // is left on whichever it was.
        let mut tv = UNSET_TV;
        let two = &raw mut tv;
        ret = if quote == b'"' {
            unsafe { eval_string(arg, two, evaluate, true) }
        } else {
            unsafe { eval_lit_string(arg, two, evaluate, true) }
        };
        if ret == FAIL {
            break;
        }
        if evaluate {
            // SAFETY: the piece just parsed is a String typval.
            unsafe { ga_concat(&raw mut ga, tv.vval.v_string) };
            unsafe { tv_clear(two) };
        }
        if cur.byte() != b'{' {
            // Found the terminating quote.
            cur.bump(1);
            break;
        }
        // SAFETY: the cursor is on the `{` of a substitution.
        let p = unsafe { eval_one_expr_in_str(cur.get(), &raw mut ga, evaluate) };
        if p.is_null() {
            ret = FAIL;
            break;
        }
        cur.set(p);
    }

    rv.v_type = VAR_STRING;
    if ret != FAIL && evaluate {
        unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
    }
    rv.vval.v_string = ga.ga_data as *mut c_char;
    OK
}

/// Read a Float out of `text`, answering how many bytes it consumed. The
/// three named values are recognised ahead of `strtod`, which does not know
/// them in every locale.
///
/// # Safety
/// `text` must be NUL-terminated and `ret_value` valid.
pub(crate) unsafe fn string2float(text: *const c_char, ret_value: *mut float_T) -> size_t {
    for (name, len, value) in [
        (c"inf", 3, f64::INFINITY),
        (c"-inf", 4, f64::NEG_INFINITY),
        (c"nan", 3, f64::NAN),
    ] {
        let (lhs, rhs) = (text as *mut c_char, name.as_ptr() as *mut c_char);
        // SAFETY: the caller's promise -- `text` is NUL-terminated, `name`
        // is a literal, and `ret_value` is valid.
        if unsafe { strncasecmp(lhs, rhs, len as size_t) } == 0 {
            unsafe { *ret_value = value as float_T };
            return len as size_t;
        }
    }
    let mut s: *mut c_char = null_mut();
    // SAFETY: as above; `strtod` leaves `s` inside `text`.
    unsafe { *ret_value = strtod(text, &raw mut s) as float_T };
    unsafe { s.offset_from(text) as size_t }
}

/// `$NAME`, with the cursor on the `$`.
///
/// # Safety
/// `arg` must point at the cursor into a writable, NUL-terminated
/// expression; `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_env_var(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into a writable,
    // NUL-terminated expression and `rettv` is valid when `evaluate`.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    cur.bump(1);
    let name = cur.get();
    let len = unsafe { get_env_len(cur.raw().cast()) };
    if !evaluate {
        return OK;
    }
    if len == 0 {
        return FAIL;
    }

    // The name is terminated in place across the two lookups.
    // SAFETY: `name` is `len` bytes inside the expression, which is
    // writable, and the byte after them is the one being blanked.
    let end = name.wrapping_offset(len as isize);
    let cc = unsafe { *end };
    unsafe { *end = NUL as c_char };
    let mut string = unsafe { vim_getenv(name) };
    if string.is_null() || unsafe { *string } as c_int == NUL {
        unsafe { xfree(string as *mut c_void) };
        // Not in the environment: let `expand_env` have it, which knows
        // the names nvim answers itself. A result that still starts with
        // `$` is the name coming back unexpanded.
        string = unsafe { expand_env_save(name.sub(1)) };
        if !string.is_null() && unsafe { *string } == b'$' as c_char {
            unsafe { xfree(string as *mut c_void) };
            string = null_mut();
        }
    }
    unsafe { *end = cc };

    rv.v_type = VAR_STRING;
    rv.vval.v_string = string;
    rv.v_lock = VarLock::Unlocked;
    OK
}
