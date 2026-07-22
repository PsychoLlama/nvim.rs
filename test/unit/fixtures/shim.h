// Compile-time environment for the fixture helpers in this directory.
// Upstream compiled them into the test build of libnvim against the real C
// headers; here the nvim declaration surface is the ffigen-generated chunk
// (target/ffi/unit-cdefs.h, via -I) -- the same declarations the unit specs
// cdef -- so the fixtures always compile against the layouts of the crate
// under test. Include this before the fixture's own header.
//
// The chunk assumes only LuaJIT's built-in types, and glibc headers must
// stay out of the translation unit (the chunk carries its own struct
// _IO_FILE, and macros like BUFSIZ would mangle same-named constants). So
// this header supplies: the built-in types, bodies for the
// tools/ffigen/deny.txt tags (defined by the harness's system imports at
// spec time; this TU is freestanding, so the stable glibc x86-64 layouts
// are inlined), the libc prototypes the fixtures call, and the upstream
// macros/static-inline helpers that never had an exported symbol.
#ifndef NVIM_TEST_UNIT_FIXTURES_SHIM_H
#define NVIM_TEST_UNIT_FIXTURES_SHIM_H

// LuaJIT built-in types the chunk assumes.
typedef __SIZE_TYPE__ size_t;
typedef long ssize_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
typedef long intptr_t;
typedef unsigned long uintptr_t;
typedef __INT8_TYPE__ int8_t;
typedef __INT16_TYPE__ int16_t;
typedef __INT32_TYPE__ int32_t;
typedef __INT64_TYPE__ int64_t;
typedef __UINT8_TYPE__ uint8_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
typedef __WCHAR_TYPE__ wchar_t;
typedef __builtin_va_list va_list;
#define bool _Bool
#define true 1
#define false 0
#define NULL ((void *)0)

// Bodies of the tools/ffigen/deny.txt tags.
struct iovec {
  void *iov_base;
  size_t iov_len;
};
struct __pthread_internal_list {
  struct __pthread_internal_list *__prev;
  struct __pthread_internal_list *__next;
};
struct __pthread_mutex_s {
  int __lock;
  unsigned int __count;
  int __owner;
  unsigned int __nusers;
  int __kind;
  short __spins;
  short __elision;
  struct __pthread_internal_list __list;
};
struct __pthread_rwlock_arch_t {
  unsigned int __readers;
  unsigned int __writers;
  unsigned int __wrphase_futex;
  unsigned int __writers_futex;
  unsigned int __pad3;
  unsigned int __pad4;
  int __cur_writer;
  int __shared;
  signed char __rwelision;
  unsigned char __pad1[7];
  unsigned long __pad2;
  unsigned int __flags;
};

#include "unit-cdefs.h"

// libc calls the fixtures make; FILE is the chunk's typedef, the symbols
// resolve from the process like everything else.
FILE *fopen(const char *, const char *);
int fclose(FILE *);
int fprintf(FILE *, const char *, ...);
void *memcpy(void *, const void *, size_t);

// nvim/macros_defs.h: fixture headers mark globals EXTERN; the defining
// declarations live in the fixture .c files themselves.
#define EXTERN extern
#define INIT(...)

// nvim/event/defs.h + nvim/event/multiqueue.h.
#define event_create(cb, ...) ((Event){ .handler = cb, .argv = { __VA_ARGS__ } })
#define multiqueue_put(q, h, ...) \
  do { \
    multiqueue_put_event(q, event_create(h, __VA_ARGS__)); \
  } while (0)

// nvim/vterm/vterm.h. CSI_ARG_MISSING and the flag/type constants are chunk
// constants; only the function-like macros need restating.
#define CSI_ARG_MASK (~(1U << 31))
#define CSI_ARG_HAS_MORE(a) ((a) & CSI_ARG_FLAG_MORE)
#define CSI_ARG(a) ((a) & CSI_ARG_MASK)
#define VTERM_COLOR_IS_INDEXED(col) \
  (((col)->type & VTERM_COLOR_TYPE_MASK) == VTERM_COLOR_INDEXED)
#define VTERM_COLOR_IS_RGB(col) \
  (((col)->type & VTERM_COLOR_TYPE_MASK) == VTERM_COLOR_RGB)
#define VTERM_COLOR_IS_DEFAULT_FG(col) (!!((col)->type & VTERM_COLOR_DEFAULT_FG))
#define VTERM_COLOR_IS_DEFAULT_BG(col) (!!((col)->type & VTERM_COLOR_DEFAULT_BG))

// nvim/vterm/vterm_defs.h: the crate dropped this enum (the fixture's
// ffi-visible signature uses int, same ABI); attrs_differ still reads the
// bits by name.
typedef enum {
  VTERM_ATTR_BOLD_MASK = 1 << 0,
  VTERM_ATTR_UNDERLINE_MASK = 1 << 1,
  VTERM_ATTR_ITALIC_MASK = 1 << 2,
  VTERM_ATTR_BLINK_MASK = 1 << 3,
  VTERM_ATTR_REVERSE_MASK = 1 << 4,
  VTERM_ATTR_STRIKE_MASK = 1 << 5,
  VTERM_ATTR_FONT_MASK = 1 << 6,
  VTERM_ATTR_FOREGROUND_MASK = 1 << 7,
  VTERM_ATTR_BACKGROUND_MASK = 1 << 8,
  VTERM_ATTR_CONCEAL_MASK = 1 << 9,
  VTERM_ATTR_SMALL_MASK = 1 << 10,
  VTERM_ATTR_BASELINE_MASK = 1 << 11,
  VTERM_ATTR_URI_MASK = 1 << 12,
  VTERM_ATTR_DIM_MASK = 1 << 13,
  VTERM_ATTR_OVERLINE_MASK = 1 << 14,
  VTERM_ALL_ATTRS_MASK = (1 << 15) - 1,
} VTermAttrMask;

// nvim/grid.h (little-endian arm of the upstream macro; the port only
// targets little-endian).
#define schar_from_ascii(x) ((schar_T)(x))

// nvim/mbyte.h static inlines (their exported dependencies -- utf8len_tab,
// utf_ptr2CharInfo_impl -- are chunk prototypes).
static inline CharInfo utf_ptr2CharInfo(char const *const p_in)
{
  uint8_t const *const p = (uint8_t const *)p_in;
  uint8_t const first = *p;
  if (first < 0x80) {
    return (CharInfo){ .value = first, .len = 1 };
  }
  int len = utf8len_tab[first];
  int32_t const code_point = utf_ptr2CharInfo_impl(p, (uintptr_t)len);
  if (code_point < 0) {
    len = 1;
  }
  return (CharInfo){ .value = code_point, .len = len };
}

static inline StrCharInfo utf_ptr2StrCharInfo(char *ptr)
{
  return (StrCharInfo){ .ptr = ptr, .chr = utf_ptr2CharInfo(ptr) };
}

#endif  // NVIM_TEST_UNIT_FIXTURES_SHIM_H
