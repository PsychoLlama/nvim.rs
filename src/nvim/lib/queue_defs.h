// Queue implemented by circularly-linked list.
//
// Adapted from libuv. Simpler and more efficient than klist.h for implementing
// queues that support arbitrary insertion/removal.
//
// Copyright (c) 2013, Ben Noordhuis <info@bnoordhuis.nl>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

#pragma once

#include <stddef.h>

typedef struct queue {
  struct queue *next;
  struct queue *prev;
} QUEUE;

#include "lib/queue_defs.h.inline.generated.h"

// Public macros.
#define QUEUE_DATA(ptr, type, field) \
  ((type *)((char *)(ptr) - offsetof(type, field)))

// Important note: the node currently being processed can be safely deleted.
// otherwise, mutating the list while QUEUE_FOREACH is iterating over its
// elements results in undefined behavior.
#define QUEUE_FOREACH(q, h, code) \
  (q) = (h)->next; \
  while ((q) != (h)) { \
    QUEUE *next = q->next; \
    code \
      (q) = next; \
  }

#ifdef USE_RUST_QUEUE
extern int rs_queue_empty(const QUEUE *q);
extern void rs_queue_init(QUEUE *q);
extern void rs_queue_add(QUEUE *h, QUEUE *n);
extern void rs_queue_insert_head(QUEUE *h, QUEUE *q);
extern void rs_queue_insert_tail(QUEUE *h, QUEUE *q);
extern void rs_queue_remove(QUEUE *q);
#endif

// ffi.cdef is unable to swallow `bool` in place of `int` here.
static inline int QUEUE_EMPTY(const QUEUE *const q)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
#ifdef USE_RUST_QUEUE
  return rs_queue_empty(q);
#else
  return q == q->next;
#endif
}

#define QUEUE_HEAD(q) (q)->next

static inline void QUEUE_INIT(QUEUE *const q)
  FUNC_ATTR_ALWAYS_INLINE
{
#ifdef USE_RUST_QUEUE
  rs_queue_init(q);
#else
  q->next = q;
  q->prev = q;
#endif
}

static inline void QUEUE_ADD(QUEUE *const h, QUEUE *const n)
  FUNC_ATTR_ALWAYS_INLINE
{
#ifdef USE_RUST_QUEUE
  rs_queue_add(h, n);
#else
  h->prev->next = n->next;
  n->next->prev = h->prev;
  h->prev = n->prev;
  h->prev->next = h;
#endif
}

static inline void QUEUE_INSERT_HEAD(QUEUE *const h, QUEUE *const q)
  FUNC_ATTR_ALWAYS_INLINE
{
#ifdef USE_RUST_QUEUE
  rs_queue_insert_head(h, q);
#else
  q->next = h->next;
  q->prev = h;
  q->next->prev = q;
  h->next = q;
#endif
}

static inline void QUEUE_INSERT_TAIL(QUEUE *const h, QUEUE *const q)
  FUNC_ATTR_ALWAYS_INLINE
{
#ifdef USE_RUST_QUEUE
  rs_queue_insert_tail(h, q);
#else
  q->next = h;
  q->prev = h->prev;
  q->prev->next = q;
  h->prev = q;
#endif
}

static inline void QUEUE_REMOVE(QUEUE *const q)
  FUNC_ATTR_ALWAYS_INLINE
{
#ifdef USE_RUST_QUEUE
  rs_queue_remove(q);
#else
  q->prev->next = q->next;
  q->next->prev = q->prev;
#endif
}
