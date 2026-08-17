//! `nvim_eval_statusline()`: rendering a statusline expression.
//!
//! The longest function in the module, because a statusline is evaluated
//! against a *window* with a fill character, a maximum width and an
//! optional statuscolumn line number, and because the `highlights` option
//! makes it report every group boundary in the result as well as the text.
//! `nvim__complete_set` shares the window plumbing.
//!
//! The expansion itself goes through [`StlJob`], the same wrapper the
//! drawing side uses -- with two differences that are the whole reason this
//! entry point exists as a separate arm of `build_stl_str_hl`: the format is
//! [`Fmt::borrowed`] (it is an API argument, not an option something can
//! `:set` underneath), and the option index is `kOptInvalid`, which makes
//! `use_sandbox` unconditionally false here.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::api::private::helpers::has_key;
use crate::statusline::{
    Fmt, HlDest, HlRuns, SIGN_SHOW_MAX, StlJob, fillchar_status_of, push, put, stl_is_global,
    win_opt,
};
use crate::winlayer::Win;

/// Everything `nvim_eval_statusline()` needs to have settled before it can
/// expand anything, and the two highlight ids the `'statuscolumn'` arm
/// leaves behind for the `highlights` answer.
struct Context {
    win: Win,
    fillchar: schar_T,
    maxwidth: c_int,
    /// The line `use_statuscol_lnum` named, or zero for "not a status
    /// column".
    statuscol_lnum: c_int,
    /// The group the whole column defaults to, and the one a `%s` item
    /// combines with.
    stc_hl_id: c_int,
    scl_hl_id: c_int,
}

pub unsafe extern "C" fn nvim_eval_statusline(
    str: String_0,
    opts: *mut KeyDict_eval_statusline,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    let empty = Dict {
        size: 0,
        capacity: 0,
        items: ptr::null_mut::<KeyValuePair>(),
    };
    // SAFETY: the API dispatcher's own frame; `str` is a checked string.
    let opts = unsafe { &mut *opts };
    // `%!` is an expression producing the real format, so there is nothing
    // to check until it has been evaluated.
    // SAFETY: `str` holds `size` readable bytes.
    let named_expr = str.size >= 2
        && unsafe { *str.data == b'%' as c_char && *str.data.add(1) == b'!' as c_char };
    if !named_expr {
        // SAFETY: `str.data` is NUL-terminated, and the message is the
        // checker's own static text.
        let errmsg = unsafe { check_stl_option(str.data) };
        if !errmsg.is_null() {
            // SAFETY: the caller's error slot.
            unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), errmsg) };
            return empty;
        }
    }

    let mut statuscol = statuscol_T::default();
    let mut sattrs = [SignTextAttrs {
        text: [0; 2],
        hl_id: 0,
    }; SIGN_SHOW_MAX as usize];
    // SAFETY: the caller's error slot and the editor's own window list.
    let Some(ctx) = (unsafe { Context::of(opts, err, &mut statuscol, &mut sattrs) }) else {
        return empty;
    };

    // SAFETY: an arena the caller owns, whose allocations outlive the reply.
    let (mut result, buf) = unsafe {
        (
            arena_dict(arena, 3),
            arena_alloc(arena, MAXPATHL as size_t, false).cast::<c_char>(),
        )
    };
    // SAFETY: `buf` is the `MAXPATHL` allocation just made, and is not
    // `NameBuff`.
    let out = unsafe { ::core::slice::from_raw_parts_mut(buf, MAXPATHL as usize) };

    // Temporarily reset 'cursorbind' to prevent side effects from moving the
    // cursor away and back.
    let mut win = ctx.win;
    let crb_save = win.w_onebuf_opt.wo_crb;
    win.w_onebuf_opt.wo_crb = false_0;
    let job = StlJob {
        win,
        // The API's own string: nothing can free it under the expander, so
        // unlike the drawing side this needs no private copy.
        // SAFETY: a checked API string.
        fmt: unsafe { Fmt::borrowed(str.data) },
        opt: (kOptInvalid, 0),
        fillchar: ctx.fillchar,
        maxwidth: ctx.maxwidth,
        hl: if opts.highlights {
            HlDest::Runs
        } else {
            HlDest::Discard
        },
        want_clicks: false,
        stcp: (ctx.statuscol_lnum != 0).then_some(&mut statuscol),
    };
    // SAFETY: the expander re-enters the editor; nothing is held across it.
    let built = unsafe { job.run(out) };
    put(
        &mut result,
        c"width",
        Object::integer(built.width as Integer),
    );
    win.w_onebuf_opt.wo_crb = crb_save;

    if let Some(runs) = built.hl {
        let hl = highlight_dicts(&ctx, opts, arena, buf, runs, built.hl_len);
        put(&mut result, c"highlights", Object::array(hl));
    }
    // SAFETY: `buf` is NUL-terminated by the expander and lives in the
    // arena, which outlives the reply.
    put(
        &mut result,
        c"str",
        Object::string(unsafe { cstr_as_string(buf) }),
    );
    result
}

impl Context {
    /// Validate the options and settle the window, the fill character and
    /// the width. Answers `None` with `err` set when something is wrong.
    ///
    /// # Safety
    /// `err` must be the caller's error slot, and `statuscol`/`sattrs` must
    /// outlive the expansion.
    unsafe fn of(
        opts: &KeyDict_eval_statusline,
        err: *mut Error,
        statuscol: &mut statuscol_T,
        sattrs: &mut [SignTextAttrs; SIGN_SHOW_MAX as usize],
    ) -> Option<Context> {
        let mut fillchar = 0 as schar_T;
        if has_key(
            opts.is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__fillchar,
        ) {
            // A fill character is one whole character, however wide.
            // SAFETY: a checked API string.
            let single = unsafe {
                *opts.fillchar.data != 0
                    && utfc_ptr2len(opts.fillchar.data) as size_t == opts.fillchar.size
            };
            if !single {
                // SAFETY: the caller's error slot.
                unsafe {
                    api_err_exp(
                        err,
                        c"fillchar".as_ptr(),
                        c"single character".as_ptr(),
                        ptr::null(),
                    )
                };
                return None;
            }
            let mut c = 0;
            // SAFETY: as above. TODO(bfredl): actually check c is single width.
            fillchar = unsafe { utfc_ptr2schar(opts.fillchar.data, &raw mut c) };
        }

        let mut use_bools = c_int::from(opts.use_winbar) + c_int::from(opts.use_tabline);
        // SAFETY: `curwin` is live, and the handle lookup answers a live
        // window or null.
        let wp = unsafe {
            if opts.use_tabline {
                curwin.get()
            } else {
                find_window_by_handle(opts.winid, err)
            }
        };
        // SAFETY: null or a live window.
        let win = unsafe { win_opt(wp) }.or_else(|| {
            // SAFETY: the caller's error slot; the lookup may already have
            // set one, which upstream overwrites with this.
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"unknown winid %d".as_ptr(),
                    opts.winid,
                )
            };
            None
        })?;

        let mut statuscol_lnum = 0;
        if has_key(
            opts.is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum,
        ) {
            statuscol_lnum = opts.use_statuscol_lnum as c_int;
            if !(statuscol_lnum > 0 && statuscol_lnum as linenr_T <= win.buffer().line_count()) {
                // SAFETY: the caller's error slot.
                unsafe {
                    api_err_invalid(
                        err,
                        c"use_statuscol_lnum".as_ptr(),
                        c"out of range".as_ptr(),
                        0,
                        false,
                    );
                }
                return None;
            }
            use_bools += 1;
        }
        if use_bools > 1 {
            const E: &CStr =
                c"Can only use one of 'use_winbar', 'use_tabline' and 'use_statuscol_lnum'";
            // SAFETY: the caller's error slot and a static message.
            unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), E.as_ptr()) };
            return None;
        }

        let (mut stc_hl_id, mut scl_hl_id) = (0, 0);
        if statuscol_lnum != 0 {
            // SAFETY: a live window and a line of its buffer.
            (stc_hl_id, scl_hl_id) =
                unsafe { statuscol_state(win, statuscol_lnum, statuscol, sattrs) };
        } else if fillchar == 0 && !opts.use_tabline {
            fillchar = if opts.use_winbar {
                win.w_p_fcs_chars.wbr
            } else {
                fillchar_status_of(win).1
            };
        }

        let maxwidth = if has_key(
            opts.is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__maxwidth,
        ) {
            opts.maxwidth as c_int
        } else if statuscol_lnum != 0 {
            // SAFETY: a live window.
            unsafe { win_col_off(win.raw()) }
        } else if opts.use_tabline || (!opts.use_winbar && stl_is_global()) {
            Columns.get()
        } else {
            win.w_width
        };

        Some(Context {
            win,
            fillchar,
            maxwidth,
            statuscol_lnum,
            stc_hl_id,
            scl_hl_id,
        })
    }
}

/// Fill in the `'statuscolumn'` state for line `lnum` -- the signs, the fold
/// and the cursor-line highlights -- and set `v:lnum`/`v:relnum`/`v:virtnum`
/// as the drawing side would have.
///
/// Answers the group the column defaults to and the one a `%s` item
/// combines with.
///
/// # Safety
/// `win` must be live, `lnum` one of its buffer's lines, and `statuscol`
/// and `sattrs` must outlive the expansion.
unsafe fn statuscol_state(
    win: Win,
    lnum: c_int,
    statuscol: &mut statuscol_T,
    sattrs: &mut [SignTextAttrs; SIGN_SHOW_MAX as usize],
) -> (c_int, c_int) {
    let lnum = lnum as linenr_T;
    let (mut line_id, mut cul_id, mut num_id) = (0, 0, 0);
    let mut cursorline_fi = foldinfo_T::default();
    // SAFETY: the caller's promise; the three ids and the sign array are
    // out-parameters of this frame.
    unsafe {
        decor_redraw_signs(
            win.raw(),
            win.buffer().raw(),
            lnum - 1,
            sattrs.as_mut_ptr(),
            &raw mut line_id,
            &raw mut cul_id,
            &raw mut num_id,
        );
    }
    statuscol.sattrs = sattrs.as_mut_ptr();
    // SAFETY: as above.
    let (foldinfo, on_cursorline) = unsafe {
        let foldinfo = fold_info(win.raw(), lnum);
        win_update_cursorline(win.raw(), &raw mut cursorline_fi);
        (foldinfo, use_cursor_line_highlight(win.raw(), lnum))
    };
    statuscol.foldinfo = foldinfo;
    statuscol.sign_cul_id = if on_cursorline { cul_id } else { 0 };

    let stc_hl_id = if num_id != 0 {
        num_id
    } else if on_cursorline {
        HLF_CLN
    } else if win.w_onebuf_opt.wo_rnu != 0 {
        // 'relativenumber' colours the lines above and below differently.
        if lnum < win.w_cursor.lnum {
            HLF_LNA
        } else {
            HLF_LNB
        }
    } else {
        HLF_N
    };
    // SAFETY: three plain number variables of the editor's own.
    unsafe {
        set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
        let rel = labs(get_cursor_rel_lnum(win.raw(), lnum) as ::core::ffi::c_long);
        set_vim_var_nr(VV_RELNUM, rel as varnumber_T);
        set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
    }
    (stc_hl_id, if on_cursorline { HLF_CLS } else { HLF_SC })
}

/// The `highlights` answer: one dictionary per group boundary the expander
/// recorded, plus a leading one when the first character carries none.
fn highlight_dicts(
    ctx: &Context,
    opts: &KeyDict_eval_statusline,
    arena: *mut Arena,
    buf: *const c_char,
    runs: HlRuns,
    runs_len: size_t,
) -> Array {
    let mut values = arena_array(arena, runs_len + 1);
    // For the tab line the default group belongs to no window.
    let ctxwin = if opts.use_tabline {
        ptr::null_mut()
    } else {
        ctx.win.raw()
    };
    let dfltname = get_default_stl_hl(ctxwin, opts.use_winbar, ctx.stc_hl_id);

    // If the first character has no highlight of its own, the default one
    // opens the list.
    if runs.first_start().is_none_or(|start| !ptr::eq(start, buf)) {
        let mut info = arena_dict(arena, 3);
        put(&mut info, c"start", Object::integer(0));
        // SAFETY: a static group name.
        put(
            &mut info,
            c"group",
            Object::string(unsafe { cstr_as_string(dfltname) }),
        );
        let mut groups = arena_array(arena, 1);
        // SAFETY: as above.
        push(
            &mut groups,
            Object::string(unsafe { cstr_as_string(dfltname) }),
        );
        put(&mut info, c"groups", Object::array(groups));
        push(&mut values, Object::dict(info));
    }

    let mut user_group = [0 as c_char; 15]; // "User" + "2147483647" + NUL
    for run in runs.iter() {
        let grpname = if run.userhl == 0 {
            get_default_stl_hl(ctxwin, opts.use_winbar, ctx.stc_hl_id)
        } else if run.userhl < 0 {
            syn_id2name(-run.userhl)
        } else {
            let (out, room, fmt) = (
                user_group.as_mut_ptr(),
                user_group.len(),
                c"User%d".as_ptr(),
            );
            // SAFETY: a local buffer with room for the widest `%d`, and an
            // arena copy of it that outlives the reply.
            unsafe {
                snprintf(out, room, fmt, run.userhl);
                arena_strdup(arena, out)
            }
        };
        // The sign column's own group combines with the sign's highlight,
        // the fold column's with nothing, everything else with the default.
        // These are POINTER comparisons upstream, and the group names are
        // interned, so a name equal by value is still a second entry.
        let combine = if run.item == STL_SIGNCOL {
            syn_id2name(ctx.scl_hl_id)
        } else if run.item == STL_FOLDCOL {
            grpname
        } else {
            dfltname
        };

        let mut info = arena_dict(arena, 3);
        // SAFETY: `run.start` is a position in `buf`.
        let start = unsafe { run.start.offset_from(buf) };
        put(&mut info, c"start", Object::integer(start as Integer));
        // SAFETY: both are NUL-terminated group names outliving the reply.
        let (grp, comb) = unsafe { (cstr_as_string(grpname), cstr_as_string(combine)) };
        put(&mut info, c"group", Object::string(grp));
        let mut groups = arena_array(arena, 1 + size_t::from(!ptr::eq(combine, grpname)));
        if !ptr::eq(combine, grpname) {
            push(&mut groups, Object::string(comb));
        }
        push(&mut groups, Object::string(grp));
        put(&mut info, c"groups", Object::array(groups));
        push(&mut values, Object::dict(info));
    }
    values
}

pub unsafe extern "C" fn nvim__complete_set(
    index: Integer,
    opts: *mut KeyDict_complete_set,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    let mut rv = arena_dict(arena, 2);
    // SAFETY: the API dispatcher's own frame.
    let opts = unsafe { &*opts };
    // SAFETY: reads the 'completeopt' flags.
    if unsafe { get_cot_flags() } & kOptCotFlagPopup as c_int as ::core::ffi::c_uint == 0 {
        // SAFETY: the caller's error slot and a static message.
        unsafe {
            api_set_error(
                err,
                kErrorTypeException,
                c"completeopt option does not include popup".as_ptr(),
            );
        }
        return rv;
    }
    if has_key(opts.is_set__complete_set_, KEYSET_OPTIDX_complete_set__info) {
        // SAFETY: a checked API string; the answer is null or a live window.
        let win = unsafe { win_opt(pum_set_info(index as c_int, opts.info.data)) };
        if let Some(win) = win {
            put(&mut rv, c"winid", Object::window(win.handle));
            put(&mut rv, c"bufnr", Object::buffer(win.buffer().handle));
        }
    }
    rv
}
