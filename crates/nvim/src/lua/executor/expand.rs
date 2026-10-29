//! Command-line completion over Lua names.
//!
//! [`nlua_expand_pat`] hands the pattern to `vim._expand_pat` and stashes the
//! results in `EXPAND_RESULTS`, which [`nlua_expand_matches`] then drains
//! -- the two-step shape exists because the caller wants the matches after
//! the Lua state has been unwound.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{nlua_error, nlua_pcall, with_main_stack};
use crate::global_cell::GlobalCell;
use crate::lua::converter::{pop_array, pop_integer};
use crate::lua::stack::At;
use crate::memory::XString;
use crate::types::{Expand, Failed};

/// The matches [`nlua_expand_pat`] produced, waiting for
/// [`nlua_expand_matches`] to take them.
static EXPAND_RESULTS: GlobalCell<Vec<XString>> = GlobalCell::new(Vec::new());

/// Complete the pattern of `expand` through `vim._expand_pat`, which answers
/// a prefix length and a list of strings.
///
/// The prefix length is how much of the pattern the matches already include,
/// so the pattern is advanced past it. Anything that goes wrong — the call,
/// the conversion, a non-string in the list, a prefix longer than the
/// pattern — leaves no matches at all.
///
/// The pattern is copied out before Lua runs: what Lua does cannot reach
/// `expand`.
pub fn nlua_expand_pat(expand: &mut Expand) {
    // From the pattern to the cursor.
    let col = usize::try_from(expand.col).unwrap_or(0);
    debug_assert!(col >= expand.pattern);
    let pattern = expand
        .line
        .get(expand.pattern..col.max(expand.pattern))
        .unwrap_or_default()
        .to_vec();
    let patlen = pattern.len();

    let mut matches = Vec::new();
    let mut prefix_len = None;

    with_main_stack(|stack| {
        stack.get_global(c"vim");
        // Upstream checks the type and raises outside any protected call, which
        // takes the editor down; one that is not a function is reported by the
        // call below instead.
        stack.get_field(At::TOP, c"_expand_pat");
        stack.push_bytes(&pattern);

        if nlua_pcall(stack, 1, 2).is_err() {
            nlua_error(stack, c"vim._expand_pat: %.*s");
            return;
        }

        let prefix = pop_integer(stack)
            .ok()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= patlen);
        if let Some(prefix) = prefix
            && let Ok(completions) = pop_array(stack)
        {
            let strings: Option<Vec<XString>> = completions
                .iter()
                .map(|item| item.as_string().map(|s| XString::from_cstr(s.as_cstr())))
                .collect();
            if let Some(strings) = strings {
                matches = strings;
                prefix_len = Some(prefix);
            }
        }

        // Whatever a previous run left undrained goes here; a failed run keeps
        // nothing.
        match prefix_len {
            Some(prefix) => {
                expand.pattern += prefix;
                EXPAND_RESULTS.set(matches);
            }
            None => EXPAND_RESULTS.set(Vec::new()),
        }
    })
}

/// Take the stashed matches. `Err` when there are none.
pub fn nlua_expand_matches() -> Result<Vec<XString>, Failed> {
    let matches = EXPAND_RESULTS.take();
    if matches.is_empty() {
        Err(Failed)
    } else {
        Ok(matches)
    }
}
