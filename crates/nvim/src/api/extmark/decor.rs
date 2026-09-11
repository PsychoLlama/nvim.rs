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
    if !(line_start >= 0 && line_start < Integer::from(MAXLNUM)) {
        error = err_out_of_range(c"line number");
        return ().reported(error);
    }
    if line_end < 0 || line_end > Integer::from(MAXLNUM) {
        line_end = Integer::from(MAXLNUM);
    }
    // A negative namespace means every one of them.
    let ns = if ns_id < 0 { 0 } else { ns_id as uint32_t };
    let start = line_start as ::core::ffi::c_int;
    let end = line_end as ::core::ffi::c_int - 1;
    extmark_clear(b, ns, start, 0, end, MAXCOL);
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

/// The `[[text, hl], ..]` chunk array every virtual-text entry point decodes.
///
/// `width` takes the total display width of the chunks, for the callers that
/// need it to lay the text out.
pub fn parse_virt_text(
    chunks: &Array,
    width: Option<&mut ::core::ffi::c_int>,
) -> Result<VirtText, Error> {
    let mut virt_text = VirtText {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
    };
    let mut cells = 0;
    for chunk in chunks.iter() {
        if let Err(refused) = push_chunk(chunk, &mut virt_text, &mut cells) {
            // The half-built text has to be released on the way out.
            // SAFETY: this frame's own vector, which nothing else names.
            unsafe { clear_virttext(&raw mut virt_text) };
            return Err(refused);
        }
    }
    if let Some(width) = width {
        *width = cells;
    }
    Ok(virt_text)
}

/// Decode one `[text, hl]` chunk into `into`, growing `cells` by its width.
fn push_chunk(
    chunk: &Object,
    into: &mut VirtText,
    cells: &mut ::core::ffi::c_int,
) -> Result<(), Error> {
    let Some(chunk) = chunk.as_array() else {
        let want = api_typename(kObjectTypeArray);
        let got = api_typename(chunk.kind());
        return Err(err_expected(c"chunk", want, Some(got)));
    };
    let head = match chunk.len() {
        1..=2 => chunk[0].as_string(),
        _ => None,
    };
    let Some(str) = head else {
        let why = c"Invalid chunk: expected Array with 1 or 2 Strings";
        return Err(Error::validation(why));
    };
    let what = c"virt_text highlight".as_ptr();
    let mut hl_id = -1;
    if let Some(hl) = chunk.get(1) {
        match hl.as_array() {
            // A stack of groups: every one but the last becomes a chunk of
            // its own with no text, and the last is this chunk's.
            Some(groups) => {
                for (n, group) in groups.iter().enumerate() {
                    // SAFETY: `what` is a NUL-terminated literal.
                    hl_id = unsafe { object_to_hl_id(group, what) }?;
                    if n + 1 < groups.len() {
                        let text = ::core::ptr::null_mut();
                        push(into, VirtTextChunk { text, hl_id });
                    }
                }
            }
            // SAFETY: as above.
            None => hl_id = unsafe { object_to_hl_id(hl, what) }?,
        }
    }
    let src = if str.is_empty() {
        c"".as_ptr()
    } else {
        str.data().cast_const()
    };
    // SAFETY: `src` is a C string -- the chunk's own bytes or a literal.
    let text = unsafe { transstr(src, false) };
    // SAFETY: `transstr` answers a NUL-terminated allocation.
    *cells += unsafe { mb_string2cells(text) } as ::core::ffi::c_int;
    push(into, VirtTextChunk { text, hl_id });
    Ok(())
}

/// `kv_push`, whose growth step c2rust expanded inline.
fn push(into: &mut VirtText, chunk: VirtTextChunk) {
    let mut vt = Kvec::new(&mut into.size, &mut into.capacity, &mut into.items);
    // SAFETY: `items` is this vector's own allocation.
    unsafe { vt.push(chunk) };
}
