//! `'tagfunc'`, the user-supplied tag lookup.
//!
//! [`find_tagfunc_tags`] calls the option's callback instead of reading any
//! tags file, validates what comes back — a list of dictionaries with at
//! least `name`, `filename` and `cmd` — and writes each one out in the
//! same shape a tags file line would have had.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::hashtab::hash_removed;
use crate::types::{
    FAIL, OK, OptionSetFlags, VAR_DICT, VAR_FIXED, VAR_LIST, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, kSpecialVarNull,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The global `'tagfunc'` callback. A buffer-local one lives in
/// `b_tfu_cb`.
static tfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ptr::null_mut(),
    },
    type_0: kCallbackNone,
});

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
///
/// # Safety
/// `args` must describe the option being set.
pub unsafe fn did_set_tagfunc(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's promise; the new value is a NUL-terminated
    // option string and `os_buf` is the buffer it applies to.
    unsafe {
        let buf = (*args).os_buf.cast::<buf_T>();
        let value = (*args).os_newval.string.data();
        let retval = if (*args).os_flags.has(OptionSetFlags::LOCAL) {
            option_set_callback_func(value, &raw mut (*buf).b_tfu_cb)
        } else {
            let retval = option_set_callback_func(value, tfu_cb.ptr());
            if retval == OK && !(*args).os_flags.has(OptionSetFlags::GLOBAL) {
                // `:set` without a scope sets the buffer-local copy too.
                set_buflocal_tfu_callback(buf);
            }
            retval
        };
        if retval == FAIL {
            (&raw const e_invarg).cast()
        } else {
            ptr::null()
        }
    }
}

/// Mark the global `'tagfunc'` callback so the collector keeps it.
///
/// # Safety
/// Must be called from a garbage-collection sweep.
pub unsafe fn set_ref_in_tagfunc(copyID: c_int) -> bool {
    // SAFETY: the caller's promise.
    unsafe { set_ref_in_callback(tfu_cb.ptr(), copyID, ptr::null_mut(), ptr::null_mut()) }
}

/// Copy the global `'tagfunc'` callback into `buf`'s local one.
///
/// # Safety
/// `buf` must be live.
pub unsafe fn set_buflocal_tfu_callback(buf: *mut buf_T) {
    // SAFETY: the caller's promise; the buffer owns its own callback.
    unsafe {
        callback_free(&raw mut (*buf).b_tfu_cb);
        if (*tfu_cb.ptr()).type_0 != kCallbackNone {
            callback_copy(&raw mut (*buf).b_tfu_cb, tfu_cb.ptr());
        }
    }
}

/// Ask `'tagfunc'` for the tags matching `pat`, instead of reading a file.
///
/// Every answer is kept: filtering is the function's own job. Answers `OK`
/// when the call succeeded, `NOTDONE` when the function returned
/// `v:null` (meaning "read the tags files after all"), and `FAIL`
/// otherwise.
///
/// # Safety
/// `pat` must be NUL-terminated, `buf_ffname` NULL or NUL-terminated, and
/// `curbuf`/`curwin` must be live.
pub(crate) unsafe fn find_tagfunc_tags(
    pat: *mut c_char,
    found: &mut Vec<Match>,
    match_count: &mut c_int,
    flags: c_int,
    buf_ffname: *mut c_char,
) -> c_int {
    // SAFETY: the caller's promise. `flag_string` and `info` outlive the
    // call they are arguments to, and the list the callback answers is
    // cleared before returning.
    unsafe {
        // The tag stack entry the jump came from, whose `user_data` the
        // function may want. One past the top means nothing was popped, so
        // the newest entry is the interesting one.
        let win = curwin.get();
        let from = if (*win).w_tagstacklen > 0 {
            let at = (*win).w_tagstackidx;
            let at = if at == (*win).w_tagstacklen {
                at - 1
            } else {
                at
            };
            (*win).w_tagstack.get(at as usize)
        } else {
            None
        };

        if *(*curbuf.get()).b_p_tfu == 0 || (*curbuf.get()).b_tfu_cb.type_0 == kCallbackNone {
            return FAIL;
        }

        // Which of "c" (the tag is at the cursor), "i" (insert-mode
        // completion) and "r" (the pattern is a regexp) apply.
        let mut flag_string = [0 as c_char; 4];
        let mut at = 0;
        for (wanted, flag) in [
            (g_tag_at_cursor.get(), b'c'),
            (flags & TAG_INS_COMP as c_int != 0, b'i'),
            (flags & TAG_REGEXP as c_int != 0, b'r'),
        ] {
            if wanted {
                flag_string[at] = flag as c_char;
                at += 1;
            }
        }

        let info = tv_dict_alloc_lock(VAR_FIXED);
        if flags & TAG_INS_COMP as c_int == 0 {
            if let Some(from) = from
                && !from.user_data.is_null()
            {
                add_str(info, c"user_data", from.user_data);
            }
        }
        if !buf_ffname.is_null() {
            add_str(info, c"buf_ffname", buf_ffname);
        }
        // Held alive for the call: the dict is ours, not the argument
        // list's.
        (*info).dv_refcount += 1;

        let mut args = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 4];
        args[0].v_type = VAR_STRING;
        args[0].vval.v_string = pat;
        args[1].v_type = VAR_STRING;
        args[1].vval.v_string = flag_string.as_mut_ptr();
        args[2].v_type = VAR_DICT;
        args[2].vval.v_dict = info;

        let mut rettv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let save_pos = (*curwin.get()).w_cursor;
        let mut result = callback_call(
            &raw mut (*curbuf.get()).b_tfu_cb,
            3,
            args.as_mut_ptr(),
            &raw mut rettv,
        ) as c_int;
        // The function may have moved the cursor, or left it somewhere
        // that no longer exists.
        (*curwin.get()).w_cursor = save_pos;
        check_cursor(curwin.get());
        (*info).dv_refcount -= 1;

        if result == FAIL {
            return FAIL;
        }
        if rettv.v_type == VAR_SPECIAL && rettv.vval.v_special == kSpecialVarNull {
            // "Read the tags files after all."
            tv_clear(&raw mut rettv);
            return NOTDONE;
        }
        if rettv.v_type != VAR_LIST || rettv.vval.v_list.is_null() {
            tv_clear(&raw mut rettv);
            emsg(gettext(E_INVALID_RETURN.as_ptr()));
            return FAIL;
        }

        let mut ntags = 0;
        let mut li = (*rettv.vval.v_list).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type != VAR_DICT {
                emsg(gettext(E_INVALID_RETURN.as_ptr()));
                break;
            }
            let Some(mfp) = tag_of((*li).li_tv.vval.v_dict, flags) else {
                emsg(gettext(E_INVALID_RETURN.as_ptr()));
                break;
            };
            // Every match is kept, and none is offered to the duplicate
            // set: `'tagfunc'` does its own filtering.
            found.push(mfp);
            ntags += 1;
            result = OK;
            li = (*li).li_next;
        }

        tv_clear(&raw mut rettv);
        *match_count = ntags;
        result
    }
}

/// Write one result dictionary out as a match.
///
/// The shape is the one a tags file line would have had, with the tags
/// file name empty: `<priority><TAG_SEP>name<TAB>file<TAB>cmd`, and then
/// `;"` and the remaining fields when there are any.
///
/// Answers `None` when the dictionary is missing `name`, `filename` or
/// `cmd`.
///
/// # Safety
/// `d` must be a live dictionary.
unsafe fn tag_of(d: *mut dict_T, flags: c_int) -> Option<Match> {
    // SAFETY: the caller's promise; every value is a NUL-terminated
    // string, and the buffer is sized before anything is written.
    unsafe {
        let fields = string_fields(d);
        let mut name = ptr::null_mut::<c_char>();
        let mut fname = ptr::null_mut::<c_char>();
        let mut cmd = ptr::null_mut::<c_char>();
        let mut kind = ptr::null_mut::<c_char>();
        let mut has_extra = false;

        // Upstream's own sizing: two for the leading bytes, then a
        // separator and the text of every value, plus each extra field's
        // key and colon, plus two for the `;"`.
        let mut len = 2;
        for field in &fields {
            len += strlen(field.value) + 1;
            match field.key().to_bytes() {
                b"name" => name = field.value,
                b"filename" => fname = field.value,
                b"cmd" => cmd = field.value,
                b"kind" => {
                    has_extra = true;
                    kind = field.value;
                }
                key => {
                    has_extra = true;
                    len += key.len() + 1;
                }
            }
        }
        if has_extra {
            len += 2;
        }
        if name.is_null() || fname.is_null() || cmd.is_null() {
            return None;
        }

        if flags & TAG_NAMES as c_int != 0 {
            // Only the name is wanted.
            let bytes = CStr::from_ptr(name).to_bytes_with_nul();
            let mut mfp = Match::zeroed(bytes.len());
            mfp.bytes().copy_from_slice(bytes);
            return Some(mfp);
        }

        let mut out = Vec::with_capacity(len);
        // A `'tagfunc'` match is always a global match in another file,
        // and it names no tags file.
        out.push(MT_GL_OTH as u8 + 1);
        out.push(TAG_SEP as u8);
        out.extend_from_slice(CStr::from_ptr(name).to_bytes());
        out.push(b'\t');
        out.extend_from_slice(CStr::from_ptr(fname).to_bytes());
        out.push(b'\t');
        out.extend_from_slice(CStr::from_ptr(cmd).to_bytes());
        if has_extra {
            out.extend_from_slice(b";\"");
            if !kind.is_null() {
                out.push(b'\t');
                out.extend_from_slice(CStr::from_ptr(kind).to_bytes());
            }
            for field in &fields {
                if matches!(
                    field.key().to_bytes(),
                    b"name" | b"filename" | b"cmd" | b"kind"
                ) {
                    continue;
                }
                out.push(b'\t');
                out.extend_from_slice(field.key().to_bytes());
                out.push(b':');
                out.extend_from_slice(CStr::from_ptr(field.value).to_bytes());
            }
        }
        out.push(0);

        // Two bytes of slack past the text, as upstream leaves.
        let mut mfp = Match::zeroed(len + 2);
        mfp.bytes()[..out.len()].copy_from_slice(&out);
        Some(mfp)
    }
}

/// One string-valued entry of a `'tagfunc'` result dictionary.
struct Field {
    /// Points into the dictionary item, which outlives the walk.
    key: *const c_char,
    value: *mut c_char,
}

impl Field {
    /// # Safety
    /// The dictionary the field came from must still be live.
    unsafe fn key(&self) -> &CStr {
        // SAFETY: the caller's promise; a dict key is NUL-terminated.
        unsafe { CStr::from_ptr(self.key) }
    }
}

/// The string-valued entries of a dictionary, in hash order.
///
/// Collected once because upstream walks the same table twice — first to
/// size the match, then to write it — and both walks skip anything that is
/// not a string.
///
/// # Safety
/// `d` must be a live dictionary.
unsafe fn string_fields(d: *mut dict_T) -> Vec<Field> {
    let mut fields = Vec::new();
    // SAFETY: the caller's promise; the walk visits `ht_used` live items
    // and every item's key and value are part of it.
    unsafe {
        let ht = &raw mut (*d).dv_hashtab;
        let mut todo = (*ht).ht_used;
        let mut hi = (*ht).ht_array;
        while todo != 0 {
            let key = (*hi).hi_key;
            if !key.is_null() && key != (&raw const hash_removed).cast_mut() {
                todo -= 1;
                let di = key
                    .byte_sub(core::mem::offset_of!(dictitem_T, di_key))
                    .cast::<dictitem_T>();
                let tv = &raw mut (*di).di_tv;
                if (*tv).v_type == VAR_STRING && !(*tv).vval.v_string.is_null() {
                    fields.push(Field {
                        key: (&raw const (*di).di_key).cast(),
                        value: (*tv).vval.v_string,
                    });
                }
            }
            hi = hi.add(1);
        }
    }
    fields
}

/// [`tv_dict_add_str`] with the key's length taken from the literal.
///
/// # Safety
/// `d` must be live and `val` NUL-terminated.
unsafe fn add_str(d: *mut dict_T, key: &CStr, val: *const c_char) {
    // SAFETY: the caller's promise.
    unsafe { tv_dict_add_str(d, key.as_ptr(), key.count_bytes(), val) };
}
