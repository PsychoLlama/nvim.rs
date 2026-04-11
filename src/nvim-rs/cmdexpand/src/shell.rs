//! User-defined and shell-command expansion, migrated from `cmdexpand.c`.
//!
//! Implements `ExpandUserDefined`, `ExpandUserList`, `ExpandUserLua`.

use libc::{c_char, c_int, c_void};

use crate::{ExpandT, SctxT};

// =============================================================================
// `fuzmatch_str_T` repr(C) mirror (same layout as in expand.rs)
// =============================================================================

/// Matches `fuzmatch_str_T` from `fuzzy.h`.
#[repr(C)]
struct FuzmatchStr {
    idx: c_int,
    _pad: c_int,
    str_: *mut c_char,
    score: c_int,
    _pad2: c_int,
}

// Re-use `RegMatch` from expand.rs to avoid duplicate type definitions.
pub use crate::expand::RegMatch;

// =============================================================================
// External C functions
// =============================================================================

type ListHandle = *mut c_void;

// =============================================================================
// typval_T repr(C) (sizeof=16, verified by static assertion in testing_shim.c)
// =============================================================================

/// Mirror of `typval_T` from `eval/typval_defs.h` (sizeof=16).
///
/// Layout: `v_type`(i32)@0, `v_lock`(i32)@4, `vval`(union, 8 bytes)@8.
/// `TV_INITIAL_VALUE` initializes to `{v_type: VAR_UNKNOWN(0), v_lock: 0, vval: 0}`.
#[repr(C)]
struct TypvalT {
    v_type: c_int,
    v_lock: c_int,
    vval: [u8; 8],
}

impl TypvalT {
    /// Create a zero-initialized typval (equivalent to `TV_INITIAL_VALUE`).
    const fn initial() -> Self {
        Self {
            v_type: 0,
            v_lock: 0,
            vval: [0u8; 8],
        }
    }

    /// Get `vval.v_list` (pointer at offset 8, reinterpreted as *mut `c_void`).
    const unsafe fn get_list(&self) -> ListHandle {
        let mut ptr = std::ptr::null_mut::<c_void>();
        std::ptr::copy_nonoverlapping(
            self.vval.as_ptr(),
            std::ptr::addr_of_mut!(ptr).cast::<u8>(),
            8,
        );
        ptr
    }

    /// Create a `VAR_STRING` typval (`v_type=2`) with the given pointer.
    ///
    /// # Safety
    ///
    /// `s` must remain valid for the lifetime of this `TypvalT`.
    #[allow(clippy::missing_const_for_fn)]
    unsafe fn from_string_ptr(s: *mut c_char) -> Self {
        let mut tv = Self::initial();
        tv.v_type = 2; // VAR_STRING
        std::ptr::copy_nonoverlapping(
            std::ptr::addr_of!(s).cast::<u8>(),
            tv.vval.as_mut_ptr(),
            std::mem::size_of::<*mut c_char>(),
        );
        tv
    }

    /// Create a `VAR_NUMBER` typval (`v_type=1`) with the given `i64` value.
    #[allow(clippy::missing_const_for_fn)]
    fn from_number(n: i64) -> Self {
        let mut tv = Self::initial();
        tv.v_type = 1; // VAR_NUMBER
        tv.vval.copy_from_slice(&n.to_ne_bytes());
        tv
    }
}

/// `VAR_LIST` type constant.
const VAR_LIST: c_int = 4;

extern "C" {
    /// `nlua_call_user_expand_func(xp, ret_tv)` — direct call.
    fn nlua_call_user_expand_func(xp: *mut ExpandT, ret_tv: *mut TypvalT);

    /// `tv_clear(tv)` — clear a `typval_T` (frees contained resources).
    fn tv_clear(tv: *mut TypvalT);

    /// `nvim_tv_list_ref(list)` — increment list refcount (in `eval_shim.c`).
    fn nvim_tv_list_ref(list: ListHandle);

    /// `tv_list_len(l)` — number of items in list (in `eval_shim.c`).
    fn nvim_tv_list_len(l: ListHandle) -> c_int;

    /// `tv_list_find(l, idx)` — find list item by index (NULL if out of range).
    fn nvim_tv_list_find(l: ListHandle, idx: c_int) -> *mut c_void;

    /// Get `v_type` of `listitem_T->li_tv`.
    fn nvim_tv_list_item_type(li: *mut c_void) -> c_int;

    /// Get string value of `listitem_T->li_tv` via `tv_get_string_chk` (in `quickfix_shim.c`).
    fn nvim_tv_list_item_string(li: *mut c_void) -> *mut c_char;

    /// `tv_list_unref(l)` — decrement refcount and free if zero.
    fn nvim_tv_list_unref(l: ListHandle);

    fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> bool;
    fn fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;
    fn fuzzymatches_to_strmatches(
        fuzmatch: *mut FuzmatchStr,
        matches: *mut *mut *mut c_char,
        count: c_int,
        funcsort: bool,
    );
    fn xfree(ptr: *mut c_void);
    fn xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn xstrnsave(s: *const c_char, n: usize) -> *mut c_char;

    /// Save `current_sctx` into `*saved` and set it from `xp->xp_script_ctx`.
    fn nvim_cmdexpand_sctx_save_and_set(xp: *const ExpandT, saved: *mut SctxT);
    /// Restore `current_sctx` from `*saved`.
    fn nvim_cmdexpand_sctx_restore(saved: *const SctxT);
    /// Null-terminate `cmdbuff[cmdlen]`, returning the displaced byte (0 if no cmdbuff).
    fn nvim_cmdexpand_cmdbuff_nullterm() -> c_char;
    /// Restore `cmdbuff[cmdlen]` to the previously saved byte.
    fn nvim_cmdexpand_cmdbuff_restore(keep: c_char);
    /// Call `call_func_retlist(arg, nargs, args)` — returns `list_T *`.
    fn nvim_cmdexpand_call_func_retlist(
        arg: *const c_char,
        nargs: c_int,
        args: *mut TypvalT,
    ) -> *mut c_void;
    /// Call `call_func_retstr(arg, nargs, args)` — returns `char *`.
    fn nvim_cmdexpand_call_func_retstr(
        arg: *const c_char,
        nargs: c_int,
        args: *mut TypvalT,
    ) -> *mut c_char;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;
const FUZZY_SCORE_NONE: c_int = c_int::MIN;
const NUL: u8 = 0;

// =============================================================================
// rs_call_user_expand_retlist / rs_call_user_expand_retstr
// (Rust port of static call_user_expand_func from cmdexpand.c)
// =============================================================================

/// Shared body of `rs_call_user_expand_retlist` and `rs_call_user_expand_retstr`.
///
/// Mirrors the deleted `call_user_expand_func` from `cmdexpand.c`.
///
/// # Safety
///
/// `xp` must be a valid, fully initialized `expand_T *`.
unsafe fn call_user_expand_core<T>(
    xp: *mut ExpandT,
    dispatch: unsafe fn(*const c_char, c_int, *mut TypvalT) -> T,
    null_val: T,
) -> T {
    // Guard: xp_arg, xp_arg[0], xp_line must all be valid/non-empty.
    if (*xp).xp_arg.is_null() || *(*xp).xp_arg == 0 || (*xp).xp_line.is_null() {
        return null_val;
    }

    let keep = nvim_cmdexpand_cmdbuff_nullterm();

    let pat = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
    let mut args = [
        TypvalT::from_string_ptr(pat), // args[0]: pattern (VAR_STRING)
        TypvalT::from_string_ptr((*xp).xp_line), // args[1]: line    (VAR_STRING)
        TypvalT::from_number(i64::from((*xp).xp_col)), // args[2]: col     (VAR_NUMBER)
        TypvalT::initial(),            // args[3]: sentinel (VAR_UNKNOWN=0)
    ];

    // Save current_sctx and set it from xp->xp_script_ctx.
    // SAFETY: we initialise with zeros; the C function fills all fields.
    let mut saved_sctx = std::mem::zeroed::<SctxT>();
    nvim_cmdexpand_sctx_save_and_set(xp, &raw mut saved_sctx);

    let ret = dispatch((*xp).xp_arg, 3, args.as_mut_ptr());

    nvim_cmdexpand_sctx_restore(&raw const saved_sctx);
    nvim_cmdexpand_cmdbuff_restore(keep);
    xfree(pat.cast());
    ret
}

/// Call a user-defined `VimL` expand function, returning the result as a list.
///
/// Exported as `rs_call_user_expand_retlist` (called from C wrappers in
/// `cmdexpand.c` which forward `nvim_cmdexpand_call_user_expand_retlist`).
///
/// # Safety
///
/// `xp` must be a valid `expand_T *`.
#[unsafe(export_name = "rs_call_user_expand_retlist")]
pub unsafe extern "C" fn rs_call_user_expand_retlist(xp: *mut ExpandT) -> ListHandle {
    call_user_expand_core(
        xp,
        |arg, n, args| nvim_cmdexpand_call_func_retlist(arg, n, args),
        std::ptr::null_mut(),
    )
}

/// Call a user-defined `VimL` expand function, returning the result as a string.
///
/// Exported as `rs_call_user_expand_retstr` (called from C wrappers in
/// `cmdexpand.c` which forward `nvim_cmdexpand_call_user_expand_retstr`).
///
/// # Safety
///
/// `xp` must be a valid `expand_T *`.
#[unsafe(export_name = "rs_call_user_expand_retstr")]
pub unsafe extern "C" fn rs_call_user_expand_retstr(xp: *mut ExpandT) -> *mut c_char {
    call_user_expand_core(
        xp,
        |arg, n, args| nvim_cmdexpand_call_func_retstr(arg, n, args),
        std::ptr::null_mut(),
    )
}

// =============================================================================
// list_to_string_matches
// =============================================================================

/// Convert a `list_T *` to a `char **` array, taking ownership of the list.
///
/// Iterates the list, copying each string item with `xstrdup`, then unrefs the list.
///
/// # Safety
///
/// `list` must be a valid `list_T *`. `matches` and `num_matches` must be non-null.
pub unsafe fn rs_list_to_string_matches(
    list: ListHandle,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) {
    let len = nvim_tv_list_len(list);
    let mut result: Vec<*mut c_char> = Vec::with_capacity(len as usize);

    for i in 0..len {
        let li = nvim_tv_list_find(list, i);
        if li.is_null() {
            continue;
        }
        // nvim_tv_list_item_type checks VAR_STRING; skip non-string items
        if nvim_tv_list_item_type(li) != crate::VAR_STRING {
            continue;
        }
        // nvim_tv_list_item_string returns NULL for non-string items (tv_get_string_chk)
        let s = nvim_tv_list_item_string(li);
        if s.is_null() {
            continue;
        }
        result.push(xmemdupz(s, libc::strlen(s)));
    }

    nvim_tv_list_unref(list);

    let count = result.len();
    let boxed = result.into_boxed_slice();
    *matches = Box::into_raw(boxed).cast::<*mut c_char>();
    *num_matches = count as c_int;
}

// =============================================================================
// ExpandUserDefined
// =============================================================================

/// Expand names with a user-defined function (`EXPAND_USER_DEFINED`).
///
/// Mirrors `ExpandUserDefined` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_defined(
    pat: *const c_char,
    xp: *mut ExpandT,
    regmatch: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    let pat_str = std::ffi::CStr::from_ptr(pat).to_bytes();
    let pat_rust = std::str::from_utf8_unchecked(pat_str);
    let fuzzy = crate::cmdline_fuzzy_complete(pat_rust);

    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retstr = rs_call_user_expand_retstr(xp);
    if retstr.is_null() {
        return FAIL;
    }

    let mut str_matches: Vec<*mut c_char> = Vec::new();
    let mut fuz_matches: Vec<FuzmatchStr> = Vec::new();

    let mut s = retstr;
    loop {
        if *s == NUL as c_char {
            break;
        }
        // Find end of this item (newline or NUL)
        let e = vim_strchr(s, c_int::from(b'\n'));
        let e: *const c_char = if e.is_null() {
            s.add(libc::strlen(s))
        } else {
            e
        };

        let keep = *e;
        // Temporarily NUL-terminate this item
        let e_mut = e.cast_mut();
        *e_mut = 0;

        let mut score: c_int = 0;
        let xp_pattern = (*xp).xp_pattern;
        let is_match = if !xp_pattern.is_null() && *xp_pattern != 0 {
            if fuzzy {
                score = fuzzy_match_str(s, pat);
                score != FUZZY_SCORE_NONE
            } else {
                vim_regexec(regmatch, s, 0)
            }
        } else {
            true
        };

        // Restore the original character
        *e_mut = keep;

        if is_match {
            let item_len = (e as usize) - (s as usize);
            let owned = xmemdupz(s, item_len);
            if fuzzy {
                fuz_matches.push(FuzmatchStr {
                    idx: fuz_matches.len() as c_int,
                    _pad: 0,
                    str_: owned,
                    score,
                    _pad2: 0,
                });
            } else {
                str_matches.push(owned);
            }
        }

        if keep == 0 {
            break;
        }
        s = e.add(1).cast_mut();
    }
    xfree(retstr.cast());

    let count = if fuzzy {
        fuz_matches.len()
    } else {
        str_matches.len()
    };

    if count == 0 {
        return OK;
    }

    if fuzzy {
        fuzzymatches_to_strmatches(
            fuz_matches.as_mut_ptr(),
            matches,
            fuz_matches.len() as c_int,
            false,
        );
        *num_matches = fuz_matches.len() as c_int;
        std::mem::forget(fuz_matches);
    } else {
        let boxed = str_matches.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<*mut c_char>();
        *matches = ptr;
        *num_matches = len as c_int;
    }

    OK
}

// =============================================================================
// ExpandUserList
// =============================================================================

/// Expand names with a list returned by a user-defined function.
///
/// Mirrors `ExpandUserList` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_list(
    xp: *mut ExpandT,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retlist = rs_call_user_expand_retlist(xp);
    if retlist.is_null() {
        return FAIL;
    }

    rs_list_to_string_matches(retlist, matches, num_matches);
    OK
}

// =============================================================================
// ExpandUserLua
// =============================================================================

/// Expand from a Lua user completion function.
///
/// Mirrors `ExpandUserLua` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_lua(
    xp: *mut ExpandT,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retlist = rs_nlua_call_user_expand_retlist(xp);
    if retlist.is_null() {
        return FAIL;
    }

    rs_list_to_string_matches(retlist, matches, num_matches);
    OK
}

/// Call `nlua_call_user_expand_func` and return the resulting `list_T *`.
///
/// Returns NULL if the result is not a list. The returned list has its
/// refcount incremented; caller must call `nvim_tv_list_unref` when done.
///
/// Mirrors `nvim_cmdexpand_nlua_call_user_expand_retlist` from `cmdexpand.c`.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` pointer.
unsafe fn rs_nlua_call_user_expand_retlist(xp: *mut ExpandT) -> ListHandle {
    let mut rettv = TypvalT::initial();
    nlua_call_user_expand_func(xp, &raw mut rettv);
    if rettv.v_type != VAR_LIST {
        tv_clear(&raw mut rettv);
        return std::ptr::null_mut();
    }
    let li = rettv.get_list();
    nvim_tv_list_ref(li);
    tv_clear(&raw mut rettv);
    li
}

// =============================================================================
// expand_shellcmd / expand_shellcmd_onedir -- migrated from cmdexpand.c
// =============================================================================

use crate::context::ew_flags::{EW_DIR, EW_EXEC, EW_FILE, EW_SHELLCMD};

/// Growing array matching C `garray_T` layout (24 bytes on 64-bit).
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

/// Hashtable item matching C `hashitem_T` layout (16 bytes on 64-bit).
#[repr(C)]
struct HashitemT {
    hi_hash: usize,
    hi_key: *mut c_char,
}

/// Hashtable matching C `hashtab_T` layout (296 bytes, 16 inline items).
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct HashtabT {
    ht_mask: usize,
    ht_used: usize,
    ht_filled: usize,
    ht_changed: c_int,
    ht_locked: c_int,
    ht_array: *mut HashitemT,
    ht_smallarray: [HashitemT; 16],
}

extern "C" {
    static hash_removed: c_char;

    fn expand_wildcards(
        num: c_int,
        pat: *mut *mut c_char,
        num_files: *mut c_int,
        files: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn vim_getenv(name: *const c_char) -> *mut c_char;
    fn path_is_absolute(fname: *const c_char) -> bool;
    fn vim_ispathsep(c: c_int) -> c_int;
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;

    fn hash_hash(key: *const c_char) -> usize;
    fn hash_lookup(
        ht: *const HashtabT,
        key: *const c_char,
        len: usize,
        hash: usize,
    ) -> *mut HashitemT;
    fn hash_add_item(ht: *mut HashtabT, hi: *mut HashitemT, key: *mut c_char, hash: usize);
    fn hash_init(ht: *mut HashtabT);
    fn hash_clear(ht: *mut HashtabT);

    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GArray, n: c_int);

    fn xmalloc(size: usize) -> *mut c_char;
    fn xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);
}

/// Maximum path length (`MAXPATHL`).
const MAXPATHL: usize = 4096;
/// Path separator string on Linux (`PATHSEPSTR`).
const PATHSEPSTR: &[u8] = b"/";
/// Environment path separator on Linux (`ENV_SEPCHAR` = `:`).
const ENV_SEPCHAR: u8 = b':';

/// Returns true if `hi` is an empty (unused or removed) hashtable slot.
///
/// Mirrors C `HASHITEM_EMPTY(hi)`.
#[inline]
unsafe fn hashitem_empty(hi: *const HashitemT) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq((*hi).hi_key, std::ptr::addr_of!(hash_removed).cast_mut())
}

/// Expands shell command wildcard matches in one `$PATH` directory.
///
/// Mirrors C `expand_shellcmd_onedir`.
///
/// # Safety
///
/// All pointers must be valid. `ht` and `gap` must be initialized.
unsafe fn expand_shellcmd_onedir(
    mut pathed_pattern: *mut c_char,
    pathlen: usize,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    flags: c_int,
    ht: *mut HashtabT,
    gap: *mut GArray,
) {
    const OK: c_int = 1;
    if expand_wildcards(1, &raw mut pathed_pattern, num_matches, matches, flags) != OK {
        return;
    }

    ga_grow(gap, *num_matches);

    for i in 0..(*num_matches as usize) {
        let name: *mut c_char = *(*matches).add(i);
        let namelen = libc::strlen(name);

        if namelen > pathlen {
            let basename = name.add(pathlen);
            let baselen = namelen - pathlen;
            let hash = hash_hash(basename);
            let hi = hash_lookup(ht, basename, baselen, hash);
            if hashitem_empty(hi) {
                // Strip the path prefix by moving name bytes left.
                std::ptr::copy(basename, name, baselen + 1); // +1 for NUL
                let slot = (*gap)
                    .ga_data
                    .cast::<*mut c_char>()
                    .add((*gap).ga_len as usize);
                *slot = name;
                (*gap).ga_len += 1;
                hash_add_item(ht, hi, name, hash);
                // name is now owned by gap; skip the xfree below
                continue;
            }
        }
        xfree(name.cast());
    }
    xfree((*matches).cast());
}

/// Completes shell command names matching `filepat` by searching `$PATH`.
///
/// Direct replacement for C `expand_shellcmd` (exported by name).
///
/// # Safety
///
/// `filepat`, `matches`, and `num_matches` must be valid non-null pointers.
#[allow(clippy::too_many_lines)]
#[export_name = "expand_shellcmd"]
pub unsafe extern "C" fn rs_expand_shellcmd(
    filepat: *mut c_char,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    flagsarg: c_int,
) {
    let buf = xmalloc(MAXPATHL);
    let mut flags = flagsarg;

    // Strip backslash-escaped spaces ("\ " -> " ").
    let patlen_orig = libc::strlen(filepat);
    let pat = xmemdupz(filepat, patlen_orig);
    {
        let pat_end = pat.add(patlen_orig);
        let mut s = pat;
        while *s != 0 {
            if *s != b'\\' as c_char {
                s = s.add(1);
                continue;
            }
            let p = s.add(1);
            if *p == b' ' as c_char {
                let remaining = pat_end.offset_from(p) as usize + 1; // +1 for NUL
                std::ptr::copy(p, s, remaining);
                // Do NOT advance s.
            } else {
                s = s.add(1);
            }
        }
    }
    let patlen = libc::strlen(pat);

    flags |= EW_FILE | EW_EXEC | EW_SHELLCMD;

    // Determine search path.
    let mut mustfree = false;
    let path: *mut c_char = if *pat == b'.' as c_char
        && (vim_ispathsep(c_int::from(*pat.add(1) as u8)) != 0
            || (*pat.add(1) == b'.' as c_char
                && vim_ispathsep(c_int::from(*pat.add(2) as u8)) != 0))
    {
        // Pattern starts with "./" or "../": search only current dir.
        flags |= EW_DIR;
        c".".as_ptr().cast_mut()
    } else if !path_is_absolute(pat) {
        let p = vim_getenv(c"PATH".as_ptr());
        if p.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            mustfree = true;
            p
        }
    } else {
        c"".as_ptr().cast_mut()
    };

    let mut ga = core::mem::MaybeUninit::<GArray>::zeroed().assume_init();
    ga_init(&raw mut ga, std::mem::size_of::<*mut c_char>() as c_int, 10);

    let mut found_ht = core::mem::MaybeUninit::<HashtabT>::zeroed().assume_init();
    hash_init(&raw mut found_ht);

    let mut did_curdir = false;
    let mut s = path;

    loop {
        if *s == 0 {
            // No more PATH entries; try current directory if not already done.
            if did_curdir {
                break;
            }
            flags |= EW_DIR;

            // Current-directory pass: pathlen == 0, no separator.
            if patlen < MAXPATHL {
                xmemcpyz(buf, pat, patlen);
                expand_shellcmd_onedir(
                    buf,
                    0,
                    matches,
                    num_matches,
                    flags,
                    &raw mut found_ht,
                    &raw mut ga,
                );
            }
            break;
        }

        // Find next ENV_SEPCHAR in the PATH string.
        let mut e = s;
        while *e != 0 && *e != ENV_SEPCHAR as c_char {
            e = e.add(1);
        }
        let dirlen = e.offset_from(s) as usize;

        if dirlen == 1 && *s == b'.' as c_char {
            did_curdir = true;
            flags |= EW_DIR;
        } else {
            flags &= !EW_DIR;
        }

        let seplen = if after_pathsep(s, s.add(dirlen)) != 0 {
            0
        } else {
            PATHSEPSTR.len()
        };

        if dirlen + seplen + patlen < MAXPATHL {
            if dirlen > 0 {
                xmemcpyz(buf, s, dirlen);
                if seplen > 0 {
                    xmemcpyz(buf.add(dirlen), PATHSEPSTR.as_ptr().cast(), seplen);
                }
            }
            xmemcpyz(buf.add(dirlen + seplen), pat, patlen);

            expand_shellcmd_onedir(
                buf,
                dirlen + seplen,
                matches,
                num_matches,
                flags,
                &raw mut found_ht,
                &raw mut ga,
            );
        }

        s = if *e != 0 { e.add(1) } else { e };
    }

    *matches = ga.ga_data.cast::<*mut c_char>();
    *num_matches = ga.ga_len;

    xfree(buf.cast());
    xfree(pat.cast());
    if mustfree {
        xfree(path.cast());
    }
    hash_clear(&raw mut found_ht);
}
