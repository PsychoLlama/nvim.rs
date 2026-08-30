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
//! through `os_getenv_noalloc`, which stages the answer in the shared
//! `NameBuff`. `cargo test` runs cases on threads; the LuaJIT harness forked
//! a child per case and never had to think about either.

#![cfg(not(miri))]

use std::ffi::{CStr, CString, c_char};
use std::ptr;

use neovim::garray::ga_clear_strings;
use neovim::os::users::{os_get_uname, os_get_userdir, os_get_username, os_get_usernames};
use neovim::types::{Failed, garray_T};

use crate::support::{cstr, editor_lock, take_bytes};

/// A zeroed `garray_T` for a call that initializes it, released with its
/// strings at the end of the case.
struct Names(garray_T);

impl Names {
    fn new() -> Self {
        // SAFETY: `garray_T` is plain data and `os_get_usernames` calls
        // `ga_init` on it before the first write.
        Names(unsafe { std::mem::zeroed() })
    }

    fn fill(&mut self) -> Result<(), Failed> {
        // SAFETY: the array is this frame's and writable.
        unsafe { os_get_usernames(&raw mut self.0) }
    }

    fn to_strings(&self) -> Vec<String> {
        let len = usize::try_from(self.0.ga_len).expect("a garray length is never negative");
        (0..len)
            .map(|i| {
                // SAFETY: `ga_data` holds `ga_len` `char *`, each an
                // `xstrdup`ed NUL-terminated name.
                let name = unsafe { *self.0.ga_data.cast::<*mut c_char>().add(i) };
                unsafe { CStr::from_ptr(name) }
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

impl Drop for Names {
    fn drop(&mut self) {
        // SAFETY: every item is an owned `xstrdup`ed string.
        unsafe { ga_clear_strings(&raw mut self.0) };
    }
}

/// The name the test process is running under, which every case below is
/// written against. `$USER` is what the Lua spec used.
fn current_username() -> String {
    std::env::var("USER").expect("$USER names the user running the tests")
}

/// Write into a 100-byte buffer and read back what landed, the way the spec's
/// `ffi.new('char[100]')` cases did.
fn into_buffer(
    write: impl FnOnce(*mut c_char, usize) -> Result<(), Failed>,
) -> (Result<(), Failed>, String) {
    let mut buffer = [0_u8; 100];
    let result = write(buffer.as_mut_ptr().cast::<c_char>(), buffer.len());
    let end = buffer.iter().position(|&b| b == 0).expect("terminated");
    (
        result,
        String::from_utf8(buffer[..end].to_vec()).expect("a user name is text"),
    )
}

#[test]
fn asking_for_every_username_without_somewhere_to_put_them_fails() {
    let _editor = editor_lock();
    // SAFETY: NULL is the case under test.
    assert_eq!(unsafe { os_get_usernames(ptr::null_mut()) }, Err(Failed));
}

#[test]
fn every_username_includes_the_one_running_the_tests() {
    let _editor = editor_lock();
    let mut names = Names::new();
    assert_eq!(names.fill(), Ok(()));

    let names = names.to_strings();
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
    // SAFETY: the buffer is `into_buffer`'s, and `len` is its true length.
    let (result, name) = into_buffer(|s, len| unsafe { os_get_username(s, len) });
    assert_eq!(result, Ok(()));
    assert_eq!(name, current_username());

    // The same answer by hand, which is what `os_get_username` is: `getuid`
    // then `os_get_uname`.
    // SAFETY: `getuid` cannot fail; the buffer is `into_buffer`'s.
    let uid = unsafe { libc::getuid() };
    let (result, by_uid) = into_buffer(|s, len| unsafe { os_get_uname(uid, s, len) });
    assert_eq!(result, Ok(()));
    assert_eq!(by_uid, name);
}

/// A uid the database has no name for still leaves something in the buffer —
/// the decimal number — and says FAIL, because a number is not a name. Both
/// halves matter: callers print what they were handed either way.
#[test]
fn an_unknown_uid_fails_and_leaves_its_number_behind() {
    let _editor = editor_lock();
    // The spec's "hoping nobody has this uid".
    const NOBODY: u32 = 2342;
    // SAFETY: the buffer is `into_buffer`'s.
    let (result, name) = into_buffer(|s, len| unsafe { os_get_uname(NOBODY, s, len) });
    assert_eq!(result, Err(Failed));
    assert_eq!(name, NOBODY.to_string());
}

#[test]
fn a_home_directory_is_found_by_name_and_nothing_else_is() {
    let _editor = editor_lock();
    // SAFETY: NULL is one of the cases under test; the others are live
    // NUL-terminated names, and each answer is an owned string.
    unsafe {
        assert!(os_get_userdir(ptr::null()).is_null(), "no name, no home");
        // The entry point rejects the empty name itself; glibc's `getpwnam`
        // also answers NULL for it, so this holds either way.
        assert!(
            os_get_userdir(c"".as_ptr()).is_null(),
            "the empty name is no name either"
        );

        let unknown = cstr("neovim_user_not_found_test");
        assert!(os_get_userdir(unknown.as_ptr()).is_null());

        let me = CString::new(current_username()).expect("a user name holds no NUL");
        let home = take_bytes(os_get_userdir(me.as_ptr()));
        assert_eq!(
            String::from_utf8(home).expect("a path is text"),
            std::env::var("HOME").expect("$HOME is set")
        );
    }
}
