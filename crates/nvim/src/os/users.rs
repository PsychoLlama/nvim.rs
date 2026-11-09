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
//! The name and home-directory entry points are driven from outside the
//! crate by `crates/nvim/tests/unit/users.rs`, which is why they are `pub`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cmdexpand::defs::Candidate;
use crate::cmdexpand::defs::Expand;
use crate::cstr;
use crate::event::uv::uv_uid_t;
use crate::global_cell::GlobalCell;
use crate::os::env::env_get_bounded;
use core::ffi::CStr;
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

/// What the password database says about one user, copied out of libc's
/// static entry. NULL fields, and an empty name, are `None`, as the C's
/// `add_user` dropped them.
#[derive(Debug)]
struct Passwd {
    name: Option<CString>,
    dir: Option<Vec<u8>>,
}

/// Copy what a `getpw*` call answered.
///
/// # Safety
///
/// `entry` is null, or the entry a `getpw*` call just answered, read before
/// any other such call can overwrite it.
unsafe fn copy_entry(entry: *const libc::passwd) -> Option<Passwd> {
    // SAFETY: the caller's contract; a non-null entry's `pw_name` and
    // `pw_dir` are null or NUL-terminated, and both are copied here.
    unsafe {
        entry.as_ref().map(|pw| Passwd {
            name: (!pw.pw_name.is_null())
                .then(|| CStr::from_ptr(pw.pw_name))
                .filter(|name| !name.is_empty())
                .map(CStr::to_owned),
            dir: (!pw.pw_dir.is_null()).then(|| CStr::from_ptr(pw.pw_dir).to_bytes().to_vec()),
        })
    }
}

/// The password database's entry for `uid`.
fn entry_by_uid(uid: uv_uid_t) -> Option<Passwd> {
    // SAFETY: `getpwuid` answers null or its static entry, copied at once.
    unsafe { copy_entry(libc::getpwuid(uid)) }
}

/// The password database's entry for the user called `name`.
fn entry_by_name(name: &CStr) -> Option<Passwd> {
    // SAFETY: a NUL-terminated name; `getpwnam` answers null or its static
    // entry, copied at once.
    unsafe { copy_entry(libc::getpwnam(name.as_ptr())) }
}

/// Every user name the system knows.
///
/// `getpwent` only enumerates *local* accounts, so `$USER` gets a second
/// look through `getpwnam`: on NIS/LDAP hosts it names a real user the
/// enumeration never mentions.
fn all_usernames() -> Vec<CString> {
    let mut names = Vec::new();
    // SAFETY: rewinds libc's own iterator.
    unsafe { libc::setpwent() };
    // SAFETY: `getpwent` answers null at the end, or its static entry,
    // copied before the next step.
    while let Some(entry) = unsafe { copy_entry(libc::getpwent()) } {
        names.extend(entry.name);
    }
    // SAFETY: closes the iterator `setpwent` opened.
    unsafe { libc::endpwent() };

    let user_env = cstr::owned(&env_get_bounded(c"USER"));
    if !user_env.is_empty()
        && !names.contains(&user_env)
        && let Some(entry) = entry_by_name(&user_env)
    {
        names.extend(entry.name);
    }
    names
}

/// The `~user` completion list. Filled on first use and never cleared:
/// [`get_users`] hands out pointers borrowed from it, exactly as the C
/// handed out pointers into a `static` array.
static COMPLETION_USERS: GlobalCell<Vec<CString>> = GlobalCell::new(Vec::new());
static COMPLETION_USERS_READY: GlobalCell<bool> = GlobalCell::new(false);

fn init_users() {
    if COMPLETION_USERS_READY.get() {
        return;
    }
    COMPLETION_USERS_READY.set(true);
    COMPLETION_USERS.set(all_usernames());
}

/// The process's real user id.
pub(crate) fn os_getuid() -> libc::uid_t {
    // SAFETY: `getuid` takes nothing and always succeeds.
    unsafe { libc::getuid() }
}

/// The process's real group id.
pub(crate) fn os_getgid() -> libc::gid_t {
    // SAFETY: `getgid` takes nothing and always succeeds.
    unsafe { libc::getgid() }
}

/// Given to `expand_generic()` to obtain user names. NULL past the end.
pub fn get_users(_expand: &Expand, idx: usize) -> Option<Candidate> {
    init_users();
    COMPLETION_USERS.with(|users| users.get(idx).map(|name| Candidate::Owned(name.clone())))
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

/// Every user name on the system, in the order the password database lists
/// them (with `$USER` last when the enumeration missed it).
pub fn os_get_usernames() -> Vec<Vec<u8>> {
    all_usernames()
        .into_iter()
        .map(CString::into_bytes)
        .collect()
}

/// The name of the user running this process, cut to `room - 1` bytes as a
/// buffer of `room` would cut it: the decimal uid when the user database has
/// no name for it.
pub(crate) fn username_within(room: usize) -> Vec<u8> {
    username_named_within(room).unwrap_or_else(|uid| uid)
}

/// The name of the user owning `uid`, cut to `room - 1` bytes as a buffer
/// of `room` would cut it: `Ok` with the user's name, `Err` with the decimal
/// uid when the password database has none for it -- a number is not a
/// name.
pub fn uname_within(uid: uv_uid_t, room: usize) -> Result<Vec<u8>, Vec<u8>> {
    let keep = room.max(1) - 1;
    let cut = |mut text: Vec<u8>| {
        text.truncate(keep);
        text
    };
    match entry_by_uid(uid).and_then(|entry| entry.name) {
        Some(name) => Ok(cut(name.into_bytes())),
        // `%d`, as upstream: a uid past `INT_MAX` prints signed. No password
        // database issues one, but the digits have to match either way.
        None => Err(cut(uid.cast_signed().to_string().into_bytes())),
    }
}

/// [`uname_within`] for the user running this process.
pub fn username_named_within(room: usize) -> Result<Vec<u8>, Vec<u8>> {
    uname_within(os_getuid(), room)
}

/// The home directory of user `name`, or `None` when there is no such
/// user.
pub fn user_home_dir(name: &CStr) -> Option<Vec<u8>> {
    if name.is_empty() {
        return None;
    }
    // No password database answers a NULL `pw_dir`, but `None` is the
    // honest answer to one.
    entry_by_name(name)?.dir
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
