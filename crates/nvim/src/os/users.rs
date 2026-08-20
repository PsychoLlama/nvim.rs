//! Operating-system user information: the `~user` completion list, the name
//! of the user running this process, and a user's home directory.
//!
//! # Boundary
//!
//! The password database (`getpwent`/`getpwnam`/`getpwuid`) is libc's, and
//! every entry it returns points into a static buffer that the next call
//! invalidates — so each accessor here copies what it wants before doing
//! anything else. None of those calls is reentrant or thread-safe; the
//! editor is single-threaded, which is the assumption upstream made too.
//!
//! The `os_get_*` entry points keep their C signatures: the unit suite
//! calls them through FFI (see `test/unit/os/users_spec.lua`).

use crate::garray::{ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::memory::{xstrdup, xstrlcpy};
use crate::os::env::os_getenv_noalloc;
use crate::types::{FAIL, OK, expand_T, garray_T, size_t, uv_uid_t};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use std::ffi::CString;

/// How well a name matches the known user names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserMatch {
    /// Matches no user name at all.
    None,
    /// Is a proper prefix of some user name.
    Prefix,
    /// Is a user name.
    Exact,
}

/// A `passwd` name field copied out of libc's static entry. NULL and empty
/// names are dropped, as the C's `add_user` did.
///
/// # Safety
///
/// `s` must be NULL or a NUL-terminated string that stays valid for the
/// call.
unsafe fn owned_name(s: *const c_char) -> Option<CString> {
    if s.is_null() {
        return None;
    }
    let name = CStr::from_ptr(s);
    (!name.is_empty()).then(|| name.to_owned())
}

/// Every user name the system knows.
///
/// `getpwent` only enumerates *local* accounts, so `$USER` gets a second
/// look through `getpwnam`: on NIS/LDAP hosts it names a real user the
/// enumeration never mentions.
fn all_usernames() -> Vec<CString> {
    let mut names = Vec::new();
    // SAFETY: the pwd walk is libc's own iterator, and `$USER` lands in a
    // shared static buffer (`os_getenv_noalloc` writes into `NameBuff`).
    // Every string is copied before the next call can invalidate it.
    unsafe {
        libc::setpwent();
        while let Some(pw) = libc::getpwent().as_ref() {
            names.extend(owned_name(pw.pw_name));
        }
        libc::endpwent();

        if let Some(user_env) = owned_name(os_getenv_noalloc(c"USER".as_ptr()))
            && !names.contains(&user_env)
            && let Some(pw) = libc::getpwnam(user_env.as_ptr()).as_ref()
        {
            names.extend(owned_name(pw.pw_name));
        }
    }
    names
}

/// The `~user` completion list. Filled on first use and never cleared:
/// [`get_users`] hands out pointers borrowed from it, exactly as the C
/// handed out pointers into a `static garray_T`.
static COMPLETION_USERS: GlobalCell<Vec<CString>> = GlobalCell::new(Vec::new());
static COMPLETION_USERS_READY: GlobalCell<bool> = GlobalCell::new(false);

fn init_users() {
    if COMPLETION_USERS_READY.get() {
        return;
    }
    COMPLETION_USERS_READY.set(true);
    COMPLETION_USERS.set(all_usernames());
}

/// Given to `expand_generic()` to obtain user names. NULL past the end.
pub fn get_users(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    init_users();
    COMPLETION_USERS.with(|users| {
        usize::try_from(idx)
            .ok()
            .and_then(|i| users.get(i))
            .map_or(ptr::null_mut(), |name| name.as_ptr().cast_mut())
    })
}

/// Whether `name` matches a user name, exactly or as a prefix.
pub fn match_user(name: &CStr) -> UserMatch {
    init_users();
    COMPLETION_USERS.with(|users| best_match(users, name.to_bytes()))
}

/// The strongest match for `name` among `users`. An exact match wins
/// immediately, so a later user name that `name` only prefixes never
/// downgrades it.
fn best_match(users: &[CString], name: &[u8]) -> UserMatch {
    let mut result = UserMatch::None;
    for user in users {
        if user.as_bytes() == name {
            return UserMatch::Exact;
        }
        if user.as_bytes().starts_with(name) {
            result = UserMatch::Prefix;
        }
    }
    result
}

/// Initialize `users` and fill it with every user name on the system.
/// FAIL only when `users` is NULL.
///
/// The names are `xstrdup`ed rather than moved out of the `CString`s: the
/// garray is a C array of `char *` that callers release with
/// `ga_clear_strings`, and the unit suite's allocator seam expects to see
/// those allocations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_usernames(users: *mut garray_T) -> c_int {
    if users.is_null() {
        return FAIL;
    }
    ga_init(users, size_of::<*mut c_char>() as c_int, 20);
    for name in all_usernames() {
        ga_grow(users, 1);
        let items = (*users).ga_data as *mut *mut c_char;
        *items.add((*users).ga_len as usize) = xstrdup(name.as_ptr());
        (*users).ga_len += 1;
    }
    OK
}

/// Write the name of the user running this process into `s` (`len` bytes,
/// always NUL-terminated). OK when a name was found.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_username(s: *mut c_char, len: size_t) -> c_int {
    os_get_uname(libc::getuid() as uv_uid_t, s, len)
}

/// Write the name of the user owning `uid` into `s` (`len` bytes, always
/// NUL-terminated). When the database has no name for it, the decimal uid
/// is written instead and the result is FAIL — a number is not a name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_uname(uid: uv_uid_t, s: *mut c_char, len: size_t) -> c_int {
    let name = libc::getpwuid(uid as libc::uid_t)
        .as_ref()
        .and_then(|pw| owned_name(pw.pw_name));
    if let Some(name) = name {
        xstrlcpy(s, name.as_ptr(), len);
        return OK;
    }
    let digits = CString::new((uid as c_int).to_string()).expect("decimal digits hold no NUL");
    xstrlcpy(s, digits.as_ptr(), len);
    FAIL
}

/// The home directory of user `name`, or NULL when there is no such user.
/// The caller owns the result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_userdir(name: *const c_char) -> *mut c_char {
    if name.is_null() || *name == 0 {
        return ptr::null_mut();
    }
    let Some(pw) = libc::getpwnam(name).as_ref() else {
        return ptr::null_mut();
    };
    if pw.pw_dir.is_null() {
        // The C handed a NULL `pw_dir` straight to `xstrdup`; no password
        // database produces one, but NULL is the honest answer.
        return ptr::null_mut();
    }
    xstrdup(pw.pw_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users(names: &[&str]) -> Vec<CString> {
        names.iter().map(|n| CString::new(*n).unwrap()).collect()
    }

    #[test]
    fn exact_match_beats_a_later_prefix_match() {
        let users = users(&["rootless", "root", "rooted"]);
        assert_eq!(best_match(&users, b"root"), UserMatch::Exact);
    }

    #[test]
    fn prefix_match_needs_no_exact_one() {
        let users = users(&["daemon", "root"]);
        assert_eq!(best_match(&users, b"roo"), UserMatch::Prefix);
        assert_eq!(best_match(&users, b"nobody"), UserMatch::None);
    }

    #[test]
    fn every_name_prefixes_the_empty_string() {
        assert_eq!(best_match(&users(&["root"]), b""), UserMatch::Prefix);
        // ... but with nobody to match, there is no match at all.
        assert_eq!(best_match(&[], b""), UserMatch::None);
    }
}
