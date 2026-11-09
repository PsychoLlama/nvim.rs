//! `Deref` for a wrapper that is, to its readers, the field it wraps.
//!
//! A wrapper type -- a builder over its `Vec`, a guard over the value it
//! lends, a view over the stack it narrows -- forwards every read to one
//! field, and writing the `Deref`/`DerefMut` pair by hand is eleven lines of
//! the same projection per type. [`deref_to_field!`] is those lines once.
//!
//! Only the *plain* projection belongs here: `&self.field` and
//! `&mut self.field`. A `Deref` that trims (`XString`'s terminator), calls a
//! method, or dereferences a raw pointer says something of its own and stays
//! written out -- the raw-pointer handles in `winlayer` and `event/` above
//! all, whose `unsafe` and its `SAFETY` belong in their own file.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

/// `Deref` to field `$field` of `$ty`, and with `mut` before the target
/// `DerefMut` too:
///
/// ```ignore
/// deref_to_field!(DictBuilder => mut Vec<KeyValuePair>, 0);
/// deref_to_field!(impl[E: SlotEntry] Slot<E> => HashItem<E>, 1);
/// ```
///
/// `impl[..]` carries the impl's generics, bounds included. The field may
/// be a reference (a lent `&mut T`): the projection's extra level is
/// coerced away.
macro_rules! deref_to_field {
    (impl[$($generics:tt)*] $ty:ty => mut $target:ty, $field:tt) => {
        $crate::deref::deref_to_field!(impl[$($generics)*] $ty => $target, $field);

        impl<$($generics)*> ::core::ops::DerefMut for $ty {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut $target {
                &mut self.$field
            }
        }
    };
    (impl[$($generics:tt)*] $ty:ty => $target:ty, $field:tt) => {
        impl<$($generics)*> ::core::ops::Deref for $ty {
            type Target = $target;

            #[inline(always)]
            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    };
    ($ty:ty => mut $target:ty, $field:tt) => {
        $crate::deref::deref_to_field!(impl[] $ty => mut $target, $field);
    };
    ($ty:ty => $target:ty, $field:tt) => {
        $crate::deref::deref_to_field!(impl[] $ty => $target, $field);
    };
}

pub(crate) use deref_to_field;

#[cfg(test)]
mod tests {
    #[derive(Debug)]
    struct Named {
        tag: u8,
        inner: Vec<u8>,
    }
    deref_to_field!(Named => mut Vec<u8>, inner);

    #[derive(Debug)]
    struct Lent<'a, T>(&'a mut T);
    deref_to_field!(impl['a, T] Lent<'a, T> => mut T, 0);

    #[derive(Debug)]
    struct Shown<T>(T);
    deref_to_field!(impl[T: Copy] Shown<T> => T, 0);

    #[test]
    fn reads_and_writes_reach_the_field() {
        let mut named = Named {
            tag: 7,
            inner: vec![1, 2],
        };
        named.push(3);
        assert_eq!(named.len(), 3);
        assert_eq!((named.tag, named.inner.as_slice()), (7, &[1, 2, 3][..]));

        let mut value = 5;
        {
            let mut lent = Lent(&mut value);
            *lent += 1;
            assert_eq!(*lent, 6);
        }
        assert_eq!(value, 6);

        assert_eq!(*Shown(9), 9);
    }
}
