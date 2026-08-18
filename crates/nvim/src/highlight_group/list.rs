//! `:highlight` with no settings to apply: printing what is set.
//!
//! [`highlight_list_one`] prints one group as the `key=value` pairs that
//! would recreate it, [`ListValue`] renders one such value and
//! [`syn_list_header`] does the column arithmetic that keeps the output in
//! line. The `get_highlight_name*` pair is command-line completion.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use crate::charset::{skiptowhite, skipwhite, vim_strsize};
use crate::eval::last_set_msg;
use crate::main::{
    Columns, got_int, include_default, include_link, include_none, msg_col, msg_silent, p_verbose,
};
use crate::message::{
    message_filtered, msg_advance, msg_clr_eos, msg_outtrans, msg_putchar, msg_puts_hl,
};
use crate::os::time::os_delay;
use crate::types::expand_T;
use crate::types::ui::kUIMessages;
use crate::ui::{ui_flush, ui_has};

use super::{
    ATTR_NAMES, EXPAND_HIGHLIGHT, EXPAND_NOTHING, HL_UNDERLINE_MASK, HLF_D, HexBuf,
    coloridx_to_name, group, highlight_num_groups,
};

/// One value in `:highlight`'s `key=value` output for a group.
enum ListValue<'a> {
    /// A set of `HL_*` bits, spelled as the comma-separated names
    /// `cterm=`/`gui=` would take. An empty set prints nothing.
    Attrs(c_int),
    /// A colour number plus one, so that 0 means "not set" and prints
    /// nothing.
    Number(c_int),
    /// A ready-made string; `None` prints nothing.
    Text(Option<&'a CStr>),
}

impl<'a> ListValue<'a> {
    /// Renders the value, or `None` if this pair is not to be printed. The
    /// buffer is the caller's because the attribute spelling is built there.
    fn render<'b>(&self, buf: &'b mut ValueBuf) -> Option<&'b CStr>
    where
        'a: 'b,
    {
        match *self {
            ListValue::Text(text) => text,
            ListValue::Number(0) => None,
            ListValue::Number(value) => Some(buf.number(value - 1)),
            ListValue::Attrs(0) => None,
            ListValue::Attrs(bits) => Some(buf.attrs(bits)),
        }
    }
}

/// Where an attribute list or a number is spelled out. 100 bytes, as
/// upstream: every attribute name and its comma fits twice over.
struct ValueBuf {
    bytes: [u8; 100],
    len: usize,
}

impl ValueBuf {
    fn new() -> Self {
        Self {
            bytes: [0; 100],
            len: 0,
        }
    }

    fn finish(&self) -> &CStr {
        CStr::from_bytes_with_nul(&self.bytes[..self.len + 1]).expect("NUL-terminated")
    }

    fn number(&mut self, value: c_int) -> &CStr {
        let text = value.to_string();
        self.len = text.len();
        self.bytes[..self.len].copy_from_slice(text.as_bytes());
        self.bytes[self.len] = 0;
        self.finish()
    }

    /// `xstrlcat`'s truncating append, which is what upstream used here.
    fn push(&mut self, text: &CStr) {
        let text = text.to_bytes();
        let room = (self.bytes.len() - 1 - self.len).min(text.len());
        self.bytes[self.len..self.len + room].copy_from_slice(&text[..room]);
        self.len += room;
        self.bytes[self.len] = 0;
    }

    /// The comma-separated names for the `HL_*` bits in `bits`.
    ///
    /// The underline styles share a field, so one of those only prints when
    /// it is exactly the style set; every other bit is a plain test, and is
    /// cleared as it prints so that `inverse` does not follow `reverse`.
    fn attrs(&mut self, mut bits: c_int) -> &CStr {
        self.len = 0;
        self.bytes[0] = 0;
        for &(name, flag) in &ATTR_NAMES {
            if flag == 0 {
                break;
            }
            let underline = flag & HL_UNDERLINE_MASK != 0;
            let hit = if underline {
                bits & HL_UNDERLINE_MASK == flag
            } else {
                bits & flag != 0
            };
            if !hit {
                continue;
            }
            if self.len != 0 {
                self.push(c",");
            }
            self.push(name);
            if !underline {
                bits &= !flag;
            }
        }
        self.finish()
    }
}

/// Prints one `key=value` pair, if the value is set.
///
/// Answers whether a header has been printed for this group by now, which is
/// threaded through the whole of [`highlight_list_one`].
///
/// # Safety
/// Writes to the message area; main thread only.
unsafe fn list_arg(id: c_int, didh: bool, value: ListValue, name: &CStr) -> bool {
    if got_int.get() {
        return false;
    }
    let mut buf = ValueBuf::new();
    let Some(text) = value.render(&mut buf) else {
        return didh;
    };

    // SAFETY: main-thread message calls with NUL-terminated strings.
    unsafe {
        let width = vim_strsize(text.as_ptr()) + name.count_bytes() as c_int + 1;
        syn_list_header(didh, width, id, false);
        if !got_int.get() {
            if !name.is_empty() {
                msg_puts_hl(name.as_ptr(), HLF_D, false);
                msg_puts_hl(c"=".as_ptr(), HLF_D, false);
            }
            msg_outtrans(text.as_ptr(), 0, false);
        }
    }
    true
}

/// One `guifg=`/`guibg=`/`guisp=` value: the name the group was given, or
/// `#rrggbb`.
fn color(idx: c_int, value: c_int, buf: &mut HexBuf) -> ListValue<'_> {
    ListValue::Text(coloridx_to_name(idx, value, buf))
}

/// Prints the group with id `id` the way `:highlight {group}` does.
///
/// # Safety
/// Writes to the message area; main thread only.
pub(crate) unsafe fn highlight_list_one(id: c_int) {
    let entry = group(id);
    // SAFETY: the name is a live static string.
    if unsafe { message_filtered(entry.name.as_ptr().cast_mut()) } {
        return;
    }
    // Don't list a specialized `@a.b` group if its parent is used instead.
    if entry.parent != 0 && entry.cleared {
        return;
    }

    let (mut fg, mut bg, mut sp) = ([0; 8], [0; 8], [0; 8]);
    let pairs: [(ListValue, &CStr); 8] = [
        (ListValue::Attrs(entry.cterm), c"cterm"),
        (ListValue::Number(entry.cterm_fg), c"ctermfg"),
        (ListValue::Number(entry.cterm_bg), c"ctermbg"),
        (ListValue::Attrs(entry.gui), c"gui"),
        (color(entry.rgb_fg_idx, entry.rgb_fg, &mut fg), c"guifg"),
        (color(entry.rgb_bg_idx, entry.rgb_bg, &mut bg), c"guibg"),
        (color(entry.rgb_sp_idx, entry.rgb_sp, &mut sp), c"guisp"),
        (ListValue::Number(entry.blend + 1), c"blend"),
    ];

    // SAFETY: main-thread message calls.
    unsafe {
        let mut didh = false;
        for (value, name) in pairs {
            didh = list_arg(id, didh, value, name);
        }

        if entry.link != 0 && !got_int.get() {
            syn_list_header(didh, 0, id, true);
            didh = true;
            msg_puts_hl(c"links to".as_ptr(), HLF_D, false);
            msg_putchar(' ' as c_int);
            msg_outtrans(group(entry.link).name.as_ptr(), 0, false);
        }

        if !didh {
            list_arg(id, didh, ListValue::Text(Some(c"cleared")), c"");
        }
        if p_verbose.get() > 0 {
            last_set_msg(entry.script_ctx);
        }
    }
}

/// Starts a line, or a column, for the next thing `:highlight` prints, and
/// prints the group's name and its `xxx` sample if this is the first.
///
/// Answers whether a new line was started, which is what the caller passes
/// back as `did_header` for the rest of the group.
///
/// # Safety
/// Writes to the message area; main thread only.
pub unsafe fn syn_list_header(
    did_header: bool,
    outlen: c_int,
    id: c_int,
    force_newline: bool,
) -> bool {
    let mut endcol = 19;
    let mut newline = true;
    let mut name_col = 0;
    let mut adjust = true;

    // SAFETY: main-thread message calls.
    unsafe {
        if !did_header {
            if !ui_has(kUIMessages) || msg_col.get() > 0 {
                msg_putchar('\n' as c_int);
            }
            if got_int.get() {
                return true;
            }
            name_col = msg_outtrans(group(id).name.as_ptr(), 0, false);
            msg_col.set(name_col);
            endcol = 15;
        } else if (ui_has(kUIMessages) || msg_silent.get() != 0) && !force_newline {
            msg_putchar(' ' as c_int);
            adjust = false;
        } else if msg_col.get() + outlen + 1 >= Columns.get() || force_newline {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                return true;
            }
        } else if msg_col.get() >= endcol {
            // Wrapping around is like starting a new line.
            newline = false;
        }

        if adjust {
            if msg_col.get() >= endcol {
                // Output at least one space.
                endcol = msg_col.get() + 1;
            }
            msg_advance(endcol);
        }

        if !did_header {
            if endcol == Columns.get() - 1 && endcol <= name_col {
                msg_putchar(' ' as c_int);
            }
            msg_puts_hl(c"xxx".as_ptr(), id, false);
            msg_putchar(' ' as c_int);
        }
    }

    newline
}

/// The `:highlight Ni...` easter egg: flashes `NI!` at the user.
///
/// # Safety
/// Writes to the message area and flushes the UI; main thread only.
unsafe fn highlight_list() {
    // SAFETY: main-thread message calls.
    unsafe {
        for i in (0..10).rev() {
            highlight_list_two(i, HLF_D);
        }
        for _ in 0..40 {
            highlight_list_two(99, 0);
        }
    }
}

/// One frame of it: a slice of `"N \x08I \x08!  \x08"` chosen by `cnt`, which
/// is either 0..9 (the first frame) or 99 (the last).
///
/// # Safety
/// See [`highlight_list`].
unsafe fn highlight_list_two(cnt: c_int, id: c_int) {
    const FRAMES: &CStr = c"N \x08I \x08!  \x08";
    // SAFETY: main-thread message calls; the index is 0 or 9, both inside.
    unsafe {
        let at = (cnt / 11) as usize;
        msg_puts_hl(FRAMES.as_ptr().add(at), id, false);
        msg_clr_eos();
        ui_flush();
        // TODO(justinmk): is this delay needed? ":hi" seems to work without it.
        os_delay(if cnt == 99 { 40 } else { cnt as u64 * 50 }, false);
    }
}

/// `strncmp(full, word, word.len()) == 0`: whether `word` is a prefix of
/// `full`. A longer `word` cannot match, because `full`'s NUL stops it.
fn is_prefix(word: &[u8], full: &[u8]) -> bool {
    word.len() <= full.len() && full.starts_with(word)
}

/// Completion for `:highlight`: group names, plus the subcommand words that
/// could still be typed at this position.
///
/// # Safety
/// `arg` is the NUL-terminated rest of the command line, which `xp` is
/// pointed into; main thread only.
pub unsafe fn set_context_in_highlight_cmd(xp: *mut expand_T, arg: *const c_char) {
    // SAFETY: the caller's expansion state and command line.
    unsafe {
        // Default: expand group names.
        (*xp).xp_context = EXPAND_HIGHLIGHT;
        (*xp).xp_pattern = arg.cast_mut();
        include_link.set(2);
        include_default.set(1);

        if *arg == 0 {
            return;
        }

        // (Part of) a subcommand already typed.
        let mut arg = arg;
        let mut p = skiptowhite(arg);
        if *p == 0 {
            return;
        }

        // Past "default" or the group name.
        include_default.set(0);
        let word = |arg: *const c_char, p: *const c_char| {
            core::slice::from_raw_parts(arg.cast::<u8>(), p.offset_from(arg) as usize)
        };
        if is_prefix(word(arg, p), b"default") {
            arg = skipwhite(p);
            (*xp).xp_pattern = arg.cast_mut();
            p = skiptowhite(arg);
        }
        if *p == 0 {
            return;
        }

        // Past the group name.
        include_link.set(0);
        if *arg.add(1) == b'i' as c_char && *arg == b'N' as c_char {
            highlight_list();
        }
        if is_prefix(word(arg, p), b"link") || is_prefix(word(arg, p), b"clear") {
            (*xp).xp_pattern = skipwhite(p);
            p = skiptowhite((*xp).xp_pattern);
            if *p != 0 {
                // Past the first group name.
                (*xp).xp_pattern = skipwhite(p);
                p = skiptowhite((*xp).xp_pattern);
            }
        }
        if *p != 0 {
            // Past the group name(s).
            (*xp).xp_context = EXPAND_NOTHING;
        }
    }
}

/// `ExpandGeneric`'s callback: the `idx`th completion candidate.
///
/// Keeps the raw signature because cmdexpand's generator table holds it as a
/// function pointer of that shape.
///
/// # Safety
/// Main thread only.
pub unsafe fn get_highlight_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    // SAFETY: as the callee.
    unsafe { get_highlight_name_ext(xp, idx, true).cast_mut() }
}

/// The `idx`th completion candidate: the group names first, then whichever of
/// `none`/`default`/`link`/`clear` the `include_*` flags allow.
///
/// A cleared group answers `""` rather than NULL, which would end the walk:
/// entries are never removed from the table, only cleared.
///
/// # Safety
/// Main thread only.
pub unsafe fn get_highlight_name_ext(
    _xp: *mut expand_T,
    idx: c_int,
    skip_cleared: bool,
) -> *const c_char {
    if idx < 0 {
        return core::ptr::null();
    }
    let groups = highlight_num_groups();
    if skip_cleared && idx < groups && group(idx + 1).cleared {
        return c"".as_ptr();
    }

    let none = include_none.get();
    let default = include_default.get();
    let link = include_link.get();
    if idx == groups && none != 0 {
        c"none".as_ptr()
    } else if idx == groups + none && default != 0 {
        c"default".as_ptr()
    } else if idx == groups + none + default && link != 0 {
        c"link".as_ptr()
    } else if idx == groups + none + default + 1 && link != 0 {
        c"clear".as_ptr()
    } else if idx >= groups {
        core::ptr::null()
    } else {
        group(idx + 1).name.as_ptr()
    }
}
