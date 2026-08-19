//! Finding and sourcing files along 'runtimepath' -- `:runtime` and everything
//! built on it.
//!
//! [`do_in_path`] is the primitive: split a path list on commas, glob each
//! entry against a pattern, and hand every match to a callback, optionally
//! stopping at the first.  [`do_in_path_and_pp`] adds 'packpath''s
//! `pack/*/start` and `pack/*/opt` trees for the `RuntimeOpts::START`/`RuntimeOpts::OPT` flags,
//! and [`do_in_runtimepath`] is the 'runtimepath' entry point that prefers the
//! cached search path when there is one (see [`super::cache`]).  The
//! `source_runtime*` wrappers pick the callback that sources what was found,
//! with the Vim-then-Lua ordering `:runtime` promises; [`runtime_get_named`]
//! and [`runtime_inspect`] are the API's read-only views of the same search.
//!
//! The pattern walk itself lives in [`expand_name_patterns`], which the cached
//! path shares: the two searches differ in where the directory list comes from
//! and in one `ExpandFlags::NOBREAK`, not in what they do with `{name}`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::path::ExpandFlags;

use crate::types::{FAIL, MAXPATHL, OK};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

/// The `[where]` qualifiers `:runtime` accepts, and the `DIP_*` set each one
/// selects.  Upstream sums the flags; they are disjoint bits, so this is the
/// same number.
const WHERE_FLAGS: [(&CStr, RuntimeOpts); 4] = [
    (c"START", RuntimeOpts::START.or(RuntimeOpts::NORTP)),
    (c"OPT", RuntimeOpts::OPT.or(RuntimeOpts::NORTP)),
    (
        c"PACK",
        RuntimeOpts::START
            .or(RuntimeOpts::OPT)
            .or(RuntimeOpts::NORTP),
    ),
    (c"ALL", RuntimeOpts::START.or(RuntimeOpts::OPT)),
];

/// Get the `DIP_*` flags from the `[where]` argument of a `:runtime` command,
/// advancing `*argp` past it.
///
/// The comparison is `strncmp` over `where_len` bytes, so a *prefix* of a
/// qualifier selects it: `:runtime STA foo` really does mean `START`.
///
/// # Safety
/// `*argp` must be a NUL-terminated string with at least `where_len` bytes
/// before its terminator.
unsafe fn get_runtime_cmd_flags(argp: *mut *mut c_char, where_len: size_t) -> RuntimeOpts {
    if where_len == 0 {
        return RuntimeOpts::NONE;
    }
    // SAFETY: the caller's out-parameter holds the argument to look at.
    let arg = unsafe { *argp };
    for (keyword, flags) in WHERE_FLAGS {
        // SAFETY: both strings are NUL-terminated and `arg` has `where_len`
        // bytes; `skipwhite` stops at the terminator.
        unsafe {
            if strncmp(arg, keyword.as_ptr(), where_len) == 0 {
                *argp = skipwhite(arg.add(where_len));
                return flags;
            }
        }
    }
    RuntimeOpts::NONE
}

/// `:runtime[!] [where] {name}`.
pub unsafe fn ex_runtime(eap: *mut exarg_T) {
    // SAFETY: `eap` is the live command being executed; `arg` is its
    // NUL-terminated argument text.
    unsafe {
        let mut arg = (*eap).arg;
        let mut flags = if (*eap).forceit != 0 {
            RuntimeOpts::ALL
        } else {
            RuntimeOpts::NONE
        };
        let where_len = skiptowhite(arg).offset_from(arg) as size_t;
        flags |= get_runtime_cmd_flags(&raw mut arg, where_len);
        debug_assert!(!arg.is_null(), "arg != NULL");
        source_runtime(arg, flags);
    }
}

/// Set the completion context for the `:runtime` command.
///
/// The `[where]` qualifier is only offered for a single-argument command line;
/// past the first argument [`runtime_expand_flags`] is forced non-zero so
/// [`expand_runtime_cmd`] stops proposing the qualifiers.
pub unsafe fn set_context_in_runtime_cmd(xp: *mut expand_T, arg: *const c_char) {
    // SAFETY: `arg` is the NUL-terminated command line tail and `xp` is the
    // live expansion context.
    unsafe {
        let mut arg = arg.cast_mut();
        let mut p = skiptowhite(arg);
        runtime_expand_flags.set(if *p != 0 {
            get_runtime_cmd_flags(&raw mut arg, p.offset_from(arg) as size_t)
        } else {
            RuntimeOpts::NONE
        });
        // Skip to the last argument.
        loop {
            p = skiptowhite_esc(arg);
            if *p == 0 {
                break;
            }
            if runtime_expand_flags.get() == RuntimeOpts::NONE {
                // With multiple arguments and no [where], an unrelated
                // non-zero flag keeps [where] out of the completion.
                runtime_expand_flags.set(RuntimeOpts::ALL);
            }
            arg = skipwhite(p);
        }
        (*xp).xp_context = EXPAND_RUNTIME;
        (*xp).xp_pattern = arg;
    }
}

/// Source every name `accept` picks out, stopping after the first unless
/// `all`.  Answers whether anything was sourced.
///
/// # Safety
/// `fnames` must hold live NUL-terminated file names, and `cookie` must be
/// what [`do_source`] takes as its `ret_sid` out-parameter.
unsafe fn source_matching(
    fnames: &[*mut c_char],
    all: bool,
    cookie: *mut c_void,
    accept: impl Fn(*mut c_char) -> bool,
) -> bool {
    let mut did_one = false;
    for &fname in fnames {
        if !accept(fname) {
            continue;
        }
        // SAFETY: `fname` is one of the caller's file names; `cookie` is its
        // `int *ret_sid`.
        unsafe { do_source(fname, false, DOSO_NONE, cookie.cast::<c_int>()) };
        did_one = true;
        if !all {
            break;
        }
    }
    did_one
}

/// Whether `fname` ends in `.{ext}`.
fn has_extension(fname: *mut c_char, ext: &CStr) -> bool {
    // SAFETY: `fname` and `ext` are NUL-terminated.
    unsafe { path_with_extension(fname, ext.as_ptr()) }
}

/// The matches as a slice.
///
/// # Safety
/// `fnames` must hold `num_fnames` entries and stay put for the borrow.
pub(crate) unsafe fn matches<'a>(num_fnames: c_int, fnames: *mut *mut c_char) -> &'a [*mut c_char] {
    if fnames.is_null() || num_fnames <= 0 {
        return &[];
    }
    // SAFETY: the caller's array, `num_fnames` long.
    unsafe { slice::from_raw_parts(fnames, num_fnames as usize) }
}

/// Source all `.vim` and `.lua` files in `fnames`, `.vim` files first.
///
/// # Safety
/// As [`source_matching`].
pub(crate) unsafe fn source_callback_vim_lua(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    // SAFETY: the callback contract: `fnames` holds `num_fnames` names.
    let fnames = unsafe { matches(num_fnames, fnames) };
    // SAFETY: as above.
    let did_one = unsafe { source_matching(fnames, all, cookie, |f| has_extension(f, c"vim")) };
    if !all && did_one {
        return true;
    }
    // SAFETY: as above.
    did_one | unsafe { source_matching(fnames, all, cookie, |f| has_extension(f, c"lua")) }
}

/// Source all files in `fnames`: `.vim` first, then `.lua`, then the rest.
///
/// # Safety
/// As [`source_matching`].
pub(crate) unsafe fn source_callback(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    // SAFETY: the callback contract, as in `source_callback_vim_lua`.
    let did_one = unsafe { source_callback_vim_lua(num_fnames, fnames, all, cookie) };
    if !all && did_one {
        return true;
    }
    // SAFETY: as above.
    let fnames = unsafe { matches(num_fnames, fnames) };
    // SAFETY: as above.
    did_one
        | unsafe {
            source_matching(fnames, all, cookie, |f| {
                !has_extension(f, c"vim") && !has_extension(f, c"lua")
            })
        }
}

/// What a search does with what it finds: the callback and the cookie handed
/// to it, which never travel apart.
#[derive(Clone, Copy)]
pub(crate) struct Visitor {
    pub(crate) callback: DoInRuntimepathCB,
    pub(crate) cookie: *mut c_void,
}

impl Visitor {
    /// Hand `files` over.
    ///
    /// The callback is never `None` at any call site — upstream's parameter is
    /// a bare function pointer, and c2rust wrapped it.
    ///
    /// # Safety
    /// `files` must hold `num_files` names the callback may read.
    unsafe fn invoke(self, num_files: c_int, files: *mut *mut c_char, all: bool) -> bool {
        let callback = self.callback.expect("do_in_path callback");
        // SAFETY: the caller's array, matched to the callback's contract.
        unsafe { callback(num_files, files, all, self.cookie) }
    }

    /// Hand over a single name.
    ///
    /// # Safety
    /// `fname` must be NUL-terminated.
    pub(crate) unsafe fn invoke_one(self, fname: *mut c_char, all: bool) -> bool {
        let mut one = [fname];
        // SAFETY: a one-element name array, which is what the callback reads.
        unsafe { self.invoke(1, one.as_mut_ptr(), all) }
    }
}

/// The [`ExpandFlags`] a `DIP_*` set asks a wildcard expansion for.
pub(crate) fn wildcard_flags(flags: RuntimeOpts) -> ExpandFlags {
    (if flags.has(RuntimeOpts::DIR) {
        ExpandFlags::DIR
    } else {
        ExpandFlags::FILE
    }) | (if flags.has(RuntimeOpts::DIRFILE) {
        ExpandFlags::DIR | ExpandFlags::FILE
    } else {
        ExpandFlags::NONE
    })
}

/// Whether `flags` asks for this entry to be skipped for being — or for not
/// being — an `after/` directory.
pub(crate) fn skips_entry(flags: RuntimeOpts, is_after: bool) -> bool {
    flags.has(RuntimeOpts::NOAFTER | RuntimeOpts::AFTER)
        && (is_after && flags.has(RuntimeOpts::NOAFTER)
            || !is_after && flags.has(RuntimeOpts::AFTER))
}

/// Expand each whitespace-separated pattern of `name` in turn at `tail` and
/// invoke `callback` for the matches.
///
/// `buf` holds one directory, path separator and prefix included, up to
/// `tail`; the patterns are written there one after another.  Stops at the
/// first pattern that matched unless `do_all`, and folds what it finds into
/// the caller's running `did_one` — which is also what its own loop tests, so
/// a match found for an earlier directory ends the walk here too.
///
/// # Safety
/// `buf` must be writable for `MAXPATHL` bytes, `tail` must point into it, and
/// `name` must be a NUL-terminated `\t `-separated pattern list.
pub(crate) unsafe fn expand_name_patterns(
    buf: *mut c_char,
    tail: *mut c_char,
    name: *mut c_char,
    ew_flags: ExpandFlags,
    do_all: bool,
    did_one: &mut bool,
    visitor: Visitor,
) {
    let mut np = name;
    // SAFETY: `np` walks the caller's NUL-terminated pattern list, and `tail`
    // points into `buf`, so the room left there is the difference.
    while unsafe { *np } != 0 && (do_all || !*did_one) {
        let used = unsafe { tail.offset_from(buf) };
        debug_assert!(
            (0..=MAXPATHL as isize).contains(&used),
            "MAXPATHL >= tail - buf"
        );
        // SAFETY: `copy_option_part` writes at most the room it is given and
        // NUL-terminates within it.
        unsafe {
            copy_option_part(
                &raw mut np,
                tail,
                (MAXPATHL as isize - used) as size_t,
                c"\t ".as_ptr().cast_mut(),
            );
        }
        if p_verbose.get() > 10 {
            // SAFETY: `buf` now holds the NUL-terminated candidate.
            unsafe {
                verbose_enter();
                smsg_c!(0, gettext(c"Searching for \"%s\"".as_ptr()), buf);
                verbose_leave();
            }
        }
        let mut pats = [buf];
        // SAFETY: a one-element pattern array; `gen_expand_wildcards` only
        // reads it.
        *did_one |=
            unsafe { gen_expand_wildcards_and_cb(1, pats.as_mut_ptr(), ew_flags, do_all, visitor) }
                == OK;
    }
}

/// The `p_verbose > 10` announcement `do_in_path` makes before it starts.
///
/// # Safety
/// All three must be NUL-terminated; `prefix` may be empty but not null.
unsafe fn announce_search(name: *mut c_char, prefix: *const c_char, path: *const c_char) {
    // SAFETY: the caller's NUL-terminated strings, formatted by `vim_snprintf`.
    unsafe {
        verbose_enter();
        if *prefix != 0 {
            smsg_c!(
                0,
                gettext(c"Searching for \"%s\" under \"%s\" in \"%s\"".as_ptr()),
                name,
                prefix,
                path,
            );
        } else {
            smsg_c!(
                0,
                gettext(c"Searching for \"%s\" in \"%s\"".as_ptr()),
                name,
                path
            );
        }
        verbose_leave();
    }
}

/// Find the patterns in `name` in all directories in `path` and invoke
/// `callback` for each match.  `prefix` is prepended to each pattern.
///
/// `RuntimeOpts::ALL` visits every match rather than stopping at the first, `RuntimeOpts::DIR`
/// looks for directories, and `RuntimeOpts::ERR` turns "nothing found" into an error
/// message rather than a verbose note.
///
/// Answers OK when something was found, FAIL otherwise.
///
/// # Safety
/// `path` and `prefix` must be NUL-terminated (`prefix` may be empty), `name`
/// may be null, and `callback` must accept `cookie`.
pub unsafe fn do_in_path(
    path: *const c_char,
    prefix: *const c_char,
    name: *mut c_char,
    flags: RuntimeOpts,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    let visitor = Visitor { callback, cookie };
    // Copy the path list: invoking the callback may change the option it came
    // from.
    // SAFETY: `path` is NUL-terminated.
    let rtp_copy = unsafe { xstrdup(path) };
    let buf = unsafe { xmallocz(MAXPATHL as size_t) }.cast::<c_char>();

    if p_verbose.get() > 10 && !name.is_null() {
        // SAFETY: the caller's strings.
        unsafe { announce_search(name, prefix, path) };
    }

    let do_all = flags.has(RuntimeOpts::ALL);
    let mut did_one = false;
    let mut rtp = rtp_copy;
    // SAFETY: `rtp` walks the copy; `buf` has `MAXPATHL` writable bytes.
    while unsafe { *rtp } != 0 && (do_all || !did_one) {
        // SAFETY: as above.
        let buflen = unsafe {
            copy_option_part(
                &raw mut rtp,
                buf,
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        // SAFETY: `buf` holds `buflen` bytes plus a terminator.
        if skips_entry(flags, unsafe { path_is_after(buf, buflen) }) {
            continue;
        }
        if name.is_null() {
            // SAFETY: `buf` holds the directory, NUL-terminated.
            unsafe { visitor.invoke_one(buf, do_all) };
            did_one = true;
            continue;
        }
        // SAFETY: the three strings are NUL-terminated.
        let room_needed =
            buflen + 2 + unsafe { strlen(prefix) } + unsafe { strlen(name) } < MAXPATHL as size_t;
        if !room_needed {
            continue;
        }
        // SAFETY: the length test above proves the directory, its separator
        // and the prefix fit, so `tail` lands inside `buf`.
        let tail = unsafe {
            add_pathsep(buf);
            strcat(buf, prefix);
            buf.add(strlen(buf))
        };
        // SAFETY: as documented on `expand_name_patterns`.
        unsafe {
            expand_name_patterns(
                buf,
                tail,
                name,
                wildcard_flags(flags),
                do_all,
                &mut did_one,
                visitor,
            );
        }
    }

    // SAFETY: both were allocated above and are no longer referenced.
    unsafe {
        xfree(buf.cast());
        xfree(rtp_copy.cast());
    }

    if !did_one && !name.is_null() {
        let basepath = if path == p_rtp.get().cast_const() {
            c"runtimepath"
        } else {
            c"packpath"
        };
        // SAFETY: `basepath` is a literal and `name` the caller's pattern.
        unsafe {
            if flags.has(RuntimeOpts::ERR) {
                semsg_c!(
                    gettext(&raw const e_dirnotf as *const c_char),
                    basepath.as_ptr(),
                    name,
                );
            } else if p_verbose.get() > 1 {
                verbose_enter();
                smsg_c!(
                    0,
                    gettext(c"not found in '%s': \"%s\"".as_ptr()),
                    basepath.as_ptr(),
                    name,
                );
                verbose_leave();
            }
        }
    }

    if did_one { OK } else { FAIL }
}

fn boolean_obj(value: bool) -> Object {
    Object {
        type_0: kObjectTypeBoolean,
        data: object_data { boolean: value },
    }
}

pub(crate) fn integer_obj(value: Integer) -> Object {
    Object {
        type_0: kObjectTypeInteger,
        data: object_data { integer: value },
    }
}

fn string_obj(string: String_0) -> Object {
    Object {
        type_0: kObjectTypeString,
        data: object_data { string },
    }
}

fn dict_obj(dict: Dict) -> Object {
    Object {
        type_0: kObjectTypeDict,
        data: object_data { dict },
    }
}

/// `nvim__runtime_inspect()`: the cached search path as it stands.
///
/// Note that this reads the cache without validating it — see
/// [`super::cache`], and the trap that in `nvim -l` script mode nothing else
/// rebuilds it either.
///
/// # Safety
/// `arena` may be null; the strings borrow the cache and live as long as it.
pub unsafe fn runtime_inspect(arena: *mut Arena) -> Array {
    let path = runtime_search_path.get();
    let mut rv = arena_array(arena, path.size);
    for i in 0..path.size {
        // SAFETY: `path` holds `size` live items.
        let item = unsafe { *path.items.add(i) };
        let mut entry = arena_dict(arena, 5);
        // SAFETY: `entry` was sized for the five keys below, `item.path` is
        // the entry's own NUL-terminated directory, and `rv` for `size` items.
        unsafe {
            dict_put(&mut entry, c"path", string_obj(cstr_as_string(item.path)));
            if item.after {
                dict_put(&mut entry, c"after", boolean_obj(true));
            }
            if item.pack_inserted {
                dict_put(&mut entry, c"pack_inserted", boolean_obj(true));
            }
            if let Some(has_lua) = item.has_lua {
                dict_put(&mut entry, c"has_lua", boolean_obj(has_lua));
            }
            dict_put(
                &mut entry,
                c"pos_in_rtp",
                integer_obj(item.pos_in_rtp as Integer),
            );
            array_add(&mut rv, dict_obj(entry));
        }
    }
    rv
}

/// `nvim__get_runtime()`: the readable files named by `pat` along the cached
/// search path.
///
/// # Safety
/// `pat` must hold `size` objects; `arena` may be null.
pub unsafe fn runtime_get_named(lua: bool, pat: Array, all: bool, arena: *mut Arena) -> Array {
    let mut ref_0: c_int = 0;
    // SAFETY: the reference is released below, before this frame ends.
    unsafe {
        let path = runtime_search_path_get_cached(&raw mut ref_0);
        let mut buf = [0 as c_char; MAXPATHL as usize];
        let rv = runtime_get_named_common(lua, pat, all, path, &mut buf, arena);
        runtime_search_path_unref(path, &raw const ref_0);
        rv
    }
}

/// [`runtime_get_named`] for a worker thread, against the snapshot
/// [`update_runtime_search_path_thread`] keeps for exactly this.
///
/// # Safety
/// As [`runtime_get_named`]. Called off the main thread; nothing here may
/// touch main-thread-only editor state.
pub unsafe fn runtime_get_named_thread(lua: bool, pat: Array, all: bool) -> Array {
    // TODO(bfredl): avoid contention between multiple worker threads?
    // SAFETY: the mutex is initialised by `runtime_init` before any thread
    // exists, and guards every access to the snapshot on both sides.
    unsafe {
        uv_mutex_lock(search_path_mutex());
        let mut buf = [0 as c_char; MAXPATHL as usize];
        let rv = runtime_get_named_common(
            lua,
            pat,
            all,
            runtime_search_path_thread.get(),
            &mut buf,
            ptr::null_mut(),
        );
        uv_mutex_unlock(search_path_mutex());
        rv
    }
}

/// Whether this search-path entry has a `lua/` subdirectory.
///
/// The answer is cached in the entry, which is why it is written through a
/// pointer: the array belongs to the process-wide search path (or to the
/// thread snapshot), not to this call.
///
/// # Safety
/// `item` must be a live entry of one of those arrays, and `buf` is scratch.
unsafe fn dir_has_lua(item: *mut SearchPathItem, buf: &mut [c_char]) -> bool {
    // SAFETY: the caller's live entry; `snprintf` NUL-terminates within `buf`.
    unsafe {
        if (*item).has_lua.is_none() {
            let size = snprintf(
                buf.as_mut_ptr(),
                buf.len(),
                c"%s/lua/".as_ptr(),
                (*item).path,
            ) as size_t;
            (*item).has_lua = Some(size < buf.len() && os_isdir(buf.as_mut_ptr()));
        }
        (*item).has_lua != Some(false)
    }
}

/// The shared body of [`runtime_get_named`] and its thread variant.
///
/// # Safety
/// `path` must be a live search path, `pat` must hold `size` objects, and
/// `arena` may be null.
unsafe fn runtime_get_named_common(
    lua: bool,
    pat: Array,
    all: bool,
    path: RuntimeSearchPath,
    buf: &mut [c_char],
    arena: *mut Arena,
) -> Array {
    let mut rv = arena_array(arena, path.size.wrapping_mul(pat.size));
    // SAFETY: `pat` holds `size` objects.
    let pats = unsafe { matches_of(pat) };
    for i in 0..path.size {
        // SAFETY: `path` holds `size` live items.
        let item = unsafe { path.items.add(i) };
        // SAFETY: as above.
        if lua && !unsafe { dir_has_lua(item, buf) } {
            continue;
        }
        for pat_item in pats {
            if pat_item.type_0 != kObjectTypeString as ObjectType {
                continue;
            }
            // SAFETY: the object is a string, so its union holds one; `buf`
            // is NUL-terminated by `snprintf` within its length.
            unsafe {
                let size = snprintf(
                    buf.as_mut_ptr(),
                    buf.len(),
                    c"%s/%s".as_ptr(),
                    (*item).path,
                    pat_item.data.string.data,
                ) as size_t;
                if size >= buf.len() || !os_file_is_readable(buf.as_mut_ptr()) {
                    continue;
                }
                array_add(
                    &mut rv,
                    string_obj(arena_string(arena, cstr_as_string(buf.as_ptr()))),
                );
            }
            if !all {
                return rv;
            }
        }
    }
    rv
}

/// An API array's items as a slice.
///
/// # Safety
/// `array` must hold `size` objects that stay put for the borrow.
unsafe fn matches_of<'a>(array: Array) -> &'a [Object] {
    if array.items.is_null() || array.size == 0 {
        return &[];
    }
    // SAFETY: the caller's array, `size` long.
    unsafe { slice::from_raw_parts(array.items, array.size) }
}

/// Find `name` in `path`, and then — for `RuntimeOpts::START`/`RuntimeOpts::OPT` — in
/// 'packpath''s package trees, invoking `callback` for each match.
///
/// Answers OK when at least one match was found.  With `name` null the
/// callback is invoked once per directory instead.
///
/// # Safety
/// As [`do_in_path`].
pub unsafe fn do_in_path_and_pp(
    path: *mut c_char,
    name: *mut c_char,
    flags: RuntimeOpts,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    // An empty `name` means "every directory", which `do_in_path` spells NULL.
    // SAFETY: `name` is null or NUL-terminated.
    let dirs_only = if !name.is_null() && unsafe { *name } == 0 {
        ptr::null_mut()
    } else {
        name
    };
    let mut done = FAIL;
    // Each round is skipped once something has been found, unless RuntimeOpts::ALL.
    let wants_more = |done: c_int| done == FAIL || flags.has(RuntimeOpts::ALL);

    if !flags.has(RuntimeOpts::NORTP) {
        // SAFETY: the caller's strings and callback.
        done |= unsafe { do_in_path(path, c"".as_ptr(), dirs_only, flags, callback, cookie) };
    }

    if wants_more(done) && flags.has(RuntimeOpts::START) {
        let after = flags.has(RuntimeOpts::AFTER);
        // The `after/` variants are searched under the package, so `AFTER`
        // is spent here and must not filter the packpath entries as well.
        let start_flags = flags.without(RuntimeOpts::AFTER);
        for prefix in [
            if after {
                c"pack/*/start/*/after/"
            } else {
                c"pack/*/start/*/"
            },
            if after {
                c"start/*/after/"
            } else {
                c"start/*/"
            },
        ] {
            // SAFETY: as above.
            done |= unsafe {
                do_in_path(
                    p_pp.get(),
                    prefix.as_ptr(),
                    name,
                    start_flags,
                    callback,
                    cookie,
                )
            };
            if !wants_more(done) {
                break;
            }
        }
    }

    if wants_more(done) && flags.has(RuntimeOpts::OPT) {
        for prefix in [c"pack/*/opt/*/", c"opt/*/"] {
            // SAFETY: as above.
            done |=
                unsafe { do_in_path(p_pp.get(), prefix.as_ptr(), name, flags, callback, cookie) };
            if !wants_more(done) {
                break;
            }
        }
    }

    done
}

/// [`do_in_path_and_pp`] over 'runtimepath', preferring the cached search
/// path.
///
/// # Safety
/// As [`do_in_path`].
pub unsafe fn do_in_runtimepath(
    name: *mut c_char,
    mut flags: RuntimeOpts,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    let mut success = FAIL;
    if !flags.has(RuntimeOpts::NORTP) {
        // SAFETY: `name` is null or NUL-terminated.
        let dirs_only = if !name.is_null() && unsafe { *name } == 0 {
            ptr::null_mut()
        } else {
            name
        };
        // SAFETY: the caller's callback and cookie.
        success |= unsafe { do_in_cached_path(dirs_only, flags, callback, cookie) };
        // The cached path already covers 'runtimepath' and the `start`
        // packages spliced into it.
        flags = flags.without(RuntimeOpts::START) | RuntimeOpts::NORTP;
    }
    // TODO(bfredl): we could integrate disabled OPT dirs into the cached path,
    // which would make ":packadd myoptpack" effective as well.
    if flags.has(RuntimeOpts::START | RuntimeOpts::OPT)
        && (success == FAIL || flags.has(RuntimeOpts::ALL))
    {
        // SAFETY: as above.
        success |= unsafe { do_in_path_and_pp(p_rtp.get(), name, flags, callback, cookie) };
    }
    success
}

/// Source the file `name` from all directories in 'runtimepath'.  `name` may
/// contain wildcards; `RuntimeOpts::ALL` sources every match rather than the first.
///
/// # Safety
/// `name` must be NUL-terminated.
pub unsafe fn source_runtime(name: *mut c_char, flags: RuntimeOpts) -> c_int {
    // SAFETY: `source_callback` takes a null cookie.
    unsafe {
        do_in_runtimepath(
            name,
            flags,
            Some(source_callback as DoInRuntimepathCBFn),
            ptr::null_mut(),
        )
    }
}

/// [`source_runtime`], but only `.vim` and `.lua` files.
///
/// # Safety
/// As [`source_runtime`].
pub unsafe fn source_runtime_vim_lua(name: *mut c_char, flags: RuntimeOpts) -> c_int {
    // SAFETY: as `source_runtime`.
    unsafe {
        do_in_runtimepath(
            name,
            flags,
            Some(source_callback_vim_lua as DoInRuntimepathCBFn),
            ptr::null_mut(),
        )
    }
}

/// [`source_runtime`] over `path` instead of 'runtimepath', and only `.vim`
/// and `.lua` files.
///
/// # Safety
/// Both must be NUL-terminated.
pub unsafe fn source_in_path_vim_lua(
    path: *mut c_char,
    name: *mut c_char,
    flags: RuntimeOpts,
) -> c_int {
    // SAFETY: as `source_runtime`.
    unsafe {
        do_in_path_and_pp(
            path,
            name,
            flags,
            Some(source_callback_vim_lua as DoInRuntimepathCBFn),
            ptr::null_mut(),
        )
    }
}

/// Expand the wildcards in `pats` and invoke `callback` for the matches.
///
/// Answers OK when files were found, FAIL otherwise.  `all` is passed through
/// to the callback, which decides whether to act on more than the first.
///
/// # Safety
/// `pats` must hold `num_pat` NUL-terminated patterns.
pub(crate) unsafe fn gen_expand_wildcards_and_cb(
    num_pat: c_int,
    pats: *mut *mut c_char,
    flags: ExpandFlags,
    all: bool,
    visitor: Visitor,
) -> c_int {
    let mut num_files: c_int = 0;
    let mut files: *mut *mut c_char = ptr::null_mut();
    // SAFETY: the caller's patterns; the two out-parameters are ours.
    unsafe {
        if gen_expand_wildcards(num_pat, pats, &raw mut num_files, &raw mut files, flags) != OK {
            return FAIL;
        }
        visitor.invoke(num_files, files, all);
        FreeWild(num_files, files);
    }
    OK
}
