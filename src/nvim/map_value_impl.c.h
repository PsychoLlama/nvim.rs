#include "nvim/assert_defs.h"
#include "nvim/map_defs.h"

#if !defined(KEY_NAME) || !defined(VAL_NAME)
// Don't error out. it is nice to type-check the file in isolation, in clangd or otherwise
# define KEY_NAME(x) x##int
# define VAL_NAME(x) quasiquote(x, ptr_t)
#endif

#define MAP_NAME(x) VAL_NAME(KEY_NAME(x))
#define MAP_TYPE MAP_NAME(Map_)
#define KEY_TYPE KEY_NAME()
#define VALUE_TYPE VAL_NAME()

// Rust extern declarations
extern VALUE_TYPE *MAP_NAME(rs_map_ref_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE **key_alloc);
extern VALUE_TYPE *MAP_NAME(rs_map_put_ref_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE **key_alloc,
                                              bool *new_item);
extern VALUE_TYPE MAP_NAME(rs_map_del_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE *key_alloc);

VALUE_TYPE *MAP_NAME(map_ref_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE **key_alloc)
{
  return MAP_NAME(rs_map_ref_)(map, key, key_alloc);
}

VALUE_TYPE *MAP_NAME(map_put_ref_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE **key_alloc,
                                   bool *new_item)
{
  return MAP_NAME(rs_map_put_ref_)(map, key, key_alloc, new_item);
}

VALUE_TYPE MAP_NAME(map_del_)(MAP_TYPE *map, KEY_TYPE key, KEY_TYPE *key_alloc)
{
  return MAP_NAME(rs_map_del_)(map, key, key_alloc);
}

#undef MAP_NAME
#undef MAP_TYPE
#undef KEY_TYPE
#undef VALUE_TYPE
#undef INITIALIZER
