//! Counting the matches, for `[N/M]` and for `searchcount()`.
//!
//! [`update_search_stat`] does the counting — forwards from the top of the
//! buffer, giving up after a timeout or after `maxcount` matches — and
//! caches the answer against the buffer's changedtick so that repeating
//! the search is cheap. [`cmdline_search_stat`] renders it into the
//! message line; [`f_searchcount`] is the Vimscript view of the same
//! numbers.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::regexp::RE_LAST;
use crate::src::nvim::search::{SEARCH_KEEP, SEARCH_STAT_DEF_TIMEOUT};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// How much room `[>999/>999]` and its `W ` prefix need.
const STAT_BUF_LEN: usize = super::SEARCH_STAT_BUF_LEN as usize;

/// What `[N/M]` and `searchcount()` report.
#[derive(Clone, Copy, Default)]
struct Stat {
    /// Which match the cursor is on, 1-based; -1 when the count was
    /// interrupted, 0 when there is nothing to report.
    cur: c_int,
    /// How many matches there are.
    cnt: c_int,
    /// The match starts exactly at the position asked about.
    exact_match: bool,
    /// 0 when the count is complete, 1 when it timed out, 2 when it hit
    /// `maxcount`.
    incomplete: c_int,
    /// The `maxcount` the numbers were counted under.
    last_maxcount: c_int,
}

/// The last count, kept so that pressing `n` again does not recount.
///
/// `at` is the position the count was made from; an all-zero `at` means
/// nothing has been counted yet. The pattern, the buffer and the buffer's
/// changedtick are remembered beside the numbers — if any of them moved,
/// the numbers are stale and the count starts again.
#[derive(Clone, Copy)]
struct Counted {
    at: pos_T,
    cur: c_int,
    cnt: c_int,
    exact_match: bool,
    incomplete: c_int,
    /// An owned copy of the pattern the numbers are for.
    pat: *mut c_char,
    patlen: size_t,
    chgtick: c_int,
    buf: *mut buf_T,
}

impl Counted {
    const NONE: Counted = Counted {
        at: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cur: 0,
        cnt: 0,
        exact_match: false,
        incomplete: 0,
        pat: ptr::null_mut(),
        patlen: 0,
        chgtick: 0,
        buf: ptr::null_mut(),
    };

    /// Whether the remembered numbers still describe the current buffer,
    /// pattern and cursor position.
    ///
    /// # Safety
    /// Reads the current buffer and the remembered pattern.
    unsafe fn still_holds(&self, cursor_pos: pos_T) -> bool {
        unsafe {
            let live = last_used_pattern();
            self.chgtick as varnumber_T == buf_get_changedtick(curbuf.get())
                // The null test suppresses clang's "NULL passed as
                // nonnull parameter" on `strncmp`.
                && !self.pat.is_null()
                && strncmp(self.pat, live.pat, self.patlen) == 0
                && self.patlen == live.patlen
                && equalpos(self.at, cursor_pos)
                && self.buf == curbuf.get()
        }
    }

    /// Throw the numbers away and start counting the current buffer again.
    fn restart(&mut self) {
        self.cur = 0;
        self.cnt = 0;
        self.exact_match = false;
        self.incomplete = 0;
        clearpos(&mut self.at);
        // SAFETY: reads a global pointer.
        self.buf = curbuf.get();
    }

    /// Record what the numbers were counted from.
    ///
    /// # Safety
    /// Takes an owned copy of the live pattern; frees the previous one.
    unsafe fn remember(&mut self, at: pos_T) {
        unsafe {
            let live = last_used_pattern();
            xfree(self.pat as *mut c_void);
            self.pat = xstrnsave(live.pat, live.patlen);
            self.patlen = live.patlen;
            self.chgtick = buf_get_changedtick(curbuf.get()) as c_int;
            self.buf = curbuf.get();
            self.at = at;
        }
    }
}

static counted: GlobalCell<Counted> = GlobalCell::new(Counted::NONE);
/// The `maxcount` [`update_search_stat`] was last asked for.
static last_maxcount: GlobalCell<c_int> = GlobalCell::new(0);

/// Add the search count `[3/19]` to the right-hand end of `msgbuf`.
///
/// See [`update_search_stat`] for the other arguments.
///
/// # Safety
/// `pos` and `cursor_pos` must be readable and `msgbuf` a writable buffer
/// of `msgbuflen` bytes followed by a NUL.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn cmdline_search_stat(
    dirc: c_int,
    pos: *mut pos_T,
    cursor_pos: *mut pos_T,
    show_top_bot_msg: bool,
    msgbuf: *mut c_char,
    msgbuflen: size_t,
    recompute: bool,
    maxcount: c_int,
    timeout: c_int,
) {
    unsafe {
        let stat = update_search_stat(dirc, *pos, *cursor_pos, recompute, maxcount, timeout);
        if stat.cur <= 0 {
            return;
        }

        // A right-to-left window has the pair the other way round, so that
        // it still reads "current of total" on screen.
        let reversed = (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && *(*curwin.get()).w_onebuf_opt.wo_rlc as c_int == 's' as c_int;
        let mut t = [0 as c_char; STAT_BUF_LEN];
        let at = t.as_mut_ptr();
        let room = STAT_BUF_LEN as size_t;
        let mut len = if stat.incomplete == 1 {
            vim_snprintf(at, room, c"[?/??]".as_ptr())
        } else if stat.cnt > maxcount && stat.cur > maxcount {
            vim_snprintf(at, room, c"[>%d/>%d]".as_ptr(), maxcount, maxcount)
        } else if stat.cnt > maxcount {
            if reversed {
                vim_snprintf(at, room, c"[>%d/%d]".as_ptr(), maxcount, stat.cur)
            } else {
                vim_snprintf(at, room, c"[%d/>%d]".as_ptr(), stat.cur, maxcount)
            }
        } else if reversed {
            vim_snprintf(at, room, c"[%d/%d]".as_ptr(), stat.cnt, stat.cur)
        } else {
            vim_snprintf(at, room, c"[%d/%d]".as_ptr(), stat.cur, stat.cnt)
        } as usize;

        // "W " marks a search that wrapped around.
        if show_top_bot_msg && len + 2 < STAT_BUF_LEN {
            t.copy_within(0..len, 2);
            t[0] = b'W' as c_char;
            t[1] = b' ' as c_char;
            len += 2;
        }

        len = len.min(msgbuflen);
        ptr::copy(t.as_ptr(), msgbuf.add(msgbuflen - len), len);

        // (Upstream clears `stat.cur` for a backward search that landed on
        // `maxcount + 1` here. Nothing reads it afterwards.)

        // Keep the message even after a redraw, but not in the history.
        msg_ext_overwrite.set(true);
        msg_ext_set_kind(c"search_count".as_ptr());
        give_warning(msgbuf, false, false);
    }
}

/// Count the matches of the last used pattern, reusing the remembered
/// numbers when they still hold.
///
/// `dirc` is 0 to leave the cursor where it is (only report), `/` to count
/// the next match and `?` the previous one. With `recompute` the numbers
/// are always counted afresh. The count gives up after `maxcount` matches
/// or `timeout` milliseconds, saying so in `Stat::incomplete`.
///
/// # Safety
/// Runs a search over the current buffer.
unsafe fn update_search_stat(
    dirc: c_int,
    pos: pos_T,
    cursor_pos: pos_T,
    recompute: bool,
    maxcount: c_int,
    timeout: c_int,
) -> Stat {
    unsafe {
        let mut c = counted.get();
        if dirc == 0 && !recompute && !equalpos(c.at, pos_T::default()) {
            return Stat {
                cur: c.cur,
                cnt: c.cnt,
                exact_match: c.exact_match,
                incomplete: c.incomplete,
                last_maxcount: p_msc.get() as c_int,
            };
        }
        last_maxcount.set(maxcount);

        // Having moved past the remembered position in the direction
        // opposite to the search means the count wrapped around.
        let wraparound =
            (dirc == '?' as c_int && lt(c.at, pos)) || (dirc == '/' as c_int && lt(pos, c.at));

        // If anything relevant changed, the count has to be recomputed.
        if !c.still_holds(cursor_pos)
            || wraparound
            || c.cur < 0
            || (maxcount > 0 && c.cur > maxcount)
            || recompute
        {
            c.restart();
        }

        // When searching backwards and having jumped to the first
        // occurrence, `cur` must stay above 1.
        let steppable = if dirc == 0 || dirc == '/' as c_int {
            c.cur < c.cnt
        } else {
            c.cur > 1
        };
        if equalpos(c.at, cursor_pos) && !wraparound && steppable {
            c.cur += if dirc == 0 {
                0
            } else if dirc == '/' as c_int {
                1
            } else {
                -1
            };
        } else {
            let save_ws = p_ws.get();
            p_ws.set(0);
            let start = if timeout > 0 {
                profile_setlimit(timeout as int64_t)
            } else {
                0
            };
            let mut done_search = false;
            let mut endpos = pos_T::default();
            // `searchit` walks `c.at` forward one match at a time; without
            // 'wrapscan' it fails once past the last one.
            while !got_int.get()
                && searchit(
                    curwin.get(),
                    curbuf.get(),
                    &raw mut c.at,
                    &raw mut endpos,
                    FORWARD,
                    ptr::null_mut(),
                    0,
                    1,
                    SEARCH_KEEP,
                    RE_LAST,
                    ptr::null_mut(),
                ) != FAIL
            {
                done_search = true;
                // Stop after passing the time limit.
                if timeout > 0 && profile_passed_limit(start) {
                    c.incomplete = 1;
                    break;
                }
                c.cnt += 1;
                if ltoreq(c.at, pos) {
                    c.cur = c.cnt;
                    if lt(pos, endpos) {
                        c.exact_match = true;
                    }
                }
                fast_breakcheck();
                if maxcount > 0 && c.cnt > maxcount {
                    c.incomplete = 2; // max count exceeded
                    break;
                }
            }
            if got_int.get() {
                c.cur = -1; // abort
            }
            if done_search {
                c.remember(pos);
            }
            p_ws.set(save_ws);
        }

        counted.set(c);
        Stat {
            cur: c.cur,
            cnt: c.cnt,
            exact_match: c.exact_match,
            incomplete: c.incomplete,
            last_maxcount: last_maxcount.get(),
        }
    }
}

/// Read an integer out of the `searchcount()` option dictionary.
///
/// Answers `None` when the value could not be converted, which is the
/// caller's cue to give up; the conversion has already reported why.
///
/// # Safety
/// `dict` must be a readable dictionary.
unsafe fn dict_number(dict: *mut dict_T, key: &CStr, current: c_int) -> Option<c_int> {
    unsafe {
        let di = tv_dict_find(dict, key.as_ptr(), -1 as ptrdiff_t);
        if di.is_null() {
            return Some(current);
        }
        let mut error = false;
        let value = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) as c_int;
        if error { None } else { Some(value) }
    }
}

/// One element of the `pos` list, which is `[lnum, col, off]`.
///
/// # Safety
/// `list` must be a readable list.
unsafe fn list_number(list: *mut list_T, index: c_int, current: c_int) -> Option<c_int> {
    unsafe {
        let li = tv_list_find(list, index);
        if li.is_null() {
            return Some(current);
        }
        let mut error = false;
        let value = tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error) as c_int;
        if error { None } else { Some(value) }
    }
}

/// `searchcount()`: the match counts as a dictionary.
///
/// # Safety
/// The Vimscript function ABI: `argvars` is the argument array and
/// `rettv` the return value.
pub unsafe extern "C" fn f_searchcount(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let mut pos = (*curwin.get()).w_cursor;
        let mut pattern = ptr::null_mut::<c_char>();
        let mut maxcount = p_msc.get() as c_int;
        let mut timeout = SEARCH_STAT_DEF_TIMEOUT as c_int;
        // Upstream also sets this when 'shortmess' contains `S`, which it
        // already is.
        let mut recompute = true;

        tv_dict_alloc_ret(rettv);

        if (*argvars).v_type != VAR_UNKNOWN {
            if tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL {
                return;
            }
            let dict = (*argvars).vval.v_dict;
            let Some(t) = dict_number(dict, c"timeout", timeout) else {
                return;
            };
            timeout = t;
            let Some(m) = dict_number(dict, c"maxcount", maxcount) else {
                return;
            };
            maxcount = m;
            let Some(r) = dict_number(dict, c"recompute", recompute as c_int) else {
                return;
            };
            recompute = r != 0;

            let di = tv_dict_find(dict, c"pattern".as_ptr(), -1 as ptrdiff_t);
            if !di.is_null() {
                pattern = tv_get_string_chk(&raw mut (*di).di_tv) as *mut c_char;
                if pattern.is_null() {
                    return;
                }
            }

            let di = tv_dict_find(dict, c"pos".as_ptr(), -1 as ptrdiff_t);
            if !di.is_null() {
                if (*di).di_tv.v_type != VAR_LIST {
                    semsg(gettext(e_invarg2.ptr().cast()), c"pos".as_ptr());
                    return;
                }
                let list = (*di).di_tv.vval.v_list;
                if tv_list_len(list) != 3 {
                    semsg(
                        gettext(e_invarg2.ptr().cast()),
                        c"List format should be [lnum, col, off]".as_ptr(),
                    );
                    return;
                }
                let Some(lnum) = list_number(list, 0, pos.lnum) else {
                    return;
                };
                pos.lnum = lnum;
                // The list is 1-based, the position is not.
                let Some(col) = list_number(list, 1, pos.col + 1) else {
                    return;
                };
                pos.col = col - 1;
                let Some(coladd) = list_number(list, 2, pos.coladd) else {
                    return;
                };
                pos.coladd = coladd;
            }
        }

        save_last_search_pattern();
        save_incsearch_state();
        'the_end: {
            if !pattern.is_null() {
                if *pattern as c_int == NUL {
                    break 'the_end;
                }
                replace_last_used_pattern(pattern);
            }
            let live = last_used_pattern();
            if live.pat.is_null() || *live.pat as c_int == NUL {
                // The previous pattern was never defined.
                break 'the_end;
            }

            let stat = update_search_stat(0, pos, pos, recompute, maxcount, timeout);
            let mut add = |key: &CStr, value: c_int| {
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    key.as_ptr(),
                    key.to_bytes().len(),
                    value as varnumber_T,
                );
            };
            add(c"current", stat.cur);
            add(c"total", stat.cnt);
            add(c"exact_match", stat.exact_match as c_int);
            add(c"incomplete", stat.incomplete);
            add(c"maxcount", stat.last_maxcount);
        }
        restore_last_search_pattern();
        restore_incsearch_state();
    }
}
