#pragma once

#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>
#include <string.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/msgpack_rpc/packer_defs.h"

#ifdef USE_RUST_MSGPACK
// Rust implementations from nvim-msgpack crate
extern void rs_mpack_w(uint8_t **ptr, uint8_t byte);
extern void rs_mpack_w2(uint8_t **ptr, uint32_t val);
extern void rs_mpack_w4(uint8_t **ptr, uint32_t val);
extern void rs_mpack_uint(uint8_t **ptr, uint32_t val);
extern void rs_mpack_nil(uint8_t **ptr);
extern void rs_mpack_bool(uint8_t **ptr, int val);
extern void rs_mpack_array(uint8_t **ptr, uint32_t size);
extern void rs_mpack_map(uint8_t **ptr, uint32_t size);

#define mpack_w(b, byte) rs_mpack_w((uint8_t **)(b), (uint8_t)(byte))
#define mpack_w2(b, v) rs_mpack_w2((uint8_t **)(b), (v))
#define mpack_w4(b, v) rs_mpack_w4((uint8_t **)(b), (v))
#define mpack_uint(b, v) rs_mpack_uint((uint8_t **)(b), (v))
#define mpack_nil(b) rs_mpack_nil((uint8_t **)(b))
#define mpack_bool(b, v) rs_mpack_bool((uint8_t **)(b), (v))
#define mpack_array(b, len) rs_mpack_array((uint8_t **)(b), (len))
#define mpack_map(b, len) rs_mpack_map((uint8_t **)(b), (len))

#else

#define mpack_w(b, byte) *(*(b))++ = (char)(byte);
static inline void mpack_w2(char **b, uint32_t v)
{
  *(*b)++ = (char)((v >> 8) & 0xff);
  *(*b)++ = (char)(v & 0xff);
}

static inline void mpack_w4(char **b, uint32_t v)
{
  *(*b)++ = (char)((v >> 24) & 0xff);
  *(*b)++ = (char)((v >> 16) & 0xff);
  *(*b)++ = (char)((v >> 8) & 0xff);
  *(*b)++ = (char)(v & 0xff);
}

static inline void mpack_uint(char **buf, uint32_t val)
{
  if (val > 0xffff) {
    mpack_w(buf, 0xce);
    mpack_w4(buf, val);
  } else if (val > 0xff) {
    mpack_w(buf, 0xcd);
    mpack_w2(buf, val);
  } else if (val > 0x7f) {
    mpack_w(buf, 0xcc);
    mpack_w(buf, val);
  } else {
    mpack_w(buf, val);
  }
}

#define mpack_nil(buf) mpack_w(buf, 0xc0)
static inline void mpack_bool(char **buf, bool val)
{
  mpack_w(buf, 0xc2 | (val ? 1 : 0));
}

static inline void mpack_array(char **buf, uint32_t len)
{
  if (len < 0x10) {
    mpack_w(buf, 0x90 | len);
  } else if (len < 0x10000) {
    mpack_w(buf, 0xdc);
    mpack_w2(buf, len);
  } else {
    mpack_w(buf, 0xdd);
    mpack_w4(buf, len);
  }
}

static inline void mpack_map(char **buf, uint32_t len)
{
  if (len < 0x10) {
    mpack_w(buf, 0x80 | len);
  } else if (len < 0x10000) {
    mpack_w(buf, 0xde);
    mpack_w2(buf, len);
  } else {
    mpack_w(buf, 0xdf);
    mpack_w4(buf, len);
  }
}

#endif  // USE_RUST_MSGPACK

static inline size_t mpack_remaining(PackerBuffer *packer)
{
  return (size_t)(packer->endptr - packer->ptr);
}

#include "msgpack_rpc/packer.h.generated.h"
