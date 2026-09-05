-- decodediff section 3: `vim.json`, i.e. crates/nvim/src/cjson/ (3 files,
-- 2,947 lines), which B15-12 rewrites.
--
-- The existing json corpus drives `json_decode()`, the *Vimscript* front
-- end in eval/decode.rs.  `vim.json` is a different decoder entirely --
-- Lua CJSON, vendored -- and nothing in the tree diffs it.  In
-- particular `fpconv.rs` is a hand-rolled grisu: the exact text
-- `vim.json.encode(0.1)` produces is a byte-for-byte contract that no
-- test asserts, and half of this file exists to pin it.
--
-- Run as `nvim --headless -l <this file>`; every line printed is part of
-- the comparison.  Nothing here may print an address, a pid or a time.

local function esc(s)
  return (tostring(s):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- Render a decoded value canonically: sorted keys, explicit types, and
--- vim.NIL distinguished from nil, because the whole point of the
--- `luanil` options is which of the two you get.
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
    -- %.17g rather than tostring(): tostring rounds, and a decoder that
    -- loses a bit would still print the same thing.
    if v ~= v then
      return 'nan'
    elseif v == math.huge then
      return 'inf'
    elseif v == -math.huge then
      return '-inf'
    end
    return string.format('%.17g|%s', v, tostring(v))
  elseif t ~= 'table' then
    return '<' .. t .. '>'
  end
  local mt = getmetatable(v)
  local tagged = mt and (mt.__is_dict or mt == vim._empty_dict_mt) and 'D' or ''
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
  return tagged .. '{' .. table.concat(parts, ',') .. '}'
end

local function try(fn, ...)
  local ok, res = pcall(fn, ...)
  if not ok then
    -- A Lua error carries this file's path and line, which differ
    -- between two checkouts; keep only the message.
    return 'ERR ' .. esc((tostring(res):gsub('^.-:%d+: ', '')))
  end
  return res
end

local function line(...)
  io.write(table.concat({ ... }, '\t'), '\n')
end

io.stdout:setvbuf('line')

-- ---------------------------------------------------------------------
-- 3a -- decode: the same shapes the Vimscript corpus uses, through the
-- other decoder, plus the ones only this one has (comments, luanil).
-- ---------------------------------------------------------------------

local DOCS = {
  '',
  ' ',
  '\t\n\r ',
  'null',
  'true',
  'false',
  'nul',
  'tru',
  'nullx',
  'n',
  '"',
  '""',
  '"a"',
  '"\\""',
  '"\\/"',
  '"\\b\\f\\n\\r\\t"',
  '"\\q"',
  '"\\u"',
  '"\\u0041"',
  '"\\u00e9"',
  '"\\u4e00"',
  '"\\ud83d\\ude00"',
  '"\\ud83d"',
  '"\\ude00"',
  '"\\ud83dx"',
  '"\\ud800"',
  '"\\udfff"',
  '"\\u0000"',
  '"a\\u0000b"',
  '"\xf0\x9f\x98\x80"',
  '"\xff"',
  '"\xc3"',
  '"\xc3\x83"',
  '"\xed\xa0\x80"',
  '"\xf4\x90\x80\x80"',
  '"\xc0\xaf"',
  '"abc',
  '"ab\\',
  '"\x01"',
  '"\x7f"',
  '0',
  '-0',
  '1',
  '-1',
  '00',
  '01',
  '-01',
  '1.',
  '1.0',
  '-1.5',
  '1e',
  '1e5',
  '1E5',
  '1e+5',
  '1e-5',
  '1.0e',
  '.5',
  '-',
  '1.2.3',
  '9007199254740992',
  '9007199254740993',
  '9223372036854775807',
  '9223372036854775808',
  '-9223372036854775808',
  '18446744073709551616',
  '1e308',
  '1e309',
  '1e400',
  '-1e400',
  '1e-400',
  '5e-324',
  '2.2250738585072014e-308',
  '1.7976931348623157e308',
  '0.1',
  '0.2',
  '0.3',
  '1/3',
  '0e0',
  '-0.0',
  '3.141592653589793',
  '[]',
  '[1]',
  '[1,2]',
  '[1,]',
  '[,1]',
  '[1 2]',
  '[',
  ']',
  '[[]]',
  '[[1],[2]]',
  '[}',
  '{]',
  '{}',
  '{"a":1}',
  '{"a":1,"b":2}',
  '{"a":1,}',
  '{"a"}',
  '{"a":}',
  '{:1}',
  '{"a":1,"a":2}',
  '{"":1}',
  '{1:2}',
  '{null:1}',
  '{"a":{"b":{"c":1}}}',
  '{"a":[1,{"b":2}]}',
  '{"a":1}x',
  '  {"a":1}  ',
  '[1,2] [3]',
  '1 2',
  '{"a":null}',
  '[null]',
  '[null,1,null]',
  '{"a":null,"b":1}',
  'NaN',
  'Infinity',
  '-Infinity',
  'nan',
  'inf',
  '// a comment\n1',
  '/* a comment */1',
  '1 // trailing',
  '1 /* trailing */',
  '/*',
  '//',
  '{"a":1 /* c */, "b":2}',
  '\xef\xbb\xbf{"a":1}',
  '[[[[[[[[[[1]]]]]]]]]]',
  ('['):rep(200) .. '1' .. (']'):rep(200),
  ('['):rep(2000) .. '1' .. (']'):rep(2000),
  ('{"a":'):rep(200) .. '1' .. ('}'):rep(200),
}

local LUANIL = {
  { 'default', nil },
  { 'obj', { luanil = { object = true } } },
  { 'arr', { luanil = { array = true } } },
  { 'both', { luanil = { object = true, array = true } } },
  { 'objfalse', { luanil = { object = false, array = false } } },
  { 'comments', { skip_comments = true } },
  { 'nocomments', { skip_comments = false } },
}

for _, doc in ipairs(DOCS) do
  for _, opt in ipairs(LUANIL) do
    local res
    if opt[2] == nil then
      res = try(vim.json.decode, doc)
    else
      res = try(vim.json.decode, doc, opt[2])
    end
    if type(res) == 'string' and res:sub(1, 4) == 'ERR ' then
      line('d3 ' .. opt[1], esc(doc), res)
    else
      line('d3 ' .. opt[1], esc(doc), show(res))
    end
  end
end

-- ---------------------------------------------------------------------
-- 3b -- encode, including the float contract.
--
-- fpconv.rs is a hand-written grisu (`fpconv_dtoa`), so every one of
-- these strings is a promise the rewrite has to keep exactly.  The
-- shortest-round-trip property is what makes 0.1 print as "0.1" rather
-- than "0.10000000000000001", and a rewrite that reaches for
-- `format!("{}")` will differ on a handful of these and on nothing else.
-- ---------------------------------------------------------------------

local FLOATS = {
  0.0,
  -0.0,
  1.0,
  -1.0,
  0.5,
  0.1,
  0.2,
  0.3,
  1 / 3,
  2 / 3,
  1e-1,
  1e-5,
  1e-10,
  1e-100,
  1e-300,
  5e-324,
  2.2250738585072014e-308,
  1e10,
  1e15,
  1e16,
  1e17,
  1e20,
  1e100,
  1e300,
  1.7976931348623157e308,
  3.141592653589793,
  2.718281828459045,
  123456789.123456789,
  1234567890123456.0,
  12345678901234567.0,
  9007199254740992.0,
  9007199254740993.0,
  0.30000000000000004,
  4.35,
  1.005,
  100.0,
  1000000.0,
  10000000.0,
  -1e-7,
  math.huge,
  -math.huge,
  0 / 0,
}
for i, f in ipairs(FLOATS) do
  line('e3 float ' .. i, esc(string.format('%.17g', f)), esc(tostring(try(vim.json.encode, f))))
end

local INTS = {
  0,
  1,
  -1,
  255,
  65536,
  2147483647,
  -2147483648,
  2 ^ 31,
  2 ^ 52,
  2 ^ 53,
  2 ^ 53 + 1,
  2 ^ 62,
  2 ^ 63,
  -(2 ^ 63),
  2 ^ 64,
}
for i, n in ipairs(INTS) do
  line('e3 int ' .. i, esc(string.format('%.17g', n)), esc(tostring(try(vim.json.encode, n))))
end

local VALUES = {
  { 'nil', nil },
  { 'NIL', vim.NIL },
  { 'true', true },
  { 'false', false },
  { 'emptystr', '' },
  { 'ascii', 'abc' },
  { 'quote', 'a"b' },
  { 'backslash', 'a\\b' },
  { 'slash', 'a/b' },
  { 'controls', '\1\2\27\127' },
  { 'nul', 'a\0b' },
  { 'newline', 'a\nb\tc\rd' },
  { 'utf8', '\230\151\165\230\156\172' },
  { 'emoji', '\240\159\152\128' },
  { 'invalid', '\255\254' },
  { 'overlong', '\192\175' },
  { 'surrogate', '\237\160\128' },
  { 'trunc', '\230\151' },
  { 'emptytable', {} },
  { 'emptydict', vim.empty_dict() },
  { 'array', { 1, 2, 3 } },
  { 'nested', { 1, { 2, { 3, { 4 } } } } },
  -- One key, on purpose: a Lua table with several string keys encodes in
  -- `pairs()` order, which is a function of the interpreter's hash seed
  -- and differs BETWEEN TWO RUNS OF THE SAME BINARY.  The multi-key
  -- shapes are `unordered` below and are reported canonically.
  { 'object', { a = 1 } },
  { 'object3', { b = 2, a = 1, c = 3 }, unordered = true },
  { 'mixedkeys', { [1] = 'a', x = 'b' }, unordered = true },
  { 'holes', { [1] = 'a', [3] = 'c' }, unordered = true },
  { 'boolkey', { [true] = 1 } },
  { 'numkeys', { [1.5] = 'x' } },
  { 'nestednil', { 1, vim.NIL, 3 } },
  { 'deep', nil }, -- filled in below
  { 'wide', nil },
}
do
  -- Back-patched by NAME, not by index: an edit to the list above moves
  -- every position and a positional patch would silently fill the wrong
  -- entry.
  local deep = {}
  local cur = deep
  for _ = 1, 60 do
    cur[1] = {}
    cur = cur[1]
  end
  local wide = {}
  for i = 1, 200 do
    wide[i] = i
  end
  for _, e in ipairs(VALUES) do
    if e[1] == 'deep' then
      e[2] = deep
    elseif e[1] == 'wide' then
      e[2] = wide
    end
  end
end

local ENCOPTS = {
  { 'default', nil },
  { 'escslash', { escape_slash = true } },
  { 'noescslash', { escape_slash = false } },
  { 'sortkeys', { sort_keys = true } },
  { 'indent2', { indent = 2 } },
  { 'indent0', { indent = 0 } },
  { 'indent8sort', { indent = 8, sort_keys = true } },
}

--- Report an encoded document.  For a value whose *key order* is a
--- function of the interpreter's hash seed, the exact text is not a
--- contract anybody can keep, so report the length and the value it
--- decodes back to -- both of which are, and both of which still catch a
--- wrong escape, a wrong separator or a lost key.  `sort_keys` makes the
--- order a contract again and those rows keep the raw text.
local function encline(label, res, unordered, sorted)
  if type(res) ~= 'string' or res:sub(1, 4) == 'ERR ' then
    line(label, esc(tostring(res)))
  elseif unordered and not sorted then
    line(label, 'len=' .. #res, show(try(vim.json.decode, res)))
  else
    line(label, esc(res))
  end
end

for _, v in ipairs(VALUES) do
  for _, o in ipairs(ENCOPTS) do
    local res
    if o[2] == nil then
      res = try(vim.json.encode, v[2])
    else
      res = try(vim.json.encode, v[2], o[2])
    end
    encline('e3 ' .. v[1] .. ' ' .. o[1], res, v.unordered, o[2] and o[2].sort_keys)
  end
end

-- the arms that must *fail*: a cycle, and a value with no JSON spelling.
do
  local cycle = {}
  cycle.self = cycle
  line('e3 cycle', esc(tostring(try(vim.json.encode, cycle))))
  local pair = { a = {} }
  pair.a.b = pair
  line('e3 cycle2', esc(tostring(try(vim.json.encode, pair))))
  local shared = {}
  line('e3 shared', esc(tostring(try(vim.json.encode, { shared, shared, shared }))))
  line('e3 fn', esc(tostring(try(vim.json.encode, print))))
  line('e3 thread', esc(tostring(try(vim.json.encode, coroutine.create(print)))))
  line('e3 badopt', esc(tostring(try(vim.json.encode, 1, 'notatable'))))
  line('e3 badindent', esc(tostring(try(vim.json.encode, { 1 }, { indent = -1 }))))
  line('e3 hugeindent', esc(tostring(try(vim.json.encode, { 1 }, { indent = 1000 }))))
  line('e3 noargs', esc(tostring(try(vim.json.decode))))
  line('e3 decodenum', esc(tostring(try(vim.json.decode, 42))))
  line('e3 decodetable', esc(tostring(try(vim.json.decode, {}))))
end

-- ---------------------------------------------------------------------
-- 3c -- the round trip.  decode(encode(v)) has to be v, and
-- encode(decode(s)) has to be stable on the second pass; a float that
-- loses a bit shows here and nowhere else.
-- ---------------------------------------------------------------------

for i, f in ipairs(FLOATS) do
  local enc = try(vim.json.encode, f)
  if type(enc) == 'string' and enc:sub(1, 4) ~= 'ERR ' then
    local back = try(vim.json.decode, enc)
    line('r3 float ' .. i, esc(enc), show(back), tostring(back == f))
  else
    line('r3 float ' .. i, esc(tostring(enc)))
  end
end

-- The document round trip.  Reported as the decoded VALUE plus the
-- encoded LENGTH rather than as the encoded text: an object with more
-- than one key encodes in hash order and the text is not reproducible,
-- while show() sorts and the length does not care.  Both still catch a
-- lost key, a wrong escape and a changed separator.
for _, doc in ipairs(DOCS) do
  local dec = try(vim.json.decode, doc)
  if type(dec) ~= 'string' or dec:sub(1, 4) ~= 'ERR ' then
    local enc = try(vim.json.encode, dec)
    local back = 'n/a'
    local len = 'n/a'
    if type(enc) == 'string' and enc:sub(1, 4) ~= 'ERR ' then
      len = 'len=' .. #enc
      back = show(try(vim.json.decode, enc))
    else
      back = esc(tostring(enc))
    end
    line('r3 doc', esc(doc), show(dec), len, back)
  end
end

-- ---------------------------------------------------------------------
-- 3d -- the arms 3a-3c leave dark.  Grown at B15-12 before the rewrite,
-- so every row here is a promise the *unchanged* tree already keeps.
--
-- What 3a-3c never reach: lua_array_length's sparse-array verdict (the
-- ratio/safe policy is the only thing between an array and an object and
-- it has three outcomes), the __len metamethod arm of json_append_data,
-- the encode-side depth limit (3a only crosses the *decode* one), the
-- keybuf's reuse across a NESTED sorted object (init_keybuf_size is
-- restored per level and only nesting proves it), and the UTF-16 sniff
-- at the head of json_decode.
-- ---------------------------------------------------------------------

local function mkdeep(n)
  local root = {}
  local cur = root
  for _ = 1, n do
    cur[1] = {}
    cur = cur[1]
  end
  return root
end

local EXTRA = {
  -- lua_array_length: max vs items * encode_sparse_ratio (2), and
  -- encode_sparse_safe (10) below which a gap is always still an array.
  { 'sparse_safe', { [1] = 1, [10] = 2 } },
  { 'sparse_edge', { [1] = 1, [11] = 2 } },
  { 'sparse_far', { [1] = 1, [100] = 2 } },
  { 'sparse_dense', { [1] = 1, [2] = 2, [3] = 3, [50] = 4 } },
  { 'zerokey', { [0] = 'z', [1] = 'a' }, unordered = true },
  { 'negkey', { [-1] = 'n', [1] = 'a' }, unordered = true },
  { 'fltkey_int', { [2.0] = 'b', [1.0] = 'a' } },
  -- __len: json_append_data calls it and then encodes 1..len RAW, so a
  -- __len that lies is the only way to encode past the real array.
  { 'len_mt', setmetatable({ 1, 2, 3 }, { __len = function() return 2 end }) },
  { 'len_mt_over', setmetatable({ 1, 2 }, { __len = function() return 4 end }) },
  { 'len_mt_zero', setmetatable({ 1, 2 }, { __len = function() return 0 end }) },
  { 'len_mt_str', setmetatable({ 1, 2 }, { __len = function() return '2' end }) },
  { 'len_mt_bad', setmetatable({ 1, 2 }, { __len = function() error('boom') end }) },
  { 'index_mt', setmetatable({}, { __index = function() return 7 end }) },
  -- the encode depth limit is 1000; 3a only ever crosses the decode one.
  { 'deep999', mkdeep(998) },
  { 'deep1200', mkdeep(1200) },
  -- keybuf reuse: the sorted path restores size/length per level, so
  -- only a nested multi-key object proves the restore.
  { 'sortnest', { b = { d = 4, c = 3 }, a = { f = 6, e = 5 } }, unordered = true },
  { 'sortnest_arr', { { z = 1, y = 2 }, { x = 3, w = 4 } }, unordered = true },
  { 'sortmixed', { [2] = 'two', b = 'bee', [1] = 'one', a = 'ay' }, unordered = true },
  { 'sortlong', nil },
  { 'bigstr', ('\1\2"\\/'):rep(500) },
  { 'bigarr', nil },
}
do
  local sortlong = {}
  for i = 1, 100 do
    sortlong[string.format('k%03d', 101 - i)] = i
  end
  local bigarr = {}
  for i = 1, 5000 do
    bigarr[i] = i * 0.5
  end
  for _, e in ipairs(EXTRA) do
    if e[1] == 'sortlong' then
      e[2], e.unordered = sortlong, true
    elseif e[1] == 'bigarr' then
      e[2] = bigarr
    end
  end
end

for _, v in ipairs(EXTRA) do
  for _, o in ipairs(ENCOPTS) do
    local res
    if o[2] == nil then
      res = try(vim.json.encode, v[2])
    else
      res = try(vim.json.encode, v[2], o[2])
    end
    encline('e3d ' .. v[1] .. ' ' .. o[1], res, v.unordered, o[2] and o[2].sort_keys)
  end
end

-- Documents 3a cannot hold: an embedded NUL (the UTF-16/UTF-32 sniff is
-- the first thing json_decode does), and the decode-side tmp strbuf,
-- which is sized from the whole document and only a long string grows.
local BINDOCS = {
  '\0',
  '\0\0',
  '\0{"a":1}',
  '{\0"a":1}',
  '\0"a"',
  '"\0"',
  '[\0]',
  '"' .. ('a'):rep(5000) .. '"',
  '"' .. ('\\n'):rep(2000) .. '"',
  '"' .. ('\\u00e9'):rep(1000) .. '"',
  '[' .. ('1,'):rep(5000) .. '1]',
  '{' .. ('"k":1,'):rep(2000) .. '"k":1}',
  ('  '):rep(5000) .. '1',
  '1' .. ('  '):rep(5000),
  '/*' .. ('x'):rep(5000) .. '*/1',
}
for i, doc in ipairs(BINDOCS) do
  for _, opt in ipairs(LUANIL) do
    local res
    if opt[2] == nil then
      res = try(vim.json.decode, doc)
    else
      res = try(vim.json.decode, doc, opt[2])
    end
    local shown = (type(res) == 'string' and res:sub(1, 4) == 'ERR ') and res or show(res)
    -- The document itself can be 5 kB; report its length and head only.
    line('d3d ' .. i .. ' ' .. opt[1], '#' .. #doc, esc(doc:sub(1, 24)), shown)
  end
end

-- encode/decode argument arity and option shapes, which only reach
-- luaL_error arms.
line('e3d optnil', esc(tostring(try(vim.json.encode, { 1 }, nil))))
line('e3d opt3', esc(tostring(try(vim.json.encode, { 1 }, {}, {}))))
line('e3d dec3', esc(tostring(try(vim.json.decode, '1', {}, {}))))
line('e3d escslash_str', esc(tostring(try(vim.json.encode, 'a/b', { escape_slash = 'yes' }))))
line('e3d sortkeys_str', esc(tostring(try(vim.json.encode, { 1 }, { sort_keys = 'yes' }))))
line('e3d indent_tbl', esc(tostring(try(vim.json.encode, { 1 }, { indent = {} }))))
line('e3d indent_str', esc(tostring(try(vim.json.encode, { 1, 2 }, { indent = '--' }))))
line('e3d indent_empty', esc(tostring(try(vim.json.encode, { 1, 2 }, { indent = '' }))))
line('e3d luanil_str', esc(tostring(try(vim.json.decode, '[null]', { luanil = 'yes' }))))
line('e3d luanil_num', esc(tostring(try(vim.json.decode, '[null]', { luanil = 1 }))))
line('e3d skipc_num', esc(tostring(try(vim.json.decode, '//x\n1', { skip_comments = 1 }))))
line('e3d newmod', esc(tostring(type(try(vim.json.new)))))
line('e3d newmod_enc', esc(tostring(try(function()
  return vim.json.new().encode({ 1, 2 })
end))))

-- Numbers only the *decoder's* strtod sees.  `decode_invalid_numbers` is on
-- by default, so json_is_invalid_number is never consulted and the whole C
-- strtod grammar -- hex floats, `inf`, `nan(...)` -- reaches the number
-- token.  3a covers inf and nan; nothing covered hex.
local NUMDOCS = {
  '0x',
  '0x0',
  '0x10',
  '0X10',
  '-0x10',
  '0xff',
  '0x1p4',
  '0x1P4',
  '0x1p+4',
  '0x1p-4',
  '0x1.8p1',
  '0x.8',
  '0x1.',
  '0x20000000000001',
  '0x20000000000003',
  '0x1p-1074',
  '0x1p-1075',
  '0x1p99999',
  '0x1p-99999',
  '0xdeadbeefcafebabe1234567890abcdefp-40',
  '0e',
  '0e+',
  '1e5x',
  '9x',
  '+1',
  '+0x10',
  'nan(1)',
  'nan()',
  'NAN',
  'INF',
  'infin',
  '-infinity',
  '1000000000000000000000000000',
  '0.000000000000000000000000001',
  '4.9406564584124654e-324',
  '2.2250738585072016e-308',
  '[0x10,1e5,inf]',
  '{"a":0x10}',
}
for i, doc in ipairs(NUMDOCS) do
  local dec = try(vim.json.decode, doc)
  local enc = 'n/a'
  if type(dec) ~= 'string' or dec:sub(1, 4) ~= 'ERR ' then
    enc = esc(tostring(try(vim.json.encode, dec)))
  end
  line('n3d ' .. i, esc(doc), show(dec), enc)
end
