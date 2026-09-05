-- decodediff section 4: `vim.mpack`, i.e. crates/nvim/src/mpack/ (5
-- files, ~4,000 lines), which B15-11 rewrites.
--
-- The existing msgpack corpus drives `msgpackparse()`, the *Vimscript*
-- front end in eval/decode.rs.  `vim.mpack` is the vendored libmpack
-- binding, a different decoder, and `rpc.rs` is built on the same
-- framer -- so a regression here is a total functional-suite failure
-- rather than a subtle diff, which is exactly why it wants a cheap
-- differential that runs in two seconds.
--
-- Covered: every token width in the type matrix, the ext types (which
-- the Vimscript decoder spells differently and the streaming API not at
-- all), the Unpacker/Packer objects, the incremental-decode contract
-- (offset in, offset out), recursion depth, and the error arms.
--
-- Run as `nvim --headless -l <this file>`.

local function esc(s)
  return (tostring(s):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

local function show(v, depth)
  depth = depth or 0
  if depth > 40 then
    return '<deep>'
  end
  if v == vim.NIL then
    return 'NIL'
  end
  local t = type(v)
  if t == 'nil' then
    return 'nil'
  elseif t == 'string' then
    return '"' .. esc(v) .. '"'
  elseif t == 'number' then
    if v ~= v then
      return 'nan'
    elseif v == math.huge then
      return 'inf'
    elseif v == -math.huge then
      return '-inf'
    end
    -- Both spellings: %.17g catches a lost bit, tostring() catches the
    -- integer/float distinction LuaJIT makes.
    return string.format('%.17g|%s', v, tostring(v))
  elseif t == 'userdata' then
    return '<userdata>'
  elseif t ~= 'table' then
    return '<' .. t .. '>'
  end
  -- A key can be a table (msgpack allows it, and the corpus has one), and
  -- `tostring` on a table is an ADDRESS: it differs between two runs by
  -- construction and would make this whole artifact non-comparable.
  -- Every key goes through the same renderer the values do.
  local keys, byname = {}, {}
  for k in pairs(v) do
    local name = type(k) == 'table' and ('<' .. show(k, depth + 1) .. '>')
      or (type(k) == 'string' and esc(k) or show(k, depth + 1))
    keys[#keys + 1] = name
    byname[name] = k
  end
  table.sort(keys)
  local parts = {}
  for _, name in ipairs(keys) do
    parts[#parts + 1] = name .. '=' .. show(v[byname[name]], depth + 1)
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function try(fn, ...)
  local ok, res, extra = pcall(fn, ...)
  if not ok then
    return 'ERR ' .. esc((tostring(res):gsub('^.-:%d+: ', '')))
  end
  return res, extra
end

local function line(...)
  io.write(table.concat({ ... }, '\t'), '\n')
end

io.stdout:setvbuf('line')

--- Build a byte string from a list of byte values, so that every corpus
--- entry is unambiguously the bytes it claims to be.
local function B(list)
  local out = {}
  for i, b in ipairs(list) do
    out[i] = string.char(b)
  end
  return table.concat(out)
end

-- ---------------------------------------------------------------------
-- 4a -- decode: the whole token matrix, as raw bytes.
-- ---------------------------------------------------------------------

local RAW = {
  { 'nil', B({ 0xc0 }) },
  { 'false', B({ 0xc2 }) },
  { 'true', B({ 0xc3 }) },
  { 'neverused', B({ 0xc1 }) },
  { 'fixint0', B({ 0x00 }) },
  { 'fixint127', B({ 0x7f }) },
  { 'negfixint', B({ 0xff }) },
  { 'negfixint32', B({ 0xe0 }) },
  { 'uint8', B({ 0xcc, 0xff }) },
  { 'uint16', B({ 0xcd, 0xff, 0xff }) },
  { 'uint32', B({ 0xce, 0xff, 0xff, 0xff, 0xff }) },
  { 'uint64max', B({ 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff }) },
  { 'uint64hi', B({ 0xcf, 0x80, 0, 0, 0, 0, 0, 0, 0 }) },
  { 'uint64maxvar', B({ 0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff }) },
  { 'uint64_2p53', B({ 0xcf, 0, 0x20, 0, 0, 0, 0, 0, 0 }) },
  { 'int8', B({ 0xd0, 0x80 }) },
  { 'int16', B({ 0xd1, 0x80, 0x00 }) },
  { 'int32', B({ 0xd2, 0x80, 0, 0, 0 }) },
  { 'int64min', B({ 0xd3, 0x80, 0, 0, 0, 0, 0, 0, 0 }) },
  { 'int64m1', B({ 0xd3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff }) },
  { 'float32', B({ 0xca, 0x3f, 0x80, 0x00, 0x00 }) },
  { 'float32_third', B({ 0xca, 0x3e, 0xaa, 0xaa, 0xab }) },
  { 'float64', B({ 0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0 }) },
  { 'float64_01', B({ 0xcb, 0x3f, 0xb9, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9a }) },
  { 'float_nan', B({ 0xcb, 0x7f, 0xf8, 0, 0, 0, 0, 0, 0 }) },
  { 'float_inf', B({ 0xcb, 0x7f, 0xf0, 0, 0, 0, 0, 0, 0 }) },
  { 'float_ninf', B({ 0xcb, 0xff, 0xf0, 0, 0, 0, 0, 0, 0 }) },
  { 'float_denorm', B({ 0xcb, 0x00, 0x00, 0, 0, 0, 0, 0, 0x01 }) },
  { 'float_negzero', B({ 0xcb, 0x80, 0, 0, 0, 0, 0, 0, 0 }) },
  { 'fixstr_empty', B({ 0xa0 }) },
  { 'fixstr_abc', B({ 0xa3, 0x61, 0x62, 0x63 }) },
  { 'str_with_nul', B({ 0xa3, 0x61, 0x00, 0x63 }) },
  { 'str_invalid_u8', B({ 0xa2, 0xff, 0xfe }) },
  { 'str_overlong', B({ 0xa2, 0xc0, 0xaf }) },
  { 'str_surrogate', B({ 0xa3, 0xed, 0xa0, 0x80 }) },
  { 'str8', B({ 0xd9, 0x02, 0x68, 0x69 }) },
  { 'str16', B({ 0xda, 0x00, 0x02, 0x68, 0x69 }) },
  { 'str32', B({ 0xdb, 0x00, 0x00, 0x00, 0x02, 0x68, 0x69 }) },
  { 'str8_zero', B({ 0xd9, 0x00 }) },
  { 'bin8', B({ 0xc4, 0x02, 0x00, 0x01 }) },
  { 'bin16', B({ 0xc5, 0x00, 0x02, 0x00, 0x01 }) },
  { 'bin32', B({ 0xc6, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01 }) },
  { 'fixarray0', B({ 0x90 }) },
  { 'fixarray3', B({ 0x93, 0x01, 0x02, 0x03 }) },
  { 'array16', B({ 0xdc, 0x00, 0x02, 0x01, 0x02 }) },
  { 'array32', B({ 0xdd, 0x00, 0x00, 0x00, 0x02, 0x01, 0x02 }) },
  { 'fixmap0', B({ 0x80 }) },
  { 'fixmap1', B({ 0x81, 0xa1, 0x61, 0x01 }) },
  { 'map16', B({ 0xde, 0x00, 0x01, 0xa1, 0x61, 0x01 }) },
  { 'map32', B({ 0xdf, 0x00, 0x00, 0x00, 0x01, 0xa1, 0x61, 0x01 }) },
  { 'map_dupkey', B({ 0x82, 0xa1, 0x61, 0x01, 0xa1, 0x61, 0x02 }) },
  { 'map_intkey', B({ 0x81, 0x01, 0x02 }) },
  { 'map_nilkey', B({ 0x81, 0xc0, 0x02 }) },
  { 'map_arraykey', B({ 0x81, 0x90, 0x02 }) },
  { 'map_mapkey', B({ 0x81, 0x80, 0x02 }) },
  { 'map_floatkey', B({ 0x81, 0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0x02 }) },
  { 'fixext1', B({ 0xd4, 0x01, 0x41 }) },
  { 'fixext2', B({ 0xd5, 0x02, 0x41, 0x42 }) },
  { 'fixext4', B({ 0xd6, 0x03, 0x41, 0x42, 0x43, 0x44 }) },
  { 'fixext8', B({ 0xd7, 0x04, 1, 2, 3, 4, 5, 6, 7, 8 }) },
  { 'fixext16', B({ 0xd8, 0x05, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 }) },
  { 'ext8', B({ 0xc7, 0x02, 0x06, 0x41, 0x42 }) },
  { 'ext16', B({ 0xc8, 0x00, 0x02, 0x07, 0x41, 0x42 }) },
  { 'ext32', B({ 0xc9, 0x00, 0x00, 0x00, 0x02, 0x08, 0x41, 0x42 }) },
  { 'ext_negtype', B({ 0xd4, 0xff, 0x41 }) },
  { 'ext_zero', B({ 0xc7, 0x00, 0x09 }) },
  { 'nested_arr', B({ 0x91, 0x91, 0x91, 0x01 }) },
  { 'nested_map', B({ 0x81, 0xa1, 0x61, 0x81, 0xa1, 0x62, 0x01 }) },
  { 'mixed', B({ 0x93, 0xa1, 0x61, 0xc3, 0x81, 0xa1, 0x62, 0x02 }) },
  { 'trailing', B({ 0x01, 0x02 }) },
  { 'empty', '' },
  { 'trunc_uint16', B({ 0xcd, 0xff }) },
  { 'trunc_str', B({ 0xa3, 0x61 }) },
  { 'trunc_array', B({ 0x93, 0x01 }) },
  { 'trunc_map', B({ 0x81, 0xa1, 0x61 }) },
  { 'trunc_ext', B({ 0xc7, 0x04, 0x01, 0x41 }) },
  { 'huge_array', B({ 0xdd, 0x7f, 0xff, 0xff, 0xff }) },
  { 'huge_map', B({ 0xdf, 0x7f, 0xff, 0xff, 0xff }) },
  { 'huge_str', B({ 0xdb, 0x7f, 0xff, 0xff, 0xff }) },
  { 'huge_bin', B({ 0xc6, 0x7f, 0xff, 0xff, 0xff }) },
}
-- deep nesting, at and past whatever the recursion limit is
do
  local deep = {}
  for _ = 1, 200 do
    deep[#deep + 1] = 0x91
  end
  deep[#deep + 1] = 0x01
  RAW[#RAW + 1] = { 'deep200', B(deep) }
  local deeper = {}
  for _ = 1, 5000 do
    deeper[#deeper + 1] = 0x91
  end
  deeper[#deeper + 1] = 0x01
  RAW[#RAW + 1] = { 'deep5000', B(deeper) }
end

for _, c in ipairs(RAW) do
  local res, off = try(vim.mpack.decode, c[2])
  line('d4 ' .. c[1], esc(c[2]), show(res), tostring(off))
  -- the offset form: decode(s, i) is the incremental contract rpc.rs is
  -- built on, and a decoder that reports the wrong consumed length is a
  -- framing bug rather than a value bug.
  local res2, off2 = try(vim.mpack.decode, c[2], 1)
  line('d4 ' .. c[1] .. ' at1', show(res2), tostring(off2))
  local res3, off3 = try(vim.mpack.decode, 'XX' .. c[2], 3)
  line('d4 ' .. c[1] .. ' at3', show(res3), tostring(off3))
  local res4, off4 = try(vim.mpack.decode, c[2], 99)
  line('d4 ' .. c[1] .. ' at99', show(res4), tostring(off4))
end

-- streaming: the same corpus fed through an Unpacker one byte at a time,
-- which is the only shape that exercises the parser's suspend/resume.
for _, c in ipairs(RAW) do
  local up = vim.mpack.Unpacker()
  local acc, pos = {}, 1
  local guard = 0
  while pos <= #c[2] and guard < 200 do
    guard = guard + 1
    local res, npos = try(up, c[2], pos)
    acc[#acc + 1] = show(res)
    if type(npos) ~= 'number' or npos <= pos then
      acc[#acc + 1] = 'stop@' .. tostring(npos)
      break
    end
    pos = npos
  end
  line('u4 ' .. c[1], table.concat(acc, '|'))
end

-- ---------------------------------------------------------------------
-- 4b -- encode.
-- ---------------------------------------------------------------------

local VALUES = {
  { 'nil', nil },
  { 'NIL', vim.NIL },
  { 'mpackNIL', vim.mpack.NIL },
  { 'true', true },
  { 'false', false },
  { 'zero', 0 },
  { 'one', 1 },
  { 'minusone', -1 },
  { 'i127', 127 },
  { 'i128', 128 },
  { 'i255', 255 },
  { 'i256', 256 },
  { 'i65535', 65535 },
  { 'i65536', 65536 },
  { 'i2147483647', 2147483647 },
  { 'im2147483648', -2147483648 },
  -- 2^53-1 is the LAST value vim.mpack.encode() survives.  `mpack_pack_number`
  -- opens with `assert(v <= 9007199254740991. && v >= -9007199254740991.)`
  -- (conv.rs:113, byte-identical to the vendored C at v0.12.4), and that
  -- assertion is compiled in for the RELEASE build too: `vim.mpack.encode(1e16)`,
  -- `encode(2^53)`, `encode(math.huge)` and `encode(0/0)` each SIGABRT the
  -- editor -- NaN because the comparison is false for it either way.
  -- They are deliberately absent from this list: an abort takes the rest
  -- of the corpus with it, and the finding is on the docket rather than
  -- in the artifact.  DO NOT add them back without a child-process
  -- harness.  The two below are the boundary, and they must keep working.
  { 'i2p53m1', 9007199254740991 },
  { 'im2p53m1', -9007199254740991 },
  { 'f0', 0.0 },
  { 'fneg0', -0.0 },
  { 'f05', 0.5 },
  { 'f01', 0.1 },
  { 'fthird', 1 / 3 },
  { 'fpi', 3.141592653589793 },
  { 'fdenorm', 5e-324 },
  { 'ftiny', 1e-300 },
  { 'fe15', 1e15 },
  { 'fnegtiny', -1e-300 },
  { 'emptystr', '' },
  { 'ascii', 'abc' },
  { 'nulstr', 'a\0b' },
  { 'utf8', '\230\151\165\230\156\172' },
  { 'invalid', '\255\254' },
  { 'longstr', ('x'):rep(300) },
  { 'verylongstr', ('x'):rep(70000) },
  { 'emptytable', {} },
  { 'emptydict', vim.empty_dict() },
  { 'array', { 1, 2, 3 } },
  { 'array16', nil },
  -- One key, on purpose: a Lua table with several keys encodes in
  -- `pairs()` order, which is a function of the interpreter's hash seed
  -- and differs BETWEEN TWO RUNS OF THE SAME BINARY (measured -- this
  -- entry was `{a=1,b=2}` and flipped between runs).  The multi-key
  -- shapes are `unordered` and are reported canonically.
  { 'object', { a = 1 } },
  { 'object2', { a = 1, b = 2 }, unordered = true },
  { 'mixedkeys', { [1] = 'a', x = 'b' }, unordered = true },
  { 'holes', { [1] = 'a', [3] = 'c' }, unordered = true },
  { 'boolkey', { [true] = 1 } },
  { 'floatkey', { [1.5] = 'x' } },
  { 'nested', { 1, { 2, { 3, { 4 } } } } },
  { 'withnil', { 1, vim.NIL, 3 } },
  { 'deep60', nil },
}
do
  -- Back-patched by NAME, not by index: an edit to the list above moves
  -- every position and a positional patch would silently fill the wrong
  -- entry.
  local a = {}
  for i = 1, 40 do
    a[i] = i
  end
  local deep = {}
  local cur = deep
  for _ = 1, 60 do
    cur[1] = {}
    cur = cur[1]
  end
  for _, e in ipairs(VALUES) do
    if e[1] == 'array16' then
      e[2] = a
    elseif e[1] == 'deep60' then
      e[2] = deep
    end
  end
end

--- Report an encoded blob.  For a value whose *key order* is a function
--- of the interpreter's hash seed the exact bytes are not a contract
--- anybody can keep, so report the length and what it decodes back to --
--- both of which are, and both of which still catch a wrong token width,
--- a lost key or a wrong map header.
local function encline(label, enc, unordered)
  if type(enc) ~= 'string' or enc:sub(1, 4) == 'ERR ' then
    line(label, esc(tostring(enc)))
  elseif unordered then
    line(label, 'len=' .. #enc, show(try(vim.mpack.decode, enc)))
  else
    line(label, esc(enc))
  end
end

for _, v in ipairs(VALUES) do
  local enc = try(vim.mpack.encode, v[2])
  encline('e4 ' .. v[1], enc, v.unordered)
  -- the round trip, which is where a width choice on encode and a width
  -- reader on decode have to agree.
  if type(enc) == 'string' and enc:sub(1, 4) ~= 'ERR ' then
    local back, off = try(vim.mpack.decode, enc)
    line('r4 ' .. v[1], show(back), tostring(off), tostring(#enc))
  end
end

-- the arms that must fail
do
  local cycle = {}
  cycle.self = cycle
  line('e4 cycle', esc(tostring(try(vim.mpack.encode, cycle))))
  line('e4 fn', esc(tostring(try(vim.mpack.encode, print))))
  line('e4 thread', esc(tostring(try(vim.mpack.encode, coroutine.create(print)))))
  line('e4 noargs', esc(tostring(try(vim.mpack.decode))))
  line('e4 decodenum', esc(tostring(try(vim.mpack.decode, 42))))
  line('e4 decodetable', esc(tostring(try(vim.mpack.decode, {}))))
  line('e4 badoffset', esc(tostring(try(vim.mpack.decode, B({ 0x01 }), 0))))
  line('e4 negoffset', esc(tostring(try(vim.mpack.decode, B({ 0x01 }), -1))))
end

-- ---------------------------------------------------------------------
-- 4c -- the Packer / Unpacker objects and the ext handlers, which are
-- the half of lmpack.rs the `encode`/`decode` shorthands never reach.
-- ---------------------------------------------------------------------

do
  local pk = vim.mpack.Packer()
  for _, v in ipairs(VALUES) do
    encline('p4 ' .. v[1], try(pk, v[2]), v.unordered)
  end

  -- ext: an encoder that tags a table, and a decoder that reads it back.
  local seen = {}
  local ext_pk = vim.mpack.Packer({
    ext = {
      [getmetatable(setmetatable({}, { __name = 'X' })) or 'nometa'] = function()
        return 7, 'PAYLOAD'
      end,
    },
  })
  line('p4 ext-packer-built', tostring(ext_pk ~= nil))
  local ext_up = vim.mpack.Unpacker({
    ext = {
      [7] = function(code, s)
        seen[#seen + 1] = tostring(code) .. ':' .. esc(s)
        return { ext = code, payload = s }
      end,
      [-1] = function(code, s)
        return { neg = code, payload = esc(s) }
      end,
    },
  })
  for _, c in ipairs({
    { 'fixext1_7', B({ 0xd4, 0x07, 0x41 }) },
    { 'ext8_7', B({ 0xc7, 0x07, 0x07, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47 }) },
    { 'fixext1_neg', B({ 0xd4, 0xff, 0x41 }) },
    { 'fixext1_other', B({ 0xd4, 0x02, 0x41 }) },
    { 'ext_in_array', B({ 0x92, 0xd4, 0x07, 0x41, 0x01 }) },
    { 'ext_as_key', B({ 0x81, 0xd4, 0x07, 0x41, 0x01 }) },
  }) do
    local res, off = try(ext_up, c[2], 1)
    line('x4 ' .. c[1], esc(c[2]), show(res), tostring(off))
  end
  line('x4 handlers-seen', table.concat(seen, '|'))

  -- an Unpacker that is asked to work while already working, which is
  -- the one error lmpack.rs raises by name.
  local reent = vim.mpack.Unpacker({
    ext = {
      [7] = function(_, s)
        return tostring(try(vim.mpack.decode, s))
      end,
    },
  })
  line('x4 reentrant', esc(tostring(try(reent, B({ 0xd4, 0x07, 0xc3 }), 1))))
end

-- ---------------------------------------------------------------------
-- 4d -- cross-decoder agreement: the bytes vim.mpack produces, read back
-- by the Vimscript decoder, and the other way round.  The two are
-- separate implementations of the same format and this is the only
-- place they meet.
-- ---------------------------------------------------------------------

for _, c in ipairs(RAW) do
  -- Build the Blob from a `0z` literal rather than from str2list(): with
  -- 'encoding' permanently utf-8, str2list decodes bytes >= 0x80 as
  -- CODEPOINTS, so half the corpus would reach msgpackparse() as
  -- different bytes than vim.mpack saw.
  local hex = c[2]:gsub('.', function(ch)
    return string.format('%02X', ch:byte())
  end)
  local parsed = try(vim.api.nvim_eval, '0z' .. hex)
  if type(parsed) ~= 'string' or parsed:sub(1, 4) ~= 'ERR ' then
    parsed = try(vim.fn.msgpackparse, parsed)
  end
  -- show() rather than vim.inspect(): inspect prints a table ADDRESS for
  -- a table used as a key, which differs between two runs.
  line('c4 ' .. c[1], esc(c[2]), show(parsed))
end
