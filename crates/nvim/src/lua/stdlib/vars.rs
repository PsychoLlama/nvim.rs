//! `vim.g` / `vim.b` / `vim.w` / `vim.t` / `vim.v`: the scope accessors.
//!
//! [`var_scope`] resolves the (scope, handle) pair the accessor was built
//! with to the dictionary it names, and [`setvar`]/[`getvar`] are the
//! `__newindex`/`__index` behind it — including the `vim.v` rules, where a
//! variable may be read-only or fixed-typed.
//!
//! A watched dictionary runs its watchers -- Vimscript, so anything -- in the
//! middle of an assignment, so no item is held across one: the value a
//! watcher is shown is a copy, and the item is found again by its key.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::api::private::helpers::{
    dict_check_writable, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
};
use crate::eval::typval::{
    DictRef, TV_INITIAL_VALUE, dict_find, dict_is_watched, dict_watcher_notify, tv_clear, tv_copy,
    tv_dict_item_remove,
};
use crate::eval::vars::{VvarStore, before_set_vvar, globvar_dict, vimvar_dict};
use crate::ex_eval::aborting;
use crate::lua::converter::{pop_typval, push_typval};
use crate::lua::executor::until_nul;
use crate::lua::stack::{At, Raise, Stack};
use crate::narrow::number_as_int;
use crate::runtime::script_autoload_named;
use crate::types::{DictItem, Error, Handle, ScopeDictItem, String_0};

/// The dictionary a scope entry holds, if it holds one yet.
fn scope_dict(entry: &ScopeDictItem) -> Option<DictRef> {
    entry.di_tv.dict_handle()
}

/// The dictionary the `(scope, handle)` pair at stack slots 1 and 2 names;
/// `None` for a scope whose dictionary does not exist yet.
///
/// A scope letter it does not know is refused, and so is a handle that names
/// nothing.
fn var_scope(stack: &mut Stack) -> Result<Option<DictRef>, Raise> {
    // `luaL_checkstring`'s C string.
    let scope = until_nul(stack.check_bytes(At::arg(1))?).to_vec();
    let handle: Handle =
        number_as_int(i64::try_from(stack.check_integer(At::arg(2))?).unwrap_or_default());
    let found: Result<Option<DictRef>, Error> =
        match scope.as_slice() {
            b"g" => Ok(Some(globvar_dict())),
            b"v" => Ok(Some(vimvar_dict())),
            b"b" => find_buffer_by_handle(handle)
                .map(|buf| buf.and_then(|buf| scope_dict(&buf.b_bufvar))),
            b"w" => find_window_by_handle(handle)
                .map(|win| win.and_then(|win| scope_dict(&win.w_winvar))),
            b"t" => find_tab_by_handle(handle)
                .map(|tabpage| tabpage.and_then(|tabpage| scope_dict(&tabpage.tp_winvar))),
            _ => return Err(Raise::message("invalid scope")),
        };
    found.map_err(|err| {
        let why = err.message_or_empty().to_bytes();
        Raise::message([b"scoped variable: ".as_slice(), why].concat())
    })
}

/// `__newindex` on a scope table: set slot 3's key to slot 4's value, or
/// delete it when slot 4 is absent or nil.
pub(crate) fn setvar(stack: &mut Stack) -> Result<c_int, Raise> {
    let dict = var_scope(stack)?.expect("a live scope holds its dictionary");
    // Copied out of the Lua state: the key outlives the value below.
    let key = String_0::from_bytes(stack.check_bytes(At::arg(3))?);

    let del = stack.top().index() < 4 || stack.is_nil(At::arg(4));

    let exists = match dict_check_writable(&dict, &key, del) {
        Ok(exists) => exists,
        Err(e) => return Err(Raise::message(e.message_or_empty().to_bytes())),
    };

    let watched = dict_is_watched(Some(&*dict));

    if del {
        if !exists {
            // Doesn't exist, nothing to do.
            return Ok(0);
        }
        if watched {
            // The watchers are shown a copy: they run Vimscript, which may
            // do anything to the item.
            let mut old = TV_INITIAL_VALUE;
            if let Some(item) = dict.find(key.as_bytes()) {
                tv_copy(&item.di_tv, &mut old);
            }
            dict_watcher_notify(&dict, key.as_cstr(), None, Some(&old));
            tv_clear(&mut old);
        }
        tv_dict_item_remove(&dict, key.as_bytes());
        return Ok(0);
    }

    // Convert the Lua value into a temporary before anything is disturbed.
    stack.push_value(At::arg(4));
    let Some(mut tv) = pop_typval(stack) else {
        return Err(Raise::message("Couldn't convert lua value"));
    };

    let mut oldtv = TV_INITIAL_VALUE;
    if exists {
        // `v:` keys are typed, and some of them run a hook on assignment.
        let verdict = dict
            .ptr_eq(&vimvar_dict())
            .then(|| before_set_vvar(key.as_bytes(), &mut tv, true, watched));
        if let Some(verdict) = verdict.filter(|verdict| *verdict != VvarStore::Store) {
            tv_clear(&mut tv);
            if verdict == VvarStore::TypeError {
                let name = until_nul(key.as_bytes());
                return Err(Raise::message(
                    [b"Setting v:".as_slice(), name, b" to value with wrong type"].concat(),
                ));
            }
            return Ok(0);
        }
    }
    match dict.edit().find_mut(key.as_bytes()) {
        Some(item) => {
            if watched {
                tv_copy(&item.di_tv, &mut oldtv);
            }
            tv_clear(&mut item.di_tv);
            tv_copy(&tv, &mut item.di_tv);
        }
        None => {
            // Added empty and then filled, as upstream does: adding a
            // Funcref under a lower-case name would be refused (E704).
            let _ = dict.edit().add_item(DictItem::boxed(key.as_bytes()));
            if let Some(item) = dict.edit().find_mut(key.as_bytes()) {
                tv_copy(&tv, &mut item.di_tv);
            }
        }
    }

    if watched {
        dict_watcher_notify(&dict, key.as_cstr(), Some(&tv), Some(&oldtv));
        tv_clear(&mut oldtv);
    }
    tv_clear(&mut tv);
    Ok(0)
}

/// `__index` on a scope table: push the value of slot 3's key, or nothing.
///
/// A miss in `g:` tries the autoload directory once before giving up.
pub(crate) fn getvar(stack: &mut Stack) -> Result<c_int, Raise> {
    let dict = var_scope(stack)?;
    stack.check_bytes(At::arg(3))?;
    let is_globals = dict
        .as_ref()
        .is_some_and(|dict| dict.ptr_eq(&globvar_dict()));
    let item = {
        let name = stack.string(At::arg(3)).unwrap_or_default();
        let mut item = dict_find(dict.as_deref(), name);
        if item.is_none() && is_globals {
            if !script_autoload_named(name, false) || aborting() {
                return Ok(0); // nil
            }
            // The autoload ran arbitrary Vimscript, so the lookup starts
            // again rather than reusing what it may have changed.
            item = dict_find(dict.as_deref(), name);
        }
        item
    };
    let Some(item) = item else {
        return Ok(0); // nil
    };
    push_typval(stack, &item.di_tv, false);
    Ok(1)
}
