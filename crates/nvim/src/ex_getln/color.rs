//! `'cmdline highlighting'`: the callback that colours the line.
//!
//! [`color_cmdline`] calls whatever the `input()` caller or `:` gave it,
//! validates the returned list of `[start, end, group]` chunks and caches the
//! answer against the line it was computed for.  [`color_expr_cmdline`] is
//! the built-in colouring of `=` expressions.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::smsg_c;
use crate::types::{VAR_LIST, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, kErrorTypeNone};

/// C's `kv_push` onto a `CmdlineColors`, doubling the heap array from 8.
unsafe fn push_chunk(colors: *mut CmdlineColors, chunk: CmdlineColorChunk) {
    unsafe {
        if (*colors).size == (*colors).capacity {
            (*colors).capacity = if (*colors).capacity != 0 {
                (*colors).capacity << 1
            } else {
                8
            };
            (*colors).items = xrealloc(
                (*colors).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<CmdlineColorChunk>() * (*colors).capacity,
            ) as *mut CmdlineColorChunk;
        }
        *(*colors).items.add((*colors).size) = chunk;
        (*colors).size += 1;
    }
}

/// Colour a `=` expression command line with the Vimscript expression parser,
/// filling the gaps the parser leaves uncoloured with `hl_id` 0.
pub(crate) unsafe fn color_expr_cmdline(
    colored_ccline: *const CmdlineInfo,
    ret_ccline_colors: *mut ColoredCmdline,
) {
    unsafe {
        let mut parser_lines: [ParserLine; 2] = [
            ParserLine {
                data: (*colored_ccline).cmdbuff,
                size: strlen((*colored_ccline).cmdbuff),
                allocated: false,
            },
            ParserLine {
                data: ::core::ptr::null::<::core::ffi::c_char>(),
                size: 0,
                allocated: false,
            },
        ];
        let mut plines_p: *mut ParserLine = parser_lines.as_mut_ptr();

        // C's `kvi_init`: a kvec whose first 16 entries live in the struct.
        let mut colors = ParserHighlight {
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
        let east: ExprAST =
            viml_pexpr_parse(&raw mut pstate, kExprFlagsDisallowEOC as ::core::ffi::c_int);
        viml_pexpr_free_ast(east);
        viml_parser_destroy(&mut pstate);

        // C's `kv_resize`: reserve exactly what the parser produced. The
        // gap-filling chunks below may still push past it.
        (*ret_ccline_colors).colors.capacity = colors.size;
        (*ret_ccline_colors).colors.items = xrealloc(
            (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<CmdlineColorChunk>() * (*ret_ccline_colors).colors.capacity,
        ) as *mut CmdlineColorChunk;

        let out = &raw mut (*ret_ccline_colors).colors;
        let mut prev_end: size_t = 0;
        let mut i: size_t = 0;
        while i < colors.size {
            let chunk: ParserHighlightChunk = *colors.items.add(i);
            debug_assert!(chunk.start.col < INT_MAX as size_t);
            debug_assert!(chunk.end_col < INT_MAX as size_t);
            if chunk.start.col != prev_end {
                push_chunk(
                    out,
                    CmdlineColorChunk {
                        start: prev_end as ::core::ffi::c_int,
                        end: chunk.start.col as ::core::ffi::c_int,
                        hl_id: 0,
                    },
                );
            }
            push_chunk(
                out,
                CmdlineColorChunk {
                    start: chunk.start.col as ::core::ffi::c_int,
                    end: chunk.end_col as ::core::ffi::c_int,
                    hl_id: syn_name2id(chunk.group),
                },
            );
            prev_end = chunk.end_col;
            i += 1;
        }
        if prev_end < (*colored_ccline).cmdlen as size_t {
            push_chunk(
                out,
                CmdlineColorChunk {
                    start: prev_end as ::core::ffi::c_int,
                    end: (*colored_ccline).cmdlen,
                    hl_id: 0,
                },
            );
        }

        // C's `kvi_destroy`: only free once the kvec has spilled to the heap.
        if colors.items != colors.init_array.as_mut_ptr() {
            xfree(colors.items as *mut ::core::ffi::c_void);
            colors.items = ::core::ptr::null_mut::<ParserHighlightChunk>();
        }
    }
}

/// Colour the command line, through the user's callback where there is one.
///
/// `colored_ccline` is also the cache: when its `prompt_id` and `cmdbuff`
/// still match what `last_colors` was computed from, this does nothing.  The
/// whole line is always coloured.
///
/// Answers true if [`super::draw::draw_cmdline`] may proceed, false if there
/// is nothing for it to do.
pub(crate) unsafe fn color_cmdline(colored_ccline: *mut CmdlineInfo) -> bool {
    unsafe {
        let mut printed_errmsg = false;

        // C's `PRINT_ERRMSG`: an error that scrolls the command line away.
        // A macro rather than a closure because `smsg` is variadic.
        macro_rules! print_errmsg {
            ($($arg:tt)*) => {{
                msg_scroll.set(1);
                msg_putchar('\n' as ::core::ffi::c_int);
                smsg_c!(HLF_E, $($arg)*);
                printed_errmsg = true;
            }};
        }

        let mut ret = true;
        let ccline_colors: *mut ColoredCmdline = &raw mut (*colored_ccline).last_colors;

        // Is the result of the previous call still valid?
        if (*ccline_colors).prompt_id == (*colored_ccline).prompt_id
            && !(*ccline_colors).cmdbuff.is_null()
            && strcmp((*ccline_colors).cmdbuff, (*colored_ccline).cmdbuff) == 0
        {
            return ret;
        }

        (*ccline_colors).colors.size = 0;

        if (*colored_ccline).cmdbuff.is_null()
            || *(*colored_ccline).cmdbuff as ::core::ffi::c_int == NUL
        {
            // Nothing to do.
            xfree((*ccline_colors).cmdbuff as *mut ::core::ffi::c_void);
            (*ccline_colors).cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
            return ret;
        }

        let mut arg_allocated = false;
        let mut arg = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: (*colored_ccline).cmdbuff,
            },
        };
        let mut tv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };

        // Both are C function-level statics. `prev_prompt_id` starts at
        // UINT_MAX so that the first prompt of a session, whatever its id,
        // counts as a new one.
        static prev_prompt_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(UINT_MAX);
        static prev_prompt_errors: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

        let mut color_cb = Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut can_free_cb = false;
        let mut err: Error = ERROR_INIT;
        let mut err_errmsg = e_intern2.as_ptr();
        let mut dgc_ret = true;

        // The C's two labels: `Ok` leaves at `color_cmdline_end`, `Err` at
        // `color_cmdline_error` (which then falls into `color_cmdline_end`).
        let outcome: Result<(), ()> = 'body: {
            if (*colored_ccline).prompt_id != prev_prompt_id.get() {
                prev_prompt_errors.set(0);
                prev_prompt_id.set((*colored_ccline).prompt_id);
            } else if prev_prompt_errors.get() >= MAX_CB_ERRORS {
                break 'body Ok(());
            }

            if (*colored_ccline).highlight_callback.type_0 != kCallbackNone {
                // Currently this should only happen while processing input()
                // prompts.
                debug_assert!((*colored_ccline).input_fn != 0);
                color_cb = (*colored_ccline).highlight_callback;
            } else if (*colored_ccline).cmdfirstc == ':' as ::core::ffi::c_int {
                // C's TRY_WRAP.
                let mut tstate: TryState = TRY_STATE_INIT;
                try_enter(&raw mut tstate);
                err_errmsg = c"E5408: Unable to get g:Nvim_color_cmdline callback: %s".as_ptr();
                let key = c"Nvim_color_cmdline";
                dgc_ret = tv_dict_get_callback(
                    get_globvar_dict(),
                    key.as_ptr(),
                    key.count_bytes() as ptrdiff_t,
                    &raw mut color_cb,
                );
                try_leave(&raw mut tstate, &raw mut err);
                can_free_cb = true;
            } else if (*colored_ccline).cmdfirstc == '=' as ::core::ffi::c_int {
                color_expr_cmdline(colored_ccline, ccline_colors);
            }
            if err.type_0 != kErrorTypeNone || !dgc_ret {
                break 'body Err(());
            }

            if color_cb.type_0 == kCallbackNone {
                break 'body Ok(());
            }
            if *(*colored_ccline)
                .cmdbuff
                .offset((*colored_ccline).cmdlen as isize) as ::core::ffi::c_int
                != NUL
            {
                arg_allocated = true;
                arg.vval.v_string = xmemdupz(
                    (*colored_ccline).cmdbuff as *const ::core::ffi::c_void,
                    (*colored_ccline).cmdlen as size_t,
                ) as *mut ::core::ffi::c_char;
            }
            // msg_start(), called by e.g. :echo, may shift the command line to
            // the first column even under msg_silent. Two ways round it
            // without altering message.c: use full_screen, or save and restore
            // msg_col. Saving full_screen does not work well with `:redraw!`;
            // msg_col is not ideal either, but it merely misses a leading `:`,
            // where full_screen leaves the line shifted one column right with
            // the cursor in the wrong place.
            //
            // TRY_WRAP too, because error messages would otherwise overwrite
            // the typed command line.
            getln_interrupted_highlight.set(false);
            let mut cbcall_ret = true;
            let mut tstate: TryState = TRY_STATE_INIT;
            try_enter(&raw mut tstate);
            err_errmsg = c"E5407: Callback has thrown an exception: %s".as_ptr();
            let saved_msg_col = msg_col.get();
            msg_silent.set(msg_silent.get() + 1);
            cbcall_ret = callback_call(&raw mut color_cb, 1, &raw mut arg, &raw mut tv);
            msg_silent.set(msg_silent.get() - 1);
            msg_col.set(saved_msg_col);
            if got_int.get() {
                getln_interrupted_highlight.set(true);
            }
            try_leave(&raw mut tstate, &raw mut err);

            if err.type_0 != kErrorTypeNone || !cbcall_ret {
                break 'body Err(());
            }
            if tv.v_type != VAR_LIST {
                print_errmsg!(
                    c"%s".as_ptr(),
                    gettext(c"E5400: Callback should return list".as_ptr())
                );
                break 'body Err(());
            }
            if tv.vval.v_list.is_null() {
                break 'body Ok(());
            }

            let mut prev_end: varnumber_T = 0;
            let mut i: ::core::ffi::c_int = 0;
            let mut li: *const listitem_T = (*tv.vval.v_list).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type != VAR_LIST {
                    print_errmsg!(gettext(c"E5401: List item %i is not a List".as_ptr()), i);
                    break 'body Err(());
                }
                let l: *const list_T = (*li).li_tv.vval.v_list;
                if tv_list_len(l) != 3 {
                    print_errmsg!(
                        gettext(c"E5402: List item %i has incorrect length: %d /= 3".as_ptr()),
                        i,
                        tv_list_len(l)
                    );
                    break 'body Err(());
                }

                let mut error = false;
                let start = tv_get_number_chk(&raw mut (*tv_list_first(l)).li_tv, &raw mut error);
                if error {
                    break 'body Err(());
                } else if !(prev_end <= start && start < (*colored_ccline).cmdlen as varnumber_T) {
                    print_errmsg!(
                        gettext(c"E5403: Chunk %i start %ld not in range [%ld, %i)".as_ptr()),
                        i,
                        start,
                        prev_end,
                        (*colored_ccline).cmdlen
                    );
                    break 'body Err(());
                } else if utf8len_tab_zero
                    [*(*colored_ccline).cmdbuff.offset(start as isize) as uint8_t as usize]
                    == 0
                {
                    print_errmsg!(
                        gettext(c"E5405: Chunk %i start %ld splits multibyte character".as_ptr()),
                        i,
                        start
                    );
                    break 'body Err(());
                }

                if start != prev_end {
                    push_chunk(
                        &raw mut (*ccline_colors).colors,
                        CmdlineColorChunk {
                            start: prev_end as ::core::ffi::c_int,
                            end: start as ::core::ffi::c_int,
                            hl_id: 0,
                        },
                    );
                }

                let end = tv_get_number_chk(
                    &raw mut (*(*tv_list_first(l)).li_next).li_tv,
                    &raw mut error,
                );
                if error {
                    break 'body Err(());
                } else if !(start < end && end <= (*colored_ccline).cmdlen as varnumber_T) {
                    print_errmsg!(
                        gettext(c"E5404: Chunk %i end %ld not in range (%ld, %i]".as_ptr()),
                        i,
                        end,
                        start,
                        (*colored_ccline).cmdlen
                    );
                    break 'body Err(());
                } else if end < (*colored_ccline).cmdlen as varnumber_T
                    && utf8len_tab_zero
                        [*(*colored_ccline).cmdbuff.offset(end as isize) as uint8_t as usize]
                        == 0
                {
                    print_errmsg!(
                        gettext(c"E5406: Chunk %i end %ld splits multibyte character".as_ptr()),
                        i,
                        end
                    );
                    break 'body Err(());
                }

                prev_end = end;
                let group = tv_get_string_chk(&raw mut (*tv_list_last(l)).li_tv);
                if group.is_null() {
                    break 'body Err(());
                }
                push_chunk(
                    &raw mut (*ccline_colors).colors,
                    CmdlineColorChunk {
                        start: start as ::core::ffi::c_int,
                        end: end as ::core::ffi::c_int,
                        hl_id: syn_name2id(group),
                    },
                );
                i += 1;
                li = (*li).li_next;
            }

            if prev_end < (*colored_ccline).cmdlen as varnumber_T {
                push_chunk(
                    &raw mut (*ccline_colors).colors,
                    CmdlineColorChunk {
                        start: prev_end as ::core::ffi::c_int,
                        end: (*colored_ccline).cmdlen,
                        hl_id: 0,
                    },
                );
            }
            prev_prompt_errors.set(0);
            Ok(())
        };

        // color_cmdline_error:
        if outcome.is_err() {
            if err.type_0 != kErrorTypeNone {
                print_errmsg!(gettext(err_errmsg), err.msg);
                api_clear_error(&raw mut err);
            }
            debug_assert!(printed_errmsg);
            prev_prompt_errors.set(prev_prompt_errors.get() + 1);
            (*ccline_colors).colors.size = 0;
            redrawcmdline();
            ret = false;
        }

        // color_cmdline_end:
        debug_assert!(err.type_0 == kErrorTypeNone);
        if can_free_cb {
            callback_free(&raw mut color_cb);
        }
        xfree((*ccline_colors).cmdbuff as *mut ::core::ffi::c_void);
        // Errors' "output" is cached just as well as regular results.
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
        ret
    }
}
