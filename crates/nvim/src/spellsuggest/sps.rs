//! The `'spellsuggest'` option: what its items mean, and the outside
//! sources two of them name.
//!
//! [`spell_check_sps`] parses the option into [`sps_flags`](super::sps_flags)
//! and `sps_limit`, which the internal search then reads. The `expr:` and
//! `file:` items instead name a source of suggestions outside the editor's
//! own word trees, and [`spell_suggest_expr`] and [`spell_suggest_file`]
//! are those two.

#![forbid(unsafe_code)]

use super::{
    MAXPATHL, MAXWLEN, SCORE_FILE, SPS_BEST, SPS_DOUBLE, SPS_FAST, SugInfo, sps_flags, sps_limit,
};
use crate::charset::getdigits_int_at;
use crate::eval::typval::list_iter;
use crate::eval::vars::{eval_spell_expr, get_spellword};
use crate::fileio::Fgets;
use crate::getchar::state::got_int;
use crate::message_fmt::msg_bytes;
use crate::option::split_option_part;
use crate::option::vars::P_SPS;
use crate::os::input::line_breakcheck;
use crate::semsg;
use crate::spell::{captype, make_case_word};
use crate::spellsuggest::collect::{
    List, add_suggestion, ascii_casecmp, check_suggestions, cleanup_suggestions,
};
use crate::types::Failed;
use crate::types::VAR_LIST;
use core::cmp::Ordering;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

/// Does a `timeout:` item name a number? The value may be negative, which
/// switches the timeout off.
fn is_timeout_value(value: &[u8]) -> bool {
    let digits = value.strip_prefix(b"-").unwrap_or(value);
    digits.first().is_some_and(u8::is_ascii_digit)
}

/// Check `'spellsuggest'` and set [`sps_flags`] and [`sps_limit`] from it.
///
/// Answers `Err` for a value the option should not take, having put both
/// back to their defaults.
pub(crate) fn spell_check_sps() -> Result<(), Failed> {
    sps_flags.set(0);
    sps_limit.set(9999);

    let sps = P_SPS.get();
    let mut rest: &[u8] = crate::cstr::until_nul(&sps);
    while !rest.is_empty() {
        let (part, after) = split_option_part(rest, MAXPATHL as usize, b",");
        rest = after;

        // Zero means "this item said nothing about the method", -1
        // means "this item is not a valid one".
        let mut f = 0;
        if part.first().is_some_and(u8::is_ascii_digit) {
            let (limit, past) = getdigits_int_at(&part, 0, true, 0);
            sps_limit.set(limit);
            if part
                .get(past)
                .is_some_and(|&b| b != 0 && !b.is_ascii_digit())
            {
                f = -1;
            }
        // Keep the three names in sync with `opt_sps_values`.
        } else if part == b"best" {
            f = SPS_BEST;
        } else if part == b"fast" {
            f = SPS_FAST;
        } else if part == b"double" {
            f = SPS_DOUBLE;
        } else if !part.starts_with(b"expr:")
            && !part.starts_with(b"file:")
            && !part
                .strip_prefix(b"timeout:".as_slice())
                .is_some_and(is_timeout_value)
        {
            f = -1;
        }

        // Only one method may be named.
        if f == -1 || (sps_flags.get() != 0 && f != 0) {
            sps_flags.set(SPS_BEST);
            sps_limit.set(9999);
            return Err(Failed);
        }
        if f != 0 {
            sps_flags.set(f);
        }
    }

    if sps_flags.get() == 0 {
        sps_flags.set(SPS_BEST);
    }
    Ok(())
}

/// Find suggestions by evaluating `expr`, the `expr:` item of
/// `'spellsuggest'`.
pub(super) fn spell_suggest_expr(su: &mut SugInfo, expr: &[u8]) {
    // The list the expression returns is owned here until the handle drops.
    // The work is split up so that `SugInfo` need not be exported to
    // the evaluator.
    if let Some(list) = eval_spell_expr(su.badword(), expr) {
        for li in list_iter(Some(&list)) {
            if li.li_tv.v_type() == VAR_LIST {
                // Each item is a [word, score] pair.
                let Some((word, score)) = get_spellword(li.li_tv.list_ref()) else {
                    continue;
                };
                if score >= 0 && score <= su.su_maxscore {
                    let (badlen, lang) = (su.su_badlen, su.su_sallang);
                    add_suggestion(su, List::Main, &word, badlen, score, 0, true, lang, false);
                }
            }
        }
    }

    check_suggestions(su, List::Main);
    let (maxscore, maxcount) = (su.su_maxscore, su.su_maxcount);
    cleanup_suggestions(&mut su.su_ga, maxscore, maxcount);
}

/// Find suggestions in `fname`, the `file:` item of `'spellsuggest'`.
///
/// Every line of the file is `badword/goodword`.
pub(super) fn spell_suggest_file(su: &mut SugInfo, fname: &[u8]) {
    let Ok(file) = File::open(OsStr::from_bytes(fname)) else {
        let fname = msg_bytes(fname);
        semsg!("E484: Can't open file {fname}");
        return;
    };
    let mut file = Fgets::new(file);

    let mut line = Vec::new();
    let mut cword = [0u8; MAXWLEN];
    while file.line_capped(&mut line, MAXWLEN * 2) && !got_int.get() {
        line_breakcheck();

        // As C reads the line: up to its first NUL.
        let line = crate::cstr::until_nul(&line);
        let Some(slash) = line.iter().position(|&b| b == b'/') else {
            continue; // no separator, so not an entry
        };
        if ascii_casecmp(su.badword(), &line[..slash]) != Ordering::Equal {
            continue;
        }

        // A match: the good word runs to the CR or NL.
        let good = &line[slash + 1..];
        let mut good = &good[..good.iter().position(|&b| b < b' ').unwrap_or(good.len())];

        // A suggestion with no case of its own takes the bad word's.
        if captype(good, good.len()).is_empty() {
            make_case_word(good, &mut cword, su.su_badflags);
            good = crate::cstr::until_nul(&cword);
        }

        let (badlen, lang) = (su.su_badlen, su.su_sallang);
        add_suggestion(
            su,
            List::Main,
            good,
            badlen,
            SCORE_FILE,
            0,
            true,
            lang,
            false,
        );
    }

    check_suggestions(su, List::Main);
    let (maxscore, maxcount) = (su.su_maxscore, su.su_maxcount);
    cleanup_suggestions(&mut su.su_ga, maxscore, maxcount);
}
