//! Namespaces: the id space every extmark and decoration lives in.
//!
//! `nvim_create_namespace` interns a name in the `namespace_ids` table and
//! hands back a monotonic id; `nvim_get_namespaces` renders the table back,
//! and `ns_initialized`/`describe_ns` are the validity and lookup helpers the
//! rest of the family funnels through.  `nvim__ns_set`/`nvim__ns_get` carry
//! the per-window visibility a namespace can be given.
//!
//! The table is a [`SlotTable`](crate::registry::SlotTable) rather than a
//! `HashMap`: `nvim_get_namespaces` answers a dict in the table's own order
//! and `describe_ns` answers the *first* name an id was created under, both
//! of which the api observes (F-P21-9), and khash -- which this was -- is
//! insertion-ordered. The two `set_*` shims that remain are the ones for
//! `w_ns_set`, a khash still embedded in `Window`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::api::private::validate::err_bad_number;
use crate::api::private::validate::handle_arg;
use crate::global_cell::GlobalCell;
use crate::narrow::number_as_u32;
use crate::registry::{IdSet, SlotTable, id_set, interned_key};
use crate::types::DictBuilder;
use crate::types::{Handle, uint32_t};
use crate::winlayer::{Win, tab_windows};

/// Namespace name -> id, in creation order.
///
/// khash, which this was, is insertion-ordered with a swap-remove; nothing
/// ever removes a namespace, so the order `nvim_get_namespaces` renders and
/// `describe_ns` searches is creation order. [`SlotTable`] keeps it.
pub(crate) static namespace_ids: GlobalCell<SlotTable<Box<[u8]>, Handle>> =
    GlobalCell::new(SlotTable::new());
/// The namespaces that are window-local rather than visible everywhere.
/// Membership only; never walked.
pub(crate) static namespace_localscope: GlobalCell<IdSet<uint32_t>> = GlobalCell::new(id_set());
pub(crate) static next_namespace_id: GlobalCell<Handle> = GlobalCell::new(Handle::new(1));

/// The id `name` is registered under, if any.
pub(crate) fn namespace_id_for(name: &[u8]) -> Option<Handle> {
    namespace_ids.with(|ids| ids.get(&interned_key(name)))
}

/// Whether namespace `ns_id` is window-local rather than visible everywhere.
pub(crate) fn ns_is_local(ns_id: uint32_t) -> bool {
    namespace_localscope.with(|scoped| scoped.contains(&ns_id))
}

/// Whether `win`'s buffer holds any extmark in namespace `ns_id`.
///
fn buffer_uses_ns(win: Win, ns_id: uint32_t) -> bool {
    win.buffer().b_extmark_ns.contains_key(&ns_id)
}

/// Whether namespace `ns_id` is one of the ones `win` shows.
fn window_shows_ns(win: Win, ns_id: uint32_t) -> bool {
    win.w_ns_set.contains(&ns_id)
}

/// Show namespace `ns_id` in `win`.
fn show_ns_in_window(mut win: Win, ns_id: uint32_t) {
    win.w_ns_set.insert(ns_id);
}

/// Stop showing namespace `ns_id` in `win`.
fn hide_ns_in_window(mut win: Win, ns_id: uint32_t) {
    win.w_ns_set.remove(&ns_id);
}

pub fn nvim_create_namespace(name: ApiString) -> Integer {
    let bytes = name.as_bytes();
    if let Some(id) = namespace_id_for(bytes)
        && id > Handle::NONE
    {
        return Integer::from(id);
    }
    let id = next_namespace_id.get();
    next_namespace_id.set(id.next());
    // The nameless namespaces -- one per anonymous `nvim_create_namespace("")`
    // -- are never interned, so each call answers a fresh id.
    if !bytes.is_empty() {
        namespace_ids.with_mut(|ids| ids.insert(interned_key(bytes), id));
    }
    Integer::from(id)
}

pub fn nvim_get_namespaces() -> ApiDict {
    namespace_ids.with(|ids| {
        let mut retval = DictBuilder::with_capacity(ids.len() as size_t);
        for (name, id) in ids.entries() {
            // `ns_key` terminated the key, and it is re-measured -- so a name
            // with an interior NUL is answered truncated, as it was when the
            // key was a `ApiString`.
            let key = crate::cstr::until_nul(name);
            let value = Object::integer(Integer::from(*id));
            retval.insert(key, value);
        }
        retval.finish()
    })
}

/// The name namespace `ns_id` was created under, or `unknown` when it was
/// created nameless or not at all.
pub fn describe_ns(ns_id: NS, unknown: &[u8]) -> Vec<u8> {
    namespace_ids.with(|ids| {
        for (name, id) in ids.entries() {
            // `ns_key`'s terminator is the whole of an empty name's key, and
            // a name with an interior NUL reads as far as the first one, as
            // it did when the answer was a C string.
            if *id == ns_id && name.len() > 1 {
                return crate::cstr::until_nul(name).to_vec();
            }
        }
        unknown.to_vec()
    })
}

pub fn ns_initialized(ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false;
    }
    ns < next_namespace_id.get().get().cast_unsigned()
}

// `nvim__ns_set` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__ns_set(ns_id: Integer, opts: &mut KeyDict_ns_opts) -> Result<(), Error> {
    if !ns_initialized(number_as_u32(ns_id)) {
        return Err(err_bad_number(c"ns_id", ns_id));
    }
    let mut set_scoped: bool = true;
    if let Some(wins) = opts.wins.as_ref() {
        if wins.len() == 0 as size_t {
            set_scoped = false;
        }
        let mut windows: IdSet<*mut Window> = id_set();
        let mut i: size_t = 0 as size_t;
        while i < wins.len() {
            // A `wins` element that is neither a window handle nor a plain
            // integer takes -1, which no window carries, so the lookup below
            // refuses it -- the transpile read its bytes as an integer.
            let item = &wins[i];
            let win: Integer = item.as_handle().or_else(|| item.as_integer()).unwrap_or(-1);
            let Some(wp) = find_window_by_handle(handle_arg(c"window handle", win)?)? else {
                return Ok(());
            };
            windows.insert(wp.raw());
            i = i.wrapping_add(1);
        }
        for win in tab_windows() {
            let wp_0 = win.raw();
            if windows.contains(&wp_0) && !window_shows_ns(win, number_as_u32(ns_id)) {
                show_ns_in_window(win, number_as_u32(ns_id));
                if buffer_uses_ns(win, number_as_u32(ns_id)) {
                    changed_window_setting(win);
                }
            }
            if window_shows_ns(win, number_as_u32(ns_id)) && !windows.contains(&wp_0) {
                hide_ns_in_window(win, number_as_u32(ns_id));
                if buffer_uses_ns(win, number_as_u32(ns_id)) {
                    changed_window_setting(win);
                }
            }
        }
    }
    if set_scoped && !ns_is_local(number_as_u32(ns_id)) {
        namespace_localscope.with_mut(|scoped| scoped.insert(number_as_u32(ns_id)));
        for win in tab_windows() {
            if buffer_uses_ns(win, number_as_u32(ns_id)) {
                changed_window_setting(win);
            }
        }
    } else if !set_scoped && ns_is_local(number_as_u32(ns_id)) {
        namespace_localscope.with_mut(|scoped| scoped.remove(&(number_as_u32(ns_id))));
        for win in tab_windows() {
            if buffer_uses_ns(win, number_as_u32(ns_id)) {
                changed_window_setting(win);
            }
        }
    }
    Ok(())
}

// `nvim__ns_get` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__ns_get(ns_id: Integer) -> Result<KeyDict_ns_opts, Error> {
    let mut opts = KeyDict_ns_opts::default();
    let mut windows = Vec::new();
    opts.wins = Some(windows.into());
    if !ns_initialized(number_as_u32(ns_id)) {
        return Err(err_bad_number(c"ns_id", ns_id));
    }
    if !ns_is_local(number_as_u32(ns_id)) {
        return Ok(opts);
    }
    let mut count: size_t = 0 as size_t;
    for win in tab_windows() {
        if window_shows_ns(win, number_as_u32(ns_id)) {
            count = count.wrapping_add(1);
        }
    }
    windows = Vec::with_capacity(count);
    for win in tab_windows() {
        if window_shows_ns(win, number_as_u32(ns_id)) {
            windows.push(Object::integer(Integer::from(win.handle())));
        }
    }
    opts.wins = Some(windows.into());
    Ok(opts)
}
