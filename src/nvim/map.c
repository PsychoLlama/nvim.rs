// map.c: Hash maps and sets
//
// parts of the implementation derived from khash.h as part of klib (MIT license)
//
// NOTE: Callers must manage memory (allocate) for keys and values.
//       Map and Set does not make its own copy of the key or value.

#include <stdbool.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"

// Rust implementations
extern void rs_mh_realloc(MapHash *h, uint32_t n_min_buckets);
extern void rs_mh_clear(MapHash *h);

void mh_realloc(MapHash *h, uint32_t n_min_buckets)
{
  rs_mh_realloc(h, n_min_buckets);
}

void mh_clear(MapHash *h)
{
  rs_mh_clear(h);
}

#define KEY_NAME(x) x##int
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, int)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, String)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##ptr_t
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##cstr_t
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, int)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##String
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, int)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##uint32_t
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, uint32_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##uint64_t
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, ssize_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, uint64_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##int64_t
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ptr_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#define VAL_NAME(x) quasiquote(x, int64_t)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

#define KEY_NAME(x) x##HlEntry
#include "nvim/map_key_impl.c.h"
#undef KEY_NAME

#define KEY_NAME(x) x##ColorKey
#include "nvim/map_key_impl.c.h"
#define VAL_NAME(x) quasiquote(x, ColorItem)
#include "nvim/map_value_impl.c.h"
#undef VAL_NAME
#undef KEY_NAME

extern void rs_pmap_del2(PMap(cstr_t) *map, const char *key);

/// Deletes a key:value pair from a string:pointer map, and frees the
/// storage of both key and value.
///
void pmap_del2(PMap(cstr_t) *map, const char *key)
{
  rs_pmap_del2(map, key);
}
