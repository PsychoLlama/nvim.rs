//! An owned `char **`: the words, and the NULL-terminated vector of
//! pointers into them that `execvp`, `environ` and libuv's spawn take.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_char};
use core::ptr;

use crate::memory::ThinCString;

/// Owned NUL-terminated words and the NULL-terminated pointer vector over
/// them, built once.
///
/// The vector points into the words' own blocks, which do not move when
/// the `CStrVec` does, so it stays valid for as long as the value lives;
/// both are freed by `Drop`.
#[derive(Debug)]
pub(crate) struct CStrVec {
    words: Vec<ThinCString>,
    /// One pointer per word, then the null.
    slots: Vec<*mut c_char>,
}

impl CStrVec {
    /// `words`, in order.
    pub(crate) fn new(words: Vec<ThinCString>) -> Self {
        let slots = words
            .iter()
            .map(|word| word.as_ptr().cast_mut())
            .chain([ptr::null_mut()])
            .collect();
        Self { words, slots }
    }

    /// Every word, in order.
    pub(crate) fn words(&self) -> impl Iterator<Item = &CStr> {
        self.words.iter().map(ThinCString::as_cstr)
    }

    /// The first word, if there is one.
    pub(crate) fn first(&self) -> Option<&CStr> {
        self.words.first().map(ThinCString::as_cstr)
    }

    /// The NULL-terminated vector, valid while `self` lives. C spells it
    /// `char **`, but its readers (`execvp`, `environ`, `uv_spawn`) only
    /// read it.
    pub(crate) fn as_ptr(&self) -> *mut *mut c_char {
        self.slots.as_ptr().cast_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::CStrVec;
    use crate::memory::ThinCString;

    #[test]
    fn the_vector_ends_in_null_after_one_pointer_per_word() {
        let words = CStrVec::new(vec![ThinCString::from("sh"), ThinCString::from("-c")]);
        assert_eq!(words.slots.len(), 3);
        assert!(words.slots[2].is_null());
        assert_eq!(
            words.words().collect::<Vec<_>>(),
            [c"sh", c"-c"],
            "the words read back in order"
        );
        assert_eq!(words.first(), Some(c"sh"));
    }

    #[test]
    fn the_vector_survives_a_move() {
        let words = CStrVec::new(vec![ThinCString::from("x")]);
        let before = words.slots[0];
        let moved = Box::new(words);
        assert_eq!(moved.slots[0], before);
        assert_eq!(moved.slots[0].cast_const(), moved.words[0].as_ptr());
    }

    #[test]
    fn an_empty_vector_is_just_the_null() {
        let words = CStrVec::new(Vec::new());
        assert_eq!(words.slots.len(), 1);
        assert_eq!(words.first(), None);
    }
}
