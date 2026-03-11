#pragma once

#include <stdbool.h>
#include <stdint.h>  // IWYU pragma: keep
#include <string.h>
#include <time.h>  // IWYU pragma: keep

#include "auto/config.h"
#include "nvim/macros_defs.h"
#include "nvim/memory_defs.h"  // IWYU pragma: keep

/// `malloc()` function signature
typedef void *(*MemMalloc)(size_t);

/// `free()` function signature
typedef void (*MemFree)(void *);

/// `calloc()` function signature
typedef void *(*MemCalloc)(size_t, size_t);

/// `realloc()` function signature
typedef void *(*MemRealloc)(void *, size_t);

#ifdef UNIT_TESTING
/// When unit testing: pointer to the `malloc()` function, may be altered
extern MemMalloc mem_malloc;

/// When unit testing: pointer to the `free()` function, may be altered
extern MemFree mem_free;

/// When unit testing: pointer to the `calloc()` function, may be altered
extern MemCalloc mem_calloc;

/// When unit testing: pointer to the `realloc()` function, may be altered
extern MemRealloc mem_realloc;
#endif

#ifdef EXITFREE
/// Indicates that free_all_mem function was or is running
extern bool entered_free_all_mem;
#endif

typedef void *(*MergeSortGetFunc)(void *);
typedef void (*MergeSortSetFunc)(void *, void *);
typedef int (*MergeSortCompareFunc)(const void *, const void *);

EXTERN size_t arena_alloc_count INIT( = 0);

#define kv_fixsize_arena(a, v, s) \
  ((v).capacity = (s), \
   (v).items = (void *)arena_alloc(a, sizeof((v).items[0]) * (v).capacity, true))

#include "memory.h.generated.h"

// Rust-provided replacements for deleted C wrapper functions (Phase 1)
extern char *xstrchrnul(const char *str, char c);
extern void *xmemscan(const void *addr, char c, size_t size);
extern size_t strcnt(const char *str, char c);
extern size_t memcnt(const void *data, char c, size_t len);
extern void *xmemrchr(const void *src, uint8_t c, size_t len);
extern bool strequal(const char *a, const char *b);
extern bool strnequal(const char *a, const char *b, size_t n);

// Rust-provided replacements for deleted C wrapper functions (Phase 2)
extern void strchrsub(char *str, char c, char x);
extern void memchrsub(void *data, char c, char x, size_t len);
extern char *xstpcpy(char *restrict dst, const char *restrict src);
extern char *xstpncpy(char *restrict dst, const char *restrict src, size_t maxlen);
extern size_t xstrlcpy(char *restrict dst, const char *restrict src, size_t dsize);
extern size_t xstrlcat(char *const dst, const char *const src, const size_t dsize);

// Rust-provided replacements for deleted C wrapper functions (Phase 3)
extern size_t xstrnlen(const char *s, size_t n);
extern void time_to_bytes(time_t time_, uint8_t buf[8]);
extern size_t arena_align_offset(size_t off);

#define XFREE_CLEAR(ptr) \
  do { \
    /* Take the address to avoid double evaluation. #1375 */ \
    void **ptr_ = (void **)&(ptr); \
    xfree(*ptr_); \
    /* coverity[dead-store] */ \
    *ptr_ = NULL; \
    (void)(*ptr_); \
  } while (0)

#define CLEAR_FIELD(field)  memset(&(field), 0, sizeof(field))
#define CLEAR_POINTER(ptr)  memset((ptr), 0, sizeof(*(ptr)))

#ifndef HAVE_STRNLEN
# define strnlen xstrnlen  // Older versions of SunOS may not have strnlen
#endif

#define STRCPY(d, s)        strcpy((char *)(d), (char *)(s))  // NOLINT(runtime/printf)

// Like strcpy() but allows overlapped source and destination.
#define STRMOVE(d, s)       memmove((d), (s), strlen(s) + 1)
