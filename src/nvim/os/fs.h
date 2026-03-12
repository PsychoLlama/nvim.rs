#pragma once

#include <stdbool.h>  // IWYU pragma: keep
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep
#include <stdio.h>  // IWYU pragma: keep
#include <uv.h>  // IWYU pragma: keep

#include "nvim/os/fs_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Declarations for functions implemented in Rust (via #[export_name]).
// These replace the auto-generated declarations for the deleted C thin wrappers.
#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif

DLLEXPORT int os_open(const char *path, int flags, int mode);
DLLEXPORT int os_mkdir(const char *path, int32_t mode);
DLLEXPORT bool os_isrealdir(const char *name);
DLLEXPORT bool os_isdir(const char *name);
DLLEXPORT bool os_path_exists(const char *path);
DLLEXPORT bool os_file_is_readable(const char *name);
DLLEXPORT int os_dirname(char *buf, size_t len);
DLLEXPORT int os_exepath(char *buffer, size_t *size);
DLLEXPORT FILE *os_fopen(const char *path, const char *flags);
DLLEXPORT int os_set_cloexec(const int fd);
DLLEXPORT int os_close(const int fd);
DLLEXPORT int os_dup(const int fd);
DLLEXPORT ptrdiff_t os_read(const int fd, bool *ret_eof, char *ret_buf, size_t size, bool non_blocking);
DLLEXPORT ptrdiff_t os_write(const int fd, const char *buf, size_t size, bool non_blocking);
DLLEXPORT int os_copy(const char *path, const char *new_path, int flags);
DLLEXPORT int32_t os_getperm(const char *name);
DLLEXPORT int os_setperm(const char *name, int perm);
DLLEXPORT bool os_file_owned(const char *fname);
DLLEXPORT int os_chown(const char *path, uv_uid_t owner, uv_gid_t group);
DLLEXPORT int os_fchown(int fd, uv_uid_t owner, uv_gid_t group);
DLLEXPORT int os_file_settime(const char *path, double atime, double mtime);
DLLEXPORT int os_file_is_writable(const char *name);
DLLEXPORT int os_rename(const char *path, const char *new_path);
DLLEXPORT int os_rmdir(const char *path);
DLLEXPORT int os_remove(const char *path);
DLLEXPORT bool os_fileinfo(const char *path, FileInfo *file_info);
DLLEXPORT bool os_fileinfo_link(const char *path, FileInfo *file_info);
DLLEXPORT bool os_fileinfo_fd(int file_descriptor, FileInfo *file_info);
DLLEXPORT bool os_fileinfo_id_equal(const FileInfo *file_info_1, const FileInfo *file_info_2);
DLLEXPORT void os_fileinfo_id(const FileInfo *file_info, FileID *file_id);
DLLEXPORT uint64_t os_fileinfo_inode(const FileInfo *file_info);
DLLEXPORT uint64_t os_fileinfo_size(const FileInfo *file_info);
DLLEXPORT uint64_t os_fileinfo_hardlinks(const FileInfo *file_info);
DLLEXPORT uint64_t os_fileinfo_blocksize(const FileInfo *file_info);
DLLEXPORT bool os_fileid(const char *path, FileID *file_id);
DLLEXPORT bool os_fileid_equal(const FileID *file_id_1, const FileID *file_id_2);
DLLEXPORT bool os_fileid_equal_fileinfo(const FileID *file_id, const FileInfo *file_info);
DLLEXPORT char *os_realpath(const char *name, char *buf, size_t len);

#include "os/fs.h.generated.h"
