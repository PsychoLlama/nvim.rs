//! The `Callback` value: building one from a typval, its lifetime (copy,
//! release, comparison, rendering) and calling it.
//!
//! What still needs `unsafe` here is the edge of two other models: Lua's
//! (releasing and rendering a `LuaRef`, calling one, registering a table as
//! a function) and the collector's raw marking stacks; plus reading a
//! partial's name, which is a pointer the partial owns.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::{LUA_NOREF, PartialRef, partial_name};
use crate::guard::Depth;
use crate::memory::ThinCString;
use core::ffi::{CStr, c_int};
use core::mem::ManuallyDrop;
use core::ptr::{null, null_mut};

use crate::ascii::ascii_isdigit;
use crate::eval::collect::{set_ref_in_item_dict, set_ref_in_item_partial};
use crate::eval::userfunc::{
    CallWith, call_func_with, func_ref_name, func_unref_name, scriptlocal_funcname,
};
use crate::eval::vars::{emsg_static, lua_partial};
use crate::eval::{ARRAY_DICT_INIT, callback_depth, check_luafunc_name, kRetNilBool};
use crate::lua::executor::{
    api_free_luaref, api_new_luaref, nlua_call_ref_quiet, nlua_funcref_str, nlua_is_table_from_lua,
    nlua_register_table_as_callable,
};
use crate::message::e_command_too_recursive;
use crate::option::vars::p_mfd;
use crate::types::{
    Callback, CallbackReader, HtStack, ListStack, OptInt, TypVal, VAR_FUNC, VAR_NUMBER,
    VAR_SPECIAL, VAR_STRING, kSpecialVarNull,
};
use crate::winlayer::Win;

/// Build a `Callback` out of whatever the user handed a builtin, into
/// `callback`, which is overwritten without being released.
///
/// A String or a Funcref names a function; a partial is taken as it is; a
/// Lua table with a `__call` is registered as one. `v:null` and the Number
/// 0 mean "no callback", which is not an error. A String that *starts with
/// a digit* is refused, because that is a channel id rather than a name.
pub fn callback_from_typval(callback: &mut Callback, arg: &TypVal) -> bool {
    let tv = arg;
    let mut failed = false;
    let cb = if let TypVal::Partial(partial) = tv
        && let Some(partial) = &**partial
    {
        Callback::Partial(ManuallyDrop::new(partial.clone()))
    } else if tv
        .string_ref()
        .is_some_and(|text| ascii_isdigit(c_int::from(text.first())))
    {
        failed = true;
        Callback::None
    } else if tv.v_type() == VAR_FUNC || tv.v_type() == VAR_STRING {
        match tv.text_or_name() {
            None => {
                failed = true;
                Callback::None
            }
            Some(name) if name.is_empty() => Callback::None,
            Some(name) => {
                // A plain String may name a script-local function, which
                // has to be resolved against the current script now.
                let scriptlocal = if tv.v_type() == VAR_STRING {
                    scriptlocal_funcname(name.as_bytes())
                } else {
                    None
                };
                let funcref = scriptlocal.map_or_else(|| name.clone(), ThinCString::from);
                func_ref_name(funcref.as_cstr());
                Callback::Funcref(ManuallyDrop::new(funcref))
            }
        }
    } else if nlua_is_table_from_lua(arg) {
        // SAFETY: a live value; the table has a `__call`.
        let name = unsafe { nlua_register_table_as_callable(arg) };
        if name.is_null() {
            failed = true;
            Callback::None
        } else {
            // SAFETY: `name` is the registered function's NUL-terminated
            // name, copied here.
            let name = ThinCString::from_cstr(unsafe { CStr::from_ptr(name) });
            Callback::Funcref(ManuallyDrop::new(name))
        }
    } else if tv.v_type() == VAR_SPECIAL || (tv.v_type() == VAR_NUMBER && tv.number_or_zero() == 0)
    {
        Callback::None
    } else {
        failed = true;
        Callback::None
    };
    *callback = cb;

    if failed {
        emsg_static(c"E921: Invalid callback argument");
        return false;
    }
    true
}

impl Callback {
    /// The callback `arg` describes — [`Callback::None`] for `v:null` and
    /// the Number 0 — or `None`, after E921, when it describes none.
    pub fn from_typval(arg: &TypVal) -> Option<Callback> {
        let mut cb = Callback::None;
        callback_from_typval(&mut cb, arg).then_some(cb)
    }

    /// Call this callback with `args`, answering whether it ran.
    pub fn call(&self, args: &[TypVal], result: &mut TypVal) -> bool {
        callback_call(self, args, result)
    }

    /// Mark what this callback keeps alive for the collector, answering
    /// whether the walk should be given up.
    pub fn mark(&self, copy_id: c_int) -> bool {
        // SAFETY: no stacks, as for a root.
        unsafe { set_ref_in_callback(self, copy_id, null_mut(), null_mut()) }
    }

    /// Release what this holds and leave it [`Callback::None`].
    pub fn clear(&mut self) {
        match ::core::mem::replace(self, Callback::None) {
            Callback::Funcref(name) => {
                let name = ManuallyDrop::into_inner(name);
                func_unref_name(name.as_cstr());
                drop(name);
            }
            Callback::Partial(partial) => drop(ManuallyDrop::into_inner(partial)),
            // NLUA_CLEAR_REF
            Callback::Lua(reference) => {
                if reference != LUA_NOREF {
                    // SAFETY: a registry index this callback owned.
                    unsafe { api_free_luaref(reference) };
                }
            }
            Callback::None => {}
        }
    }

    /// A second owner of what this holds.
    pub fn duplicate(&self) -> Callback {
        match self {
            Callback::Partial(partial) => Callback::Partial(ManuallyDrop::new((**partial).clone())),
            Callback::Funcref(name) => {
                func_ref_name(name.as_cstr());
                Callback::Funcref(ManuallyDrop::new((**name).clone()))
            }
            Callback::Lua(reference) => Callback::Lua(api_new_luaref(*reference)),
            Callback::None => Callback::None,
        }
    }

    /// Whether this and `other` name the same function: upstream's
    /// `tv_callback_equal`.
    pub fn same_as(&self, other: &Callback) -> bool {
        match (self, other) {
            (Callback::None, Callback::None) => true,
            (Callback::Funcref(a), Callback::Funcref(b)) => a.as_bytes() == b.as_bytes(),
            (Callback::Partial(a), Callback::Partial(b)) => a.ptr_eq(b),
            (Callback::Lua(a), Callback::Lua(b)) => a == b,
            _ => false,
        }
    }

    /// Store this callback in `tv` as a Vimscript value, taking a reference
    /// of the value's own. `tv` is overwritten without being released.
    ///
    /// A Lua callback has no Vimscript form and comes out as `v:null`.
    pub fn put(&self, tv: &mut TypVal) {
        match self {
            Callback::Partial(partial) => tv.write_partial(Some((**partial).clone())),
            Callback::Funcref(name) => {
                tv.write_func_name(Some((**name).clone()));
                func_ref_name(name.as_cstr());
            }
            // A Lua callback and no callback at all have no Vimscript form.
            Callback::Lua(_) | Callback::None => tv.write_special(kSpecialVarNull),
        }
    }
}

/// Drop whatever `callback` holds and leave it `kCallbackNone`.
pub fn callback_free(callback: &mut Callback) {
    callback.clear();
}

/// Store `cb` in `tv` as a Vimscript value; see [`Callback::put`].
pub fn callback_put(cb: &Callback, tv: &mut TypVal) {
    cb.put(tv);
}

/// Copy `src` into `dest`, taking a reference to whatever it holds. `dest`
/// is overwritten without being released.
pub fn callback_copy(dest: &mut Callback, src: &Callback) {
    *dest = src.duplicate();
}

/// A description of `callback`, as `string()` prints it.
///
/// The text is upstream's `snprintf` into a hundred bytes, and is cut where
/// that cut it.
pub fn callback_to_string(callback: &Callback) -> ThinCString {
    const MSGLEN: usize = 100;
    let mut text: Vec<u8> = match callback {
        Callback::Lua(reference) => {
            // SAFETY: a registry index; the rendering is a block of its own,
            // which the answer takes over.
            let rendered = unsafe { ThinCString::from_raw(nlua_funcref_str(*reference)) };
            return rendered.unwrap_or_else(ThinCString::empty);
        }
        Callback::Funcref(name) => [b"<vim function: ", name.as_bytes(), b">"].concat(),
        Callback::Partial(partial) => {
            // A partial with no name of its own prints as `%s` did NULL.
            let name = partial
                .pt_name
                .as_ref()
                .map_or(&b"(null)"[..], |n| n.as_bytes());
            [b"<vim partial: ", name, b">"].concat()
        }
        // Anything else is an empty string.
        Callback::None => Vec::new(),
    };
    text.truncate(MSGLEN - 1);
    ThinCString::from_vec(text)
}

/// How deep the callback nesting currently is.
pub(crate) fn get_callback_depth() -> c_int {
    callback_depth.get()
}

/// The prefix that makes a funcref name a Lua one.
const VLUA: &CStr = c"v:lua.";

/// Call `callback` with `args`.
pub fn callback_call(callback: &Callback, args: &[TypVal], result: &mut TypVal) -> bool {
    if OptInt::from(callback_depth.get()) > p_mfd() {
        emsg_static(e_command_too_recursive);
        return false;
    }

    let lua;
    let (name, partial): (&CStr, Option<&PartialRef>) = match callback {
        Callback::Funcref(funcref) => {
            let full = funcref.as_cstr();
            match full.to_bytes_with_nul().strip_prefix(VLUA.to_bytes()) {
                Some(rest) => {
                    let name = CStr::from_bytes_until_nul(rest).expect("a terminated name");
                    if check_luafunc_name(name.to_bytes(), false) == 0 {
                        return false;
                    }
                    lua = lua_partial();
                    (name, lua.as_ref())
                }
                None => (full, None),
            }
        }
        Callback::Partial(held) => (partial_name(held), Some(&**held)),
        Callback::Lua(luaref) => {
            // A Lua reference is called directly, with no arguments —
            // this is the "is it still wanted" question, not a
            // general-purpose call.
            let no_args = ARRAY_DICT_INIT;
            // SAFETY: the reference is the one the callback owns, and the
            // call is handed no arguments, no arena and no error sink.
            let rv = unsafe { nlua_call_ref_quiet(*luaref, null(), no_args, kRetNilBool) };
            return rv.as_boolean().unwrap_or(false);
        }
        Callback::None => return false,
    };

    let mut with = CallWith::new(true);
    with.firstline = Win::current().w_cursor.lnum;
    with.lastline = Win::current().w_cursor.lnum;
    with.partial = partial;
    // The un-bump is the guard's, so that an early exit cannot skip it.
    let depth = Depth::of(&callback_depth);
    let ret = call_func_with(name, None, result, args, with);
    drop(depth);
    ret.is_ok()
}

/// Mark what a callback keeps alive for the collector.
///
/// # Safety
/// The two stacks as `set_ref_in_item`'s.
pub unsafe fn set_ref_in_callback(
    callback: &Callback,
    copy_id: c_int,
    ht_stack: *mut *mut HtStack,
    list_stack: *mut *mut ListStack,
) -> bool {
    match callback {
        // SAFETY: the stacks are the caller's.
        Callback::Partial(partial) => unsafe {
            set_ref_in_item_partial(partial, copy_id, ht_stack, list_stack)
        },
        // A Lua reference is the Lua garbage collector's, not this one's,
        // and nothing that reaches here should hold one.
        Callback::Lua(_) => unreachable!("set_ref_in_callback on a Lua callback"),
        // A funcref and no callback at all hold nothing collectable.
        Callback::Funcref(_) | Callback::None => false,
    }
}

/// Mark what a callback *reader* keeps alive: its callback, and the `self`
/// dictionary it would be called with.
///
/// # Safety
/// `reader` must be valid; the stacks as `set_ref_in_callback`'s.
pub(crate) unsafe fn set_ref_in_callback_reader(
    reader: *mut CallbackReader,
    copy_id: c_int,
    ht_stack: *mut *mut HtStack,
    list_stack: *mut *mut ListStack,
) -> bool {
    // SAFETY: the caller's promise -- the reader outlives the call, and its
    // `cb` is the callback it owns.
    if unsafe { set_ref_in_callback(&(*reader).cb, copy_id, ht_stack, list_stack) } {
        return true;
    }
    // SAFETY: as above.
    let self_dict = unsafe { (*reader).self_0 };
    // SAFETY: the reader's own dictionary, null or live; the stacks are the
    // caller's.
    unsafe { set_ref_in_item_dict(self_dict, copy_id, ht_stack, list_stack) }
}
