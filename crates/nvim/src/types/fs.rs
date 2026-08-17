#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone, Default)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
