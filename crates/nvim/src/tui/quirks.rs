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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::os::env::{env_exists, getenv_owned};
use crate::tui::state::KEY_ENCODING_XTERM;
use crate::tui::state::{KeyEncoding, TerminfoExt};
use crate::tui::terminfo::caps::{
    TERM_CARRIAGE_RETURN, TERM_CURSOR_INVISIBLE, TERM_CURSOR_NORMAL, TERM_ENTER_CA_MODE,
    TERM_ENTER_ITALICS_MODE, TERM_EXIT_CA_MODE, TERM_FROM_STATUS_LINE, TERM_PARM_DOWN_CURSOR,
    TERM_PARM_LEFT_CURSOR, TERM_PARM_RIGHT_CURSOR, TERM_PARM_UP_CURSOR, TERM_RESET_CURSOR_COLOR,
    TERM_RESET_CURSOR_STYLE, TERM_SET_A_BACKGROUND, TERM_SET_A_FOREGROUND, TERM_SET_CURSOR_COLOR,
    TERM_SET_CURSOR_STYLE, TERM_SET_LR_MARGIN, TERM_SET_RGB_BACKGROUND, TERM_SET_RGB_FOREGROUND,
    TERM_SET_UNDERLINE_STYLE, TERM_TO_STATUS_LINE, TerminfoDef,
};
use crate::tui::terminfo::entry::TerminfoEntry;
use crate::tui::terminfo::{is_term_family, terminfo_is_bsd_console};
use core::ffi::{CStr, c_int};
use std::borrow::Cow;

/// The two key-encoding schemes this module chooses between. `KeyEncoding`
/// is a bare integer alias, so name the values rather than spelling them.
const KEY_ENCODING_LEGACY: KeyEncoding = 0;

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
/// entry.
fn cap(ti: &TerminfoEntry, slot: TerminfoDef) -> Option<&[u8]> {
    ti.def(slot)
}

fn set(ti: &mut TerminfoEntry, slot: TerminfoDef, val: &'static CStr) {
    ti.set_def(slot, val);
}

fn set_if_empty(ti: &mut TerminfoEntry, slot: TerminfoDef, val: &'static CStr) {
    if !ti.has(slot) {
        set(ti, slot, val);
    }
}

fn clear(ti: &mut TerminfoEntry, slot: TerminfoDef) {
    ti.defs[slot as usize] = None;
}

/// Replace capability `slot` with what `trim` leaves of it, when that is
/// less than the whole.
fn trim(ti: &mut TerminfoEntry, slot: TerminfoDef, trim: impl FnOnce(&[u8]) -> &[u8]) {
    let Some(full) = cap(ti, slot) else {
        return;
    };
    let trimmed = trim(full);
    if trimmed.len() != full.len() {
        let owned = trimmed.to_vec();
        ti.defs[slot as usize] = Some(Cow::Owned(owned));
    }
}

/// The value of `$name`, or `None` when it is unset or empty.
fn env(name: &CStr) -> Option<Vec<u8>> {
    getenv_owned(name).map(|value| value.to_vec())
}

/// Is `$name` set to something non-empty?
fn env_is_set(name: &CStr) -> bool {
    env_exists(name, true)
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
#[derive(Debug)]
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
        let bsdvt = terminfo_is_bsd_console(term);

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
        ti.has_tc_or_rgb
            || (cap(ti, TERM_SET_RGB_FOREGROUND).is_some()
                && cap(ti, TERM_SET_RGB_BACKGROUND).is_some())
    }
}

// ------------------------------------------------------------------ patching

/// Correct the description's outright errors and omissions, in place.
pub fn patch_terminfo_bugs(ti: &mut TerminfoEntry, t: &Terminal) {
    // A `cursor_normal` that starts by re-enabling blink, or (on the linux
    // console) ends by resetting the shape, fights nvim's own cursor
    // handling. Trim both.
    trim(ti, TERM_CURSOR_NORMAL, |full| {
        let bytes = full.strip_prefix(SHOW_CURSOR_BLINK_PREFIX).unwrap_or(full);
        match bytes.strip_suffix(LINUX_SET_SHAPE_0) {
            Some(trimmed) if t.linuxvt => trimmed,
            _ => bytes,
        }
    });
    if t.linuxvt {
        trim(ti, TERM_CURSOR_INVISIBLE, |full| {
            full.strip_suffix(LINUX_SET_SHAPE_1).unwrap_or(full)
        });
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
            set_if_empty(ti, TERM_TO_STATUS_LINE, c"\x1b]0;");
            set_if_empty(ti, TERM_FROM_STATUS_LINE, c"\x07");
        }
        set_if_empty(ti, TERM_ENTER_ITALICS_MODE, c"\x1b[3m");
        set_if_empty(ti, TERM_SET_LR_MARGIN, c"\x1b[%i%p1%d;%p2%ds");
    } else if t.rxvt {
        set_if_empty(ti, TERM_ENTER_ITALICS_MODE, c"\x1b[3m");
        set_if_empty(ti, TERM_TO_STATUS_LINE, c"\x1b]2");
        set_if_empty(ti, TERM_FROM_STATUS_LINE, c"\x07");
        // rxvt's own alternate-screen entries do not save the cursor.
        set(ti, TERM_ENTER_CA_MODE, c"\x1b[?1049h");
        set(ti, TERM_EXIT_CA_MODE, c"\x1b[?1049l");
        // urxvt starts with a steady block cursor (unless configured for an
        // underline, which nothing here can see), and `\x1b[0 q` gives a
        // *blinking* block rather than that default. So "reset" sets a
        // steady block, the likeliest starting shape; and rxvt stays out of
        // the DECSCUSR list below, which would put `\x1b[0 q` back.
        set(ti, TERM_RESET_CURSOR_STYLE, c"\x1b[2 q");
        set(ti, TERM_SET_CURSOR_STYLE, c"\x1b[%p1%d q");
    } else if t.screen {
        set_if_empty(ti, TERM_TO_STATUS_LINE, c"\x1b_");
        set_if_empty(ti, TERM_FROM_STATUS_LINE, c"\x1b\\");
    } else if t.tmux {
        // Deliberately not merged with the `screen` arm above: screen
        // running inside tmux matches both, and taking the `screen` arm
        // first is what leaves italics unset there. Merging the two and
        // testing `tmux` inside gives screen-in-tmux italics it has never
        // had.
        set_if_empty(ti, TERM_TO_STATUS_LINE, c"\x1b_");
        set_if_empty(ti, TERM_FROM_STATUS_LINE, c"\x1b\\");
        set_if_empty(ti, TERM_ENTER_ITALICS_MODE, c"\x1b[3m");
    } else if t.interix {
        set_if_empty(ti, TERM_CARRIAGE_RETURN, c"\r");
    } else if t.linuxvt {
        set_if_empty(ti, TERM_PARM_UP_CURSOR, c"\x1b[%p1%dA");
        set_if_empty(ti, TERM_PARM_DOWN_CURSOR, c"\x1b[%p1%dB");
        set_if_empty(ti, TERM_PARM_RIGHT_CURSOR, c"\x1b[%p1%dC");
        set_if_empty(ti, TERM_PARM_LEFT_CURSOR, c"\x1b[%p1%dD");
    } else if !t.putty && t.iterm {
        set(ti, TERM_ENTER_CA_MODE, c"\x1b[?1049h");
        set(ti, TERM_EXIT_CA_MODE, c"\x1b[?1049l");
        set_if_empty(ti, TERM_ENTER_ITALICS_MODE, c"\x1b[3m");
    }

    // Colour counts. A description stuck at 8 or 16 colours on a terminal
    // that has had 256 for a decade is the commonest terminfo bug of all.
    if ti.max_colors < 256 {
        if t.true_xterm || t.iterm || t.iterm_pretending_xterm {
            ti.max_colors = 256;
            set(ti, TERM_SET_A_FOREGROUND, XTERM_SETAF_256_COLON);
            set(ti, TERM_SET_A_BACKGROUND, XTERM_SETAB_256_COLON);
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
            set(ti, TERM_SET_A_FOREGROUND, XTERM_SETAF_256);
            set(ti, TERM_SET_A_BACKGROUND, XTERM_SETAB_256);
        }
    }
    if ti.max_colors < 16 && t.colorterm.is_some() {
        ti.max_colors = 16;
        set_if_empty(ti, TERM_SET_A_FOREGROUND, XTERM_SETAF_16);
        set_if_empty(ti, TERM_SET_A_BACKGROUND, XTERM_SETAB_16);
    }

    // These reset the cursor to a blinking block rather than to the shape it
    // had, so nvim must not use their `reset_cursor_style`.
    if t.st || (t.vte_version != 0 && t.vte_version < 3900) || t.konsole_version != 0 {
        clear(ti, TERM_RESET_CURSOR_STYLE);
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
        set(ti, TERM_SET_CURSOR_STYLE, c"\x1b[%p1%d q");
        set(ti, TERM_RESET_CURSOR_STYLE, c"\x1b[0 q");
    } else if t.linuxvt {
        // The linux console spells cursor shape with a private sequence, and
        // numbers the shapes differently.
        set(
            ti,
            TERM_SET_CURSOR_STYLE,
            c"\x1b[?%?%p1%{2}%<%t%{8}%e%p1%{2}%=%t%{112}%e%p1%{3}%=%t%{4}%e%p1%{4}%=%t%{4}%e%p1%{5}%=%t%{2}%e%p1%{6}%=%t%{2}%e%{0}%;%dc",
        );
        set(ti, TERM_RESET_CURSOR_STYLE, c"\x1b[?c");
    }
}

// ---------------------------------------------------------------- augmenting

/// What [`augment_terminfo`] concluded, over and above the capabilities it
/// wrote into the description.
#[derive(Debug)]
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
        TERM_SET_RGB_FOREGROUND,
        if colon_rgb {
            c"\x1b[38:2:%p1%d:%p2%d:%p3%dm"
        } else {
            c"\x1b[38;2;%p1%d;%p2%d;%p3%dm"
        },
    );
    set_if_empty(
        ti,
        TERM_SET_RGB_BACKGROUND,
        if colon_rgb {
            c"\x1b[48:2:%p1%d:%p2%d:%p3%dm"
        } else {
            c"\x1b[48;2;%p1%d;%p2%d;%p3%dm"
        },
    );

    // Cursor colour. iTerm uses the linux-console-style `Pl` sequence, which
    // needs wrapping when tmux is in the way; everything else uses OSC 12.
    if cap(ti, TERM_SET_CURSOR_COLOR).is_none() {
        if t.iterm || t.iterm_pretending_xterm {
            set(
                ti,
                TERM_SET_CURSOR_COLOR,
                if t.tmux {
                    c"\x1bPtmux;\x1b\x1b]Pl%p1%06x\x1b\\\x1b\\"
                } else {
                    c"\x1b]Pl%p1%06x\x1b\\"
                },
            );
        } else if (t.xterm || t.hterm || t.rxvt || t.tmux || t.alacritty || t.st)
            && (t.vte_version == 0 || t.vte_version >= 3900)
        {
            set(ti, TERM_SET_CURSOR_COLOR, c"\x1b]12;%p1%s\x07");
        }
    }
    let set_cursor_color_as_str =
        cap(ti, TERM_SET_CURSOR_COLOR).is_some_and(|setter| contains(setter, b"%s"));
    if ti.has(TERM_SET_CURSOR_COLOR) {
        set_if_empty(ti, TERM_RESET_CURSOR_COLOR, c"\x1b]112\x07");
    }

    Augmentation {
        can_resize_screen: t.dtterm || t.xterm || t.konsole_version != 0 || t.teraterm || t.rxvt,
        can_set_title: cap(ti, TERM_TO_STATUS_LINE).is_some()
            && cap(ti, TERM_FROM_STATUS_LINE).is_some(),
        set_cursor_color_as_str,
        // A description that already knows `set_underline_style` settles it.
        // Otherwise only versions known to have grown the feature qualify,
        // and everything else has to be asked at runtime.
        extended_underline: cap(ti, TERM_SET_UNDERLINE_STYLE).is_some()
            || t.vte_version >= 5102
            || t.konsole_version >= 221170
            || ti.su
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
