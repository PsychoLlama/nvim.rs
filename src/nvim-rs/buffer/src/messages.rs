//! Translated message string accessors for the buffer crate.
//!
//! Consolidates all message string wrappers that were previously C functions in
//! `buffer_shim.c`. Each function calls `gettext()` on the English message string
//! (which serves as the gettext msgid) and returns the translated string.
//!
//! The English strings are copied byte-for-byte from the C source so that
//! gettext lookup is identical regardless of where the string literal lives.
//!
//! # PRId64 note
//!
//! On Linux x86_64, `PRId64` expands to `"ld"`, so format strings using
//! `"%" PRId64` become `"%ld"` in the gettext key. The constants below use
//! `%ld` directly to match.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_ulong};

extern "C" {
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn ngettext(msgid1: *const c_char, msgid2: *const c_char, n: c_ulong) -> *const c_char;
}

/// Inline helper: translate an English string via gettext at runtime.
#[inline]
pub unsafe fn gt(s: *const c_char) -> *const c_char {
    gettext(s)
}

// ---------------------------------------------------------------------------
// Message string constants (copied exactly from buffer_shim.c / headers).
// These serve as gettext msgids; any whitespace or character difference would
// silently break translations.
// ---------------------------------------------------------------------------

/// "[No Name]" -- displayed for unnamed buffers.
pub const MSG_NO_NAME: &std::ffi::CStr = c"[No Name]";

/// "E382: Cannot write, 'buftype' option is set"
pub const MSG_E382: &std::ffi::CStr = c"E382: Cannot write, 'buftype' option is set";

/// "[Quickfix List]" -- from buffer.h `msg_qflist`
pub const MSG_QFLIST: &std::ffi::CStr = c"[Quickfix List]";

/// "[Location List]" -- from buffer.h `msg_loclist`
pub const MSG_LOCLIST: &std::ffi::CStr = c"[Location List]";

/// "[Command Line]"
pub const MSG_COMMAND_LINE: &std::ffi::CStr = c"[Command Line]";

/// "[Prompt]"
pub const MSG_PROMPT: &std::ffi::CStr = c"[Prompt]";

/// "[Scratch]"
pub const MSG_SCRATCH: &std::ffi::CStr = c"[Scratch]";

/// "E23: No alternate file" -- from errors.h `e_noalt`
pub const MSG_E_NOALT: &std::ffi::CStr = c"E23: No alternate file";

/// " ((%d) of %d)" -- invalid arg number format
pub const MSG_ARG_NUMBER_INVALID: &std::ffi::CStr = c" ((%d) of %d)";

/// " (%d of %d)" -- arg number format
pub const MSG_ARG_NUMBER: &std::ffi::CStr = c" (%d of %d)";

/// "All"
pub const MSG_ALL: &std::ffi::CStr = c"All";

/// "Top"
pub const MSG_TOP: &std::ffi::CStr = c"Top";

/// "Bot"
pub const MSG_BOT: &std::ffi::CStr = c"Bot";

/// "%d%%" -- percentage format
pub const MSG_PCT: &std::ffi::CStr = c"%d%%";

/// "%3s" -- 3-char string format
pub const MSG_3S: &std::ffi::CStr = c"%3s";

/// " [Modified]"
pub const MSG_MODIFIED: &std::ffi::CStr = c" [Modified]";

/// "[Not edited]"
pub const MSG_NOT_EDITED: &std::ffi::CStr = c"[Not edited]";

/// "[New]"
pub const MSG_NEW: &std::ffi::CStr = c"[New]";

/// "[Read errors]"
pub const MSG_READ_ERRORS: &std::ffi::CStr = c"[Read errors]";

/// "[RO]"
pub const MSG_RO: &std::ffi::CStr = c"[RO]";

/// "[readonly]"
pub const MSG_READONLY: &std::ffi::CStr = c"[readonly]";

/// "--No lines in buffer--" -- from globals.h `no_lines_msg`
pub const MSG_NO_LINES: &std::ffi::CStr = c"--No lines in buffer--";

/// "line %ld of %ld --%d%%-- col " -- fileinfo line format
/// (PRId64 expands to "ld" on Linux x86_64)
pub const MSG_FILEINFO_LINE_FMT: &std::ffi::CStr = c"line %ld of %ld --%d%%-- col ";

/// "line %ld" -- buflist line format
/// (PRId64 expands to "ld" on Linux x86_64)
pub const MSG_BUFLIST_LINE_FMT: &std::ffi::CStr = c"line %ld";

/// "%ld line --%d%%--" -- singular form for ngettext line count
/// (PRId64 expands to "ld" on Linux x86_64)
pub const MSG_LINE_COUNT_SINGULAR: &std::ffi::CStr = c"%ld line --%d%%--";

/// "%ld lines --%d%%--" -- plural form for ngettext line count
/// (PRId64 expands to "ld" on Linux x86_64)
pub const MSG_LINE_COUNT_PLURAL: &std::ffi::CStr = c"%ld lines --%d%%--";

// ---------------------------------------------------------------------------
// Translated accessor functions.
// These replace the C wrappers that called _("...") directly.
// ---------------------------------------------------------------------------

/// Get translated "[No Name]" string.
pub unsafe fn no_name_msg() -> *const c_char {
    gt(MSG_NO_NAME.as_ptr())
}

/// Get translated E382 error message string.
pub unsafe fn e382_msg() -> *const c_char {
    gt(MSG_E382.as_ptr())
}

/// Get translated "[Quickfix List]" string.
pub unsafe fn msg_qflist() -> *const c_char {
    gt(MSG_QFLIST.as_ptr())
}

/// Get translated "[Location List]" string.
pub unsafe fn msg_loclist() -> *const c_char {
    gt(MSG_LOCLIST.as_ptr())
}

/// Get translated "[Command Line]" string.
pub unsafe fn msg_command_line() -> *const c_char {
    gt(MSG_COMMAND_LINE.as_ptr())
}

/// Get translated "[Prompt]" string.
pub unsafe fn msg_prompt() -> *const c_char {
    gt(MSG_PROMPT.as_ptr())
}

/// Get translated "[Scratch]" string.
pub unsafe fn msg_scratch() -> *const c_char {
    gt(MSG_SCRATCH.as_ptr())
}

/// Get translated "E23: No alternate file" string.
pub unsafe fn e_noalt() -> *const c_char {
    gt(MSG_E_NOALT.as_ptr())
}

/// Get translated " ((%d) of %d)" format string.
pub unsafe fn msg_arg_number_invalid() -> *const c_char {
    gt(MSG_ARG_NUMBER_INVALID.as_ptr())
}

/// Get translated " (%d of %d)" format string.
pub unsafe fn msg_arg_number() -> *const c_char {
    gt(MSG_ARG_NUMBER.as_ptr())
}

/// Get translated "All" string.
pub unsafe fn msg_all() -> *const c_char {
    gt(MSG_ALL.as_ptr())
}

/// Get translated "Top" string.
pub unsafe fn msg_top() -> *const c_char {
    gt(MSG_TOP.as_ptr())
}

/// Get translated "Bot" string.
pub unsafe fn msg_bot() -> *const c_char {
    gt(MSG_BOT.as_ptr())
}

/// Get translated "%d%%" format string.
pub unsafe fn msg_pct() -> *const c_char {
    gt(MSG_PCT.as_ptr())
}

/// Get translated "%3s" format string.
pub unsafe fn msg_3s() -> *const c_char {
    gt(MSG_3S.as_ptr())
}

/// Get translated " [Modified]" string.
pub unsafe fn msg_modified() -> *const c_char {
    gt(MSG_MODIFIED.as_ptr())
}

/// Get translated "[Not edited]" string.
pub unsafe fn msg_not_edited() -> *const c_char {
    gt(MSG_NOT_EDITED.as_ptr())
}

/// Get translated "[New]" string.
pub unsafe fn msg_new() -> *const c_char {
    gt(MSG_NEW.as_ptr())
}

/// Get translated "[Read errors]" string.
pub unsafe fn msg_read_errors() -> *const c_char {
    gt(MSG_READ_ERRORS.as_ptr())
}

/// Get translated "[RO]" string.
pub unsafe fn msg_ro() -> *const c_char {
    gt(MSG_RO.as_ptr())
}

/// Get translated "[readonly]" string.
pub unsafe fn msg_readonly() -> *const c_char {
    gt(MSG_READONLY.as_ptr())
}

/// Get translated "--No lines in buffer--" string.
pub unsafe fn no_lines_msg() -> *const c_char {
    gt(MSG_NO_LINES.as_ptr())
}

/// Get translated line-position format string "line %ld of %ld --%d%%-- col ".
pub unsafe fn fileinfo_line_fmt() -> *const c_char {
    gt(MSG_FILEINFO_LINE_FMT.as_ptr())
}

/// Get translated "line %ld" format string.
pub unsafe fn buflist_line_fmt() -> *const c_char {
    gt(MSG_BUFLIST_LINE_FMT.as_ptr())
}

/// Get translated plural line-count format string.
///
/// Returns the singular form `"%ld line --%d%%--"` or plural form
/// `"%ld lines --%d%%--"` depending on `n`.
///
/// Mirrors the C `nvim_ngettext_line_count` wrapper which calls `NGETTEXT`.
pub unsafe fn ngettext_line_count(n: i64) -> *const c_char {
    ngettext(
        MSG_LINE_COUNT_SINGULAR.as_ptr(),
        MSG_LINE_COUNT_PLURAL.as_ptr(),
        n as c_ulong,
    )
}
