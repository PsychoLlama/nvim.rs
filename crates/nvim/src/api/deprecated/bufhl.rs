//! Buffer highlights and virtual text before extmarks.
//!
//! `nvim_buf_add_highlight`, `nvim_buf_clear_highlight` and
//! `nvim_buf_set_virtual_text` are the pre-namespace decoration API; each is
//! now a shim that allocates or reuses a namespace (`src2ns`) and calls
//! `extmark_set`.  `nvim_buf_get_number` is the handle-to-number accessor from
//! before handles *were* numbers.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub unsafe extern "C" fn nvim_buf_get_number(mut buffer: Buffer, mut err: *mut Error) -> Integer {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return 0 as Integer;
        }
        return (*buf).handle as Integer;
    }
}

unsafe fn src2ns(mut src_id: *mut Integer) -> uint32_t {
    unsafe {
        if *src_id == 0 as Integer {
            *src_id = nvim_create_namespace(String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            });
        }
        if *src_id < 0 as Integer {
            return ((1 as ::core::ffi::c_int as uint32_t) << 31 as ::core::ffi::c_int)
                .wrapping_sub(1 as uint32_t);
        }
        return *src_id as uint32_t;
    }
}

pub unsafe extern "C" fn nvim_buf_clear_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut line_start: Integer,
    mut line_end: Integer,
    mut err: *mut Error,
) {
    unsafe {
        nvim_buf_clear_namespace(buffer, ns_id, line_start, line_end, err);
    }
}

pub unsafe extern "C" fn nvim_buf_add_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut hl_group: String_0,
    mut line: Integer,
    mut col_start: Integer,
    mut col_end: Integer,
    mut err: *mut Error,
) -> Integer {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return 0 as Integer;
        }
        if !(line >= 0 as Integer && line < MAXLNUM as ::core::ffi::c_int as Integer) {
            api_err_invalid(
                err,
                c"line number".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return 0 as Integer;
        }
        if !(col_start >= 0 as Integer && col_start <= MAXCOL as ::core::ffi::c_int as Integer) {
            api_err_invalid(
                err,
                c"column".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return 0 as Integer;
        }
        if col_end < 0 as Integer || col_end > MAXCOL as ::core::ffi::c_int as Integer {
            col_end = MAXCOL as ::core::ffi::c_int as Integer;
        }
        let mut ns: uint32_t = src2ns(&raw mut ns_id);
        if !(line < (*buf).b_ml.ml_line_count as Integer) {
            return ns_id;
        }
        let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if hl_group.size > 0 as size_t {
            hl_id = syn_check_group(hl_group.data, hl_group.size);
        } else {
            return ns_id;
        }
        let mut end_line: ::core::ffi::c_int = line as ::core::ffi::c_int;
        if col_end == MAXCOL as ::core::ffi::c_int as Integer {
            col_end = 0 as Integer;
            end_line += 1;
        }
        let mut decor: DecorInline = DECOR_INLINE_INIT;
        decor.data.hl.hl_id = hl_id;
        extmark_set(
            buf,
            ns,
            ::core::ptr::null_mut::<uint32_t>(),
            line as ::core::ffi::c_int,
            col_start as colnr_T,
            end_line,
            col_end as colnr_T,
            decor,
            MT_FLAG_DECOR_HL as uint16_t,
            true,
            false,
            false,
            false,
            ::core::ptr::null_mut::<Error>(),
        );
        return ns_id;
    }
}

pub unsafe extern "C" fn nvim_buf_set_virtual_text(
    mut buffer: Buffer,
    mut src_id: Integer,
    mut line: Integer,
    mut chunks: Array,
    mut _opts: *mut KeyDict_empty,
    mut err: *mut Error,
) -> Integer {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return 0 as Integer;
        }
        if line < 0 as Integer || line >= MAXLNUM as ::core::ffi::c_int as Integer {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Line number outside range".as_ptr(),
            );
            return 0 as Integer;
        }
        let mut ns_id: uint32_t = src2ns(&raw mut src_id);
        let mut width: ::core::ffi::c_int = 0;
        let mut virt_text: VirtText = parse_virt_text(chunks, err, &raw mut width);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return 0 as Integer;
        }
        let mut existing: *mut DecorVirtText =
            decor_find_virttext(buf, line as ::core::ffi::c_int, ns_id as uint64_t);
        if !existing.is_null() {
            clear_virttext(&raw mut (*existing).data.virt_text);
            (*existing).data.virt_text = virt_text;
            (*existing).width = width;
            return src_id;
        }
        let mut vt: *mut DecorVirtText =
            xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
        *vt = DecorVirtText {
            flags: 0 as uint8_t,
            hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
            priority: DECOR_PRIORITY_BASE as DecorPriority,
            width: 0 as ::core::ffi::c_int,
            col: 0 as ::core::ffi::c_int,
            pos: kVPosEndOfLine,
            data: C2Rust_Unnamed_2 {
                virt_text: VirtText {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
            },
            next: ::core::ptr::null_mut::<DecorVirtText>(),
        };
        (*vt).data.virt_text = virt_text;
        (*vt).width = width;
        (*vt).priority = 0 as DecorPriority;
        let mut decor: DecorInline = DecorInline {
            ext: true,
            data: DecorInlineData {
                ext: DecorExt {
                    sh_idx: DECOR_ID_INVALID as uint32_t,
                    vt: vt,
                },
            },
        };
        extmark_set(
            buf,
            ns_id,
            ::core::ptr::null_mut::<uint32_t>(),
            line as ::core::ffi::c_int,
            0 as colnr_T,
            -1 as ::core::ffi::c_int,
            -1 as colnr_T,
            decor,
            0 as uint16_t,
            true,
            false,
            false,
            false,
            ::core::ptr::null_mut::<Error>(),
        );
        return src_id;
    }
}
