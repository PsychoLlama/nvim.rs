//! Loading a parser library and registering it under a language name.
//!
//! `vim._ts_add_language_from_object` is the one way in: it `dlopen`s the
//! library, calls its `tree_sitter_<lang>` constructor, checks the ABI
//! version against [`TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION`] and
//! stores the `TSLanguage *` in [`LANGUAGES`]. `inspect_language` renders a
//! loaded language's symbols, fields and metadata back to Lua.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int, c_void};
use core::ptr::NonNull;
use std::ffi::CString;

use super::*;
use crate::event::libuv::{uv_dlclose, uv_dlerror, uv_dlopen, uv_dlsym};
use crate::lua::stack::{Kind, Raise};
use crate::types::uv_lib_t;

/// `luaL_checklstring` as the C string `luaL_checkstring` answers: up to
/// the first NUL, copied out of the slot.
pub(crate) fn checked_c_string(stack: &mut Stack, at: At) -> Result<CString, Raise> {
    let bytes = until_nul(stack.check_bytes(at)?);
    Ok(CString::new(bytes).expect("cut at the first NUL"))
}

/// The key a language is registered under: its name plus the terminator.
fn language_key(name: &CStr) -> &[u8] {
    name.to_bytes_with_nul()
}

/// A loaded language: what a parser library's constructor answered. The
/// library is never closed once a language from it is registered, so the
/// language is valid for the rest of the process, and its names with it.
#[derive(Clone, Copy)]
pub(crate) struct Language(NonNull<TSLanguage>);

impl Language {
    pub(crate) fn as_ptr(self) -> *const TSLanguage {
        self.0.as_ptr().cast_const()
    }

    fn symbol_count(self) -> u32 {
        // SAFETY: a live language, throughout.
        unsafe { ts_language_symbol_count(self.as_ptr()) }
    }

    fn symbol_type(self, symbol: TSSymbol) -> TSSymbolType {
        // SAFETY: as above.
        unsafe { ts_language_symbol_type(self.as_ptr(), symbol) }
    }

    /// The name of `symbol`, the language's own.
    fn symbol_name(self, symbol: TSSymbol) -> &'static [u8] {
        // SAFETY: as above; a name is NUL-terminated, or null.
        unsafe { c_bytes(ts_language_symbol_name(self.as_ptr(), symbol)) }
    }

    fn field_count(self) -> u32 {
        // SAFETY: as above.
        unsafe { ts_language_field_count(self.as_ptr()) }
    }

    /// The name of field `id`; `None` for an id the language has no name
    /// for.
    fn field_name(self, id: TSFieldId) -> Option<&'static [u8]> {
        // SAFETY: as above; a name is NUL-terminated, or null.
        unsafe {
            let name = ts_language_field_name_for_id(self.as_ptr(), id);
            (!name.is_null()).then(|| c_bytes(name))
        }
    }

    fn is_wasm(self) -> bool {
        // SAFETY: as above.
        unsafe { ts_language_is_wasm(self.as_ptr()) }
    }

    pub(crate) fn abi_version(self) -> u32 {
        // SAFETY: as above.
        unsafe { ts_language_abi_version(self.as_ptr()) }
    }

    fn state_count(self) -> u32 {
        // SAFETY: as above.
        unsafe { ts_language_state_count(self.as_ptr()) }
    }

    /// The grammar's version triple, when it carries one.
    fn metadata(self) -> Option<[u8; 3]> {
        // SAFETY: as above; the metadata is the language's own, or null.
        unsafe {
            let metadata = ts_language_metadata(self.as_ptr()).as_ref()?;
            Some([
                metadata.major_version,
                metadata.minor_version,
                metadata.patch_version,
            ])
        }
    }

    /// The supertype symbols.
    fn supertypes(self) -> &'static [TSSymbol] {
        let mut count: u32 = 0;
        // SAFETY: as above; the array is the language's own.
        unsafe {
            let supertypes = ts_language_supertypes(self.as_ptr(), &raw mut count);
            if supertypes.is_null() {
                return &[];
            }
            core::slice::from_raw_parts(supertypes, count as usize)
        }
    }

    /// The subtypes of `supertype` as upstream lists them: `subtypes[1..=n]`
    /// of the `n` the language has -- one past the first, and one past the
    /// end. Kept as it is (filed upstream); the entry past the end is the
    /// next one of the language's subtype map.
    fn subtypes_as_listed(self, supertype: TSSymbol) -> Vec<TSSymbol> {
        let mut count: u32 = 0;
        // SAFETY: as above; the map is the language's own, and upstream
        // reads its entry past the subtypes asked for.
        unsafe {
            let subtypes = ts_language_subtypes(self.as_ptr(), supertype, &raw mut count);
            (1..=count as usize).map(|at| *subtypes.add(at)).collect()
        }
    }
}

/// The language named by the string at `at`, or the error that it is not
/// loaded.
pub(crate) fn check_language(stack: &mut Stack, at: At) -> Result<Language, Raise> {
    let name = checked_c_string(stack, at)?;
    let found = LANGUAGES.with(|loaded| loaded.get(language_key(&name)).copied());
    match found.and_then(NonNull::new) {
        Some(language) => Ok(Language(language)),
        _ => {
            let mut message = b"no such language: ".to_vec();
            message.extend_from_slice(name.to_bytes());
            Err(Raise::message(message))
        }
    }
}

/// `vim._ts_has_language(name)`.
pub(crate) fn has_language(stack: &mut Stack) -> Result<c_int, Raise> {
    let name = checked_c_string(stack, At::arg(1))?;
    let loaded = LANGUAGES.with(|loaded| loaded.contains_key(language_key(&name)));
    stack.push_boolean(loaded);
    Ok(1)
}

/// `vim._ts_remove_language(name)`: whether it was loaded.
pub(crate) fn remove_language(stack: &mut Stack) -> Result<c_int, Raise> {
    let name = checked_c_string(stack, At::arg(1))?;
    // The name the table owns goes with the entry; the `TSLanguage` belongs
    // to the library it was loaded from.
    let present = LANGUAGES
        .with_mut(|loaded| loaded.remove(language_key(&name)))
        .is_some();
    stack.push_boolean(present);
    Ok(1)
}

/// The error text libuv has for `library`, cut where upstream's 1,025-byte
/// copy (`xstrlcpy`) cuts it.
fn dl_error(library: &uv_lib_t) -> Vec<u8> {
    // SAFETY: a handle this module zeroed and libuv filled; `uv_dlerror`
    // answers its message, or a static one when it has none, NUL-terminated
    // and alive until the handle is closed, which is after the copy.
    let text = unsafe { c_bytes(uv_dlerror(library)) };
    text[..text.len().min(1024)].to_vec()
}

/// `dlopen` the parser library at `path` and call its constructor,
/// `tree_sitter_<symbol>`.
fn load_language_from_object(path: &CStr, name: &CStr, symbol: &CStr) -> Result<Language, Raise> {
    let mut library = uv_lib_t {
        handle: core::ptr::null_mut(),
        errmsg: core::ptr::null_mut(),
    };
    // SAFETY: a NUL-terminated path, and a handle this frame owns.
    if unsafe { uv_dlopen(path.as_ptr(), &raw mut library) } != 0 {
        let error = dl_error(&library);
        // SAFETY: the handle that just failed, closed after its message is
        // copied out.
        unsafe { uv_dlclose(&raw mut library) };
        let mut message = b"Failed to load parser for language '".to_vec();
        message.extend_from_slice(name.to_bytes());
        message.extend_from_slice(b"': uv_dlopen: ");
        message.extend_from_slice(&error);
        return Err(Raise::message(message));
    }
    // `snprintf` into 128 bytes: at most 127 of them.
    let mut constructor_name = b"tree_sitter_".to_vec();
    constructor_name.extend_from_slice(symbol.to_bytes());
    constructor_name.truncate(127);
    let constructor_name = CString::new(constructor_name).expect("from C strings");
    let mut constructor: *mut c_void = core::ptr::null_mut();
    // SAFETY: the open library, a NUL-terminated name, and a slot for the
    // address.
    if unsafe {
        uv_dlsym(
            &raw mut library,
            constructor_name.as_ptr(),
            &raw mut constructor,
        )
    } != 0
    {
        let error = dl_error(&library);
        // SAFETY: as above.
        unsafe { uv_dlclose(&raw mut library) };
        let mut message = b"Failed to load parser: uv_dlsym: ".to_vec();
        message.extend_from_slice(&error);
        return Err(Raise::message(message));
    }
    // SAFETY: `tree_sitter_<lang>` is a parser library's constructor, a C
    // function of no arguments answering its language; the library stays
    // loaded for as long as the language is used.
    let language = unsafe {
        let constructor: unsafe extern "C" fn() -> *const TSLanguage =
            core::mem::transmute::<*mut c_void, _>(constructor);
        constructor()
    };
    let Some(language) = NonNull::new(language.cast_mut()) else {
        // SAFETY: the open library, which nothing of it outlives.
        unsafe { uv_dlclose(&raw mut library) };
        let mut message = b"Failed to load parser ".to_vec();
        message.extend_from_slice(path.to_bytes());
        message.extend_from_slice(b": internal error");
        return Err(Raise::message(message));
    };
    // The library stays open: the language is its.
    Ok(Language(language))
}

/// `vim._ts_add_language_from_object(path, lang[, symbol])`.
pub(crate) fn add_language(stack: &mut Stack) -> Result<c_int, Raise> {
    let path = checked_c_string(stack, At::arg(1))?;
    let name = checked_c_string(stack, At::arg(2))?;
    let symbol = if stack.top().index() >= 3 && stack.kind(At::arg(3)) != Kind::Nil {
        checked_c_string(stack, At::arg(3))?
    } else {
        name.clone()
    };
    if LANGUAGES.with(|loaded| loaded.contains_key(language_key(&name))) {
        stack.push_boolean(true);
        return Ok(1);
    }
    let language = load_language_from_object(&path, &name, &symbol)?;
    let version = language.abi_version();
    if !(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION..=TREE_SITTER_LANGUAGE_VERSION)
        .contains(&version)
    {
        let mut message = b"ABI version mismatch for ".to_vec();
        message.extend_from_slice(path.to_bytes());
        // `%d` of a `uint32_t`.
        message.extend_from_slice(
            format!(
                ": supported between {TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION} and \
                 {TREE_SITTER_LANGUAGE_VERSION}, found {}",
                version.cast_signed()
            )
            .as_bytes(),
        );
        return Err(Raise::message(message));
    }
    let key: Box<[u8]> = language_key(&name).into();
    LANGUAGES.with_mut(|loaded| loaded.insert(key, language.0.as_ptr()));
    stack.push_boolean(true);
    Ok(1)
}

/// Set `field` of the table on top to the value on top.
fn set_field_bytes(stack: &mut Stack, field: &[u8]) {
    let field = CString::new(until_nul(field)).expect("cut at the first NUL");
    stack.set_field(At::from_top(2), &field);
}

/// `vim._ts_inspect_language(lang)`: its symbols, fields, ABI version,
/// metadata, state count and supertypes.
pub(crate) fn inspect_language(stack: &mut Stack) -> Result<c_int, Raise> {
    let language = check_language(stack, At::arg(1))?;
    stack.new_table(0, 2);

    let symbol_count = language.symbol_count();
    debug_assert!(symbol_count < u32::MAX >> 1, "nsymbols < INT_MAX");
    stack.new_table(symbol_count.saturating_sub(1) as usize, 1);
    for symbol in 0..symbol_count {
        let symbol = TSSymbol::try_from(symbol).unwrap_or(TSSymbol::MAX);
        let kind = language.symbol_type(symbol);
        if kind == SYMBOL_TYPE_AUXILIARY {
            continue;
        }
        let name = language.symbol_name(symbol);
        let named = kind != SYMBOL_TYPE_ANONYMOUS;
        stack.push_boolean(named);
        if named {
            set_field_bytes(stack, name);
        } else {
            // `"\"%s\""` into 256 bytes: at most 255 of them.
            let mut quoted = Vec::with_capacity(name.len() + 2);
            quoted.push(b'"');
            quoted.extend_from_slice(name);
            quoted.push(b'"');
            quoted.truncate(255);
            set_field_bytes(stack, &quoted);
        }
    }
    stack.set_field(At::from_top(2), c"symbols");

    let field_count = language.field_count();
    stack.new_table(field_count as usize, 1);
    for field in 1..=field_count {
        match language.field_name(TSFieldId::try_from(field).unwrap_or(TSFieldId::MAX)) {
            Some(name) => stack.push_bytes(name),
            None => stack.push_nil(),
        }
        stack.raw_set_index(
            At::from_top(2),
            c_int::try_from(field).unwrap_or(c_int::MAX),
        );
    }
    stack.set_field(At::from_top(2), c"fields");

    stack.push_boolean(language.is_wasm());
    stack.set_field(At::from_top(2), c"_wasm");
    stack.push_integer(integer(language.abi_version()));
    stack.set_field(At::from_top(2), c"abi_version");

    if let Some(versions) = language.metadata() {
        stack.new_table(0, 3);
        for (value, field) in
            versions
                .into_iter()
                .zip([c"major_version", c"minor_version", c"patch_version"])
        {
            stack.push_integer(lua_Integer::from(value));
            stack.set_field(At::from_top(2), field);
        }
        stack.set_field(At::from_top(2), c"metadata");
    }

    stack.push_integer(integer(language.state_count()));
    stack.set_field(At::from_top(2), c"state_count");

    let supertypes = language.supertypes();
    stack.new_table(0, supertypes.len());
    for &supertype in supertypes {
        let subtypes = language.subtypes_as_listed(supertype);
        stack.new_table(subtypes.len(), 0);
        for (slot, &subtype) in (1..).zip(&subtypes) {
            stack.push_bytes(language.symbol_name(subtype));
            stack.raw_set_index(At::from_top(2), slot);
        }
        set_field_bytes(stack, language.symbol_name(supertype));
    }
    stack.set_field(At::from_top(2), c"supertypes");
    Ok(1)
}
