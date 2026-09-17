//! Vetting a string option's value, and the sweeps that re-vet every
//! buffer's copy of one.
//!
//! Two things live here that the rest of the tree leans on.
//!
//! The **empty string option**. A string option's variable is never null:
//! out of memory, `xstrdup` may hand back null, and everything downstream
//! dereferences the variable, so a null one is replaced by the shared
//! [`empty_option`]. That one allocation is aliased by every option holding
//! "", which is why freeing a string option has to go through
//! [`free_string_option`] rather than `xfree`, and why "does this option own
//! its value" is [`is_empty_option`] rather than a pointer comparison
//! spelled out at each of the three dozen places that ask.
//!
//! The **rejection message**. A check that has to name what it disliked
//! formats an owned [`OptError`]; upstream formatted into a caller-supplied
//! `errbuf` and let the caller decline one by passing null, which is how a
//! set could fail and report nothing.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::charset::transchar;
use crate::global_cell::GlobalCell;
use crate::guard::secure;
use crate::indent_c::parse_cino;
use crate::memory::{XString, xfree};
use crate::option::{kOptFlagNDname, kOptFlagNFname, valid_name};
use crate::options::{
    kOptBackupcopy, kOptBelloff, kOptCasemap, kOptClipboard, kOptCompleteopt, kOptDisplay,
    kOptFoldopen, kOptJumpoptions, kOptRedrawdebug, kOptSessionoptions, kOptSwitchbuf,
    kOptTabclose, kOptTagcase, kOptTermpastefilter, kOptViewoptions, kOptVirtualedit,
    kOptWildoptions, opt_scl_values,
};
use crate::os::cshim::gettext;
use crate::strings::vim_snprintf;
use crate::types::{Buffer, Failed, NUL, OptError, StlOpt, size_t, uint32_t};

use super::{
    SCL_NO, check_str_opt, e_illegal_character_after_chr, e_unbalanced_groups,
    e_unclosed_expression_sequence, opt_strings_ok,
};
use crate::decoration::SCL_NUM;

use crate::winlayer::Buf;
/// The options whose bitmask is derived from a value the startup sequence
/// may have installed without going through `:set`.
pub fn didset_string_options() {
    for idx in [
        kOptCasemap,
        kOptBackupcopy,
        kOptBelloff,
        kOptCompleteopt,
        kOptSessionoptions,
        kOptViewoptions,
        kOptFoldopen,
        kOptDisplay,
        kOptJumpoptions,
        kOptRedrawdebug,
        kOptTagcase,
        kOptTermpastefilter,
        kOptVirtualedit,
        kOptSwitchbuf,
        kOptTabclose,
        kOptWildoptions,
        kOptClipboard,
    ] {
        // SAFETY: a null `varp` asks for the option's own global variable.
        let _ = unsafe { check_str_opt(idx, None) };
    }
}

/// "E539: Illegal character <x>", naming the byte it did not like.
pub fn illegal_char(c: c_int) -> OptError {
    let fmt = gettext(c"E539: Illegal character <%s>");
    // SAFETY: `message` is the buffer the formatter is told the size of,
    // and `transchar` answers a C string.
    OptError::Owned(XString::filled(OptError::ROOM, |buf| unsafe {
        vim_snprintf(
            buf,
            OptError::ROOM as size_t,
            fmt.as_ptr(),
            transchar(c).as_ptr(),
        );
    }))
}

/// "E535: Illegal character after <%c>", for the options that spell a field
/// as a character followed by a value.
pub(crate) fn illegal_char_after_chr(c: c_int) -> OptError {
    let fmt = gettext(e_illegal_character_after_chr);
    // SAFETY: `message` is the buffer the formatter is told the size of,
    // and the format takes one `int`.
    OptError::Owned(XString::filled(OptError::ROOM, |buf| unsafe {
        vim_snprintf(buf, OptError::ROOM as size_t, fmt.as_ptr(), c);
    }))
}

/// Give a freshly allocated buffer's string options their initial value.
///
/// A buffer is allocated **zeroed**, and all-zero bytes are not a valid
/// `Option<XString>`: the niche the `Option` uses is the vector's capacity,
/// so a zeroed field reads as `Some` over a null pointer. Every string
/// option is therefore written -- with `write`, which drops nothing --
/// before anything can read or drop it. That includes the five in the
/// buffer's own syntax block, which is part of the buffer;
/// `crate::syntax::init_synblock` writes the same five for the standalone
/// block `:ownsyntax` allocates, and writing `None` twice costs nothing.
///
/// This is where upstream's "replace each null option value with the shared
/// empty string" sweep went: `None` *is* that shared empty string, so the
/// value a fresh buffer starts with is written once here instead of patched
/// up on every `buf_copy_options`.
///
/// # Safety
///
/// `at` must point at a freshly allocated buffer whose string options have
/// not been read, written or dropped.
pub unsafe fn init_buf_string_options(at: *mut Buffer) {
    // SAFETY: the caller's promise -- each address is one of the buffer's
    // own fields, and `write` does not drop what was there.
    unsafe {
        (&raw mut (*at).b_p_bkc).write(None);
        (&raw mut (*at).b_p_bh).write(None);
        (&raw mut (*at).b_p_bt).write(None);
        (&raw mut (*at).b_p_cino).write(None);
        (&raw mut (*at).b_p_cink).write(None);
        (&raw mut (*at).b_p_cinw).write(None);
        (&raw mut (*at).b_p_cinsd).write(None);
        (&raw mut (*at).b_p_com).write(None);
        (&raw mut (*at).b_p_cms).write(None);
        (&raw mut (*at).b_p_cot).write(None);
        (&raw mut (*at).b_p_cpt).write(None);
        (&raw mut (*at).b_p_cfu).write(None);
        (&raw mut (*at).b_p_ofu).write(None);
        (&raw mut (*at).b_p_tfu).write(None);
        (&raw mut (*at).b_p_ffu).write(None);
        (&raw mut (*at).b_p_fenc).write(None);
        (&raw mut (*at).b_p_ff).write(None);
        (&raw mut (*at).b_p_ft).write(None);
        (&raw mut (*at).b_p_fo).write(None);
        (&raw mut (*at).b_p_flp).write(None);
        (&raw mut (*at).b_p_isk).write(None);
        (&raw mut (*at).b_p_def).write(None);
        (&raw mut (*at).b_p_inc).write(None);
        (&raw mut (*at).b_p_inex).write(None);
        (&raw mut (*at).b_p_inde).write(None);
        (&raw mut (*at).b_p_indk).write(None);
        (&raw mut (*at).b_p_fp).write(None);
        (&raw mut (*at).b_p_fex).write(None);
        (&raw mut (*at).b_p_kp).write(None);
        (&raw mut (*at).b_p_lop).write(None);
        (&raw mut (*at).b_p_menc).write(None);
        (&raw mut (*at).b_p_mps).write(None);
        (&raw mut (*at).b_p_nf).write(None);
        (&raw mut (*at).b_p_qe).write(None);
        (&raw mut (*at).b_p_sua).write(None);
        (&raw mut (*at).b_p_syn).write(None);
        (&raw mut (*at).b_p_vsts).write(None);
        (&raw mut (*at).b_p_vsts_nopaste).write(None);
        (&raw mut (*at).b_p_vts).write(None);
        (&raw mut (*at).b_p_keymap).write(None);
        (&raw mut (*at).b_p_gefm).write(None);
        (&raw mut (*at).b_p_gp).write(None);
        (&raw mut (*at).b_p_mp).write(None);
        (&raw mut (*at).b_p_efm).write(None);
        (&raw mut (*at).b_p_ep).write(None);
        (&raw mut (*at).b_p_path).write(None);
        (&raw mut (*at).b_p_tags).write(None);
        (&raw mut (*at).b_p_tc).write(None);
        (&raw mut (*at).b_p_dict).write(None);
        (&raw mut (*at).b_p_dia).write(None);
        (&raw mut (*at).b_p_tsr).write(None);
        (&raw mut (*at).b_p_tsrfu).write(None);
        (&raw mut (*at).b_p_lw).write(None);
        (&raw mut (*at).b_s.b_p_spc).write(None);
        (&raw mut (*at).b_s.b_p_spf).write(None);
        (&raw mut (*at).b_s.b_p_spl).write(None);
        (&raw mut (*at).b_s.b_p_spo).write(None);
        (&raw mut (*at).b_s.b_syn_isk).write(None);
    }
}

/// Rebuild what a buffer derives from its string options.
///
/// Upstream spelled this "replace every null option value with the shared
/// empty string", and then re-derived the 'cinoptions' cache from the value
/// it had just made dereferenceable. A buffer's string options own their
/// bytes now and `None` is the shared empty string, so nothing is left to
/// patch up and the cache is the whole of it.
pub fn check_buf_options(buffer: Buf) {
    // SAFETY: the caller's buffer, whose 'cinoptions' this re-reads.
    unsafe { parse_cino(buffer) };
}

/// The value every string option with nothing of its own points at.
///
/// Upstream declares it `char empty_string_option[]` and lets thirty-odd
/// places compare an option variable against that address by hand. Here the
/// static is private and [`empty_option`]/[`is_empty_option`] are the only
/// way to reach it, so the sentinel is a question callers ask by name.
///
/// It keeps a cell rather than becoming a `ConstTable`: upstream's array is
/// writable, and putting one byte in `.rodata` would turn a write nobody has
/// proved impossible into a fault. Nothing dereferences the cell — both
/// accessors only want the address — so `as_raw` is the whole of its use.
static EMPTY_OPTION: GlobalCell<[c_char; 1]> = GlobalCell::new([0]);

/// The shared value to give a string option that has none of its own.
pub(crate) const fn empty_option() -> *mut c_char {
    EMPTY_OPTION.as_raw().cast::<c_char>()
}

/// What a string option's *local* copy — a field of a window, a buffer or a
/// syntax block — shows the readers that still want a `char *` or a `&CStr`.
///
/// The global values have [`crate::options::vars::StrOpt`]; this is the
/// same four questions for a local one, whose storage is the field itself.
/// `None` is upstream's shared empty string: the option owns nothing and
/// holds no value of its own, so a global-local option falls back and a
/// `:setlocal` reads as unset.
pub(crate) trait LocalOptStr {
    /// The value as the `char *` the option protocol and the C callees
    /// still speak, which is the shared empty string when the field owns
    /// nothing.
    ///
    /// **The pointer is the field's own buffer**, and lives until the field
    /// is written — not until the end of the caller's statement. A reader
    /// that keeps it across anything that can set an option is reading
    /// freed bytes; take [`value`](Self::value) or a copy instead.
    fn value_ptr(&self) -> *mut c_char;

    /// The value's bytes, without the terminator.
    fn bytes(&self) -> &[u8];

    /// The value's first byte, which is 0 for a field that owns nothing --
    /// upstream's `*p` on a variable that is never null.
    fn first_byte(&self) -> u8;

    /// Whether the field owns no string of its own — upstream's
    /// `is_empty_option` on a local copy. **Not** "the value is empty": a
    /// local option explicitly set to `""` owns an empty string.
    fn is_unset(&self) -> bool;
}

impl LocalOptStr for Option<XString> {
    fn value_ptr(&self) -> *mut c_char {
        self.as_ref()
            .map_or_else(empty_option, |value| value.as_ptr().cast_mut())
    }

    fn bytes(&self) -> &[u8] {
        self.as_deref().unwrap_or_default()
    }

    fn first_byte(&self) -> u8 {
        self.bytes().first().copied().unwrap_or(0)
    }

    fn is_unset(&self) -> bool {
        self.is_none()
    }
}

/// Whether a string option's value is that shared one, which answers two
/// questions at once: the option owns no allocation, so it must not be
/// freed, and it holds no value of its own, so a global-local one falls
/// back and a `:setlocal` reads as unset.
pub(crate) fn is_empty_option(value: *const c_char) -> bool {
    ptr::eq(value, empty_option().cast_const())
}

/// Free a string option's value, unless it is the shared empty string every
/// option holding "" points at.
///
/// # Safety
/// `p` is null, the shared empty string, or an allocation this owns.
pub unsafe fn free_string_option(p: *mut c_char) {
    if !is_empty_option(p) {
        // SAFETY: as documented above.
        unsafe { xfree(p.cast::<c_void>()) };
    }
}

/// Is `val` a name 'filetype', 'syntax' or 'keymap' will accept?
pub(crate) fn valid_filetype(val: &CStr) -> bool {
    valid_name(val, b".-_")
}

/// Parse 'signcolumn' and, given a window, store the width range it asks
/// for.
///
/// `scl` overrides the window's own value; a null `window` only validates. The
/// two halves are separate grammars: everything the generated table lists
/// ("no", "yes", "yes:1".."yes:9", "auto", "auto:1".."auto:9", "number"),
/// and then the `auto:<min>-<max>` range, which the table cannot enumerate
/// and which is parsed by hand.
///
/// # Safety
/// `scl` is null or a C string; `window` is null or a live window.
pub unsafe fn check_signcolumn(scl: *mut c_char, window: Option<Win>) -> Result<(), Failed> {
    let val = match (scl.is_null(), window) {
        (false, _) => scl.cast_const(),
        (true, Some(w)) => w.w_onebuf_opt.wo_scl.value_ptr(),
        (true, None) => empty_option(),
    };
    // SAFETY: an option value is a C string.
    let val = unsafe { CStr::from_ptr(val) }.to_bytes();
    if val.is_empty() {
        return Err(Failed);
    }

    // SAFETY: the table's own array, and no mask is wanted.
    let listed = unsafe { opt_strings_ok(val.as_ptr().cast::<c_char>(), &opt_scl_values, false) };

    let (min, max) = if listed {
        let Some(w) = window else {
            return Ok(());
        };
        // 'number' only wins when the window is actually showing numbers.
        let numbered = w.w_onebuf_opt.wo_nu != 0 || w.w_onebuf_opt.wo_rnu != 0;
        match val {
            [b'n', b'o', ..] => (SCL_NO, SCL_NO),
            [b'n', b'u', ..] if numbered => (SCL_NUM, SCL_NUM),
            [b'y', b'e', b's', b':', n, ..] => (digit(*n), digit(*n)),
            [b'y', ..] => (1, 1),
            [b'a', b'u', b't', b'o', b':', n, ..] => (0, digit(*n)),
            _ => (0, 1),
        }
    } else {
        // "auto:<min>-<max>", the one spelling the table cannot list.
        let [b'a', b'u', b't', b'o', b':', min, b'-', max] = val else {
            return Err(Failed);
        };
        if !ascii_isdigit(c_int::from(*min)) || !ascii_isdigit(c_int::from(*max)) {
            return Err(Failed);
        }
        let (min, max) = (digit(*min), digit(*max));
        if min < 1 || max < 2 || min > 8 || min >= max {
            return Err(Failed);
        }
        if window.is_none() {
            return Ok(());
        }
        (min, max)
    };

    // The two `return`s above rule out an absent window.
    let mut window = window.expect("a window to set 'signcolumn' on");
    window.w_minscwidth = min;
    window.w_maxscwidth = max;
    // Keep the width the window is currently drawing inside the new
    // range, without widening it on its own.
    let held = if min <= 0 {
        0
    } else {
        max.min(window.w_scwidth)
    };
    window.w_scwidth = min.max(held);
    Ok(())
}

/// One ASCII digit as its value. Every caller has already established that
/// the byte is a digit, or is in the half of 'signcolumn' the option table
/// vetted.
fn digit(byte: u8) -> c_int {
    c_int::from(byte) - c_int::from(b'0')
}

/// Check a 'statusline'-format value. Answers an untranslated message when
/// the format is bad.
///
/// Upstream formats the message into a function-local static, because the
/// answer has to outlive the call and the caller passes no buffer. The
/// answer is owned instead, so two checks in flight cannot collide.
///
/// # Safety
/// `s` is a C string.
pub(crate) unsafe fn check_stl_option(s: *mut c_char) -> Result<(), OptError> {
    let illegal = |c: c_int| Err(illegal_char(c));

    // SAFETY: the caller's C string.
    let mut rest = unsafe { CStr::from_ptr(s) }.to_bytes();
    let mut groupdepth: c_int = 0;

    while let Some(at) = rest.iter().position(|&b| b == b'%') {
        // Past the `%`. The value may end here, in which case the item is
        // the terminator and the membership test below rejects it.
        rest = &rest[at + 1..];
        match rest.first() {
            // "%%", the truncation mark and the item separator take no
            // width, no precision and no argument.
            Some(&b'%' | &b'<' | &b'=') => {
                rest = &rest[1..];
                continue;
            }
            Some(&b')') => {
                rest = &rest[1..];
                groupdepth -= 1;
                if groupdepth < 0 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        // A minimum width, optionally left-aligned.
        rest = rest.strip_prefix(b"-").unwrap_or(rest);
        rest = &rest[rest.iter().take_while(|b| b.is_ascii_digit()).count()..];
        // A user highlight group takes the width as its number and stops
        // there.
        if rest.first() == Some(&b'*') {
            continue;
        }
        // A maximum width.
        if let Some(after) = rest.strip_prefix(b".") {
            rest = &after[after.iter().take_while(|b| b.is_ascii_digit()).count()..];
        }
        if rest.first() == Some(&b'(') {
            groupdepth += 1;
            continue;
        }
        let Some(&item) = rest.first() else {
            return illegal(NUL);
        };
        if StlOpt::from_byte(item).is_none() {
            return illegal(c_int::from(item));
        }
        if item == b'{' {
            rest = &rest[1..];
            // "%{%…%}" re-evaluates its result as another format, and its
            // terminator is "%}" rather than a bare "}".
            let reevaluate = rest.first() == Some(&b'%');
            if reevaluate {
                rest = &rest[1..];
                if rest.first() == Some(&b'}') {
                    return illegal(c_int::from(b'}'));
                }
            }
            let close = if reevaluate {
                rest.windows(2).position(|w| w == b"%}").map(|at| at + 1)
            } else {
                rest.iter().position(|&b| b == b'}')
            };
            let Some(close) = close else {
                return Err(e_unclosed_expression_sequence.into());
            };
            rest = &rest[close..];
        }
    }

    if groupdepth != 0 {
        return Err(e_unbalanced_groups.into());
    }
    Ok(())
}

/// Does `val` hold a character an option marked as a file or directory name
/// refuses? The set is wider while 'secure' is on.
///
/// # Safety
/// `val` is a C string.
pub fn check_illegal_path_names(val: &CStr, flags: uint32_t) -> bool {
    let val = val.to_bytes();
    let holds = |set: &[u8]| val.iter().any(|b| set.contains(b));
    (flags & kOptFlagNFname as uint32_t != 0
        && holds(if secure.get() != 0 {
            b"/\\*?[|;&<>\r\n"
        } else {
            b"/\\*?[<>\r\n"
        }))
        || (flags & kOptFlagNDname as uint32_t != 0 && holds(b"*?[|;&<>\r\n"))
}
