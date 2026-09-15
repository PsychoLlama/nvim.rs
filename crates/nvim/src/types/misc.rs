#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// khash/kvec instantiations named by the macro that built them, plus
// libtermkey's getstr hook.
#![allow(non_camel_case_types)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use core::mem::ManuallyDrop;

use super::*;

pub struct AutoCmdVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut AutoCmd,
}
/// The khash-derived maps and sets still in the tree: the URL set the TUI
/// writes OSC 8 ids from, the glyph cache's variable-stride index, and the
/// marktree's splice damage. Everything else the editor keeps is an
/// `IdMap`/`IdSet`/`SlotTable` now (see [`crate::registry`]).
///
/// None of them is `Copy`. Each owns its [`MapHash`]'s bucket table and the
/// `keys` (and, for a map, `values`) array beside it, all of which
/// `map_destroy`/`set_destroy` free exactly once.
pub struct Map_uint64_t_MTDamagePair {
    pub set: Set_uint64_t,
    pub values: *mut MTDamagePair,
}
pub type OptIndex = ::core::ffi::c_int;
pub struct ParserHighlight {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserHighlightChunk,
    pub init_array: [ParserHighlightChunk; 16],
}
/// The [`DictItem`] a scope dictionary is reached through, whose key is the
/// empty string and whose value is the scope dictionary itself.
///
/// It is `unref_var_dict` that gives up the reference -- a buffer, a window
/// and a tab page each embed one of these and are ordinary Rust values, so
/// without [`ManuallyDrop`](core::mem::ManuallyDrop) freeing one would
/// release the scope twice.  The item is wrapped rather than duplicated: its
/// key is the empty string, which is inline, so nothing is leaked by not
/// dropping it.
///
/// `#[repr(transparent)]`: the scopes hand their own entry around as a bare
/// `*mut DictItem`, and the collector names its value by adding
/// `offset_of!(DictItem, di_tv)` to the field's offset in a buffer, a window
/// or a tab page.  Both are casts through this wrapper.
#[repr(transparent)]
pub struct ScopeDictItem(pub ManuallyDrop<DictItem>);

impl ScopeDictItem {
    /// The item itself, which is what a hashtab slot names.
    pub fn item(&mut self) -> *mut DictItem {
        &raw mut *self.0
    }
}

impl ::core::ops::Deref for ScopeDictItem {
    type Target = DictItem;

    fn deref(&self) -> &DictItem {
        &self.0
    }
}

impl ::core::ops::DerefMut for ScopeDictItem {
    fn deref_mut(&mut self) -> &mut DictItem {
        &mut self.0
    }
}
#[derive(Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Clone)]
pub struct Set_glyph {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_char,
}
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
/// A vector of owned API strings: ShaDa's register contents, the one keyset
/// field whose value is a list of binaries rather than an [`Object`] tree.
#[derive(Clone, Default)]
pub struct StringArray(Vec<String_0>);

impl StringArray {
    /// No strings and nothing allocated.
    pub const EMPTY: Self = Self(Vec::new());

    pub fn push(&mut self, str: String_0) {
        self.0.push(str);
    }
}

impl From<Vec<String_0>> for StringArray {
    fn from(items: Vec<String_0>) -> Self {
        Self(items)
    }
}

impl ::core::ops::Deref for StringArray {
    type Target = [String_0];

    fn deref(&self) -> &[String_0] {
        &self.0
    }
}

impl<'a> IntoIterator for &'a StringArray {
    type Item = &'a String_0;
    type IntoIter = ::core::slice::Iter<'a, String_0>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
#[derive(Copy, Clone)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type TermKey_Terminfo_Getstr_Hook = unsafe extern "C" fn(
    *const ::core::ffi::c_char,
    *const ::core::ffi::c_char,
    *mut ::core::ffi::c_void,
) -> *const ::core::ffi::c_char;
pub type VTermOutputCallback =
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ();
#[derive(Copy, Clone)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
pub struct caller_scope {
    pub script_ctx: ScriptCtx,
    pub es_entry: EStack,
    pub autocmd_fname: *mut ::core::ffi::c_char,
    pub autocmd_match: *mut ::core::ffi::c_char,
    pub autocmd_fname_full: bool,
    pub autocmd_bufnr: ::core::ffi::c_int,
    pub funccalp: *mut ::core::ffi::c_void,
}
/// A dictionary item's key: its own bytes, NUL-terminated.
///
/// Short keys -- which in practice is nearly all of them, and *every* key of
/// the four kinds of item that are embedded in something bigger rather than
/// allocated by the dictionary -- live in the item.  Upstream got that by
/// over-allocating the item and letting the hash table's slot point into the
/// tail, so the item could be recovered by subtracting an offset from the
/// slot's key pointer; the slot names the item directly now
/// ([`SlotEntry`](crate::hashtab::SlotEntry)), and the key is the item's own.
///
/// The bytes are always NUL-terminated, whichever arm holds them, because
/// the probe compares NUL-terminated keys and always has.
///
/// `#[repr(u8)]` for one reason: it puts a real tag byte in the value and
/// fixes `Inline`'s discriminant at zero, so **all-zero storage is a valid
/// empty key** -- and dropping one is a no-op.  That is what lets a
/// `DictItem` sit in `xcalloc`'d memory and be filled in field by field: a
/// funccall's twelve fixed variables, a buffer's `b:changedtick`, a scope's
/// own entry.  It costs nothing: the tag lands in the tail padding either
/// way (asserted below).
#[derive(Clone)]
#[repr(u8)]
pub enum DictKey {
    /// Up to [`DictKey::INLINE_MAX`] bytes plus the NUL, in the item itself.
    ///
    /// First, and with a zero length, so that all-zero storage is a valid
    /// empty key: a `FuncCall`'s twelve fixed variables arrive `xcalloc`'d.
    Inline {
        len: uint8_t,
        bytes: [uint8_t; DictKey::INLINE_CAP],
    },
    /// A longer key, NUL included.
    Heap(Box<[uint8_t]>),
}

impl DictKey {
    /// How many bytes the inline arm holds, NUL included.  Sized so that the
    /// enum is no larger than the boxed arm forces it to be, and so that
    /// every embedded item's key fits: the longest is a funccall's
    /// twenty-character short name.
    pub const INLINE_CAP: usize = 22;
    /// The longest key that stays in the item.
    pub const INLINE_MAX: usize = Self::INLINE_CAP - 1;

    /// The empty key, which is what an uninitialised item carries.
    pub const EMPTY: Self = DictKey::Inline {
        len: 0,
        bytes: [0; Self::INLINE_CAP],
    };

    /// A key holding a copy of `bytes`, NUL-terminated.
    pub fn new(bytes: &[uint8_t]) -> Self {
        if bytes.len() <= Self::INLINE_MAX {
            let mut inline = [0; Self::INLINE_CAP];
            inline[..bytes.len()].copy_from_slice(bytes);
            DictKey::Inline {
                len: uint8_t::try_from(bytes.len()).expect("an inline key is at most 21 bytes"),
                bytes: inline,
            }
        } else {
            let mut heap = Vec::with_capacity(bytes.len() + 1);
            heap.extend_from_slice(bytes);
            heap.push(0);
            DictKey::Heap(heap.into_boxed_slice())
        }
    }

    /// The key's bytes, without the terminating NUL.
    pub fn bytes(&self) -> &[uint8_t] {
        match self {
            DictKey::Inline { len, bytes } => &bytes[..usize::from(*len)],
            DictKey::Heap(boxed) => &boxed[..boxed.len() - 1],
        }
    }

    /// How many bytes the key is, without the NUL.
    pub fn len(&self) -> usize {
        self.bytes().len()
    }

    /// Whether the key is the empty string, which is a key like any other:
    /// `{'': 1}` is a dictionary of one entry.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The key's first byte, NUL-terminated: upstream's `di_key`.
    ///
    /// This is what every hash-table probe reads, so it is a match and a
    /// field address and nothing else.  Going through
    /// [`as_c_str`](Self::as_c_str) instead cost `evalbench` two and a half
    /// per cent: `CStr::from_bytes_with_nul` *validates*, which is a scan of
    /// the key on every lookup.
    pub fn as_ptr(&self) -> *const ::core::ffi::c_char {
        match self {
            DictKey::Inline { bytes, .. } => bytes.as_ptr().cast(),
            DictKey::Heap(boxed) => boxed.as_ptr().cast(),
        }
    }

    /// The key as a C string, for a caller that wants one rather than a
    /// pointer.  Validating, and therefore linear: not for a probe.
    pub fn as_c_str(&self) -> &::core::ffi::CStr {
        let with_nul = match self {
            DictKey::Inline { len, bytes } => &bytes[..usize::from(*len) + 1],
            DictKey::Heap(boxed) => boxed,
        };
        ::core::ffi::CStr::from_bytes_with_nul(with_nul).expect("a key is NUL-terminated once")
    }
    /// A key of `len` bytes, every one of them NUL, and a pointer the
    /// caller may write those `len` bytes through.
    ///
    /// For the decoders that learn a key's length from a header and then
    /// receive its bytes in chunks: the storage is the key's own from the
    /// start, so a message that ends early leaves a short key rather than a
    /// dangling one.
    pub fn zeroed(len: usize) -> Self {
        if len <= Self::INLINE_MAX {
            DictKey::Inline {
                len: uint8_t::try_from(len).expect("an inline key is at most 21 bytes"),
                bytes: [0; Self::INLINE_CAP],
            }
        } else {
            DictKey::Heap(vec![0; len + 1].into_boxed_slice())
        }
    }

    /// The key's bytes, writable, without the terminating NUL. Pairs with
    /// [`zeroed`](Self::zeroed); the NUL past them is not writable through
    /// this.
    pub fn bytes_mut(&mut self) -> &mut [uint8_t] {
        match self {
            DictKey::Inline { len, bytes } => &mut bytes[..usize::from(*len)],
            DictKey::Heap(boxed) => {
                let end = boxed.len() - 1;
                &mut boxed[..end]
            }
        }
    }
}

impl PartialEq for DictKey {
    /// Byte equality, whichever arm each side is in.
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for DictKey {}

impl ::core::fmt::Debug for DictKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.bytes().escape_ascii().to_string(), f)
    }
}

impl From<&::core::ffi::CStr> for DictKey {
    fn from(key: &::core::ffi::CStr) -> Self {
        Self::new(key.to_bytes())
    }
}

impl From<&[uint8_t]> for DictKey {
    fn from(key: &[uint8_t]) -> Self {
        Self::new(key)
    }
}

impl From<&str> for DictKey {
    fn from(key: &str) -> Self {
        Self::new(key.as_bytes())
    }
}

#[cfg(not(randomized_layout))]
const _: () = {
    assert!(::core::mem::size_of::<DictKey>() == 24);
    assert!(::core::mem::size_of::<DictItem>() == 48);
};

impl DictItem {
    /// The item's key, without the terminating NUL.
    ///
    /// The cheap spelling, and the one that shares its name with
    /// `DictItemRef::key`: both answer bytes in constant time, so a caller
    /// reaching for "the key" gets the same thing on either type.
    pub fn key(&self) -> &[uint8_t] {
        self.di_key.bytes()
    }

    /// The item's key as a C string, for a caller that wants a *string*.
    ///
    /// **Not for a hot path.** Building one validates the key (see
    /// [`DictKey::as_c_str`]); a caller that wants a pointer to pass on
    /// wants [`DictKey::as_ptr`] on [`di_key`](Self::di_key) instead.
    pub fn key_cstr(&self) -> &::core::ffi::CStr {
        self.di_key.as_c_str()
    }
}

/// One entry of a [`Dict`](crate::types::Dict), owning its key.
///
/// `di_lock` is the *slot's* lock: `:lockvar` on a variable locks the item it
/// lives in, not the value it currently holds.
///
/// This is the only item shape there is.  The four kinds that are embedded
/// in something bigger -- a funccall's fixed variables, a scope dictionary's
/// own entry ([`ScopeDictItem`]), `b:changedtick` and a `v:` row -- were
/// separate structs only because each spelled the flexible key member out at
/// a different length; an owned key makes them all this.
pub struct DictItem {
    pub di_tv: TypVal,
    pub di_lock: VarLock,
    pub di_flags: uint8_t,
    pub di_key: DictKey,
}
pub struct ModEntry {
    pub flag: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct nvim_stats_s {
    pub fsync: int64_t,
    pub redraw: int64_t,
    pub log_skip: int16_t,
}
#[derive(Copy, Clone)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
