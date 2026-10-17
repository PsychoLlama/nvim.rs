//! What the last match captured, for the code that runs *inside* a
//! substitution: `submatch()` and `submatch(n, 1)`, and the list a `\=`
//! expression's function is handed as its argument.
//!
//! These read `rsm`, the snapshot [`super::substitute`] takes of the match
//! it is expanding before evaluating the expression. A `\=` expression may
//! run a search or a substitution of its own; the snapshot is what lets
//! `submatch()` keep answering about the substitution the user wrote.
//! `can_f_submatch` is what says the snapshot is live.
//!
//! A string match's snapshot holds the text of each group, so nothing here
//! borrows the string the match ran over; a buffer match's holds positions,
//! and the lines are read back from the buffer when asked for — as upstream
//! read them, so an expression that changed them is seen to have.

#![forbid(unsafe_code)]

use crate::eval::gc::{RootId, unroot_list};
use crate::eval::partial_name;
use crate::eval::typval::{NumBuf, TV_INITIAL_VALUE, tv_clear};
use crate::eval::userfunc::{CallWith, call_func_with};
use crate::memory::{ThinCString, XString};
use crate::types::{VAR_FUNC, VAR_PARTIAL, VAR_UNKNOWN, ptrdiff_t};
use core::ffi::c_int;

use super::{RegSubMatch, can_f_submatch, rsm};
use crate::eval::typval::{ListRef, SL_SIZE, tv_list_alloc};
use crate::memline::Lines;
use crate::types::{ColNr, LPos, LineNr, ListItem, RegMMatch, RegMatch, TypVal, UserFunc, VarLock};
use crate::winlayer::Buf;

impl RegSubMatch {
    /// The snapshot of a string match over `line`: each group's text.
    pub(crate) fn of_string(matches: &RegMatch, line: &[u8]) -> RegSubMatch {
        RegSubMatch::Str(core::array::from_fn(|no| {
            matches.group_bytes(no, line).map(<[u8]>::to_vec)
        }))
    }

    /// The snapshot of a buffer match over `buffer` from `firstlnum` on: where
    /// each group starts and ends.
    pub(crate) fn of_buffer(
        buffer: Buf,
        firstlnum: LineNr,
        maxline: LineNr,
        matches: &RegMMatch,
    ) -> RegSubMatch {
        RegSubMatch::Buf {
            buf: buffer,
            firstlnum,
            maxline,
            startpos: matches.startpos,
            endpos: matches.endpos,
        }
    }
}

/// A buffer match's snapshot, as the readers below want it.
struct BufferMatch {
    buf: Buf,
    firstlnum: LineNr,
    maxline: LineNr,
    start: LPos,
    end: LPos,
}

/// What the snapshot says about group `no`: its text for a string match,
/// its span for a buffer one, or `None` — no snapshot, a group that did not
/// match, or a group number past the ten there are.
enum Group {
    Str(Option<Vec<u8>>),
    Buf(BufferMatch),
}

fn group(no: usize) -> Option<Group> {
    rsm.with(|snapshot| match snapshot {
        RegSubMatch::None => None,
        RegSubMatch::Str(groups) => Some(Group::Str(groups.get(no)?.clone())),
        RegSubMatch::Buf {
            buf,
            firstlnum,
            maxline,
            startpos,
            endpos,
        } => Some(Group::Buf(BufferMatch {
            buf: *buf,
            firstlnum: *firstlnum,
            maxline: *maxline,
            start: *startpos.get(no)?,
            end: *endpos.get(no)?,
        })),
    })
}

impl BufferMatch {
    /// Line `lnum` of the match, counted from its first line: `None` above
    /// line 1, an empty line past the match's last one.
    fn line(&self, lnum: LineNr, read: impl FnOnce(&[u8])) -> Option<()> {
        if self.firstlnum + lnum < 1 {
            return None;
        }
        if lnum > self.maxline {
            read(&[]);
        } else {
            read(Lines::in_buffer(self.buf).line(self.firstlnum + lnum));
        }
        Some(())
    }

    /// Up to `len` bytes of line `lnum` from column `from`, stopping at the
    /// line's end: `read` sees them. A line the match names but the buffer
    /// no longer has gives no bytes, which is upstream's anti-crash check
    /// moved inside the walk.
    fn head(&self, lnum: LineNr, from: ColNr, len: usize, read: impl FnOnce(&[u8])) {
        let _ = self.line(lnum, |line| {
            let rest = usize::try_from(from)
                .ok()
                .and_then(|at| line.get(at..))
                .unwrap_or(&[]);
            read(&rest[..len.min(rest.len())]);
        });
    }

    /// How long line `lnum` is; 0 for either stand-in.
    fn line_len(&self, lnum: LineNr) -> ColNr {
        let mut len = 0;
        let _ = self.line(lnum, |line| {
            len = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
        });
        len
    }
}

/// A count that upstream held in a `size_t`: negative is huge, which the
/// readers then bound by the line's end.
fn span_len(len: ColNr) -> usize {
    usize::try_from(len).unwrap_or(usize::MAX)
}

/// Fill `argv[argskip]` — the caller's empty list — with the submatches, and
/// report how many arguments the call now has.
///
/// This is the `argv_func` a `\=` expression's function is called through,
/// so it runs before the function body sees its arguments. A function that
/// does not take a submatches argument gets none: the list stays as the
/// caller left it.
pub(crate) fn fill_submatch_list(argv: &[TypVal], argskip: usize, func: &UserFunc) -> usize {
    let body = func.body();
    if !body.varargs && body.args.len() <= argskip {
        return argskip;
    }

    // The list is the caller's own; it starts empty and gets one item per
    // capture. Only `substitute()` calls a function this way, so the match
    // is a string one; a buffer match's groups read as unset, where upstream
    // dereferenced the string match it did not have.
    let Some(list) = argv[argskip].list_shared() else {
        return argskip + 1;
    };
    let items = &mut list.edit().lv_items;
    items.reserve_exact(SL_SIZE);
    for i in 0..SL_SIZE {
        let li_tv = match group(i) {
            Some(Group::Str(Some(bytes))) => TypVal::string_from(&bytes),
            _ => TypVal::string(None),
        };
        items.push(ListItem {
            li_tv,
            li_lock: VarLock::Fixed,
        });
    }
    argskip + 1
}

/// The text capture `no` matched, as an allocated string the caller owns.
/// `None` outside a substitution and for a capture that did not participate.
///
/// A buffer match's capture can span lines, in which case the breaks come
/// back as newlines.
pub(crate) fn reg_submatch(no: c_int) -> Option<XString> {
    if !can_f_submatch.get() {
        return None;
    }
    let no = usize::try_from(no).ok()?;
    let m = match group(no)? {
        Group::Str(text) => return text.as_deref().map(XString::from_bytes),
        Group::Buf(m) => m,
    };
    let mut lnum = m.start.lnum;
    if lnum < 0 || m.end.lnum < 0 {
        return None;
    }
    // Anti-crash check; cannot happen.
    m.line(lnum, |_| {})?;
    let (scol, ecol) = (m.start.col, m.end.col);
    let mut text = XString::new();
    if m.end.lnum == lnum {
        // Within one line: from the start column to the end one.
        m.head(lnum, scol, span_len(ecol - scol), |bytes| {
            text.push_bytes(bytes);
        });
        return Some(text);
    }

    // The rest of the start line, then whole lines, then the head of the end
    // line. Each break travels as a newline.
    let rest = span_len(m.line_len(lnum) - scol);
    m.head(lnum, scol, rest, |bytes| text.push_bytes(bytes));
    text.push_byte(b'\n');
    lnum += 1;
    while lnum < m.end.lnum {
        let whole = span_len(m.line_len(lnum));
        m.head(lnum, 0, whole, |bytes| text.push_bytes(bytes));
        text.push_byte(b'\n');
        lnum += 1;
    }
    m.head(lnum, 0, span_len(ecol), |bytes| text.push_bytes(bytes));
    Some(text)
}

/// [`reg_submatch`] as one list item per line, which is what
/// `submatch(no, 1)` returns.
pub(crate) fn reg_submatch_list(no: c_int) -> Option<ListRef> {
    if !can_f_submatch.get() {
        return None;
    }
    let no = usize::try_from(no).ok()?;
    let m = match group(no)? {
        // A string match is one item.
        Group::Str(text) => {
            let text = text?;
            let list = tv_list_alloc(1);
            list.edit().push_bytes(Some(&text));
            return Some(list);
        }
        Group::Buf(m) => m,
    };
    let (slnum, elnum) = (m.start.lnum, m.end.lnum);
    if slnum < 0 || elnum < 0 {
        return None;
    }
    let (scol, ecol) = (m.start.col, m.end.col);

    let list = tv_list_alloc((elnum - slnum + 1) as isize);
    let push = |lnum: LineNr, from: ColNr, len: usize| {
        let found = m.line(lnum, |line| {
            let rest = usize::try_from(from)
                .ok()
                .and_then(|at| line.get(at..))
                .unwrap_or(&[]);
            list.edit().push_bytes(Some(&rest[..len.min(rest.len())]));
        });
        // A line above the buffer is a NULL string.
        if found.is_none() {
            list.edit().push_bytes(None);
        }
    };
    if slnum == elnum {
        push(slnum, scol, span_len(ecol - scol));
    } else {
        // The first line from the start column on, the whole lines between,
        // and the head of the last.
        push(slnum, scol, usize::MAX);
        for lnum in slnum + 1..elnum {
            push(lnum, 0, usize::MAX);
        }
        push(elnum, 0, span_len(ecol));
    }
    Some(list)
}

/// Call `expr` — a funcref or a partial — with the submatches as its one
/// argument, and return its result's text. `None` when the call failed,
/// which has already reported itself.
pub(crate) fn call_replacement(expr: &TypVal) -> Option<Vec<u8>> {
    // `fill_submatch_list` fills the list in if the function takes an
    // argument at all. Like upstream's static list it is locked and out of
    // the collector's registry: this frame owns it.
    let match_list = tv_list_alloc(ptrdiff_t::try_from(SL_SIZE).unwrap_or(0));
    {
        let l = match_list.edit();
        unroot_list(l.lv_root);
        l.lv_root = RootId::NONE;
        l.lv_lock = VarLock::Fixed;
    }
    let argv = [TypVal::list(Some(match_list.clone()))];

    let mut rettv = TV_INITIAL_VALUE;
    rettv.write_string(None);

    let mut with = CallWith::new(true);
    with.argv_func = Some(fill_submatch_list);
    let name = if expr.v_type() == VAR_FUNC {
        Some(expr.func_name().map_or(c"", ThinCString::as_cstr))
    } else if expr.v_type() == VAR_PARTIAL {
        with.partial = expr.partial_shared();
        Some(expr.partial_ref().map_or(c"", partial_name))
    } else {
        None
    };
    if let Some(name) = name {
        let _ = call_func_with(name, None, &mut rettv, &argv, with);
    }
    drop(argv);
    drop(match_list);

    // An unknown return type means the call failed and has already said
    // so; there is no second error to report.
    let text = if rettv.v_type() == VAR_UNKNOWN {
        None
    } else {
        let mut buf = NumBuf::new();
        buf.string_chk(&rettv).map(|s| s.to_bytes().to_vec())
    };
    tv_clear(&mut rettv);
    text
}
