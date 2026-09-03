//! The precomputed 'runtimepath' search path, and the thread-safe copy of it.
//!
//! Walking 'runtimepath' *and* 'packpath' for every `:runtime` is O(entries)
//! filesystem calls, so the resolved, ordered, deduplicated list of directories
//! is built once ([`runtime_search_path_build`]) and reused until either option
//! changes.  `after/` directories sort last and `pack/*/start` entries are
//! spliced in where the pack was found, which is the ordering the whole plugin
//! ecosystem depends on and which nothing else in the tree recomputes.
//!
//! There are two copies.  The main one is invalidated by
//! [`did_set_runtimepackpath`] and rebuilt lazily by
//! [`runtime_search_path_validate`]; the second is a refcounted snapshot taken
//! under a mutex for the *thread* that serves `nvim_get_runtime_file` off the
//! main loop, which is why [`copy_runtime_search_path`] and
//! [`runtime_search_path_unref`] exist at all.
//!
//! # The refcount
//!
//! `runtime_search_path_ref` is a single borrow slot, not a count: the first
//! reader of an unowned cache parks the address of its own `int` there and
//! becomes the owner.  A rebuild while somebody owns the cache leaves the old
//! copy alone and gives the new readers a fresh one, and whoever owns it frees
//! it on the way out ([`runtime_search_path_unref`]).  That is what lets
//! `do_in_cached_path` source files that themselves change 'runtimepath'.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::memory::xstrlcpy;
use crate::message_fmt::c_str;
use crate::path::ExpandFlags;
use crate::semsg;
use crate::smsg;

use crate::types::{FAIL, MAXPATHL, OK};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// The mutex guarding [`runtime_search_path_thread`], named once so the raw
/// handle has a single spelling.
///
/// A [`SharedCell`], so this is *not* a main-thread-only accessor: the whole
/// point of the snapshot is that a worker thread reads it.
pub(crate) fn search_path_mutex() -> *mut uv_mutex_t {
    runtime_search_path_mutex.ptr()
}

/// Initialise the runtime family's process-wide state.
pub unsafe fn runtime_init() {
    // SAFETY: called once at startup, before any thread can reach the mutex.
    unsafe { uv_mutex_init(search_path_mutex()) };
}

impl RuntimeSearchPath {
    /// Hand a built path over to the cache.
    ///
    /// The three fields *are* a `Vec`'s: the struct stays because the cache
    /// is shared by a hand-rolled borrow slot and by a second copy the
    /// worker threads read, neither of which a `Vec` can express. Everything
    /// that builds or walks one goes through here.
    ///
    /// An empty `Vec`'s pointer is dangling rather than heap, so an empty
    /// path is stored as the null [`runtime_search_path_free`] accepts.
    pub(crate) fn from_vec(path: Vec<SearchPathItem>) -> Self {
        if path.capacity() == 0 {
            return RuntimeSearchPath {
                size: 0,
                capacity: 0,
                items: ptr::null_mut(),
            };
        }
        let mut path = core::mem::ManuallyDrop::new(path);
        RuntimeSearchPath {
            size: path.len(),
            capacity: path.capacity(),
            items: path.as_mut_ptr(),
        }
    }

    /// The path's items, as a slice.
    ///
    /// # Safety
    /// The path must be the live cache or a copy nothing has freed.
    pub(crate) unsafe fn as_slice<'a>(&self) -> &'a [SearchPathItem] {
        if self.items.is_null() {
            // `from_raw_parts` rejects a null base even for an empty slice.
            return &[];
        }
        // SAFETY: the caller's path, `size` items long.
        unsafe { core::slice::from_raw_parts(self.items, self.size) }
    }

    /// Take a path back, to grow or to free it.
    ///
    /// # Safety
    /// Nothing else may still be walking this path -- the borrow slot in
    /// [`runtime_search_path_get_cached`] is what decides that.
    pub(crate) unsafe fn into_vec(self) -> Vec<SearchPathItem> {
        if self.items.is_null() {
            return Vec::new();
        }
        // SAFETY: the caller's path, which only `from_vec` above ever
        // filled, so the three fields are a `Vec`'s own.
        unsafe { Vec::from_raw_parts(self.items, self.size, self.capacity) }
    }
}

/// Borrow the cached search path, validating it first.
///
/// The `ref_0` out-parameter is the caller's borrow token; it must be handed
/// back to [`runtime_search_path_unref`] before the caller's frame ends.
///
/// # Safety
/// `ref_0` must point at a local `int` that outlives the borrow.
pub(crate) unsafe fn runtime_search_path_get_cached(ref_0: *mut c_int) -> RuntimeSearchPath {
    // SAFETY: rebuilding may source files, which is why this happens before
    // the borrow is taken.
    unsafe { runtime_search_path_validate() };

    // SAFETY: the caller's local.
    unsafe { *ref_0 = 0 };
    if runtime_search_path_ref.get().is_null() {
        // The cache was unreferenced: take it, so a rebuild underneath us
        // does not free what we are about to walk.
        // SAFETY: as above.
        unsafe { *ref_0 += 1 };
        runtime_search_path_ref.set(ref_0);
    }
    runtime_search_path.get()
}

/// A deep copy of `src`, paths included.
///
/// # Safety
/// `src` must be a live search path.
unsafe fn copy_runtime_search_path(src: RuntimeSearchPath) -> RuntimeSearchPath {
    // SAFETY: the caller's live path.
    let src = unsafe { src.as_slice() };
    let mut dst = Vec::with_capacity(src.len());
    for &item in src {
        // SAFETY: each item owns its own NUL-terminated directory.
        dst.push(SearchPathItem {
            path: unsafe { xstrdup(item.path) },
            ..item
        });
    }
    RuntimeSearchPath::from_vec(dst)
}

/// Release a borrow taken by [`runtime_search_path_get_cached`], freeing the
/// path when this borrow outlived a rebuild.
///
/// # Safety
/// `ref_0` must be the same token, and `path` the same value, the matching
/// `get_cached` handed out.
pub(crate) unsafe fn runtime_search_path_unref(path: RuntimeSearchPath, ref_0: *const c_int) {
    // SAFETY: the caller's token.
    if unsafe { *ref_0 } == 0 {
        return;
    }
    if runtime_search_path_ref.get() == ref_0.cast_mut() {
        // Still the live cache: hand it back unowned.
        runtime_search_path_ref.set(ptr::null_mut());
    } else {
        // A rebuild replaced it while we were reading; this copy is ours.
        // SAFETY: nothing else refers to it any more.
        unsafe { runtime_search_path_free(path) };
    }
}

/// Find the file `name` in the cached search path and invoke `callback` for
/// each match.  `name` may contain wildcards.
///
/// `RuntimeOpts::ALL` visits every match, `RuntimeOpts::DIR` looks for directories, `RuntimeOpts::ERR`
/// turns "nothing found" into an error message.  Answers OK when something
/// was found.
///
/// # Safety
/// `name` may be null; `callback` must accept `cookie`.
pub(crate) unsafe fn do_in_cached_path(
    name: *mut c_char,
    flags: RuntimeOpts,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    if p_verbose.get() > 10 && !name.is_null() {
        // SAFETY: `name` is NUL-terminated.
        unsafe { verbose_enter() };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let name = unsafe { c_str(name) };
        smsg!(0, "Searching for \"{name}\" in runtime path");
        unsafe { verbose_leave() };
    }

    let visitor = Visitor { callback, cookie };
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let mut ref_0: c_int = 0;
    // SAFETY: `ref_0` is this frame's borrow token, released below.
    let path = unsafe { runtime_search_path_get_cached(&raw mut ref_0) };
    let do_all = flags.has(RuntimeOpts::ALL);
    let ew_flags = wildcard_flags(flags) | ExpandFlags::NOBREAK;
    let mut did_one = false;

    for j in 0..path.size {
        // SAFETY: `path` holds `size` live items with NUL-terminated paths.
        let item = unsafe { *path.items.add(j) };
        if skips_entry(flags, item.after) {
            continue;
        }
        if name.is_null() {
            // SAFETY: the entry's own NUL-terminated directory.
            unsafe { visitor.invoke_one(item.path, do_all) };
            continue;
        }
        // SAFETY: both are NUL-terminated.
        let buflen = unsafe { cstr::bytes_at(item.path) }.len();
        if buflen + unsafe { cstr::bytes_at(name) }.len() + 2 >= MAXPATHL as size_t {
            continue;
        }
        // SAFETY: the length test above proves the directory and its separator
        // fit, so `tail` lands inside `buf`.
        let tail = unsafe {
            xstrlcpy(buf.as_mut_ptr(), item.path, MAXPATHL as usize);
            add_pathsep(buf.as_mut_ptr());
            buf.as_mut_ptr().add(cstr::bytes_at(buf.as_ptr()).len())
        };
        // SAFETY: as documented on `expand_name_patterns`.
        unsafe {
            expand_name_patterns(
                buf.as_mut_ptr(),
                tail,
                name,
                ew_flags,
                do_all,
                &mut did_one,
                visitor,
            )
        };
    }

    if !did_one && !name.is_null() {
        // SAFETY: `name` is the caller's NUL-terminated pattern.
        if flags.has(RuntimeOpts::ERR) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!(
                "E919: Directory not found in '{}': \"{name}\"",
                "runtime path"
            );
        } else if p_verbose.get() > 1 {
            unsafe { verbose_enter() };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            smsg!(0, "not found in runtime path: \"{name}\"");
            unsafe { verbose_leave() };
        }
    }

    // SAFETY: the token this frame took above.
    unsafe { runtime_search_path_unref(path, &raw const ref_0) };
    if did_one { OK } else { FAIL }
}

/// The directories already in the search path, by their bytes: the dedup
/// `runtime_search_path_build` walks with. Membership only.
type PathSet = IdSet<Box<[u8]>>;

/// Add `entry` to the search path unless it is already there.
///
/// The set holds a copy of the directory only to answer "already seen";
/// the item's `path` is its own allocation, which the caller frees with
/// [`runtime_search_path_free`].
///
/// # Safety
/// `entry` must be NUL-terminated and `search_path` must be live.
unsafe fn push_path(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    entry: *mut c_char,
    after: bool,
    pos_in_rtp: size_t,
) -> bool {
    // SAFETY: the caller's NUL-terminated entry.
    let key: Box<[u8]> = unsafe { CStr::from_ptr(entry) }.to_bytes().into();
    if !rtp_used.insert(key) {
        return false;
    }
    // SAFETY: as above.
    let path = unsafe { xstrdup(entry) };
    search_path.push(SearchPathItem {
        path,
        after,
        pack_inserted: false,
        has_lua: None,
        pos_in_rtp,
    });
    true
}

/// Expand one 'runtimepath' entry — which may hold wildcards — and add every
/// directory it names to the search path.
///
/// # Safety
/// As [`push_path`].
unsafe fn expand_rtp_entry(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    entry: *mut c_char,
    after: bool,
    pos_in_rtp: size_t,
) {
    // SAFETY: the caller's NUL-terminated entry.
    if rtp_used.contains(unsafe { CStr::from_ptr(entry) }.to_bytes()) {
        return;
    }
    if unsafe { *entry } == 0 {
        unsafe { push_path(search_path, rtp_used, entry, after, pos_in_rtp) };
    }

    let mut num_files: c_int = 0;
    let mut files: *mut *mut c_char = ptr::null_mut();
    let mut pat = [entry];
    // SAFETY: a one-element pattern array; the matches are ours until
    // `free_wild`.
    if unsafe {
        gen_expand_wildcards(
            1,
            pat.as_mut_ptr(),
            &raw mut num_files,
            &raw mut files,
            ExpandFlags::DIR | ExpandFlags::NOBREAK,
        )
    }
    .is_err()
    {
        return;
    }
    for i in 0..num_files as usize {
        // Reusing the position is fine: it only has to be monotonic, not
        // strictly increasing.
        unsafe { push_path(search_path, rtp_used, *files.add(i), after, pos_in_rtp) };
    }
    unsafe { free_wild(num_files, files) };
}

/// The two shapes a 'packpath' entry's `start` packages can take.
const START_PATTERNS: [&CStr; 2] = [c"/pack/*/start/*", c"/start/*"];

/// Add one 'packpath' entry's `start` packages to the search path, and queue
/// their `after/` directories for the end of the build.
///
/// # Safety
/// `pack_entry` must be NUL-terminated and `pack_entry_len` its length; the
/// three vectors must be live.
unsafe fn expand_pack_entry(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    after_path: &mut Vec<Vec<u8>>,
    pack_entry: *mut c_char,
    pack_entry_len: size_t,
    pos_in_rtp: size_t,
) {
    let mut buf = [0 as c_char; MAXPATHL as usize];
    for start_pat in START_PATTERNS {
        if pack_entry_len + start_pat.count_bytes() + 1 > buf.len() {
            continue;
        }
        // SAFETY: the length test above proves both halves fit in `buf`, and
        // `xstrlcpy` NUL-terminates within the size it is given.
        unsafe { xstrlcpy(buf.as_mut_ptr(), pack_entry, buf.len()) };
        unsafe {
            xstrlcpy(
                buf.as_mut_ptr().add(pack_entry_len),
                start_pat.as_ptr(),
                buf.len() - pack_entry_len,
            )
        };
        unsafe { expand_rtp_entry(search_path, rtp_used, buf.as_mut_ptr(), false, pos_in_rtp) };

        // The `after/` directories go in one block at the end of the
        // build, so they sort behind every non-`after` entry.
        // SAFETY: `buf` was NUL-terminated by the two copies above.
        let mut after = unsafe { cstr::bytes_at(buf.as_ptr()) }.to_vec();
        after.extend_from_slice(b"/after\0");
        after_path.push(after);
    }
}

/// Whether `buf` names an `after/` directory.
///
/// Only a component spelled exactly `after` counts.  Vim 8 treats `foo/bar_after`
/// and `Xafter` as `after` directories in some code paths but not all; this is
/// the strict reading.
///
/// # Safety
/// `buf` must hold `buflen` readable bytes plus a terminator.
pub(crate) unsafe fn path_is_after(buf: *mut c_char, buflen: size_t) -> bool {
    // SAFETY: the caller's buffer, indexed inside the length just tested.
    buflen >= 5
        && (buflen < 6 || vim_ispathsep(unsafe { *buf.add(buflen - 6) } as c_int))
        && unsafe { cstr::eq_bytes(buf.add(buflen - 5), b"after") }
}

/// Build the ordered search path from 'runtimepath' and 'packpath'.
///
/// The order is what the whole plugin ecosystem depends on:
///
/// 1. every non-`after` 'runtimepath' entry, in order, with the `start`
///    packages of any 'packpath' entry that the entry also names spliced in
///    right behind it;
/// 2. the `start` packages of every 'packpath' entry 'runtimepath' did *not*
///    name;
/// 3. every `after/` directory of those packages;
/// 4. the `after/` tail of 'runtimepath'.
///
/// `pos_in_rtp` is the byte offset of the entry in 'runtimepath' that put a
/// directory here; entries that were not spelled in 'runtimepath' share the
/// offset of the comma before its `after/` tail, which keeps the sequence
/// monotonic. [`add_pack_dir_to_rtp`] splices new entries by comparing it.
unsafe fn runtime_search_path_build() -> RuntimeSearchPath {
    let mut pack_entries: Vec<String_0> = Vec::new();
    let mut pack_used: IdMap<Box<[u8]>, c_int> = id_map();
    let mut rtp_used: PathSet = id_set();
    let mut search_path: Vec<SearchPathItem> = Vec::new();
    let mut after_path: Vec<Vec<u8>> = Vec::new();
    let mut buf = [0 as c_char; MAXPATHL as usize];

    // 'packpath' first, only to record which entries exist: they are matched
    // against 'runtimepath' below, and whatever is left over is appended.
    // Note that the recorded strings point into 'packpath' itself.
    let mut entry = p_pp.get();
    // SAFETY: `entry` walks 'packpath'; `buf` has `MAXPATHL` writable bytes.
    while unsafe { *entry } != 0 {
        let cur_entry = entry;
        // SAFETY: as above.
        let buflen = unsafe {
            copy_option_part(
                &raw mut entry,
                buf.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        let the_entry = String_0::from_raw_parts(cur_entry, buflen);
        pack_entries.push(the_entry);
        // SAFETY: `the_entry` names `buflen` bytes of 'packpath'.
        pack_used.insert(unsafe { the_entry.as_bytes() }.into(), 0);
    }

    // 'runtimepath' up to its first `after/` entry.
    let mut rtp_entry = p_rtp.get();
    // SAFETY: `rtp_entry` walks 'runtimepath'.
    while unsafe { *rtp_entry } != 0 {
        let cur_entry = rtp_entry;
        // SAFETY: as above.
        let buflen = unsafe {
            copy_option_part(
                &raw mut rtp_entry,
                buf.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        // SAFETY: `buf` holds `buflen` bytes plus a terminator.
        if unsafe { path_is_after(buf.as_mut_ptr(), buflen) } {
            // Leave it for the tail loop below, entry and all.
            rtp_entry = cur_entry;
            break;
        }
        // SAFETY: `cur_entry` points into 'runtimepath'.
        let pos_in_rtp = unsafe { cur_entry.offset_from(p_rtp.get()) } as size_t;
        // Fact: 'runtimepath' entries can contain wildcards.
        // SAFETY: the frame's own vectors, and `buf` is NUL-terminated.
        unsafe {
            expand_rtp_entry(
                &mut search_path,
                &mut rtp_used,
                buf.as_mut_ptr(),
                false,
                pos_in_rtp,
            )
        };
        // SAFETY: `buf` is NUL-terminated.
        let seen = pack_used.get_mut(unsafe { CStr::from_ptr(buf.as_ptr()) }.to_bytes());
        if let Some(seen) = seen {
            // Mark this 'packpath' entry as already covered.
            *seen += 1;
            unsafe {
                expand_pack_entry(
                    &mut search_path,
                    &mut rtp_used,
                    &mut after_path,
                    buf.as_mut_ptr(),
                    buflen,
                    pos_in_rtp,
                )
            };
        }
    }

    // What follows was not spelled in 'runtimepath'.  Keeping `pos_in_rtp`
    // monotonic means giving it the comma between the two halves.
    // SAFETY: `rtp_entry` points into 'runtimepath'.
    let mut sentinel_pos_in_rtp = unsafe { rtp_entry.offset_from(p_rtp.get()) } as size_t;
    sentinel_pos_in_rtp -= usize::from(sentinel_pos_in_rtp > 0);

    for &item in &pack_entries {
        // SAFETY: `item` names its own bytes of 'packpath'.
        if pack_used
            .get(unsafe { item.as_bytes() })
            .copied()
            .unwrap_or(0)
            == 0
        {
            unsafe {
                expand_pack_entry(
                    &mut search_path,
                    &mut rtp_used,
                    &mut after_path,
                    item.data(),
                    item.len(),
                    sentinel_pos_in_rtp,
                )
            };
        }
    }

    // The packages' `after/` directories.
    for dir in &mut after_path {
        // SAFETY: each entry is this frame's own NUL-terminated buffer.
        unsafe {
            expand_rtp_entry(
                &mut search_path,
                &mut rtp_used,
                dir.as_mut_ptr().cast(),
                true,
                sentinel_pos_in_rtp,
            )
        };
    }

    // The `after/` tail of 'runtimepath'.
    // SAFETY: `rtp_entry` walks the rest of 'runtimepath'.
    while unsafe { *rtp_entry } != 0 {
        let cur_entry = rtp_entry;
        // SAFETY: as above.
        let buflen = unsafe {
            copy_option_part(
                &raw mut rtp_entry,
                buf.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        let pos_in_rtp = unsafe { cur_entry.offset_from(p_rtp.get()) } as size_t;
        unsafe {
            expand_rtp_entry(
                &mut search_path,
                &mut rtp_used,
                buf.as_mut_ptr(),
                path_is_after(buf.as_mut_ptr(), buflen),
                pos_in_rtp,
            )
        };
    }

    // The strings in `pack_entries` are not owned, the `after_path` ones go
    // with the frame, and the two tables hold only copies
    // -- `search_path`'s items own theirs, which is what the caller gets.
    RuntimeSearchPath::from_vec(search_path)
}

/// `'runtimepath'`/`'packpath'` changed: the cache no longer describes them.
pub unsafe fn did_set_runtimepackpath(_args: &mut optset_T) -> Option<&CStr> {
    runtime_search_path_valid.set(false);
    None
}

/// Free a search path and every string in it.
///
/// # Safety
/// Nothing may still refer to `path`.
unsafe fn runtime_search_path_free(path: RuntimeSearchPath) {
    // SAFETY: the caller's contract; the vector is dropped with the frame.
    for item in unsafe { path.into_vec() } {
        // SAFETY: each item owns its own string.
        unsafe { xfree(item.path.cast()) };
    }
}

/// Rebuild the cached search path if it has been invalidated.
pub unsafe fn runtime_search_path_validate() {
    // The path cannot be rebuilt in an async context. A plugin will invoke
    // itself asynchronously from sync code in the same plugin, so the lua or
    // autoload module it is looking for is almost certainly in the cached path
    // already: a stale cache beats an error here.
    // SAFETY: a plain read of the Lua state's nesting flags.
    if !unsafe { nlua_is_deferred_safe() } || runtime_search_path_valid.get() {
        return;
    }
    if runtime_search_path_ref.get().is_null() {
        // SAFETY: nothing is borrowing the old path, so it can go. The UI
        // flush is upstream's guard against recursion through a UI callback.
        unsafe { msg_ext_ui_flush() };
        unsafe { runtime_search_path_free(runtime_search_path.get()) };
    }
    // SAFETY: building sources nothing; it only globs.
    runtime_search_path.set(unsafe { runtime_search_path_build() });
    runtime_search_path_valid.set(true);
    // Initially unowned.
    runtime_search_path_ref.set(ptr::null_mut());
    // SAFETY: the worker threads' snapshot follows the main one.
    unsafe { update_runtime_search_path_thread(true) };
}

/// Refresh the snapshot the worker threads read.
///
/// Without `force` this is a no-op unless the main cache is valid and the
/// snapshot is not — that is, it is the cheap "catch up if you are behind"
/// call the `:packadd` family makes.
pub unsafe fn update_runtime_search_path_thread(force: bool) {
    if !force && !(runtime_search_path_valid.get() && !runtime_search_path_valid_thread.get()) {
        return;
    }
    // SAFETY: the mutex is the only thing standing between this and the worker
    // threads' reads; nothing between lock and unlock can block on them.
    unsafe { uv_mutex_lock(search_path_mutex()) };
    unsafe { runtime_search_path_free(runtime_search_path_thread.get()) };
    runtime_search_path_thread.set(unsafe { copy_runtime_search_path(runtime_search_path.get()) });
    unsafe { uv_mutex_unlock(search_path_mutex()) };
    runtime_search_path_valid_thread.set(true);
}
