//! Removing marks, and the decoration provider bridge.
//!
//! `nvim_buf_del_extmark` takes one mark and `nvim_buf_clear_namespace` a
//! range's worth.  `nvim_set_decoration_provider` is the other half of the
//! family: instead of marks stored in the buffer, a set of `LuaRef` callbacks
//! the redraw loop asks for decorations per window, line and buffer.
//! `parse_virt_text` is shared with it -- the `[[text, hl], ..]` chunk array
//! decoder every virtual-text entry point uses.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_expected, err_out_of_range};
use crate::kvec::Kvec;

pub fn nvim_buf_del_extmark(
    buf: BufferHandle,
    ns_id: Integer,
    id: Integer,
) -> Result<Boolean, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(false);
    };
    if !ns_initialized(ns_id as uint32_t) {
        error = err_bad_number(c"ns_id", ns_id);
        return false.reported(error);
    }
    extmark_del_id(b, ns_id as uint32_t, id as uint32_t).reported(error)
}

pub fn nvim_buf_clear_namespace(
    buf: BufferHandle,
    ns_id: Integer,
    line_start: Integer,
    mut line_end: Integer,
) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    if !(line_start >= 0 as Integer && line_start < MAXLNUM as Integer) {
        error = err_out_of_range(c"line number");
        return ().reported(error);
    }
    if line_end < 0 as Integer || line_end > MAXLNUM as Integer {
        line_end = MAXLNUM as Integer;
    }
    let ns = if ns_id < 0 as Integer {
        0 as uint32_t
    } else {
        ns_id as uint32_t
    };
    let start = line_start as ::core::ffi::c_int;
    let end = line_end as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    let maxcol = MAXCOL as ::core::ffi::c_int;
    extmark_clear(b, ns, start, 0 as ColNr, end, maxcol);
    ().reported(error)
}

/// # Safety
///
/// `opts` must point at the `KeyDict_set_decoration_provider` the dispatcher
/// filled in, live for the call.
pub unsafe fn nvim_set_decoration_provider(
    ns_id: Integer,
    opts: *mut KeyDict_set_decoration_provider,
) {
    let p: *mut DecorProvider = get_decor_provider(ns_id as NS, true);
    debug_assert!(!p.is_null(), "p != NULL");
    unsafe { decor_provider_clear(p) };
    redraw_all_later(UPD_NOT_VALID);
    // Each callback the caller named moves into the provider, and the
    // keyset gives up its reference so that the release walk does not free
    // what the provider now holds.
    // SAFETY: `opts` is the caller's keyset and `p` the provider just
    // cleared; both are live for the call.
    unsafe {
        let callbacks: [(&mut Option<LuaRef>, &mut LuaRef); 9] = [
            (&mut (*opts).on_start, &mut (*p).redraw_start),
            (&mut (*opts).on_buf, &mut (*p).redraw_buf),
            (&mut (*opts).on_win, &mut (*p).redraw_win),
            (&mut (*opts).on_line, &mut (*p).redraw_line),
            (&mut (*opts).on_range, &mut (*p).redraw_range),
            (&mut (*opts).on_end, &mut (*p).redraw_end),
            (&mut (*opts)._on_hl_def, &mut (*p).hl_def),
            (&mut (*opts)._on_spell_nav, &mut (*p).spell_nav),
            (&mut (*opts)._on_conceal_line, &mut (*p).conceal_line),
        ];
        for (source, dest) in callbacks {
            if source.is_some_and(|reference| reference > 0) {
                *dest = source.take().expect("just tested");
            }
        }
    }
    unsafe { (*p).state = kDecorProviderActive };
    unsafe { (*p).hl_valid += 1 };
    unsafe { (*p).hl_cached = false };
}

/// # Safety
///
/// `chunks` must be a well-formed API array, its `size` elements initialized.
/// `width` must point at a writable `int` the caller owns.
pub unsafe fn parse_virt_text(
    chunks: &Array,
    width: *mut ::core::ffi::c_int,
) -> Result<VirtText, Error> {
    let mut virt_text: VirtText = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    // The refusal is held rather than returned: the half-built text has to
    // be released on the way out.
    let failed;
    '_free_exit: {
        while i < chunks.len() {
            // SAFETY: `i` is below `chunks.size`.
            let Some(chunk) = chunks[i].as_array() else {
                let want = api_typename(kObjectTypeArray);
                // SAFETY: as above.
                let got = api_typename((chunks[i]).kind());
                failed = err_expected(c"chunk", want, Some(got));
                break '_free_exit;
            };
            let head = match chunk.len() {
                // SAFETY: a non-empty array names its first item.
                1..=2 => (chunk[0]).as_string(),
                _ => None,
            };
            let Some(str) = head else {
                let why = c"Invalid chunk: expected Array with 1 or 2 Strings";
                failed = Error::validation(why);
                break '_free_exit;
            };
            let mut hl_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            's_146: {
                if chunk.len() == 2 as size_t {
                    let hl = &chunk[1];
                    if let Some(arr) = hl.as_array() {
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= arr.len() {
                                break 's_146;
                            }
                            let item = &arr[j];
                            let what = c"virt_text highlight".as_ptr();
                            // SAFETY: `what` is a NUL-terminated literal.
                            hl_id = match unsafe { object_to_hl_id(item, what) } {
                                Ok(id) => id,
                                Err(e) => {
                                    failed = e;
                                    break '_free_exit;
                                }
                            };
                            if j < arr.len().wrapping_sub(1 as size_t) {
                                // `kv_push`, whose growth step c2rust expanded inline.
                                let mut vt = Kvec::new(
                                    &mut virt_text.size,
                                    &mut virt_text.capacity,
                                    &mut virt_text.items,
                                );
                                let text = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                // SAFETY: `items` is this vector's own allocation.
                                unsafe { vt.push(VirtTextChunk { text, hl_id }) };
                            }
                            j = j.wrapping_add(1);
                        }
                    } else {
                        let what = c"virt_text highlight".as_ptr();
                        // SAFETY: `hl` is the caller's object.
                        hl_id = match unsafe { object_to_hl_id(hl, what) } {
                            Ok(id) => id,
                            Err(e) => {
                                failed = e;
                                break '_free_exit;
                            }
                        };
                    }
                }
            }
            let src = if str.len() > 0 as size_t {
                str.data() as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            };
            // SAFETY: `src` is a C string -- the chunk's own bytes or a literal.
            let text: *mut ::core::ffi::c_char = unsafe { transstr(src, false) };
            w += unsafe { mb_string2cells(text) } as ::core::ffi::c_int;
            // `kv_push`, whose growth step c2rust expanded inline.
            let mut vt = Kvec::new(
                &mut virt_text.size,
                &mut virt_text.capacity,
                &mut virt_text.items,
            );
            // SAFETY: `items` is this vector's own allocation.
            unsafe { vt.push(VirtTextChunk { text, hl_id }) };
            i = i.wrapping_add(1);
        }
        if !width.is_null() {
            unsafe { *width = w };
        }
        return Ok(virt_text);
    }
    unsafe { clear_virttext(&raw mut virt_text) };
    Err(failed)
}
