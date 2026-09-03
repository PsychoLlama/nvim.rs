//! Namespace registration: the order `nvim_get_namespaces` answers in.
//!
//! The table behind it was a khash, which is insertion-ordered with a
//! swap-remove, and the dict this renders goes out on the wire in that
//! order. Nothing removes a namespace, so "insertion order" is "creation
//! order" — and that is what a client sees.

use std::ffi::CStr;
use std::ptr;

use neovim::api::extmark::{describe_ns, nvim_create_namespace, nvim_get_namespaces};
use neovim::types::String_0;

use super::support::{Sandbox, editor_lock};

/// The names in `nvim_get_namespaces`' dict, in the order it lists them.
///
/// # Safety
/// The caller holds the editor lock.
unsafe fn namespace_names() -> Vec<Vec<u8>> {
    let dict = nvim_get_namespaces(ptr::null_mut());
    (0..dict.size)
        .map(|i| {
            let item = &*dict.items.add(i);
            CStr::from_ptr(item.key.data()).to_bytes().to_vec()
        })
        .collect()
}

/// `nvim_create_namespace` interns the name and hands back a monotone id;
/// asking again answers the same one.
#[test]
fn a_name_keeps_its_id() {
    let _sandbox = Sandbox::globals();
    // SAFETY: the sandbox holds the editor lock, and the names are live.
    unsafe {
        let first = nvim_create_namespace(String_0::from_cstr(c"unit-keeps-1"));
        let second = nvim_create_namespace(String_0::from_cstr(c"unit-keeps-2"));
        assert!(first > 0 && second > first);
        assert_eq!(
            nvim_create_namespace(String_0::from_cstr(c"unit-keeps-1")),
            first
        );
        // An empty name is never interned: every call is a fresh id.
        let anon = nvim_create_namespace(String_0::NULL);
        assert!(anon > second);
        assert!(nvim_create_namespace(String_0::NULL) > anon);
    }
}

/// The listing order is creation order — the khash property this table's
/// replacement has to keep. Checked over the names this case creates, since
/// other cases in the same process create their own.
#[test]
fn namespaces_are_listed_in_creation_order() {
    let _sandbox = Sandbox::globals();
    let created: [&CStr; 3] = [c"unit-order-c", c"unit-order-a", c"unit-order-b"];
    // SAFETY: the sandbox holds the editor lock, and the names are live.
    let names = unsafe {
        for name in created {
            nvim_create_namespace(String_0::from_cstr(name));
        }
        namespace_names()
    };
    let mine: Vec<&[u8]> = names
        .iter()
        .map(Vec::as_slice)
        .filter(|name| name.starts_with(b"unit-order-"))
        .collect();
    let expected: Vec<&[u8]> = created.iter().map(|name| name.to_bytes()).collect();
    assert_eq!(mine, expected);
}

/// `describe_ns` answers the name an id was created under, and the
/// caller's fallback for one that was not.
#[test]
fn describe_ns_answers_the_interned_name() {
    let _editor = editor_lock();
    // SAFETY: the lock is held; the name is live for the call.
    let id = unsafe { nvim_create_namespace(String_0::from_cstr(c"unit-describe")) };
    let name = describe_ns(id.try_into().unwrap(), c"(none)".as_ptr());
    // SAFETY: the answer points into the table's own key, which is a `Box`
    // the table never moves and never frees.
    assert_eq!(unsafe { CStr::from_ptr(name) }, c"unit-describe");
    let missing = describe_ns(1 << 30, c"(none)".as_ptr());
    // SAFETY: as above; here it is the caller's own literal.
    assert_eq!(unsafe { CStr::from_ptr(missing) }, c"(none)");
}
