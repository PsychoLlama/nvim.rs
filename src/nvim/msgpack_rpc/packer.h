#pragma once

#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>
#include <string.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/msgpack_rpc/packer_defs.h"

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

static inline size_t mpack_remaining(PackerBuffer *packer)
{
  return (size_t)(packer->endptr - packer->ptr);
}

// The following functions are implemented in Rust and exported under their original names.
extern void mpack_check_buffer(PackerBuffer *packer);
extern void mpack_uint64(char **ptr, uint64_t val);
extern void mpack_integer(char **ptr, Integer val);
extern void mpack_float8(char **ptr, double val);
extern void mpack_raw(const char *data, size_t len, PackerBuffer *packer);
extern void mpack_ext(char *buf, size_t len, int8_t type, PackerBuffer *packer);
extern void mpack_handle(ObjectType type, handle_T handle, PackerBuffer *packer);

#include "msgpack_rpc/packer.h.generated.h"
