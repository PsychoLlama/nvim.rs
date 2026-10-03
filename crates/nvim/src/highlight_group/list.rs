//! `:highlight` with no settings to apply: printing what is set.
//!
//! [`highlight_list_one`] prints one group as the `key=value` pairs that
//! would recreate it, [`ListValue`] renders one such value and
//! [`syn_list_header`] does the column arithmetic that keeps the output in
//! line. The `get_highlight_name*` pair is command-line completion.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::charset::skip;
use crate::cstr;
use crate::types::Candidate;
use core::ffi::{CStr, c_char, c_int};

use crate::charset::vim_strsize;
use crate::eval::last_set_msg;
use crate::getchar::state::got_int;
use crate::highlight::HlAttrFlags;
use crate::highlight::state::{include_default, include_link, include_none};
use crate::message::state::{msg_col, msg_silent};
use crate::message::{
    message_filtered, msg_advance, msg_clr_eos, msg_display, msg_putchar, msg_str_hl,
};
use crate::option::vars::p_verbose;
use crate::os::time::os_delay;
use crate::types::ui::kUIMessages;
use crate::types::{Expand, ExpandContext};
use crate::ui::state::Columns;
use crate::ui::{ui_flush, ui_has};

use super::{ATTR_NAMES, HLF_D, HexBuf, coloridx_to_name, group, highlight_num_groups};

/// One value in `:highlight`'s `key=value` output for a group.
enum ListValue<'a> {
    /// A set of attribute bits, spelled as the comma-separated names
    /// `cterm=`/`gui=` would take. An empty set prints nothing.
    Attrs(HlAttrFlags),
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
            ListValue::Attrs(bits) if bits.is_empty() => None,
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
    fn attrs(&mut self, mut bits: HlAttrFlags) -> &CStr {
        self.len = 0;
        self.bytes[0] = 0;
        for &(name, flag) in &ATTR_NAMES {
            if flag.is_empty() {
                break;
            }
            let underline = flag.has(HlAttrFlags::UNDERLINE_MASK);
            let hit = if underline {
                bits.masked(HlAttrFlags::UNDERLINE_MASK) == flag
            } else {
                bits.has(flag)
            };
            if !hit {
                continue;
            }
            if self.len != 0 {
                self.push(c",");
            }
            self.push(name);
            if !underline {
                bits.clear(flag);
            }
        }
        self.finish()
    }
}

/// Prints one `key=value` pair, if the value is set.
///
/// Answers whether a header has been printed for this group by now, which is
/// threaded through the whole of [`highlight_list_one`].
fn list_arg(id: c_int, didh: bool, value: ListValue, name: &CStr) -> bool {
    if got_int.get() {
        return false;
    }
    let mut buf = ValueBuf::new();
    let Some(text) = value.render(&mut buf) else {
        return didh;
    };

    // SAFETY: main-thread message calls with NUL-terminated strings.
    let width = unsafe { vim_strsize(text.as_ptr()) } + name.count_bytes() as c_int + 1;
    syn_list_header(didh, width, id, false);
    if !got_int.get() {
        if !name.is_empty() {
            msg_str_hl(name, HLF_D, false);
            msg_str_hl(c"=", HLF_D, false);
        }
        msg_display(text, 0, false);
    }
    true
}

/// One `guifg=`/`guibg=`/`guisp=` value: the name the group was given, or
/// `#rrggbb`.
fn color(idx: c_int, value: c_int, buf: &mut HexBuf) -> ListValue<'_> {
    ListValue::Text(coloridx_to_name(idx, value, buf))
}

/// Prints the group with id `id` the way `:highlight {group}` does.
pub(crate) fn highlight_list_one(id: c_int) {
    let entry = group(id);
    // SAFETY: the name is a live static string.
    if message_filtered(unsafe { cstr::at(entry.name.as_ptr().cast_mut()) }) {
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

    let mut didh = false;
    for (value, name) in pairs {
        didh = list_arg(id, didh, value, name);
    }

    if entry.link != 0 && !got_int.get() {
        syn_list_header(didh, 0, id, true);
        didh = true;
        msg_str_hl(c"links to", HLF_D, false);
        msg_putchar(' ' as c_int);
        msg_display(group(entry.link).name, 0, false);
    }

    if !didh {
        list_arg(id, didh, ListValue::Text(Some(c"cleared")), c"");
    }
    if p_verbose() > 0 {
        last_set_msg(entry.script_ctx);
    }
}

/// Starts a line, or a column, for the next thing `:highlight` prints, and
/// prints the group's name and its `xxx` sample if this is the first.
///
/// Answers whether a new line was started, which is what the caller passes
/// back as `did_header` for the rest of the group.
pub(crate) fn syn_list_header(
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
    if !did_header {
        if !ui_has(kUIMessages) || msg_col.get() > 0 {
            msg_putchar('\n' as c_int);
        }
        if got_int.get() {
            return true;
        }
        name_col = msg_display(group(id).name, 0, false);
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
        msg_str_hl(c"xxx", id, false);
        msg_putchar(' ' as c_int);
    }

    newline
}

/// The `:highlight Ni...` easter egg: flashes `NI!` at the user.
fn highlight_list() {
    // SAFETY: main-thread message calls.
    for i in (0..10).rev() {
        highlight_list_two(i, HLF_D);
    }
    for _ in 0..40 {
        highlight_list_two(99, 0);
    }
}

/// One frame of it: a slice of `"N \x08I \x08!  \x08"` chosen by `cnt`, which
/// is either 0..9 (the first frame) or 99 (the last).
fn highlight_list_two(cnt: c_int, id: c_int) {
    const FRAMES: &[u8] = b"N \x08I \x08!  \x08\0";
    // The index is 0 or 9, both inside.
    let at = (cnt / 11) as usize;
    msg_str_hl(cstr::in_bytes(&FRAMES[at..]), id, false);
    msg_clr_eos();
    ui_flush();
    // TODO(justinmk): is this delay needed? ":hi" seems to work without it.
    os_delay(if cnt == 99 { 40 } else { cnt as u64 * 50 }, false);
}

/// `strncmp(full, word, word.len()) == 0`: whether `word` is a prefix of
/// `full`. A longer `word` cannot match, because `full`'s NUL stops it.
fn is_prefix(word: &[u8], full: &[u8]) -> bool {
    word.len() <= full.len() && full.starts_with(word)
}

/// Completion for `:highlight`: group names, plus the subcommand words that
/// could still be typed at this position. The argument starts at `arg` in
/// the completion's line.
pub(crate) fn set_context_in_highlight_cmd(expand: &mut Expand, arg: usize) {
    // Default: expand group names.
    expand.context = ExpandContext::Highlight;
    expand.pattern = arg;
    include_link.set(2);
    include_default.set(1);

    let line = expand.line_cstr().to_bytes().to_vec();
    let at = |i: usize| line.get(i).copied().unwrap_or(0);
    let skipwhite = |i: usize| i + skip::white(line.get(i..).unwrap_or_default());
    let skiptowhite = |i: usize| i + skip::to_white(line.get(i..).unwrap_or_default());

    if at(arg) == 0 {
        return;
    }

    // (Part of) a subcommand already typed.
    let mut arg = arg;
    let mut p = skiptowhite(arg);
    if at(p) == 0 {
        return;
    }

    // Past "default" or the group name.
    include_default.set(0);
    if is_prefix(&line[arg..p], b"default") {
        arg = skipwhite(p);
        expand.pattern = arg;
        p = skiptowhite(arg);
    }
    if at(p) == 0 {
        return;
    }

    // Past the group name.
    include_link.set(0);
    if at(arg + 1) == b'i' && at(arg) == b'N' {
        highlight_list();
    }
    if is_prefix(&line[arg..p], b"link") || is_prefix(&line[arg..p], b"clear") {
        expand.pattern = skipwhite(p);
        p = skiptowhite(expand.pattern);
        if at(p) != 0 {
            // Past the first group name.
            expand.pattern = skipwhite(p);
            p = skiptowhite(expand.pattern);
        }
    }
    if at(p) != 0 {
        // Past the group name(s).
        expand.context = ExpandContext::Nothing;
    }
}

/// `expand_generic`'s callback: the `idx`th completion candidate.
pub(crate) fn get_highlight_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let idx = c_int::try_from(idx).ok()?;
    // SAFETY: a group's name or a literal; NUL-terminated, null past the end.
    let name = unsafe { cstr::at_opt(get_highlight_name_ext(idx, true)) }?;
    Some(Candidate::Owned(name.to_owned()))
}

/// The `idx`th completion candidate: the group names first, then whichever of
/// `none`/`default`/`link`/`clear` the `include_*` flags allow.
///
/// A cleared group answers `""` rather than NULL, which would end the walk:
/// entries are never removed from the table, only cleared.
///
/// # Safety
/// Main thread only.
pub(crate) unsafe fn get_highlight_name_ext(idx: c_int, skip_cleared: bool) -> *const c_char {
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
