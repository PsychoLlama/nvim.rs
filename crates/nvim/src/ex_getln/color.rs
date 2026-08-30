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
use crate::message_fmt::msg_cstr;
use crate::os::cshim::gettext_ptr;
use crate::tr_plural;
use crate::types::{NUL, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VarLock};

/// Colour a `=` expression command line with the Vimscript expression parser,
/// filling the gaps the parser leaves uncoloured with `hl_id` 0.
pub(crate) unsafe fn color_expr_cmdline(
    colored_ccline: Cc,
    ret_ccline_colors: *mut ColoredCmdline,
) {
    let mut parser_lines: [ParserLine; 2] = [
        ParserLine {
            data: colored_ccline.text(),
            size: unsafe { strlen(colored_ccline.text()) },
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

    let mut pstate: ParserState = unsafe { ::core::mem::zeroed() };
    unsafe {
        viml_parser_init(
            &raw mut pstate,
            Some(parser_simple_get_line),
            &raw mut plines_p as *mut ::core::ffi::c_void,
            &raw mut colors,
        )
    };
    let east: ExprAST =
        unsafe { viml_pexpr_parse(&raw mut pstate, kExprFlagsDisallowEOC as ::core::ffi::c_int) };
    unsafe { viml_pexpr_free_ast(east) };
    viml_parser_destroy(&mut pstate);

    // C's `kv_resize`: reserve exactly what the parser produced. The
    // gap-filling chunks below may still push past it.
    // SAFETY: the command line's own chunk list, taken above.
    unsafe { (*ret_ccline_colors).reserve_chunks(colors.size) };

    let mut prev_end: size_t = 0;
    let mut i: size_t = 0;
    while i < colors.size {
        let chunk: ParserHighlightChunk = unsafe { *colors.items.add(i) };
        debug_assert!(chunk.start.col < INT_MAX as size_t);
        debug_assert!(chunk.end_col < INT_MAX as size_t);
        if chunk.start.col != prev_end {
            let coloured = CmdlineColorChunk {
                start: prev_end as ::core::ffi::c_int,
                end: chunk.start.col as ::core::ffi::c_int,
                hl_id: 0,
            };
            // SAFETY: the command line's own chunk list, taken above.
            unsafe { (*ret_ccline_colors).push(coloured) };
        }
        let coloured = CmdlineColorChunk {
            start: chunk.start.col as ::core::ffi::c_int,
            end: chunk.end_col as ::core::ffi::c_int,
            hl_id: unsafe { syn_name2id(chunk.group) },
        };
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ret_ccline_colors).push(coloured) };
        prev_end = chunk.end_col;
        i += 1;
    }
    if prev_end < colored_ccline.len() as size_t {
        let coloured = CmdlineColorChunk {
            start: prev_end as ::core::ffi::c_int,
            end: colored_ccline.len(),
            hl_id: 0,
        };
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ret_ccline_colors).push(coloured) };
    }

    // C's `kvi_destroy`: only free once the kvec has spilled to the heap.
    if colors.items != colors.init_array.as_mut_ptr() {
        unsafe { xfree(colors.items as *mut ::core::ffi::c_void) };
        colors.items = ::core::ptr::null_mut::<ParserHighlightChunk>();
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
    let mut printed_errmsg = false;

    // C's `PRINT_ERRMSG`: an error that scrolls the command line away.
    // A macro rather than a closure so the message stays a literal.
    macro_rules! print_errmsg {
        ($($arg:tt)*) => {{
            msg_scroll.set(1);
            // SAFETY: a message call on the main thread.
            unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
            let _: bool = $crate::smsg!(HLF_E, $($arg)*);
            printed_errmsg = true;
        }};
    }

    let mut ret = true;
    // SAFETY: `Cc::raw` is the live command line, and the address of one of
    // its fields is its own plus a constant.
    let ccline_colors: *mut ColoredCmdline =
        unsafe { &raw mut (*colored_ccline.raw()).last_colors };

    // Is the result of the previous call still valid?
    let (id, text) = (colored_ccline.prompt_id, colored_ccline.bytes());
    // SAFETY: the command line's own chunk list, taken above.
    if unsafe { (*ccline_colors).is_current(id, text) } {
        return ret;
    }

    // SAFETY: the command line's own chunk list, taken above.
    unsafe { (*ccline_colors).clear_chunks() };

    if !colored_ccline.in_use() || unsafe { *colored_ccline.text() } as ::core::ffi::c_int == NUL {
        // Nothing to do.
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ccline_colors).forget() };
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
            unsafe { try_enter(&raw mut tstate) };
            err_errmsg = c"E5408: Unable to get g:Nvim_color_cmdline callback: %s".as_ptr();
            let key = c"Nvim_color_cmdline";
            dgc_ret = unsafe {
                tv_dict_get_callback(
                    get_globvar_dict(),
                    key.as_ptr(),
                    key.count_bytes() as ptrdiff_t,
                    &raw mut color_cb,
                )
            };
            unsafe { try_leave(&raw mut tstate, &mut err) };
            can_free_cb = true;
        } else if colored_ccline.cmdfirstc == '=' as ::core::ffi::c_int {
            unsafe { color_expr_cmdline(colored_ccline, ccline_colors) };
        }
        if err.is_set() || !dgc_ret {
            break 'body Label::Error;
        }

        if color_cb.type_0 == kCallbackNone {
            break 'body Label::End;
        }
        if unsafe { *colored_ccline.at(colored_ccline.len()) } as ::core::ffi::c_int != NUL {
            arg_allocated = true;
            arg.vval.v_string = unsafe {
                xmemdupz(
                    colored_ccline.text() as *const ::core::ffi::c_void,
                    colored_ccline.len() as size_t,
                )
            } as *mut ::core::ffi::c_char;
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
        unsafe { try_enter(&raw mut tstate) };
        err_errmsg = c"E5407: Callback has thrown an exception: %s".as_ptr();
        let saved_msg_col = msg_col.get();
        let silenced = Suppress::messages();
        cbcall_ret = unsafe { callback_call(&raw mut color_cb, 1, &raw mut arg, &raw mut tv) };
        drop(silenced);
        msg_col.set(saved_msg_col);
        if got_int.get() {
            getln_interrupted_highlight.set(true);
        }
        unsafe { try_leave(&raw mut tstate, &mut err) };

        if err.is_set() || !cbcall_ret {
            break 'body Label::Error;
        }
        if tv.v_type != VAR_LIST {
            print_errmsg!("E5400: Callback should return list");
            break 'body Label::Error;
        }
        if unsafe { tv.vval.v_list }.is_null() {
            break 'body Label::End;
        }

        let mut prev_end: varnumber_T = 0;
        let mut i: ::core::ffi::c_int = 0;
        let mut li: *const listitem_T = unsafe { (*tv.vval.v_list).lv_first };
        while !li.is_null() {
            if unsafe { (*li).li_tv.v_type } != VAR_LIST {
                print_errmsg!("E5401: List item {i} is not a List");
                break 'body Label::Error;
            }
            let l: *const list_T = unsafe { (*li).li_tv.vval.v_list };
            if unsafe { tv_list_len(l) } != 3 {
                // SAFETY: `l` is the list item just checked.
                let len = unsafe { tv_list_len(l) };
                print_errmsg!("E5402: List item {i} has incorrect length: {len} /= 3");
                break 'body Label::Error;
            }

            let mut error = false;
            let start =
                unsafe { tv_get_number_chk(&raw mut (*tv_list_first(l)).li_tv, &raw mut error) };
            if error {
                break 'body Label::Error;
            } else if !(prev_end <= start && start < colored_ccline.len() as varnumber_T) {
                let end = colored_ccline.len();
                print_errmsg!("E5403: Chunk {i} start {start} not in range [{prev_end}, {end})");
                break 'body Label::Error;
            } else if utf8len_tab_zero
                [unsafe { *colored_ccline.at(start as ::core::ffi::c_int) } as uint8_t as usize]
                == 0
            {
                print_errmsg!("E5405: Chunk {i} start {start} splits multibyte character");
                break 'body Label::Error;
            }

            if start != prev_end {
                let coloured = CmdlineColorChunk {
                    start: prev_end as ::core::ffi::c_int,
                    end: start as ::core::ffi::c_int,
                    hl_id: 0,
                };
                // SAFETY: the command line's own chunk list, taken above.
                unsafe { (*ccline_colors).push(coloured) };
            }

            let end = unsafe {
                tv_get_number_chk(
                    &raw mut (*(*tv_list_first(l)).li_next).li_tv,
                    &raw mut error,
                )
            };
            if error {
                break 'body Label::Error;
            } else if !(start < end && end <= colored_ccline.len() as varnumber_T) {
                let limit = colored_ccline.len();
                print_errmsg!("E5404: Chunk {i} end {end} not in range ({start}, {limit}]");
                break 'body Label::Error;
            } else if end < colored_ccline.len() as varnumber_T
                && utf8len_tab_zero
                    [unsafe { *colored_ccline.at(end as ::core::ffi::c_int) } as uint8_t as usize]
                    == 0
            {
                print_errmsg!("E5406: Chunk {i} end {end} splits multibyte character");
                break 'body Label::Error;
            }

            prev_end = end;
            let group = unsafe { numbuf.string_chk(&raw mut (*tv_list_last(l)).li_tv) };
            if group.is_null() {
                break 'body Label::Error;
            }
            let coloured = CmdlineColorChunk {
                start: start as ::core::ffi::c_int,
                end: end as ::core::ffi::c_int,
                hl_id: unsafe { syn_name2id(group) },
            };
            // SAFETY: the command line's own chunk list, taken above.
            unsafe { (*ccline_colors).push(coloured) };
            i += 1;
            li = unsafe { (*li).li_next };
        }

        if prev_end < colored_ccline.len() as varnumber_T {
            let coloured = CmdlineColorChunk {
                start: prev_end as ::core::ffi::c_int,
                end: colored_ccline.len(),
                hl_id: 0,
            };
            // SAFETY: the command line's own chunk list, taken above.
            unsafe { (*ccline_colors).push(coloured) };
        }
        prev_prompt_errors.set(0);
        Label::End
    };

    // color_cmdline_error:
    if matches!(outcome, Label::Error) {
        if err.is_set() {
            let why = msg_cstr(err.message_or_empty());
            // SAFETY: `err_errmsg` is a static NUL-terminated message.
            let template = unsafe { gettext_ptr(err_errmsg) };
            print_errmsg!("{}", tr_plural!(template, why));
            err.clear();
        }
        debug_assert!(printed_errmsg);
        prev_prompt_errors.set(prev_prompt_errors.get() + 1);
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ccline_colors).clear_chunks() };
        unsafe { redrawcmdline() };
        ret = false;
    }

    // color_cmdline_end:
    debug_assert!(!err.is_set());
    if can_free_cb {
        unsafe { callback_free(&raw mut color_cb) };
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
        let s = unsafe { arg.vval.v_string };
        // SAFETY: `arg` holds this frame's own NUL-terminated copy.
        let text = unsafe { ::core::slice::from_raw_parts(s, strlen(s)) };
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ccline_colors).remember(id, text) };
        unsafe { xfree(s as *mut ::core::ffi::c_void) };
    } else {
        let text = colored_ccline.bytes();
        // SAFETY: the command line's own chunk list, taken above.
        unsafe { (*ccline_colors).remember(id, text) };
    }
    unsafe { tv_clear(&raw mut tv) };
    ret
}
