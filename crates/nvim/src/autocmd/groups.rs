//! Autocommand groups: the two maps, and `:augroup`.
//!
//! A group is an id and a name, held in `map_augroup_name_to_id` and
//! `map_augroup_id_to_name`; [`augroup_add`] allocates an id (reviving a
//! deleted group's), [`augroup_del`] releases the name but *not* the
//! autocommands defined under it, and [`augroup_name`] renders an id --
//! including `AUGROUP_DELETED`, whose name is the `--Deleted--`
//! placeholder.  [`do_augroup`] is `:augroup` and `:augroup!`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::Candidate;

/// The table key for `name`: its bytes and a NUL, because the listing
/// prints a key as a C string.
///
/// The empty name is a key like any other, which is what khash made of the
/// null name `nvim_del_augroup_by_id` reached `augroup_find` with, by way
/// of `augroup_name` answering it for an id no group ever had.
fn group_key(name: &[u8]) -> Box<[u8]> {
    let mut key = Vec::with_capacity(name.len() + 1);
    key.extend_from_slice(name);
    key.push(0);
    key.into_boxed_slice()
}

/// Drop a group's entries from the two tables.
///
/// No `name` leaves the name table alone, which is how `:augroup! X`
/// releases the *id* of a group whose name it has just re-pointed at
/// `AUGROUP_DELETED`.
fn augroup_map_del(id: ::core::ffi::c_int, name: Option<&[u8]>) {
    if let Some(name) = name {
        let key = group_key(name);
        map_augroup_name_to_id.with_mut(|groups| groups.remove(&key[..]));
    }
    if id > 0 {
        map_augroup_id_to_name.with_mut(|names| names.remove(&id));
    }
}

/// The name a deleted-but-still-referenced group lists under, translated
/// once and cached, so that `:language` cannot move it during a session.
pub(crate) fn get_deleted_augroup() -> &'static CStr {
    /// See [`get_deleted_augroup`].
    static DELETED: GlobalCell<Option<&'static ::core::ffi::CStr>> = GlobalCell::new(None);

    DELETED.get().unwrap_or_else(|| {
        let translated = gettext(c"--Deleted--");
        DELETED.set(Some(translated));
        translated
    })
}

/// The id of the group called `name`, creating one if there is not
/// already an id for that name.
pub fn augroup_add(name: &[u8]) -> ::core::ffi::c_int {
    debug_assert!(!name.eq_ignore_ascii_case(b"end"));

    let existing_id = augroup_find(name);
    if existing_id > 0 {
        debug_assert!(existing_id != AUGROUP_DELETED);
        return existing_id;
    }

    // The name still maps to `AUGROUP_DELETED` from an earlier
    // `:augroup! name`; that mapping has to go before a fresh id can
    // take the name.
    if existing_id == AUGROUP_DELETED {
        augroup_map_del(existing_id, Some(name));
    }

    let next_id = next_augroup_id.get();
    next_augroup_id.set(next_id + 1);
    // The two tables each own their copy of the name.
    let key = group_key(name);
    map_augroup_name_to_id.with_mut(|groups| groups.insert(key.clone(), next_id));
    map_augroup_id_to_name.with_mut(|names| names.insert(next_id, key));
    next_id
}

/// Delete the group called `name`.
///
/// `stupid_legacy_mode` is `:augroup! {name}`: a group that still holds
/// autocommands keeps them and is merely renamed `--Deleted--`, leaving
/// them defined and unreachable (O-B14-2).  Everywhere else the
/// autocommands go with the group.
///
/// No `name` is the null `char *` `nvim_del_augroup_by_id` hands over for
/// an id no group ever had: it is looked up as the empty name, and the
/// error spells it `[NULL]`, as the C `%s` did.
pub fn augroup_del(name: Option<&[u8]>, stupid_legacy_mode: bool) {
    let shown = name;
    let name = name.unwrap_or_default();
    let group = augroup_find(name);
    if group == AUGROUP_ERROR {
        let name = msg_bytes(shown.unwrap_or(b"[NULL]"));
        semsg!("E367: No such group: \"{name}\"");
        return;
    } else if group == current_augroup.get() {
        emsg(gettext(c"E936: Cannot delete the current group"));
        return;
    }

    let in_use = AutoEvent::all()
        .any(|event| with_event_cmds(event, |cmds| cmds.live().any(|(_, pat)| pat.group == group)));
    if in_use && stupid_legacy_mode {
        let warning = gettext(c"W19: Deleting augroup that is still in use");
        give_warning(warning, true, true);
        // Re-point the *name* at the deleted-group id and give up the old
        // id, leaving the autocommands on it. The name keeps its place in
        // the listing, which is what khash's `map_put_ref` over an existing
        // key did.
        let key = group_key(name);
        map_augroup_name_to_id.with_mut(|groups| groups.insert(key, AUGROUP_DELETED));
        augroup_map_del(group, None);
        return;
    }
    if in_use {
        for event in AutoEvent::all() {
            aucmd_del_where(event, |pat| pat.group == group);
        }
    }

    // Nothing is using the group, so it can go for real.
    augroup_map_del(group, Some(name));
    au_cleanup();
}

/// The id of the group called `name`, or `AUGROUP_ERROR` when there is
/// none.  `AUGROUP_DELETED` is an answer of its own: the name is known and
/// belongs to a group `:augroup!` renamed.
pub fn augroup_find(name: &[u8]) -> ::core::ffi::c_int {
    let key = group_key(name);
    let existing_id = map_augroup_name_to_id
        .with(|groups| groups.get(&key[..]))
        .unwrap_or(0);
    if existing_id == AUGROUP_DELETED || existing_id > 0 {
        existing_id
    } else {
        AUGROUP_ERROR
    }
}

/// The name of group `group`, or `None` when no group ever had that id.
///
/// `next_augroup_id` is the source of truth about which ids have existed:
/// the map shrinks when a group is deleted, so its size is not.  The id
/// one past the last is spelled `END`, which is what makes `:augroup`
/// completion terminate. A deleted group's name is the `--Deleted--`
/// placeholder; [`augroup_is_deleted`] tells it from a group really called
/// that.
pub fn augroup_name(mut group: ::core::ffi::c_int) -> Option<XString> {
    debug_assert!(group != 0);

    if group == AUGROUP_DELETED {
        return Some(XString::from_cstr(get_deleted_augroup()));
    }
    if group == AUGROUP_ALL {
        group = current_augroup.get();
    }
    if group == next_augroup_id.get() {
        return Some(XString::from_cstr(c"END"));
    }
    if group > next_augroup_id.get() {
        return None;
    }

    let name = map_augroup_id_to_name.with(|names| {
        names
            .get(&group)
            .map(|name| XString::from_bytes(&name[..name.len() - 1]))
    });
    // The id existed but is no longer in the table, so it was deleted.
    Some(name.unwrap_or_else(|| XString::from_cstr(get_deleted_augroup())))
}

/// Whether [`augroup_name`] answers the deleted-group placeholder for
/// `group`: the id is `AUGROUP_DELETED`, or one that existed and is gone.
pub(crate) fn augroup_is_deleted(group: ::core::ffi::c_int) -> bool {
    if group == AUGROUP_DELETED {
        return true;
    }
    let group = if group == AUGROUP_ALL {
        current_augroup.get()
    } else {
        group
    };
    group < next_augroup_id.get()
        && !map_augroup_id_to_name.with(|names| names.get(&group).is_some())
}

/// Whether a group called `name` exists.
pub fn augroup_exists(name: &[u8]) -> bool {
    augroup_find(name) > 0
}

/// [`augroup_exists`] for a name the caller holds.
pub(crate) fn augroup_exists_of(name: &::core::ffi::CStr) -> bool {
    augroup_exists(name.to_bytes())
}

/// `:augroup`: switch to a group, leave one, delete one, or list them.
pub fn do_augroup(arg: &CStr, del_group: bool) {
    let arg = arg.to_bytes();
    if del_group {
        if arg.is_empty() {
            emsg(gettext(e_argreq));
        } else {
            augroup_del(Some(arg), true);
        }
    } else if arg.eq_ignore_ascii_case(b"end") {
        current_augroup.set(AUGROUP_DEFAULT);
    } else if !arg.is_empty() {
        current_augroup.set(augroup_add(arg));
    } else {
        msg_start();
        msg_ext_set_kind(c"list_cmd");
        // The listing is the table's own order, which is creation order.
        // The names are copied out first: a message can run a redraw, and
        // nothing may hold the table across that.
        let entries: Vec<(Box<[u8]>, ::core::ffi::c_int)> = map_augroup_name_to_id.with(|groups| {
            groups
                .entries()
                .iter()
                .map(|(name, id)| (name.clone(), *id))
                .collect()
        });
        for (name, id) in entries {
            // A group `:augroup!` renamed lists as `--Deleted--`; its
            // key is still the old name.
            if id > 0 {
                msg_str(cstr::in_bytes(&name));
            } else if let Some(name) = augroup_name(id) {
                msg_str(name.as_cstr());
            }
            msg_str(c"  ");
        }
        msg_clr_eos();
        msg_end();
    }
}

/// Completion source for a group name: [`augroup_name`] answers `None`
/// once `idx` runs past the last id.
pub fn expand_get_augroup_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let group = ::core::ffi::c_int::try_from(idx).ok()? + 1;
    augroup_name(group).map(|name| Candidate::Owned(name.as_cstr().to_owned()))
}

/// The group a leading word of `arg` names, and how far past it (and the
/// blanks after it) the rest starts.
///
/// A name that is not a group is *not* consumed and answers
/// `AUGROUP_ALL`, which is how `:autocmd BufEnter …` is told from
/// `:autocmd MyGroup BufEnter …` without a lookahead.
pub(crate) fn arg_augroup_get(arg: &[u8]) -> (::core::ffi::c_int, usize) {
    let len = arg
        .iter()
        .position(|&c| c == 0 || ascii_iswhite(::core::ffi::c_int::from(c)) || c == b'|')
        .unwrap_or(arg.len());
    if len == 0 {
        return (AUGROUP_ALL, 0);
    }
    let group = augroup_find(&arg[..len]);
    if group == AUGROUP_ERROR {
        return (AUGROUP_ALL, 0);
    }
    (group, len + crate::charset::skip::white(&arg[len..]))
}
