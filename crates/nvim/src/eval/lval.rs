//! Resolving the left-hand side of an assignment.
//!
//! [`get_lval`] walks a name and its subscripts down to the place a value
//! is written, and answers it as an [`LValue`]; [`set_var_lval`] performs
//! the assignment, `:unlet` and `:lockvar` delete or lock through it, and
//! `islocked()` and the function-name parser read it.
//!
//! **The place is held, not pointed at.** A subscript's index expression is
//! user code, and it may remove the item being indexed, grow the List that
//! holds it past its capacity, unlet the variable, or replace the container
//! outright. So a [`Target`] keeps the *container* by a counted handle and
//! the item by its index or key, and finds the item again whenever it is
//! read or written: a removed item is an error, not freed memory. The
//! container a subscript selects into is read out of its slot *after* that
//! subscript's index expression ran, which is what the C does too (a
//! replaced List is the one written; see the `vars` battery row).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// The write half, which reads the record this one fills in.
mod assign;
pub(crate) use self::assign::*;

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::c_int;
use core::ops::ControlFlow;

use crate::ascii::ascii_iswhite;
use crate::cstr::byte_at;
use crate::eval::typval::{
    BlobRef, DictRef, ListRef, NumBuf, blob_check_index, blob_check_range, blob_len, index_of,
    list_check_range_index_one, list_check_range_index_two, list_items_mut, tv_blob_alloc_ret,
    tv_check_str, tv_dict_alloc, tv_get_number, tv_list_alloc_ret,
};
use crate::eval::userfunc::get_funccal_args_dict;
use crate::eval::vars::{
    clear_local, emsg_static, get_vimvar_dict, valid_varname_named, var_check_lock_named,
    var_check_ro_named, var_wrong_func_name_named, with_var,
};
use crate::eval::{
    Cursor, FNE_INCL_BR, GLV_NO_AUTOLOAD, GLV_QUIET, GLV_READ_ONLY, e_cannot_slice_dictionary,
    e_missbrac, eval1, expanded_name, name_end, tv_is_luafunc,
};
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::memory::XString;
use crate::message::state::emsg_severe;
use crate::types::{
    DictItem, TypVal, VAR_BLOB, VAR_DEF_SCOPE, VAR_DICT, VAR_LIST, VarLock, kListLenUnknown,
};

/// A freshly declared typval.
pub(super) const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// The part of a List or Blob a subscript named: one index, or a range.
#[derive(Clone, Copy, Default)]
pub(crate) struct Span {
    /// The index, or the range's first; a List's is made non-negative.
    pub(crate) n1: c_int,
    /// The range's last index, when it has one.
    pub(crate) n2: c_int,
    /// `[n1:n2]` rather than `[n1]`.
    pub(crate) range: bool,
    /// `[n1:]`: the range runs to the end.
    pub(crate) empty2: bool,
}

/// Where a value lives, held so that it can be found again after user code
/// has run.
pub(crate) enum Slot {
    /// The variable the name resolved to, found again by name.
    Variable,
    /// Item `index` of a List.
    Item { list: ListRef, index: usize },
    /// The item under `key` in a Dictionary.
    Key { dict: DictRef, key: Vec<u8> },
}

/// What a left-hand side resolved to.
pub(crate) enum Target {
    /// A whole variable, by name: nothing was subscripted.
    Variable,
    /// The value in a slot -- a List item (the first of a slice when
    /// `span.range`), a Dictionary item, or the variable itself when a
    /// `.` after its name selected nothing (`v:lua.name`, `x.=`).
    Slot { slot: Slot, span: Span },
    /// A key a Dictionary does not have yet.
    NewKey { dict: DictRef, key: Vec<u8> },
    /// A Blob byte, or a byte range.
    Blob { blob: BlobRef, span: Span },
}

/// A resolved left-hand side.
pub(crate) struct LValue<'a> {
    /// The text it was read from: its first byte to the end of the string
    /// that holds it.
    text: &'a [u8],
    /// The curly-brace name's expansion, when the name had braces.
    expanded: Option<XString>,
    /// Whether there is a name at all. A curly-brace name whose expression
    /// failed quietly has none, and its caller carries on parsing.
    named: bool,
    /// The variable name's length in `text`, before any subscript.
    root_len: usize,
    /// Where in `text` the left-hand side ends.
    end: usize,
    /// The place.
    pub(crate) target: Target,
}

impl<'a> LValue<'a> {
    /// A left-hand side read from `text` that names nothing yet.
    const fn new(text: &'a [u8]) -> Self {
        LValue {
            text,
            expanded: None,
            named: false,
            root_len: 0,
            end: 0,
            target: Target::Variable,
        }
    }

    /// The whole variable that the first `len` bytes of `text` name, which
    /// is how `:unlet $ENV` hands an environment variable to the same
    /// callbacks as the rest.
    pub(crate) const fn variable(text: &'a [u8], len: usize) -> Self {
        LValue {
            text,
            expanded: None,
            named: true,
            root_len: len,
            end: len,
            target: Target::Variable,
        }
    }

    /// Whether there is a name.
    pub(crate) const fn has_name(&self) -> bool {
        self.named
    }

    /// The left-hand side as a message names it: the expansion of a
    /// curly-brace name, or the text from the name to the last subscript.
    pub(crate) fn name(&self) -> &[u8] {
        match &self.expanded {
            Some(expanded) => expanded,
            None => &self.text[..self.end],
        }
    }

    /// The left-hand side and everything after it on the line: what the C
    /// printed where it quoted the name up to its terminator without
    /// cutting it first.
    pub(crate) fn name_and_rest(&self) -> &[u8] {
        match &self.expanded {
            Some(expanded) => expanded,
            None => self.text,
        }
    }

    /// The curly-brace name's expansion.
    pub(crate) const fn expanded(&self) -> Option<&XString> {
        self.expanded.as_ref()
    }

    /// Run `f` over the value in the target's slot and the lock that slot
    /// carries, or answer `None` when the target is not a slot or the slot
    /// has gone. `f` must not run user code: see [`with_var`].
    pub(crate) fn with_slot<R>(
        &mut self,
        f: impl FnOnce(&mut TypVal, &mut VarLock) -> R,
    ) -> Option<R> {
        let Target::Slot { slot, .. } = &mut self.target else {
            return None;
        };
        let name = match &self.expanded {
            Some(expanded) => expanded,
            None => &self.text[..self.root_len],
        };
        slot_value(slot, name, true, |tv, lock| f(tv, lock))
    }
}

/// Run `f` over the value `slot` holds and the slot's lock, or answer `None`
/// when it holds none any more: the variable `name` is gone, the List is
/// shorter, or the key was removed.
fn slot_value<R>(
    slot: &mut Slot,
    name: &[u8],
    no_autoload: bool,
    f: impl FnOnce(&mut TypVal, &mut VarLock) -> R,
) -> Option<R> {
    match slot {
        Slot::Variable => with_var(name, no_autoload, |item| item_value(item, f)),
        Slot::Item { list, index } => list_items_mut(Some(list))
            .get_mut(*index)
            .map(|item| f(&mut item.li_tv, &mut item.li_lock)),
        Slot::Key { dict, key } => dict.find_mut(key).map(|item| item_value(item, f)),
    }
}

/// A dictionary item's value and lock, borrowed apart.
fn item_value<R>(item: &mut DictItem, f: impl FnOnce(&mut TypVal, &mut VarLock) -> R) -> R {
    f(&mut item.di_tv, &mut item.di_lock)
}

/// A container, as a subscript sees it.
enum Container {
    List(Option<ListRef>),
    Dict(Option<DictRef>),
    Blob(Option<BlobRef>),
    /// Anything else: a subscript on it is refused.
    Other,
}

impl Container {
    fn of(tv: &TypVal) -> Self {
        match tv.v_type() {
            VAR_LIST => Container::List(tv.list_handle()),
            VAR_DICT => Container::Dict(tv.dict_handle()),
            VAR_BLOB => Container::Blob(tv.blob_handle()),
            _ => Container::Other,
        }
    }
}

/// How the slot the walk starts at answered.
enum Opened {
    /// The container is ready to be subscripted.
    Ready,
    /// The variable is `v:lua`, which only a function name subscripts.
    LuaFunc,
}

/// What one subscript's key text came to.
enum Key {
    /// A `.key`: the bytes from `start` for `len`.
    Text { start: usize, len: usize },
    /// A `[key]`: the first index expression, as a string.
    Value,
}

/// The subscript walk's state: where the walk is, and the slot holding the
/// container the next subscript selects into.
struct Walk<'t, 'v> {
    text: &'t [u8],
    /// The variable's name, to find it again by.
    root: &'t [u8],
    /// Where in `text` the walk has got to.
    at: usize,
    /// The value about to be assigned, or `None` for an `:unlet` and the
    /// readers, which assign nothing.
    value: Option<&'v mut TypVal>,
    unlet: bool,
    flags: c_int,
    /// `GLV_QUIET`: resolve, but report nothing.
    quiet: bool,
    /// Whether finding the variable the first time may source an autoload
    /// script. A second look never does.
    no_autoload: bool,
    /// Where the container being subscripted lives.
    slot: Slot,
    /// The container in `slot`, as last read.
    container: Container,
    /// Whether user code may have run since `container` was read out of
    /// `slot`: an index expression was evaluated.
    stale: bool,
    /// The last List or Blob subscript.
    span: Span,
}

impl Walk<'_, '_> {
    /// Which subscript `at` opens -- `.` for a key, `[` for an index -- or
    /// `None` when it opens none and the walk is over. `.=` and `..` are
    /// the concat operators, not a key.
    fn opener(&self) -> Option<u8> {
        let c = byte_at(self.text, self.at);
        let next = byte_at(self.text, self.at + 1);
        let opens = c == b'[' || (c == b'.' && next != b'=' && next != b'.');
        opens.then_some(c)
    }

    /// The text from `at` on, for the messages that quote the rest.
    fn rest(&self, at: usize) -> &[u8] {
        self.text.get(at..).unwrap_or_default()
    }

    /// Run `f` over the slot's value, reporting a slot that has gone.
    fn with_slot<R>(&mut self, f: impl FnOnce(&mut TypVal, &mut VarLock) -> R) -> Option<R> {
        let found = slot_value(&mut self.slot, self.root, self.no_autoload, f);
        self.no_autoload = true;
        if found.is_none() && !self.quiet {
            match &self.slot {
                Slot::Variable => {
                    let name = msg_bytes(self.root);
                    semsg!("E121: Undefined variable: {name}");
                }
                Slot::Item { index, .. } => {
                    let index = i64::from(index_of(*index));
                    semsg!("E684: List index out of range: {index}");
                }
                Slot::Key { key, .. } => {
                    let key = msg_bytes(key);
                    semsg!("E716: Key not present in Dictionary: \"{key}\"");
                }
            }
        }
        found
    }

    /// Read the container out of the slot again if an index expression has
    /// run since it was last read.
    fn refresh(&mut self) -> Option<()> {
        if self.stale {
            self.container = self.with_slot(|tv, _| Container::of(tv))?;
            self.stale = false;
        }
        Some(())
    }

    /// The checks a container has to pass before a subscript may select
    /// into it, and the empty List or Blob that a null one stands in for.
    /// The first call is the one that finds the variable, and the one that
    /// notices `v:lua`.
    fn open(&mut self, opener: u8, first: bool) -> Option<Opened> {
        enum Refused {
            Dot,
            Index,
        }
        let checked = self.with_slot(|tv, _| {
            if first && tv_is_luafunc(tv) {
                return Ok(None);
            }
            let kind = tv.v_type();
            if opener == b'.' && kind != VAR_DICT {
                return Err(Refused::Dot);
            }
            if kind != VAR_LIST && kind != VAR_DICT && kind != VAR_BLOB {
                return Err(Refused::Index);
            }
            // A null List or Blob works like an empty one; allocate now.
            if kind == VAR_LIST && tv.list_ref().is_none() {
                tv_list_alloc_ret(tv, kListLenUnknown.try_into().unwrap_or(-1));
            } else if kind == VAR_BLOB && tv.blob_ref().is_none() {
                tv_blob_alloc_ret(tv);
            }
            Ok(Some(Container::of(tv)))
        })?;
        let container = match checked {
            Ok(Some(container)) => container,
            Ok(None) => return Some(Opened::LuaFunc),
            Err(Refused::Dot) => {
                if !self.quiet {
                    let name = msg_bytes(self.text);
                    semsg!("E1203: Dot can only be used on a dictionary: {name}");
                }
                return None;
            }
            Err(Refused::Index) => {
                if !self.quiet {
                    emsg_static(c"E689: Can only index a List, Dictionary or Blob");
                }
                return None;
            }
        };
        self.container = container;
        self.stale = false;

        if self.span.range {
            if !self.quiet {
                emsg_static(c"E708: [:] must come last");
            }
            return None;
        }
        Some(Opened::Ready)
    }

    /// The `.key` at `at`: a run of name characters.
    fn parse_key(&mut self) -> Option<Key> {
        let start = self.at + 1;
        let len = self.text[start.min(self.text.len())..]
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
            .count();
        if len == 0 {
            if !self.quiet {
                emsg_static(c"E713: Cannot use empty key after .");
            }
            return None;
        }
        self.at = start + len;
        Some(Key::Text { start, len })
    }

    /// Step past the white space at `at`.
    fn skip_white(&mut self) {
        self.at += crate::charset::skip::white(self.rest(self.at));
    }

    /// The `[expr]` or `[expr : expr]` at `at`, evaluated into `var1` and
    /// `var2`. A range sets `span.range`, and `span.empty2` for a `[n:]`.
    fn parse_index(
        &mut self,
        var1: &mut TypVal,
        var2: &mut TypVal,
        empty1: &mut bool,
    ) -> Option<Key> {
        self.at += 1;
        self.skip_white();
        *empty1 = byte_at(self.text, self.at) == b':';
        if !*empty1 {
            self.eval_into(var1)?;
            self.skip_white();
        }

        if byte_at(self.text, self.at) == b':' {
            self.parse_range(var2)?;
        } else {
            self.span.range = false;
        }

        if byte_at(self.text, self.at) != b']' {
            if !self.quiet {
                emsg_static(e_missbrac);
            }
            return None;
        }
        self.at += 1;
        Some(Key::Value)
    }

    /// One index expression, evaluated at `at` into `var` and checked for
    /// being usable as a string.
    fn eval_into(&mut self, var: &mut TypVal) -> Option<()> {
        let mut cursor = Cursor::new(self.text);
        cursor.set_offset(self.at);
        let evaluated = eval1(&mut cursor, var, true);
        self.at = cursor.offset();
        self.stale = true;
        (evaluated.is_ok() && tv_check_str(var)).then_some(())
    }

    /// The `: expr]` half of a `[n : m]`, with `at` on the colon.
    fn parse_range(&mut self, var2: &mut TypVal) -> Option<()> {
        self.refresh()?;
        if matches!(self.container, Container::Dict(_)) {
            if !self.quiet {
                emsg_static(e_cannot_slice_dictionary);
            }
            return None;
        }
        // The value being assigned has to be sliceable too. No value is
        // `:unlet`, which assigns nothing.
        let sliceable = self.value.as_deref().is_none_or(|v| {
            (v.v_type() == VAR_LIST && v.list_ref().is_some())
                || (v.v_type() == VAR_BLOB && v.blob_ref().is_some())
        });
        if !sliceable {
            if !self.quiet {
                emsg_static(c"E709: [:] requires a List or Blob value");
            }
            return None;
        }
        // Past the `:` and the white space after it.
        self.at += 1;
        self.skip_white();
        if byte_at(self.text, self.at) == b']' {
            self.span.empty2 = true;
        } else {
            self.span.empty2 = false;
            self.eval_into(var2)?;
        }
        self.span.range = true;
        Some(())
    }

    /// Select what the subscript just parsed names out of the container,
    /// moving the slot onto it. `Break` with the target when there is
    /// nothing left to descend into: a Blob byte, or a key that does not
    /// exist yet.
    fn descend(
        &mut self,
        key: &Key,
        var1: &TypVal,
        var2: &TypVal,
        empty1: bool,
    ) -> Option<ControlFlow<Target>> {
        self.refresh()?;
        match core::mem::replace(&mut self.container, Container::Other) {
            Container::Dict(dict) => self.dict_item(dict, key, var1),
            Container::Blob(blob) => {
                let blob = match blob {
                    Some(blob) => blob,
                    None => self.with_slot(|tv, _| {
                        tv_blob_alloc_ret(tv);
                        tv.blob_handle().expect("a blob, just allocated")
                    })?,
                };
                self.blob(&blob, var1, var2, empty1)?;
                // A Blob byte is never a container, so this is the end.
                Some(ControlFlow::Break(Target::Blob {
                    blob,
                    span: self.span,
                }))
            }
            Container::List(list) => self.list_item(list, var1, var2, empty1),
            // The index expression put something else in the slot: no
            // List has that index.
            Container::Other => self.list_item(None, var1, var2, empty1),
        }
    }

    /// Resolve one `.key` or `[key]` against `dict`.
    fn dict_item(
        &mut self,
        dict: Option<DictRef>,
        key: &Key,
        var1: &TypVal,
    ) -> Option<ControlFlow<Target>> {
        let mut numbuf = NumBuf::new();
        let key_bytes: Vec<u8> = match *key {
            Key::Text { start, len } => self.text[start..start + len].to_vec(),
            Key::Value => numbuf.string(var1).to_bytes().to_vec(),
        };
        // A null Dict is an empty Dict; allocate one into the slot now.
        let dict = match dict {
            Some(dict) => dict,
            None => {
                let fresh = tv_dict_alloc();
                let held = fresh.clone();
                self.with_slot(|tv, _| tv.write_dict(Some(fresh)))?;
                held
            }
        };
        let existing = dict
            .find(&key_bytes)
            .map(|item| (item.di_flags, tv_is_luafunc(&item.di_tv)));

        // Assigning into a scope dictionary: check that the name is a valid
        // variable name, and a valid *function* name too unless the scope is
        // `l:` or `g:`. Overwriting a builtin function is not allowed.
        let scope = dict.dv_scope;
        if let Some(value) = self.value.as_deref()
            && scope != 0
        {
            let wrong = (scope == VAR_DEF_SCOPE
                && value.is_func()
                && var_wrong_func_name_named(&key_bytes, existing.is_none()))
                || !valid_varname_named(&key_bytes);
            if wrong {
                return None;
            }
        }

        let is_value_key = matches!(key, Key::Value);
        if existing.is_some_and(|(_, lua)| lua) && is_value_key && self.value.is_none() {
            semsg!("E461: Illegal variable name: v:['lua']");
            return None;
        }

        let Some((flags, _)) = existing else {
            // A "v:" or "a:" variable cannot be added.
            if dict.as_ptr() == get_vimvar_dict() || dict.as_ptr() == get_funccal_args_dict() {
                let name = msg_bytes(self.text);
                semsg!("E461: Illegal variable name: {name}");
                return None;
            }
            // The key does not exist. It may be added -- unless something
            // follows it to subscript, or this is an `:unlet`.
            let after = byte_at(self.text, self.at);
            if after == b'[' || after == b'.' || self.unlet {
                if !self.quiet {
                    // A `.key` is quoted to the end of the line, as the C
                    // quoted it straight out of the command.
                    let shown = match *key {
                        Key::Text { start, .. } => self.rest(start),
                        Key::Value => &key_bytes,
                    };
                    let shown = msg_bytes(shown);
                    semsg!("E716: Key not present in Dictionary: \"{shown}\"");
                }
                return None;
            }
            return Some(ControlFlow::Break(Target::NewKey {
                dict,
                key: key_bytes,
            }));
        };

        // An existing item: check it may be changed.
        let flags = c_int::from(flags);
        let named = &self.text[..self.at];
        let refused = self.flags & GLV_READ_ONLY.cast_signed() == 0
            && (var_check_ro_named(flags, named) || var_check_lock_named(flags, named));
        if refused {
            return None;
        }
        self.slot = Slot::Key {
            dict,
            key: key_bytes,
        };
        Some(ControlFlow::Continue(()))
    }

    /// Resolve a `[n]` or `[n:m]` against `blob`.
    fn blob(&mut self, blob: &BlobRef, var1: &TypVal, var2: &TypVal, empty1: bool) -> Option<()> {
        let bloblen = blob_len(Some(blob));
        self.span.n1 = if empty1 {
            0
        } else {
            index_of(tv_get_number(var1))
        };
        let n1 = i64::from(self.span.n1);
        blob_check_index(bloblen, n1, self.quiet).ok()?;
        if self.span.range && !self.span.empty2 {
            self.span.n2 = index_of(tv_get_number(var2));
            let n2 = i64::from(self.span.n2);
            blob_check_range(bloblen, n1, n2, self.quiet).ok()?;
        }
        Some(())
    }

    /// Resolve a `[n]` or `[n:m]` against `list`, moving the slot onto the
    /// item it selected.
    fn list_item(
        &mut self,
        list: Option<ListRef>,
        var1: &TypVal,
        var2: &TypVal,
        empty1: bool,
    ) -> Option<ControlFlow<Target>> {
        self.span.n1 = if empty1 {
            0
        } else {
            index_of(tv_get_number(var1))
        };
        let quiet = self.quiet;
        let at = list_check_range_index_one(list.as_deref(), &mut self.span.n1, quiet)?;
        if self.span.range && !self.span.empty2 {
            self.span.n2 = index_of(tv_get_number(var2));
            let span = &mut self.span;
            list_check_range_index_two(list.as_deref(), &mut span.n1, at, &mut span.n2, quiet)
                .ok()?;
        }
        let list = list.expect("an index of the list was found");
        self.slot = Slot::Item { list, index: at };
        Some(ControlFlow::Continue(()))
    }

    /// Every subscript from `at` on, one container at a time. `var1` and
    /// `var2` are the caller's, so that a refusal part-way through still
    /// leaves whichever was evaluated for it to release.
    fn walk(&mut self, var1: &mut TypVal, var2: &mut TypVal) -> Option<ControlFlow<(), Target>> {
        // The variable is looked up whatever follows its name: an undefined
        // one is E121 even before a `.=`, and `v:lua` stops the walk.
        let Some(mut opener) = self.opener() else {
            if self.with_slot(|tv, _| tv_is_luafunc(tv))? {
                return Some(ControlFlow::Break(()));
            }
            return Some(ControlFlow::Continue(Target::Slot {
                slot: Slot::Variable,
                span: self.span,
            }));
        };
        let mut first = true;
        loop {
            if let Opened::LuaFunc = self.open(opener, first)? {
                return Some(ControlFlow::Break(()));
            }
            first = false;
            let mut empty1 = false;
            let key = if opener == b'.' {
                self.parse_key()?
            } else {
                self.parse_index(var1, var2, &mut empty1)?
            };
            if let ControlFlow::Break(target) = self.descend(&key, var1, var2, empty1)? {
                return Some(ControlFlow::Continue(target));
            }
            clear_local(var1);
            clear_local(var2);
            match self.opener() {
                Some(next) => opener = next,
                None => break,
            }
        }
        let slot = core::mem::replace(&mut self.slot, Slot::Variable);
        Some(ControlFlow::Continue(Target::Slot {
            slot,
            span: self.span,
        }))
    }
}

/// Resolve the left-hand side `text` starts with, for an assignment of
/// `value` or (with none) an `:unlet` or a reader. Answers what it resolved
/// and where it ended in `text`; an end of `None` is an error already
/// reported, and the left-hand side may still have a name, which is how a
/// caller tells "carry on parsing" from "stop".
///
/// `skip` only finds the end of the name: nothing is evaluated or looked
/// up.
pub(crate) fn get_lval<'a>(
    text: &'a [u8],
    value: Option<&mut TypVal>,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> (LValue<'a>, Option<usize>) {
    let quiet = flags & GLV_QUIET.cast_signed() != 0;
    let mut lval = LValue::new(text);

    if skip {
        // Only the name matters; nothing is resolved.
        let end = name_end(text, FNE_INCL_BR | fne_flags).end;
        lval.named = true;
        lval.root_len = end;
        lval.end = end;
        return (lval, Some(end));
    }

    let found = name_end(text, fne_flags);
    let p = found.end;
    let after = byte_at(text, p);
    if let Some(open) = found.brace_open {
        // A curly-braces name: expand it, unless there is an error
        // already.
        if unlet
            && !ascii_iswhite(c_int::from(after))
            && ends_excmd(c_int::from(after)) == 0
            && after != b'['
            && after != b'.'
        {
            let rest = msg_bytes(&text[p..]);
            semsg!("E488: Trailing characters: {rest}");
            return (lval, None);
        }
        lval.expanded = expanded_name(&text[..p], open, found.brace_close);
        if lval.expanded.is_none() {
            if !aborting() && !quiet {
                emsg_severe.set(true);
                let name = msg_bytes(text);
                semsg!("E475: Invalid argument: {name}");
                return (lval, None);
            }
        } else {
            lval.named = true;
        }
    } else {
        lval.named = true;
    }
    lval.root_len = p;
    lval.end = p;

    // Nothing is subscripted: the name is the whole left-hand side.
    if (after != b'[' && after != b'.') || !lval.named {
        return (lval, Some(p));
    }

    let LValue { expanded, .. } = &lval;
    let root = match expanded {
        Some(expanded) => expanded,
        None => &text[..p],
    };
    let mut walk = Walk {
        text,
        root,
        at: p,
        value,
        unlet,
        flags,
        quiet,
        // Only a write keeps the lookup from sourcing an autoload script.
        no_autoload: flags & (GLV_NO_AUTOLOAD | GLV_READ_ONLY).cast_signed()
            != GLV_READ_ONLY.cast_signed(),
        slot: Slot::Variable,
        container: Container::Other,
        stale: false,
        span: Span::default(),
    };
    // The two index expressions. They outlive the walk, so a refusal
    // part-way through still releases whichever of them was evaluated.
    let mut var1 = UNSET_TV;
    let mut var2 = UNSET_TV;
    let walked = walk.walk(&mut var1, &mut var2);
    clear_local(&mut var1);
    clear_local(&mut var2);
    let end = walk.at;
    match walked {
        None => (lval, None),
        Some(ControlFlow::Break(())) => {
            // `v:lua`: the `.` and the name after it are the caller's.
            lval.target = Target::Slot {
                slot: Slot::Variable,
                span: Span::default(),
            };
            (lval, Some(p))
        }
        Some(ControlFlow::Continue(target)) => {
            lval.target = target;
            lval.end = end;
            (lval, Some(end))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::typval::tv_list_alloc;
    use crate::global_cell::editor_state_lock;
    use crate::types::VarNumber;

    /// The number in `slot`, or `None` when the slot has gone. A List or
    /// Dict slot never looks a variable up, so the name is not read.
    fn read(slot: &mut Slot) -> Option<VarNumber> {
        slot_value(slot, b"", true, |tv, _| tv.number_or_zero())
    }

    /// Write `n` into `slot`, answering whether it was still there.
    fn write(slot: &mut Slot, n: VarNumber) -> bool {
        slot_value(slot, b"", true, |tv, lock| {
            *tv = TypVal::Number(n);
            *lock = VarLock::Unlocked;
        })
        .is_some()
    }

    /// The item array moving -- the List growing past its capacity, an item
    /// before it going -- is what used to leave a target pointing at freed
    /// memory. The slot holds the List and an index, so it reads whatever
    /// is at that index now, and nothing once the List is shorter.
    #[test]
    fn a_list_slot_is_found_again_after_its_items_move() {
        let _held = editor_state_lock();
        let mut list = tv_list_alloc(2);
        list.push_number(1);
        list.push_number(2);
        let mut slot = Slot::Item {
            list: list.clone(),
            index: 1,
        };
        for n in 0..100 {
            list.push_number(n);
        }
        assert_eq!(read(&mut slot), Some(2));
        assert!(write(&mut slot, 20));
        assert_eq!(
            list_items_mut(Some(&mut list))[1].li_tv.number_or_zero(),
            20
        );

        list.remove_at(0);
        assert_eq!(
            read(&mut slot),
            Some(0),
            "the index names the next item now"
        );

        let last = list.len() - 1;
        list.remove_range(0, last);
        assert_eq!(read(&mut slot), None);
        assert!(!write(&mut slot, 5));
    }

    /// A Dict slot holds a reference of its own: the item going is a miss,
    /// and every other reference going leaves the dictionary alive for the
    /// slot to find the key again in.
    #[test]
    fn a_dict_slot_outlives_its_item_and_the_other_references() {
        let _held = editor_state_lock();
        let mut dict = tv_dict_alloc();
        dict.add_number(b"a", 7).expect("a fresh key");
        let mut slot = Slot::Key {
            dict: dict.clone(),
            key: b"a".to_vec(),
        };
        assert_eq!(read(&mut slot), Some(7));

        assert!(dict.remove_key(b"a").is_some());
        assert_eq!(read(&mut slot), None);

        dict.add_number(b"a", 8).expect("the key is free again");
        drop(dict);
        assert_eq!(read(&mut slot), Some(8));
        assert!(write(&mut slot, 9));
        assert_eq!(read(&mut slot), Some(9));
    }
}
