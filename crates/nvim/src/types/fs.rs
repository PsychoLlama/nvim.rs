#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Default)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
}
#[derive(Default)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
/// A `stat` answer.
///
/// `Copy`: a value.
#[derive(Default)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
