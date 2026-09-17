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
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{Bad, err_expected, err_invalid};
use crate::api_error;
use crate::statusline::{
    Fmt, HlDest, HlRuns, SIGN_SHOW_MAX, StlJob, fillchar_status_of, push, put, stl_is_global,
};
use crate::types::{MAXPATHL, OptionSetFlags, StlOpt};
use crate::winlayer::Win;

/// Everything `nvim_eval_statusline()` needs to have settled before it can
/// expand anything, and the two highlight ids the `'statuscolumn'` arm
/// leaves behind for the `highlights` answer.
struct Context {
    win: Win,
    fillchar: ScreenChar,
    maxwidth: c_int,
    /// The line `use_statuscol_lnum` named, or zero for "not a status
    /// column".
    statuscol_lnum: c_int,
    /// The group the whole column defaults to, and the one a `%s` item
    /// combines with.
    stc_hl_id: c_int,
    scl_hl_id: c_int,
}

/// # Safety
///
/// `str` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `opts` must point at the `KeyDict_eval_statusline` the
/// dispatcher filled in, live for the call. `arena` must point at a live
/// arena, which the memory this answers with is taken from and must outlive.
pub unsafe fn nvim_eval_statusline(
    str: String_0,
    opts: *mut KeyDict_eval_statusline,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    let empty = ApiDict::EMPTY;
    // SAFETY: the API dispatcher's own frame; `str` is a checked string.
    let opts = unsafe { &mut *opts };
    // `%!` is an expression producing the real format, so there is nothing
    // to check until it has been evaluated.
    // SAFETY: `str` holds `size` readable bytes.
    let named_expr = str.len() >= 2
        && unsafe { *str.data() == b'%' as c_char && *str.data().add(1) == b'!' as c_char };
    if !named_expr {
        // SAFETY: `str.data` is NUL-terminated, and the message is the
        // checker's own static text.
        if let Err(errmsg) = unsafe { check_stl_option(str.data()) } {
            error = Error::validation(errmsg.as_cstr());
            return empty.reported(error);
        }
    }

    let mut statuscol = StatusCol::default();
    let mut sattrs = [SignTextAttrs {
        text: [0; 2],
        hl_id: 0,
    }; SIGN_SHOW_MAX as usize];
    let ctx = Context::of(opts, &mut statuscol, &mut sattrs)?;

    // SAFETY: an arena the caller owns, whose allocations outlive the reply.
    let (mut result, buf) = unsafe {
        (
            ApiDict::with_capacity(3),
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
    win.w_onebuf_opt.wo_crb = 0;
    let job = StlJob {
        win,
        // The API's own string: nothing can free it under the expander, so
        // unlike the drawing side this needs no private copy.
        // SAFETY: a checked API string.
        fmt: unsafe { Fmt::borrowed(str.data()) },
        opt: (kOptInvalid, OptionSetFlags::NONE),
        fillchar: ctx.fillchar,
        maxwidth: ctx.maxwidth,
        hl: if opts.highlights.unwrap_or(false) {
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
        Object::string(unsafe { cstr_to_string(buf) }),
    );
    result.reported(error)
}

impl Context {
    /// Validate the options and settle the window, the fill character and
    /// the width.
    fn of(
        opts: &KeyDict_eval_statusline,
        statuscol: &mut StatusCol,
        sattrs: &mut [SignTextAttrs; SIGN_SHOW_MAX as usize],
    ) -> Result<Context, Error> {
        let (use_winbar, use_tabline) = (
            opts.use_winbar.unwrap_or(false),
            opts.use_tabline.unwrap_or(false),
        );
        let mut fillchar = 0 as ScreenChar;
        if let Some(given) = opts.fillchar.as_ref() {
            // A fill character is one whole character, however wide.
            // SAFETY: a checked API string.
            let single = unsafe {
                *given.data() != 0 && utfc_ptr2len(given.data()) as size_t == given.len()
            };
            if !single {
                return Err(err_expected(c"fillchar", c"single character", None));
            }
            let mut c = 0;
            // SAFETY: as above. TODO(bfredl): actually check c is single width.
            fillchar = unsafe { utfc_ptr2schar(given.data(), &raw mut c) };
        }

        let mut use_bools = c_int::from(use_winbar) + c_int::from(use_tabline);
        let winid = opts.winid.unwrap_or(0);
        let win = if use_tabline {
            Win::current_or_none()
        } else {
            // The lookup's own refusal is thrown away: upstream overwrites
            // it with the message below.
            find_window_by_handle(winid).unwrap_or_default()
        };
        let Some(win) = win else {
            return Err(api_error!(kErrorTypeException, "unknown winid {winid}"));
        };

        let mut statuscol_lnum = 0;
        if let Some(lnum) = opts.use_statuscol_lnum {
            statuscol_lnum = lnum as c_int;
            if !(statuscol_lnum > 0 && statuscol_lnum as LineNr <= win.buffer().line_count()) {
                let key = c"use_statuscol_lnum".as_ptr();
                let why = c"out of range".as_ptr();
                // SAFETY: the names and values are NUL-terminated strings.
                // SAFETY: both are the caller's NUL-terminated strings.
                let (key, why) = unsafe { (cstr::at(key), cstr::at(why)) };
                return Err(err_invalid(key, Bad::Bare(why)));
            }
            use_bools += 1;
        }
        if use_bools > 1 {
            const E: &CStr =
                c"Can only use one of 'use_winbar', 'use_tabline' and 'use_statuscol_lnum'";
            return Err(Error::validation(E));
        }

        let (mut stc_hl_id, mut scl_hl_id) = (0, 0);
        if statuscol_lnum != 0 {
            (stc_hl_id, scl_hl_id) = statuscol_state(win, statuscol_lnum, statuscol, sattrs);
        } else if fillchar == 0 && !use_tabline {
            fillchar = if use_winbar {
                win.w_p_fcs_chars.wbr
            } else {
                fillchar_status_of(win).1
            };
        }

        let maxwidth = if let Some(given) = opts.maxwidth {
            given as c_int
        } else if statuscol_lnum != 0 {
            win.col_off()
        } else if use_tabline || (!use_winbar && stl_is_global()) {
            Columns.get()
        } else {
            win.w_width
        };

        Ok(Context {
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
fn statuscol_state(
    win: Win,
    lnum: c_int,
    statuscol: &mut StatusCol,
    sattrs: &mut [SignTextAttrs; SIGN_SHOW_MAX as usize],
) -> (c_int, c_int) {
    let lnum = lnum as LineNr;
    let (mut line_id, mut cul_id, mut num_id) = (0, 0, 0);
    let mut cursorline_fi = FoldInfo::default();
    let (buf, signs) = (win.buffer(), sattrs.as_mut_ptr());
    let ids = (&raw mut line_id, &raw mut cul_id, &raw mut num_id);
    // SAFETY: the caller's promise; the three ids `ids` names and the sign
    // array are out-parameters of this frame.
    unsafe { decor_redraw_signs(win, buf, lnum - 1, signs, ids.0, ids.1, ids.2) };
    statuscol.sattrs = sattrs.as_mut_ptr();
    // SAFETY: as above.
    let (foldinfo, on_cursorline) = unsafe {
        let foldinfo = fold_info(win, lnum);
        win_update_cursorline(win, &raw mut cursorline_fi);
        (foldinfo, use_cursor_line_highlight(win, lnum))
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
    set_vim_var_nr(Vv::Lnum, lnum as VarNumber);
    let rel = unsafe { labs(get_cursor_rel_lnum(win, lnum) as ::core::ffi::c_long) };
    set_vim_var_nr(Vv::Relnum, rel as VarNumber);
    set_vim_var_nr(Vv::Virtnum, 0 as VarNumber);
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
    let mut values = Array::with_capacity(runs_len + 1);
    // For the tab line the default group belongs to no window.
    let ctxwin = (!opts.use_tabline.unwrap_or(false)).then_some(ctx.win);
    let dfltname = get_default_stl_hl(ctxwin, opts.use_winbar.unwrap_or(false), ctx.stc_hl_id);

    // If the first character has no highlight of its own, the default one
    // opens the list.
    if runs.first_start().is_none_or(|start| !ptr::eq(start, buf)) {
        let mut info = ApiDict::with_capacity(3);
        put(&mut info, c"start", Object::integer(0));
        // SAFETY: a static group name.
        put(
            &mut info,
            c"group",
            Object::string(unsafe { cstr_to_string(dfltname) }),
        );
        let mut groups = Array::with_capacity(1);
        // SAFETY: as above.
        push(
            &mut groups,
            Object::string(unsafe { cstr_to_string(dfltname) }),
        );
        put(&mut info, c"groups", Object::array(groups));
        push(&mut values, Object::dict(info));
    }

    let mut user_group = [0 as c_char; 15]; // "User" + "2147483647" + NUL
    for run in runs.iter() {
        let grpname = if run.userhl == 0 {
            get_default_stl_hl(ctxwin, opts.use_winbar.unwrap_or(false), ctx.stc_hl_id)
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
            unsafe { snprintf(out, room, fmt, run.userhl) };
            unsafe { arena_strdup(arena, out) }
        };
        // The sign column's own group combines with the sign's highlight,
        // the fold column's with nothing, everything else with the default.
        // These are POINTER comparisons upstream, and the group names are
        // interned, so a name equal by value is still a second entry.
        let combine = if run.item == Some(StlOpt::SignCol) {
            syn_id2name(ctx.scl_hl_id)
        } else if run.item == Some(StlOpt::FoldCol) {
            grpname
        } else {
            dfltname
        };

        let mut info = ApiDict::with_capacity(3);
        // SAFETY: `run.start` is a position in `buf`.
        let start = unsafe { run.start.offset_from(buf) };
        put(&mut info, c"start", Object::integer(start as Integer));
        // SAFETY: both are NUL-terminated group names outliving the reply.
        let (grp, comb) = unsafe { (cstr_to_string(grpname), cstr_to_string(combine)) };
        put(&mut info, c"group", Object::string(grp.clone()));
        let mut groups = Array::with_capacity(1 + size_t::from(!ptr::eq(combine, grpname)));
        if !ptr::eq(combine, grpname) {
            push(&mut groups, Object::string(comb));
        }
        push(&mut groups, Object::string(grp));
        put(&mut info, c"groups", Object::array(groups));
        push(&mut values, Object::dict(info));
    }
    values
}

/// # Safety
///
/// `opts` must point at the `KeyDict_complete_set` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
// `nvim__complete_set` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__complete_set(
    index: Integer,
    opts: *mut KeyDict_complete_set,
) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    let mut rv = ApiDict::with_capacity(2);
    // SAFETY: the API dispatcher's own frame.
    let opts = unsafe { &*opts };
    if get_cot_flags() & kOptCotFlagPopup as c_int as ::core::ffi::c_uint == 0 {
        error = Error::exception(c"completeopt option does not include popup");
        return rv.reported(error);
    }
    if let Some(info) = opts.info.as_ref() {
        // SAFETY: a checked API string.
        let win = unsafe { pum_set_info(index as c_int, info.data()) };
        if let Some(win) = win {
            put(&mut rv, c"winid", Object::window(win.handle));
            put(&mut rv, c"bufnr", Object::buffer(win.buffer().handle));
        }
    }
    rv.reported(error)
}
