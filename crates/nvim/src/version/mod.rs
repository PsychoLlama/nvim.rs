//! What this build says it is: the `:version` and `:intro` screens, and the
//! predicates (`has("nvim-…")`, `has("patch-…")`, `v:version`) that scripts
//! ask instead of reading them.
//!
//! Nvim was forked from Vim 7.4.160. Vim originated from Stevie version 3.6
//! (Fish disk 217) by GRWalter (Fred).

#![deny(unsafe_op_in_unsafe_fn)]

mod vim_patches;

use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use std::ffi::CString;

use self::vim_patches::VIM_BASELINES;
use crate::api::private::helpers::api_free_object;
use crate::buffer::buf_is_empty;
use crate::charset::vim_strsize;
use crate::drawscreen::screenclear;
use crate::getchar::plain_vgetc;
use crate::grid::{grid_line_flush, grid_line_puts, grid_line_start};
use crate::highlight_group::{HLF_8, syn_id2attr, syn_name2id};
use crate::lua::executor::{kRetObject, nlua_exec};
use crate::main::{
    Columns, Rows, curbuf, curwin, default_gridview, firstwin, got_int, hl_attr_active, msg_col,
    p_ls, p_shm, p_verbose, starting, topframe,
};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::message::{msg_ext_set_kind, msg_putchar, msg_puts};
use crate::os::cshim::gettext;
use crate::os::env::{default_vim_dir, default_vimruntime_dir};
use crate::types::builders::static_cstring;
use crate::types::ui::{kUIMessages, kUIMultigrid};
use crate::types::{
    Arena, Array, Error, OptInt, ShmFlag, exarg_T, kErrorTypeNone, kObjectTypeString, tabpage_T,
};
use crate::ui::ui_has;
use crate::window::{LOWEST_WIN_ID, one_window};

pub const NVIM_VERSION_MAJOR: c_int = 0;
pub const NVIM_VERSION_MINOR: c_int = 12;
pub const NVIM_VERSION_PATCH: c_int = 4;

/// The banner, NUL-terminated for the C-ABI callers that print it
/// (`:version`, `nvim -v`, the intro screen, shada's generator field). The
/// version is `build.rs`'s: the CalVer release tag when HEAD is one, else
/// `dev-<short sha>[-dirty]`, else whatever `$NVIM_RS_VERSION` declared.
pub const LONG_VERSION: &CStr = terminated(concat!("nvim.rs ", env!("NVIM_RS_VERSION"), "\0"));

/// How `:version` describes this build's toolchain: the cargo profile
/// `build.rs` was invoked under, and the `rustc` cargo handed it.
const BUILD_LINE: &CStr = terminated(concat!(
    "Build: ",
    env!("NVIM_RS_PROFILE"),
    ", rustc ",
    env!("NVIM_RS_RUSTC"),
    "\0"
));

/// The vimrc `:version` reports as this build's system-wide one.
const SYS_VIMRC_FILE: &CStr = c"$VIM/sysinit.vim";

/// A `&'static CStr` from a `concat!`-assembled literal that ends in a NUL.
/// Wrong input is a compile error, which is the point: everything here is
/// handed to C.
const fn terminated(text: &'static str) -> &'static CStr {
    match CStr::from_bytes_with_nul(text.as_bytes()) {
        Ok(text) => text,
        Err(_) => panic!("the literal must end in exactly one NUL"),
    }
}

/// `has("nvim-MAJOR[.MINOR[.PATCH]]")`: whether this build is that Nvim
/// release or newer.
///
/// # Safety
/// `version_str` is a NUL-terminated string.
pub unsafe fn has_nvim_version(version_str: *const c_char) -> bool {
    // SAFETY: the caller's obligation.
    let asked = unsafe { CStr::from_ptr(version_str) };
    at_most_this_nvim(asked.to_bytes())
}

/// Whether `MAJOR[.MINOR[.PATCH]]` names this Nvim release or an older one.
///
/// An absent component reads as 0, and each present one is read up to its
/// first non-digit -- C's `atoi`, which is what this replaced, stopped
/// there. A component that does not *start* with a digit is a hard no, so
/// `0.` and `0.x` are false while `0.12.4-dev` is true.
fn at_most_this_nvim(asked: &[u8]) -> bool {
    /// An absent component, or one that leads with digits.
    fn component(text: Option<&[u8]>) -> Option<c_int> {
        match text {
            None => Some(0),
            Some(text) => leading_number(text),
        }
    }

    let mut parts = asked.splitn(3, |&byte| byte == b'.');
    let (Some(major), Some(minor), Some(patch)) = (
        parts.next().and_then(leading_number),
        component(parts.next()),
        component(parts.next()),
    ) else {
        return false;
    };
    (major, minor, patch) <= (NVIM_VERSION_MAJOR, NVIM_VERSION_MINOR, NVIM_VERSION_PATCH)
}

/// The value of `text`'s leading run of digits, or `None` when it has none.
/// Saturates rather than wrapping; the C overflowed.
fn leading_number(text: &[u8]) -> Option<c_int> {
    let digits = text.iter().take_while(|byte| byte.is_ascii_digit());
    let mut value: Option<c_int> = None;
    for digit in digits {
        let so_far = value.unwrap_or(0);
        value = Some(
            so_far
                .saturating_mul(10)
                .saturating_add((digit - b'0') as c_int),
        );
    }
    value
}

/// The oldest Vim this port claims, as `v:version` spells a version:
/// `MAJOR * 100 + MINOR`.
pub fn min_vim_version() -> c_int {
    VIM_BASELINES[0].number
}

/// The same version as `:version` writes it: `8.1`. The swap file's header
/// records it, which is what makes an nvim swap file readable by the Vim it
/// claims compatibility with.
pub fn min_vim_version_name() -> &'static CStr {
    VIM_BASELINES[0].name
}

/// The newest Vim patch merged into that oldest line -- the four digits
/// `v:versionlong` carries below the version itself.
pub fn highest_patch() -> c_int {
    VIM_BASELINES[0].patches[0]
}

/// `has("patch-M.m.PPPP")` and `has("patchNNNN")`: whether Vim patch `n` of
/// the `M * 100 + m` release line made it into this build.
///
/// `major_minor_version` of 0 asks about the baseline line (the bare
/// `patchNNNN` spelling). A line older than the baseline is covered
/// wholesale; one this port never tracked is a flat no.
pub fn has_vim_patch(n: c_int, major_minor_version: c_int) -> bool {
    let baseline = if major_minor_version > 0 {
        if major_minor_version < min_vim_version() {
            return true;
        }
        match VIM_BASELINES
            .iter()
            .find(|baseline| baseline.number == major_minor_version)
        {
            Some(baseline) => baseline,
            None => return false,
        }
    } else {
        &VIM_BASELINES[0]
    };
    // The lists descend, so the search's comparison runs the other way round.
    baseline
        .patches
        .binary_search_by(|patch| n.cmp(patch))
        .is_ok()
}

/// `:version`. Vim's `:version 9.99` ("this script needs Vim 9.99") is
/// silently ignored rather than printing anything.
///
/// # Safety
/// `eap` is a live `exarg_T`.
pub unsafe fn ex_version(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation; `arg` is NUL-terminated.
    unsafe {
        if *(*eap).arg != 0 {
            return;
        }
        // Start the banner below the ":version" the user typed. The message
        // UI lays its own messages out, so it needs no help.
        if !ui_has(kUIMessages) {
            msg_putchar(b'\n' as c_int);
        }
        list_version();
    }
}

/// Print `s`, moving to the next line first if it would otherwise wrap, and
/// bracketing it when `wrap` marks it as the current item.
///
/// # Safety
/// The message machinery must be usable.
unsafe fn version_msg_wrap(s: &CStr, wrap: bool) {
    // SAFETY: `s` is NUL-terminated by construction.
    unsafe {
        let len = vim_strsize(s.as_ptr()) + if wrap { 2 } else { 0 };
        if !got_int.get()
            && len < Columns.get()
            && msg_col.get() + len >= Columns.get()
            && !s.to_bytes().starts_with(b"\n")
        {
            msg_putchar(b'\n' as c_int);
        }
        if got_int.get() {
            return;
        }
        if wrap {
            msg_puts(c"[".as_ptr());
        }
        msg_puts(s.as_ptr());
        if wrap {
            msg_puts(c"]".as_ptr());
        }
    }
}

/// [`version_msg_wrap`] for a line that is not the current item.
///
/// # Safety
/// The message machinery must be usable.
unsafe fn version_msg(s: &CStr) {
    // SAFETY: the caller's obligation.
    unsafe { version_msg_wrap(s, false) }
}

/// Print `items` in as many columns as `'columns'` affords, filling column
/// by column, with `items[current]` in brackets. `:args` lists the argument
/// list this way, with the current file bracketed.
///
/// # Safety
/// The message machinery must be usable.
pub unsafe fn list_in_columns(items: &[&CStr], current: c_int) {
    // SAFETY: every item is NUL-terminated by construction.
    unsafe {
        let count = items.len() as c_int;
        // The widest item, plus the gap that separates two columns.
        let width = 1 + items
            .iter()
            .enumerate()
            .map(|(i, item)| vim_strsize(item.as_ptr()) + if i as c_int == current { 2 } else { 0 })
            .max()
            .unwrap_or(0);

        // Too narrow even for one column: one item per line, wrapped.
        if Columns.get() < width {
            for (i, item) in items.iter().enumerate() {
                version_msg_wrap(item, i as c_int == current);
                if msg_col.get() > 0 && (i as c_int) < count - 1 {
                    msg_putchar(b'\n' as c_int);
                }
            }
            return;
        }

        let ncol = (Columns.get() + 1) / width;
        let nrow = count / ncol + if count % ncol != 0 { 1 } else { 0 };
        let mut cur_row = 1;
        for i in 0..nrow * ncol {
            if got_int.get() {
                break;
            }
            // Filled top-to-bottom, so the item printed here is the one
            // `nrow` rows down the previous column.
            let idx = i / ncol + i % ncol * nrow;
            let last_col = (i + 1) % ncol == 0;
            let Some(item) = items.get(idx as usize) else {
                // A hole in the last column: only the row break is owed.
                if msg_col.get() > 0 {
                    if cur_row < nrow {
                        msg_putchar(b'\n' as c_int);
                    }
                    cur_row += 1;
                }
                continue;
            };
            if idx == current {
                msg_putchar(b'[' as c_int);
            }
            msg_puts(item.as_ptr());
            if idx == current {
                msg_putchar(b']' as c_int);
            }
            if last_col {
                if msg_col.get() > 0 && cur_row < nrow {
                    msg_putchar(b'\n' as c_int);
                }
                cur_row += 1;
            } else {
                while msg_col.get() % width != 0 {
                    msg_putchar(b' ' as c_int);
                }
            }
        }
    }
}

/// Print the Lua runtime's own version string, which only it knows.
///
/// # Safety
/// The Lua state and the message machinery must be usable.
pub unsafe fn list_lua_version() {
    const CODE: &CStr = c"return ((jit and jit.version) and jit.version or _VERSION)";

    // SAFETY: the caller's obligation. `CODE` is borrowed, not owned, by the
    // `String_0`; `nlua_exec` only reads it.
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let ret = nlua_exec(
            static_cstring(CODE),
            ptr::null(),
            Array {
                size: 0,
                capacity: 0,
                items: ptr::null_mut(),
            },
            kRetObject,
            ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        debug_assert!(err.type_0 == kErrorTypeNone, "a literal chunk cannot fail");
        // Not a debug assertion: the union field read below depends on it.
        assert!(ret.type_0 == kObjectTypeString, "_VERSION is a string");
        msg_puts(ret.data.string.data);
        api_free_object(ret);
    }
}

/// The `:version` screen. `nvim -v` prints the same thing, and `nvim -V1 -v`
/// (or `:verbose version`) adds the build and path details.
///
/// # Safety
/// The message machinery must be usable.
pub unsafe fn list_version() {
    // SAFETY: the caller's obligation.
    unsafe {
        msg_ext_set_kind(c"list_cmd".as_ptr());
        msg_puts(LONG_VERSION.as_ptr());
        msg_putchar(b'\n' as c_int);
        // The Nvim release this port tracks -- the version every
        // compatibility surface (`has('nvim-…')`, `v:version`, the API
        // metadata) answers with.
        let compat = CString::new(format!(
            "NVIM v{NVIM_VERSION_MAJOR}.{NVIM_VERSION_MINOR}.{NVIM_VERSION_PATCH} compatible"
        ))
        .expect("version numbers hold no NUL");
        msg_puts(compat.as_ptr());
        msg_putchar(b'\n' as c_int);
        list_lua_version();

        if p_verbose.get() > 0 as OptInt {
            msg_putchar(b'\n' as c_int);
            msg_puts(BUILD_LINE.as_ptr());
            msg_putchar(b'\n' as c_int);
            msg_puts(c"Vim versions: ".as_ptr());
            for (i, baseline) in VIM_BASELINES.iter().enumerate() {
                if i != 0 {
                    msg_puts(c", ".as_ptr());
                }
                msg_puts(baseline.name.as_ptr());
            }
            version_msg(c"\n");
            version_msg(translate(c"   system vimrc file: \""));
            version_msg(SYS_VIMRC_FILE);
            version_msg(c"\"\n");
            for (label, dir) in [
                (c"  fall-back for $VIM: \"", default_vim_dir.get()),
                (c" f-b for $VIMRUNTIME: \"", default_vimruntime_dir.get()),
            ] {
                if *dir == 0 {
                    continue;
                }
                version_msg(translate(label));
                version_msg(CStr::from_ptr(dir));
                version_msg(c"\"\n");
            }
        }

        version_msg(if p_verbose.get() > 0 as OptInt {
            c"\nRun :checkhealth for more info"
        } else if starting.get() != 0 {
            c"\nRun \"nvim -V1 -v\" for more info"
        } else {
            c"\nRun \":verbose version\" for more info"
        });
    }
}

/// Whether the intro screen is still what the window shows: an untouched
/// first buffer in the first window, and `'shortmess'` permitting.
///
/// # Safety
/// The editor's globals must be live.
pub unsafe fn may_show_intro() -> bool {
    // SAFETY: the caller's obligation.
    unsafe {
        buf_is_empty(curbuf.get())
            && (*curbuf.get()).b_fname.is_null()
            && (*curbuf.get()).handle == 1
            && (*curwin.get()).handle == LOWEST_WIN_ID as c_int
            && one_window(curwin.get(), ptr::null_mut::<tabpage_T>())
            && !ShmFlag::INTRO.is_in(CStr::from_ptr(p_shm.get()))
    }
}

/// The intro screen, top to bottom. The first three lines are the logo, and
/// [`LONG_VERSION`] is spliced in where the empty line ends the logo.
///
/// Every line is run through `gettext`; [`NEWS_TEMPLATE`] additionally takes
/// the Nvim release whose `:help news` the bundled runtime carries.
const INTRO_LINES: [&CStr; 18] = [
    c"│ ╲ ││",
    c"││╲╲││",
    c"││ ╲ │",
    c"",
    LONG_VERSION,
    c"────────────────────────────────────────────",
    c"Nvim is open source and freely distributable",
    c"https://neovim.io/#chat",
    c"────────────────────────────────────────────",
    c"type  :help nvim<Enter>     if you are new! ",
    c"type  :checkhealth<Enter>   to optimize Nvim",
    c"type  :q<Enter>             to exit         ",
    c"type  :help<Enter>          for help        ",
    c"────────────────────────────────────────────",
    NEWS_TEMPLATE,
    c"────────────────────────────────────────────",
    c"Help poor children in Uganda!",
    c"type  :help Kuwasha<Enter>  for information ",
];

/// The one intro line with substitutions: the two `%s` are the major and
/// minor of the Nvim release the bundled documentation describes.
const NEWS_TEMPLATE: &CStr = c"type  :help news<Enter>     for v%s.%s notes ";

/// The translation of `msg`, or `msg` itself when there is none.
///
/// `gettext` answers with a pointer into the loaded message catalogue, which
/// nvim never unloads, so the result outlives every caller here.
fn translate(msg: &'static CStr) -> &'static CStr {
    // SAFETY: `msg` is NUL-terminated, and so is anything gettext returns.
    unsafe { CStr::from_ptr(gettext(msg.as_ptr())) }
}

/// [`NEWS_TEMPLATE`], translated and filled in.
///
/// The C reached for `snprintf` on a string a translator supplied; splicing
/// on `%s` cannot be talked into reading arguments that were never passed.
fn news_line() -> CString {
    let major = NVIM_VERSION_MAJOR.to_string();
    let minor = NVIM_VERSION_MINOR.to_string();
    let mut args = [major.as_bytes(), minor.as_bytes()].into_iter();

    let mut rest = translate(NEWS_TEMPLATE).to_bytes();
    let mut line = Vec::with_capacity(rest.len());
    while let Some(at) = rest.windows(2).position(|pair| pair == b"%s") {
        line.extend_from_slice(&rest[..at]);
        line.extend_from_slice(args.next().unwrap_or_default());
        rest = &rest[at + 2..];
    }
    line.extend_from_slice(rest);
    CString::new(line).expect("the catalogue holds no NUL mid-string")
}

/// Draw the intro screen, unless the window is too small to hold it --
/// `:intro` (`colon`) asks for it regardless.
///
/// # Safety
/// The grid must be ready to draw on.
pub unsafe fn intro_message(colon: bool) {
    // SAFETY: the caller's obligation.
    unsafe {
        // Centre the block vertically, ignoring the line the empty entry
        // above the version costs.
        let mut blanklines = Rows.get() - (INTRO_LINES.len() as c_int - 1);
        if p_ls.get() > 1 as OptInt {
            blanklines -= Rows.get() - (*topframe.get()).fr_height;
        }
        let top = blanklines.max(0) / 2;
        if !(top >= 2 && Columns.get() >= 50 || colon) {
            return;
        }

        let news = news_line();
        for (i, line) in INTRO_LINES.iter().enumerate() {
            // gettext("") answers with the catalogue's own header, so blank
            // lines stay untranslated.
            let mesg = if *line == NEWS_TEMPLATE {
                news.as_c_str()
            } else if line.is_empty() {
                c""
            } else {
                translate(line)
            };
            let row = top + i as c_int;
            if !mesg.is_empty() && row < Rows.get() - 1 {
                do_intro_line(row, mesg, colon, i < 3);
            }
        }
    }
}

/// Draw one centred intro line, highlighting what it recognises: the logo's
/// diagonals, the horizontal rules, the version banner, and the `:command`
/// and `<key>` mentions in the instructions.
///
/// # Safety
/// The grid must be ready to draw on.
unsafe fn do_intro_line(row: c_int, mesg: &CStr, colon: bool, is_logo: bool) {
    let text = mesg.to_bytes();
    // SAFETY: `mesg` is NUL-terminated and `text.len()` bytes long, so every
    // pointer below stays within it.
    unsafe {
        let mut col = ((Columns.get() - vim_strsize(mesg.as_ptr())) / 2).max(0);
        grid_line_start(
            if !colon && ui_has(kUIMultigrid) {
                &raw mut (*firstwin.get()).w_grid
            } else {
                default_gridview.ptr()
            },
            row,
        );
        let byte_at = |at: usize| mesg.as_ptr().add(at);

        let attr_of = |group: &CStr| syn_id2attr(syn_name2id(group.as_ptr()));

        if is_logo {
            // The logo's leading strokes are the frame, everything from the
            // first diagonal on is the letter.
            let (frame_attr, letter_attr) = (attr_of(c"Special"), attr_of(c"String"));
            let mut seen_diagonal = false;
            let mut at = 0;
            while at < text.len() {
                let clen = utfc_ptr2len(byte_at(at));
                let mut attr = 0;
                if text[at] >= 0x80 {
                    seen_diagonal |= clen == 3 && utf_ptr2char(byte_at(at)) == 0x2572;
                    attr = if seen_diagonal {
                        letter_attr
                    } else {
                        frame_attr
                    };
                }
                col += grid_line_puts(col, byte_at(at), clen, attr);
                at += clen as usize;
            }
            grid_line_flush();
            return;
        }

        // Two lines are one colour throughout: the banner -- matched on the
        // lowercase "nvim" of `nvim.rs …`, which no other intro line starts
        // with ("Nvim is open source…" capitalizes it) -- and the horizontal
        // rules, drawn one U+2500 at a time.
        let is_sep = utfc_ptr2len(mesg.as_ptr()) == 3 && utf_ptr2char(mesg.as_ptr()) == 0x2500;
        if text.starts_with(b"nvim") || is_sep {
            let clen = if is_sep { 3 } else { 1 };
            let attr = attr_of(if is_sep { c"NonText" } else { c"String" });
            let mut at = 0;
            while at < text.len() {
                col += grid_line_puts(col, byte_at(at), clen, attr);
                at += clen as usize;
            }
            grid_line_flush();
            return;
        }

        // The rest is prose with `:command<Key>` mentions in it. Each pass
        // takes the run up to the next `<`, or the whole `<…>` that follows
        // a `>`, so the two get their own highlights.
        let mut at = 0;
        while at < text.len() {
            let mut len = 0;
            while at + len < text.len()
                && (len == 0 || (text[at + len] != b'<' && text[at + len - 1] != b'>'))
            {
                len += utfc_ptr2len(byte_at(at + len)) as usize;
            }
            let special_attr = *hl_attr_active.get().add(HLF_8 as usize);
            let colon_at = text[at..at + len].iter().position(|&byte| byte == b':');
            match colon_at {
                _ if text[at] == b'<' => {
                    col += grid_line_puts(col, byte_at(at), len as c_int, special_attr);
                }
                // `:command` immediately before a `<Key>`: the colon reads as
                // punctuation, the command as an identifier.
                Some(colon_at) if text.get(at + len) == Some(&b'<') => {
                    col += grid_line_puts(col, byte_at(at), colon_at as c_int, 0);
                    col += grid_line_puts(col, byte_at(at + colon_at), 1, special_attr);
                    col += grid_line_puts(
                        col,
                        byte_at(at + colon_at + 1),
                        (len - colon_at - 1) as c_int,
                        attr_of(c"Identifier"),
                    );
                }
                _ => col += grid_line_puts(col, byte_at(at), len as c_int, 0),
            }
            at += len;
        }
        grid_line_flush();
    }
}

/// `:intro` -- the intro screen on demand, until a key is pressed.
///
/// # Safety
/// The editor's globals must be live.
pub unsafe fn ex_intro(_eap: *mut exarg_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        screenclear();
        intro_message(true);
        plain_vgetc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nvim_version_omits_what_it_does_not_care_about() {
        assert!(at_most_this_nvim(b"0"));
        assert!(at_most_this_nvim(b"0.12"));
        assert!(at_most_this_nvim(b"0.12.4"));
        assert!(!at_most_this_nvim(b"0.12.5"));
        assert!(!at_most_this_nvim(b"0.13"));
        assert!(!at_most_this_nvim(b"1"));
    }

    #[test]
    fn nvim_version_reads_each_component_as_atoi_did() {
        // Digits run out where the component does; the rest is ignored.
        assert!(at_most_this_nvim(b"0.12.4-dev"));
        assert!(at_most_this_nvim(b"00.012.0004"));
        // But a component that leads with anything else is not a version.
        assert!(!at_most_this_nvim(b""));
        assert!(!at_most_this_nvim(b"0."));
        assert!(!at_most_this_nvim(b"0.x"));
        assert!(!at_most_this_nvim(b" 0.12"));
        // Absurdly long numbers saturate, so they read as newer, not older.
        assert!(!at_most_this_nvim(b"99999999999999999999.0.0"));
    }

    #[test]
    fn vim_patches_are_answered_from_their_own_baseline() {
        // Vim 8.1.2000 is in; 8.1's 2331 patches stop well short of 9999.
        assert!(has_vim_patch(2000, 801));
        assert!(!has_vim_patch(9999, 801));
        // Patch 2 is in 9.1 but not 9.2, and the bare spelling asks 8.1.
        assert!(has_vim_patch(2, 901));
        assert!(!has_vim_patch(2, 902));
        assert!(has_vim_patch(2, 0));
        // Anything older than the baseline counts; anything untracked does not.
        assert!(has_vim_patch(9999, 800));
        assert!(!has_vim_patch(1, 903));
    }
}
