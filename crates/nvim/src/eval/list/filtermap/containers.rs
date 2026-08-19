//! The four per-container walks behind `filter()`/`map()`/`mapnew()`/
//! `foreach()`.
//!
//! A Dict walks its hashtab with the table locked so a callback cannot
//! rehash it; a List asks each item for its successor *after* the callback
//! has run, because the callback may have removed the item; a Blob walks
//! bytes and rewrites them in place, closing the gap when `filter()` drops
//! one; a String walks characters and rebuilds the result in a `garray_T`.
//! Each is entered from [`super::filter_map`] and calls back into
//! [`super::filter_map_one`].
//!
//! Every pointer these four touch arrives already wrapped by the safe layer
//! in [`crate::eval::list`], which is what lets the file forbid
//! `unsafe` outright: the walks themselves are index arithmetic and a lock
//! discipline, and that is the part worth reading.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;
use core::ptr;

use super::{FilterMap, filter_map_one};
use crate::eval::list::{
    Blob, Dict, List, TvRef, UNKNOWN_TV, char_len, check_fixed, check_lock, check_ro, clear_tv,
    clear_vim_var, err, list_alloc_ret, number_arm, set_key_nr, set_key_string, set_key_type,
    string_bytes, string_tv,
};
use crate::garray::Gap;
use crate::main::{did_emsg, e_invalblob, e_string_required};
use crate::types::{
    VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_LOCKED, VAR_NUMBER, VAR_STRING, VAR_UNLOCKED, Vv,
    garray_T, typval_T, typval_vval_union, varnumber_T,
};

/// A byte-item `garray_T`, the way `ga_init(&ga, sizeof(char), 80)` leaves
/// one: the String walk's output buffer.
const BYTE_GARRAY: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 1,
    ga_growsize: 80,
    ga_data: ptr::null_mut(),
};

/// `filter()`/`map()`/`mapnew()`/`foreach()` over a Dict.
///
/// The dict is locked and its hashtab pinned for the walk, so a callback
/// that tries to add a key gets `E741` instead of rehashing the array being
/// stepped over.  `filter()`'s own removal only leaves a tombstone in a slot
/// the cursor has already passed.
pub(crate) fn filter_map_dict(
    d: Dict,
    filtermap: FilterMap,
    arg_errmsg: &CStr,
    expr: &mut typval_T,
    rettv: &mut typval_T,
) {
    if filtermap == FilterMap::MapNew {
        rettv.v_type = VAR_DICT;
        rettv.vval.v_dict = ptr::null_mut();
    }
    if d.is_null() || (filtermap == FilterMap::Filter && check_lock(d.lock(), arg_errmsg)) {
        return;
    }

    let d_ret = (filtermap == FilterMap::MapNew).then(|| Dict::alloc_ret(rettv));

    let prev_lock = d.lock();
    if prev_lock == VAR_UNLOCKED {
        d.set_lock(VAR_LOCKED);
    }
    d.hash_lock();
    for di in d.items() {
        if filtermap == FilterMap::Map
            && (check_lock(di.lock(), arg_errmsg) || check_ro(di.flags(), arg_errmsg))
        {
            break;
        }
        set_key_string(di.key());
        let mut newtv = UNKNOWN_TV;
        let mut rem = false;
        let ok = filter_map_one(di.tv(), expr, filtermap, &mut newtv, &mut rem);
        clear_vim_var(Vv::Key);
        if !ok || did_emsg.get() != 0 {
            clear_tv(&mut newtv);
            break;
        }
        match filtermap {
            // map(): replace the dict item value.
            FilterMap::Map => di.set_tv(newtv),
            // mapnew(): add the item value to the new dict.
            FilterMap::MapNew => {
                let added = d_ret
                    .expect("mapnew allocated one")
                    .add_tv(di.key(), &mut newtv);
                clear_tv(&mut newtv);
                if !added {
                    break;
                }
            }
            // filter(false): remove the item from the dict.
            FilterMap::Filter if rem => {
                if check_fixed(di.flags(), arg_errmsg) || check_ro(di.flags(), arg_errmsg) {
                    break;
                }
                d.remove_item(di);
            }
            FilterMap::Filter | FilterMap::Foreach => {}
        }
    }
    d.hash_unlock();
    d.set_lock(prev_lock);
}

/// `filter()`/`map()`/`mapnew()`/`foreach()` over a Blob.
///
/// Bytes are rewritten in place -- into the copy, for `mapnew()` -- and
/// `filter()`'s removal closes the gap and steps the cursor back, so the
/// byte that moved down is the next one visited.  `v:key` keeps counting up
/// regardless, which is why there are two counters.
pub(crate) fn filter_map_blob(
    blob_arg: Blob,
    filtermap: FilterMap,
    arg_errmsg: &CStr,
    expr: &mut typval_T,
    rettv: &mut typval_T,
) {
    if filtermap == FilterMap::MapNew {
        rettv.v_type = VAR_BLOB;
        rettv.vval.v_blob = ptr::null_mut();
    }
    let b = blob_arg;
    if b.is_null() || (filtermap == FilterMap::Filter && check_lock(b.lock(), arg_errmsg)) {
        return;
    }

    let b_ret = if filtermap == FilterMap::MapNew {
        b.copy_to(rettv)
    } else {
        b
    };

    // set_vim_var_nr() doesn't set the type.
    set_key_type(VAR_NUMBER);

    let prev_lock = b.lock();
    if prev_lock == VAR_UNLOCKED {
        b.set_lock(VAR_LOCKED);
    }

    let mut i = 0;
    let mut idx = 0;
    while i < b.len() {
        let val = varnumber_T::from(b.byte(i));
        let mut tv = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: val },
        };
        set_key_nr(idx);
        let mut newtv = UNKNOWN_TV;
        let mut rem = false;
        if !filter_map_one(TvRef::of(&mut tv), expr, filtermap, &mut newtv, &mut rem)
            || did_emsg.get() != 0
        {
            break;
        }
        if filtermap != FilterMap::Foreach {
            if newtv.v_type != VAR_NUMBER && newtv.v_type != VAR_BOOL {
                clear_tv(&mut newtv);
                err(&e_invalblob);
                break;
            }
            if filtermap != FilterMap::Filter {
                let byte = number_arm(&newtv);
                if byte != val {
                    // A blob element takes the number's low byte, as
                    // upstream's `(uint8_t)` cast does.
                    b_ret.set_byte(i, byte.cast_unsigned().to_le_bytes()[0]);
                }
            } else if rem {
                b.remove_byte(i);
                i -= 1;
            }
        }
        idx += 1;
        i += 1;
    }

    b.set_lock(prev_lock);
}

/// `filter()`/`map()`/`mapnew()`/`foreach()` over a String.
///
/// A String is not a container that can be changed in place, so all four
/// forms build a fresh one: `map()`/`mapnew()` concatenate what the callback
/// answered -- which has to be a String -- and `filter()`/`foreach()`
/// concatenate the character itself, unless `filter()` dropped it.
pub(crate) fn filter_map_string(
    s: &[u8],
    filtermap: FilterMap,
    expr: &mut typval_T,
    rettv: &mut typval_T,
) {
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();

    // set_vim_var_nr() doesn't set the type.
    set_key_type(VAR_NUMBER);

    let mut ga = BYTE_GARRAY;
    let mut idx = 0;
    let mut at = 0;
    while at < s.len() {
        // `utfc_ptr2len` stops at the terminator, so the character always
        // fits; the clamp is what makes that a fact rather than a promise.
        let len = char_len(&s[at..]).min(s.len() - at);
        let ch = &s[at..at + len];
        let mut tv = string_tv(ch);

        set_key_nr(idx);
        let mut newtv = UNKNOWN_TV;
        let mut rem = false;
        if !filter_map_one(TvRef::of(&mut tv), expr, filtermap, &mut newtv, &mut rem)
            || did_emsg.get() != 0
        {
            clear_tv(&mut newtv);
            clear_tv(&mut tv);
            break;
        }
        if matches!(filtermap, FilterMap::Map | FilterMap::MapNew) {
            if newtv.v_type != VAR_STRING {
                clear_tv(&mut newtv);
                clear_tv(&mut tv);
                err(&e_string_required);
                break;
            }
            Gap(&mut ga).concat(string_bytes(&newtv));
        } else if filtermap == FilterMap::Foreach || !rem {
            Gap(&mut ga).concat(ch);
        }

        clear_tv(&mut newtv);
        clear_tv(&mut tv);

        idx += 1;
        at += len;
    }
    Gap(&mut ga).append(0);
    rettv.vval.v_string = ga.ga_data.cast();
}

/// `filter()`/`map()`/`mapnew()`/`foreach()` over a List.
///
/// The list is locked for the walk, so the only thing that can shorten it is
/// `filter()` itself -- and it does so through `tv_list_item_remove`, which
/// hands back the item that followed and fixes up any watcher parked on the
/// one it freed.  Every other step asks the *current* item for its
/// successor, after the callback has returned; a successor remembered from
/// before the call would be a freed pointer.
pub(crate) fn filter_map_list(
    l: List,
    filtermap: FilterMap,
    arg_errmsg: &CStr,
    expr: &mut typval_T,
    rettv: &mut typval_T,
) {
    if filtermap == FilterMap::MapNew {
        rettv.v_type = VAR_LIST;
        rettv.vval.v_list = ptr::null_mut();
    }
    if l.is_null() || (filtermap == FilterMap::Filter && check_lock(l.locked(), arg_errmsg)) {
        return;
    }

    let l_ret = (filtermap == FilterMap::MapNew).then(|| list_alloc_ret(rettv));

    // set_vim_var_nr() doesn't set the type.
    set_key_type(VAR_NUMBER);

    let prev_lock = l.locked();
    if prev_lock == VAR_UNLOCKED {
        l.set_lock(VAR_LOCKED);
    }

    let mut idx = 0;
    let mut cur = l.first();
    while let Some(li) = cur {
        if filtermap == FilterMap::Map && check_lock(li.lock(), arg_errmsg) {
            break;
        }
        set_key_nr(idx);
        let mut newtv = UNKNOWN_TV;
        let mut rem = false;
        if !filter_map_one(li.tv(), expr, filtermap, &mut newtv, &mut rem) {
            break;
        }
        if did_emsg.get() != 0 {
            clear_tv(&mut newtv);
            break;
        }
        match filtermap {
            // map(): replace the list item value.
            FilterMap::Map => li.set_tv(newtv),
            // mapnew(): append the list item value.
            FilterMap::MapNew => l_ret.expect("mapnew allocated one").append_owned(newtv),
            FilterMap::Filter | FilterMap::Foreach => {}
        }
        cur = if filtermap == FilterMap::Filter && rem {
            l.remove_item(li)
        } else {
            li.next()
        };
        idx += 1;
    }

    l.set_lock(prev_lock);
}
