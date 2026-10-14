//! The accessors the rest of the editor reads options through.
//!
//! Most of these exist because a value is not simply "the variable": a
//! global-local option falls back to the global when the local copy is
//! unset, 'shortmess' has an abbreviation that stands for four other flags,
//! 'virtualedit' and 'cursorlineopt' are read as parsed flag words, and
//! 'scrolloff' is forced to zero in a terminal buffer.

#![forbid(unsafe_code)]

use crate::optionstr::{OptString, local_or_global};
use crate::strings::find_bytes;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_uint};

use crate::buffer::{buf_is_prompt, current_buf};
use crate::drawscreen::redraw_buf_status_later;
use crate::drawscreen::state::{need_maketitle, redraw_tabline};
use crate::eval::typval::{DictRef, callback_free, tv_dict_alloc};
use crate::eval::{callback_from_typval, eval_expr};
use crate::memory::XString;
use crate::option::vars::{
    P_BS, P_CPO, P_FFS, P_SHM, P_SISO, P_SO, bkc_flags, p_magic, p_sh, p_siso, p_so, ve_flags,
};
use crate::options::*;
use crate::optionstr::{OptStringRef, empty_option};
use crate::os::env::{env_set, vim_getenv_owned};
use crate::path::{full_name_save, tail_index};
use crate::regexp::state::{OPTION_MAGIC_OFF, OPTION_MAGIC_ON};
use crate::search::state::magic_overruled;
use crate::state::mode::State;
use crate::types::{
    BsFlag, Callback, CpoFlag, ExArg, Failed, OptInt, OptVal, OptionSetFlags, ScriptId, ShmFlag,
    TypVal, int64_t, uint8_t,
};

use super::{
    EOL_DOS, EOL_MAC, EOL_UNIX, FORCE_BIN, get_option, get_varp, kOptScopeBuf, kOptScopeWin,
    option_has_scope, optval_as_tv, set_option_direct, slot_value,
};

use crate::state::MODE_TERMINAL;
pub(crate) use crate::strings::{copy_option_part, skip_to_option_part};
use crate::winlayer::Win;

/// 'equalprg', local where set.
pub(crate) fn get_equalprg() -> XString {
    local_or_global(&Buf::current().b_p_ep, P_EP).get()
}

/// 'findfunc', local where set.
pub(crate) fn get_findfunc() -> XString {
    local_or_global(&Buf::current().b_p_ffu, P_FFU).get()
}

/// Whether 'shortmess' asks for message `x` to be shortened. The `a` flag is
/// an abbreviation standing for these four and nothing else.
pub(crate) fn shortmess(x: ShmFlag) -> bool {
    const ABBREVIATED: [ShmFlag; 4] = [ShmFlag::RO, ShmFlag::MOD, ShmFlag::LINES, ShmFlag::WRI];
    P_SHM.has_byte(x.byte())
        || (P_SHM.has_byte(ShmFlag::ABBREVIATIONS.byte()) && ABBREVIATED.contains(&x))
}

/// Whether 'cpoptions' contains `flag`.
pub(crate) fn cpo_has(flag: CpoFlag) -> bool {
    P_CPO.has_byte(flag.byte())
}

/// Record where a vimrc was found in `$MYVIMRC`/`$MYVIMDIR`, unless the
/// environment variable is already set.
pub(crate) fn vimrc_found(fname: &CStr, envname: &CStr) {
    if vim_getenv_owned(envname).is_some() {
        return;
    }
    let _ = env_set(envname, full_name_save(fname, false).as_cstr());
}

/// What a 'cursorlineopt' value asks for, as `w_p_culopt_flags` bits, or
/// `Err` for a value that does not parse.
fn culopt_flags(value: &[u8]) -> Result<uint8_t, Failed> {
    const WORDS: [(&[u8], c_int); 4] = [
        (b"line", kOptCuloptFlagLine as c_int),
        (
            b"both",
            (kOptCuloptFlagLine | kOptCuloptFlagNumber) as c_int,
        ),
        (b"number", kOptCuloptFlagNumber as c_int),
        (b"screenline", kOptCuloptFlagScreenline as c_int),
    ];
    let mut flags: c_int = 0;
    let mut rest = value;
    while let Some(&first) = rest.first() {
        if let Some(&(word, bits)) = WORDS.iter().find(|(word, _)| rest.starts_with(word)) {
            rest = &rest[word.len()..];
            flags |= bits;
        } else if first != b',' {
            // Anything the words above did not consume is a syntax error.
            return Err(Failed);
        }
        match rest.first() {
            None => {}
            Some(b',') => rest = &rest[1..],
            Some(_) => return Err(Failed),
        }
    }
    // "line" and "screenline" are mutually exclusive; "both" implies
    // "line", so it collides with "screenline" too.
    if flags & kOptCuloptFlagLine as c_int != 0 && flags & kOptCuloptFlagScreenline as c_int != 0 {
        return Err(Failed);
    }
    Ok(uint8_t::try_from(flags).expect("four flag bits"))
}

/// Parse 'cursorlineopt' into `window.w_p_culopt_flags`, from `val` or from the
/// window's own value. `Err` for a value that does not parse; the flags are
/// only stored on success.
pub(crate) fn fill_culopt_flags(val: Option<&CStr>, mut window: Win) -> Result<(), Failed> {
    let flags = match val {
        Some(val) => culopt_flags(val.to_bytes()),
        None => culopt_flags(window.w_onebuf_opt.wo_culopt.bytes()),
    }?;
    window.w_p_culopt_flags = flags;
    Ok(())
}

/// Whether patterns are magic right now — `\v`/`\V` in the pattern override
/// 'magic' for the pattern they appear in.
pub(crate) fn magic_isset() -> bool {
    match magic_overruled.get() {
        OPTION_MAGIC_ON => true,
        OPTION_MAGIC_OFF => false,
        _ => p_magic(),
    }
}

/// The callback a function option's value names: `Ok(None)` for an empty
/// value, which clears the option's callback, and `Err` — the option keeping
/// the one it has — when the value names none.
///
/// A lambda, `function(...)` or `funcref(...)` is an expression, and is
/// evaluated: user code, which may set this very option. The text is
/// copied first, and the answer is a value the caller installs only once
/// the evaluation is over ([`install_callback`]).
pub(crate) fn callback_from_option(optval: &CStr) -> Result<Option<Callback>, Failed> {
    if optval.is_empty() {
        return Ok(None);
    }
    let text = XString::from_cstr(optval);
    let tv = if text.starts_with(b"{")
        || text.starts_with(b"function(")
        || text.starts_with(b"funcref(")
    {
        eval_expr(&text).ok_or(Failed)?
    } else {
        TypVal::string(Some(text.into()))
    };
    let mut cb = Callback::None;
    if !callback_from_typval(&mut cb, &tv) || !cb.is_set() {
        return Err(Failed);
    }
    Ok(Some(cb))
}

/// Install what [`callback_from_option`] answered: the slot's old callback
/// goes, and the new one -- if the value named one -- takes its place.
pub(crate) fn install_callback(slot: &mut Callback, cb: Option<Callback>) {
    callback_free(slot);
    if let Some(cb) = cb {
        *slot = cb;
    }
}

/// Whether 'backspace' allows backspacing over `what`. A prompt buffer never
/// lets the prompt itself be backspaced over.
pub(crate) fn can_bs(what: BsFlag) -> bool {
    if what == BsFlag::START && buf_is_prompt(current_buf()) {
        return false;
    }
    if P_BS.first_byte() == b'2' {
        return what != BsFlag::NOSTOP;
    }
    P_BS.has_byte(what.byte())
}

/// 'backupcopy' as flags, local where set.
pub(crate) fn get_bkc_flags(buffer: Buf) -> c_uint {
    match buffer.b_bkc_flags {
        0 => bkc_flags.get(),
        local => local,
    }
}

impl Buf {
    /// 'formatlistpat', local where set.
    ///
    /// The answer is the option's own buffer; the callers read it straight
    /// through `vim_regcomp`.
    pub(crate) fn formatlistpat(self) -> *mut c_char {
        local_or_global(&self.b_p_flp, P_FLP).value_ptr()
    }
}

/// 'virtualedit' as flags, local where set. The two "none" bits only exist
/// so a window can spell out that it overrides the global value with
/// nothing, so they never reach a caller.
///
pub(crate) fn get_ve_flags(window: Win) -> c_uint {
    let flags = match window.w_onebuf_opt.wo_ve_flags {
        0 => ve_flags.get(),
        local => local,
    };
    flags & !(kOptVeFlagNone | kOptVeFlagNoneU)
}

impl Win {
    /// 'showbreak', local where set. `"NONE"` is how a window says "no
    /// leader" against a global value that has one.
    ///
    /// The answer is the option's own buffer, and the draw path reads it
    /// before anything can set an option.
    pub(crate) fn showbreak_leader(self) -> *mut c_char {
        let local = &self.w_onebuf_opt.wo_sbr;
        if local.bytes() == b"NONE" {
            return empty_option();
        }
        local_or_global(local, P_SBR).value_ptr()
    }

    /// Whether the window draws a 'showbreak' leader at all: upstream's
    /// `*get_showbreak_value(wp) != NUL`, without the pointer.
    pub(crate) fn has_showbreak(self) -> bool {
        let local = &self.w_onebuf_opt.wo_sbr;
        local.bytes() != b"NONE" && local_or_global(local, P_SBR).first_byte() != 0
    }
}

/// The buffer's line ending. 'binary' forces Unix whatever 'fileformat' says.
///
pub(crate) fn get_fileformat(buffer: Buf) -> c_int {
    let c = buffer.b_p_ff.bytes().first().copied().unwrap_or(0);
    if buffer.b_p_bin != 0 || c == b'u' {
        EOL_UNIX
    } else if c == b'm' {
        EOL_MAC
    } else {
        EOL_DOS
    }
}

/// [`get_fileformat`] with a command's `++ff`/`++bin` overriding the buffer.
///
pub(crate) fn get_fileformat_force(buffer: Buf, excmd: Option<&ExArg>) -> c_int {
    let (force_ff, force_bin) =
        excmd.map_or((0, 0), |command| (command.force_ff, command.force_bin));
    let c = if force_ff != 0 {
        force_ff
    } else {
        let binary = if force_bin != 0 {
            (force_bin == FORCE_BIN) as c_int
        } else {
            buffer.b_p_bin
        };
        if binary != 0 {
            return EOL_UNIX;
        }
        c_int::from(buffer.b_p_ff.bytes().first().copied().unwrap_or(0))
    };
    match c as u8 {
        b'u' => EOL_UNIX,
        b'm' => EOL_MAC,
        _ => EOL_DOS,
    }
}

/// The line ending a new file gets: the first entry of 'fileformats'.
pub(crate) fn default_fileformat() -> c_int {
    match P_FFS.first_byte() {
        b'm' => EOL_MAC,
        b'd' => EOL_DOS,
        _ => EOL_UNIX,
    }
}

/// Set 'fileformat' to `eol_style` and redraw what shows it.
pub(crate) fn set_fileformat(eol_style: c_int, opt_flags: OptionSetFlags) {
    let name = match eol_style {
        EOL_UNIX => Some(c"unix"),
        EOL_MAC => Some(c"mac"),
        EOL_DOS => Some(c"dos"),
        _ => None,
    };
    // SAFETY: the names are static; `curbuf` is live.
    if let Some(name) = name {
        set_option_direct(
            kOptFileformat,
            OptVal::static_string(name),
            opt_flags,
            0 as ScriptId,
        );
    }
    redraw_buf_status_later(Buf::current());
    redraw_tabline.set(true);
    need_maketitle.set(true);
}

/// One part of a separated option value and the rest after it: what
/// [`copy_option_part`] copies and where it leaves its cursor, over a slice
/// whose end reads as the terminator.
///
/// The part is cut to `maxlen - 1` bytes. A leading `'.'` is taken without
/// being tested against the separators, so `.` can start a path entry; a
/// backslash before a separator makes it a literal and is dropped. The
/// separator ending the part is stepped over -- unless it is a comma, which
/// goes with the blanks after it, as `skip_to_option_part` steps.
pub(crate) fn split_option_part<'a>(
    option: &'a [u8],
    maxlen: usize,
    sep_chars: &[u8],
) -> (Vec<u8>, &'a [u8]) {
    // Sized to the first separator, which is the part's length unless a
    // backslash escapes one: a part pushed byte by byte into an empty
    // vector reallocated three or four times.
    let room = option
        .iter()
        .position(|&c| c == 0 || sep_chars.contains(&c))
        .unwrap_or(option.len());
    let mut part = Vec::with_capacity(room.min(maxlen));
    let rest = split_option_part_with(option, maxlen, sep_chars, |run| part.extend_from_slice(run));
    (part, rest)
}

/// [`split_option_part`] handing the part to `put` a run at a time instead of
/// collecting it: the bytes `copy_option_part`'s buffer gets, in order, with
/// no copy in between. Answers the rest after the part.
///
/// A run is everything up to the next backslash, separator or terminator;
/// the byte a backslash keeps (itself, or the separator it escapes) is a run
/// of its own.
pub(crate) fn split_option_part_with<'a>(
    option: &'a [u8],
    maxlen: usize,
    sep_chars: &[u8],
    mut put: impl FnMut(&[u8]),
) -> &'a [u8] {
    let at = |i: usize| option.get(i).copied().unwrap_or(0);
    let is_sep = |c: u8| c != 0 && sep_chars.contains(&c);
    // What the part may hold; the leading `.` is not counted against it.
    let room = maxlen.wrapping_sub(1);
    let mut len = 0;
    let mut i = 0;
    if at(0) == b'.' {
        put(b".");
        len = 1;
        i = 1;
    }
    loop {
        let end = option[i..]
            .iter()
            .position(|&c| c == 0 || c == b'\\' || is_sep(c))
            .map_or(option.len(), |n| i + n);
        let take = (end - i).min(room.saturating_sub(len));
        put(&option[i..i + take]);
        len += take;
        i = end;
        if at(i) != b'\\' {
            break;
        }
        // A backslash before a separator keeps the separator; any other
        // backslash keeps itself.
        if is_sep(at(i + 1)) {
            i += 1;
        }
        if len < room {
            put(&option[i..=i]);
            len += 1;
        }
        i += 1;
    }
    if at(i) != 0 && at(i) != b',' {
        i += 1;
    }
    skip_option_separator(&option[i.min(option.len())..])
}

/// The rest of an option value past the separator a part ended on: a comma,
/// then any spaces -- upstream's `skip_to_option_part`.
pub(crate) fn skip_option_separator(rest: &[u8]) -> &[u8] {
    let rest = rest.strip_prefix(b",").unwrap_or(rest);
    let blanks = rest.iter().take_while(|&&b| b == b' ').count();
    &rest[blanks..]
}

/// The parts of a comma-separated option value, each as [`split_option_part`]
/// copies it with no length limit.
pub(crate) fn option_parts(option: &[u8]) -> impl Iterator<Item = Vec<u8>> + '_ {
    let mut rest = option;
    core::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let (part, after) = split_option_part(rest, usize::MAX, b",");
        rest = after;
        Some(part)
    })
}

/// [`split_option_part`] over an option value the caller holds: the first
/// part of `option`, cut to `maxlen - 1` bytes, and where the next part
/// starts.
pub(crate) fn option_part(option: &CStr, maxlen: usize, sep_chars: &CStr) -> (Vec<u8>, usize) {
    let bytes = option.to_bytes();
    let (part, rest) = split_option_part(bytes, maxlen.max(2), sep_chars.to_bytes());
    (part, bytes.len() - rest.len())
}

/// One part of a comma-separated option, appended to `part`, with the rest of
/// the option answered back.
///
/// [`copy_option_part`] over slices, with `,` the separator and no length
/// limit: a backslash before a comma makes it a literal, a leading `.` is taken
/// whatever it is, and the separator and the blanks behind it are dropped.
/// One `part` buffer serves a whole walk.
pub(crate) fn next_option_part<'a>(option: &'a [u8], part: &mut Vec<u8>) -> &'a [u8] {
    part.clear();
    let mut i = 0;
    // A leading '.' is copied without being tested against the separators.
    if option.first() == Some(&b'.') {
        part.push(b'.');
        i = 1;
    }
    while i < option.len() && option[i] != b',' {
        // A backslash escapes the separator, and is dropped.
        if option[i] == b'\\' && option.get(i + 1) == Some(&b',') {
            i += 1;
        }
        part.push(option[i]);
        i += 1;
    }
    if i < option.len() {
        i += 1; // the separator itself
    }
    while option.get(i) == Some(&b' ') {
        i += 1;
    }
    &option[i..]
}

/// Whether 'shell' is a csh derivative, which needs its own quoting.
pub(crate) fn csh_like_shell() -> bool {
    p_sh(|sh| {
        let sh = sh.to_bytes();
        find_bytes(&sh[tail_index(sh)..], b"csh").is_some()
    })
}

/// Whether 'shell' is fish, which needs its own quoting.
pub(crate) fn fish_like_shell() -> bool {
    p_sh(|sh| {
        let sh = sh.to_bytes();
        find_bytes(&sh[tail_index(sh)..], b"fish").is_some()
    })
}

/// Every buffer-local (`buffer`) or window-local option of the current
/// buffer and window, as a dictionary — what `b:` and `w:` expose.
pub(crate) fn get_winbuf_options(buffer: bool) -> DictRef {
    let scope = if buffer { kOptScopeBuf } else { kOptScopeWin };
    let dict = tv_dict_alloc();
    for opt_idx in kOptAleph..kOptCount {
        if !option_has_scope(opt_idx, scope) {
            continue;
        }
        // `get_varp` hands back the variable for the current buffer and
        // window.
        let varp = get_varp(opt_idx);
        if varp.is_none() {
            continue;
        }
        // A copy of the option's value; the dictionary takes its own.
        let tv = optval_as_tv(slot_value(opt_idx, varp), true);
        let name = get_option(opt_idx).fullname;
        let _ = dict.edit().add_tv(name.to_bytes(), &tv);
    }
    dict
}

/// 'scrolloff' for a window, local where set. A terminal buffer never
/// scrolls off, whatever the option says.
///
pub(crate) fn get_scrolloff_value(window: Win) -> int64_t {
    if State.get() & MODE_TERMINAL != 0 && !window.w_buffer.terminal.is_null() {
        return 0;
    }
    match window.w_onebuf_opt.wo_so {
        local if local < 0 => p_so(),
        local => local,
    }
}

/// 'sidescrolloff' for a window, local where set.
///
pub(crate) fn get_sidescrolloff_value(window: Win) -> int64_t {
    match window.w_onebuf_opt.wo_siso {
        local if local < 0 => p_siso(),
        local => local,
    }
}

/// Which scroll margin: 'scrolloff', in lines, or 'sidescrolloff', in
/// columns.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollMargin {
    Lines,
    Columns,
}

/// The scroll margin in effect for a window: its own value where it has set
/// one, the global option otherwise.
///
/// C reaches this through an `OptInt *`, because the three callers that want
/// it *write* through it -- `showmatch`, `do_ecmd`'s `recenter` and
/// `update_topline`'s mouse-drag arm each set the margin aside for one
/// redraw and put it back. Resolving the fallback once and answering a
/// `get`/`set` pair is the same thing without the address, which is also
/// what keeps the write from landing in the wrong scope.
#[derive(Clone, Copy)]
pub(crate) enum ScrollOff {
    /// The window's own value, which it has set.
    Window(Win, ScrollMargin),
    /// The global option, because the window has not set its own.
    Global(ScrollMargin),
}

impl ScrollOff {
    /// The margin `win` reads right now.
    pub(crate) fn of(win: Win, margin: ScrollMargin) -> Self {
        let local = match margin {
            ScrollMargin::Lines => win.w_onebuf_opt.wo_so,
            ScrollMargin::Columns => win.w_onebuf_opt.wo_siso,
        };
        if local >= 0 {
            Self::Window(win, margin)
        } else {
            Self::Global(margin)
        }
    }

    /// The value in effect.
    pub(crate) fn get(self) -> OptInt {
        match self {
            Self::Window(win, ScrollMargin::Lines) => win.w_onebuf_opt.wo_so,
            Self::Window(win, ScrollMargin::Columns) => win.w_onebuf_opt.wo_siso,
            Self::Global(ScrollMargin::Lines) => p_so(),
            Self::Global(ScrollMargin::Columns) => p_siso(),
        }
    }

    /// Write it back where it came from.
    pub(crate) fn set(self, value: OptInt) {
        match self {
            Self::Window(mut win, ScrollMargin::Lines) => win.w_onebuf_opt.wo_so = value,
            Self::Window(mut win, ScrollMargin::Columns) => win.w_onebuf_opt.wo_siso = value,
            Self::Global(ScrollMargin::Lines) => P_SO.set(value),
            Self::Global(ScrollMargin::Columns) => P_SISO.set(value),
        }
    }
}

#[cfg(test)]
mod tests {
    //! The run-at-a-time option-part split and backslash halving against
    //! the byte-at-a-time walks they replaced, over every short string of a
    //! small alphabet that holds each byte the walks branch on.

    use super::{split_option_part, split_option_part_with};
    use crate::charset::backslash_halved;

    /// The byte walk `split_option_part` was, verbatim.
    fn split_by_bytes<'a>(
        option: &'a [u8],
        maxlen: usize,
        sep_chars: &[u8],
    ) -> (Vec<u8>, &'a [u8]) {
        let at = |i: usize| option.get(i).copied().unwrap_or(0);
        let is_sep = |c: u8| c != 0 && sep_chars.contains(&c);
        let mut part = Vec::new();
        let mut i = 0;
        if at(0) == b'.' {
            part.push(b'.');
            i = 1;
        }
        loop {
            let c = at(i);
            if c == 0 || is_sep(c) {
                break;
            }
            if c == b'\\' && is_sep(at(i + 1)) {
                i += 1;
            }
            if part.len() < maxlen.wrapping_sub(1) {
                part.push(at(i));
            }
            i += 1;
        }
        if at(i) != 0 && at(i) != b',' {
            i += 1;
        }
        (
            part,
            super::skip_option_separator(&option[i.min(option.len())..]),
        )
    }

    /// `backslash_halved` as it was, a byte at a time.
    fn halved_by_bytes(text: &[u8]) -> Vec<u8> {
        let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
        let mut out = Vec::new();
        let mut at = 0;
        while at < text.len() {
            if text[at] == b'\\' && at + 1 < text.len() {
                at += 1;
            }
            out.push(text[at]);
            at += 1;
        }
        out
    }

    /// Every string of up to `len` bytes over `alphabet` (shorter under Miri,
    /// where the full set takes minutes).
    fn strings(alphabet: &[u8], len: usize) -> Vec<Vec<u8>> {
        let mut all = vec![Vec::new()];
        let mut last = vec![Vec::new()];
        for _ in 0..len {
            let next: Vec<Vec<u8>> = last
                .iter()
                .flat_map(|s: &Vec<u8>| {
                    alphabet.iter().map(move |&b| {
                        let mut t = s.clone();
                        t.push(b);
                        t
                    })
                })
                .collect();
            all.extend(next.iter().cloned());
            last = next;
        }
        all
    }

    #[test]
    fn a_part_split_by_runs_is_the_byte_walk() {
        for text in strings(b".\\, :a\0", if cfg!(miri) { 3 } else { 6 }) {
            for maxlen in [0, 1, 2, 3, 5, usize::MAX] {
                for seps in [&b","[..], b", ", b":", b""] {
                    let want = split_by_bytes(&text, maxlen, seps);
                    assert_eq!(
                        split_option_part(&text, maxlen, seps),
                        want,
                        "{text:?} {maxlen} {seps:?}"
                    );
                    let mut runs = Vec::new();
                    let rest = split_option_part_with(&text, maxlen, seps, |run| {
                        runs.extend_from_slice(run)
                    });
                    assert_eq!((runs, rest), want, "{text:?} {maxlen} {seps:?}");
                }
            }
        }
    }

    #[test]
    fn halving_by_runs_is_the_byte_walk() {
        for text in strings(b"\\a\0", if cfg!(miri) { 4 } else { 7 }) {
            assert_eq!(backslash_halved(&text), halved_by_bytes(&text), "{text:?}");
        }
    }
}
