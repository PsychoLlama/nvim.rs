//! A C flag word as a type the compiler can check.
//!
//! The C headers spell each flag family as a run of `#define`s or an
//! anonymous enum, and c2rust re-emits those constants into *every* file that
//! included the header. So a family like `EW_*` exists eleven times over,
//! under three different integer types (`c_int`, `c_uint`, and a
//! `C2Rust_Unnamed_NN` alias), and the call sites are full of `as c_int`
//! casts reconciling the copies. Worse, nothing relates a constant to the
//! parameter it is passed as: a `WILD_*` value fits an `EW_*` parameter and
//! the compiler has no opinion about it.
//!
//! [`flag_set!`] declares one family as a single `Copy` newtype whose members
//! are associated constants. The parameter type then *is* the check, the
//! duplicate declarations collapse to one, and the casts go away.
//!
//! This is for the families a caller `|`s together. A family that is an
//! enumeration — one value at a time, with a meaningful "what is it" question
//! — wants a real `enum` and an exhaustive `match` instead; see `WildMode`.
#![forbid(unsafe_code)]

/// Declare a C flag family as a newtype over `c_int`.
///
/// ```ignore
/// crate::flag_set! {
///     /// How a shell command's input and output are wired up.
///     pub struct ShellOpts;
///
///     /// `:%!cmd` — the command filters the buffer.
///     const FILTER = 1;
///     const EXPAND = 2;
/// }
/// ```
///
/// The generated type has `NONE`, the named members, `bits`/`from_bits` for
/// the boundaries where a raw `c_int` is unavoidable, `has` (any of the
/// asked-for bits are set), `has_all` (all of them), `masked`,
/// `without`/`clear`, `is_empty`, and `|`/`|=`.
#[macro_export]
macro_rules! flag_set {
    (
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident;
        $( $(#[$cmeta:meta])* const $MEMBER:ident = $value:expr; )+
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $Name(::core::ffi::c_int);

        // Every family gets the whole vocabulary whether or not it happens to
        // need all of it: a `bits`/`from_bits` pair a family never crosses an
        // edge with, or a `without` nothing in it clears, is the macro being
        // uniform, not the member being dead.
        #[allow(dead_code)]
        impl $Name {
            /// No flag at all.
            pub const NONE: Self = Self(0);

            $( $(#[$cmeta])* pub const $MEMBER: Self = Self($value); )+

            /// The flag word as the C `int` the unrewritten callees take.
            #[inline]
            pub const fn bits(self) -> ::core::ffi::c_int {
                self.0
            }

            /// A flag word arriving from C, or from a caller that still
            /// threads one as an `int`.
            #[inline]
            pub const fn from_bits(bits: ::core::ffi::c_int) -> Self {
                Self(bits)
            }

            /// Whether *any* of `flags`' bits are set — the C `opts & FOO`
            /// test, and `opts & (FOO | BAR)` when `flags` names several.
            #[inline]
            pub const fn has(self, flags: Self) -> bool {
                self.0 & flags.0 != 0
            }

            /// Whether *every* one of `flags`' bits is set — the C
            /// `(opts & (FOO | BAR)) == (FOO | BAR)` test, which is a
            /// different question from [`has`](Self::has) and reads the
            /// same until you look twice.
            #[inline]
            pub const fn has_all(self, flags: Self) -> bool {
                self.0 & flags.0 == flags.0
            }

            /// Both sets of flags — `|`, in a `const` context, where the
            /// operator trait cannot be called.
            #[inline]
            pub const fn or(self, flags: Self) -> Self {
                Self(self.0 | flags.0)
            }

            /// `self` when `cond` holds and nothing otherwise — C's
            /// `cond ? FOO : 0`, which is how half of these are built up.
            #[inline]
            pub const fn when(self, cond: bool) -> Self {
                if cond { self } else { Self::NONE }
            }

            /// Only the bits `flags` names: C's `opts & MASK`, for a
            /// family with a sub-field that is asked *which* of several
            /// mutually exclusive values it holds.
            #[inline]
            pub const fn masked(self, flags: Self) -> Self {
                Self(self.0 & flags.0)
            }

            /// Drop `flags` in place: C's `opts &= ~FOO`. The `|=`
            /// counterpart, and the reason the field's own name need not
            /// appear twice.
            #[inline]
            pub const fn clear(&mut self, flags: Self) {
                self.0 &= !flags.0;
            }

            /// Every flag of `self` that is not in `flags`: C's `& ~FOO`.
            #[inline]
            pub const fn without(self, flags: Self) -> Self {
                Self(self.0 & !flags.0)
            }

            #[inline]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }
        }

        impl ::core::ops::BitOr for $Name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                self.or(rhs)
            }
        }

        impl ::core::ops::BitOrAssign for $Name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl ::core::fmt::Debug for $Name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "{}({:#x})", ::core::stringify!($Name), self.0)
            }
        }
    };
}
