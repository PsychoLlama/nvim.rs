//! `'cmdline highlighting'`: the callback that colours the line.
//!
//! [`color_cmdline`] calls whatever the `input()` caller or `:` gave it,
//! validates the returned list of `[start, end, group]` chunks and caches the
//! answer against the line it was computed for.  [`color_expr_cmdline`] is
//! the built-in colouring of `=` expressions.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn color_expr_cmdline(
    colored_ccline: *const CmdlineInfo,
    ret_ccline_colors: *mut ColoredCmdline,
) {
    unsafe {
        let mut parser_lines: [ParserLine; 2] = [
            ParserLine {
                data: (*colored_ccline).cmdbuff,
                size: strlen((*colored_ccline).cmdbuff),
                allocated: false_0 != 0,
            },
            ParserLine {
                data: ::core::ptr::null::<::core::ffi::c_char>(),
                size: 0 as size_t,
                allocated: false_0 != 0,
            },
        ];
        let mut plines_p: *mut ParserLine = &raw mut parser_lines as *mut ParserLine;
        let mut colors: ParserHighlight = ParserHighlight {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserHighlightChunk>(),
            init_array: [ParserHighlightChunk {
                start: ParserPosition { line: 0, col: 0 },
                end_col: 0,
                group: ::core::ptr::null::<::core::ffi::c_char>(),
            }; 16],
        };
        colors.capacity = colors.init_array.len();
        colors.items = colors.init_array.as_mut_ptr();
        let mut pstate: ParserState = ::core::mem::zeroed();
        viml_parser_init(
            &raw mut pstate,
            Some(parser_simple_get_line),
            &raw mut plines_p as *mut ::core::ffi::c_void,
            &raw mut colors,
        );
        let mut east: ExprAST =
            viml_pexpr_parse(&raw mut pstate, kExprFlagsDisallowEOC as ::core::ffi::c_int);
        viml_pexpr_free_ast(east);
        viml_parser_destroy(&mut pstate);
        (*ret_ccline_colors).colors.capacity = colors.size;
        (*ret_ccline_colors).colors.items = xrealloc(
            (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<CmdlineColorChunk>()
                .wrapping_mul((*ret_ccline_colors).colors.capacity),
        ) as *mut CmdlineColorChunk;
        let mut prev_end: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < colors.size {
            let chunk: ParserHighlightChunk = *colors.items.offset(i as isize);
            '_c2rust_label: {
                if chunk.start.col < 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                    b"chunk.start.col < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3313 as ::core::ffi::c_uint,
                    b"void color_expr_cmdline(const CmdlineInfo *const, ColoredCmdline *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            '_c2rust_label_0: {
                if chunk.end_col < 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                    b"chunk.end_col < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3314 as ::core::ffi::c_uint,
                    b"void color_expr_cmdline(const CmdlineInfo *const, ColoredCmdline *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            if chunk.start.col != prev_end {
                if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
                    (*ret_ccline_colors).colors.capacity =
                        if (*ret_ccline_colors).colors.capacity != 0 {
                            (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                    (*ret_ccline_colors).colors.items = xrealloc(
                        (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<CmdlineColorChunk>()
                            .wrapping_mul((*ret_ccline_colors).colors.capacity),
                    )
                        as *mut CmdlineColorChunk;
                } else {
                };
                let c2rust_fresh12 = (*ret_ccline_colors).colors.size;
                (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
                *(*ret_ccline_colors)
                    .colors
                    .items
                    .offset(c2rust_fresh12 as isize) = CmdlineColorChunk {
                    start: prev_end as ::core::ffi::c_int,
                    end: chunk.start.col as ::core::ffi::c_int,
                    hl_id: 0 as ::core::ffi::c_int,
                };
            }
            if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
                (*ret_ccline_colors).colors.capacity = if (*ret_ccline_colors).colors.capacity != 0
                {
                    (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*ret_ccline_colors).colors.items = xrealloc(
                    (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<CmdlineColorChunk>()
                        .wrapping_mul((*ret_ccline_colors).colors.capacity),
                ) as *mut CmdlineColorChunk;
            } else {
            };
            let c2rust_fresh13 = (*ret_ccline_colors).colors.size;
            (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
            *(*ret_ccline_colors)
                .colors
                .items
                .offset(c2rust_fresh13 as isize) = CmdlineColorChunk {
                start: chunk.start.col as ::core::ffi::c_int,
                end: chunk.end_col as ::core::ffi::c_int,
                hl_id: syn_name2id(chunk.group),
            };
            prev_end = chunk.end_col;
            i = i.wrapping_add(1);
        }
        if prev_end < (*colored_ccline).cmdlen as size_t {
            if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
                (*ret_ccline_colors).colors.capacity = if (*ret_ccline_colors).colors.capacity != 0
                {
                    (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*ret_ccline_colors).colors.items = xrealloc(
                    (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<CmdlineColorChunk>()
                        .wrapping_mul((*ret_ccline_colors).colors.capacity),
                ) as *mut CmdlineColorChunk;
            } else {
            };
            let c2rust_fresh14 = (*ret_ccline_colors).colors.size;
            (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
            *(*ret_ccline_colors)
                .colors
                .items
                .offset(c2rust_fresh14 as isize) = CmdlineColorChunk {
                start: prev_end as ::core::ffi::c_int,
                end: (*colored_ccline).cmdlen,
                hl_id: 0 as ::core::ffi::c_int,
            };
        }
        if colors.items != &raw mut colors.init_array as *mut ParserHighlightChunk {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut colors.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        }
    }
}

pub(crate) unsafe extern "C" fn color_cmdline(mut colored_ccline: *mut CmdlineInfo) -> bool {
    unsafe {
        let mut cbcall_ret: bool = false;
        let mut prev_end: varnumber_T = 0;
        let mut i: ::core::ffi::c_int = 0;
        let mut printed_errmsg: bool = false_0 != 0;
        let mut ret: bool = true_0 != 0;
        let mut ccline_colors: *mut ColoredCmdline = &raw mut (*colored_ccline).last_colors;
        if (*ccline_colors).prompt_id == (*colored_ccline).prompt_id
            && !(*ccline_colors).cmdbuff.is_null()
            && strcmp((*ccline_colors).cmdbuff, (*colored_ccline).cmdbuff)
                == 0 as ::core::ffi::c_int
        {
            return ret;
        }
        (*ccline_colors).colors.size = 0 as size_t;
        if (*colored_ccline).cmdbuff.is_null()
            || *(*colored_ccline).cmdbuff as ::core::ffi::c_int == NUL
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*ccline_colors).cmdbuff as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            return ret;
        }
        let mut arg_allocated: bool = false_0 != 0;
        let mut arg: typval_T = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: (*colored_ccline).cmdbuff,
            },
        };
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        static prev_prompt_errors: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        let mut color_cb: Callback = Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut can_free_cb: bool = false_0 != 0;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut err_errmsg: *const ::core::ffi::c_char =
            &raw const e_intern2 as *const ::core::ffi::c_char;
        let mut dgc_ret: bool = true_0 != 0;
        '_color_cmdline_end: {
            if (*colored_ccline).prompt_id != prev_prompt_id.get() {
                prev_prompt_errors.set(0 as ::core::ffi::c_int);
                prev_prompt_id.set((*colored_ccline).prompt_id);
            } else if prev_prompt_errors.get() >= MAX_CB_ERRORS as ::core::ffi::c_int {
                break '_color_cmdline_end;
            }
            if (*colored_ccline).highlight_callback.type_0 as ::core::ffi::c_uint
                != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                '_c2rust_label: {
                    if (*colored_ccline).input_fn != 0 {
                    } else {
                        __assert_fail(
                            b"colored_ccline->input_fn\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3408 as ::core::ffi::c_uint,
                            b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                color_cb = (*colored_ccline).highlight_callback;
            } else if (*colored_ccline).cmdfirstc == ':' as ::core::ffi::c_int {
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                err_errmsg = b"E5408: Unable to get g:Nvim_color_cmdline callback: %s\0".as_ptr()
                    as *const ::core::ffi::c_char;
                dgc_ret = tv_dict_get_callback(
                    get_globvar_dict(),
                    b"Nvim_color_cmdline\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                    &raw mut color_cb,
                );
                try_leave(&raw mut tstate, &raw mut err);
                can_free_cb = true_0 != 0;
            } else if (*colored_ccline).cmdfirstc == '=' as ::core::ffi::c_int {
                color_expr_cmdline(colored_ccline, ccline_colors);
            }
            '_color_cmdline_error: {
                if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                    || !dgc_ret)
                {
                    if color_cb.type_0 as ::core::ffi::c_uint
                        == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break '_color_cmdline_end;
                    } else {
                        if *(*colored_ccline)
                            .cmdbuff
                            .offset((*colored_ccline).cmdlen as isize)
                            as ::core::ffi::c_int
                            != NUL
                        {
                            arg_allocated = true_0 != 0;
                            arg.vval.v_string = xmemdupz(
                                (*colored_ccline).cmdbuff as *const ::core::ffi::c_void,
                                (*colored_ccline).cmdlen as size_t,
                            )
                                as *mut ::core::ffi::c_char;
                        }
                        getln_interrupted_highlight.set(false_0 != 0);
                        cbcall_ret = true_0 != 0;
                        let mut tstate_0: TryState = TryState {
                            current_exception: ::core::ptr::null_mut::<except_T>(),
                            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                            msg_list: ::core::ptr::null::<*const msglist_T>(),
                            got_int: 0,
                            did_throw: false,
                            need_rethrow: 0,
                            did_emsg: 0,
                        };
                        try_enter(&raw mut tstate_0);
                        err_errmsg = b"E5407: Callback has thrown an exception: %s\0".as_ptr()
                            as *const ::core::ffi::c_char;
                        let saved_msg_col: ::core::ffi::c_int = msg_col.get();
                        (*msg_silent.ptr()) += 1;
                        cbcall_ret = callback_call(
                            &raw mut color_cb,
                            1 as ::core::ffi::c_int,
                            &raw mut arg,
                            &raw mut tv,
                        );
                        (*msg_silent.ptr()) -= 1;
                        msg_col.set(saved_msg_col);
                        if got_int.get() {
                            getln_interrupted_highlight.set(true);
                        }
                        try_leave(&raw mut tstate_0, &raw mut err);
                        if !(err.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                            || !cbcall_ret)
                        {
                            if tv.v_type as ::core::ffi::c_uint
                                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                msg_scroll.set(true_0);
                                msg_putchar('\n' as ::core::ffi::c_int);
                                smsg(
                                    HLF_E,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    gettext(b"E5400: Callback should return list\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                );
                                printed_errmsg = true_0 != 0;
                            } else if tv.vval.v_list.is_null() {
                                break '_color_cmdline_end;
                            } else {
                                prev_end = 0 as varnumber_T;
                                i = 0 as ::core::ffi::c_int;
                                let l_: *const list_T = tv.vval.v_list;
                                's_561: {
                                    if !l_.is_null() {
                                        let mut li: *const listitem_T = (*l_).lv_first;
                                        loop {
                                            if li.is_null() {
                                                break 's_561;
                                            }
                                            if (*li).li_tv.v_type as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                msg_scroll.set(1 as ::core::ffi::c_int);
                                                msg_putchar('\n' as ::core::ffi::c_int);
                                                smsg(
                                                    HLF_E,
                                                    gettext(
                                                        b"E5401: List item %i is not a List\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    i,
                                                );
                                                printed_errmsg = true;
                                                break '_color_cmdline_error;
                                            } else {
                                                let l: *const list_T = (*li).li_tv.vval.v_list;
                                                if tv_list_len(l) != 3 as ::core::ffi::c_int {
                                                    msg_scroll.set(1 as ::core::ffi::c_int);
                                                    msg_putchar('\n' as ::core::ffi::c_int);
                                                    smsg(
                                                    HLF_E,
                                                    gettext(
                                                        b"E5402: List item %i has incorrect length: %d /= 3\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    i,
                                                    tv_list_len(l),
                                                );
                                                    printed_errmsg = true;
                                                    break '_color_cmdline_error;
                                                } else {
                                                    let mut error: bool = false;
                                                    let start: varnumber_T = tv_get_number_chk(
                                                        &raw mut (*tv_list_first(l)).li_tv,
                                                        &raw mut error,
                                                    );
                                                    if error {
                                                        break '_color_cmdline_error;
                                                    }
                                                    if !(prev_end <= start
                                                        && start
                                                            < (*colored_ccline).cmdlen
                                                                as varnumber_T)
                                                    {
                                                        msg_scroll.set(1 as ::core::ffi::c_int);
                                                        msg_putchar('\n' as ::core::ffi::c_int);
                                                        smsg(
                                                        HLF_E,
                                                        gettext(
                                                            b"E5403: Chunk %i start %ld not in range [%ld, %i)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        i,
                                                        start,
                                                        prev_end,
                                                        (*colored_ccline).cmdlen,
                                                    );
                                                        printed_errmsg = true;
                                                        break '_color_cmdline_error;
                                                    } else if (*utf8len_tab_zero.ptr())
                                                        [*(*colored_ccline)
                                                            .cmdbuff
                                                            .offset(start as isize)
                                                            as uint8_t
                                                            as usize]
                                                        as ::core::ffi::c_int
                                                        == 0 as ::core::ffi::c_int
                                                    {
                                                        msg_scroll.set(1 as ::core::ffi::c_int);
                                                        msg_putchar('\n' as ::core::ffi::c_int);
                                                        smsg(
                                                        HLF_E,
                                                        gettext(
                                                            b"E5405: Chunk %i start %ld splits multibyte character\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        i,
                                                        start,
                                                    );
                                                        printed_errmsg = true;
                                                        break '_color_cmdline_error;
                                                    } else {
                                                        if start != prev_end {
                                                            if (*ccline_colors).colors.size
                                                                == (*ccline_colors).colors.capacity
                                                            {
                                                                (*ccline_colors).colors.capacity =
                                                                    if (*ccline_colors)
                                                                        .colors
                                                                        .capacity
                                                                        != 0
                                                                    {
                                                                        (*ccline_colors).colors.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                    } else {
                                                                        8 as size_t
                                                                    };
                                                                (*ccline_colors).colors.items = xrealloc(
                                                                (*ccline_colors).colors.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ::core::mem::size_of::<
                                                                    CmdlineColorChunk,
                                                                >(
                                                                )
                                                                .wrapping_mul(
                                                                    (*ccline_colors)
                                                                        .colors
                                                                        .capacity,
                                                                ),
                                                            )
                                                                as *mut CmdlineColorChunk;
                                                            } else {
                                                            };
                                                            let c2rust_fresh9 =
                                                                (*ccline_colors).colors.size;
                                                            (*ccline_colors).colors.size =
                                                                (*ccline_colors)
                                                                    .colors
                                                                    .size
                                                                    .wrapping_add(1);
                                                            *(*ccline_colors)
                                                                .colors
                                                                .items
                                                                .offset(c2rust_fresh9 as isize) =
                                                                CmdlineColorChunk {
                                                                    start: prev_end
                                                                        as ::core::ffi::c_int,
                                                                    end: start
                                                                        as ::core::ffi::c_int,
                                                                    hl_id: 0 as ::core::ffi::c_int,
                                                                };
                                                        }
                                                        let end: varnumber_T = tv_get_number_chk(
                                                            &raw mut (*(*tv_list_first(l)).li_next)
                                                                .li_tv,
                                                            &raw mut error,
                                                        );
                                                        if error {
                                                            break '_color_cmdline_error;
                                                        }
                                                        if !(start < end
                                                            && end
                                                                <= (*colored_ccline).cmdlen
                                                                    as varnumber_T)
                                                        {
                                                            msg_scroll.set(1 as ::core::ffi::c_int);
                                                            msg_putchar('\n' as ::core::ffi::c_int);
                                                            smsg(
                                                            HLF_E,
                                                            gettext(
                                                                b"E5404: Chunk %i end %ld not in range (%ld, %i]\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            i,
                                                            end,
                                                            start,
                                                            (*colored_ccline).cmdlen,
                                                        );
                                                            printed_errmsg = true;
                                                            break '_color_cmdline_error;
                                                        } else if end
                                                            < (*colored_ccline).cmdlen
                                                                as varnumber_T
                                                            && (*utf8len_tab_zero.ptr())
                                                                [*(*colored_ccline)
                                                                    .cmdbuff
                                                                    .offset(end as isize)
                                                                    as uint8_t
                                                                    as usize]
                                                                as ::core::ffi::c_int
                                                                == 0 as ::core::ffi::c_int
                                                        {
                                                            msg_scroll.set(1 as ::core::ffi::c_int);
                                                            msg_putchar('\n' as ::core::ffi::c_int);
                                                            smsg(
                                                            HLF_E,
                                                            gettext(
                                                                b"E5406: Chunk %i end %ld splits multibyte character\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            i,
                                                            end,
                                                        );
                                                            printed_errmsg = true;
                                                            break '_color_cmdline_error;
                                                        } else {
                                                            prev_end = end;
                                                            let group: *const ::core::ffi::c_char =
                                                                tv_get_string_chk(
                                                                    &raw mut (*tv_list_last(l))
                                                                        .li_tv,
                                                                );
                                                            if group.is_null() {
                                                                break '_color_cmdline_error;
                                                            }
                                                            if (*ccline_colors).colors.size
                                                                == (*ccline_colors).colors.capacity
                                                            {
                                                                (*ccline_colors).colors.capacity =
                                                                    if (*ccline_colors)
                                                                        .colors
                                                                        .capacity
                                                                        != 0
                                                                    {
                                                                        (*ccline_colors).colors.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                    } else {
                                                                        8 as size_t
                                                                    };
                                                                (*ccline_colors).colors.items = xrealloc(
                                                                (*ccline_colors).colors.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ::core::mem::size_of::<
                                                                    CmdlineColorChunk,
                                                                >(
                                                                )
                                                                .wrapping_mul(
                                                                    (*ccline_colors)
                                                                        .colors
                                                                        .capacity,
                                                                ),
                                                            )
                                                                as *mut CmdlineColorChunk;
                                                            } else {
                                                            };
                                                            let c2rust_fresh10 =
                                                                (*ccline_colors).colors.size;
                                                            (*ccline_colors).colors.size =
                                                                (*ccline_colors)
                                                                    .colors
                                                                    .size
                                                                    .wrapping_add(1);
                                                            *(*ccline_colors)
                                                                .colors
                                                                .items
                                                                .offset(c2rust_fresh10 as isize) =
                                                                CmdlineColorChunk {
                                                                    start: start
                                                                        as ::core::ffi::c_int,
                                                                    end: end as ::core::ffi::c_int,
                                                                    hl_id: syn_name2id(group),
                                                                };
                                                            i += 1;
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if prev_end < (*colored_ccline).cmdlen as varnumber_T {
                                    if (*ccline_colors).colors.size
                                        == (*ccline_colors).colors.capacity
                                    {
                                        (*ccline_colors).colors.capacity =
                                            if (*ccline_colors).colors.capacity != 0 {
                                                (*ccline_colors).colors.capacity
                                                    << 1 as ::core::ffi::c_int
                                            } else {
                                                8 as size_t
                                            };
                                        (*ccline_colors).colors.items = xrealloc(
                                            (*ccline_colors).colors.items
                                                as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<CmdlineColorChunk>()
                                                .wrapping_mul((*ccline_colors).colors.capacity),
                                        )
                                            as *mut CmdlineColorChunk;
                                    } else {
                                    };
                                    let c2rust_fresh11 = (*ccline_colors).colors.size;
                                    (*ccline_colors).colors.size =
                                        (*ccline_colors).colors.size.wrapping_add(1);
                                    *(*ccline_colors)
                                        .colors
                                        .items
                                        .offset(c2rust_fresh11 as isize) = CmdlineColorChunk {
                                        start: prev_end as ::core::ffi::c_int,
                                        end: (*colored_ccline).cmdlen,
                                        hl_id: 0 as ::core::ffi::c_int,
                                    };
                                }
                                prev_prompt_errors.set(0 as ::core::ffi::c_int);
                                break '_color_cmdline_end;
                            }
                        }
                    }
                }
            }
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                msg_scroll.set(true_0);
                msg_putchar('\n' as ::core::ffi::c_int);
                smsg(HLF_E, gettext(err_errmsg), err.msg);
                printed_errmsg = true_0 != 0;
                api_clear_error(&raw mut err);
            }
            '_c2rust_label_1: {
                if printed_errmsg {
                } else {
                    __assert_fail(
                        b"printed_errmsg\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3557 as ::core::ffi::c_uint,
                        b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*prev_prompt_errors.ptr()) += 1;
            (*ccline_colors).colors.size = 0 as size_t;
            redrawcmdline();
            ret = false_0 != 0;
        }
        '_c2rust_label_0: {
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            } else {
                __assert_fail(
                    b"!ERROR_SET(&err)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3538 as ::core::ffi::c_uint,
                    b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if can_free_cb {
            callback_free(&raw mut color_cb);
        }
        xfree((*ccline_colors).cmdbuff as *mut ::core::ffi::c_void);
        (*ccline_colors).prompt_id = (*colored_ccline).prompt_id;
        if arg_allocated {
            (*ccline_colors).cmdbuff = arg.vval.v_string;
        } else {
            (*ccline_colors).cmdbuff = xmemdupz(
                (*colored_ccline).cmdbuff as *const ::core::ffi::c_void,
                (*colored_ccline).cmdlen as size_t,
            ) as *mut ::core::ffi::c_char;
        }
        tv_clear(&raw mut tv);
        return ret;
    }
}
