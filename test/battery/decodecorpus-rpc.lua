-- decodediff section 5: the msgpack-RPC *channel*, i.e.
-- crates/nvim/src/msgpack_rpc/ (unpacker/, packer/, channel/, server.rs).
--
-- Sections 1-4 all decode a msgpack document that is already in memory:
-- msgpackparse(), vim.mpack, and the two json codecs.  None of them
-- touches the streaming framer the RPC transport is built on, and the
-- survey's GAP 4 says so outright -- `envelope.rs`, `trace.rs` and
-- `server.rs` are watched by nothing except the functional suite, which
-- only ever sends WELL-FORMED messages and only ever looks at the
-- decoded value.
--
-- What this adds, in the corpus's existing form (one line per case,
-- byte-identical output is the pass):
--
--   5a  Round trips over a real rpc job channel.  Every Object type the
--       packer can emit, the msgpack width boundaries the wire format
--       pins (fixstr/str8/str16/str32, fixarray/array16/array32,
--       fixmap/map16), the three ext handle types in BOTH directions,
--       the error-response shape, and the generated keyset decoder
--       (`unpack_keydict`) including its rejection messages.
--   5b  Malformed framing, one fresh child per case, fed as RAW bytes
--       to `--embed`'s stdin.  This is the only way to reach
--       `unpacker_parse_header`'s error arms, `chan_close_on_err` and
--       the `nvim_error_event` reply path: an rpc-mode channel cannot
--       express a bad envelope.  Observables are the child's own
--       `RPC:` log lines (trace.rs's two formats plus the close
--       message), whatever bytes it wrote back, and whether the
--       malformed input killed it.
--
-- Nondeterminism, all removed by `scrub` below: channel ids, request
-- ids, pointers and byte counts in the DBG log, temp paths, and the
-- pid/timestamp prefix every log line carries.  The child is always
-- `-u NONE -i NONE --embed --headless`, so it has no init, no shada and
-- no UI.
--
-- Run as `nvim --headless -u NONE -i NONE -l <this file>`.

local NVIM = vim.v.progpath
local TMP = vim.fn.tempname()
vim.fn.mkdir(TMP, 'p')

local CHILD = { NVIM, '-u', 'NONE', '-i', 'NONE', '--embed', '--headless' }

local function esc(s)
  return (tostring(s):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- Renders a decoded value with no address, no iteration order and no
-- float ambiguity in it -- the same contract section 4's `show` has.
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
  elseif t == 'boolean' then
    return tostring(v)
  elseif t == 'number' then
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
  local keys = {}
  for k in pairs(v) do
    keys[#keys + 1] = k
  end
  table.sort(keys, function(a, b)
    return tostring(a) < tostring(b)
  end)
  local parts = {}
  for _, k in ipairs(keys) do
    local rendered = show(v[k], depth + 1)
    if type(k) == 'number' and v[1] ~= nil then
      parts[#parts + 1] = rendered
    else
      parts[#parts + 1] = esc(tostring(k)) .. '=' .. rendered
    end
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

-- Anything that names a channel, a request, an address or a clock has
-- to go: two runs of this corpus must produce the same bytes.
local function scrub(s)
  s = tostring(s)
  s = s:gsub(TMP, '$TMP')
  s = s:gsub('channel %d+', 'channel N')
  s = s:gsub('ch %d+', 'ch N')
  s = s:gsub('id=%d+', 'id=N')
  s = s:gsub('request id %d+', 'request id N')
  s = s:gsub('0x%x+', 'PTR')
  return s
end

local out = {}
local function row(label, text)
  out[#out + 1] = label .. '\t' .. text
end

-- `chansend` writes a string's bytes verbatim, so a payload is just the
-- hex spelled out.  The *reading* side is not symmetric: a job's stdout
-- arrives as readfile()-style items where a stream NUL shows up as '\n'
-- inside an item and a stream NL is an item boundary, so rejoining with
-- '\n' folds the two together.  The reply is only ever hex-dumped and
-- compared against itself, so the fold costs nothing.
local function wire(hex)
  local bytes = {}
  for pair in hex:gsub('%s', ''):gmatch('%x%x') do
    bytes[#bytes + 1] = string.char(tonumber(pair, 16))
  end
  return table.concat(bytes)
end

local function hexof(s)
  return (s:gsub('.', function(c)
    return string.format('%02x', c:byte())
  end))
end

-- A fixstr header plus its bytes, for the method names below.
local function str(s)
  assert(#s < 32)
  return string.format('%02x', 0xa0 + #s) .. hexof(s)
end

-- ---------------------------------------------------------------------
-- 5a  Round trips over an rpc channel
-- ---------------------------------------------------------------------

local job = vim.fn.jobstart(CHILD, { rpc = true })

local function call(label, ...)
  local ok, res = pcall(vim.rpcrequest, job, ...)
  if ok then
    row(label, show(res))
  else
    row(label, 'ERR ' .. esc(scrub(res)))
  end
  return ok, res
end

local function eval(label, expr)
  call('a:' .. label, 'nvim_eval', expr)
end

-- The scalar matrix.  Each of these is packed by the child's
-- `mpack_object_inner` and decoded by this side's `api_parse_enter`.
eval('nil', 'v:null')
eval('true', 'v:true')
eval('false', 'v:false')
eval('zero', '0')
eval('neg1', '-1')
eval('fixint_max', '127')
eval('uint8', '255')
eval('uint16', '65535')
eval('uint32', '4294967295')
eval('int64_max', '9223372036854775807')
eval('int64_min', '-9223372036854775807 - 1')
eval('float', '1.5')
eval('float_neg0', '-0.0')
eval('float_big', '1.0e308')
eval('str_empty', '""')
eval('str_ascii', '"abc"')
eval('str_high', 'list2str([255, 254])')
eval('str_utf8', '"\\u00e9\\u4e2d"')
eval('blob', '0z00010203')
eval('list_empty', '[]')
eval('list_nested', '[1, [2, [3, [4]]]]')
eval('dict_empty', '{}')
eval('dict', "{'a': 1, 'b': [2], 'c': {'d': 3}}")
eval('dict_emptykey', "{'': 1}")
eval('funcref', 'function("tr")')

-- The width boundaries.  Both sides pick a header width from the length,
-- and the exact boundary is wire format, not an implementation detail.
for _, n in ipairs({ 31, 32, 255, 256, 65535, 65536 }) do
  local ok, res = pcall(vim.rpcrequest, job, 'nvim_eval', ('repeat("x", %d)'):format(n))
  row('a:str_len_' .. n, ok and ('#=' .. #res) or ('ERR ' .. esc(scrub(res))))
end
for _, n in ipairs({ 15, 16, 65535, 65536 }) do
  local ok, res = pcall(vim.rpcrequest, job, 'nvim_eval', ('range(%d)'):format(n))
  row('a:arr_len_' .. n, ok and ('#=' .. #res .. ',last=' .. tostring(res[#res])) or ('ERR ' .. esc(scrub(res))))
end
for _, n in ipairs({ 15, 16 }) do
  local expr = ('{k -> k}->call([0])'):format(n)
  local _ = expr
  local ok, res = pcall(
    vim.rpcrequest,
    job,
    'nvim_eval',
    ("range(%d)->map({i -> string(i)})->reduce({acc, k -> extend(acc, {k: 1})}, {})"):format(n)
  )
  row('a:map_len_' .. n, ok and ('#=' .. #vim.tbl_keys(res)) or ('ERR ' .. esc(scrub(res))))
end

-- The three ext types, out and back.  `mpack_handle` writes them and
-- `ext_object` reads them; a handle that survives a round trip proves
-- both directions agree on the type byte.
local okbuf, buf = call('a:ext_buf', 'nvim_get_current_buf')
call('a:ext_win', 'nvim_get_current_win')
call('a:ext_tab', 'nvim_get_current_tabpage')
if okbuf then
  call('a:ext_buf_back', 'nvim_buf_line_count', buf)
end

-- Error responses: the `[type, message]` pair `report_call_error` reads.
call('a:err_parse', 'nvim_eval', '1+')
call('a:err_unknown_method', 'no_such_method_at_all')
call('a:err_arity', 'nvim_eval')
call('a:err_argtype', 'nvim_eval', 7)
call('a:err_badhandle', 'nvim_buf_line_count', 9999)

-- `unpack_keydict`, the generated keyset decoder.  Its rejection
-- messages name the offending key with `%.*s`, which nothing else in
-- the tree exercises.
call('a:keydict_ok', 'nvim_set_option_value', 'number', true, { scope = 'local' })
call('a:keydict_readback', 'nvim_get_option_value', 'number', { scope = 'local' })
call('a:keydict_wrongtype', 'nvim_get_option_value', 'number', { scope = 1 })
call('a:keydict_notmap', 'nvim_get_option_value', 'number', 1)
call('a:keydict_emptykey', 'nvim_get_option_value', 'number', { [''] = 1 })
call('a:keydict_unknownkey', 'nvim_get_option_value', 'number', { bogus_key = 1 })
call('a:keydict_boolslot', 'nvim_get_option_value', 'number', { scope = 'local', buf = false })

-- A notification carries no id and is never answered; the request that
-- follows it proves the stream stayed in step.
vim.rpcnotify(job, 'nvim_set_var', 'decodediff', 42)
call('a:after_notify', 'nvim_get_var', 'decodediff')

-- `nvim_get_mode` is the one "fast" handler that is deferred instead of
-- run from the read callback.
call('a:get_mode', 'nvim_get_mode')

vim.fn.jobstop(job)

-- ---------------------------------------------------------------------
-- 5c  `unpack()` over raw bytes
-- ---------------------------------------------------------------------
--
-- `nvim__unpack` is the one API entry point that hands arbitrary bytes
-- straight to the RPC unpacker's tree parser, so this is the only cheap
-- way to reach every arm of `api_parse_enter` and `ext_object` -- the
-- extension objects in particular, which no other corpus produces at
-- all -- and all four of `unpack`'s own error messages ("too deep",
-- "incomplete", "invalid", "trailing data").

local function unpacked(label, hex)
  local ok, res = pcall(vim.api.nvim__unpack, wire(hex))
  row('c:' .. label, ok and show(res) or ('ERR ' .. esc(scrub(res))))
end

unpacked('nil', 'c0')
unpacked('false', 'c2')
unpacked('true', 'c3')
unpacked('fixint0', '00')
unpacked('fixint127', '7f')
unpacked('negfixint', 'ff')
unpacked('uint8', 'ccff')
unpacked('uint16', 'cdffff')
unpacked('uint32', 'ceffffffff')
unpacked('uint64_max', 'cfffffffffffffffff')
unpacked('uint64_hi', 'cf8000000000000000')
unpacked('int8', 'd080')
unpacked('int16', 'd18000')
unpacked('int32', 'd280000000')
unpacked('int64_min', 'd38000000000000000')
unpacked('float32', 'ca3f800000')
unpacked('float64', 'cb3ff0000000000000')
unpacked('float_nan', 'cb7ff8000000000000')
unpacked('float_inf', 'cb7ff0000000000000')
unpacked('fixstr_empty', 'a0')
unpacked('fixstr_abc', 'a3616263')
unpacked('str_with_nul', 'a3610063')
unpacked('str_invalid_u8', 'a2fffe')
unpacked('str8', 'd9026869')
unpacked('str16', 'da00026869')
unpacked('str32', 'db000000026869')
unpacked('str_len_31', 'bf' .. string.rep('78', 31))
unpacked('str_len_32', 'd920' .. string.rep('78', 32))
unpacked('bin8', 'c4020001')
unpacked('bin16', 'c500020001')
unpacked('bin32', 'c6000000020001')
unpacked('fixarray0', '90')
unpacked('fixarray3', '93010203')
unpacked('array16', 'dc0002 0102')
unpacked('array32', 'dd00000002 0102')
unpacked('fixmap0', '80')
unpacked('fixmap_str', '82 a161 01 a162 02')
unpacked('map_dupkey', '82 a161 01 a161 02')
unpacked('map_emptykey', '81 a0 01')
unpacked('map_intkey', '81 01 02')
unpacked('map_binkey', '81 c40161 01')
unpacked('map16', 'de0001 a161 01')
unpacked('map32', 'df00000001 a161 01')
unpacked('map_nested', '81 a161 81 a162 01')
-- The three handle extensions, plus the arms `ext_object` answers nil
-- for: a type outside the three, and a payload that is not a uint.
unpacked('ext_buffer', 'd400 01')
unpacked('ext_window', 'd401 01')
unpacked('ext_tabpage', 'd402 01')
unpacked('ext_type_3', 'd403 01')
unpacked('ext_type_neg', 'd4ff 01')
unpacked('ext_payload_str', 'd400 a0')
unpacked('ext_payload_sint', 'd400 ff')
-- Exactly `EXT_PAYLOAD_MAX`, and one byte past it.
unpacked('ext_payload_9', 'c709 00 cfffffffffffffffff')
unpacked('ext_over_9', 'c70a 00' .. string.rep('41', 10))
unpacked('ext8_zero', 'c700 00')
unpacked('ext16', 'c80001 00 01')
unpacked('ext32', 'c900000001 00 01')
-- Depth: `unpack` reports "too deep" rather than failing the message.
for _, n in ipairs({ 1, 15, 16, 31, 32, 33 }) do
  unpacked('deep' .. n, string.rep('91', n) .. '01')
end
-- The four error arms.
unpacked('empty', '')
unpacked('trailing', '0102')
unpacked('trunc_str', 'a361')
unpacked('trunc_array', '93 01')
unpacked('trunc_map', '82 a161 01')
unpacked('trunc_uint', 'cf 00')
unpacked('reserved_c1', 'c1')

-- ---------------------------------------------------------------------
-- 5b  Malformed framing, one child per case
-- ---------------------------------------------------------------------

local function framing(label, hex, opts)
  opts = opts or {}
  local log = TMP .. '/' .. label .. '.log'
  local exited, chunks = nil, {}
  local j = vim.fn.jobstart(CHILD, {
    rpc = false,
    env = { NVIM_LOG_FILE = log },
    on_stdout = function(_, data)
      chunks[#chunks + 1] = table.concat(data, '\n')
    end,
    on_exit = function(_, code)
      exited = code
    end,
  })
  local payload = wire(hex)
  if opts.bytewise then
    for i = 1, #payload do
      vim.fn.chansend(j, payload:sub(i, i))
      vim.wait(2)
    end
  else
    vim.fn.chansend(j, payload)
  end
  -- Whether the child dies on its own is half the observable: a framing
  -- error closes the stdio channel, and closing stdio exits the editor.
  vim.wait(1500, function()
    return exited ~= nil
  end, 10)
  local self_closed = exited ~= nil
  if not self_closed then
    pcall(vim.fn.chanclose, j, 'stdin')
    vim.wait(3000, function()
      return exited ~= nil
    end, 10)
  end
  pcall(vim.fn.jobstop, j)

  local reply = table.concat(chunks)
  local lines = {}
  if vim.fn.filereadable(log) == 1 then
    for _, l in ipairs(vim.fn.readfile(log)) do
      -- `RPC:` is `msgpack_rpc/`'s tag.  The `Stream` lines are
      -- `event/stream.rs`'s `stream_may_close` and `event/rstream.rs`'s
      -- `read_cb`, and they are the only place the child writes down what
      -- its *event* layer did: which of its streams was closed, and
      -- whether the read that ended it was an error, an EOF or a
      -- deliberate close.  `scrub` turns the `%p` into PTR.  (P20-19.)
      local msg = l:match('RPC: .*') or l:match('clos%a+ Stream.*')
      if msg then
        lines[#lines + 1] = scrub(msg)
      end
    end
  end
  row(
    'b:' .. label,
    ('self_closed=%s exit=%s reply=%s log=[%s]'):format(
      tostring(self_closed),
      tostring(exited),
      reply == '' and '-' or (esc(reply) .. '/' .. hexof(reply)),
      esc(table.concat(lines, ' | '))
    )
  )
end

local EVAL = str('nvim_eval')
local ONE_PLUS_ONE = '91' .. str('1+1')

-- The happy path, so every arm below is read against a known-good line.
framing('ok_request', '9400 01' .. EVAL .. ONE_PLUS_ONE)
framing('ok_request_bytewise', '9400 01' .. EVAL .. ONE_PLUS_ONE, { bytewise = true })
framing('ok_two_requests', '9400 01' .. EVAL .. ONE_PLUS_ONE .. '9400 02' .. EVAL .. ONE_PLUS_ONE)
framing('ok_notification', '9302' .. str('nvim_set_var') .. '92' .. str('x') .. '01')
-- A method name may arrive as bin: `token_matches` treats STR and BIN as
-- interchangeable, and clients differ on which they send.
framing('ok_bin_method', '9400 01 c409' .. hexof('nvim_eval') .. ONE_PLUS_ONE)

-- `unpacker_parse_header`'s rejection arms, one per condition.
framing('hdr_not_array', 'c0')
framing('hdr_array_len2', '9200 01')
framing('hdr_array_len5', '9500 01' .. EVAL .. ONE_PLUS_ONE .. 'c0')
framing('hdr_len3_request', '9300 01' .. EVAL)
framing('hdr_len4_notification', '9402' .. str('nvim_set_var') .. '92' .. str('x') .. '01')
framing('hdr_type_5', '9405 01' .. EVAL .. ONE_PLUS_ONE)
framing('hdr_type_not_uint', '9400 a130' .. EVAL .. ONE_PLUS_ONE)
framing('hdr_id_not_uint', '9400' .. str('x') .. EVAL .. ONE_PLUS_ONE)
framing('hdr_method_not_str', '9400 01 05' .. ONE_PLUS_ONE)
framing('hdr_method_empty', '9400 01 a0 90')
framing('hdr_method_100', '9400 01 d964' .. hexof(string.rep('a', 100)) .. '90')
framing('hdr_method_101', '9400 01 d965' .. hexof(string.rep('a', 101)) .. '90')
framing('hdr_reserved_c1', 'c1')
framing('hdr_unknown_method', '9400 01' .. str('no_such') .. '90')
-- `Invalid method: ` is sixteen bytes, so these two error messages are
-- exactly 31 and 32 bytes long -- the fixstr/str8 boundary the packer's
-- width table picks, which is wire format and which nothing else here
-- can see. The reply is hex-dumped, so the header byte is the answer.
framing('hdr_unknown_method_31', '9400 01' .. str('no_such_method!') .. '90')
framing('hdr_unknown_method_32', '9400 01' .. str('no_such_method!!') .. '90')

-- The body, once the header has been accepted.
framing('body_args_not_array', '9400 01' .. EVAL .. str('x'))
framing('body_args_map', '9400 01' .. EVAL .. '81' .. str('a') .. '01')
framing('body_deep_32', '9400 01' .. EVAL .. string.rep('91', 32) .. '01')
framing('body_deep_33', '9400 01' .. EVAL .. string.rep('91', 33) .. '01')
framing('body_truncated', '9400 01' .. EVAL .. '9200')

-- Responses.  Only a request this editor sent has a frame to complete,
-- so every one of these is an unknown id.
framing('resp_unknown_id', '9401 07 c0 c0')
framing('resp_errored', '9401 07 92 01' .. str('boom') .. 'c0')

-- The extension objects the wire uses for handles.
framing('ext_buffer_0', '9400 01' .. str('nvim_buf_line_count') .. '91 d400 00')
framing('ext_type_7', '9400 01' .. str('nvim_buf_line_count') .. '91 d407 01')
framing('ext_payload_str', '9400 01' .. str('nvim_buf_line_count') .. '91 d400 a0')

-- `unpack_keydict`.  Only a raw message can present a duplicate key or
-- a non-string one; a Lua table cannot.
local OPTVAL = str('nvim_get_option_value')
local SCOPE_LOCAL = str('scope') .. str('local')
framing('kd_ok', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81' .. SCOPE_LOCAL)
framing('kd_duplicate', '9400 01' .. OPTVAL .. '92' .. str('number') .. '82' .. SCOPE_LOCAL .. SCOPE_LOCAL)
framing('kd_empty_key', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81 a0 01')
framing('kd_int_key', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81 01 01')
framing('kd_not_map', '9400 01' .. OPTVAL .. '92' .. str('number') .. '90')
framing('kd_wrong_value', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81' .. str('scope') .. '01')
framing('kd_unknown_key', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81' .. str('bogus') .. '01')
framing('kd_bool_slot', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81' .. str('buf') .. 'c2')
framing('kd_bool_slot_bad', '9400 01' .. OPTVAL .. '92' .. str('number') .. '81' .. str('buf') .. 'a1 78')

for _, line in ipairs(out) do
  print(line)
end
vim.fn.delete(TMP, 'rf')
