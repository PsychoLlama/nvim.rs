//! The `:syntax` item options, and the containment test.
//!
//! [`get_syn_options`] parses the flag words (`contained`, `oneline`, `keepend`,
//! `conceal`, `nextgroup=`, ...) that may follow any item definition, and
//! [`get_id_list`] parses a group list (`contains=a,b,@cl,ALLBUT,TOP`) into the
//! `int16_t` id array the state machine tests against. [`in_id_list`] is that
//! test -- it runs once per candidate pattern per column, so it is on the
//! per-cell path even though the rest of this module is not.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::skip;
use crate::cstr;
use crate::mbyte::{char_at, cluster_len};
use crate::memory::ThinCString;
use crate::narrow::as_index;
use crate::narrow::len_as_int;
use crate::types::Failed;
use core::ffi::{CStr, c_int};

use super::*;
use crate::ascii::NUL;
use crate::regexp::RE_MAGIC;
use crate::winlayer::Win;

/// Where a `:syntax` command's group name ends and its next argument begins.
#[derive(Debug)]
pub(crate) struct GroupName {
    /// How many bytes of the line the name itself takes.
    pub(crate) len: usize,
    /// Offset of the first argument after the name.
    pub(crate) rest: usize,
}

/// Split off a `:syntax` command's group-name argument.
///
/// `line` is the rest of the command line. Answers `None` when the command
/// ended instead of naming an argument; the argument may be a pattern, in
/// which `|` is allowed, so only a NUL counts as the end.
pub(crate) fn split_group_name(line: &[u8]) -> Option<GroupName> {
    let len = skip::to_white(line);
    let rest = len + skip::white(&line[len..]);
    if ends_excmd(byte(line, 0)) != 0 || byte(line, rest) == NUL {
        return None;
    }
    Some(GroupName { len, rest })
}

/// The byte at `at`, as the pointer walk this replaces read it.
///
/// `line` is the rest of a command line, so its length *is* the terminator's
/// offset and a read at or past it answers NUL — which is what every
/// `ends_excmd`/`ascii_iswhite` test below is asking about.
fn byte(line: &[u8], at: usize) -> c_int {
    c_int::from(cstr::byte_at(line, at))
}

/// What an option word takes after its name.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum OptArg {
    /// Nothing: a bare flag word that ORs its `HL_*` into the item's flags.
    Flag,
    /// `contains=`
    Contains,
    /// `containedin=`
    ContainedIn,
    /// `nextgroup=`
    NextGroup,
    /// `cchar=`
    Cchar,
}

impl OptArg {
    /// Whether the name must be followed by `=` rather than by the end of the
    /// argument.
    fn takes_value(self) -> bool {
        self != OptArg::Flag
    }
}

/// One recognised option word.
#[derive(Debug)]
struct SynFlag {
    name: &'static CStr,
    arg: OptArg,
    /// The `HL_*` a bare flag word sets; 0 for the ones that take a value.
    flags: SynFlags,
}

/// A `const fn` constructor keeps each entry on one line under rustfmt.
const fn flag(name: &'static CStr, arg: OptArg, flags: SynFlags) -> SynFlag {
    SynFlag { name, arg, flags }
}

/// Every option word any `:syntax` item definition accepts.
///
/// Searched **last to first**, which is the order upstream's `--fidx` loop
/// walks it. Nothing here depends on that order — every name that is a prefix
/// of another (`conceal`/`concealends`, `contained`/`containedin`) is
/// separated by the "followed by white space, `=` or the end of the command"
/// test below — but the order is kept as documentation of that fact.
static FLAG_TAB: [SynFlag; 19] = [
    flag(c"contained", OptArg::Flag, SynFlags::CONTAINED),
    flag(c"oneline", OptArg::Flag, SynFlags::ONELINE),
    flag(c"keepend", OptArg::Flag, SynFlags::KEEPEND),
    flag(c"extend", OptArg::Flag, SynFlags::EXTEND),
    flag(c"excludenl", OptArg::Flag, SynFlags::EXCLUDENL),
    flag(c"transparent", OptArg::Flag, SynFlags::TRANSP),
    flag(c"skipnl", OptArg::Flag, SynFlags::SKIPNL),
    flag(c"skipwhite", OptArg::Flag, SynFlags::SKIPWHITE),
    flag(c"skipempty", OptArg::Flag, SynFlags::SKIPEMPTY),
    flag(c"grouphere", OptArg::Flag, SynFlags::SYNC_HERE),
    flag(c"groupthere", OptArg::Flag, SynFlags::SYNC_THERE),
    flag(c"display", OptArg::Flag, SynFlags::DISPLAY),
    flag(c"fold", OptArg::Flag, SynFlags::FOLD),
    flag(c"conceal", OptArg::Flag, SynFlags::CONCEAL),
    flag(c"concealends", OptArg::Flag, SynFlags::CONCEALENDS),
    flag(c"cchar", OptArg::Cchar, SynFlags::NONE),
    flag(c"contains", OptArg::Contains, SynFlags::NONE),
    flag(c"containedin", OptArg::ContainedIn, SynFlags::NONE),
    flag(c"nextgroup", OptArg::NextGroup, SynFlags::NONE),
];

/// Could `c` start an option word? A cheap reject, because this runs for every
/// word of a `:syntax keyword` command with a large keyword list.
///
/// Upstream spells this as `strchr(first_letters, *arg)`, which answers the
/// terminator for a NUL `*arg` and so scans the whole table once for nothing;
/// the outcome is the same either way.
fn starts_option(c: u8) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        b'c' | b'o' | b'k' | b'e' | b't' | b's' | b'g' | b'd' | b'f' | b'n'
    )
}

/// Does `rest` begin with `f`'s name, followed by what `f` requires?
///
/// The comparison is ASCII-case-insensitive, which is what upstream's
/// doubled-case name table (`"cCoOnNtTaAiInNeEdD"`) spells out a byte at a
/// time. The end of the line never matches a letter, so a name that runs
/// past it is no match.
fn flag_matches(rest: &[u8], f: &SynFlag) -> bool {
    let name = f.name.to_bytes();
    if !rest
        .get(..name.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(name))
    {
        return false;
    }
    let after = byte(rest, name.len());
    ascii_iswhite(after)
        || if f.arg.takes_value() {
            after == c_int::from(b'=')
        } else {
            ends_excmd(after) != 0
        }
}

/// Which option word `rest` starts with, if any.
///
/// `keyword` is set while parsing `:syntax keyword`, where `display`, `fold`
/// and `extend` are keywords rather than options — a match on one of those is
/// reported as no match at all, which stops option parsing right there.
fn find_flag(rest: &[u8], keyword: bool) -> Option<&'static SynFlag> {
    let f = FLAG_TAB.iter().rev().find(|f| flag_matches(rest, f))?;
    if keyword
        && (f.flags == SynFlags::DISPLAY
            || f.flags == SynFlags::FOLD
            || f.flags == SynFlags::EXTEND)
    {
        return None;
    }
    Some(f)
}

/// Read the item options at `line[at..]`, answering the offset of the first
/// argument that is not one, or why one was refused.
///
/// Callable at any point in an argument list and repeatedly, so that options
/// before, between and after the patterns of a `:syntax region` all land in
/// the same [`SynOptArg`].
pub(crate) fn read_item_options(
    line: &[u8],
    mut at: usize,
    opt: &mut SynOptArg,
    conceal_char: &mut c_int,
    skip: bool,
) -> Result<usize, SyntaxError> {
    if cur_syn_block().b_syn_conceal != 0 {
        opt.flags |= SynFlags::CONCEAL;
    }

    while starts_option(cstr::byte_at(line, at)) {
        let Some(f) = find_flag(&line[at..], opt.keyword) else {
            // Not an option word after all: the caller reads whatever this
            // is. A `:syntax keyword` argument reaches here for every
            // keyword that happens to start with an option's first letter.
            break;
        };
        match f.arg {
            OptArg::Contains => {
                if !opt.has_cont_list {
                    return Err(SyntaxError::ContainsNotAccepted);
                }
                at = read_id_list(line, at, 8, &mut opt.cont_list, skip).map_err(|e| e.why)?;
            }
            OptArg::ContainedIn => {
                at = read_id_list(line, at, 11, &mut opt.cont_in_list, skip).map_err(|e| e.why)?;
            }
            OptArg::NextGroup => {
                at = read_id_list(line, at, 9, &mut opt.next_list, skip).map_err(|e| e.why)?;
            }
            OptArg::Cchar => {
                // `cchar` is five letters and `flag_matches` already
                // required the `=`, so the character starts at `at + 6`.
                let cchar = &line[(at + 6).min(line.len())..];
                *conceal_char = char_at(cchar);
                if !vim_isprintc(*conceal_char) {
                    return Err(SyntaxError::InvalidCchar);
                }
                // Upstream advances by `utfc_ptr2len(arg + 6) - 1` *before*
                // this test and by seven after it; at the end of the line
                // that length is zero and the intermediate pointer is one
                // before the argument, which nothing reads. Ordering the
                // test first says the same thing without the underflow.
                at += 6 + cluster_len(cchar);
                at += skip::white(&line[at.min(line.len())..]);
            }
            OptArg::Flag => {
                opt.flags |= f.flags;
                at += f.name.count_bytes();
                at += skip::white(&line[at..]);
                if f.flags == SynFlags::SYNC_HERE || f.flags == SynFlags::SYNC_THERE {
                    at = read_sync_group(line, at, opt)?;
                } else if f.flags == SynFlags::FOLD && foldmethod_is_syntax(Win::current()) {
                    fold_update_all(Win::current()); // Need to update folds later.
                }
            }
        }
    }
    Ok(at)
}

/// Read the group name after `grouphere`/`groupthere` and record the pattern
/// index it names in `opt.sync_idx`.
///
/// Answers the offset of what follows it.
fn read_sync_group(line: &[u8], at: usize, opt: &mut SynOptArg) -> Result<usize, SyntaxError> {
    if !opt.takes_sync_idx {
        return Err(SyntaxError::SyncGroupNotAccepted);
    }
    let end = at + skip::to_white(&line[at..]);
    if end == at {
        return Err(SyntaxError::Unexplained);
    }
    let name = &line[at..end];

    if name == b"NONE" {
        opt.sync_idx = NONE_IDX;
    } else {
        // The named group has to already have a region START item: this is
        // an index into the pattern array, not an id.
        let owned = cstr::owned(name);
        // SAFETY: an owned NUL-terminated copy of the name.
        let syn_id = syn_name2id(&owned);
        let block = cur_syn_block();
        let found = block.patterns().iter().rposition(|spp| {
            c_int::from(spp.sp_syn.id) == syn_id && c_int::from(spp.sp_type) == SPTYPE_START
        });
        match found {
            Some(i) => opt.sync_idx = len_as_int(i),
            None => return Err(SyntaxError::NoRegionItem(ThinCString::from_bytes(name))),
        }
    }

    Ok(end + skip::white(&line[end..]))
}

/// What one pass of [`parse_id_list`] found.
#[derive(Debug)]
struct IdListPass {
    /// The ids, in the order the list named them.
    ids: Vec<int16_t>,
    /// Where the scan stopped. Answered even on failure — `:syntax cluster`
    /// reports its error against it.
    end: usize,
    /// Why the whole list is to be discarded.
    failed: Option<SyntaxError>,
}

/// A group list that was refused, and where the scan stopped.
#[derive(Debug)]
pub(crate) struct ListRefused {
    /// The offset `:syntax cluster` reports its E475 against.
    pub(crate) at: usize,
    pub(crate) why: SyntaxError,
}

/// Turn a `contains=`-style group list into a list of ids.
///
/// `at` stands on the keyword; the answer is the offset past the list, or
/// why it was refused and where the scan stopped. An existing `list` is
/// kept and the newly parsed one discarded.
pub(crate) fn read_id_list(
    line: &[u8],
    at: usize,
    keylen: usize,
    list: &mut IdList,
    skip: bool,
) -> Result<usize, ListRefused> {
    // The list is parsed more than once. A name that is a regexp matches
    // the group table as it stands, and a *later* name in the same list
    // can create a group that the regexp would also have matched
    // ("contains=a.*b,axb"), so the pass has to be repeated until it stops
    // growing. Upstream spells this as a two-round loop that resets its
    // own counter back to round 1.
    let mut previous: Option<usize> = None;
    let pass = loop {
        let pass = parse_id_list(line, at, keylen, skip);
        if pass.failed.is_some() {
            break pass;
        }
        match previous {
            Some(n) if pass.ids.len() <= n => break pass,
            _ => previous = Some(pass.ids.len()),
        }
    };

    if let Some(why) = pass.failed {
        return Err(ListRefused { at: pass.end, why });
    }
    // An already-parsed list is kept; upstream allocates the second one
    // and frees it again.
    if list.is_none() {
        *list = IdList::from_ids(&pass.ids);
    }
    Ok(pass.end)
}

/// One pass over the `keyword=a,b,@cl` at `line[at..]`.
fn parse_id_list(line: &[u8], at: usize, keylen: usize, skip: bool) -> IdListPass {
    let mut ids: Vec<int16_t> = Vec::new();
    let after_key = (at + keylen).min(line.len());
    let mut p = after_key + skip::white(&line[after_key..]);

    if byte(line, p) != c_int::from(b'=') {
        return IdListPass {
            ids,
            end: p,
            failed: Some(SyntaxError::MissingEqualSign(ThinCString::from_bytes(
                &line[at..],
            ))),
        };
    }
    p += 1; // the `=` itself; it is not the terminator, so this is in range
    p += skip::white(&line[p..]);
    if ends_excmd(byte(line, p)) != 0 {
        return IdListPass {
            ids,
            end: p,
            failed: Some(SyntaxError::EmptyArgument(ThinCString::from_bytes(
                &line[at..],
            ))),
        };
    }

    loop {
        let mut end = p;
        while !matches!(cstr::byte_at(line, end), 0 | b',') && !ascii_iswhite(byte(line, end)) {
            end += 1;
        }

        match parse_id_name(line, at, p, end, skip, &mut ids) {
            Ok(Some(id)) => ids.push(id16(id)),
            // A regexp name pushed its own matches, or `skip` is on.
            Ok(None) => {}
            Err(why) => {
                return IdListPass {
                    ids,
                    end: p,
                    failed: Some(why),
                };
            }
        }

        p = end + skip::white(&line[end..]);
        if byte(line, p) != c_int::from(b',') {
            break;
        }
        p += 1; // skip the comma between arguments; it is not the terminator
        p += skip::white(&line[p..]);
        if ends_excmd(byte(line, p)) != 0 {
            break;
        }
    }

    IdListPass {
        ids,
        end: p,
        failed: None,
    }
}

/// Resolve one name of a group list.
///
/// `at` stands on the list's keyword, whose first letter decides whether
/// `ALL`/`TOP`/... are accepted; `line[start..end]` is the name. Answers the
/// id to add, `None` when the name added its own (a regexp) or added nothing
/// (`@cluster` while skipping), or why the name was refused.
fn parse_id_name(
    line: &[u8],
    at: usize,
    start: usize,
    end: usize,
    skip: bool,
    ids: &mut Vec<int16_t>,
) -> Result<Option<c_int>, SyntaxError> {
    let text_len = end - start;
    // Leave room in front for the `^` and behind for the `$` the regexp
    // form needs.
    let mut name: Vec<u8> = Vec::with_capacity(text_len + 3);
    name.push(b'^');
    name.extend_from_slice(&line[start..end]);
    name.push(0);
    let text = &name[1..1 + text_len];

    if text == b"ALLBUT" || text == b"ALL" || text == b"TOP" || text == b"CONTAINED" {
        // Only `contains=` and `containedin=` accept these, which is what
        // upstream tests by the keyword's first letter.
        if !cstr::byte_at(line, at).eq_ignore_ascii_case(&b'C') {
            return Err(SyntaxError::NotAllowedHere(ThinCString::from_bytes(text)));
        }
        if !ids.is_empty() {
            return Err(SyntaxError::MustBeFirst(ThinCString::from_bytes(text)));
        }
        let base = match text[0] {
            b'A' => SYNID_ALLBUT,
            b'T' => SYNID_TOP,
            _ => SYNID_CONTAINED,
        };
        return Ok(Some(base + current_syn_inc_tag.get()));
    }

    if text.first() == Some(&b'@') {
        if skip {
            return Ok(None);
        }
        // The cluster table says why it refused the name; E409 follows.
        return match syn_check_cluster(&text[1..]) {
            Ok(id) => Ok(Some(id)),
            Err(Failed) => Err(unknown_group(&line[start..])),
        };
    }

    if !text.iter().any(|b| b"\\.*^$~[".contains(b)) {
        // The group table says why it refused the name; E409 follows.
        return match syn_check_group(text) {
            0 => Err(unknown_group(&line[start..])),
            id => Ok(Some(id)),
        };
    }

    // A regexp matching group names: add every group it matches.
    name.pop();
    name.push(b'$');
    name.push(0);
    let mut regmatch = RegMatch::new(vim_regcomp(cstr::in_bytes(&name), RE_MAGIC), true);
    if regmatch.regprog.is_none() {
        // The compiler has said why.
        return Err(SyntaxError::Reported(Failed));
    }
    let mut matched = false;
    let mut i = highlight_num_groups();
    while i > 0 {
        i -= 1;
        if vim_regexec(&mut regmatch, syn_id2name(i + 1), 0) {
            ids.push(id16(i + 1));
            matched = true;
        }
    }
    drop(regmatch);
    if !matched {
        return Err(unknown_group(&line[start..]));
    }
    Ok(None)
}

/// E409 for the name at the start of `text`.
fn unknown_group(text: &[u8]) -> SyntaxError {
    SyntaxError::UnknownGroup(ThinCString::from_bytes(text))
}

impl SynState {
    /// Is the syntax group `ssp` in the id list `list` of the item at
    /// `cur_si`?
    ///
    /// `cur_si` is the current item, or `None` when the `containedin` list
    /// is not being checked. This runs once per candidate pattern per column:
    /// keep it fast.
    pub(crate) fn in_id_list(
        &self,
        cur_si: Option<c_int>,
        list: &IdList,
        ssp: sp_syn,
        cont_in_list: &IdList,
        flags: SynFlags,
    ) -> bool {
        let block = self.block();
        // If the group has a `containedin` list and `cur_si` is in it, it is
        // admitted whatever `list` says.
        if let Some(mut si) = cur_si
            && !cont_in_list.is_none()
            && !self.item(si).si_flags.has(SynFlags::MATCH)
        {
            // Ignore transparent items without a contains argument, double
            // checking that we don't go back past the first one.
            while self.item(si).si_flags.has(SynFlags::TRANS_CONT) && si > 0 {
                si -= 1;
            }
            // si_idx is -1 for keywords, which never contain anything.
            let si_idx = self.item(si).si_idx;
            if si_idx >= 0 {
                let spp = block.pattern(si_idx);
                if id_list_has(block, cont_in_list, spp.sp_syn, spp.sp_flags, 0) {
                    return true;
                }
            }
        }
        id_list_has(block, list, ssp, flags, 0)
    }
}

/// The list half of [`SynState::in_id_list`], over `block`'s clusters, with
/// the cluster recursion depth threaded through rather than kept in a static.
///
/// A cluster that includes itself indirectly would recurse forever, so the
/// depth is capped at 30.
fn id_list_has(
    block: SynBlockRef,
    list: &IdList,
    ssp: sp_syn,
    flags: SynFlags,
    depth: c_int,
) -> bool {
    let mut ids = match list {
        IdList::Absent => return false,
        // A transparent item that is not inside anything: only
        // not-contained groups are admitted.
        IdList::TopLevel => return !flags.has(SynFlags::CONTAINED),
        IdList::Ids(ids) => &ids[..],
    };

    // Is this top-level (i.e. not `contained`) in the file it was declared
    // in? For an included file that is not the same as SynFlags::CONTAINED, which
    // is set unconditionally there.
    let toplevel = !flags.has(SynFlags::CONTAINED) || flags.has(SynFlags::INCLUDED_TOPLEVEL);

    // A leading ALLBUT/TOP/CONTAINED inverts the answer, and requires the
    // group to be at the same `:syntax include` level as the list.
    let id = ssp.id;
    let mut retval = true;
    if let Some((&first, rest)) = ids.split_first() {
        let item = c_int::from(first);
        if (SYNID_ALLBUT..SYNID_CLUSTER).contains(&item) {
            let level = if item < SYNID_TOP {
                // ALL or ALLBUT: accept all groups in the same file.
                item - SYNID_ALLBUT
            } else if item < SYNID_CONTAINED {
                // TOP: accept all not-contained groups in the same file.
                if !toplevel {
                    return false;
                }
                item - SYNID_TOP
            } else {
                // CONTAINED: accept all contained groups in the same file.
                if toplevel {
                    return false;
                }
                item - SYNID_CONTAINED
            };
            if level != ssp.inc_tag {
                return false;
            }
            ids = rest;
            retval = false;
        }
    }

    for &item in ids {
        if item == id {
            return retval;
        }
        if c_int::from(item) >= SYNID_CLUSTER {
            let scl_list = &block.clusters()[as_index(c_int::from(item) - SYNID_CLUSTER)].scl_list;
            if !scl_list.is_none()
                && depth < 30
                && id_list_has(block, scl_list, ssp, flags, depth + 1)
            {
                return retval;
            }
        }
    }
    !retval
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Which option word a command-line tail names, and what kind it is.
    fn flag_of(rest: &str) -> Option<(&'static str, OptArg)> {
        let f = find_flag(rest.as_bytes(), false)?;
        Some((f.name.to_str().expect("ASCII"), f.arg))
    }

    #[test]
    fn a_bare_flag_word_is_recognised_at_the_end_of_the_line() {
        assert_eq!(flag_of("contained"), Some(("contained", OptArg::Flag)));
        assert_eq!(flag_of("transparent"), Some(("transparent", OptArg::Flag)));
        assert_eq!(flag_of("skipwhite"), Some(("skipwhite", OptArg::Flag)));
        assert_eq!(flag_of("conceal"), Some(("conceal", OptArg::Flag)));
        assert_eq!(flag_of("grouphere"), Some(("grouphere", OptArg::Flag)));
    }

    #[test]
    fn a_bare_flag_word_is_recognised_before_white_space_and_a_bar() {
        assert_eq!(flag_of("contained /x/"), Some(("contained", OptArg::Flag)));
        assert_eq!(flag_of("fold | echo"), Some(("fold", OptArg::Flag)));
        // `|` and `"` end the command, which is what makes them an ending
        // for a value-less option word too.
        assert_eq!(flag_of("fold\" a comment"), Some(("fold", OptArg::Flag)));
    }

    #[test]
    fn case_is_ignored_and_a_prefix_is_not_a_match() {
        assert_eq!(flag_of("CONTAINED"), Some(("contained", OptArg::Flag)));
        assert_eq!(flag_of("Skipnl"), Some(("skipnl", OptArg::Flag)));
        // `contain` is a prefix of two option words and is neither of them.
        assert_eq!(flag_of("contain"), None);
        assert_eq!(flag_of("containedinx=a"), None);
    }

    #[test]
    fn the_names_that_are_prefixes_of_each_other_stay_apart() {
        assert_eq!(flag_of("contained"), Some(("contained", OptArg::Flag)));
        assert_eq!(
            flag_of("containedin=a,b"),
            Some(("containedin", OptArg::ContainedIn))
        );
        assert_eq!(flag_of("conceal"), Some(("conceal", OptArg::Flag)));
        assert_eq!(flag_of("concealends"), Some(("concealends", OptArg::Flag)));
        assert_eq!(flag_of("cchar=x"), Some(("cchar", OptArg::Cchar)));
        assert_eq!(
            flag_of("contains=@cl"),
            Some(("contains", OptArg::Contains))
        );
    }

    #[test]
    fn an_option_taking_a_value_needs_its_equals_sign() {
        // A value-less ending is not an ending for these.
        assert_eq!(flag_of("contains"), None);
        assert_eq!(flag_of("nextgroup"), None);
        assert_eq!(flag_of("cchar"), None);
        // ... but white space is, because upstream lets `contains =a` through
        // and diagnoses the missing `=` as E405 rather than as a stray word.
        assert_eq!(flag_of("contains =a"), Some(("contains", OptArg::Contains)));
        assert_eq!(
            flag_of("nextgroup=x"),
            Some(("nextgroup", OptArg::NextGroup))
        );
        // An empty list still parses as the option; E406 comes later.
        assert_eq!(flag_of("contains="), Some(("contains", OptArg::Contains)));
    }

    #[test]
    fn an_unknown_word_is_no_option_at_all() {
        assert_eq!(flag_of("matchgroup=Foo"), None);
        assert_eq!(flag_of("start=/x/"), None);
        assert_eq!(flag_of(""), None);
        assert_eq!(flag_of("/pattern/"), None);
    }

    #[test]
    fn keyword_mode_hides_the_three_words_that_are_keywords_there() {
        for word in ["display", "fold", "extend"] {
            assert!(find_flag(word.as_bytes(), false).is_some(), "{word}");
            assert!(find_flag(word.as_bytes(), true).is_none(), "{word}");
        }
        // Everything else is still an option while reading `:syntax keyword`.
        assert!(find_flag(b"contained", true).is_some());
    }

    #[test]
    fn the_cheap_reject_lets_every_option_word_through() {
        for f in &FLAG_TAB {
            let first = f.name.to_bytes()[0];
            assert!(starts_option(first), "{:?}", f.name);
            assert!(starts_option(first.to_ascii_uppercase()), "{:?}", f.name);
        }
        assert!(!starts_option(b'/'));
        assert!(!starts_option(0));
    }

    #[test]
    fn a_group_name_is_split_from_what_follows_it() {
        let split = split_group_name(b"myGroup /pat/ contained").expect("an argument follows");
        assert_eq!(split.len, 7);
        assert_eq!(split.rest, 8);

        // Runs of blanks collapse; the name itself never contains one.
        let split = split_group_name(b"g\t \t/pat/").expect("an argument follows");
        assert_eq!(split.len, 1);
        assert_eq!(split.rest, 4);
    }

    #[test]
    fn a_group_name_with_nothing_after_it_is_no_argument() {
        assert!(split_group_name(b"myGroup").is_none());
        assert!(split_group_name(b"myGroup   ").is_none());
        assert!(split_group_name(b"").is_none());
        // `|` and `"` end an Ex command, so they never start the name...
        assert!(split_group_name(b"| echo").is_none());
        // ... but they are ordinary bytes inside the argument, because the
        // argument may be a pattern.
        let split = split_group_name(b"g /a|b/").expect("an argument follows");
        assert_eq!((split.len, split.rest), (1, 2));
    }
}
