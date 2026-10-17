//! Counting the matches, for `[N/M]` and for `searchcount()`.
//!
//! [`update_search_stat`] does the counting — forwards from the top of the
//! buffer, giving up after a timeout or after `maxcount` matches — and
//! caches the answer against the buffer's changedtick so that repeating
//! the search is cheap. [`cmdline_search_stat`] renders it into the
//! message line; [`f_searchcount`] is the Vimscript view of the same
//! numbers.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::eval::typval::{NumBuf, list_items};
use crate::memory::XString;
use crate::optionstr::OptString;
use crate::regexp::RE_LAST;
use crate::search::{SEARCH_KEEP, SEARCH_STAT_DEF_TIMEOUT};
use crate::semsg;
use crate::types::{FAIL, VAR_LIST};
use crate::winlayer::{Buf, BufId, Win};
use core::ffi::{CStr, c_int};

/// How much room `[>999/>999]` and its `W ` prefix need.
const STAT_BUF_LEN: usize = SEARCH_STAT_BUF_LEN as usize;

/// What `[N/M]` and `searchcount()` report.
#[derive(Default)]
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
#[derive(Default)]
struct Counted {
    at: Pos,
    cur: c_int,
    cnt: c_int,
    exact_match: bool,
    incomplete: c_int,
    /// A copy of the pattern the numbers are for.
    pat: Option<XString>,
    chgtick: c_int,
    /// Which buffer the numbers were counted in, by identity rather than by
    /// address: a wiped buffer's `Buffer` can be handed back out by the
    /// allocator, and a *handle* is never reissued (p23-5).
    buf: Option<BufId>,
}

impl Counted {
    const NONE: Counted = Counted {
        at: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cur: 0,
        cnt: 0,
        exact_match: false,
        incomplete: 0,
        pat: None,
        chgtick: 0,
        buf: None,
    };

    /// Whether the remembered numbers still describe the current buffer,
    /// pattern and cursor position.
    fn still_holds(&self, cursor_pos: Pos) -> bool {
        VarNumber::from(self.chgtick) == buf_get_changedtick(Buf::current())
            && self.pat.as_deref().is_some_and(last_used_pattern_is)
            && equalpos(self.at, cursor_pos)
            && self.buf == Some(Buf::current().id())
    }

    /// Throw the numbers away and start counting the current buffer again.
    fn restart(&mut self) {
        self.cur = 0;
        self.cnt = 0;
        self.exact_match = false;
        self.incomplete = 0;
        clearpos(&mut self.at);
        self.buf = Some(Buf::current().id());
    }

    /// Record what the numbers were counted from.
    fn remember(&mut self, at: Pos) {
        self.pat = last_search_pat();
        self.chgtick = buf_get_changedtick(Buf::current()) as c_int;
        self.buf = Some(Buf::current().id());
        self.at = at;
    }
}

static counted: GlobalCell<Counted> = GlobalCell::new(Counted::NONE);
/// The `maxcount` [`update_search_stat`] was last asked for.
static last_maxcount: GlobalCell<c_int> = GlobalCell::new(0);

/// Add the search count `[3/19]` to the right-hand end of `msgbuf`, the
/// echoed search command with its terminator.
///
/// See [`update_search_stat`] for the other arguments.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmdline_search_stat(
    dirc: c_int,
    pos: Pos,
    cursor_pos: Pos,
    show_top_bot_msg: bool,
    msgbuf: &mut [u8],
    recompute: bool,
    maxcount: c_int,
    timeout: c_int,
) {
    let stat = update_search_stat(dirc, pos, cursor_pos, recompute, maxcount, timeout);
    if stat.cur <= 0 {
        return;
    }

    // A right-to-left window has the pair the other way round, so that
    // it still reads "current of total" on screen.
    let reversed = Win::current().w_onebuf_opt.wo_rl != 0
        && Win::current().w_onebuf_opt.wo_rlc.first_byte() as c_int == 's' as c_int;
    // Four digits is the most `'maxsearchcount'` allows on either side of
    // the slash, so the longest of these is `[>9999/>9999]` -- thirteen
    // bytes, and upstream's `STAT_BUF_LEN` scratch buffer never truncated
    // one.
    let mut count = if stat.incomplete == 1 {
        "[?/??]".to_string()
    } else if stat.cnt > maxcount && stat.cur > maxcount {
        format!("[>{maxcount}/>{maxcount}]")
    } else if stat.cnt > maxcount {
        if reversed {
            format!("[>{}/{}]", maxcount, stat.cur)
        } else {
            format!("[{}/>{}]", stat.cur, maxcount)
        }
    } else if reversed {
        format!("[{}/{}]", stat.cnt, stat.cur)
    } else {
        format!("[{}/{}]", stat.cur, stat.cnt)
    };

    // "W " marks a search that wrapped around.
    if show_top_bot_msg && count.len() + 2 < STAT_BUF_LEN {
        count.insert_str(0, "W ");
    }

    // The count goes into the last bytes before the terminator.
    let msgbuflen = msgbuf.len() - 1;
    let len = count.len().min(msgbuflen);
    msgbuf[msgbuflen - len..msgbuflen].copy_from_slice(&count.as_bytes()[..len]);

    // (Upstream clears `stat.cur` for a backward search that landed on
    // `maxcount + 1` here. Nothing reads it afterwards.)

    // Keep the message even after a redraw, but not in the history.
    msg_ext_overwrite.set(true);
    msg_ext_set_kind(c"search_count");
    let text = CStr::from_bytes_until_nul(msgbuf).unwrap_or_default();
    give_warning(text, false, false);
}

/// Count the matches of the last used pattern, reusing the remembered
/// numbers when they still hold.
///
/// `dirc` is 0 to leave the cursor where it is (only report), `/` to count
/// the next match and `?` the previous one. With `recompute` the numbers
/// are always counted afresh. The count gives up after `maxcount` matches
/// or `timeout` milliseconds, saying so in `Stat::incomplete`.
fn update_search_stat(
    dirc: c_int,
    pos: Pos,
    cursor_pos: Pos,
    recompute: bool,
    maxcount: c_int,
    timeout: c_int,
) -> Stat {
    let mut c = counted.take();
    if dirc == 0 && !recompute && !equalpos(c.at, Pos::default()) {
        let stat = Stat {
            cur: c.cur,
            cnt: c.cnt,
            exact_match: c.exact_match,
            incomplete: c.incomplete,
            last_maxcount: p_msc() as c_int,
        };
        counted.set(c);
        return stat;
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
        let save_ws = p_ws();
        P_WS.set(false);
        let start = if timeout > 0 {
            profile_setlimit(timeout as int64_t)
        } else {
            0
        };
        let mut done_search = false;
        let mut endpos = Pos::default();
        // `searchit` walks `c.at` forward one match at a time; without
        // 'wrapscan' it fails once past the last one.
        let (win, buf) = (Some(Win::current()), Buf::current());
        while !got_int.get()
            && searchit(
                win,
                buf,
                &mut c.at,
                Some(&mut endpos),
                FORWARD,
                b"",
                1,
                SEARCH_KEEP,
                RE_LAST,
                None,
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
        P_WS.set(save_ws);
    }

    let stat = Stat {
        cur: c.cur,
        cnt: c.cnt,
        exact_match: c.exact_match,
        incomplete: c.incomplete,
        last_maxcount: last_maxcount.get(),
    };
    counted.set(c);
    stat
}

/// Read an integer out of the `searchcount()` option dictionary.
///
/// Answers `None` when the value could not be converted, which is the
/// caller's cue to give up; the conversion has already reported why.
fn dict_number(dict: Option<&Dict>, key: &CStr, current: c_int) -> Option<c_int> {
    let Some(di) = dict_find(dict, key.to_bytes()) else {
        return Some(current);
    };
    tv_get_number_chk(&di.di_tv)
        .ok()
        .map(|value| value as c_int)
}

/// One element of the `pos` list, which is `[lnum, col, off]`.
fn list_number(item: &TypVal) -> Option<c_int> {
    tv_get_number_chk(item).ok().map(|value| value as c_int)
}

/// `searchcount()`: the match counts as a dictionary.
pub fn f_searchcount(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut pos = Win::current().w_cursor;
    let mut pattern: Option<Vec<u8>> = None;
    let mut maxcount = p_msc() as c_int;
    let mut timeout = SEARCH_STAT_DEF_TIMEOUT as c_int;
    // Upstream also sets this when 'shortmess' contains `S`, which it
    // already is.
    let mut recompute = true;

    tv_dict_alloc_ret(result);

    if !args.is_empty() {
        if tv_check_for_nonnull_dict_arg(args, 0).is_err() {
            return;
        }
        let dict = args[0].dict_ref();
        let Some(t) = dict_number(dict, c"timeout", timeout) else {
            return;
        };
        timeout = t;
        let Some(m) = dict_number(dict, c"maxcount", maxcount) else {
            return;
        };
        maxcount = m;
        let Some(r) = dict_number(dict, c"recompute", c_int::from(recompute)) else {
            return;
        };
        recompute = r != 0;

        if let Some(di) = dict_find(dict, b"pattern") {
            let Some(text) = numbuf.string_chk(&di.di_tv) else {
                return;
            };
            pattern = Some(text.to_bytes().to_vec());
        }

        if let Some(di) = dict_find(dict, b"pos") {
            if di.di_tv.v_type() != VAR_LIST {
                semsg!("E475: Invalid argument: {}", "pos");
                return;
            }
            let items = list_items(di.di_tv.list_ref());
            if items.len() != 3 {
                semsg!("E475: Invalid argument: List format should be [lnum, col, off]");
                return;
            }
            let Some(lnum) = list_number(&items[0].li_tv) else {
                return;
            };
            pos.lnum = lnum;
            // The list is 1-based, the position is not.
            let Some(col) = list_number(&items[1].li_tv) else {
                return;
            };
            pos.col = col - 1;
            let Some(coladd) = list_number(&items[2].li_tv) else {
                return;
            };
            pos.coladd = coladd;
        }
    }

    save_last_search_pattern();
    save_incsearch_state();
    'the_end: {
        if let Some(pattern) = &pattern {
            if pattern.is_empty() {
                break 'the_end;
            }
            replace_last_used_pattern(pattern);
        }
        if last_search_pat().is_none_or(|live| live.is_empty()) {
            // The previous pattern was never defined.
            break 'the_end;
        }

        let stat = update_search_stat(0, pos, pos, recompute, maxcount, timeout);
        // The dictionary this function itself allocated above.
        if let Some(dict) = result.dict_mut() {
            let values = [
                (&b"current"[..], stat.cur),
                (b"total", stat.cnt),
                (b"exact_match", c_int::from(stat.exact_match)),
                (b"incomplete", stat.incomplete),
                (b"maxcount", stat.last_maxcount),
            ];
            for (key, value) in values {
                let _ = dict.add_number(key, VarNumber::from(value));
            }
        }
    }
    restore_last_search_pattern();
    restore_incsearch_state();
}
