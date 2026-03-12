//! User account operations
//!
//! Provides functions for querying system user information.
//! Replaces `src/nvim/os/users.c`.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Mutex;

use crate::{FAIL, OK};

// =============================================================================
// C function imports
// =============================================================================

extern "C" {
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_char);

    #[link_name = "snprintf"]
    fn c_snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;

    #[link_name = "xstrlcpy"]
    fn c_xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;

    #[link_name = "os_getenv_noalloc"]
    fn c_os_getenv_noalloc(name: *const c_char) -> *mut c_char;

    #[link_name = "ga_init"]
    fn c_ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);

    #[link_name = "ga_grow"]
    fn c_ga_grow(gap: *mut GArray, n: c_int);
}

// =============================================================================
// garray_T layout (matches C struct exactly)
// =============================================================================

#[repr(C)]
#[allow(clippy::struct_field_names)]
pub(crate) struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArray {
    fn items(&self) -> *mut *mut c_char {
        self.ga_data.cast()
    }
}

/// Append a `*mut c_char` to a GArray of char pointers.
/// Equivalent to `GA_APPEND(char *, gap, val)`.
unsafe fn ga_append_str(ga: *mut GArray, val: *mut c_char) {
    c_ga_grow(ga, 1);
    // ga_len is always non-negative after ga_init; cast to usize is safe.
    #[allow(clippy::cast_sign_loss)]
    let idx = (*ga).ga_len as usize;
    *(*ga).items().add(idx) = val;
    (*ga).ga_len += 1;
}

// =============================================================================
// User cache
// =============================================================================

/// Newtype wrapping raw C string pointers for the user cache.
///
/// SAFETY: The pointers are allocated with nvim's `xstrdup` and are valid for
/// the lifetime of the process. Access is gated by the `Mutex`, so there is no
/// concurrent aliasing.
struct UserPtr(*mut c_char);
unsafe impl Send for UserPtr {}
unsafe impl Sync for UserPtr {}

/// Cached user names (equivalent to `static garray_T ga_users` in C).
static GA_USERS: Mutex<Option<Vec<UserPtr>>> = Mutex::new(None);

// =============================================================================
// Public API
// =============================================================================

/// Initialize `users` garray and fill it with OS usernames.
///
/// Returns `OK` on success, `FAIL` on failure.
///
/// # Safety
///
/// `users` must be a valid pointer to a `garray_T`.
#[export_name = "os_get_usernames"]
pub unsafe extern "C" fn rs_os_get_usernames(users: *mut c_void) -> c_int {
    if users.is_null() {
        return FAIL;
    }
    let users: *mut GArray = users.cast();

    c_ga_init(users, std::mem::size_of::<*mut c_char>() as c_int, 20);

    #[cfg(unix)]
    {
        libc::setpwent();
        loop {
            let pw = libc::getpwent();
            if pw.is_null() {
                break;
            }
            if !(*pw).pw_name.is_null() && *(*pw).pw_name != 0 {
                let copy = nvim_xstrdup((*pw).pw_name);
                ga_append_str(users, copy);
            }
        }
        libc::endpwent();

        // Also check $USER for NIS/LDAP users not listed by getpwent()
        let user_env = c_os_getenv_noalloc(c"USER".as_ptr());
        if !user_env.is_null() && *user_env != 0 {
            // Check if already in the list
            // ga_len is always non-negative; cast to usize is safe.
            #[allow(clippy::cast_sign_loss)]
            let len = (*users).ga_len as usize;
            let items = (*users).items();
            let mut found = false;
            for i in 0..len {
                let local = *items.add(i);
                if libc::strcmp(local, user_env) == 0 {
                    found = true;
                    break;
                }
            }
            if !found {
                let pw = libc::getpwnam(user_env);
                if !pw.is_null() && !(*pw).pw_name.is_null() && *(*pw).pw_name != 0 {
                    let copy = nvim_xstrdup((*pw).pw_name);
                    ga_append_str(users, copy);
                }
            }
        }
    }

    OK
}

/// Gets the username that owns the current Nvim process.
///
/// # Safety
///
/// `s` must point to a valid buffer of at least `len` bytes.
#[export_name = "os_get_username"]
pub unsafe extern "C" fn rs_os_get_username(s: *mut c_char, len: usize) -> c_int {
    #[cfg(unix)]
    {
        rs_os_get_uname(libc::getuid(), s, len)
    }
    #[cfg(not(unix))]
    {
        rs_os_get_uname(0, s, len)
    }
}

/// Gets the username associated with `uid`.
///
/// Returns `OK` if a username was found, `FAIL` otherwise.
/// On failure, writes the numeric uid as a string.
///
/// # Safety
///
/// `s` must point to a valid buffer of at least `len` bytes.
#[export_name = "os_get_uname"]
pub unsafe extern "C" fn rs_os_get_uname(uid: u32, s: *mut c_char, len: usize) -> c_int {
    #[cfg(unix)]
    {
        let pw = libc::getpwuid(uid);
        if !pw.is_null() && !(*pw).pw_name.is_null() && *(*pw).pw_name != 0 {
            c_xstrlcpy(s, (*pw).pw_name, len);
            return OK;
        }
    }
    // Fall back to writing the numeric uid
    c_snprintf(s, len, c"%d".as_ptr(), uid as c_int);
    FAIL
}

/// Gets the home directory for the given username.
///
/// Returns a C string (allocated with nvim's `xstrdup`) or NULL.
/// Caller must free the result with `xfree`.
///
/// # Safety
///
/// `name` must be a valid C string or NULL.
#[export_name = "os_get_userdir"]
pub unsafe extern "C" fn rs_os_get_userdir(name: *const c_char) -> *mut c_char {
    if name.is_null() || *name == 0 {
        return ptr::null_mut();
    }
    #[cfg(unix)]
    {
        let pw = libc::getpwnam(name);
        if !pw.is_null() && !(*pw).pw_dir.is_null() {
            return nvim_xstrdup((*pw).pw_dir);
        }
    }
    ptr::null_mut()
}

/// Free the cached user list.
///
/// Called from `free_all_mem()` at exit.
#[export_name = "free_users"]
pub extern "C" fn rs_free_users() {
    let mut guard = GA_USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(ref mut users) = *guard {
        for UserPtr(ptr) in users.drain(..) {
            unsafe { nvim_xfree(ptr) };
        }
    }
    *guard = None;
}

/// Lazy-initialize the user name cache.
fn init_users() {
    let mut guard = GA_USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.is_some() {
        return;
    }

    let mut users: Vec<UserPtr> = Vec::new();

    #[cfg(unix)]
    unsafe {
        libc::setpwent();
        loop {
            let pw = libc::getpwent();
            if pw.is_null() {
                break;
            }
            if !(*pw).pw_name.is_null() && *(*pw).pw_name != 0 {
                let copy = nvim_xstrdup((*pw).pw_name);
                users.push(UserPtr(copy));
            }
        }
        libc::endpwent();

        // Check $USER for NIS/LDAP users not listed by getpwent()
        let user_env = c_os_getenv_noalloc(c"USER".as_ptr());
        if !user_env.is_null() && *user_env != 0 {
            let already_in = users.iter().any(|p| libc::strcmp(p.0, user_env) == 0);
            if !already_in {
                let pw = libc::getpwnam(user_env);
                if !pw.is_null() && !(*pw).pw_name.is_null() && *(*pw).pw_name != 0 {
                    users.push(UserPtr(nvim_xstrdup((*pw).pw_name)));
                }
            }
        }
    }

    *guard = Some(users);
}

/// Given to `ExpandGeneric()` to obtain user names.
///
/// # Safety
///
/// `xp` is unused. Returns a pointer into the cache; valid until `free_users()`.
#[export_name = "get_users"]
pub unsafe extern "C" fn rs_get_users(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    init_users();
    // idx is always checked >= 0 before indexing.
    #[allow(clippy::cast_sign_loss)]
    let idx_usize = if idx >= 0 {
        idx as usize
    } else {
        return ptr::null_mut();
    };
    let ptr = GA_USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(|users| users.get(idx_usize).map(|u| u.0))
        .unwrap_or(ptr::null_mut());
    ptr
}

/// Check whether `name` matches a user name.
///
/// Returns:
/// - `0` if `name` does not match any user name
/// - `1` if `name` partially matches the beginning of a user name
/// - `2` if `name` fully matches a user name
///
/// # Safety
///
/// `name` must be a valid C string.
#[export_name = "match_user"]
pub unsafe extern "C" fn rs_match_user(name: *mut c_char) -> c_int {
    init_users();
    let n = libc::strlen(name);
    // Walk the list under the lock; 0 = no match, 1 = partial, 2 = full.
    // We avoid early returns so the lock guard is dropped before we return.
    GA_USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map_or(0, |users| {
            let mut partial = 0;
            for entry in users {
                if libc::strcmp(entry.0, name) == 0 {
                    return 2;
                }
                if libc::strncmp(entry.0, name, n) == 0 {
                    partial = 1;
                }
            }
            partial
        })
}
