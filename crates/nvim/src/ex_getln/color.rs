//! `'cmdline highlighting'`: the callback that colours the line.
//!
//! [`color_cmdline`] calls whatever the `input()` caller or `:` gave it,
//! validates the returned list of `[start, end, group]` chunks and caches the
//! answer against the line it was computed for.  [`color_expr_cmdline`] is
//! the built-in colouring of `=` expressions.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::guard::Suppress;
use crate::smsg_c;
use crate::types::{NUL, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VarLock, kErrorTypeNone};

/// Colour a `=` expression command line with the Vimscript expression parser,
/// filling the gaps the parser leaves uncoloured with `hl_id` 0.
pub(crate) unsafe fn color_expr_cmdline(
    colored_ccline: Cc,
    ret_ccline_colors: *mut ColoredCmdline,
) {
    unsafe {
        let mut parser_lines: [ParserLine; 2] = [
            ParserLine {
                data: colored_ccline.text(),
                size: strlen(colored_ccline.text()),
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
        (*ret_ccline_colors).reserve_chunks(colors.size);

        let mut prev_end: size_t = 0;
        let mut i: size_t = 0;
        while i < colors.size {
            let chunk: ParserHighlightChunk = *colors.items.add(i);
            debug_assert!(chunk.start.col < INT_MAX as size_t);
            debug_assert!(chunk.end_col < INT_MAX as size_t);
            if chunk.start.col != prev_end {
                (*ret_ccline_colors).push(CmdlineColorChunk {
                    start: prev_end as ::core::ffi::c_int,
                    end: chunk.start.col as ::core::ffi::c_int,
                    hl_id: 0,
                });
            }
            (*ret_ccline_colors).push(CmdlineColorChunk {
                start: chunk.start.col as ::core::ffi::c_int,
                end: chunk.end_col as ::core::ffi::c_int,
                hl_id: syn_name2id(chunk.group),
            });
            prev_end = chunk.end_col;
            i += 1;
        }
        if prev_end < colored_ccline.len() as size_t {
            (*ret_ccline_colors).push(CmdlineColorChunk {
                start: prev_end as ::core::ffi::c_int,
                end: colored_ccline.len(),
                hl_id: 0,
            });
        }

        // C's `kvi_destroy`: only free once the kvec has spilled to the heap.
        if colors.items != colors.init_array.as_mut_ptr() {
            xfree(colors.items as *mut ::core::ffi::c_void);
            colors.items = ::core::ptr::null_mut::<ParserHighlightChunk>();
        }
    }
}

/// Which of upstream's two labels `color_cmdline`'s body leaves by.
///
/// Not a `Result`: neither answer carries anything, and "the callback had
/// nothing to colour" is not an error — it takes the same exit as a full set
/// of chunks.
enum Label {
    /// `color_cmdline_end`: keep whatever chunks were pushed.
    End,
    /// `color_cmdline_error`: the callback failed or answered something
    /// unusable, and every arm that reaches here has already printed why.
    /// The chunks are thrown away and the line redrawn.
    Error,
}

/// Colour the command line, through the user's callback where there is one.
///
/// `colored_ccline` is also the cache: when its `prompt_id` and `cmdbuff`
/// still match what `last_colors` was computed from, this does nothing.  The
/// whole line is always coloured.
///
/// Answers true if [`super::draw::draw_cmdline`] may proceed, false if there
/// is nothing for it to do.
pub(crate) unsafe fn color_cmdline(colored_ccline: Cc) -> bool {
    let mut numbuf = NumBuf::new();
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
        let ccline_colors: *mut ColoredCmdline = &raw mut (*colored_ccline.raw()).last_colors;

        // Is the result of the previous call still valid?
        if (*ccline_colors).is_current(colored_ccline.prompt_id, colored_ccline.bytes()) {
            return ret;
        }

        (*ccline_colors).clear_chunks();

        if !colored_ccline.in_use() || *colored_ccline.text() as ::core::ffi::c_int == NUL {
            // Nothing to do.
            (*ccline_colors).forget();
            return ret;
        }

        let mut arg_allocated = false;
        let mut arg = typval_T {
            v_type: VAR_STRING,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union {
                v_string: colored_ccline.text(),
            },
        };
        let mut tv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_number: 0 },
        };

        // Both are C function-level statics. `prev_prompt_id` starts at
        // UINT_MAX so that the first prompt of a session, whatever its id,
        // counts as a new one.
        static prev_prompt_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(UINT_MAX);
        static prev_prompt_errors: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

        let mut color_cb = Callback {
            data: Callback_data {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut can_free_cb = false;
        let mut err: Error = ERROR_INIT;
        let mut err_errmsg = e_intern2.as_ptr();
        let mut dgc_ret = true;

        let outcome = 'body: {
            if colored_ccline.prompt_id != prev_prompt_id.get() {
                prev_prompt_errors.set(0);
                prev_prompt_id.set(colored_ccline.prompt_id);
            } else if prev_prompt_errors.get() >= MAX_CB_ERRORS {
                break 'body Label::End;
            }

            if colored_ccline.highlight_callback.type_0 != kCallbackNone {
                // Currently this should only happen while processing input()
                // prompts.
                debug_assert!(colored_ccline.input_fn != 0);
                color_cb = colored_ccline.highlight_callback.clone();
            } else if colored_ccline.cmdfirstc == ':' as ::core::ffi::c_int {
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
            } else if colored_ccline.cmdfirstc == '=' as ::core::ffi::c_int {
                color_expr_cmdline(colored_ccline, ccline_colors);
            }
            if err.type_0 != kErrorTypeNone || !dgc_ret {
                break 'body Label::Error;
            }

            if color_cb.type_0 == kCallbackNone {
                break 'body Label::End;
            }
            if *colored_ccline.at(colored_ccline.len()) as ::core::ffi::c_int != NUL {
                arg_allocated = true;
                arg.vval.v_string = xmemdupz(
                    colored_ccline.text() as *const ::core::ffi::c_void,
                    colored_ccline.len() as size_t,
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
            let silenced = Suppress::messages();
            cbcall_ret = callback_call(&raw mut color_cb, 1, &raw mut arg, &raw mut tv);
            drop(silenced);
            msg_col.set(saved_msg_col);
            if got_int.get() {
                getln_interrupted_highlight.set(true);
            }
            try_leave(&raw mut tstate, &raw mut err);

            if err.type_0 != kErrorTypeNone || !cbcall_ret {
                break 'body Label::Error;
            }
            if tv.v_type != VAR_LIST {
                print_errmsg!(
                    c"%s".as_ptr(),
                    gettext(c"E5400: Callback should return list".as_ptr())
                );
                break 'body Label::Error;
            }
            if tv.vval.v_list.is_null() {
                break 'body Label::End;
            }

            let mut prev_end: varnumber_T = 0;
            let mut i: ::core::ffi::c_int = 0;
            let mut li: *const listitem_T = (*tv.vval.v_list).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type != VAR_LIST {
                    print_errmsg!(gettext(c"E5401: List item %i is not a List".as_ptr()), i);
                    break 'body Label::Error;
                }
                let l: *const list_T = (*li).li_tv.vval.v_list;
                if tv_list_len(l) != 3 {
                    print_errmsg!(
                        gettext(c"E5402: List item %i has incorrect length: %d /= 3".as_ptr()),
                        i,
                        tv_list_len(l)
                    );
                    break 'body Label::Error;
                }

                let mut error = false;
                let start = tv_get_number_chk(&raw mut (*tv_list_first(l)).li_tv, &raw mut error);
                if error {
                    break 'body Label::Error;
                } else if !(prev_end <= start && start < colored_ccline.len() as varnumber_T) {
                    print_errmsg!(
                        gettext(c"E5403: Chunk %i start %ld not in range [%ld, %i)".as_ptr()),
                        i,
                        start,
                        prev_end,
                        colored_ccline.len()
                    );
                    break 'body Label::Error;
                } else if utf8len_tab_zero
                    [*colored_ccline.at(start as ::core::ffi::c_int) as uint8_t as usize]
                    == 0
                {
                    print_errmsg!(
                        gettext(c"E5405: Chunk %i start %ld splits multibyte character".as_ptr()),
                        i,
                        start
                    );
                    break 'body Label::Error;
                }

                if start != prev_end {
                    (*ccline_colors).push(CmdlineColorChunk {
                        start: prev_end as ::core::ffi::c_int,
                        end: start as ::core::ffi::c_int,
                        hl_id: 0,
                    });
                }

                let end = tv_get_number_chk(
                    &raw mut (*(*tv_list_first(l)).li_next).li_tv,
                    &raw mut error,
                );
                if error {
                    break 'body Label::Error;
                } else if !(start < end && end <= colored_ccline.len() as varnumber_T) {
                    print_errmsg!(
                        gettext(c"E5404: Chunk %i end %ld not in range (%ld, %i]".as_ptr()),
                        i,
                        end,
                        start,
                        colored_ccline.len()
                    );
                    break 'body Label::Error;
                } else if end < colored_ccline.len() as varnumber_T
                    && utf8len_tab_zero
                        [*colored_ccline.at(end as ::core::ffi::c_int) as uint8_t as usize]
                        == 0
                {
                    print_errmsg!(
                        gettext(c"E5406: Chunk %i end %ld splits multibyte character".as_ptr()),
                        i,
                        end
                    );
                    break 'body Label::Error;
                }

                prev_end = end;
                let group = numbuf.string_chk(&raw mut (*tv_list_last(l)).li_tv);
                if group.is_null() {
                    break 'body Label::Error;
                }
                (*ccline_colors).push(CmdlineColorChunk {
                    start: start as ::core::ffi::c_int,
                    end: end as ::core::ffi::c_int,
                    hl_id: syn_name2id(group),
                });
                i += 1;
                li = (*li).li_next;
            }

            if prev_end < colored_ccline.len() as varnumber_T {
                (*ccline_colors).push(CmdlineColorChunk {
                    start: prev_end as ::core::ffi::c_int,
                    end: colored_ccline.len(),
                    hl_id: 0,
                });
            }
            prev_prompt_errors.set(0);
            Label::End
        };

        // color_cmdline_error:
        if matches!(outcome, Label::Error) {
            if err.type_0 != kErrorTypeNone {
                print_errmsg!(gettext(err_errmsg), err.msg);
                api_clear_error(&raw mut err);
            }
            debug_assert!(printed_errmsg);
            prev_prompt_errors.set(prev_prompt_errors.get() + 1);
            (*ccline_colors).clear_chunks();
            redrawcmdline();
            ret = false;
        }

        // color_cmdline_end:
        debug_assert!(err.type_0 == kErrorTypeNone);
        if can_free_cb {
            callback_free(&raw mut color_cb);
        }
        // Errors' "output" is cached just as well as regular results.
        //
        // Which text is cached is C's choice kept: the copy `arg` was given
        // when one was made -- the line as the callback saw it, which the
        // callback may since have edited -- and the line itself otherwise.
        // C took ownership of `arg`'s copy here; the cache owns its bytes
        // now, so the copy is released instead.
        let id = colored_ccline.prompt_id;
        if arg_allocated {
            let s = arg.vval.v_string;
            (*ccline_colors).remember(id, ::core::slice::from_raw_parts(s, strlen(s)));
            xfree(s as *mut ::core::ffi::c_void);
        } else {
            (*ccline_colors).remember(id, colored_ccline.bytes());
        }
        tv_clear(&raw mut tv);
        ret
    }
}
