//! The finished line: making it exactly `maxwidth` cells wide.
//!
//! Two passes, and at most one of them runs. When the text came out too long
//! it is cut, at the `%<` mark if there is one and at the end otherwise; when
//! it came out short and the format asked for `%=`, the slack is spread over
//! the separators. Both move text that the recorded items point into, so
//! both fix up the item offsets as they go.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::{
    Built, Kind, StlScratch, cells_at, char_len_at, fill_len, free_cstring, put_fill, strsize_at,
};
use crate::src::nvim::types::schar_T;

/// Cut the line down to `maxwidth` cells.
pub(super) fn truncate(
    s: &mut StlScratch,
    out: &mut [u8],
    built: &mut Built,
    outputlen: usize,
    maxwidth: c_int,
    fillchar: schar_T,
) {
    let last = built.items(s.items.len()).end;
    // Where to cut: the `%<` item if the format has one, the first item
    // otherwise, and the start of the line when there are no items at all.
    let mut item_idx = built.evalstart;
    let mut trunc = if built.itemcnt == 0 {
        0
    } else {
        let mut at = s.items[item_idx].start;
        for i in built.evalstart..last {
            if s.items[i].kind == Kind::Trunc {
                at = s.items[i].start;
                item_idx = i;
                break;
            }
        }
        at
    };

    if built.width - strsize_at(out, trunc) >= maxwidth {
        // Everything before the cut is already too wide, so cut the *end*
        // off instead: walk forward to the last character that fits.
        trunc = 0;
        built.width = 0;
        loop {
            built.width += cells_at(out, trunc);
            if built.width >= maxwidth {
                break;
            }
            trunc += char_len_at(out, trunc);
        }

        // Forget any item that starts past the cut.
        for i in built.evalstart..last {
            if s.items[i].start > trunc {
                for j in i..last {
                    if s.items[j].kind == Kind::ClickFunc {
                        free_cstring(s.items[j].cmd);
                        s.items[j].cmd = core::ptr::null_mut();
                    }
                }
                // Upstream stores the absolute index here where the count
                // belongs, so a *nested* expansion (one whose `evalstart` is
                // not zero) goes on to read items above the ones it wrote.
                // Reproduced rather than fixed.
                built.itemcnt = i;
                break;
            }
        }

        out[trunc] = b'>';
        trunc += 1;
        out[trunc] = 0;
    } else {
        let mut end = outputlen;

        // How many bytes to remove from the cut point.
        let mut trunc_len = 0usize;
        while built.width >= maxwidth {
            built.width -= cells_at(out, trunc + trunc_len);
            trunc_len += char_len_at(out, trunc + trunc_len);
        }

        // Close the gap, keeping the NUL, and mark the cut with a `<`.
        let trunc_end = trunc + trunc_len;
        out.copy_within(trunc_end..end + 1, trunc + 1);
        end -= trunc_end - (trunc + 1);
        out[trunc] = b'<';

        // Move the items back, less the byte the `<` took; anything inside
        // the removed run collapses onto the `<`.
        let item_offset = trunc_len as isize - 1;
        for i in item_idx..built.evalstart + built.itemcnt {
            if s.items[i].start >= trunc_end {
                s.items[i].start = (s.items[i].start as isize - item_offset) as usize;
            } else {
                s.items[i].start = trunc;
            }
        }

        let mut at = if built.width + 1 < maxwidth {
            end
        } else {
            trunc
        };
        // Fill up for half a double-width character.
        loop {
            built.width += 1;
            if built.width >= maxwidth {
                break;
            }
            at = put_fill(out, at, fillchar);
        }
    }
    built.width = maxwidth;
}

/// Spread the leftover width over the `%=` separators.
pub(super) fn spread(
    s: &mut StlScratch,
    out: &mut [u8],
    built: &mut Built,
    maxwidth: c_int,
    fillchar: schar_T,
) {
    let last = built.items(s.items.len()).end;
    let mut count = 0;
    for i in built.evalstart..last {
        if s.items[i].kind == Kind::Separate {
            s.separators[count] = i;
            count += 1;
        }
    }
    if count == 0 {
        return;
    }

    // Every separator but the last gets an equal share; the last one takes
    // whatever the division left over.
    let slack = maxwidth - built.width;
    let standard_spaces = slack / count as c_int;
    let final_spaces = slack - standard_spaces * (count as c_int - 1);

    for l in 0..count {
        let cells = if l == count - 1 {
            final_spaces
        } else {
            standard_spaces
        };
        let dislocation = cells as usize * fill_len(fillchar);
        let start = s.items[s.separators[l]].start;
        let seploc = start + dislocation;

        // Shift the rest of the line right, NUL included, and fill the gap.
        let len = super::cstr_at(out, start).to_bytes().len();
        out.copy_within(start..start + len + 1, seploc);
        let mut at = start;
        while at < seploc {
            at = put_fill(out, at, fillchar);
        }

        for i in s.separators[l] + 1..last {
            s.items[i].start += dislocation;
        }
    }
    built.width = maxwidth;
}
