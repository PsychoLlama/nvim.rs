//! `:for`: the list of things to iterate, and the step from one to the next.
//!
//! A [`ForInfo`] holds one of three iterations: a Blob by byte, a String by
//! character, or a List by item. Only the List form is *live*: the loop
//! holds a watcher on it and sees items added or removed while it runs. The
//! Blob and the String are copied up front, so changing the original
//! mid-loop has no effect.
//!
//! The condition stack owns the `ForInfo` of each `:for` it has open, and
//! dropping it is what ends the iteration: the List loses its watcher and
//! the reference the loop held.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{BlobRef, ListRef, blob_copy, blob_len, index_of, tv_copy};
use crate::eval::vars::{VarList, ex_let_vars, skip_var_list};
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::{e_string_list_or_blob_required, eval0_in_cmd};
use crate::guard::Suppress;
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::types::{ExArg, ListWatch, TypVal, VAR_BLOB, VAR_LIST, VAR_STRING, VarNumber};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// One `:for` loop's iteration: its targets and what it walks.
pub(crate) struct ForInfo {
    /// The loop's variable list, read off the header.
    targets: VarList,
    walk: Walk,
}

/// What a `:for` walks.
enum Walk {
    /// Nothing: a failed header, a skipped one, or a null List or Blob.
    Nothing,
    /// A List, through the cursor `watch` names on it.
    List { list: ListRef, watch: u32 },
    /// The loop's own copy of a Blob, and the next byte.
    Blob { blob: BlobRef, next: usize },
    /// A String taken over from the header's value, and the next byte.
    Text { text: ThinCString, next: usize },
}

impl Drop for ForInfo {
    fn drop(&mut self) {
        if let Walk::List { list, watch } = &self.walk {
            // The watcher `eval_for_line` added; the handle's own drop
            // releases the reference the loop held.
            list.edit().watch_remove(*watch);
        }
    }
}

/// Read the `for x in expr` header and set up the iteration. The answer is
/// always a `ForInfo`, even on the error paths, because `:endfor` drops it
/// either way; the flag is whether the loop must not run.
///
/// `skip` parses the header without evaluating it.
pub(crate) fn eval_for_line(excmd: &mut ExArg, skip: bool) -> (Box<ForInfo>, bool) {
    let mut info = Box::new(ForInfo {
        targets: VarList {
            end: 0,
            count: 0,
            semicolon: false,
        },
        walk: Walk::Nothing,
    });

    let arg = excmd.line.arg;
    let Some(targets) = skip_var_list(excmd.line.rest_of(arg), false) else {
        return (info, true);
    };
    info.targets = VarList { end: 0, ..targets };
    let line = &excmd.line;
    let at = line.skip_white(arg + targets.end);
    if !line.starts_with(at, b"in") || !matches!(line.byte_at(at + 2), 0 | b' ' | b'\t') {
        emsg_static(c"E690: Missing \"in\" after :for");
        return (info, true);
    }

    let _skipping = skip.then(Suppress::emsg_skip);
    let at = line.skip_white(at + 2);
    let mut tv = UNSET_TV;
    if eval0_in_cmd(excmd, at, &mut tv, !skip).is_err() {
        return (info, true);
    }
    if !skip {
        match tv.v_type() {
            VAR_LIST => {
                // The reference moves into the loop, and the watcher is
                // what keeps the cursor valid across changes to the List
                // while the loop runs. A null List leaves `tv` holding
                // nothing to release.
                if let Some(list) = tv.take_list() {
                    let watch = list.edit().watch_add();
                    info.walk = Walk::List { list, watch };
                }
            }
            VAR_BLOB => {
                if tv.blob_ref().is_some() {
                    // Copied, so the loop is not affected by later changes
                    // to the Blob it was handed.
                    let mut copy = UNSET_TV;
                    blob_copy(tv.blob_ref(), &mut copy);
                    if let Some(blob) = copy.take_blob() {
                        info.walk = Walk::Blob { blob, next: 0 };
                    }
                }
                clear_local(&mut tv);
            }
            VAR_STRING => {
                // The String is taken over rather than copied; a null one
                // walks as the empty string.
                let text = tv.take_string().unwrap_or_else(ThinCString::empty);
                info.walk = Walk::Text { text, next: 0 };
            }
            _ => {
                emsg_static(e_string_list_or_blob_required);
                clear_local(&mut tv);
            }
        }
    }
    (info, false)
}

/// Assign the next item to the loop variables. False when the iteration is
/// over, or when the assignment failed.
///
/// `arg` is the loop's variable list. The loop's state is advanced before the
/// assignment runs: assigning runs the targets' index expressions, which may
/// edit the List being walked.
pub(crate) fn next_for_item(info: &mut ForInfo, arg: &[u8]) -> bool {
    let targets = info.targets;
    let mut value = UNSET_TV;
    match &mut info.walk {
        Walk::Nothing => return false,
        Walk::Blob { blob, next } => {
            if *next >= usize::try_from(blob_len(Some(blob))).unwrap_or(0) {
                return false;
            }
            let at = i32::try_from(*next).expect("a Blob index fits an int");
            value.write_number(VarNumber::from(blob.byte(at)));
            *next += 1;
        }
        Walk::Text { text, next } => {
            let rest = text.as_bytes().get(*next..).unwrap_or_default();
            let len = cluster_len(rest);
            if len == 0 {
                return false;
            }
            value.write_string(Some(ThinCString::from_bytes(&rest[..len])));
            *next += len;
        }
        Walk::List { list, watch } => {
            // The cursor is an index into the list, which `watch_shift` moves
            // at every insert and removal so that it keeps naming the same
            // item. `ENDED` is upstream's NULL `lw_item`, and is sticky: a
            // loop whose body appends to the list it is walking still ends.
            let Ok(at) = usize::try_from(list.watch_index(*watch)) else {
                return false;
            };
            let len = list.len();
            let Some(item) = list.items().get(at) else {
                return false;
            };
            // The item is copied out: assigning it may edit the List.
            tv_copy(&item.li_tv, &mut value);
            let following = if at + 1 >= len {
                ListWatch::ENDED
            } else {
                index_of(at + 1)
            };
            list.edit().set_watch_index(*watch, following);
        }
    }
    let ok = ex_let_vars(arg, &mut value, false, targets, false, None).is_ok();
    // Whatever the targets did not take is this frame's to release.
    clear_local(&mut value);
    ok
}
