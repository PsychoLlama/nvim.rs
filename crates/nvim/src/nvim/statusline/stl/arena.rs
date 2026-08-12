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

use core::ffi::{c_char, c_int};
use core::ptr;

use super::{
    STL_FOLDCOL, STL_HIGHLIGHT_COMB, STL_SIGNCOL, kStlClickDisabled, kStlClickFuncRun,
    kStlClickTabClose, kStlClickTabSwitch,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::types::{StlClickDefinition, StlClickRecord, StlFlag, stl_hlrec_t};

/// What one `%` item turned into. Upstream's `stl_item_t`, with the `char *`
/// into the output buffer replaced by a byte offset into it -- which is what
/// lets every stage that shifts the text around be safe code.
#[derive(Clone, Copy)]
pub(super) struct StlItem {
    /// Where the item starts in the output buffer.
    pub start: usize,
    /// `%@Func@`'s function name: an `xmalloc`ed C string this item owns
    /// until it is handed to the caller's click table or freed by the
    /// truncation pass. Null for every other kind.
    pub cmd: *mut c_char,
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
            cmd: ptr::null_mut(),
            minwid: 0,
            maxwid: 0,
            kind: Kind::Normal,
        }
    }
}

/// The kinds of item the format language has. Upstream's anonymous enum in
/// `stl_item_t`.
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

    /// The `STL_*` letter the highlight table records for this kind, which
    /// is what tells the `'statuscolumn'` drawer a run apart.
    pub(super) fn hl_item(self) -> StlFlag {
        match self {
            Kind::HighlightSign => STL_SIGNCOL,
            Kind::HighlightFold => STL_FOLDCOL,
            Kind::HighlightCombining => STL_HIGHLIGHT_COMB,
            _ => 0,
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
    /// The highlight runs handed back through `hltab`.
    hltab: Vec<stl_hlrec_t>,
    /// The click records handed back through `tabtab`.
    tabtab: Vec<StlClickRecord>,
    /// The indices of the `%=` items, filled by the spread pass.
    pub separators: Vec<usize>,
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
            hltab: Vec::new(),
            tabtab: Vec::new(),
            separators: Vec::new(),
            curitem: 0,
        }
    }

    /// Make room for one more item, growing the arenas by half when the
    /// cursor has reached the end of them.
    ///
    /// The highlight and click tables get one entry more than there are
    /// items, because the last one is the terminator the caller stops at.
    /// This is the only place any arena is reallocated, which is what keeps
    /// a live `hltab`/`tabtab` pointer valid for as long as it is in C.
    ///
    /// Upstream checks this once per format item rather than once per item
    /// *recorded*, which is one slot short for a `'statuscolumn'` sign
    /// column: `%s` records one item per sign, up to nine of them, and
    /// writes past the arena when the cursor was near its end. Here every
    /// recording call asks first, which cannot overflow and grows the arena
    /// at the same points otherwise.
    pub fn grow(&mut self) {
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
        self.hltab.resize(len + 1, EMPTY_HLREC);
        self.tabtab.resize(len + 1, EMPTY_CLICKREC);
    }

    /// Record `item` and advance the cursor.
    pub fn push_item(&mut self, item: StlItem) {
        self.grow();
        let at = self.curitem;
        self.items[at] = item;
        self.curitem = at + 1;
    }

    /// Record `kind` starting at `start`, which is all the items that carry
    /// no argument need.
    pub fn push(&mut self, kind: Kind, start: usize) {
        self.push_item(StlItem {
            start,
            kind,
            ..StlItem::default()
        });
    }
}

/// A highlight run that is not one: the table's terminator.
const EMPTY_HLREC: stl_hlrec_t = stl_hlrec_t {
    start: ptr::null_mut(),
    userhl: 0,
    item: 0,
};

/// A click record that is not one: the table's terminator.
const EMPTY_CLICKREC: StlClickRecord = StlClickRecord {
    def: StlClickDefinition {
        type_0: kStlClickDisabled,
        tabnr: 0,
        func: ptr::null_mut(),
    },
    start: ptr::null(),
};

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
    pub fn items(&self, arena: usize) -> core::ops::Range<usize> {
        self.evalstart..(self.evalstart + self.itemcnt).min(arena)
    }
}

// ---------------------------------------------------------------------------
// The out-parameter tables
// ---------------------------------------------------------------------------

/// Fill the highlight table from this level's items and answer it.
pub(super) fn collect_highlights(
    s: &mut StlScratch,
    out: &[u8],
    built: &Built,
) -> *mut stl_hlrec_t {
    let mut n = 0;
    for i in built.items(s.items.len()) {
        let item = s.items[i];
        if !item.kind.is_highlight() {
            continue;
        }
        s.hltab[n] = stl_hlrec_t {
            start: out[item.start..].as_ptr().cast::<c_char>().cast_mut(),
            userhl: item.minwid,
            item: item.kind.hl_item(),
        };
        n += 1;
    }
    s.hltab[n] = EMPTY_HLREC;
    s.hltab.as_mut_ptr()
}

/// Fill the click table from this level's items and answer it.
///
/// A `%@Func@` name's ownership moves into the table here; the caller frees
/// it when it clears its click definitions.
pub(super) fn collect_clicks(s: &mut StlScratch, out: &[u8], built: &Built) -> *mut StlClickRecord {
    let mut n = 0;
    for i in built.items(s.items.len()) {
        let item = s.items[i];
        let def = match item.kind {
            Kind::TabPage if item.minwid == 0 => StlClickDefinition {
                type_0: kStlClickDisabled,
                tabnr: 0,
                func: ptr::null_mut(),
            },
            Kind::TabPage if item.minwid > 0 => StlClickDefinition {
                type_0: kStlClickTabSwitch,
                tabnr: item.minwid,
                func: ptr::null_mut(),
            },
            Kind::TabPage => StlClickDefinition {
                type_0: kStlClickTabClose,
                tabnr: -item.minwid,
                func: ptr::null_mut(),
            },
            Kind::ClickFunc => StlClickDefinition {
                type_0: kStlClickFuncRun,
                tabnr: item.minwid,
                func: item.cmd,
            },
            _ => continue,
        };
        s.tabtab[n] = StlClickRecord {
            def,
            start: out[item.start..].as_ptr().cast::<c_char>(),
        };
        n += 1;
    }
    s.tabtab[n] = EMPTY_CLICKREC;
    s.tabtab.as_mut_ptr()
}
