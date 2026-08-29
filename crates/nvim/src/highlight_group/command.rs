//! [`do_highlight`], the `:highlight` command.
//!
//! It parses `:hi [default] {group} key=value ...` — and the `clear` and
//! `link` forms — writing what it finds into the group's table entry. Each
//! key family has its own rules about what `default` and `init` mean for an
//! already-set value, which is why they are one function each rather than a
//! table.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::highlight::HlAttrFlags;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};

use crate::api::private::helpers::cstr_as_string;
use crate::ascii::ascii_isdigit;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_all_later};
use crate::eval::vars::do_unlet;
use crate::ex_docmd::ends_excmd;
use crate::lua::executor::nlua_set_sctx;
use crate::main::{
    cterm_normal_bg_color, cterm_normal_fg_color, current_sctx, got_int, need_highlight_changed,
    normal_bg, normal_fg, normal_sp, p_bg, starting, t_colors, updating_screen,
};
use crate::message::{emsg, msg_ext_set_kind};
use crate::message_fmt::{c_str, msg_bytes};
use crate::option::{option_was_set, reset_option_was_set, set_option_value_give_err};
use crate::options::kOptBackground;
use crate::os::cshim::gettext;
use crate::types::ui::kUILinegrid;
use crate::types::{OptVal, OptValData, OptionSetFlags, estack_T};
use crate::ui::{ui_default_colors_set, ui_has, ui_refresh, ui_rgb_attached};

use super::{
    ATTR_NAMES, SG_CTERM, SG_GUI, SG_LINK, cterm_color_index,
    e_group_has_settings_highlight_link_ignored, group, highlight_attr_set_all, highlight_clear,
    highlight_list_one, highlight_num_groups, hl_has_settings, init_highlight, kColorIdxNone,
    kOptValTypeString, lookup_color, name_to_color, restore_cterm_colors, set_hl_attr,
    syn_check_group, syn_name2id_len, with_group,
};
use crate::highlight_group::highlight_changed;

/// The command line being parsed, as bytes plus the pointer the error
/// messages need: several of them print the rest of the line from the key
/// that failed, so a slice is not enough.
struct Line {
    base: *const c_char,
    bytes: &'static [u8],
    at: usize,
}

impl Line {
    /// # Safety
    /// `line` outlives the parse.
    unsafe fn new(line: *const c_char) -> Self {
        // SAFETY: the caller's NUL-terminated command line.
        let bytes = unsafe { CStr::from_ptr(line) }.to_bytes();
        Self {
            base: line,
            // The borrow lives as long as the parse, which is inside the
            // caller's own borrow of the line.
            bytes: unsafe { core::mem::transmute::<&[u8], &'static [u8]>(bytes) },
            at: 0,
        }
    }

    fn peek(&self) -> u8 {
        self.bytes.get(self.at).copied().unwrap_or(0)
    }

    fn at_end(&self) -> bool {
        ends_excmd(c_int::from(self.peek())) != 0
    }

    /// A pointer at offset `at`, for `%s` in an error message.
    fn ptr(&self, at: usize) -> *const c_char {
        // SAFETY: `at` is an offset this parse produced, so it is inside the
        // line or on its terminator.
        unsafe { self.base.add(at) }
    }

    fn skip_white(&mut self) {
        while matches!(self.peek(), b' ' | b'\t') {
            self.at += 1;
        }
    }

    /// Consumes up to the next white space and answers what was skipped.
    fn word(&mut self) -> &'static [u8] {
        let start = self.at;
        while !matches!(self.peek(), 0 | b' ' | b'\t') {
            self.at += 1;
        }
        &self.bytes[start..self.at]
    }

    /// [`Self::word`] followed by the white space after it.
    fn word_then_space(&mut self) -> &'static [u8] {
        let word = self.word();
        self.skip_white();
        word
    }
}

/// `strncmp(full, word, word.len()) == 0`: whether `word` is a prefix of
/// `full`, which is how `:hi` accepts `def`, `cle` and `li`.
fn is_prefix(word: &[u8], full: &[u8]) -> bool {
    word.len() <= full.len() && full.starts_with(word)
}

/// Handles `:highlight`.
///
/// `forceit` is the `!`, which allows a link over a group that has its own
/// settings; `init` marks the compiled-in defaults and colour schemes, which
/// do not overwrite anything the user set. `:highlight clear` calls back in
/// with both set for every group.
///
/// # Safety
/// Runs messages, autocommands and redraws; main thread only.
pub(crate) unsafe fn do_highlight(line: *const c_char, forceit: bool, init: bool) {
    // SAFETY: the caller's NUL-terminated command line, live for the parse.
    let mut line = unsafe { Line::new(line) };

    // No argument: list all highlighting.
    if !init && line.at_end() {
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        let mut id = 1;
        while id <= highlight_num_groups() && !got_int.get() {
            // TODO(brammool): only call when the group has attributes set
            unsafe { highlight_list_one(id) };
            id += 1;
        }
        return;
    }

    let mut name = line.word_then_space();
    let dodefault = is_prefix(name, b"default");
    if dodefault {
        name = line.word_then_space();
    }
    // An else-if chain, not two tests: the empty word left by a bare
    // `:hi default` is a prefix of both, and upstream calls it `clear`.
    let doclear = is_prefix(name, b"clear");
    let dolink = !doclear && is_prefix(name, b"link");

    // ":highlight {group-name}": list highlighting for one group.
    if !doclear && !dolink && line.at_end() {
        let id = unsafe { syn_name2id_len(name.as_ptr().cast(), name.len()) };
        if id == 0 {
            let shown = msg_bytes(name);
            semsg!("E411: Highlight group not found: {shown}");
        } else {
            unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
            unsafe { highlight_list_one(id) };
        }
        return;
    }

    if dolink {
        unsafe { highlight_link(&mut line, forceit, init, dodefault) };
        return;
    }

    if doclear {
        name = line.word_then_space();
        if name.is_empty() {
            // ":highlight clear": back to the compiled-in defaults.
            unsafe { do_unlet(c"g:colors_name".as_ptr(), 13, true) };
            restore_cterm_colors();
            for id in 1..=highlight_num_groups() {
                highlight_clear(id);
            }
            unsafe { init_highlight(true, true) };
            unsafe { highlight_changed() };
            unsafe { redraw_all_later(UPD_NOT_VALID) };
            return;
        }
    }

    // Find the group name in the table. If it does not exist yet, add it.
    let id = unsafe { syn_check_group(name.as_ptr().cast(), name.len()) };
    if id == 0 {
        return; // Failed.
    }

    // Return if "default" was used and the group already has settings.
    if dodefault && hl_has_settings(id, true) {
        return;
    }

    // A copy, so that the end can tell whether anything actually changed.
    let before = group(id);
    let is_normal_group = before.name_u == c"NORMAL";

    // Clear the highlighting for ":hi clear {group}" and ":hi clear".
    if doclear || (forceit && init) {
        highlight_clear(id);
        if !doclear {
            with_group(id, |entry| entry.set = 0);
        }
    }

    let mut state = KeyLoop {
        id,
        init,
        is_normal_group,
        did_change: false,
        error: false,
    };
    if !doclear {
        unsafe { state.run(&mut line) };
    }

    let mut did_highlight_changed = false;
    if !state.error && is_normal_group {
        // Every group may be using "bg" and/or "fg", which just moved.
        unsafe { highlight_attr_set_all() };

        if !ui_has(kUILinegrid) && starting.get() == 0 {
            // Older UIs assume the screen is cleared after the Normal
            // group changes.
            unsafe { ui_refresh() };
        } else {
            // TUI and newer UIs repaint themselves; the UPD_NOT_VALID
            // redraw below still handles `guibg=fg` and friends.
            unsafe { ui_default_colors_set() };
        }
        did_highlight_changed = true;
        unsafe { redraw_all_later(UPD_NOT_VALID) };
    } else {
        unsafe { set_hl_attr(id) };
    }
    with_group(id, |entry| {
        entry.script_ctx = current_sctx.get();
        entry.script_ctx.sc_lnum += sourcing_lnum();
        unsafe { nlua_set_sctx(&raw mut entry.script_ctx) };
    });

    // Call `highlight_changed` once after a sequence of `:highlight`
    // commands, and only if an attribute actually changed.
    if (state.did_change || group(id) != before) && !did_highlight_changed {
        // Do not redraw while redrawing: evaluating 'statusline' can
        // change the StatusLine group.
        if !updating_screen.get() {
            unsafe { redraw_all_later(UPD_NOT_VALID) };
        }
        need_highlight_changed.set(true);
    }
}

/// The innermost `estack_T`, which is what `SOURCING_LNUM`/`SOURCING_NAME`
/// read.
fn sourcing() -> estack_T {
    crate::runtime::innermost_frame()
}

/// `SOURCING_LNUM`: the line of the script being sourced.
pub(crate) fn sourcing_lnum() -> c_int {
    sourcing().es_lnum
}

/// `:highlight [default] link {from} {to}`.
///
/// # Safety
/// See [`do_highlight`].
unsafe fn highlight_link(line: &mut Line, forceit: bool, init: bool, dodefault: bool) {
    // SAFETY: the caller's live line, and the editor's own tables.
    let from_at = line.at;
    let from = line.word_then_space();
    let to = line.word_then_space();
    if from.is_empty() || to.is_empty() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(line.ptr(from_at)) };
        semsg!("E412: Not enough arguments: \":highlight link {arg0}\"");
        return;
    }
    if !line.at_end() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(line.ptr(from_at)) };
        semsg!("E413: Too many arguments: \":highlight link {arg0}\"");
        return;
    }

    let from_id = unsafe { syn_check_group(from.as_ptr().cast(), from.len()) };
    let to_id = if to.starts_with(b"NONE") {
        0
    } else {
        unsafe { syn_check_group(to.as_ptr().cast(), to.len()) }
    };

    if from_id <= 0 {
        return;
    }
    if dodefault && (forceit || group(from_id).deflink == 0) {
        with_group(from_id, |entry| {
            entry.deflink = to_id;
            entry.deflink_sctx = current_sctx.get();
            entry.deflink_sctx.sc_lnum += sourcing_lnum();
            unsafe { nlua_set_sctx(&raw mut entry.deflink_sctx) };
        });
    }

    let entry = group(from_id);
    if init && entry.set != 0 {
        return;
    }
    if to_id > 0 && !forceit && !init && hl_has_settings(from_id, dodefault) {
        // Don't allow a link when the group already has highlighting,
        // unless '!' is used.
        if sourcing_name_is_null() && !dodefault {
            emsg(gettext(e_group_has_settings_highlight_link_ignored));
        }
    } else if entry.link != to_id
        || entry.script_ctx.sc_sid != current_sctx.get().sc_sid
        || entry.cleared
    {
        with_group(from_id, |entry| {
            if !init {
                entry.set |= SG_LINK as c_int;
            }
            entry.link = to_id;
            entry.script_ctx = current_sctx.get();
            entry.script_ctx.sc_lnum += sourcing_lnum();
            unsafe { nlua_set_sctx(&raw mut entry.script_ctx) };
            entry.cleared = false;
        });
        unsafe { redraw_all_later(UPD_SOME_VALID) };
        // Only call highlight_changed() once after multiple changes.
        need_highlight_changed.set(true);
    }
}

/// `SOURCING_NAME == NULL`: whether the innermost entry names a script.
fn sourcing_name_is_null() -> bool {
    sourcing().es_name.is_null()
}

/// The `key=value` pairs of one `:highlight {group} ...` command.
struct KeyLoop {
    id: c_int,
    init: bool,
    is_normal_group: bool,
    /// A `gui*=` handler said the value it wrote differs from the old one.
    did_change: bool,
    error: bool,
}

impl KeyLoop {
    /// # Safety
    /// See [`do_highlight`].
    unsafe fn run(&mut self, line: &mut Line) {
        // SAFETY: the caller's live line, and the editor's own tables.
        while !line.at_end() {
            let key_at = line.at;
            if line.peek() == b'=' {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(line.ptr(key_at)) };
                semsg!("E415: Unexpected equal sign: {arg0}");
                break;
            }

            // Isolate the key: "term", "ctermfg", "guibg", ...
            let start = line.at;
            while !matches!(line.peek(), 0 | b' ' | b'\t' | b'=') {
                line.at += 1;
            }
            let key = line.bytes[start..line.at].to_ascii_uppercase();
            if key.len() > 63 {
                emsg(gettext(c"E423: Illegal argument"));
                break;
            }
            line.skip_white();

            if key == b"NONE" {
                if !self.init || group(self.id).set == 0 {
                    if !self.init {
                        with_group(self.id, |entry| entry.set |= (SG_CTERM + SG_GUI) as c_int);
                    }
                    highlight_clear(self.id);
                }
                continue;
            }

            if line.peek() != b'=' {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(line.ptr(key_at)) };
                semsg!("E416: Missing equal sign: {arg0}");
                break;
            }
            line.at += 1;
            line.skip_white();

            // Isolate the argument, which may be 'quoted'.
            let quoted = line.peek() == b'\'';
            let arg_at = line.at + usize::from(quoted);
            let end = if quoted {
                match line.bytes[arg_at..].iter().position(|&b| b == b'\'') {
                    Some(at) => arg_at + at,
                    None => {
                        // SAFETY: a message argument the caller holds as a NUL-terminated string.
                        let arg0 = unsafe { c_str(line.ptr(key_at)) };
                        semsg!("E475: Invalid argument: {arg0}");
                        break;
                    }
                }
            } else {
                let mut at = line.at;
                while !matches!(line.bytes.get(at).copied().unwrap_or(0), 0 | b' ' | b'\t') {
                    at += 1;
                }
                at
            };
            line.at = end;
            if end == arg_at {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(line.ptr(key_at)) };
                semsg!("E417: Missing argument: {arg0}");
                break;
            }
            if end - arg_at > 511 {
                emsg(gettext(c"E423: Illegal argument"));
                break;
            }
            let arg = &line.bytes[arg_at..end];
            if quoted {
                line.at += 1;
            }

            if !unsafe { self.store(&key, arg, line.ptr(key_at)) } {
                break;
            }

            with_group(self.id, |entry| {
                entry.cleared = false;
                // When highlighting has been given for a group, don't
                // link it.
                if !self.init || entry.set & SG_LINK as c_int == 0 {
                    entry.link = 0;
                }
            });
            line.skip_white();
        }
    }

    /// Applies one `key=arg` pair. Answers false to stop the loop, having set
    /// [`Self::error`] and reported it.
    ///
    /// # Safety
    /// See [`do_highlight`]. `key_start` points into the command line.
    unsafe fn store(&mut self, key: &[u8], arg: &[u8], key_start: *const c_char) -> bool {
        // SAFETY: the caller's line, and the editor's own tables.
        match key {
            b"TERM" | b"CTERM" | b"GUI" => unsafe { self.set_attrs(key, arg) },
            b"CTERMFG" | b"CTERMBG" => unsafe { self.set_cterm_color(key, arg, key_start) },
            b"GUIFG" | b"GUIBG" | b"GUISP" => {
                unsafe { self.set_gui_color(key, arg) };
                true
            }
            // Fonts, and the raw terminal codes, are ignored.
            b"FONT" | b"START" | b"STOP" => true,
            b"BLEND" => {
                let blend = if arg == b"NONE" { -1 } else { parse_int(arg) };
                with_group(self.id, |entry| entry.blend = blend);
                true
            }
            _ => {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let key_start = unsafe { c_str(key_start) };
                semsg!("E423: Illegal argument: {key_start}");
                self.error = true;
                false
            }
        }
    }

    /// `term=`/`cterm=`/`gui=`: a comma-separated list of attribute names.
    ///
    /// # Safety
    /// See [`do_highlight`].
    unsafe fn set_attrs(&mut self, key: &[u8], arg: &[u8]) -> bool {
        let mut attr = HlAttrFlags::NONE;
        let mut off = 0;
        while off < arg.len() {
            // Reverse order, as upstream: the longer names have to be tried
            // first, since `underdouble` starts with `under`.
            let found = ATTR_NAMES.iter().rev().find(|(name, _)| {
                let name = name.to_bytes();
                arg[off..].len() >= name.len()
                    && arg[off..off + name.len()].eq_ignore_ascii_case(name)
            });
            let Some(&(name, flag)) = found else {
                semsg!("E418: Illegal value: {}", msg_bytes(arg));
                self.error = true;
                return false;
            };
            if flag.has(HlAttrFlags::UNDERLINE_MASK) {
                // The underline styles share a field.
                attr.clear(HlAttrFlags::UNDERLINE_MASK);
            }
            attr |= flag;
            off += name.count_bytes();
            if arg.get(off) == Some(&b',') {
                off += 1;
            }
        }

        // "term=" is accepted and ignored.
        if key[0] == b'C' {
            if !self.init || group(self.id).set & SG_CTERM as c_int == 0 {
                with_group(self.id, |entry| {
                    if !self.init {
                        entry.set |= SG_CTERM as c_int;
                    }
                    entry.cterm = attr;
                    entry.cterm_bold = false;
                });
            }
        } else if key[0] == b'G' && (!self.init || group(self.id).set & SG_GUI as c_int == 0) {
            with_group(self.id, |entry| {
                if !self.init {
                    entry.set |= SG_GUI as c_int;
                }
                entry.gui = attr;
            });
        }
        true
    }

    /// `ctermfg=`/`ctermbg=`: a number, `fg`/`bg`, or one of the sixteen
    /// colour names.
    ///
    /// # Safety
    /// See [`do_highlight`].
    unsafe fn set_cterm_color(&mut self, key: &[u8], arg: &[u8], key_start: *const c_char) -> bool {
        let foreground = key[5] == b'F';
        if self.init && group(self.id).set & SG_CTERM as c_int != 0 {
            return true;
        }
        // SAFETY: main-thread message and option calls.
        if !self.init {
            with_group(self.id, |entry| entry.set |= SG_CTERM as c_int);
        }
        // Setting the foreground colour undoes a "bold" that was only
        // there to reach a light colour.
        if foreground && group(self.id).cterm_bold {
            with_group(self.id, |entry| {
                entry.cterm.clear(HlAttrFlags::BOLD);
                entry.cterm_bold = false;
            });
        }

        let color = if arg.first().is_some_and(|&b| ascii_isdigit(c_int::from(b))) {
            parse_int(arg)
        } else if arg.eq_ignore_ascii_case(b"fg") {
            if cterm_normal_fg_color.get() == 0 {
                emsg(gettext(c"E419: FG color unknown"));
                self.error = true;
                return false;
            }
            cterm_normal_fg_color.get() - 1
        } else if arg.eq_ignore_ascii_case(b"bg") {
            if cterm_normal_bg_color.get() <= 0 {
                emsg(gettext(c"E420: BG color unknown"));
                self.error = true;
                return false;
            }
            cterm_normal_bg_color.get() - 1
        } else {
            let name = arg.to_vec();
            let name =
                CStr::from_bytes_with_nul(&[name.as_slice(), b"\0"].concat()).map(CStr::to_owned);
            let idx = name.ok().and_then(|name| cterm_color_index(&name));
            let Some(idx) = idx else {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let key_start = unsafe { c_str(key_start) };
                semsg!("E421: Color name or number not recognized: {key_start}");
                self.error = true;
                return false;
            };
            let (color, bold) = lookup_color(idx, foreground);
            // Set or reset bold to get the light foreground colours some
            // terminals (e.g. "linux") only have that way.
            if bold == Some(true) {
                with_group(self.id, |entry| {
                    entry.cterm |= HlAttrFlags::BOLD;
                    entry.cterm_bold = true;
                });
            } else if bold == Some(false) {
                with_group(self.id, |entry| entry.cterm.clear(HlAttrFlags::BOLD));
            }
            color
        };

        // Stored plus one, so that 0 can mean "NONE" (colour -1).
        if foreground {
            with_group(self.id, |entry| entry.cterm_fg = color + 1);
            if self.is_normal_group {
                cterm_normal_fg_color.set(color + 1);
            }
        } else {
            with_group(self.id, |entry| entry.cterm_bg = color + 1);
            if self.is_normal_group {
                cterm_normal_bg_color.set(color + 1);
                unsafe { self.guess_background(color) };
            }
        }
        true
    }

    /// A dark `Normal` background means `'background'` should be `dark`; fix
    /// it if the user has not said otherwise.
    ///
    /// # Safety
    /// Sets an option, which can fire autocommands; main thread only.
    unsafe fn guess_background(&self, color: c_int) {
        // SAFETY: main-thread option calls.
        if ui_rgb_attached() || color < 0 {
            return;
        }
        let dark = if t_colors.get() < 16 {
            Some(color == 0 || color == 4)
        } else if color < 16 {
            // Limit the heuristic to the standard 16 colours.
            Some(color < 7 || color == 8)
        } else {
            None
        };
        let Some(dark) = dark else { return };
        if dark == (unsafe { *p_bg.get() } == b'd' as c_char) || option_was_set(kOptBackground) {
            return;
        }
        set_option_value_give_err(
            kOptBackground,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe {
                        cstr_as_string(if dark {
                            c"dark".as_ptr()
                        } else {
                            c"light".as_ptr()
                        })
                    },
                },
            },
            OptionSetFlags::NONE,
        );
        reset_option_was_set(kOptBackground);
    }

    /// `guifg=`/`guibg=`/`guisp=`.
    ///
    /// # Safety
    /// See [`do_highlight`].
    unsafe fn set_gui_color(&mut self, key: &[u8], arg: &[u8]) {
        if self.init && group(self.id).set & SG_GUI as c_int != 0 {
            return;
        }
        if !self.init {
            with_group(self.id, |entry| entry.set |= SG_GUI as c_int);
        }

        let (color, idx) = if arg == b"NONE" {
            (-1, kColorIdxNone)
        } else {
            let owned = [arg, b"\0"].concat();
            match CStr::from_bytes_with_nul(&owned) {
                Ok(name) => name_to_color(name),
                // An embedded NUL cannot reach here: the parse stopped at it.
                Err(_) => (-1, kColorIdxNone),
            }
        };

        let changed = with_group(self.id, |entry| {
            let (old, old_idx) = match key[3] {
                b'F' => (entry.rgb_fg, entry.rgb_fg_idx),
                b'B' => (entry.rgb_bg, entry.rgb_bg_idx),
                _ => (entry.rgb_sp, entry.rgb_sp_idx),
            };
            match key[3] {
                b'F' => (entry.rgb_fg, entry.rgb_fg_idx) = (color, idx),
                b'B' => (entry.rgb_bg, entry.rgb_bg_idx) = (color, idx),
                _ => (entry.rgb_sp, entry.rgb_sp_idx) = (color, idx),
            }
            color != old || idx != old_idx
        });
        self.did_change = changed;

        if self.is_normal_group {
            let entry = group(self.id);
            match key[3] {
                b'F' => normal_fg.set(entry.rgb_fg),
                b'B' => normal_bg.set(entry.rgb_bg),
                _ => normal_sp.set(entry.rgb_sp),
            }
        }
    }
}

/// `atoi`/`strtol`: leading digits, 0 for anything else.
fn parse_int(arg: &[u8]) -> c_int {
    let text = arg
        .iter()
        .position(|b| !b.is_ascii_digit() && *b != b'-' && *b != b'+')
        .map_or(arg, |at| &arg[..at]);
    core::str::from_utf8(text)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0)
}
