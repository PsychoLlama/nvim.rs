//! The argument lists, and the two numbers kept beside them.
//!
//! [`ARG_LISTS`] owns every list: the global one, the one `:args` without a
//! window prints, and each window-local one under the id `:arglocal` minted
//! for it. A window names its list by that id (`w_alist`), so a list is
//! reached through the registry for the length of one borrow and never by
//! an address that could outlive it. [`MAX_ALIST_ID`] mints the ids, and
//! [`ARG_HAD_LAST`] records that the last file in the global list has been
//! reached.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::types::{ArgList, ArgListId, Refcount};
use core::ffi::c_int;
use std::collections::BTreeMap;

/// Every argument list there is.
pub(crate) struct ArgLists {
    global: ArgList,
    local: BTreeMap<ArgListId, ArgList>,
}

impl ArgLists {
    /// The list `id` names, if it is still alive.
    pub(crate) fn get_mut(&mut self, id: ArgListId) -> Option<&mut ArgList> {
        match id {
            ArgListId::GLOBAL => Some(&mut self.global),
            id => self.local.get_mut(&id),
        }
    }

    /// Register a new local list under `id`.
    pub(crate) fn insert(&mut self, id: ArgListId, list: ArgList) {
        self.local.insert(id, list);
    }

    /// Unregister the local list `id`, handing it back.
    pub(crate) fn remove(&mut self, id: ArgListId) -> Option<ArgList> {
        self.local.remove(&id)
    }
}

pub(crate) static ARG_LISTS: GlobalCell<ArgLists> = GlobalCell::new(ArgLists {
    global: ArgList {
        entries: Vec::new(),
        refcount: Refcount::ZERO,
    },
    local: BTreeMap::new(),
});
pub(crate) static MAX_ALIST_ID: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static ARG_HAD_LAST: GlobalCell<bool> = GlobalCell::new(false);
