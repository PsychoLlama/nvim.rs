//! Decoration Provider Infrastructure for Neovim
//!
//! This crate provides Rust implementations for decoration provider handling
//! from `src/nvim/decoration_provider.c`. Decoration providers are plugin
//! callbacks that dynamically provide decorations during redraw.
//!
//! # Architecture
//!
//! The decoration provider system consists of:
//! - Provider state management (enabled/disabled/error tracking)
//! - Callback invocation (start, buf, win, line, range, end)
//! - Namespace highlight caching
//!
//! # FFI Pattern
//!
//! This crate uses the opaque handle pattern for DecorProvider:
//! - C owns the `decor_providers` vector and `DecorProvider` structs
//! - Rust accesses fields via accessor functions defined in C
//! - Rust exports FFI functions for operations that can be migrated

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::match_same_arms)]
#![allow(dead_code)]

pub mod accessors;
pub mod constants;
pub mod state;
pub mod types;

// Re-export main types
pub use accessors::*;
pub use constants::*;
pub use state::*;
pub use types::*;
