//! `xemit.c`: grouping changes into hunks, and writing a unified diff.
//!
//! [`get_hunk`] is shared with `xdiffi`'s hunk-callback walk, which is how
//! `:diffupdate` and `vim.diff{on_hunk=}` get the same hunk boundaries as
//! the text writer without going near [`emit_diff`].
//!
//! The `XDL_EMIT_FUNCNAMES`/`XDL_EMIT_FUNCCONTEXT` machinery — the `@@ ... @@
//! func_name` suffix and the "extend context to the enclosing function"
//! logic — is `#if 0`-ed out of the vendored source, so `xdemitconf_t`'s
//! `find_func`/`find_func_priv` are dead and every hunk header here carries
//! an empty function name.
//!
//! Ported from LibXDiff by Davide Libenzi (File Differential Library),
//! Copyright (C) 2003 Davide Libenzi. LibXDiff is LGPL-2.1-or-later, and
//! this port stays under that license (text: licenses/LGPL-2.1.txt).

#![forbid(unsafe_code)]

use crate::xdiff::ffi::Emit;
use crate::xdiff::xtypes::{Change, EmitConf, Env, XDL_EMIT_NO_HUNK_HDR, XdFile, XdResult};
use crate::xdiff::xutils::{emit_diffrec, emit_hunk_hdr};

/// Starting at `*start`, find the last change that belongs in the same hunk.
///
/// Two changes share a hunk when the unchanged run between them is short
/// enough to be printed as context for both — `2 * ctxlen + interhunkctxlen`
/// lines. Ignorable changes (all-blank, under `XDF_IGNORE_BLANK_LINES`)
/// neither open a hunk nor extend one; leading ones are skipped by advancing
/// `*start`, which is why it is in-out.
///
/// `None` means the skip ran off the end and there is no hunk left.
pub fn get_hunk(script: &[Change], start: &mut usize, xecfg: &EmitConf) -> Option<usize> {
    let max_common = 2 * xecfg.ctxlen + xecfg.interhunkctxlen;
    let max_ignorable = xecfg.ctxlen;
    // Blank lines skipped so far; they still count toward `max_common`.
    let mut ignored = 0i64;

    // Drop ignorable changes that are too far in front of any real one.
    let mut p = *start;
    while p < script.len() && script[p].ignore {
        let next = p + 1;
        if next >= script.len()
            || script[next].i1 - (script[p].i1 + script[p].chg1) >= max_ignorable
        {
            *start = next;
        }
        p = next;
    }
    if *start >= script.len() {
        return None;
    }

    let mut last = *start;
    let mut prev = *start;
    let mut cur = prev + 1;
    while cur < script.len() {
        let distance = script[cur].i1 - (script[prev].i1 + script[prev].chg1);
        if distance > max_common {
            break;
        }
        if distance < max_ignorable && (!script[cur].ignore || last == prev) {
            last = cur;
            ignored = 0;
        } else if distance < max_ignorable && script[cur].ignore {
            ignored += script[cur].chg2;
        } else if last != prev
            && script[cur].i1 + ignored - (script[last].i1 + script[last].chg1) > max_common
        {
            break;
        } else if !script[cur].ignore {
            last = cur;
            ignored = 0;
        } else {
            ignored += script[cur].chg2;
        }
        prev = cur;
        cur += 1;
    }

    Some(last)
}

/// Write one body line: its marker, the line, and the no-final-newline note.
fn emit_record(xdf: &XdFile<'_>, ri: i64, pre: &[u8], emit: &mut Emit<'_>) -> XdResult {
    emit_diffrec(xdf.line(ri), pre, emit)
}

/// Write the whole diff in unified format.
///
/// Reached only from `vim.diff()` without an `on_hunk` callback; everything
/// else in the tree installs `xdemitconf_t.hunk_func` and takes
/// `xdiffi::call_hunk_func` instead.
pub fn emit_diff(
    xe: &Env<'_>,
    script: &[Change],
    xecfg: &EmitConf,
    emit: &mut Emit<'_>,
) -> XdResult {
    let mut at = 0usize;

    while at < script.len() {
        let mut first = at;
        let Some(last) = get_hunk(script, &mut first, xecfg) else {
            break;
        };
        let (start, end) = (script[first], script[last]);

        let hdr1 = (start.i1 - xecfg.ctxlen).max(0);
        let mut hdr2 = (start.i2 - xecfg.ctxlen).max(0);

        // Trailing context, clamped to whatever is left of both files.
        let lctx = xecfg
            .ctxlen
            .min(xe.xdf1.nrec() - (end.i1 + end.chg1))
            .min(xe.xdf2.nrec() - (end.i2 + end.chg2));
        let e1 = end.i1 + end.chg1 + lctx;
        let e2 = end.i2 + end.chg2 + lctx;

        if xecfg.flags & XDL_EMIT_NO_HUNK_HDR == 0 {
            emit_hunk_hdr(hdr1 + 1, e1 - hdr1, hdr2 + 1, e2 - hdr2, emit)?;
        }

        // Leading context.
        while hdr2 < start.i2 {
            emit_record(&xe.xdf2, hdr2, b" ", emit)?;
            hdr2 += 1;
        }

        // The changes, with the unchanged lines between any two of them.
        let mut k = first;
        let mut s1 = start.i1;
        let mut s2 = start.i2;
        loop {
            let ch = script[k];
            while s1 < ch.i1 && s2 < ch.i2 {
                emit_record(&xe.xdf2, s2, b" ", emit)?;
                s1 += 1;
                s2 += 1;
            }
            s1 = ch.i1;
            while s1 < ch.i1 + ch.chg1 {
                emit_record(&xe.xdf1, s1, b"-", emit)?;
                s1 += 1;
            }
            s2 = ch.i2;
            while s2 < ch.i2 + ch.chg2 {
                emit_record(&xe.xdf2, s2, b"+", emit)?;
                s2 += 1;
            }
            if k == last {
                break;
            }
            s1 = ch.i1 + ch.chg1;
            s2 = ch.i2 + ch.chg2;
            k += 1;
        }

        // Trailing context.
        let mut s2 = end.i2 + end.chg2;
        while s2 < e2 {
            emit_record(&xe.xdf2, s2, b" ", emit)?;
            s2 += 1;
        }

        at = last + 1;
    }

    Ok(())
}
