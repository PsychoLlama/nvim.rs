//! `'tagfunc'`, the user-supplied tag lookup.
//!
//! [`find_tagfunc_tags`] calls the option's callback instead of reading any
//! tags file, validates what comes back — a list of dictionaries with at
//! least `name`, `filename` and `cmd` — and writes each one out in the
//! same shape a tags file line would have had.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::eval::typval::CallFrame;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::optionstr::OptString;
use crate::types::TypVal;
use crate::types::{OptError, OptionSetFlags, VAR_DICT, VAR_LIST, VarLock, kSpecialVarNull};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

/// The global `'tagfunc'` callback. A buffer-local one lives in
/// `b_tfu_cb`.
static tfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback::None);

/// Whether a `'tagfunc'` call is in progress.
///
/// It must not start another, and it must not be allowed to rewrite the
/// tag stack it is being asked to fill.
pub(crate) static tfu_in_use: GlobalCell<bool> = GlobalCell::new(false);

/// The error every malformed `'tagfunc'` answer is reported with.
const E_INVALID_RETURN: &CStr = c"E987: Invalid return value from tagfunc";

/// `'tagfunc'` was set: turn the option's value into a callback.
///
/// The value can be a function name, `function(<name>)`, `funcref(<name>)`
/// or a lambda. Answers NULL, or the error message for an invalid one.
pub fn did_set_tagfunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let value = args
        .os_newval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .as_cstr();
    // Evaluated before either slot is reached: the value may be an
    // expression, and evaluating it may set the option again.
    let cb = callback_from_option(value);
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        cb.map(|cb| install_callback(&mut buf.b_tfu_cb, cb))
    } else {
        let retval = cb.map(|cb| tfu_cb.with_mut(|slot| install_callback(slot, cb)));
        if retval.is_ok() && !args.os_flags.has(OptionSetFlags::GLOBAL) {
            // `:set` without a scope sets the buffer-local copy too.
            set_buflocal_tfu_callback(buf);
        }
        retval
    };
    if retval.is_err() {
        Err(e_invarg.into())
    } else {
        Ok(())
    }
}

/// Mark the global `'tagfunc'` callback so the collector keeps it.
pub fn set_ref_in_tagfunc(copy_id: c_int) -> bool {
    tfu_cb.with(|cb| set_ref_in_callback(cb, copy_id, None, None))
}

/// Copy the global `'tagfunc'` callback into `buffer`'s local one.
///
/// Safe: the buffer is live by [`Buf`]'s promise, and the global callback
/// lives in a static.
pub fn set_buflocal_tfu_callback(mut buffer: Buf) {
    callback_free(&mut buffer.b_tfu_cb);
    tfu_cb.with(|global| {
        if global.is_set() {
            callback_copy(&mut buffer.b_tfu_cb, global);
        }
    });
}

/// What `'tagfunc'` said.
pub(crate) enum TagfuncAnswer {
    /// It answered `v:null`: read the tags files after all.
    Declined,
    /// There is no usable function, or the call failed.
    Unusable,
    /// The tags it found, up to the first malformed one.
    Tags(Vec<Found>),
}

/// Ask `'tagfunc'` for the tags matching `pat`, instead of reading a file.
///
/// Every answer is kept: filtering is the function's own job. `buf_ffname`
/// is the caller's copy of the buffer name the function is told about.
pub(crate) fn find_tagfunc_tags(
    pat: &CStr,
    flags: c_int,
    buf_ffname: Option<&CStr>,
) -> TagfuncAnswer {
    if Buf::current().b_p_tfu.first_byte() == 0 || !Buf::current().b_tfu_cb.is_set() {
        return TagfuncAnswer::Unusable;
    }

    // Which of "c" (the tag is at the cursor), "i" (insert-mode
    // completion) and "r" (the pattern is a regexp) apply.
    let mut flag_string = Vec::with_capacity(3);
    for (wanted, flag) in [
        (g_tag_at_cursor.get(), b'c'),
        (flags & TAG_INS_COMP as c_int != 0, b'i'),
        (flags & TAG_REGEXP as c_int != 0, b'r'),
    ] {
        if wanted {
            flag_string.push(flag);
        }
    }

    let info = tv_dict_alloc_lock(VarLock::Fixed);
    if flags & TAG_INS_COMP as c_int == 0 {
        // The tag stack entry the jump came from, whose `user_data` the
        // function may want. One past the top means nothing was popped,
        // so the newest entry is the interesting one.
        let win = Win::current();
        let at = match win.w_tagstackidx {
            at if at == win.w_tagstacklen => at - 1,
            at => at,
        };
        let user_data = (win.w_tagstacklen > 0)
            .then(|| usize::try_from(at).ok())
            .flatten()
            .and_then(|at| win.w_tagstack.get(at))
            .and_then(|from| from.user_data.as_ref());
        if let Some(user_data) = user_data {
            let _ = info.edit().add_str(b"user_data", Some(user_data.as_cstr()));
        }
    }
    if let Some(buf_ffname) = buf_ffname {
        let _ = info.edit().add_str(b"buf_ffname", Some(buf_ffname));
    }
    // Copies of the pattern and the flags and a reference to the
    // dictionary, all of which the frame releases.
    let mut args = CallFrame::<3>::new();
    args.push_owned(TypVal::string_from(pat.to_bytes()));
    args.push_owned(TypVal::string_from(&flag_string));
    args.push_owned(TypVal::dict(Some(info.clone())));

    let mut rettv = TV_INITIAL_VALUE;
    let save_pos = Win::current().w_cursor;
    // A copy: the function may set 'tagfunc' again.
    let mut callback = Buf::current().b_tfu_cb.duplicate();
    let called = callback_call(&callback, args.args(), &mut rettv);
    callback_free(&mut callback);
    // The function may have moved the cursor, or left it somewhere
    // that no longer exists.
    Win::current().w_cursor = save_pos;
    check_cursor(Win::current());
    drop(args);
    drop(info);

    if !called {
        return TagfuncAnswer::Unusable;
    }
    if rettv.as_special() == Some(kSpecialVarNull) {
        // "Read the tags files after all."
        tv_clear(&mut rettv);
        return TagfuncAnswer::Declined;
    }
    let Some(list) = rettv.list_handle().filter(|_| rettv.v_type() == VAR_LIST) else {
        tv_clear(&mut rettv);
        tag_emsg(E_INVALID_RETURN);
        return TagfuncAnswer::Unusable;
    };

    let mut tags = Vec::new();
    for li in list.items() {
        let parsed = (li.li_tv.v_type() == VAR_DICT)
            .then(|| li.li_tv.dict_handle())
            .flatten()
            .and_then(|d| tag_of(&d, flags));
        let Some(found) = parsed else {
            tag_emsg(E_INVALID_RETURN);
            break;
        };
        // Every match is kept, and none is offered to the duplicate
        // set: `'tagfunc'` does its own filtering.
        tags.push(found);
    }

    drop(list);
    tv_clear(&mut rettv);
    TagfuncAnswer::Tags(tags)
}

/// Write one result dictionary out as a match.
///
/// The shape is the one a tags file line would have had, with the tags
/// file name empty: `name<TAB>file<TAB>cmd`, and then `;"` and the
/// remaining fields when there are any. A `'tagfunc'` match is always a
/// global match in another file.
///
/// Answers `None` when the dictionary is missing `name`, `filename` or
/// `cmd`.
fn tag_of(d: &Dict, flags: c_int) -> Option<Found> {
    let fields = string_fields(d);
    let mut name = None;
    let mut fname = None;
    let mut cmd = None;
    let mut kind = None;
    let mut has_extra = false;

    for field in &fields {
        match field.key {
            b"name" => name = Some(field.value),
            b"filename" => fname = Some(field.value),
            b"cmd" => cmd = Some(field.value),
            b"kind" => {
                has_extra = true;
                kind = Some(field.value);
            }
            _ => has_extra = true,
        }
    }
    let (Some(name), Some(fname), Some(cmd)) = (name, fname, cmd) else {
        return None;
    };

    if flags & TAG_NAMES as c_int != 0 {
        // Only the name is wanted.
        return Some(Found::new(name.to_bytes().to_vec(), MT_GL_OTH as usize));
    }

    // An empty tags file name, and the line.
    let mut out = vec![0];
    out.extend_from_slice(name.to_bytes());
    out.push(b'\t');
    out.extend_from_slice(fname.to_bytes());
    out.push(b'\t');
    out.extend_from_slice(cmd.to_bytes());
    if has_extra {
        out.extend_from_slice(b";\"");
        if let Some(kind) = kind {
            out.push(b'\t');
            out.extend_from_slice(kind.to_bytes());
        }
        for field in &fields {
            if matches!(field.key, b"name" | b"filename" | b"cmd" | b"kind") {
                continue;
            }
            out.push(b'\t');
            out.extend_from_slice(field.key);
            out.push(b':');
            out.extend_from_slice(field.value.to_bytes());
        }
    }
    Some(Found::new(out, MT_GL_OTH as usize))
}

/// One string-valued entry of a `'tagfunc'` result dictionary, borrowed
/// from the item.
struct Field<'a> {
    key: &'a [u8],
    value: &'a CStr,
}

/// The string-valued entries of a dictionary, in hash order.
///
/// Collected once because the match is written in two walks over the
/// same table — the three fields every match has, then the rest — and both
/// skip anything that is not a string.
fn string_fields(d: &Dict) -> Vec<Field<'_>> {
    d.items()
        .filter_map(|item| {
            item.di_tv.string_cstr().map(|value| Field {
                key: item.di_key.bytes(),
                value,
            })
        })
        .collect()
}
