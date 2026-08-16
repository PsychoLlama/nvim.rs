//! Expanding one wildcard pattern against the file system.
//!
//! [`do_path_expand`] is the recursive walk: it takes the pattern apart one
//! component at a time, turns the component into a regexp, and reads each
//! directory that the components before it matched, recursing on `**` to the
//! depth the pattern asks for. [`addfile`] is what decides whether a name the
//! walk found belongs in the answer, and [`match_suffix`] ranks the results
//! by `'suffixes'`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::regexp::{RE_MAGIC, RE_NOBREAK};

/// How deep a `**` may recurse. Upstream's limit, and the reason a pattern
/// over a symlink loop terminates.
const MAX_STAR_DEPTH: c_int = 100;

/// The `**` recursion depth of the walk currently running. A global because
/// [`do_path_expand`] recurses through the file-system scan rather than
/// carrying state down.
static STARDEPTH: GlobalCell<c_int> = GlobalCell::new(0);

/// Does `p` hold a character that means expansion is needed — including a
/// leading `~`, unless it is the last character?
///
/// A backslash escapes the character after it.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn path_has_wildcard(p: *const c_char) -> bool {
    // SAFETY: the caller's promise.
    let p = unsafe { CStr::from_ptr(p) }.to_bytes();
    has_any_of(p, b"*?[{`'$", true)
}

/// Does `p` hold a character [`do_path_expand`] itself can expand? `~` and
/// `$` are not among them: those are the shell's and `expand_env`'s.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn path_has_exp_wildcard(p: *const c_char) -> bool {
    // SAFETY: the caller's promise.
    let p = unsafe { CStr::from_ptr(p) }.to_bytes();
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

/// `qsort` comparator putting two expanded names in [`pathcmp`] order.
///
/// Still a `qsort`: with `'fileignorecase'` two names that differ compare
/// equal, and which of them comes first is only what `qsort` decided.
///
/// # Safety
/// Both must name a `*mut c_char` holding a NUL-terminated string.
unsafe extern "C" fn pstrcmp(a: *const c_void, b: *const c_void) -> c_int {
    unsafe { pathcmp(*a.cast::<*const c_char>(), *b.cast::<*const c_char>(), -1) }
}

/// Expand `path`'s wildcards, adding every name that matches to `gap`.
///
/// # Safety
/// `path` must be a NUL-terminated string and `gap` an initialised array of
/// allocated strings.
pub(crate) unsafe fn path_expand(
    gap: *mut garray_T,
    path: *const c_char,
    flags: ExpandFlags,
) -> usize {
    unsafe { do_path_expand(gap, path, 0, flags, false) }
}

/// A directory scan that answers `"."` and `".."` before the entries libuv
/// reports, which do not include them.
struct ScanWithDots {
    dir: Directory,
    count: u32,
}

impl ScanWithDots {
    /// Read `path`, or answer `None` when it cannot be read at all.
    ///
    /// # Safety
    /// `path` must be a NUL-terminated string.
    unsafe fn open(path: *const c_char) -> Option<Self> {
        unsafe {
            let mut dir = Directory::default();
            if !os_file_is_readable(path) || !os_scandir(&raw mut dir, path) {
                return None;
            }
            Some(ScanWithDots { dir, count: 0 })
        }
    }

    /// The next entry's name, valid until the following call.
    unsafe fn next(&mut self) -> Option<&CStr> {
        unsafe {
            self.count += 1;
            let name = match self.count {
                1 => c".".as_ptr(),
                2 => c"..".as_ptr(),
                _ => os_scandir_next(&raw mut self.dir),
            };
            (!name.is_null()).then(|| CStr::from_ptr(name))
        }
    }
}

impl Drop for ScanWithDots {
    fn drop(&mut self) {
        // SAFETY: the request was filled by a successful `os_scandir`.
        unsafe { os_closedir(&raw mut self.dir) }
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
/// saying where that component is. Characters before `pattern + wildoff` are
/// never taken for wildcards.
///
/// # Safety
/// `buf` must have room for `pattern` and a NUL.
unsafe fn split_wild_component(
    pattern: &[u8],
    wildoff: usize,
    flags: ExpandFlags,
    buf: &mut [u8],
) -> Split {
    unsafe {
        // With every letter a wildcard, a name that differs only in case is
        // still a match.
        let icase = p_fic.get() == 0 && flags.has(ExpandFlags::ICASE);
        // Where the component starts, and whether a wildcard has turned up
        // in it yet.
        let mut comp = 0;
        let mut seen_wild = false;
        // The read and write positions advance together: nothing is dropped
        // here, only stopped at.
        let mut at = 0;
        while at < pattern.len() {
            let p = pattern.as_ptr().add(at).cast::<c_char>();
            if at >= wildoff && rem_backslash(p) {
                // The backslash stays for `file_pat_to_reg_pat` to drop; the
                // character it escapes is copied with the step below.
                buf[at] = pattern[at];
                at += 1;
            } else if vim_ispathsep_nocolon(pattern[at] as c_int) {
                if seen_wild {
                    break;
                }
                // A later component; the wildcard is not in this one.
                comp = at + 1;
            } else if at >= wildoff
                && (b"*?[{~$".contains(&pattern[at])
                    || (icase && mb_isalpha(utf_ptr2char(pattern.as_ptr().add(at).cast()))))
            {
                seen_wild = true;
            }
            // A character is copied whole, however many bytes it takes.
            let charlen = utfc_ptr2len(pattern.as_ptr().add(at).cast()) as usize;
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
            if rem_backslash(buf.as_ptr().add(at).cast()) {
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

/// Expand the wildcards in `path` into `gap`, one component at a time.
///
/// Characters before `path + wildoff` are taken literally — that is how the
/// recursion tells the part it has already resolved from the part it still
/// has to match. `didstar` says the caller has already recursed for a `**`,
/// so this level should not do it again.
///
/// Answers how many names were added.
///
/// # Safety
/// `path` must be a NUL-terminated string and `gap` an initialised array of
/// allocated strings.
pub(crate) unsafe fn do_path_expand(
    gap: *mut garray_T,
    path: *const c_char,
    wildoff: usize,
    flags: ExpandFlags,
    didstar: bool,
) -> usize {
    unsafe {
        let start_len = (*gap).ga_len;

        // Expanding "**" may take a long time; let CTRL-C out of it.
        if STARDEPTH.get() > 0 && !flags.has(ExpandFlags::NOBREAK) {
            os_breakcheck();
            if got_int.get() {
                return 0;
            }
        }

        let pattern = CStr::from_ptr(path).to_bytes();
        // Room for the pattern and any one name it grows into.
        let buflen = pattern.len() + MAXPATHL as usize;
        let mut buf = vec![0u8; buflen];
        let split = split_wild_component(pattern, wildoff, flags, &mut buf);
        let dir_len = split.comp;
        let rest = pattern.as_ptr().add(split.rest).cast::<c_char>();

        // A name starting with a dot is only matched by a pattern that does.
        let starts_with_dot = buf[dir_len] == b'.';
        let pat = file_pat_to_reg_pat(
            buf.as_ptr().add(dir_len).cast(),
            buf.as_ptr().add(split.comp_end).cast(),
            core::ptr::null_mut(),
            0,
        );
        if pat.is_null() {
            return 0;
        }
        let mut regmatch = regmatch_T {
            // Ignore case if given 'wildignorecase', else respect
            // 'fileignorecase'.
            rm_ic: flags.has(ExpandFlags::ICASE) || p_fic.get() != 0,
            ..Default::default()
        };
        let silent = flags.has(ExpandFlags::NOERROR | ExpandFlags::NOTWILD);
        if silent {
            *emsg_silent.ptr() += 1;
        }
        let nobreak = flags.has(ExpandFlags::NOBREAK);
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC | if nobreak { RE_NOBREAK } else { 0 });
        if silent {
            *emsg_silent.ptr() -= 1;
        }
        xfree(pat.cast());
        if regmatch.regprog.is_null() && !flags.has(ExpandFlags::NOTWILD) {
            return 0;
        }
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
            STARDEPTH.set(STARDEPTH.get() + 1);
            do_path_expand(gap, buf.as_ptr().cast(), dir_len, flags, true);
            STARDEPTH.set(STARDEPTH.get() - 1);
        }
        // Back to the directory the component lives in.
        buf[dir_len] = 0;

        let dirpath = if dir_len == 0 {
            c".".as_ptr()
        } else {
            buf.as_ptr().cast()
        };
        if let Some(mut scan) = ScanWithDots::open(dirpath) {
            while !got_int.get() {
                let Some(name) = scan.next() else { break };
                let name = name.to_bytes();
                if !name_is_wanted(name, starts_with_dot, flags)
                    || !name_matches(
                        &mut regmatch,
                        name,
                        flags,
                        &pattern[dir_len..],
                        split.comp_end - dir_len,
                    )
                {
                    continue;
                }
                let len = dir_len + write_at(&mut buf, dir_len, &[name]);
                if len + 1 >= buflen {
                    continue;
                }

                if split.starstar && STARDEPTH.get() < MAX_STAR_DEPTH {
                    // For "**" first go deeper in the tree to find matches.
                    write_at(&mut buf, len, &[b"/**", &pattern[split.rest..]]);
                    STARDEPTH.set(STARDEPTH.get() + 1);
                    do_path_expand(gap, buf.as_ptr().cast(), len + 1, flags, true);
                    STARDEPTH.set(STARDEPTH.get() - 1);
                }

                write_at(&mut buf, len, &[&pattern[split.rest..]]);
                if path_has_exp_wildcard(rest) {
                    // Another component to expand.
                    if STARDEPTH.get() < MAX_STAR_DEPTH {
                        STARDEPTH.set(STARDEPTH.get() + 1);
                        do_path_expand(gap, buf.as_ptr().cast(), len + 1, flags, false);
                        STARDEPTH.set(STARDEPTH.get() - 1);
                    }
                } else {
                    // No more wildcards: the escaping in what is left is the
                    // caller's, and comes off before the name is looked up.
                    if *rest != 0 {
                        backslash_halve(buf.as_mut_ptr().add(len + 1).cast());
                    }
                    let mut file_info = FileInfo::default();
                    let found = if flags.has(ExpandFlags::ALLLINKS) {
                        os_fileinfo_link(buf.as_ptr().cast(), &raw mut file_info)
                    } else {
                        os_path_exists(buf.as_ptr().cast())
                    };
                    if found {
                        addfile(gap, buf.as_mut_ptr().cast(), flags);
                    }
                }
            }
        }
        vim_regfree(regmatch.regprog);

        // When interrupted the matches probably won't be used, and sorting
        // can be slow.
        let matches = ((*gap).ga_len - start_len) as usize;
        if matches > 0 && !got_int.get() {
            qsort(
                (*gap)
                    .ga_data
                    .cast::<*mut c_char>()
                    .add(start_len as usize)
                    .cast(),
                matches,
                size_of::<*mut c_char>(),
                Some(pstrcmp),
            );
        }
        matches
    }
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
/// [`ExpandFlags::NOTWILD`] asked for the component's own text, `comp_len` bytes of
/// `comp`, to be compared literally.
///
/// # Safety
/// `comp` must hold at least `comp_len` bytes.
unsafe fn name_matches(
    regmatch: &mut regmatch_T,
    name: &[u8],
    flags: ExpandFlags,
    comp: &[u8],
    comp_len: usize,
) -> bool {
    unsafe {
        if !regmatch.regprog.is_null() && vim_regexec(regmatch, name.as_ptr().cast(), 0) {
            return true;
        }
        flags.has(ExpandFlags::NOTWILD)
            && path_fnamencmp(
                comp.as_ptr().cast(),
                name.as_ptr().cast(),
                comp_len as size_t,
            ) == 0
    }
}

/// Does `p` hold a wildcard character only a shell can expand?
///
/// A brace needs one when non-existing names are wanted, since only the
/// shell invents those; a backtick or quote needs one when it is matched.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub(crate) unsafe fn has_special_wildchar(p: *mut c_char, flags: ExpandFlags) -> bool {
    // SAFETY: the caller's promise.
    let bytes = unsafe { CStr::from_ptr(p) }.to_bytes();
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

/// Add the file `f` to `gap`, unless the `EW_*` flags say it does not
/// belong: a name that does not exist, a directory when only files were
/// asked for, a file that is not executable when only executables were.
///
/// # Safety
/// `f` must be a NUL-terminated string and `gap` an initialised array of
/// allocated strings.
pub unsafe fn addfile(gap: *mut garray_T, f: *mut c_char, flags: ExpandFlags) {
    unsafe {
        let mut file_info = FileInfo::default();
        let exists = if flags.has(ExpandFlags::ALLLINKS) {
            os_fileinfo_link(f, &raw mut file_info)
        } else {
            os_path_exists(f)
        };
        if !flags.has(ExpandFlags::NOTFOUND) && !exists {
            return;
        }

        let isdir = os_isdir(f);
        if isdir && !flags.has(ExpandFlags::DIR) || !isdir && !flags.has(ExpandFlags::FILE) {
            return;
        }
        // Directories are accepted whether or not they are executable. When
        // this is `expand_shellcmd` looking, do not use $PATH.
        if !isdir
            && flags.has(ExpandFlags::EXEC)
            && !os_can_exe(f, core::ptr::null_mut(), !flags.has(ExpandFlags::SHELLCMD))
        {
            return;
        }

        // Room for the name, its NUL, and the separator a directory gets.
        let name = CStr::from_ptr(f).to_bytes_with_nul();
        let p: *mut c_char = xmalloc(name.len() + usize::from(isdir)).cast();
        core::ptr::copy_nonoverlapping(name.as_ptr().cast(), p, name.len());
        if isdir && flags.has(ExpandFlags::ADDSLASH) {
            add_pathsep(p);
        }
        ga_grow(gap, 1);
        *(*gap)
            .ga_data
            .cast::<*mut c_char>()
            .add((*gap).ga_len as usize) = p;
        (*gap).ga_len += 1;
    }
}

/// Does `fname` end in one of the `'suffixes'`?
///
/// An empty entry in `'suffixes'` stands for "a name with no dot in it".
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn match_suffix(fname: *mut c_char) -> bool {
    unsafe {
        let mut suf_buf = [0 as c_char; MAXSUFLEN as usize];
        let fnamelen = CStr::from_ptr(fname).to_bytes().len();
        let mut setsuflen = 0;
        let mut setsuf = p_su.get();
        while *setsuf != 0 {
            setsuflen = copy_option_part(
                &raw mut setsuf,
                suf_buf.as_mut_ptr(),
                MAXSUFLEN as size_t,
                c".,".as_ptr().cast_mut(),
            ) as usize;
            if setsuflen == 0 {
                // An empty entry matches a name without a '.' in it.
                if vim_strchr(path_tail(fname), c_int::from(b'.')).is_null() {
                    setsuflen = 1;
                    break;
                }
            } else {
                if fnamelen >= setsuflen
                    && path_fnamencmp(
                        suf_buf.as_ptr(),
                        fname.add(fnamelen - setsuflen),
                        setsuflen as size_t,
                    ) == 0
                {
                    break;
                }
                setsuflen = 0;
            }
        }
        setsuflen != 0
    }
}
