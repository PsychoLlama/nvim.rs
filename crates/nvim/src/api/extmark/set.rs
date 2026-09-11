//! `nvim_buf_set_extmark()`: placing a mark and its decoration.
//!
//! One function, and the largest in the api, because the keyset it takes is the
//! whole decoration surface: an id, an end position, a highlight (or a list of
//! them), a sign, virtual text with a position and a highlight mode, virtual
//! lines, conceal, spell, a url, a priority and the gravity of both ends.  Each
//! is validated, packed into the inline or allocated decoration representation
//! as its size allows, and handed to `extmark_set`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid, err_out_of_range,
};
use crate::decoration::DecorStateRef;
use crate::kvec::Kvec;
use crate::winlayer::Live;

/// The keyset this call was handed, with checked field access: the pointer the
/// dispatcher passes stays live for the whole call, so one promise at the head
/// buys every `opts.field` below.
type Opts = Live<KeyDict_set_extmark>;

/// # Safety
///
/// `opts` must point at the `KeyDict_set_extmark` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_buf_set_extmark(
    buf: BufferHandle,
    ns_id: Integer,
    mut line: Integer,
    mut col: Integer,
    opts: *mut KeyDict_set_extmark,
) -> Result<Integer, Error> {
    let mut error = Error::none();
    // SAFETY: the dispatcher's keyset outlives this call.
    let mut opts = unsafe { Opts::new(opts) };
    let mut id: uint32_t;
    let mut line2: ::core::ffi::c_int;
    let strict: bool;
    let mut col2: ColNr;
    let mut virt_lines_flags: ::core::ffi::c_int;
    let right_gravity: bool;
    let mut len: ColNr;
    let mut hl: DecorHighlightInline = DECOR_HIGHLIGHT_INLINE_INIT;
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: DecorVirtText = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: DecorVirtText_data::Text(VirtText {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        }),
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut virt_lines: DecorVirtText = DecorVirtText {
        flags: kVTIsLines as ::core::ffi::c_int as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: DecorVirtText_data::Lines(VirtLines {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<virt_line>(),
        }),
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut url: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut has_hl: bool = false;
    let mut has_hl_multiple: bool = false;
    let b = find_buffer_by_handle(buf)?;
    '_error: {
        if let Some(b) = b {
            if !ns_initialized(ns_id as uint32_t) {
                error = err_bad_number(c"ns_id", ns_id);
            } else {
                id = 0 as uint32_t;
                if let Some(given) = opts.id {
                    if !(given > 0 as Integer) {
                        let want = c"positive Integer";
                        error = err_expected(c"id", want, None);
                        break '_error;
                    }
                    id = given as uint32_t;
                }
                line2 = -1 as ::core::ffi::c_int;
                if let Some(end_line) = opts.end_line {
                    if opts.end_row.is_some() {
                        let why = c"cannot use both 'end_row' and 'end_line'";
                        error = Error::validation(why);
                        break '_error;
                    }
                    opts.end_row = Some(end_line);
                }
                strict = opts.strict.unwrap_or(true);
                // `end_line` wrote `end_row` above, so one test covers both.
                if let Some(val) = opts.end_row {
                    if !(val >= 0 as Integer
                        && !(val > b.line_count() as Integer && strict as ::core::ffi::c_int != 0))
                    {
                        error = err_out_of_range(c"end_row");
                        break '_error;
                    }
                    line2 = val as ::core::ffi::c_int;
                }
                col2 = -1 as ColNr;
                if let Some(mut val_0) = opts.end_col {
                    if !(val_0 >= -1 as Integer && val_0 <= MAXCOL as ::core::ffi::c_int as Integer)
                    {
                        error = err_out_of_range(c"end_col");
                        break '_error;
                    }
                    if val_0 == -1 as Integer {
                        val_0 = MAXCOL as ::core::ffi::c_int as Integer;
                    }
                    col2 = val_0 as ::core::ffi::c_int as ColNr;
                }
                if let Some(given_hl_group) = opts.hl_group.as_ref() {
                    's_293: {
                        if let Some(arr) = given_hl_group.as_array() {
                            if arr.len() >= 1 as size_t {
                                hl.hl_id = match unsafe {
                                    object_to_hl_id(&arr[0], c"hl_group item".as_ptr())
                                } {
                                    Ok(id) => id,
                                    Err(e) => {
                                        error = e;
                                        break '_error;
                                    }
                                };
                            }
                            let mut i: size_t = 1 as size_t;
                            loop {
                                if i >= arr.len() {
                                    break 's_293;
                                }
                                let hl_id: ::core::ffi::c_int = match unsafe {
                                    object_to_hl_id(&arr[i], c"hl_group item".as_ptr())
                                } {
                                    Ok(id) => id,
                                    Err(e) => {
                                        error = e;
                                        break '_error;
                                    }
                                };
                                if hl_id != 0 {
                                    has_hl_multiple = true;
                                }
                                i = i.wrapping_add(1);
                            }
                        } else {
                            hl.hl_id = match unsafe {
                                object_to_hl_id(&given_hl_group, c"hl_group".as_ptr())
                            } {
                                Ok(id) => id,
                                Err(e) => {
                                    error = e;
                                    break '_error;
                                }
                            };
                        }
                    }
                    has_hl = hl.hl_id > 0 as ::core::ffi::c_int;
                }
                sign.hl_id = opts.sign_hl_group.unwrap_or(0) as ::core::ffi::c_int;
                sign.cursorline_hl_id = opts.cursorline_hl_group.unwrap_or(0) as ::core::ffi::c_int;
                sign.number_hl_id = opts.number_hl_group.unwrap_or(0) as ::core::ffi::c_int;
                sign.line_hl_id = opts.line_hl_group.unwrap_or(0) as ::core::ffi::c_int;
                if sign.hl_id != 0
                    || sign.cursorline_hl_id != 0
                    || sign.number_hl_id != 0
                    || sign.line_hl_id != 0
                {
                    sign.flags = (sign.flags as ::core::ffi::c_int
                        | kSHIsSign as ::core::ffi::c_int)
                        as uint16_t;
                }
                if let Some(conceal) = opts.conceal.as_ref() {
                    hl.flags = (hl.flags as ::core::ffi::c_int | kSHConceal as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true;
                    if conceal.len() > 0 as size_t {
                        let mut ch: ::core::ffi::c_int = 0;
                        hl.conceal_char = unsafe { utfc_ptr2schar(conceal.data(), &raw mut ch) };
                        if !(hl.conceal_char != 0 && vim_isprintc(ch) as ::core::ffi::c_int != 0) {
                            let why = c"conceal char has to be printable";
                            error = Error::validation(why);
                            break '_error;
                        }
                    }
                }
                if let Some(conceal_lines) = opts.conceal_lines.as_ref() {
                    hl.flags = (hl.flags as ::core::ffi::c_int
                        | kSHConcealLines as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true;
                    if conceal_lines.len() > 0 as size_t
                        && !(unsafe { *conceal_lines.data() } as ::core::ffi::c_int
                            == '\0' as ::core::ffi::c_int)
                    {
                        let why = c"conceal_lines has to be an empty string";
                        error = Error::validation(why);
                        break '_error;
                    }
                }
                if let Some(given) = opts.virt_text.as_ref() {
                    let width = &raw mut virt_text.width;
                    match unsafe { parse_virt_text(&given, width) } {
                        Ok(text) => *virt_text.data.text_mut() = text,
                        Err(e) => {
                            error = e;
                            break '_error;
                        }
                    }
                }
                if let Some(str) = opts.virt_text_pos.as_ref() {
                    if unsafe { strequal(c"eol".as_ptr(), str.data()) } {
                        virt_text.pos = kVPosEndOfLine;
                    } else if unsafe { strequal(c"overlay".as_ptr(), str.data()) } {
                        virt_text.pos = kVPosOverlay;
                    } else if unsafe { strequal(c"right_align".as_ptr(), str.data()) } {
                        virt_text.pos = kVPosRightAlign;
                    } else if unsafe { strequal(c"eol_right_align".as_ptr(), str.data()) } {
                        virt_text.pos = kVPosEndOfLineRightAlign;
                    } else if unsafe { strequal(c"inline".as_ptr(), str.data()) } {
                        virt_text.pos = kVPosInline;
                    } else if true {
                        // SAFETY: the value the keyset carried, live for this call.
                        error = err_bad_value(c"virt_text_pos", str.as_cstr());
                        break '_error;
                    }
                }
                if let Some(win_col) = opts.virt_text_win_col {
                    virt_text.col = win_col as ::core::ffi::c_int;
                    virt_text.pos = kVPosWinCol;
                }
                hl.flags = (hl.flags as ::core::ffi::c_int
                    | if opts.hl_eol.unwrap_or(false) {
                        kSHHlEol as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint16_t;
                virt_text.flags = (virt_text.flags as ::core::ffi::c_int
                    | ((if opts.virt_text_hide.unwrap_or(false) {
                        kVTHide as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if opts.virt_text_repeat_linebreak.unwrap_or(false) {
                        kVTRepeatLinebreak as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }))) as uint8_t;
                if let Some(str_0) = opts.hl_mode.as_ref() {
                    if unsafe { strequal(c"replace".as_ptr(), str_0.data()) } {
                        virt_text.hl_mode = kHlModeReplace as ::core::ffi::c_int as uint8_t;
                    } else if unsafe { strequal(c"combine".as_ptr(), str_0.data()) } {
                        virt_text.hl_mode = kHlModeCombine as ::core::ffi::c_int as uint8_t;
                    } else if unsafe { strequal(c"blend".as_ptr(), str_0.data()) } {
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                            && true
                        {
                            let why = c"cannot use 'blend' hl_mode with inline virtual text";
                            error = Error::validation(why);
                            break '_error;
                        }
                        virt_text.hl_mode = kHlModeBlend as ::core::ffi::c_int as uint8_t;
                    } else if true {
                        // SAFETY: the value the keyset carried, live for this call.
                        error = err_bad_value(c"hl_mode", str_0.as_cstr());
                        break '_error;
                    }
                }
                virt_lines_flags = if opts.virt_lines_leftcol.unwrap_or(false) {
                    kVLLeftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if let Some(str_1) = opts.virt_lines_overflow.as_ref() {
                    if unsafe { strequal(c"scroll".as_ptr(), str_1.data()) } {
                        virt_lines_flags |= kVLScroll as ::core::ffi::c_int;
                    } else if !unsafe { strequal(c"trunc".as_ptr(), str_1.data()) } && true {
                        // SAFETY: the value the keyset carried, live for this call.
                        error = err_bad_value(c"virt_lines_overflow", str_1.as_cstr());
                        break '_error;
                    }
                }
                's_785: {
                    if let Some(a) = opts.virt_lines.as_ref() {
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= a.len() {
                                break 's_785;
                            }
                            // SAFETY: the pointer the caller handed this call.
                            let item = &a[j];
                            let Object::Array(item) = item else {
                                let want = api_typename(kObjectTypeArray);
                                let got = api_typename(item.kind());
                                error = err_expected(c"virt_text_line", want, Some(got));
                                break '_error;
                            };
                            let mut dummig: ::core::ffi::c_int = 0;
                            let dummy_width = &raw mut dummig;
                            // SAFETY: the array the caller's item names.
                            let jtem: VirtText =
                                match unsafe { parse_virt_text(&item, dummy_width) } {
                                    Ok(jtem) => jtem,
                                    Err(e) => {
                                        error = e;
                                        break '_error;
                                    }
                                };
                            // `kv_push`, whose growth step c2rust expanded inline.
                            let lines = virt_lines.data.lines_mut();
                            let mut vl =
                                Kvec::new(&mut lines.size, &mut lines.capacity, &mut lines.items);
                            let line = virt_line {
                                line: jtem,
                                flags: virt_lines_flags,
                            };
                            // SAFETY: `items` is this vector's own allocation.
                            unsafe { vl.push(line) };
                            if error.is_set() {
                                break '_error;
                            }
                            j = j.wrapping_add(1);
                        }
                    }
                }
                virt_lines.flags = (virt_lines.flags as ::core::ffi::c_int
                    | if opts.virt_lines_above.unwrap_or(false) {
                        kVTLinesAbove as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint8_t;
                if let Some(priority) = opts.priority {
                    if !(priority >= 0 as Integer && priority <= 65535 as Integer) {
                        error = err_out_of_range(c"priority");
                        break '_error;
                    }
                    hl.priority = priority as DecorPriority;
                    sign.priority = priority as DecorPriority;
                    virt_text.priority = priority as DecorPriority;
                    virt_lines.priority = priority as DecorPriority;
                }
                if let Some(sign_text) = opts.sign_text.as_ref() {
                    sign.text[0 as ::core::ffi::c_int as usize] = 0 as ScreenChar;
                    if unsafe {
                        init_sign_text(
                            sign_text.data(),
                            &raw mut sign.text as *mut ScreenChar,
                            false,
                        )
                    }
                    .is_err()
                    {
                        error = err_invalid(c"sign_text", Bad::Unsaid);
                        break '_error;
                    }
                    sign.flags = (sign.flags as ::core::ffi::c_int
                        | kSHIsSign as ::core::ffi::c_int)
                        as uint16_t;
                }
                right_gravity = opts.right_gravity.unwrap_or(true);
                if line2 == -1 as ::core::ffi::c_int
                    && col2 == -1 as ::core::ffi::c_int
                    && opts.end_right_gravity.is_some()
                {
                    let why = c"cannot set end_right_gravity without end_row or end_col";
                    error = Error::validation(why);
                } else {
                    len = 0 as ColNr;
                    if let Some(spell) = opts.spell {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | if spell {
                                kSHSpellOn as ::core::ffi::c_int
                            } else {
                                kSHSpellOff as ::core::ffi::c_int
                            }) as uint16_t;
                        has_hl = true;
                    }
                    if let Some(given) = opts.url.as_ref() {
                        url = string_to_cstr(&given);
                        has_hl = true;
                    }
                    if opts.ui_watched.unwrap_or(false) {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | kSHUIWatched as ::core::ffi::c_int)
                            as uint16_t;
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            hl.flags = (hl.flags as ::core::ffi::c_int
                                | kSHUIWatchedOverlay as ::core::ffi::c_int)
                                as uint16_t;
                        }
                        has_hl = true;
                    }
                    if !(line >= 0 as Integer) {
                        error = err_out_of_range(c"line");
                    } else {
                        if line > b.line_count() as Integer {
                            if strict {
                                error = err_out_of_range(c"line");
                                break '_error;
                            }
                            line = b.line_count() as Integer;
                        } else if line < b.line_count() as Integer {
                            len = (if opts.ephemeral.unwrap_or(false) {
                                MAXCOL as ::core::ffi::c_int
                            } else {
                                unsafe { b.line_len(line as LineNr + 1) }
                            }) as ColNr;
                        }
                        if col == -1 as Integer {
                            col = len as Integer;
                        } else if col > len as Integer {
                            if strict {
                                error = err_out_of_range(c"col");
                                break '_error;
                            }
                            col = len as Integer;
                        } else if col < -1 as Integer && true {
                            error = err_out_of_range(c"col");
                            break '_error;
                        }
                        if col2 >= 0 as ::core::ffi::c_int {
                            if line2 >= 0 as ::core::ffi::c_int
                                && (line2 as LineNr) < b.line_count()
                            {
                                len = (if opts.ephemeral.unwrap_or(false) {
                                    MAXCOL as ::core::ffi::c_int
                                } else {
                                    unsafe { b.line_len(line2 as LineNr + 1) }
                                }) as ColNr;
                            } else if line2 as LineNr == b.line_count() {
                                len = 0 as ::core::ffi::c_int as ColNr;
                            } else {
                                line2 = line as ::core::ffi::c_int;
                            }
                            if col2 > len {
                                if strict {
                                    error = err_out_of_range(c"end_col");
                                    break '_error;
                                }
                                col2 = len;
                            }
                        } else if line2 >= 0 as ::core::ffi::c_int {
                            col2 = 0 as ::core::ffi::c_int as ColNr;
                        }
                        if opts.ephemeral.unwrap_or(false)
                            && !unsafe { DecorStateRef::current() }.win.is_null()
                            && unsafe { (*DecorStateRef::current().win).w_buffer } == b.raw()
                        {
                            let r: ::core::ffi::c_int = line as ::core::ffi::c_int;
                            let c: ::core::ffi::c_int = col as ::core::ffi::c_int;
                            if line2 == -1 as ::core::ffi::c_int {
                                line2 = r;
                                col2 = c as ColNr;
                            }
                            let mut subpriority: DecorPriority = 0 as DecorPriority;
                            if let Some(given) = opts._subpriority {
                                if !(given >= 0 as Integer && given <= 65535 as Integer) {
                                    error = err_out_of_range(c"_subpriority");
                                    break '_error;
                                }
                                subpriority = given as DecorPriority;
                            }
                            if virt_text.data.text().size != 0 {
                                // SAFETY: inside a decoration provider, so the
                                // redraw's decor state is set.
                                let state = unsafe { DecorStateRef::current() };
                                let c2 = col2 as ::core::ffi::c_int;
                                let vt = decor_put_vt(virt_text, ::core::ptr::null_mut());
                                // SAFETY: `state` is the redraw's own decor state.
                                unsafe { decor_range_add_virt(state, r, c, line2, c2, vt, true) };
                            }
                            if virt_lines.data.lines().size != 0 {
                                // SAFETY: inside a decoration provider, so the
                                // redraw's decor state is set.
                                let state = unsafe { DecorStateRef::current() };
                                let c2 = col2 as ::core::ffi::c_int;
                                let vt = decor_put_vt(virt_lines, ::core::ptr::null_mut());
                                // SAFETY: `state` is the redraw's own decor state.
                                unsafe { decor_range_add_virt(state, r, c, line2, c2, vt, true) };
                            }
                            if has_hl {
                                let mut sh: DecorSignHighlight = decor_sh_from_inline(hl);
                                sh.url = url;
                                // SAFETY: inside a decoration provider, so the
                                // redraw's decor state is set.
                                let state = unsafe { DecorStateRef::current() };
                                let c2 = col2 as ::core::ffi::c_int;
                                let shp = &raw mut sh;
                                let ns = ns_id as uint32_t;
                                // SAFETY: `state` is the redraw's own decor state
                                // and `shp` this frame's highlight.
                                unsafe {
                                    decor_range_add_sh(
                                        state,
                                        r,
                                        c,
                                        line2,
                                        c2,
                                        shp,
                                        true,
                                        ns,
                                        id,
                                        subpriority,
                                    )
                                };
                            }
                        } else if opts.ephemeral.unwrap_or(false) {
                            let why =
                                c"cannot set emphemeral mark outside of a decoration provider";
                            error = Error::exception(why);
                            break '_error;
                        } else {
                            let mut decor_flags = MtFlags::NONE;
                            let mut decor_alloc: *mut DecorVirtText =
                                ::core::ptr::null_mut::<DecorVirtText>();
                            if virt_text.data.text().size != 0 {
                                decor_alloc = decor_put_vt(virt_text, decor_alloc);
                                if virt_text.pos as ::core::ffi::c_uint
                                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    decor_flags |= MtFlags::DECOR_VIRT_TEXT_INLINE;
                                }
                            }
                            if virt_lines.data.lines().size != 0 {
                                decor_alloc = decor_put_vt(virt_lines, decor_alloc);
                                decor_flags |= MtFlags::DECOR_VIRT_LINES;
                            }
                            let mut decor_indexed: uint32_t = DECOR_ID_INVALID as uint32_t;
                            if sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int
                                != 0
                            {
                                sign.next = decor_indexed;
                                decor_indexed = decor_put_sh(sign);
                                if sign.text[0 as ::core::ffi::c_int as usize] != 0 {
                                    decor_flags |= MtFlags::DECOR_SIGNTEXT;
                                }
                                if sign.number_hl_id != 0
                                    || sign.line_hl_id != 0
                                    || sign.cursorline_hl_id != 0
                                {
                                    decor_flags |= MtFlags::DECOR_SIGNHL;
                                }
                            }
                            if has_hl_multiple {
                                let Some(arr_0) = opts.hl_group.as_ref().and_then(Object::as_array)
                                else {
                                    unreachable!("`has_hl_multiple` is set only under an Array")
                                };
                                let mut i_0: size_t = arr_0.len().wrapping_sub(1 as size_t);
                                while i_0 > 0 as size_t {
                                    // The same objects resolved above, so a
                                    // refusal here is impossible; zero is the
                                    // id an unresolvable name would have got.
                                    let hl_id_0: ::core::ffi::c_int = unsafe {
                                        object_to_hl_id(&arr_0[i_0], c"hl_group item".as_ptr())
                                    }
                                    .unwrap_or(0);
                                    if hl_id_0 > 0 as ::core::ffi::c_int {
                                        let mut sh_0: DecorSignHighlight =
                                            DECOR_SIGN_HIGHLIGHT_INIT;
                                        sh_0.hl_id = hl_id_0;
                                        sh_0.flags = (if opts.hl_eol.unwrap_or(false) {
                                            kSHHlEol as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        })
                                            as uint16_t;
                                        sh_0.next = decor_indexed;
                                        decor_indexed = decor_put_sh(sh_0);
                                        decor_flags |= MtFlags::DECOR_HL;
                                    }
                                    i_0 = i_0.wrapping_sub(1);
                                }
                            }
                            if hl.flags as ::core::ffi::c_int
                                & kSHConcealLines as ::core::ffi::c_int
                                != 0
                            {
                                decor_flags |= MtFlags::DECOR_CONCEAL_LINES;
                            }
                            let mut decor: DecorInline = DECOR_INLINE_INIT;
                            if !decor_alloc.is_null()
                                || decor_indexed != DECOR_ID_INVALID as uint32_t
                                || !url.is_null()
                                || schar_high(hl.conceal_char) as ::core::ffi::c_int != 0
                            {
                                if has_hl {
                                    let mut sh_1: DecorSignHighlight = decor_sh_from_inline(hl);
                                    sh_1.url = url;
                                    sh_1.next = decor_indexed;
                                    decor_indexed = decor_put_sh(sh_1);
                                }
                                decor.ext = true;
                                decor.data.ext = DecorExt {
                                    sh_idx: decor_indexed,
                                    vt: decor_alloc,
                                };
                            } else {
                                decor.data.hl = hl;
                            }
                            if has_hl {
                                decor_flags |= MtFlags::DECOR_HL;
                            }
                            unsafe {
                                extmark_set(
                                    b,
                                    ns_id as uint32_t,
                                    &raw mut id,
                                    line as ::core::ffi::c_int,
                                    col as ColNr,
                                    line2,
                                    col2,
                                    decor,
                                    decor_flags,
                                    right_gravity,
                                    opts.end_right_gravity.unwrap_or(false),
                                    !opts.undo_restore.unwrap_or(true),
                                    opts.invalidate.unwrap_or(false),
                                )
                            };
                            if error.is_set() {
                                unsafe { decor_free(decor) };
                                return (0 as Integer).reported(error);
                            }
                        }
                        return (id as Integer).reported(error);
                    }
                }
            }
        }
    }
    unsafe { clear_virttext(&raw mut *virt_text.data.text_mut()) };
    unsafe { clear_virtlines(&raw mut *virt_lines.data.lines_mut()) };
    if !url.is_null() {
        unsafe { xfree(url as *mut ::core::ffi::c_void) };
    }
    (0 as Integer).reported(error)
}
