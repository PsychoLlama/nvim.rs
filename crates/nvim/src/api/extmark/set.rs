//! `nvim_buf_set_extmark()`: placing a mark and its decoration.
//!
//! One function, and the largest in the api, because the keyset it takes is the
//! whole decoration surface: an id, an end position, a highlight (or a list of
//! them), a sign, virtual text with a position and a highlight mode, virtual
//! lines, conceal, spell, a url, a priority and the gravity of both ends.  Each
//! is validated, packed into the inline or allocated decoration representation
//! as its size allows, and handed to `extmark_set`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid, err_out_of_range,
};
use crate::decoration::DecorStateRef;
use crate::kvec::Kvec;
use crate::winlayer::{Buf, Live};

/// The keyset this call was handed, with checked field access: the pointer the
/// dispatcher passes stays live for the whole call, so one promise at the head
/// buys every `opts.field` below.
type Opts = Live<KeyDict_set_extmark>;

pub unsafe fn nvim_buf_set_extmark(
    buf: Buffer,
    ns_id: Integer,
    mut line: Integer,
    mut col: Integer,
    opts: *mut KeyDict_set_extmark,
) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    // SAFETY: the dispatcher's keyset outlives this call.
    let mut opts = unsafe { Opts::new(opts) };
    let mut id: uint32_t = 0;
    let mut line2: ::core::ffi::c_int = 0;
    let mut did_end_line: bool = false;
    let mut strict: bool = false;
    let mut col2: colnr_T = 0;
    let mut virt_lines_flags: ::core::ffi::c_int = 0;
    let mut right_gravity: bool = false;
    let mut len: colnr_T = 0;
    let mut hl: DecorHighlightInline = DECOR_HIGHLIGHT_INLINE_INIT;
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: DecorVirtText = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: DecorVirtText_data {
            virt_text: VirtText {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut virt_lines: DecorVirtText = DecorVirtText {
        flags: kVTIsLines as ::core::ffi::c_int as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: DecorVirtText_data {
            virt_lines: VirtLines {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<virt_line>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut url: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut has_hl: bool = false;
    let mut has_hl_multiple: bool = false;
    let b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    '_error: {
        if !b.is_null() {
            // SAFETY: non-null, so the handle named a live buffer; nothing
            // below frees it.
            let b = unsafe { Buf::new(b) };
            if !ns_initialized(ns_id as uint32_t) {
                error = err_bad_number(c"ns_id", ns_id);
            } else {
                id = 0 as uint32_t;
                if has_key(opts.is_set__set_extmark_, KEYSET_OPTIDX_set_extmark__id) {
                    if !(opts.id > 0 as Integer) {
                        let want = c"positive Integer";
                        error = err_expected(c"id", want, None);
                        break '_error;
                    }
                    id = opts.id as uint32_t;
                }
                line2 = -1 as ::core::ffi::c_int;
                did_end_line = false;
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__end_line,
                ) {
                    if has_key(opts.is_set__set_extmark_, 10 as ::core::ffi::c_int) {
                        let why = c"cannot use both 'end_row' and 'end_line'";
                        error = Error::validation(why);
                        break '_error;
                    }
                    let end_line = opts.end_line;
                    opts.end_row = end_line;
                    did_end_line = true;
                }
                strict = if has_key(opts.is_set__set_extmark_, KEYSET_OPTIDX_set_extmark__strict) {
                    opts.strict as ::core::ffi::c_int
                } else {
                    1
                } != 0;
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__end_row,
                ) || did_end_line as ::core::ffi::c_int != 0
                {
                    let mut val: Integer = opts.end_row;
                    if !(val >= 0 as Integer
                        && !(val > b.line_count() as Integer && strict as ::core::ffi::c_int != 0))
                    {
                        error = err_out_of_range(c"end_row");
                        break '_error;
                    }
                    line2 = val as ::core::ffi::c_int;
                }
                col2 = -1 as colnr_T;
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__end_col,
                ) {
                    let mut val_0: Integer = opts.end_col;
                    if !(val_0 >= -1 as Integer && val_0 <= MAXCOL as ::core::ffi::c_int as Integer)
                    {
                        error = err_out_of_range(c"end_col");
                        break '_error;
                    }
                    if val_0 == -1 as Integer {
                        val_0 = MAXCOL as ::core::ffi::c_int as Integer;
                    }
                    col2 = val_0 as ::core::ffi::c_int as colnr_T;
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__hl_group,
                ) {
                    's_293: {
                        if opts.hl_group.type_0 as ::core::ffi::c_uint
                            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let mut arr: Array = unsafe { opts.hl_group.data.array };
                            if arr.size >= 1 as size_t {
                                hl.hl_id = unsafe {
                                    object_to_hl_id(
                                        *arr.items.offset(0 as ::core::ffi::c_int as isize),
                                        c"hl_group item".as_ptr(),
                                        &mut error,
                                    )
                                };
                                if error.is_set() {
                                    break '_error;
                                }
                            }
                            let mut i: size_t = 1 as size_t;
                            loop {
                                if i >= arr.size {
                                    break 's_293;
                                }
                                let mut hl_id: ::core::ffi::c_int = unsafe {
                                    object_to_hl_id(
                                        *arr.items.add(i),
                                        c"hl_group item".as_ptr(),
                                        &mut error,
                                    )
                                };
                                if error.is_set() {
                                    break '_error;
                                }
                                if hl_id != 0 {
                                    has_hl_multiple = true;
                                }
                                i = i.wrapping_add(1);
                            }
                        } else {
                            hl.hl_id = unsafe {
                                object_to_hl_id(opts.hl_group, c"hl_group".as_ptr(), &mut error)
                            };
                            if error.is_set() {
                                break '_error;
                            }
                        }
                    }
                    has_hl = hl.hl_id > 0 as ::core::ffi::c_int;
                }
                sign.hl_id = opts.sign_hl_group as ::core::ffi::c_int;
                sign.cursorline_hl_id = opts.cursorline_hl_group as ::core::ffi::c_int;
                sign.number_hl_id = opts.number_hl_group as ::core::ffi::c_int;
                sign.line_hl_id = opts.line_hl_group as ::core::ffi::c_int;
                if sign.hl_id != 0
                    || sign.cursorline_hl_id != 0
                    || sign.number_hl_id != 0
                    || sign.line_hl_id != 0
                {
                    sign.flags = (sign.flags as ::core::ffi::c_int
                        | kSHIsSign as ::core::ffi::c_int)
                        as uint16_t;
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__conceal,
                ) {
                    hl.flags = (hl.flags as ::core::ffi::c_int | kSHConceal as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true;
                    if opts.conceal.len() > 0 as size_t {
                        let mut ch: ::core::ffi::c_int = 0;
                        hl.conceal_char =
                            unsafe { utfc_ptr2schar(opts.conceal.data(), &raw mut ch) };
                        if !(hl.conceal_char != 0
                            && unsafe { vim_isprintc(ch) } as ::core::ffi::c_int != 0)
                        {
                            let why = c"conceal char has to be printable";
                            error = Error::validation(why);
                            break '_error;
                        }
                    }
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__conceal_lines,
                ) {
                    hl.flags = (hl.flags as ::core::ffi::c_int
                        | kSHConcealLines as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true;
                    if opts.conceal_lines.len() > 0 as size_t
                        && !(unsafe { *opts.conceal_lines.data() } as ::core::ffi::c_int
                            == '\0' as ::core::ffi::c_int)
                    {
                        let why = c"conceal_lines has to be an empty string";
                        error = Error::validation(why);
                        break '_error;
                    }
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__virt_text,
                ) {
                    let slot = &mut error;
                    let width = &raw mut virt_text.width;
                    virt_text.data.virt_text =
                        unsafe { parse_virt_text(opts.virt_text, slot, width) };
                    if error.is_set() {
                        break '_error;
                    }
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__virt_text_pos,
                ) {
                    let mut str: String_0 = opts.virt_text_pos;
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
                        error = err_bad_value(c"virt_text_pos", unsafe { str.as_cstr() });
                        break '_error;
                    }
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__virt_text_win_col,
                ) {
                    virt_text.col = opts.virt_text_win_col as ::core::ffi::c_int;
                    virt_text.pos = kVPosWinCol;
                }
                hl.flags = (hl.flags as ::core::ffi::c_int
                    | if opts.hl_eol as ::core::ffi::c_int != 0 {
                        kSHHlEol as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint16_t;
                virt_text.flags = (virt_text.flags as ::core::ffi::c_int
                    | ((if opts.virt_text_hide as ::core::ffi::c_int != 0 {
                        kVTHide as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if opts.virt_text_repeat_linebreak as ::core::ffi::c_int != 0 {
                        kVTRepeatLinebreak as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }))) as uint8_t;
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__hl_mode,
                ) {
                    let mut str_0: String_0 = opts.hl_mode;
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
                        error = err_bad_value(c"hl_mode", unsafe { str_0.as_cstr() });
                        break '_error;
                    }
                }
                virt_lines_flags = if opts.virt_lines_leftcol as ::core::ffi::c_int != 0 {
                    kVLLeftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__virt_lines_overflow,
                ) {
                    let mut str_1: String_0 = opts.virt_lines_overflow;
                    if unsafe { strequal(c"scroll".as_ptr(), str_1.data()) } {
                        virt_lines_flags |= kVLScroll as ::core::ffi::c_int;
                    } else if !unsafe { strequal(c"trunc".as_ptr(), str_1.data()) } && true {
                        // SAFETY: the value the keyset carried, live for this call.
                        error = err_bad_value(c"virt_lines_overflow", unsafe { str_1.as_cstr() });
                        break '_error;
                    }
                }
                's_785: {
                    if has_key(
                        opts.is_set__set_extmark_,
                        KEYSET_OPTIDX_set_extmark__virt_lines,
                    ) {
                        let mut a: Array = opts.virt_lines;
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= a.size {
                                break 's_785;
                            }
                            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                                != unsafe { (*a.items.add(j)).type_0 } as ::core::ffi::c_uint
                            {
                                let want = api_typename(kObjectTypeArray);
                                // SAFETY: the pointer the caller handed this call.
                                let got = unsafe { api_typename((*a.items.add(j)).type_0) };
                                error = err_expected(c"virt_text_line", want, Some(got));
                                break '_error;
                            }
                            let mut dummig: ::core::ffi::c_int = 0;
                            let (slot, dummy_width) = (&mut error, &raw mut dummig);
                            let mut jtem: VirtText = unsafe {
                                parse_virt_text((*a.items.add(j)).data.array, slot, dummy_width)
                            };
                            // `kv_push`, whose growth step c2rust expanded inline.
                            // SAFETY: `virt_lines` was built with the lines arm.
                            let lines = unsafe { &mut virt_lines.data.virt_lines };
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
                    | if opts.virt_lines_above as ::core::ffi::c_int != 0 {
                        kVTLinesAbove as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint8_t;
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__priority,
                ) {
                    if !(opts.priority >= 0 as Integer && opts.priority <= 65535 as Integer) {
                        error = err_out_of_range(c"priority");
                        break '_error;
                    }
                    hl.priority = opts.priority as DecorPriority;
                    sign.priority = opts.priority as DecorPriority;
                    virt_text.priority = opts.priority as DecorPriority;
                    virt_lines.priority = opts.priority as DecorPriority;
                }
                if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__sign_text,
                ) {
                    sign.text[0 as ::core::ffi::c_int as usize] = 0 as schar_T;
                    if unsafe {
                        init_sign_text(
                            opts.sign_text.data(),
                            &raw mut sign.text as *mut schar_T,
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
                right_gravity = if has_key(
                    opts.is_set__set_extmark_,
                    KEYSET_OPTIDX_set_extmark__right_gravity,
                ) {
                    opts.right_gravity as ::core::ffi::c_int
                } else {
                    1
                } != 0;
                if line2 == -1 as ::core::ffi::c_int
                    && col2 == -1 as ::core::ffi::c_int
                    && has_key(opts.is_set__set_extmark_, 30 as ::core::ffi::c_int)
                {
                    let why = c"cannot set end_right_gravity without end_row or end_col";
                    error = Error::validation(why);
                } else {
                    len = 0 as colnr_T;
                    if has_key(opts.is_set__set_extmark_, KEYSET_OPTIDX_set_extmark__spell) {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | if opts.spell as ::core::ffi::c_int != 0 {
                                kSHSpellOn as ::core::ffi::c_int
                            } else {
                                kSHSpellOff as ::core::ffi::c_int
                            }) as uint16_t;
                        has_hl = true;
                    }
                    if has_key(opts.is_set__set_extmark_, KEYSET_OPTIDX_set_extmark__url) {
                        url = unsafe { string_to_cstr(opts.url) };
                        has_hl = true;
                    }
                    if opts.ui_watched {
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
                            len = (if opts.ephemeral as ::core::ffi::c_int != 0 {
                                MAXCOL as ::core::ffi::c_int
                            } else {
                                unsafe { b.line_len(line as linenr_T + 1) }
                            }) as colnr_T;
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
                                && (line2 as linenr_T) < b.line_count()
                            {
                                len = (if opts.ephemeral as ::core::ffi::c_int != 0 {
                                    MAXCOL as ::core::ffi::c_int
                                } else {
                                    unsafe { b.line_len(line2 as linenr_T + 1) }
                                }) as colnr_T;
                            } else if line2 as linenr_T == b.line_count() {
                                len = 0 as ::core::ffi::c_int as colnr_T;
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
                            col2 = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if opts.ephemeral as ::core::ffi::c_int != 0
                            && !unsafe { DecorStateRef::current() }.win.is_null()
                            && unsafe { (*DecorStateRef::current().win).w_buffer } == b.raw()
                        {
                            let mut r: ::core::ffi::c_int = line as ::core::ffi::c_int;
                            let mut c: ::core::ffi::c_int = col as ::core::ffi::c_int;
                            if line2 == -1 as ::core::ffi::c_int {
                                line2 = r;
                                col2 = c as colnr_T;
                            }
                            let mut subpriority: DecorPriority = 0 as DecorPriority;
                            if has_key(
                                opts.is_set__set_extmark_,
                                KEYSET_OPTIDX_set_extmark___subpriority,
                            ) {
                                if !(opts._subpriority >= 0 as Integer
                                    && opts._subpriority <= 65535 as Integer)
                                {
                                    error = err_out_of_range(c"_subpriority");
                                    break '_error;
                                }
                                subpriority = opts._subpriority as DecorPriority;
                            }
                            if unsafe { virt_text.data.virt_text.size } != 0 {
                                // SAFETY: inside a decoration provider, so the
                                // redraw's decor state is set.
                                let state = unsafe { DecorStateRef::current() };
                                let c2 = col2 as ::core::ffi::c_int;
                                let vt = decor_put_vt(virt_text, ::core::ptr::null_mut());
                                // SAFETY: `state` is the redraw's own decor state.
                                unsafe { decor_range_add_virt(state, r, c, line2, c2, vt, true) };
                            }
                            if unsafe { virt_lines.data.virt_lines.size } != 0 {
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
                        } else if opts.ephemeral {
                            let why =
                                c"cannot set emphemeral mark outside of a decoration provider";
                            error = Error::exception(why);
                            break '_error;
                        } else {
                            let mut decor_flags: uint16_t = 0 as uint16_t;
                            let mut decor_alloc: *mut DecorVirtText =
                                ::core::ptr::null_mut::<DecorVirtText>();
                            if unsafe { virt_text.data.virt_text.size } != 0 {
                                decor_alloc = decor_put_vt(virt_text, decor_alloc);
                                if virt_text.pos as ::core::ffi::c_uint
                                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_VIRT_TEXT_INLINE)
                                        as uint16_t;
                                }
                            }
                            if unsafe { virt_lines.data.virt_lines.size } != 0 {
                                decor_alloc = decor_put_vt(virt_lines, decor_alloc);
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_VIRT_LINES)
                                    as uint16_t;
                            }
                            let mut decor_indexed: uint32_t = DECOR_ID_INVALID as uint32_t;
                            if sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int
                                != 0
                            {
                                sign.next = decor_indexed;
                                decor_indexed = decor_put_sh(sign);
                                if sign.text[0 as ::core::ffi::c_int as usize] != 0 {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNTEXT)
                                        as uint16_t;
                                }
                                if sign.number_hl_id != 0
                                    || sign.line_hl_id != 0
                                    || sign.cursorline_hl_id != 0
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNHL)
                                        as uint16_t;
                                }
                            }
                            if has_hl_multiple {
                                let mut arr_0: Array = unsafe { opts.hl_group.data.array };
                                let mut i_0: size_t = arr_0.size.wrapping_sub(1 as size_t);
                                while i_0 > 0 as size_t {
                                    let mut hl_id_0: ::core::ffi::c_int = unsafe {
                                        object_to_hl_id(
                                            *arr_0.items.add(i_0),
                                            c"hl_group item".as_ptr(),
                                            &mut error,
                                        )
                                    };
                                    if hl_id_0 > 0 as ::core::ffi::c_int {
                                        let mut sh_0: DecorSignHighlight =
                                            DECOR_SIGN_HIGHLIGHT_INIT;
                                        sh_0.hl_id = hl_id_0;
                                        sh_0.flags = (if opts.hl_eol as ::core::ffi::c_int != 0 {
                                            kSHHlEol as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        })
                                            as uint16_t;
                                        sh_0.next = decor_indexed;
                                        decor_indexed = decor_put_sh(sh_0);
                                        decor_flags = (decor_flags as ::core::ffi::c_int
                                            | MT_FLAG_DECOR_HL)
                                            as uint16_t;
                                    }
                                    i_0 = i_0.wrapping_sub(1);
                                }
                            }
                            if hl.flags as ::core::ffi::c_int
                                & kSHConcealLines as ::core::ffi::c_int
                                != 0
                            {
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_CONCEAL_LINES)
                                    as uint16_t;
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
                                decor_flags = (decor_flags as ::core::ffi::c_int | MT_FLAG_DECOR_HL)
                                    as uint16_t;
                            }
                            unsafe {
                                extmark_set(
                                    b.raw(),
                                    ns_id as uint32_t,
                                    &raw mut id,
                                    line as ::core::ffi::c_int,
                                    col as colnr_T,
                                    line2,
                                    col2,
                                    decor,
                                    decor_flags,
                                    right_gravity,
                                    opts.end_right_gravity,
                                    if has_key(
                                        opts.is_set__set_extmark_,
                                        KEYSET_OPTIDX_set_extmark__undo_restore,
                                    ) {
                                        opts.undo_restore as ::core::ffi::c_int
                                    } else {
                                        1
                                    } == 0,
                                    opts.invalidate,
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
    unsafe { clear_virttext(&raw mut virt_text.data.virt_text) };
    unsafe { clear_virtlines(&raw mut virt_lines.data.virt_lines) };
    if !url.is_null() {
        unsafe { xfree(url as *mut ::core::ffi::c_void) };
    }
    (0 as Integer).reported(error)
}
