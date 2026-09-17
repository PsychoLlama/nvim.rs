//! 'cpoptions' held at a value for the length of an operation.
//!
//! A dozen places compile a pattern, read a digraph or match a pair with
//! `'cpoptions'` emptied or forced, so that a user's flags cannot change
//! what the operation means, and put the option back afterwards. Upstream
//! writes that out at each site, and three of the copies had grown their own
//! guard type. [`SavedCpo`] is the one guard, and the restore is what it
//! exists for.
//!
//! **Putting the value back is not always an assignment.** Where user code
//! runs under the guard — a `{skip}` expression, an autocommand, a plugin
//! sourced by `:helpgrep` — that code may set `'cpoptions'` itself. The
//! option owning *nothing* is how the guard recognises its own empty value
//! (`Option::None`, which is upstream's shared `empty_string_option`): still
//! unset means nothing touched it, and the saved value goes straight back.
//! An option that owns an *empty* string was set and restored behind the
//! guard's back, and the saved value has to go back through the option
//! machinery so that everything watching `'cpoptions'` hears about it. A
//! non-empty one is the user's, and the guard leaves it alone.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use crate::memory::XString;
use crate::option::vars::{P_CPO, p_cpo};
use crate::options::kOptCpoptions;
use crate::types::{OptVal, OptionSetFlags, String_0};

use super::set_option_value_give_err;

/// 'cpoptions' held at a value until this drops. See the module docs.
pub(crate) struct SavedCpo {
    /// What the option owned on entry.
    saved: Option<XString>,
    /// Whether user code may have set the option meanwhile, which is what
    /// makes the restore more than an assignment.
    reentrant: bool,
}

impl SavedCpo {
    /// Empty 'cpoptions' until the guard drops, for an operation that runs
    /// no user code.
    pub(crate) fn empty() -> Self {
        Self {
            saved: P_CPO.clear(),
            reentrant: false,
        }
    }

    /// Empty 'cpoptions' until the guard drops, for an operation that hands
    /// control to user code — an expression, an autocommand, a plugin.
    pub(crate) fn empty_under_user_code() -> Self {
        Self {
            saved: P_CPO.clear(),
            reentrant: true,
        }
    }

    /// 'cpoptions' held at `flags` until the guard drops.
    pub(crate) fn held(flags: &CStr) -> Self {
        Self {
            saved: P_CPO.swap(Some(XString::from_cstr(flags))),
            reentrant: false,
        }
    }
}

impl Drop for SavedCpo {
    fn drop(&mut self) {
        let saved = self.saved.take();
        if !self.reentrant || P_CPO.is_unset() {
            P_CPO.restore(saved);
            return;
        }
        if p_cpo(CStr::is_empty) {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal::string(String_0::from_xstring(saved.unwrap_or_default())),
                OptionSetFlags::NONE,
            );
        }
    }
}
