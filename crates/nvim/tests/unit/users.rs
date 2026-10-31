//! The password database, as `os/users.rs` reads it.
//!
//! A port of `test/unit/os/users_spec.lua`. Everything here asks the real
//! system about the real user running the test, which is what made the Lua
//! spec worth having: `best_match`'s table logic is unit-testable in-crate
//! and already is, but "does `getpwuid` of our own uid answer `$USER`" is a
//! statement about libc that only an integration test can make.
//!
//! Two reasons every case takes the editor lock. The `getpw*` family is not
//! reentrant — each call hands back a pointer into one static `passwd` entry
//! that the next call invalidates — and `os_get_usernames` reads `$USER`
//! from the process environment. `cargo test` runs cases on threads; the LuaJIT harness forked
//! a child per case and never had to think about either.

#![cfg(not(miri))]

use std::ffi::CString;

use neovim::os::users::{os_get_usernames, uname_within, user_home_dir, username_named_within};

use crate::support::{cstr, editor_lock};

/// The name the test process is running under, which every case below is
/// written against. `$USER` is what the Lua spec used.
fn current_username() -> String {
    std::env::var("USER").expect("$USER names the user running the tests")
}

/// The spec's `ffi.new('char[100]')` buffers, as the room the answers are
/// cut to.
const ROOM: usize = 100;

/// Bytes the database answered, as text.
fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("a user name is text")
}

#[test]
fn every_username_includes_the_one_running_the_tests() {
    let _editor = editor_lock();
    let names: Vec<String> = os_get_usernames()
        .into_iter()
        .map(|name| String::from_utf8_lossy(&name).into_owned())
        .collect();
    assert!(!names.is_empty(), "the password database is not empty");
    assert!(
        names.contains(&current_username()),
        "{:?} is not in {names:?}",
        current_username()
    );
}

#[test]
fn the_process_owner_is_named_by_its_own_uid() {
    let _editor = editor_lock();
    let name = username_named_within(ROOM).map(text);
    assert_eq!(name, Ok(current_username()));

    // The same answer by hand, which is what `username_named_within` is:
    // `getuid` then `uname_within`.
    // SAFETY: `getuid` cannot fail.
    let uid = unsafe { libc::getuid() };
    assert_eq!(uname_within(uid, ROOM).map(text), name);
    // And cut to the room, terminator counted.
    let short = uname_within(uid, 3).map(text);
    assert_eq!(short, Ok(current_username().chars().take(2).collect()));
}

/// A uid the database has no name for still answers something -- the
/// decimal number -- and says `Err`, because a number is not a name. Both
/// halves matter: callers print what they were handed either way.
#[test]
fn an_unknown_uid_fails_and_leaves_its_number_behind() {
    let _editor = editor_lock();
    // The spec's "hoping nobody has this uid".
    const NOBODY: u32 = 2342;
    assert_eq!(
        uname_within(NOBODY, ROOM).map_err(text),
        Err(NOBODY.to_string())
    );
}

#[test]
fn a_home_directory_is_found_by_name_and_nothing_else_is() {
    let _editor = editor_lock();
    // The entry point rejects the empty name itself; glibc's `getpwnam`
    // also answers NULL for it, so this holds either way.
    assert!(user_home_dir(c"").is_none(), "the empty name is no name");
    assert!(user_home_dir(&cstr("neovim_user_not_found_test")).is_none());

    let me = CString::new(current_username()).expect("a user name holds no NUL");
    let home = user_home_dir(&me).expect("the current user has a home");
    assert_eq!(
        String::from_utf8(home).expect("a path is text"),
        std::env::var("HOME").expect("$HOME is set")
    );
}
