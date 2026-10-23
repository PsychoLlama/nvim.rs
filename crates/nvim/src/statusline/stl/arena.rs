//! The arenas: the item stack, and the tables handed back through it.
//!
//! One expansion records a [`StlItem`] per `%` item, in the order the format
//! named them, and the post-processing stages read that back to place the
//! highlight runs and the click regions. Upstream keeps all five arrays as
//! function-local `static`s, so they are **shared across a recursive
//! expansion** -- `%{nvim_eval_statusline(...)}` lands back in the expander
//! with the outer level's items still on the stack, which two of the arms
//! rely on. See [`StlScratch`] for what that makes observable, and the
//! module docs of [`super`] for the borrow discipline it forces.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use std::ffi::CString;

use super::{kStlClickDisabled, kStlClickFuncRun, kStlClickTabClose, kStlClickTabSwitch};
use crate::global_cell::GlobalCell;
use crate::types::{StlClickDefinition, StlClickRecord, StlHlRun, StlOpt};

/// What one `%` item turned into. Upstream's `StlItem`, with the `char *`
/// into the output buffer replaced by a byte offset into it -- which is what
/// lets every stage that shifts the text around be safe code.
#[derive(Clone, Copy)]
pub(super) struct StlItem {
    /// Where the item starts in the output buffer.
    pub start: usize,
    /// `%@Func@`'s function name, as one more than its slot in
    /// [`StlScratch::cmds`], which holds it until it is handed to the
    /// caller's click records or dropped by the truncation pass. 0 for every
    /// other kind.
    pub cmd: u32,
    /// The item's minimum width, or the argument of the kinds that overload
    /// it -- a user highlight number, a negative syntax id, a tab page
    /// number (negative to close it), a `%@Func@` click id.
    pub minwid: c_int,
    /// The item's maximum width. Only a [`Kind::Group`] reads it.
    pub maxwid: c_int,
    pub kind: Kind,
}

impl Default for StlItem {
    fn default() -> Self {
        StlItem {
            start: 0,
            cmd: 0,
            minwid: 0,
            maxwid: 0,
            kind: Kind::Normal,
        }
    }
}

/// The kinds of item the format language has. Upstream's anonymous enum in
/// `StlItem`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Kind {
    /// Text, laid out by the item's width fields.
    Normal,
    /// An item that produced nothing.
    Empty,
    /// The opening `%(` of a group.
    Group,
    /// `%=`, where leftover width is spread.
    Separate,
    /// `%N*` or `%#name#`.
    Highlight,
    /// `%$name$`, which keeps the attributes already in force.
    HighlightCombining,
    /// A `'statuscolumn'` sign, which carries its own highlight.
    HighlightSign,
    /// A `'statuscolumn'` fold column, likewise.
    HighlightFold,
    /// `%NT` or `%NX`.
    TabPage,
    /// `%@Func@`.
    ClickFunc,
    /// `%<`, where the line is cut when it does not fit.
    Trunc,
}

impl Kind {
    /// Whether the highlight table takes this kind.
    pub(super) fn is_highlight(self) -> bool {
        matches!(
            self,
            Kind::Highlight | Kind::HighlightCombining | Kind::HighlightFold | Kind::HighlightSign
        )
    }

    /// The item the highlight table records for this kind, which is what
    /// tells the `'statuscolumn'` drawer one run from another. `None` for
    /// a kind that is not one of the three -- upstream's zero, which is
    /// not a letter.
    pub(super) fn hl_item(self) -> Option<StlOpt> {
        match self {
            Kind::HighlightSign => Some(StlOpt::SignCol),
            Kind::HighlightFold => Some(StlOpt::FoldCol),
            Kind::HighlightCombining => Some(StlOpt::HighlightComb),
            _ => None,
        }
    }
}

/// The five arenas one expansion works in, plus the item cursor.
///
/// Every field is shared across a recursive expansion, exactly as upstream's
/// six `static`s are. Two of those sharings are observable:
///
/// * **`items` and `curitem` are one stack.** A nested expansion appends
///   above the outer one's cursor and restores it on the way out, and the
///   outer items stay readable -- which the group-elision scan and `%X`'s
///   search for the tab number it closes both rely on, since both walk
///   *below* their own `evalstart`.
/// * **`groupitems` is indexed by a group depth that restarts at zero in
///   every expansion.** A `%{}` inside a `%(...%)` whose expression opens a
///   group of its own therefore overwrites the outer group's remembered item
///   index, and the outer `%)` closes against the wrong item. That is
///   upstream's behaviour and is reproduced rather than fixed.
pub(super) struct StlScratch {
    /// The items, in the order the format named them. Longer than
    /// [`Self::curitem`]: entries above the cursor are the arena's spare
    /// capacity, and are fully written before they are read.
    pub items: Vec<StlItem>,
    /// For each open group depth, the index of its `%(` item.
    pub groupitems: Vec<usize>,
    /// The indices of the `%=` items, filled by the spread pass.
    pub separators: Vec<usize>,
    /// The `%@Func@` names the items refer to, taken as they are handed
    /// out or dropped. Emptied when the outermost expansion is done.
    cmds: Vec<Option<CString>>,
    /// One past the last item written, across all recursion levels.
    pub curitem: usize,
}

/// Items an expansion starts with room for: upstream's `stl_items_len`.
const INITIAL_ITEMS: usize = 20;

impl StlScratch {
    const fn new() -> Self {
        StlScratch {
            items: Vec::new(),
            groupitems: Vec::new(),
            separators: Vec::new(),
            cmds: Vec::new(),
            curitem: 0,
        }
    }

    /// Make room for one more item, growing the arenas by half when the
    /// cursor has reached the end of them.
    ///
    /// Every recording call asks first. Upstream checked once per format
    /// item until v0.12.6, which overran the arena when a `'statuscolumn'`
    /// `%s` (one item per sign) or `%l` (up to three) came after many
    /// items; 04197decd6 added the check at those extra sites, which is the
    /// shape this already had.
    pub(super) fn grow(&mut self) {
        if self.curitem < self.items.len() {
            return;
        }
        let len = if self.items.is_empty() {
            INITIAL_ITEMS
        } else {
            self.items.len() * 3 / 2
        };
        self.items.resize(len, StlItem::default());
        self.groupitems.resize(len, 0);
        self.separators.resize(len, 0);
    }

    /// Record `item` and advance the cursor.
    pub(super) fn push_item(&mut self, item: StlItem) {
        self.grow();
        let at = self.curitem;
        self.items[at] = item;
        self.curitem = at + 1;
    }

    /// Keep `name` for a `%@Func@` item, answering what its `cmd` is.
    pub(super) fn add_cmd(&mut self, name: CString) -> u32 {
        self.cmds.push(Some(name));
        u32::try_from(self.cmds.len()).expect("fewer names than items")
    }

    /// Take the name an item's `cmd` refers to, if it is still there.
    pub(super) fn take_cmd(&mut self, cmd: u32) -> Option<CString> {
        let at = usize::try_from(cmd).ok()?.checked_sub(1)?;
        self.cmds.get_mut(at)?.take()
    }

    /// Forget every name, once no expansion is running.
    pub(super) fn clear_cmds(&mut self) {
        self.cmds.clear();
    }

    /// Record `kind` starting at `start`, which is all the items that carry
    /// no argument need.
    pub(super) fn push(&mut self, kind: Kind, start: usize) {
        self.push_item(StlItem {
            start,
            kind,
            ..StlItem::default()
        });
    }
}

/// The arenas. See the module docs for the borrow discipline.
static SCRATCH: GlobalCell<StlScratch> = GlobalCell::new(StlScratch::new());

/// Work on the arenas.
///
/// The borrow must not span an evaluation -- see the module docs.
pub(super) fn with_scratch<R>(f: impl FnOnce(&mut StlScratch) -> R) -> R {
    SCRATCH.with_mut(f)
}

/// What one expansion left behind: the width it drew, and which slice of the
/// item arena describes it.
pub(super) struct Built {
    pub width: c_int,
    /// The first item of this recursion level.
    pub evalstart: usize,
    /// How many items it wrote. Note this is the *item* count, which is what
    /// `hltab_len` answers -- not the number of highlight runs.
    pub itemcnt: usize,
}

impl Built {
    /// The items of this recursion level, bounded by the arena.
    ///
    /// The bound is not redundant: [`fill::truncate`] reproduces upstream's
    /// habit of storing an absolute item index in the *count*, which for a
    /// nested expansion names a range running off the end. Upstream reads
    /// past the arena there; here the reading stops at it.
    pub(super) fn items(&self, arena: usize) -> core::ops::Range<usize> {
        self.evalstart..(self.evalstart + self.itemcnt).min(arena)
    }
}

// ---------------------------------------------------------------------------
// What an expansion hands back

/// Where an item starts, as the caller's records spell it: a byte of the
/// built text, or the end of it.
///
/// The bound matches [`Built::items`]: a stale slot reached through
/// upstream's index-for-count store can hold an offset from a *previous*
/// expansion's buffer, and that offset may be past this one.
fn start_of(len: usize, at: usize) -> usize {
    at.min(len)
}

/// The highlight runs of this level's items, in order.
pub(super) fn collect_highlights(s: &StlScratch, len: usize, built: &Built) -> Vec<StlHlRun> {
    built
        .items(s.items.len())
        .map(|i| &s.items[i])
        .filter(|item| item.kind.is_highlight())
        .map(|item| StlHlRun {
            start: start_of(len, item.start),
            userhl: item.minwid,
            item: item.kind.hl_item(),
        })
        .collect()
}

/// The click records of this level's items, in order.
///
/// A `%@Func@` name moves from its item into the record here; the caller
/// keeps it in its click definitions.
pub(super) fn collect_clicks(s: &mut StlScratch, len: usize, built: &Built) -> Vec<StlClickRecord> {
    let mut recs = Vec::new();
    for i in built.items(s.items.len()) {
        let item = s.items[i];
        let (type_0, tabnr, func) = match item.kind {
            Kind::TabPage if item.minwid == 0 => (kStlClickDisabled, 0, None),
            Kind::TabPage if item.minwid > 0 => (kStlClickTabSwitch, item.minwid, None),
            Kind::TabPage => (kStlClickTabClose, -item.minwid, None),
            Kind::ClickFunc => (kStlClickFuncRun, item.minwid, s.take_cmd(item.cmd)),
            _ => continue,
        };
        recs.push(StlClickRecord {
            def: StlClickDefinition {
                type_0,
                tabnr,
                func: 0,
            },
            func,
            start: start_of(len, item.start),
        });
    }
    recs
}
