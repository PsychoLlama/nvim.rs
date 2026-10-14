//! Expanding one wildcard pattern against the file system.
//!
//! [`do_path_expand`] is the recursive walk: it takes the pattern apart one
//! component at a time, turns the component into a regexp, and reads each
//! directory that the components before it matched, recursing on `**` to the
//! depth the pattern asks for. [`addfile`] is what decides whether a name the
//! walk found belongs in the answer, and [`match_suffix`] ranks the results
//! by `'suffixes'`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::{backslash_halved, escapes_next};
use crate::cstr;
use crate::memory::XString;
use crate::option::split_option_part;
use crate::option::vars::P_SU;
use core::ffi::c_int;
use std::ffi::CStr;

use super::*;
use crate::fileio::file_pat_to_regpat;
use crate::guard::{Depth, Suppress};
use crate::mbyte::{char_at, cluster_len};
use crate::os::fs::{link_exists, path_exists};
use crate::regexp::{OwnedMatch, RE_MAGIC, RE_NOBREAK};
use crate::types::MAXPATHL;

/// How deep a `**` may recurse. Upstream's limit, and the reason a pattern
/// over a symlink loop terminates.
const MAX_STAR_DEPTH: c_int = 100;

/// The `**` recursion depth of the walk currently running. A global because
/// [`do_path_expand`] recurses through the file-system scan rather than
/// carrying state down.
static STARDEPTH: GlobalCell<c_int> = GlobalCell::new(0);

/// Does `p` hold a character that means expansion is needed — including a
/// leading `~`, unless it is the last character? A backslash escapes the
/// character after it.
pub(crate) fn has_wildcard(p: &[u8]) -> bool {
    has_any_of(cstr::until_nul(p), b"*?[{`'$", true)
}

/// Does `p` hold a character [`do_path_expand`] itself can expand? `~` and
/// `$` are not among them: those are the shell's and `expand_env`'s. `p`
/// is a name's bytes, without a NUL.
pub(crate) fn has_exp_wildcard(p: &[u8]) -> bool {
    has_any_of(p, b"*?[{", false)
}

/// Does `p` hold one of `wildcards`, or — when `tilde` is set — a `~` that
/// is not the last character? A backslash escapes what follows it.
///
/// Upstream walks whole characters; scanning bytes finds the same ones,
/// because every byte of a multibyte character is at least 0x80 and none of
/// these is.
fn has_any_of(p: &[u8], wildcards: &[u8], tilde: bool) -> bool {
    let mut at = 0;
    while at < p.len() {
        if p[at] == b'\\' && at + 1 < p.len() {
            // Escaped: neither byte counts.
            at += 2;
            continue;
        }
        if wildcards.contains(&p[at]) || (tilde && p[at] == b'~' && at + 1 < p.len()) {
            return true;
        }
        at += 1;
    }
    false
}

/// Expand `path`'s wildcards, adding every name that matches to `names`.
pub(crate) fn path_expand(names: &mut Vec<XString>, path: &[u8], flags: ExpandFlags) -> usize {
    do_path_expand(names, path, 0, flags, false)
}

/// A directory scan that answers `"."` and `".."` before the entries libuv
/// reports, which do not include them.
struct ScanWithDots {
    dir: Directory,
    count: u32,
}

impl ScanWithDots {
    /// Read `path`, or answer `None` when it cannot be read at all.
    fn open(path: &CStr) -> Option<Self> {
        let mut dir = Directory::default();
        if !os_file_is_readable(path) || !os_scandir(&mut dir, path) {
            return None;
        }
        Some(ScanWithDots { dir, count: 0 })
    }

    /// The next entry's name, valid until the following call.
    fn next(&mut self) -> Option<&CStr> {
        self.count += 1;
        match self.count {
            1 => Some(c"."),
            2 => Some(c".."),
            _ => os_scandir_next(&mut self.dir),
        }
    }
}

impl Drop for ScanWithDots {
    fn drop(&mut self) {
        os_closedir(&mut self.dir);
    }
}

/// The first component of a pattern that holds a wildcard, and where the
/// components around it are.
struct Split {
    /// Where the component starts in the buffer.
    comp: usize,
    /// Where it ends. The buffer holds a NUL there and nothing after it.
    comp_end: usize,
    /// Where the components after it start, in the pattern.
    rest: usize,
    /// Does the component hold a `**`?
    starstar: bool,
}

/// Copy `pattern` into `buf` up to the end of its first wildcard component,
/// saying where that component is. Characters before `wildoff` are never
/// taken for wildcards. `buf` must have room for `pattern` and a NUL.
fn split_wild_component(
    pattern: &[u8],
    wildoff: usize,
    flags: ExpandFlags,
    buf: &mut [u8],
) -> Split {
    // With every letter a wildcard, a name that differs only in case is
    // still a match.
    let icase = !p_fic() && flags.has(ExpandFlags::ICASE);
    // Where the component starts, and whether a wildcard has turned up
    // in it yet.
    let mut comp = 0;
    let mut seen_wild = false;
    // The read and write positions advance together: nothing is dropped
    // here, only stopped at.
    let mut at = 0;
    while at < pattern.len() {
        if at >= wildoff && escapes_next(&pattern[at..]) {
            // The backslash stays for `file_pat_to_reg_pat` to drop; the
            // character it escapes is copied with the step below.
            buf[at] = pattern[at];
            at += 1;
        } else if vim_ispathsep_nocolon(c_int::from(pattern[at])) {
            if seen_wild {
                break;
            }
            // A later component; the wildcard is not in this one.
            comp = at + 1;
        } else if at >= wildoff
            && (b"*?[{~$".contains(&pattern[at]) || (icase && mb_isalpha(char_at(&pattern[at..]))))
        {
            seen_wild = true;
        }
        // A character is copied whole, however many bytes it takes. The
        // slice's own end bounds the measurement, so a sequence cut short by
        // it is one byte and the copy below cannot reach past `pattern`.
        let charlen = cluster_len(&pattern[at..]);
        buf[at..at + charlen].copy_from_slice(&pattern[at..at + charlen]);
        at += charlen;
    }
    let rest = at;
    let mut comp_end = at;
    buf[comp_end] = 0;

    // The backslashes before the component are the caller's escaping and
    // come off now; the ones inside it belong to the regexp.
    let mut at = wildoff;
    while at < comp {
        if escapes_next(&buf[at..]) {
            buf.copy_within(at + 1..comp_end + 1, at);
            comp_end -= 1;
            comp -= 1;
        }
        at += 1;
    }

    let starstar = buf[comp..comp_end].windows(2).any(|pair| pair == b"**");
    Split {
        comp,
        comp_end,
        rest,
        starstar,
    }
}

/// Write `parts` into `buf` at `at`, NUL-terminated, and answer how long
/// they were.
///
/// Like `vim_snprintf`: what does not fit is dropped, but the answer is
/// still the length it would have taken, which is how the caller notices.
fn write_at(buf: &mut [u8], at: usize, parts: &[&[u8]]) -> usize {
    let mut room = buf.len() - at - 1;
    let mut end = at;
    for part in parts {
        let fits = part.len().min(room);
        buf[end..end + fits].copy_from_slice(&part[..fits]);
        end += fits;
        room -= fits;
    }
    buf[end] = 0;
    parts.iter().map(|part| part.len()).sum()
}

/// Remove the escaping backslashes from the string at `buf[at..]`, in place.
fn halve_backslashes_at(buf: &mut [u8], at: usize) {
    let halved = backslash_halved(&buf[at..]);
    buf[at..at + halved.len()].copy_from_slice(&halved);
    buf[at + halved.len()] = 0;
}

/// Expand the wildcards in `path` into `names`, one component at a time.
///
/// Characters before `wildoff` are taken literally — that is how the
/// recursion tells the part it has already resolved from the part it still
/// has to match. `didstar` says the caller has already recursed for a `**`,
/// so this level should not do it again.
///
/// Answers how many names were added.
pub(crate) fn do_path_expand(
    names: &mut Vec<XString>,
    path: &[u8],
    wildoff: usize,
    flags: ExpandFlags,
    didstar: bool,
) -> usize {
    let start_len = names.len();

    // Expanding "**" may take a long time; let CTRL-C out of it.
    if STARDEPTH.get() > 0 && !flags.has(ExpandFlags::NOBREAK) {
        os_breakcheck();
        if got_int.get() {
            return 0;
        }
    }

    let pattern = cstr::until_nul(path);
    // Room for the pattern and any one name it grows into.
    let buflen = pattern.len() + MAXPATHL as usize;
    let mut buf = vec![0u8; buflen];
    let split = split_wild_component(pattern, wildoff, flags, &mut buf);
    let dir_len = split.comp;
    let rest = &pattern[split.rest..];

    // A name starting with a dot is only matched by a pattern that does.
    let starts_with_dot = buf[dir_len] == b'.';
    let Some(pat) = file_pat_to_regpat(cstr::in_bytes(&buf[dir_len..])) else {
        return 0;
    };
    // Ignore case if given 'wildignorecase', else respect 'fileignorecase'.
    let ignore_case = flags.has(ExpandFlags::ICASE) || p_fic();
    let silent = flags
        .has(ExpandFlags::NOERROR | ExpandFlags::NOTWILD)
        .then(Suppress::emsg_silent);
    let nobreak = flags.has(ExpandFlags::NOBREAK);
    let compiled = OwnedMatch::compile(
        pat.as_cstr(),
        RE_MAGIC | if nobreak { RE_NOBREAK } else { 0 },
        ignore_case,
    );
    drop(silent);
    drop(pat);
    if compiled.is_none() && !flags.has(ExpandFlags::NOTWILD) {
        return 0;
    }
    let mut regmatch = compiled.unwrap_or_else(OwnedMatch::none);
    regmatch.rm_ic = ignore_case;
    // A "**" by itself also matches no directory at all, so the
    // components after it are tried against the directory reached so
    // far.
    if !didstar
        && STARDEPTH.get() < MAX_STAR_DEPTH
        && split.starstar
        && split.comp_end - dir_len == 2
        && pattern.get(split.rest) == Some(&b'/')
    {
        write_at(&mut buf, dir_len, &[&pattern[split.rest + 1..]]);
        let stardepth = Depth::of(&STARDEPTH);
        do_path_expand(names, cstr::until_nul(&buf), dir_len, flags, true);
        drop(stardepth);
    }
    // Back to the directory the component lives in.
    buf[dir_len] = 0;

    let dirpath = if dir_len == 0 {
        c"."
    } else {
        cstr::in_bytes(&buf)
    };
    if let Some(mut scan) = ScanWithDots::open(dirpath) {
        // The component as the pattern spells it, for a literal match.
        let comp = &pattern[dir_len.min(pattern.len())..];
        while !got_int.get() {
            let Some(name) = scan.next() else { break };
            if !name_is_wanted(name.to_bytes(), starts_with_dot, flags)
                || !name_matches(&mut regmatch, name, flags, comp, split.comp_end - dir_len)
            {
                continue;
            }
            let len = dir_len + write_at(&mut buf, dir_len, &[name.to_bytes()]);
            if len + 1 >= buflen {
                continue;
            }

            if split.starstar && STARDEPTH.get() < MAX_STAR_DEPTH {
                // For "**" first go deeper in the tree to find matches.
                write_at(&mut buf, len, &[b"/**", rest]);
                let stardepth = Depth::of(&STARDEPTH);
                do_path_expand(names, cstr::until_nul(&buf), len + 1, flags, true);
                drop(stardepth);
            }

            write_at(&mut buf, len, &[rest]);
            if has_exp_wildcard(rest) {
                // Another component to expand.
                if STARDEPTH.get() < MAX_STAR_DEPTH {
                    let _stardepth = Depth::of(&STARDEPTH);
                    do_path_expand(names, cstr::until_nul(&buf), len + 1, flags, false);
                }
            } else {
                // No more wildcards: the escaping in what is left is the
                // caller's, and comes off before the name is looked up.
                if !rest.is_empty() {
                    halve_backslashes_at(&mut buf, len + 1);
                }
                let found_name = cstr::in_bytes(&buf);
                let found = if flags.has(ExpandFlags::ALLLINKS) {
                    link_exists(found_name)
                } else {
                    path_exists(found_name)
                };
                if found {
                    addfile(names, found_name, flags);
                }
            }
        }
    }
    drop(regmatch);

    // When interrupted the matches probably won't be used, and sorting
    // can be slow.
    let matches = names.len() - start_len;
    if matches > 0 && !got_int.get() {
        // Stable: with 'fileignorecase' two names that differ can compare
        // equal, and they keep the order the scan found them in.
        names[start_len..].sort_by(|a, b| pathcmp(a, b, None).cmp(&0));
    }
    matches
}

/// Is `name` one the walk should even try to match? A name starting with a
/// dot is hidden unless the pattern starts with one too, and `.` and `..`
/// are only ever answered for [`ExpandFlags::DODOT`].
fn name_is_wanted(name: &[u8], starts_with_dot: bool, flags: ExpandFlags) -> bool {
    if name.first() != Some(&b'.') || starts_with_dot {
        return true;
    }
    flags.has(ExpandFlags::DODOT) && name != b"." && name != b".."
}

/// Does `name` match the component? Either the compiled pattern says so, or
/// [`ExpandFlags::NOTWILD`] asked for the component's own text — the first
/// `comp_len` bytes of `comp` — to be compared literally.
fn name_matches(
    regmatch: &mut OwnedMatch,
    name: &CStr,
    flags: ExpandFlags,
    comp: &[u8],
    comp_len: usize,
) -> bool {
    if !regmatch.regprog.is_null() && vim_regexec(regmatch, name, 0) {
        return true;
    }
    flags.has(ExpandFlags::NOTWILD) && path_fnamencmp(comp, name.to_bytes(), comp_len) == 0
}

/// Does `p` hold a wildcard character only a shell can expand?
///
/// A brace needs one when non-existing names are wanted, since only the
/// shell invents those; a backtick or quote needs one when it is matched.
///
/// `p` is a pattern's bytes, without a NUL.
pub(crate) fn has_special_wildchar(p: &[u8], flags: ExpandFlags) -> bool {
    let bytes = p;
    let mut at = 0;
    while at < bytes.len() {
        let c = bytes[at];
        // A line break is not part of a name at all.
        if c == b'\r' || c == b'\n' {
            break;
        }
        if c == b'\\' && matches!(bytes.get(at + 1), Some(&next) if next != b'\r' && next != b'\n')
        {
            at += 2;
            continue;
        }
        if matches!(c, b'`' | b'\'' | b'{') {
            let rest = &bytes[at..];
            let claimed = match c {
                // Braces are the shell's only when they can invent names,
                // and only when they are closed.
                b'{' => flags.has(ExpandFlags::NOTFOUND) && rest.contains(&b'}'),
                // A quote or backtick has to be matched by another.
                _ => rest[1..].contains(&c),
            };
            if claimed {
                return true;
            }
        }
        at += 1;
    }
    false
}

/// Add the file `f` to `names`, unless the flags say it does not belong: a
/// name that does not exist, a directory when only files were asked for, a
/// file that is not executable when only executables were.
pub fn addfile(names: &mut Vec<XString>, f: &CStr, flags: ExpandFlags) {
    let exists = if flags.has(ExpandFlags::ALLLINKS) {
        link_exists(f)
    } else {
        path_exists(f)
    };
    if !flags.has(ExpandFlags::NOTFOUND) && !exists {
        return;
    }

    let isdir = dir_exists(f);
    if isdir && !flags.has(ExpandFlags::DIR) || !isdir && !flags.has(ExpandFlags::FILE) {
        return;
    }
    // Directories are accepted whether or not they are executable. When
    // this is `expand_shellcmd` looking, do not use $PATH.
    let use_path = !flags.has(ExpandFlags::SHELLCMD);
    if !isdir && flags.has(ExpandFlags::EXEC) && !can_execute(f, use_path) {
        return;
    }

    let mut name = f.to_bytes().to_vec();
    if isdir && flags.has(ExpandFlags::ADDSLASH) {
        add_pathsep(&mut name);
    }
    names.push(XString::from_vec(name));
}

/// Does `fname` end in one of the `'suffixes'`?
///
/// An empty entry in `'suffixes'` stands for "a name with no dot in it".
pub fn match_suffix(fname: &[u8]) -> bool {
    let fname = cstr::until_nul(fname);
    let su = P_SU.get();
    let mut setsuf: &[u8] = &su;
    while !setsuf.is_empty() {
        let (suffix, rest) = split_option_part(setsuf, MAXSUFLEN as usize, b".,");
        setsuf = rest;
        if suffix.is_empty() {
            // An empty entry matches a name without a '.' in it.
            if !fname[tail_index(fname)..].contains(&b'.') {
                return true;
            }
        } else if fname.len() >= suffix.len()
            && path_fnamencmp(&suffix, &fname[fname.len() - suffix.len()..], suffix.len()) == 0
        {
            return true;
        }
    }
    false
}
