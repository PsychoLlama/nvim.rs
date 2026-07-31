//! Which terminal is this, and what does its description get wrong?
//!
//! terminfo says what a terminal *claims*. Real terminals lie: they ship
//! descriptions that predate features they have since grown, they inherit
//! entries from an ancestor they no longer resemble, and they sit behind
//! multiplexers that pass through less than the terminal underneath. So
//! before the TUI trusts a description it runs it past the accumulated
//! knowledge here.
//!
//! Two passes, in order, both driven by the same [`Terminal`] identification:
//!
//! - [`patch_terminfo_bugs`] corrects what the description says wrongly.
//! - [`augment_terminfo`] adds what it has no way to say at all -- terminfo
//!   has no capability for truecolour, cursor colour or focus reporting.
//!
//! Identifying the terminal is guesswork built from `$TERM` and a handful of
//! environment variables terminals set about themselves. It happens once, in
//! [`Terminal::identify`], rather than being re-derived per question.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::memory::{arena_memdupz, xfree};
use crate::src::nvim::os::env::{os_env_exists, os_getenv};
use crate::src::nvim::tui::terminfo::caps::{
    TerminfoDef, kTerm_carriage_return, kTerm_cursor_invisible, kTerm_cursor_normal,
    kTerm_enter_ca_mode, kTerm_enter_italics_mode, kTerm_exit_ca_mode, kTerm_from_status_line,
    kTerm_parm_down_cursor, kTerm_parm_left_cursor, kTerm_parm_right_cursor, kTerm_parm_up_cursor,
    kTerm_reset_cursor_color, kTerm_reset_cursor_style, kTerm_set_a_background,
    kTerm_set_a_foreground, kTerm_set_cursor_color, kTerm_set_cursor_style, kTerm_set_lr_margin,
    kTerm_set_rgb_background, kTerm_set_rgb_foreground, kTerm_set_underline_style,
    kTerm_to_status_line,
};
use crate::src::nvim::tui::terminfo::{is_term_family, terminfo_is_bsd_console};
use crate::src::nvim::types::{Arena, KeyEncoding, TerminfoEntry, TerminfoExt};
use core::ffi::{CStr, c_char, c_int};

/// The two key-encoding schemes this module chooses between. `KeyEncoding`
/// is a bare integer alias, so name the values rather than spelling them.
const KEY_ENCODING_LEGACY: KeyEncoding = 0;
const KEY_ENCODING_XTERM: KeyEncoding = 2;

/// The suffix a linux console's `cursor_normal` carries when it also resets
/// the cursor to the "default" shape, and the matching one for
/// `cursor_invisible`. nvim drives cursor shape itself, so the trailing
/// reset has to come off or every shape change is immediately undone.
const LINUX_SET_SHAPE_0: &[u8] = b"\x1b[?0c";
const LINUX_SET_SHAPE_1: &[u8] = b"\x1b[?1c";

/// The prefix some `cursor_normal` entries carry, which turns cursor blink
/// back on as a side effect of showing the cursor.
const SHOW_CURSOR_BLINK_PREFIX: &[u8] = b"\x1b[?12l";

// Descriptions that predate 256-colour support get these grafted on. The
// colon-separated forms are the ISO 8613-6 spelling; only terminals known to
// parse it get them, since the rest render the parameters as text.
const XTERM_SETAF_256_COLON: &CStr =
    c"\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38:5:%p1%d%;m";
const XTERM_SETAB_256_COLON: &CStr =
    c"\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48:5:%p1%d%;m";
const XTERM_SETAF_256: &CStr = c"\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m";
const XTERM_SETAB_256: &CStr =
    c"\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m";
const XTERM_SETAF_16: &CStr = c"\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e39%;m";
const XTERM_SETAB_16: &CStr = c"\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e39%;m";

// ------------------------------------------------------------------ helpers

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// The capability in `slot` as bytes, or `None` when the description has no
/// entry. The result borrows the capability's own storage, not `ti`, so the
/// caller stays free to write the slot back.
fn cap(ti: &TerminfoEntry, slot: TerminfoDef) -> Option<&'static [u8]> {
    let p = ti.defs[slot as usize];
    // SAFETY: a non-null `defs` entry is a NUL-terminated string owned by
    // either the arena the description was parsed into or by static storage.
    // Both outlive the entry itself, which is what `'static` stands in for
    // here -- the arena is freed only when the description is discarded.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_bytes())
}

fn set(ti: &mut TerminfoEntry, slot: TerminfoDef, val: &'static CStr) {
    ti.defs[slot as usize] = val.as_ptr();
}

fn set_if_empty(ti: &mut TerminfoEntry, slot: TerminfoDef, val: &'static CStr) {
    if ti.defs[slot as usize].is_null() {
        set(ti, slot, val);
    }
}

fn clear(ti: &mut TerminfoEntry, slot: TerminfoDef) {
    ti.defs[slot as usize] = core::ptr::null();
}

/// Copy `bytes` into `arena` as a NUL-terminated string.
///
/// A capability trimmed at the end is not a suffix of the original, so it
/// needs storage of its own. The arena the description was parsed into is
/// exactly that: it lives and dies with the description.
///
/// # Safety
/// `arena` must point to a live arena that outlives every use of the result.
unsafe fn arena_dup(arena: *mut Arena, bytes: &[u8]) -> *const c_char {
    unsafe { arena_memdupz(arena, bytes.as_ptr().cast(), bytes.len()) }
}

/// The value of `$name`, or `None` when it is unset.
fn env(name: &CStr) -> Option<Vec<u8>> {
    // SAFETY: `os_getenv` takes a NUL-terminated name and returns either
    // null or a freshly allocated NUL-terminated string that becomes ours to
    // free -- which is why the value is copied out before the free.
    unsafe {
        let p = os_getenv(name.as_ptr());
        if p.is_null() {
            return None;
        }
        let owned = CStr::from_ptr(p).to_bytes().to_vec();
        xfree(p.cast());
        Some(owned)
    }
}

/// Is `$name` set to something non-empty?
fn env_is_set(name: &CStr) -> bool {
    // SAFETY: the name is NUL-terminated; `true` asks it to ignore "".
    unsafe { os_env_exists(name.as_ptr(), true) }
}

/// The leading decimal digits of a self-reported version, 0 if there are
/// none. This is `strtol` minus the bases and sign handling no caller used.
fn version_of(v: Option<&[u8]>) -> c_int {
    let bytes = match v {
        Some(b) => b,
        None => return 0,
    };
    let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    core::str::from_utf8(&bytes[..digits])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ------------------------------------------------------------ identification

/// Everything the TUI worked out about the terminal it is talking to, from
/// `$TERM` and the variables terminals set about themselves.
///
/// Every field is a guess. `$TERM` is routinely wrong -- terminals emulate
/// each other and multiplexers rewrite it -- which is why several of these
/// are "X pretending to be an xterm" rather than plain "is X".
pub struct Terminal {
    pub xterm: bool,
    pub hterm: bool,
    pub kitty: bool,
    pub linuxvt: bool,
    pub bsdvt: bool,
    pub rxvt: bool,
    pub teraterm: bool,
    pub putty: bool,
    pub screen: bool,
    pub tmux: bool,
    pub st: bool,
    pub gnome: bool,
    pub iterm: bool,
    pub alacritty: bool,
    pub foot: bool,
    pub cygwin: bool,
    pub ghostty: bool,
    pub dtterm: bool,
    /// The Interix console, whose description omits `carriage_return`.
    pub interix: bool,
    pub nsterm: bool,

    /// An iTerm/GNOME/MATE terminal whose `$TERM` says "xterm". Each is
    /// found by a variable the terminal itself sets, since `$TERM` is no
    /// help, and each supports more than the xterm description admits.
    pub iterm_pretending_xterm: bool,
    pub gnome_pretending_xterm: bool,
    pub mate_pretending_xterm: bool,
    /// A real xterm, which announces itself in `$XTERM_VERSION`. The BSD
    /// consoles claim to be xterms and must not qualify.
    pub true_xterm: bool,
    /// Whether `$XTERM_VERSION` is set at all, which the linux console uses
    /// as a hint that something more capable is in the way.
    pub has_xterm_version: bool,

    /// Terminals that wrap to the next line the instant the last column is
    /// written rather than waiting for the next character. The drawing code
    /// must never write that cell directly.
    pub wraps_after_last_column: bool,

    /// Whether `$TERM` itself mentions 256 colours.
    pub term_says_256: bool,

    /// `$VTE_VERSION` and `$KONSOLE_VERSION`, 0 when absent. Both gate
    /// features these two grew without their descriptions keeping up.
    pub vte_version: c_int,
    pub konsole_version: c_int,
    /// WezTerm's `$TERM_PROGRAM_VERSION`, which is a date -- hence the
    /// string comparison rather than a number.
    pub wezterm_version: Option<Vec<u8>>,
    /// `$COLORTERM`. Its mere presence means at least 16 colours; two
    /// specific values claim truecolour.
    pub colorterm: Option<Vec<u8>>,
}

impl Terminal {
    /// Work out which terminal `term` -- the `$TERM` nvim resolved -- names.
    pub fn identify(term: Option<&CStr>) -> Self {
        let name = term.map(CStr::to_bytes).unwrap_or(b"");
        let family = |f: &[u8]| is_term_family(name, f);
        let has =
            |v: &Option<Vec<u8>>, needle: &[u8]| v.as_deref().is_some_and(|b| contains(b, needle));

        let colorterm = env(c"COLORTERM");
        let term_program = env(c"TERM_PROGRAM");
        let term_program_version = env(c"TERM_PROGRAM_VERSION");
        let has_xterm_version = env(c"XTERM_VERSION").is_some();

        let iterm_env = has(&term_program, b"iTerm.app");
        let nsterm = has(&term_program, b"Apple_Terminal") || family(b"nsterm");
        // Constant `false` off the BSDs; kept so a BSD build can switch the
        // workarounds it guards back on.
        let bsdvt = terminfo_is_bsd_console(term.map_or(core::ptr::null(), CStr::as_ptr));

        // An xterm by description, or Apple's Terminal, which is close
        // enough that the same workarounds apply.
        let xterm = family(b"xterm") || nsterm;

        let konsole = family(b"konsole")
            || env_is_set(c"KONSOLE_PROFILE_NAME")
            || env_is_set(c"KONSOLE_DBUS_SESSION");
        let cygwin = family(b"cygwin");
        let interix = family(b"interix");

        Terminal {
            xterm,
            hterm: family(b"hterm"),
            kitty: family(b"xterm-kitty"),
            linuxvt: family(b"linux"),
            bsdvt,
            rxvt: family(b"rxvt"),
            teraterm: family(b"teraterm"),
            putty: family(b"putty"),
            screen: family(b"screen"),
            tmux: family(b"tmux") || env_is_set(c"TMUX"),
            st: family(b"st"),
            gnome: family(b"gnome") || family(b"vte"),
            iterm: family(b"iterm")
                || family(b"iterm2")
                || family(b"iTerm.app")
                || family(b"iTerm2.app"),
            alacritty: family(b"alacritty"),
            foot: family(b"foot"),
            cygwin,
            ghostty: family(b"xterm-ghostty"),
            dtterm: family(b"dtterm"),
            interix,
            nsterm,

            iterm_pretending_xterm: xterm && iterm_env,
            gnome_pretending_xterm: xterm && has(&colorterm, b"gnome-terminal"),
            mate_pretending_xterm: xterm && has(&colorterm, b"mate-terminal"),
            true_xterm: xterm && has_xterm_version && !bsdvt,
            has_xterm_version,

            wraps_after_last_column: family(b"conemu") || cygwin || family(b"win32con") || interix,
            term_says_256: contains(name, b"256"),

            vte_version: version_of(env(c"VTE_VERSION").as_deref()),
            // Konsole only started reporting a version late, so a Konsole
            // with none counts as version 1 -- enough for the "is it Konsole
            // at all" tests, low enough to fail every real version gate.
            konsole_version: match env(c"KONSOLE_VERSION") {
                Some(v) => version_of(Some(&v)),
                None if konsole => 1,
                None => 0,
            },
            wezterm_version: (term_program.as_deref() == Some(b"WezTerm"))
                .then_some(term_program_version)
                .flatten(),
            colorterm,
        }
    }

    /// Does this terminal support 24-bit colour?
    ///
    /// Either `$COLORTERM` says so, or the description carries the `Tc`/`RGB`
    /// extension, or it has both RGB setter capabilities.
    pub fn has_truecolor(&self, ti: &TerminfoEntry) -> bool {
        if matches!(self.colorterm.as_deref(), Some(b"truecolor" | b"24bit")) {
            return true;
        }
        ti.has_Tc_or_RGB
            || (cap(ti, kTerm_set_rgb_foreground).is_some()
                && cap(ti, kTerm_set_rgb_background).is_some())
    }
}

// ------------------------------------------------------------------ patching

/// Correct the description's outright errors and omissions, in place.
///
/// # Safety
/// `arena` must point to a live arena outliving every use of `ti`'s
/// capability pointers.
pub unsafe fn patch_terminfo_bugs(ti: &mut TerminfoEntry, arena: *mut Arena, t: &Terminal) {
    // A `cursor_normal` that starts by re-enabling blink, or (on the linux
    // console) ends by resetting the shape, fights nvim's own cursor
    // handling. Trim both. Dropping the prefix leaves a suffix of the
    // original, still NUL-terminated where it stands; dropping the suffix
    // does not, so that one needs a copy.
    let normal = ti.defs[kTerm_cursor_normal as usize];
    if !normal.is_null() {
        // SAFETY: non-null capability pointers are NUL-terminated strings.
        let full = unsafe { CStr::from_ptr(normal) }.to_bytes();
        let mut bytes = full;
        let mut patched = normal;
        if let Some(rest) = full.strip_prefix(SHOW_CURSOR_BLINK_PREFIX) {
            bytes = rest;
            patched = rest.as_ptr().cast();
        }
        if t.linuxvt
            && let Some(trimmed) = bytes.strip_suffix(LINUX_SET_SHAPE_0)
        {
            // SAFETY: `arena` is the caller's live terminfo arena.
            patched = unsafe { arena_dup(arena, trimmed) };
        }
        ti.defs[kTerm_cursor_normal as usize] = patched;
    }
    let invisible = ti.defs[kTerm_cursor_invisible as usize];
    if !invisible.is_null() && t.linuxvt {
        // SAFETY: as above.
        let bytes = unsafe { CStr::from_ptr(invisible) }.to_bytes();
        if let Some(trimmed) = bytes.strip_suffix(LINUX_SET_SHAPE_1) {
            // SAFETY: as above.
            ti.defs[kTerm_cursor_invisible as usize] = unsafe { arena_dup(arena, trimmed) };
        }
    }

    // Multiplexers and kitty advertise back-colour-erase but do not honour
    // it, leaving cleared regions painted in the wrong colour.
    if t.tmux || t.screen || t.kitty {
        ti.bce = false;
    }

    // Status-line, italics and margin capabilities descriptions routinely omit.
    if t.xterm || t.hterm {
        if !t.hterm {
            // hterm has no status line; leaving these unset is what stops
            // nvim trying to set a title it cannot set.
            set_if_empty(ti, kTerm_to_status_line, c"\x1b]0;");
            set_if_empty(ti, kTerm_from_status_line, c"\x07");
        }
        set_if_empty(ti, kTerm_enter_italics_mode, c"\x1b[3m");
        set_if_empty(ti, kTerm_set_lr_margin, c"\x1b[%i%p1%d;%p2%ds");
    } else if t.rxvt {
        set_if_empty(ti, kTerm_enter_italics_mode, c"\x1b[3m");
        set_if_empty(ti, kTerm_to_status_line, c"\x1b]2");
        set_if_empty(ti, kTerm_from_status_line, c"\x07");
        // rxvt's own alternate-screen entries do not save the cursor.
        set(ti, kTerm_enter_ca_mode, c"\x1b[?1049h");
        set(ti, kTerm_exit_ca_mode, c"\x1b[?1049l");
    } else if t.screen {
        set_if_empty(ti, kTerm_to_status_line, c"\x1b_");
        set_if_empty(ti, kTerm_from_status_line, c"\x1b\\");
    } else if t.tmux {
        // Deliberately not merged with the `screen` arm above: screen
        // running inside tmux matches both, and taking the `screen` arm
        // first is what leaves italics unset there. Merging the two and
        // testing `tmux` inside gives screen-in-tmux italics it has never
        // had.
        set_if_empty(ti, kTerm_to_status_line, c"\x1b_");
        set_if_empty(ti, kTerm_from_status_line, c"\x1b\\");
        set_if_empty(ti, kTerm_enter_italics_mode, c"\x1b[3m");
    } else if t.interix {
        set_if_empty(ti, kTerm_carriage_return, c"\r");
    } else if t.linuxvt {
        set_if_empty(ti, kTerm_parm_up_cursor, c"\x1b[%p1%dA");
        set_if_empty(ti, kTerm_parm_down_cursor, c"\x1b[%p1%dB");
        set_if_empty(ti, kTerm_parm_right_cursor, c"\x1b[%p1%dC");
        set_if_empty(ti, kTerm_parm_left_cursor, c"\x1b[%p1%dD");
    } else if !t.putty && t.iterm {
        set(ti, kTerm_enter_ca_mode, c"\x1b[?1049h");
        set(ti, kTerm_exit_ca_mode, c"\x1b[?1049l");
        set_if_empty(ti, kTerm_enter_italics_mode, c"\x1b[3m");
    }

    // Colour counts. A description stuck at 8 or 16 colours on a terminal
    // that has had 256 for a decade is the commonest terminfo bug of all.
    if ti.max_colors < 256 {
        if t.true_xterm || t.iterm || t.iterm_pretending_xterm {
            ti.max_colors = 256;
            set(ti, kTerm_set_a_foreground, XTERM_SETAF_256_COLON);
            set(ti, kTerm_set_a_background, XTERM_SETAB_256_COLON);
        } else if t.konsole_version != 0
            || t.xterm
            || t.gnome
            || t.rxvt
            || t.st
            || t.putty
            || t.linuxvt
            || t.mate_pretending_xterm
            || t.gnome_pretending_xterm
            || t.tmux
            || t.colorterm.as_deref().is_some_and(|c| contains(c, b"256"))
            || t.term_says_256
        {
            ti.max_colors = 256;
            set(ti, kTerm_set_a_foreground, XTERM_SETAF_256);
            set(ti, kTerm_set_a_background, XTERM_SETAB_256);
        }
    }
    if ti.max_colors < 16 && t.colorterm.is_some() {
        ti.max_colors = 16;
        set_if_empty(ti, kTerm_set_a_foreground, XTERM_SETAF_16);
        set_if_empty(ti, kTerm_set_a_background, XTERM_SETAB_16);
    }

    // These reset the cursor to a blinking block rather than to the shape it
    // had, so nvim must not use their `reset_cursor_style`.
    if t.st || (t.vte_version != 0 && t.vte_version < 3900) || t.konsole_version != 0 {
        clear(ti, kTerm_reset_cursor_style);
    }

    // DECSCUSR: nearly universal, described almost nowhere.
    let has_decscusr = !t.bsdvt
        && (t.xterm
            || t.putty
            || t.hterm
            || t.vte_version != 0
            || t.konsole_version != 0
            || t.tmux
            || t.screen
            || t.st
            || t.rxvt
            || t.iterm
            || t.iterm_pretending_xterm
            || t.teraterm
            || t.alacritty
            || t.cygwin
            || t.foot
            || t.kitty
            || t.ghostty
            || (t.linuxvt && (t.has_xterm_version || t.colorterm.is_some())));
    if has_decscusr {
        set(ti, kTerm_set_cursor_style, c"\x1b[%p1%d q");
        set(ti, kTerm_reset_cursor_style, c"\x1b[0 q");
    } else if t.linuxvt {
        // The linux console spells cursor shape with a private sequence, and
        // numbers the shapes differently.
        set(
            ti,
            kTerm_set_cursor_style,
            c"\x1b[?%?%p1%{2}%<%t%{8}%e%p1%{2}%=%t%{112}%e%p1%{3}%=%t%{4}%e%p1%{4}%=%t%{4}%e%p1%{5}%=%t%{2}%e%p1%{6}%=%t%{2}%e%{0}%;%dc",
        );
        set(ti, kTerm_reset_cursor_style, c"\x1b[?c");
    }
}

// ---------------------------------------------------------------- augmenting

/// What [`augment_terminfo`] concluded, over and above the capabilities it
/// wrote into the description.
pub struct Augmentation {
    pub can_resize_screen: bool,
    pub can_set_title: bool,
    /// Whether `set_cursor_color` takes a colour name (`%s`) rather than the
    /// six hex digits the other spelling wants.
    pub set_cursor_color_as_str: bool,
    /// Whether the extended underline styles can be assumed rather than
    /// queried for at runtime.
    pub extended_underline: bool,
    pub key_encoding: KeyEncoding,
    pub ext: TerminfoExt,
}

/// Add the capabilities terminfo has no way to describe.
pub fn augment_terminfo(ti: &mut TerminfoEntry, t: &Terminal) -> Augmentation {
    let ext = TerminfoExt {
        // Every terminal nvim supports spells alternate-font this way, and
        // terminfo has no slot for it.
        enter_altfont_mode: Some(c"\x1b[11m"),
        // rxvt needs its private focus-reporting toggle alongside the
        // standard one.
        enable_focus_reporting: Some(if t.rxvt {
            c"\x1b[?1004h\x1b]777;focus;on\x07"
        } else {
            c"\x1b[?1004h"
        }),
        disable_focus_reporting: Some(if t.rxvt {
            c"\x1b[?1004l\x1b]777;focus;off\x07"
        } else {
            c"\x1b[?1004l"
        }),
        reset_scroll_region: (t.putty || t.xterm || t.hterm || t.rxvt).then_some(c"\x1b[r"),
    };

    // Truecolour setters. The colon-separated form is the standard one, but
    // only these terminals parse it -- and a multiplexer in the way rules it
    // out, because the multiplexer is what would have to do the parsing.
    let colon_rgb = !t.tmux
        && !t.screen
        && t.vte_version == 0
        && (t.iterm || t.iterm_pretending_xterm || t.true_xterm);
    set_if_empty(
        ti,
        kTerm_set_rgb_foreground,
        if colon_rgb {
            c"\x1b[38:2:%p1%d:%p2%d:%p3%dm"
        } else {
            c"\x1b[38;2;%p1%d;%p2%d;%p3%dm"
        },
    );
    set_if_empty(
        ti,
        kTerm_set_rgb_background,
        if colon_rgb {
            c"\x1b[48:2:%p1%d:%p2%d:%p3%dm"
        } else {
            c"\x1b[48;2;%p1%d;%p2%d;%p3%dm"
        },
    );

    // Cursor colour. iTerm uses the linux-console-style `Pl` sequence, which
    // needs wrapping when tmux is in the way; everything else uses OSC 12.
    if cap(ti, kTerm_set_cursor_color).is_none() {
        if t.iterm || t.iterm_pretending_xterm {
            set(
                ti,
                kTerm_set_cursor_color,
                if t.tmux {
                    c"\x1bPtmux;\x1b\x1b]Pl%p1%06x\x1b\\\x1b\\"
                } else {
                    c"\x1b]Pl%p1%06x\x1b\\"
                },
            );
        } else if (t.xterm || t.hterm || t.rxvt || t.tmux || t.alacritty || t.st)
            && (t.vte_version == 0 || t.vte_version >= 3900)
        {
            set(ti, kTerm_set_cursor_color, c"\x1b]12;%p1%s\x07");
        }
    }
    let mut set_cursor_color_as_str = false;
    if let Some(setter) = cap(ti, kTerm_set_cursor_color) {
        set_cursor_color_as_str = contains(setter, b"%s");
        set_if_empty(ti, kTerm_reset_cursor_color, c"\x1b]112\x07");
    }

    Augmentation {
        can_resize_screen: t.dtterm || t.xterm || t.konsole_version != 0 || t.teraterm || t.rxvt,
        can_set_title: cap(ti, kTerm_to_status_line).is_some()
            && cap(ti, kTerm_from_status_line).is_some(),
        set_cursor_color_as_str,
        // A description that already knows `set_underline_style` settles it.
        // Otherwise only versions known to have grown the feature qualify,
        // and everything else has to be asked at runtime.
        extended_underline: cap(ti, kTerm_set_underline_style).is_some()
            || t.vte_version >= 5102
            || t.konsole_version >= 221170
            || ti.Su
            || t.wezterm_version
                .as_deref()
                .is_some_and(|v| v > b"20210203-095643".as_slice()),
        // kitty's own keyboard protocol takes precedence over the xterm
        // modifyOtherKeys encoding, and VTE only learned the latter in 0.54.
        key_encoding: if t.kitty || (t.vte_version != 0 && t.vte_version < 5400) {
            KEY_ENCODING_LEGACY
        } else {
            KEY_ENCODING_XTERM
        },
        ext,
    }
}
