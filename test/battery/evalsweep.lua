-- Driver for the eval-substrate differential sweep; see
-- evalsweep.sh.
--
-- Covers the value layer that batch B14 rewrites:
--
--   §1  eval/typval.rs, eval/encode.rs, eval/decode.rs
--       string() / :echo / json_encode() / msgpackdump() are four of the
--       six instantiations of TYPVAL_ENCODE_DEFINE_CONV_FUNCTIONS, run
--       over one shared corpus so a rewrite that collapses them into a
--       generic walker has to keep every sink answering what it does
--       today.  json_decode()/msgpackparse() close the round trips, and
--       the typval half (type/empty/len/copy/deepcopy/lock/compare/
--       sort/identity/coercion) is asked directly.
--
--   §2  lua/converter.rs, api/private/converter.rs
--       The other two instantiations, plus their inverses.  Four
--       distinct paths, and the sweep drives all four over the same
--       corpus:
--         typval -> lua      vim.fn.eval()          nlua_push_typval
--         typval -> Object   nvim_eval()            vim_to_object
--         lua    -> typval   vim.fn.string(v)       nlua_pop_typval
--         lua    -> Object   nvim_call_function()   nlua_pop_Object
--       Plus luaeval()/v:lua from the Vimscript side and the table
--       ambiguities (list vs dict, holes, vim.NIL, vim.empty_dict(),
--       vim.type_idx/val_idx) the converter has to disambiguate.
--
--   §3  api/private/validate.rs and the generated dispatch wrappers
--       The exact text of a rejected nvim_* call.  Error strings are
--       what a rewrite silently changes and what no type checker sees:
--       api_err_invalid/api_err_exp/api_err_required/api_err_conflict
--       each pick between two format strings on `strchr(name, ' ')`,
--       and check_string_array picks between two more.  Every arm here
--       is reached by a real call.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time or path outside the work directory.
-- Three artifacts: the readable report on stdout, a canonical
-- (sorted-key) JSON dump on $EVAL_STRUCT of every structured answer,
-- and stderr -- which in a headless process is where nvim's own
-- messages go, and is the only view of some of them.
--
-- EVALSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.

local work = assert(os.getenv('EVAL_WORK'), 'EVAL_WORK unset')
local structpath = assert(os.getenv('EVAL_STRUCT'), 'EVAL_STRUCT unset')
local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('EVALSWEEP_ONLY')
if only == '' then
  only = nil
end

-- Unbuffered: nvim's own messages go to stderr, but a Lua error would
-- otherwise lose the tail of the report.
io.stdout:setvbuf('line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before: a path is the
--- classic answer that differs between two working directories and
--- survives the three-runs rule.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>'):gsub('%.%.%.[^%s\'"]-evalsweep%.lua', '<SCRIPT>')  -- LuaJIT elides a chunk name past ~60 chars
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  -- id() and a Lua table's tostring() are addresses.  They are recorded
  -- only as equal-or-not (see the identity section); the raw value can
  -- still leak into an error text, so mask it here too.
  text = text:gsub('0x%x+', '<ADDR>')
  -- Lambdas and anonymous functions are numbered from a global counter.
  -- The count is stable for a fixed section order, but it is one edit
  -- away from re-baselining half the artifact, so mask it.
  text = text:gsub('<lambda>%d+', '<lambda>')
  return text
end

--- Escape to one printable line, so a byte difference shows in the diff
--- and a report line stays a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  vim.json.encode walks a Lua table in hash order,
-- which is stable within a build but is not something to bet a byte
-- oracle on; keys are sorted here instead.  Empty tables are tagged so
-- an empty list and an empty dict stay distinguishable -- that
-- distinction is the whole point of §2.
-- ---------------------------------------------------------------------
local canon

local function canon_number(value)
  if value ~= value then
    return '"nan"'
  elseif value == math.huge then
    return '"inf"'
  elseif value == -math.huge then
    return '"-inf"'
  elseif value == math.floor(value) and math.abs(value) < 2 ^ 53 then
    -- LuaJIT has one number type, so a Vim Number and an integral Vim
    -- Float arrive here indistinguishable.  Which of the two a
    -- converter produced is asked of Vimscript instead (`type()` after
    -- the trip back), not of this formatter.
    return string.format('%d', value)
  end
  return string.format('%.17g', value)
end

canon = function(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'nil' then
    return 'nil'
  elseif kind == 'number' then
    return canon_number(value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    return '"<' .. kind .. '>"'
  end
  if next(value) == nil then
    -- vim.empty_dict() carries a metatable; a bare {} does not, and the
    -- converters answer differently for the two.
    return getmetatable(value) and '{}' or '[]'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  -- Keys are sorted by their rendered form.  A non-string key is
  -- rendered in angle brackets so it cannot collide with a string key
  -- of the same spelling -- which is not hypothetical: the api's
  -- `special` Object representation of a Float is
  -- `{[vim.type_idx] = vim.types.float, [vim.val_idx] = n}`, and
  -- vim.type_idx/vim.val_idx are the booleans `true` and `false`.
  local keys, byname = {}, {}
  for key in pairs(value) do
    -- A table or function key would otherwise render as its address,
    -- which is the one thing an artifact may not carry; recurse instead.
    local name = type(key) == 'string' and esc(key)
      or ('<' .. (type(key) == 'table' and canon(key) or scrub(tostring(key))) .. '>')
    keys[#keys + 1] = name
    byname[name] = key
  end
  table.sort(keys)
  local parts = {}
  for _, name in ipairs(keys) do
    parts[#parts + 1] = '"' .. name .. '":' .. canon(value[byname[name]])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, ' ', canon(value), '\n')
end

-- ---------------------------------------------------------------------
-- Asking things.  An error is observable behaviour -- for the encoders
-- it is often the *only* observable behaviour -- so every failure is
-- reported, never swallowed.
-- ---------------------------------------------------------------------

--- Normalise an error to its message: a pcall against vim.fn or
--- vim.api prefixes the Lua source position, which is a line number in
--- this file and would re-baseline the artifact on any edit.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

--- Evaluate a Vimscript expression that answers a *string*, and report
--- it.  §1 deliberately stays inside Vimscript: routing an answer
--- through a Lua value would mix §2's converters into §1's question.
local function veval(label, expr)
  local ok, res = pcall(vim.fn.eval, expr)
  if not ok then
    emit(label, '!', esc(errtext(res)))
    struct(label, { err = errtext(res) })
    return nil
  end
  emit(label, '=', esc(scrub(res)))
  struct(label, res)
  return res
end

--- Report a Lua-side answer under a label, both readably and
--- canonically.
local function answer(label, value)
  emit(label, '=', esc(canon(value)))
  struct(label, value)
  return value
end

--- Call something, answering its error text rather than dying.
local function attempt(label, fn, ...)
  local ok, res = pcall(fn, ...)
  if ok then
    return answer(label, res)
  end
  emit(label, '!', esc(errtext(res)))
  struct(label, { err = errtext(res) })
  return nil
end

local function command(cmd)
  local ok, res = pcall(vim.api.nvim_command, cmd)
  if not ok then
    return errtext(res)
  end
  return nil
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- The corpus.  One ordered list, built into g:C_<name> once, then asked
-- by every §1 and §2 section.  Order is part of the artifact: a value
-- inserted in the middle re-numbers nothing (names, not indices), but
-- it does move every later line, so append.
--
-- What earns a place here is a value the six encoder instantiations do
-- *not* agree about: NUL-bearing and invalid-UTF-8 strings, NaN/inf,
-- blobs, the v:msgpack_types special dicts, funcrefs/partials, locked
-- containers and self-reference.  A corpus of well-formed ASCII gates
-- nothing, because every sink answers those the same way.
-- ---------------------------------------------------------------------
local CORPUS = {
  -- numbers
  { 'num_zero', '0' },
  { 'num_pos', '42' },
  { 'num_neg', '-7' },
  { 'num_max', '9223372036854775807' },
  { 'num_min', '-9223372036854775807 - 1' },
  { 'num_53', '9007199254740993' },
  { 'num_hex', '0x7fffffff' },
  -- floats
  { 'flt_zero', '0.0' },
  { 'flt_negzero', '-0.0' },
  { 'flt_one', '1.0' },
  { 'flt_third', '1.0 / 3.0' },
  { 'flt_big', '1.0e100' },
  { 'flt_small', '1.0e-100' },
  { 'flt_nan', '0.0 / 0.0' },
  { 'flt_inf', '1.0 / 0.0' },
  { 'flt_ninf', '-1.0 / 0.0' },
  -- strings
  { 'str_empty', "''" },
  { 'str_ascii', "'hello'" },
  { 'str_quotes', [["a'b\"c"]] },
  { 'str_backslash', [["a\\b"]] },
  { 'str_newline', [["a\nb"]] },
  { 'str_tab', [["a\tb"]] },
  { 'str_ctrl', [["\x01\x02\x1f\x7f"]] },
  { 'str_nul', 'nr2char(0)' },
  { 'str_utf8', "'ab\206\177\206\178\230\151\165'" },
  { 'str_badutf8', [["\xc3\x28\xff"]] },
  { 'str_latin1', [["\xff\xfe\x80"]] },
  { 'str_solidus', [["a/b</script>"]] },
  { 'str_long', "repeat('xy', 200)" },
  -- blobs
  { 'blob_empty', '0z' },
  { 'blob_bytes', '0z00112233FF' },
  { 'blob_nul', '0z000000' },
  -- lists
  { 'list_empty', '[]' },
  { 'list_nums', '[1, 2, 3]' },
  { 'list_mixed', "[1, 'two', 3.5, v:null, v:true, v:false, [], {}, 0z00]" },
  { 'list_nested', '[[[[1]]], [[2], 3], 4]' },
  { 'list_deep', "eval('[' . repeat('[', 30) . '1' . repeat(']', 30) . ']')" },
  -- dicts
  { 'dict_empty', '{}' },
  { 'dict_simple', "{'a': 1, 'b': 2}" },
  { 'dict_numkeys', "{'1': 1, '10': 2, '2': 3}" },
  { 'dict_emptykey', "{'': 1}" },
  { 'dict_oddkeys', "{\"a\\nb\": 1, \"\\xff\": 2, 'a\"b': 3}" },
  { 'dict_nested', "{'a': {'b': {'c': [1, {'d': 2}]}}}" },
  -- specials
  { 'sp_null', 'v:null' },
  { 'sp_true', 'v:true' },
  { 'sp_false', 'v:false' },
  -- callables
  { 'fn_ref', "function('tr')" },
  { 'fn_partial', "function('tr', ['a'])" },
  { 'fn_partial_dict', "function('tr', ['a'], {'self': 1})" },
  { 'fn_lambda', '{ x -> x + 1 }' },
  -- msgpack special dicts: encode.rs's other half, and the only reason
  -- msgpackparse()/msgpackdump() round-trip at all.
  { 'mp_string', "{'_TYPE': v:msgpack_types.string, '_VAL': ['ab', 'cd']}" },
  { 'mp_str_bad', "{'_TYPE': v:msgpack_types.string, '_VAL': [\"a\\xffb\", \"c\"]}" },
  { 'mp_ext', "{'_TYPE': v:msgpack_types.ext, '_VAL': [3, ['payload']]}" },
  { 'mp_map', "{'_TYPE': v:msgpack_types.map, '_VAL': [[1, 'one'], [[], 2]]}" },
  { 'mp_array', "{'_TYPE': v:msgpack_types.array, '_VAL': [1, 2]}" },
  { 'mp_bool', "{'_TYPE': v:msgpack_types.boolean, '_VAL': 1}" },
  { 'mp_nil', "{'_TYPE': v:msgpack_types.nil, '_VAL': 0}" },
  { 'mp_int', "{'_TYPE': v:msgpack_types.integer, '_VAL': [1, 0, 0, 12]}" },
  { 'mp_float', "{'_TYPE': v:msgpack_types.float, '_VAL': 0.5}" },
  { 'mp_bad', "{'_TYPE': v:msgpack_types.string, '_VAL': 'notalist'}" },
  -- self-reference and sharing: copyID territory
  {
    'rec_list',
    nil,
    setup = { 'let g:C_rec_list = [1, 2]', 'call add(g:C_rec_list, g:C_rec_list)' },
  },
  {
    'rec_dict',
    nil,
    setup = { "let g:C_rec_dict = {'a': 1}", 'let g:C_rec_dict.self = g:C_rec_dict' },
  },
  {
    'shared',
    nil,
    setup = {
      'let g:C_inner = [1, 2]',
      'let g:C_shared = [g:C_inner, g:C_inner]',
    },
  },
  -- locked containers
  {
    'locked_list',
    nil,
    setup = { 'let g:C_locked_list = [1, [2]]', 'lockvar g:C_locked_list' },
  },
  {
    'locked_deep',
    nil,
    setup = { "let g:C_locked_deep = {'a': [1]}", 'lockvar! g:C_locked_deep' },
  },
  -- String payloads whose *ownership* differs: the null string (a String
  -- holding no allocation at all), multibyte text around a newline, and
  -- funcrefs whose names are a global, a script-local `<SNR>` and a
  -- partial over a user function.
  { 'str_null', 'v:_null_string' },
  { 'str_mbnl', [["é\n中"]] },
  {
    'fn_user',
    nil,
    setup = {
      "call execute(['function! EvalSweepF(...)', 'return a:000', 'endfunction'])",
      "let g:C_fn_user = function('EvalSweepF')",
    },
  },
  {
    'fn_script',
    nil,
    exec = table.concat({
      'function! s:SweepS()',
      '  return 2',
      'endfunction',
      "let g:C_fn_script = function('s:SweepS')",
    }, '\n'),
  },
  { 'fn_user_partial', "function('EvalSweepF', [v:_null_string, ''])" },
}

local NAMES = {}

local function build_corpus()
  for _, entry in ipairs(CORPUS) do
    local name = entry[1]
    NAMES[#NAMES + 1] = name
    local cmds = entry.setup or entry.exec and {} or { 'let g:C_' .. name .. ' = ' .. entry[2] }
    if entry.exec then
      -- A script of its own, for what needs a script context (`s:`).
      local ok, res = pcall(vim.api.nvim_exec2, entry.exec, {})
      if not ok then
        emit('corpus', name, '! SETUP', esc(errtext(res)))
      end
    end
    for _, cmd in ipairs(cmds) do
      local err = command(cmd)
      if err then
        emit('corpus', name, '! SETUP', esc(err))
      end
    end
  end
  emit('corpus', 'count', #NAMES)
end

--- Ask one Vimscript expression per corpus value.  `%s` in the template
--- is the g: name.
local function per_value(prefix, template)
  for _, name in ipairs(NAMES) do
    veval(prefix .. ' ' .. name, (template:gsub('%%s', 'g:C_' .. name)))
  end
end

-- ---------------------------------------------------------------------
-- §1 -- typval / encode / decode
-- ---------------------------------------------------------------------

section('s1-string', function()
  -- encode_vim_to_string: the sink `string()` uses.
  per_value('string', 'string(%s)')
  -- And the round trip back through the Vimscript parser.  A value that
  -- does not survive it is still an answer -- eval() failing is what
  -- funcrefs-with-dicts and self-reference do.
  per_value('rtstring', 'string(eval(string(%s)))')
end)

section('s1-echo', function()
  -- encode_vim_to_echo: a *different* instantiation from string(), and
  -- the two differ in a handful of the macro's hooks.  execute() sets
  -- msg_silent, which is fine here: the question is what the encoder
  -- emitted, not whether the pager ran.
  per_value('echo', "execute('echo %s')")
  per_value('echon', "execute('echon %s')")
end)

section('s1-json', function()
  -- encode_vim_to_json, plus json_decode() closing the loop.
  per_value('json', 'json_encode(%s)')
  per_value('rtjson', 'string(json_decode(json_encode(%s)))')
end)

section('s1-jsondec', function()
  -- decode.rs asked directly: parse_json_string / parse_json_number and
  -- the error texts, which are the half no round trip reaches.
  local TEXTS = {
    '{}',
    '[]',
    'null',
    'true',
    'false',
    '0',
    '-0',
    '1e3',
    '1E3',
    '1.5',
    '-1.5e-3',
    '1.0e400',
    '12345678901234567890',
    '9223372036854775807',
    '9223372036854775808',
    '-9223372036854775808',
    '01',
    '.5',
    '5.',
    '+1',
    '"a"',
    '"a b"',
    '"a\tb"',
    '""',
    '"\\u0000"',
    '"\\u00e9"',
    '"\\ud83d\\ude00"',
    '"\\ud83d"',
    '"\\udc00"',
    '"\\/"',
    '"\\b\\f\\n\\r\\t"',
    '"\\x41"',
    '"unterminated',
    '"\\u12"',
    '[1,2,]',
    '[1 2]',
    '{"a":1,}',
    '{a:1}',
    '{"a" 1}',
    '{"a":1,"a":2}',
    '{"":1}',
    ' \t\r\n[1]\t ',
    '[1][2]',
    '',
    '  ',
    'nul',
    'NaN',
    'Infinity',
    '[[[[[[[[[[1]]]]]]]]]]',
    '"\\ud83d\\ud83d"',
    '\xc3\x28',
    '"\xc3\x28"',
    '[null,true,false]',
    '{"_TYPE":1}',
    '{"a":{"b":[1,{"c":null}]}}',
  }
  for index, text in ipairs(TEXTS) do
    local label = string.format('jsondec %02d %s', index, esc(text))
    attempt(label, vim.fn.json_decode, text)
    -- json_decode() and the Vimscript-level call answer through the
    -- same decoder, but the Lua caller converts the result; ask for the
    -- Vimscript rendering too so §1 stays inside Vimscript.
    veval(label .. ' str', 'string(json_decode(' .. vim.fn.string(text) .. '))')
  end
end)

section('s1-msgpack', function()
  -- encode_vim_to_msgpack, as bytes and as the string list, plus
  -- msgpackparse() closing the loop.
  per_value('mpdump', "string(msgpackdump([%s]))")
  per_value('mpdumpb', "string(msgpackdump([%s], 'B'))")
  per_value('rtmp', 'string(msgpackparse(msgpackdump([%s])))')
  per_value('rtmpb', "string(msgpackparse(msgpackdump([%s], 'B')))")
end)

section('s1-msgpackparse', function()
  -- Hand-written msgpack, so the decoder's arms are reached without
  -- going through the encoder first: every family of format byte, and
  -- the truncated/invalid shapes.
  local BLOBS = {
    '0z', -- empty
    '0z00', -- fixint 0
    '0z7F', -- fixint 127
    '0zFF', -- negative fixint -1
    '0zCC80', -- uint8 128
    '0zCD0100', -- uint16 256
    '0zCE00010000', -- uint32
    '0zCF0000000100000000', -- uint64
    '0zD0FF', -- int8 -1
    '0zD1FF00', -- int16
    '0zD2FFFFFFFF', -- int32
    '0zD3FFFFFFFFFFFFFFFF', -- int64
    '0zCA3F800000', -- float32 1.0
    '0zCB3FF0000000000000', -- float64 1.0
    '0zCB7FF0000000000000', -- float64 +inf
    '0zCB7FF8000000000000', -- float64 nan
    '0zC0', -- nil
    '0zC2', -- false
    '0zC3', -- true
    '0zA0', -- fixstr ""
    '0zA161', -- fixstr "a"
    '0zA2C328', -- fixstr, invalid utf-8
    '0zA30061FF', -- fixstr with an embedded NUL
    '0zC403000102', -- bin8
    '0zC50004DEADBEEF', -- bin16
    '0z90', -- fixarray []
    '0z9301A16101', -- fixarray [1,"a",1]
    '0z80', -- fixmap {}
    '0z81A16101', -- fixmap {"a":1}
    '0z8101A161', -- fixmap {1:"a"} -- a non-string key
    '0z8190A161', -- fixmap {[]:"a"} -- an unrepresentable key
    '0zD40100', -- fixext1
    '0zC7000300', -- ext8, zero length
    '0zC70102AABB', -- ext8
    '0zCC', -- truncated
    '0zA5', -- truncated fixstr
    '0z9101', -- fixarray of one
    '0z01C0', -- two objects in one stream
    '0zC1', -- never-used byte
    '0z93C0C2C3', -- [nil,false,true]
  }
  for index, blob in ipairs(BLOBS) do
    local label = string.format('mpparse %02d %s', index, blob)
    veval(label, 'string(msgpackparse(' .. blob .. '))')
    veval(label .. ' rt', "string(msgpackdump(msgpackparse(" .. blob .. "), 'B'))")
  end
end)

section('s1-meta', function()
  -- typval.rs asked directly: the shape questions.
  per_value('type', 'type(%s)')
  per_value('empty', 'empty(%s)')
  per_value('len', 'len(%s)')
  per_value('count', "string(count([%s, %s], %s))")
  per_value('get', "string(get(%s, 0, 'DEF'))")
  per_value('getk', "string(get(%s, 'a', 'DEF'))")
  per_value('bool', 'string(!!%s)')
  per_value('str', 'string(%s . "")')
  per_value('nr', 'string(%s + 0)')
end)

section('s1-copy', function()
  -- tv_copy / tv_dict_copy / tv_list_copy and the deep variants.
  per_value('copy', 'string(copy(%s))')
  per_value('deepcopy', 'string(deepcopy(%s))')
  per_value('deepcopy-noref', 'string(deepcopy(%s, 1))')
  -- Identity, not address: whether copy() shares the inner container
  -- and deepcopy() does not is the assertion, and id() answers an
  -- address that no artifact may carry.
  for _, name in ipairs(NAMES) do
    local var = 'g:C_' .. name
    veval(
      'ident ' .. name,
      string.format(
        "string([%s is %s, copy(%s) is %s, deepcopy(%s) is %s, %s == deepcopy(%s)])",
        var,
        var,
        var,
        var,
        var,
        var,
        var,
        var
      )
    )
    -- tv_copy resets v_lock on the copy; the only way to see that is to
    -- put the copy in a variable and ask.
    command('silent! unlockvar! g:CP')
    command('silent! unlet! g:CP')
    veval(
      'copylock ' .. name,
      string.format(
        "execute('let g:CP = copy(g:C_%s)') . string([islocked('g:CP'), islocked('g:C_%s')])",
        name,
        name
      )
    )
    veval(
      'deepcopylock ' .. name,
      string.format("execute('let g:CP = deepcopy(g:C_%s)') . string(islocked('g:CP'))", name)
    )
    command('silent! unlockvar! g:CP')
    command('silent! unlet! g:CP')
  end
end)

section('s1-copydepth', function()
  -- var_item_copy / tv_list_copy / tv_dict_copy: the *depth* of a copy,
  -- which no rendering of the copy can show.  The assertion is what
  -- happens to the ORIGINAL when the copy is written through: a shallow
  -- copy shares its inner containers, a deep one does not.
  --
  -- The fixtures are rebuilt here rather than taken from the corpus:
  -- these cases write, and a probe that writes to the corpus makes every
  -- later section a function of this one.
  local SHAPES = {
    { 'list', "[1, [2, [3]], 'a']" },
    { 'dict', "{'a': {'b': [1]}, 'c': 1}" },
    { 'mixed', "[{'a': [1]}, [{'b': 2}]]" },
    { 'blobs', '[0z0011, 0z2233]' },
    { 'flat', "[1, 2, 3]" },
  }
  local KINDS = {
    { 'copy', 'copy(g:T)' },
    { 'deepcopy', 'deepcopy(g:T)' },
    { 'deepcopy1', 'deepcopy(g:T, 1)' },
  }
  local WRITES = {
    "let g:A[0] = 'MUT'",
    "let g:A[1][0] = 'MUTI'",
    "let g:A[1][1][0] = 'MUTII'",
    "let g:A['a']['b'][0] = 'MUTK'",
    "call add(g:A, 'MUTA')",
    "call add(g:A[1], 'MUTIA')",
  }
  for _, shape in ipairs(SHAPES) do
    for _, kind in ipairs(KINDS) do
      command('silent! unlet! g:T')
      command('silent! unlet! g:A')
      command('let g:T = ' .. shape[2])
      local err = command('let g:A = ' .. kind[2])
      local label = string.format('copydepth %s %s', shape[1], kind[1])
      if err then
        emit(label, '! SETUP', esc(err))
      else
        for _, write in ipairs(WRITES) do
          command('silent! ' .. write)
        end
        veval(label, 'string([g:T, g:A])')
      end
    end
  end
  command('silent! unlet! g:T')
  command('silent! unlet! g:A')
end)

section('s1-lock', function()
  -- tv_item_lock / islocked, including the recursive arm.
  --
  -- NOTHING HERE MAY WRITE TO THE CORPUS.  The first draft asked
  -- `extend(g:C_<name>, {"zz": 1})` of every value to get the
  -- locked-write error text, which silently added a `zz` key to every
  -- corpus dict and made every later section's answer a function of
  -- this one.  The write probes run over values this section builds and
  -- throws away.
  for _, name in ipairs(NAMES) do
    veval('locked ' .. name, string.format("string(islocked('g:C_%s'))", name))
    veval('lockedi ' .. name, string.format("string(islocked('g:C_%s[0]'))", name))
    veval('lockedk ' .. name, string.format("string(islocked('g:C_%s.a'))", name))
  end
  -- The lock matrix, over throwaway values.  `lockvar` locks the
  -- variable, `lockvar!` locks it and everything under it, and
  -- `lockvar N` stops N levels down; the three answer differently only
  -- for a nested container, so the fixture is one.
  local DEPTHS = { 'lockvar', 'lockvar!', 'lockvar 0', 'lockvar 1', 'lockvar 2', 'lockvar 3' }
  local SHAPES = {
    { 'list', "[1, [2, [3]]]" },
    { 'dict', "{'a': {'b': {'c': 1}}}" },
    { 'blob', '0z001122' },
    { 'str', "'abc'" },
    { 'num', '7' },
  }
  local WRITES = {
    { 'extend', "call extend(g:L, has_key(g:L, 'a') ? {'zz': 1} : [9])" },
    { 'add', 'call add(g:L, 9)' },
    { 'remove0', 'call remove(g:L, 0)' },
    { 'removea', "call remove(g:L, 'a')" },
    { 'inner', "let g:L[0][0] = 99" },
    { 'innerk', "let g:L['a']['b'] = 99" },
    { 'assign', 'let g:L = 0' },
    { 'unlet', 'unlet g:L' },
    { 'map', "call map(g:L, 'v:val')" },
    { 'sort', 'call sort(g:L)' },
    { 'filter', "call filter(g:L, '1')" },
  }
  for _, shape in ipairs(SHAPES) do
    for _, depth in ipairs(DEPTHS) do
      for _, write in ipairs(WRITES) do
        local label = string.format('lockw %s %s %s', shape[1], depth:gsub(' ', '_'), write[1])
        command('silent! unlet! g:L')
        command('let g:L = ' .. shape[2])
        command('silent! ' .. depth .. ' g:L')
        veval(
          label,
          string.format(
            "execute('try | %s | catch | echo v:exception | endtry')",
            write[2]:gsub("'", "''")
          )
        )
        veval(label .. ' after', "exists('g:L') ? string(g:L) : 'GONE'")
        command('silent! unlockvar! g:L')
      end
    end
  end
  command('silent! unlet! g:L')
end)

section('s1-compare', function()
  -- tv_equal and the comparison operators over every ordered pair of a
  -- small but deliberately awkward set.  This is the section a
  -- tv_equal arm swap has to answer to.
  local OPS = { '==', '!=', '>', '<', '>=', '<=', 'is', 'isnot', '==#', '==?' }
  local VALS = {
    '0',
    '1',
    '-1',
    '0.0',
    '1.0',
    "''",
    "'0'",
    "'1'",
    "'a'",
    "'A'",
    'v:null',
    'v:true',
    'v:false',
    '[]',
    '[1]',
    "['a']",
    "['A']",
    '{}',
    "{'a': 1}",
    "{'a': 'b'}",
    "{'a': 'B'}",
    '0z',
    '0z00',
    '0.0 / 0.0',
  }
  for _, op in ipairs(OPS) do
    for i, a in ipairs(VALS) do
      for j, b in ipairs(VALS) do
        local label = string.format('cmp %s %02d %02d', op, i, j)
        veval(label, string.format('string(%s %s %s)', a, op, b))
      end
    end
  end
end)

section('s1-coerce', function()
  -- The string<->number edges: tv_get_number/tv_get_string and their
  -- "strict" siblings.
  local EXPRS = {
    "'3abc' + 1",
    "'abc' + 1",
    "'0x10' + 0",
    "'012' + 0",
    "'0o17' + 0",
    "'0b101' + 0",
    "' 42 ' + 0",
    "'-42' + 0",
    "'+42' + 0",
    "'1e3' + 0",
    "'1.5' + 0",
    "'.5' + 0",
    '1 . 2',
    '1.5 . ""',
    "'5' * '4'",
    "'5' / '0'",
    "'5' % '0'",
    '5 / 0',
    '-5 / 0',
    '0 / 0',
    '5 % 0',
    '1.0 / 0',
    "str2nr('0x1f', 16)",
    "str2nr('17', 8)",
    "str2nr('101', 2)",
    "str2nr('99999999999999999999')",
    "str2float('1.5e3')",
    "str2float('nan')",
    "str2float('inf')",
    "str2float('-inf')",
    "str2float('abc')",
    'float2nr(1.9)',
    'float2nr(-1.9)',
    'float2nr(1.0e100)',
    'float2nr(0.0 / 0.0)',
    "printf('%d', 1.0)",
    "printf('%s', 1.0)",
    "printf('%f', 1.0 / 3.0)",
    "printf('%g', 1.0 / 3.0)",
    "printf('%e', 1.0 / 3.0)",
    "printf('%.17g', 1.0 / 3.0)",
    "printf('%g', 0.0 / 0.0)",
    "printf('%f', 1.0 / 0.0)",
    "printf('%s', 0.0 / 0.0)",
    "printf('%s', [1, 2])",
    "printf('%s', {'a': 1})",
    "printf('%s', v:null)",
    "string(0.1 + 0.2)",
    "string(1.0e15)",
    "string(1.0e16)",
    "string(1.0e-5)",
    "string(123456789.123456789)",
    "1 == 1.0",
    "'1' == 1",
    "'1.0' == 1.0",
    "v:true == 1",
    "v:null == 0",
    "v:null == ''",
  }
  for index, expr in ipairs(EXPRS) do
    veval(string.format('coerce %02d %s', index, esc(expr)), 'string(' .. expr .. ')')
  end
end)

section('s1-listops', function()
  -- The list/dict primitives typval.rs owns, over shapes whose answers
  -- differ: sort()'s flags, uniq(), reverse(), extend(), remove(),
  -- insert(), index(), flatten(), the slice arms.
  local EXPRS = {
    "sort([3, 1, 2, 10, -1])",
    "sort(['b', 'a', 'B', 'A', '10', '9'])",
    "sort(['b', 'a', 'B', 'A'], 'i')",
    "sort(['10', '9', '1e2'], 'n')",
    "sort(['10', '9', '1e2'], 'N')",
    "sort(['10', '9', '1e2'], 'f')",
    "sort([1, 'a', 2.0, v:null, [], {}, v:true])",
    "sort([1, 'a', 2.0, v:null, [], {}, v:true], 'l')",
    "uniq([1, 1, 2, 2, 1])",
    "uniq(['a', 'A', 'a'], 'i')",
    "reverse([1, 2, 3])",
    "extend([1], [2, 3])",
    "extend([1, 2], [3], 0)",
    "extend({'a': 1}, {'a': 2, 'b': 3})",
    "extend({'a': 1}, {'a': 2}, 'keep')",
    "insert([1, 2], 0)",
    "insert([1, 2], 0, 1)",
    "insert([1, 2], 0, -1)",
    "index([1, 2, 3], 2)",
    "index(['a', 'A'], 'A', 0, 1)",
    "flatten([1, [2, [3, [4]]]])",
    "flatten([1, [2, [3, [4]]]], 1)",
    "[1, 2, 3, 4][1:2]",
    "[1, 2, 3, 4][-2:]",
    "[1, 2, 3, 4][3:1]",
    "'abcdef'[1:3]",
    "'abcdef'[-2:]",
    "0z00112233[1:2]",
    "add([], [])",
    "join([1, 'a', 2.0], '-')",
    "split('a,b,,c', ',')",
    "split('a,b,,c', ',', 1)",
    "keys({'b': 1, 'a': 2, '10': 3, '2': 4})",
    "values({'b': 1, 'a': 2})",
    "items({'b': 1, 'a': 2})",
    "map([1, 2, 3], 'v:val * 2')",
    "filter([1, 2, 3], 'v:val > 1')",
    "map({'a': 1, 'b': 2}, 'v:val * 2')",
    "reduce([1, 2, 3], { a, b -> a + b }, 0)",
    "max([1, 2, 3])",
    "min([])",
    "max({'a': 5, 'b': 2})",
    "repeat([1, 2], 3)",
    "repeat('ab', 3)",
    "repeat(0z0011, 2)",
    "remove([1, 2, 3], 1)",
    "remove([1, 2, 3], 0, 1)",
    "remove({'a': 1, 'b': 2}, 'a')",
    "remove(0z001122, 1)",
    "list2blob([1, 2, 255])",
    "blob2list(0z001122)",
    "list2str([104, 105])",
    "str2list('hi')",
    "list2str([0x4e2d])",
    "str2list('\226\150\136')",
  }
  for index, expr in ipairs(EXPRS) do
    veval(string.format('listop %02d %s', index, esc(expr)), 'string(' .. expr .. ')')
  end
end)

-- ---------------------------------------------------------------------
-- §2 -- lua <-> Vimscript
-- ---------------------------------------------------------------------

-- Self-reference has no image in Lua: the converters recurse until the
-- Lua stack refuses to grow ("E1502: Lua failed to grow stack"), and the
-- resulting error cannot even be formatted without more stack, so it
-- escapes the per-case pcall and truncates the section.  §1 covers these
-- two values (string() renders them, json_encode() rejects them, and
-- that is the copyID assertion); §2 records them as skipped.
local LUA_SKIP = { rec_list = true, rec_dict = true }

section('s2-tv2lua', function()
  -- Four paths, one corpus.  vim.fn.eval() is nlua_push_typval;
  -- nvim_eval() is vim_to_object followed by object_to_lua; and the two
  -- disagreeing is itself a recorded answer.
  for _, name in ipairs(NAMES) do
    local var = 'g:C_' .. name
    if LUA_SKIP[name] then
      emit('tv2lua ' .. name, '# SKIPPED (self-reference)')
    else
      attempt('tv2lua ' .. name, vim.fn.eval, var)
      attempt('tv2obj ' .. name, vim.api.nvim_eval, var)
    end
  end
end)

section('s2-roundtrip', function()
  -- ... and back.  A value that survives typval -> lua -> typval must
  -- render identically; where it does not, the difference is the
  -- answer.  string() is the renderer on both ends so the comparison is
  -- of the *values*, not of two encoders.
  for _, name in ipairs(NAMES) do
    local var = 'g:C_' .. name
    if LUA_SKIP[name] then
      emit('rt2 ' .. name, '# SKIPPED (self-reference)')
      goto continue
    end
    local before = select(2, pcall(vim.fn.eval, 'string(' .. var .. ')'))
    local ok, value = pcall(vim.fn.eval, var)
    if not ok then
      emit('rt2 ' .. name, '! (typval->lua)', esc(errtext(value)))
    else
      local back = select(2, pcall(vim.fn.string, value))
      emit('rt2 ' .. name, '|', esc(scrub(before)), '->', esc(scrub(back)))
      struct('rt2 ' .. name, { before = before, after = back })
      -- and the Object path for the same trip
      local ok2, back2 = pcall(vim.api.nvim_call_function, 'string', { value })
      emit(
        'rt2obj ' .. name,
        '|',
        esc(scrub(before)),
        '->',
        ok2 and esc(scrub(back2)) or ('! ' .. esc(errtext(back2)))
      )
    end
    ::continue::
  end
end)

section('s2-lua2tv', function()
  -- The lua side of the conversion, over the shapes the converter has
  -- to disambiguate.  Every case is asked four ways: what Vimscript
  -- says it is (type()), what it renders as (string()), what it looks
  -- like after a full lua -> typval -> lua trip, and what the Object
  -- path makes of it.
  local CASES = {
    { 'nil', 'nil' },
    { 'vimnil', 'vim.NIL' },
    { 'true', 'true' },
    { 'false', 'false' },
    { 'int', '7' },
    { 'negint', '-7' },
    { 'zero', '0' },
    { 'float', '7.5' },
    { 'floatint', '7.0' },
    { 'bigint', '9007199254740993' },
    { 'maxint', '9223372036854775807' },
    { 'hugefloat', '1e308' },
    { 'inf', 'math.huge' },
    { 'ninf', '-math.huge' },
    { 'nan', '0/0' },
    { 'str', '"hello"' },
    { 'strempty', '""' },
    { 'strnul', '"a\\0b"' },
    { 'strbad', '"\\xc3\\x28"' },
    { 'strutf8', '"\\xce\\xb1\\xce\\xb2"' },
    { 'emptytable', '{}' },
    { 'emptydict', 'vim.empty_dict()' },
    { 'list', '{1, 2, 3}' },
    { 'dict', '{a = 1, b = 2}' },
    { 'mixed', '{1, 2, a = 3}' },
    { 'sparse', '{[1] = 1, [3] = 3}' },
    { 'holes', '{1, 2, nil, 4}' },
    { 'nilinside', '{1, vim.NIL, 3}' },
    { 'numkeys', '{[1] = "a", [2] = "b"}' },
    { 'strnumkeys', '{["1"] = "a", ["2"] = "b"}' },
    { 'boolkey', '{[true] = 1}' },
    { 'floatkey', '{[1.5] = 1}' },
    { 'nested', '{a = {b = {c = {1, 2}}}}' },
    { 'deep', 'loadstring("local t = 1 for _ = 1, 40 do t = {t} end return t")()' },
    { 'typeidx_arr', '{[vim.type_idx] = vim.types.array}' },
    { 'typeidx_dict', '{[vim.type_idx] = vim.types.dictionary}' },
    { 'typeidx_float', '{[vim.type_idx] = vim.types.float, [vim.val_idx] = 1.5}' },
    { 'typeidx_arrval', '{[vim.type_idx] = vim.types.array, 1, 2}' },
    { 'func', 'function() return 1 end' },
    { 'listoffunc', '{function() return 1 end}' },
    { 'metatable', 'setmetatable({}, {__index = function() return 1 end})' },
    { 'blobish', 'vim.fn.eval("0z00112233")' },
    { 'bigtable', 'loadstring("local t = {} for i = 1, 100 do t[i] = i end return t")()' },
  }
  for _, case in ipairs(CASES) do
    local name, src = case[1], case[2]
    local chunk = assert(loadstring or load)('return ' .. src)
    local built, value = pcall(chunk)
    if not built then
      emit('lua2tv ' .. name, '! BUILD', esc(errtext(value)))
    else
      answer('lua2tv ' .. name .. ' in', value)
      attempt('lua2tv ' .. name .. ' type', vim.fn.type, value)
      attempt('lua2tv ' .. name .. ' string', vim.fn.string, value)
      attempt('lua2tv ' .. name .. ' back', vim.fn.copy, value)
      attempt(
        'lua2tv ' .. name .. ' obj',
        vim.api.nvim_call_function,
        'string',
        { value }
      )
      attempt('lua2tv ' .. name .. ' objback', vim.api.nvim_call_function, 'copy', { value })
    end
  end
end)

section('s2-luaeval', function()
  -- The Vimscript-facing half: luaeval() and v:lua, which reach the
  -- converters through a different entry point than vim.fn does.
  -- `nvim_exec_lua` is deliberately absent from the `vim.api` table, so
  -- the globals v:lua reaches are defined here directly.
  function _G.EvalsweepEcho(...)
    return { n = select('#', ...), args = { ... } }
  end
  function _G.EvalsweepId(v)
    return v
  end
  _G.EvalsweepTable = { nested = { fn = function(v)
    return { v }
  end } }
  local EXPRS = {
    'nil',
    'true',
    '1',
    '1.0',
    '"s"',
    '{}',
    '{1,2}',
    '{a=1}',
    'vim.NIL',
    'vim.empty_dict()',
    'vim.types',
    'vim.type_idx',
    '{[vim.type_idx]=vim.types.dictionary}',
    'select(2, pcall(error, "boom"))',
    'setmetatable({}, {__tostring = function() return "x" end})',
    'coroutine.create(function() end)',
    'print',
    '0/0',
    'math.huge',
    'string.rep("a", 5)',
  }
  for index, expr in ipairs(EXPRS) do
    local label = string.format('luaeval %02d %s', index, esc(expr))
    veval(label, 'string(luaeval(' .. vim.fn.string(expr) .. '))')
    veval(label .. ' type', 'type(luaeval(' .. vim.fn.string(expr) .. '))')
  end
  -- luaeval's `_A` argument is the lua <- typval direction again.
  for _, name in ipairs(NAMES) do
    if LUA_SKIP[name] then
      emit('luaevalA ' .. name, '# SKIPPED (self-reference)')
    else
      veval('luaevalA ' .. name, string.format("string(luaeval('_A', g:C_%s))", name))
      veval('luaevalT ' .. name, string.format("string(luaeval('type(_A)', g:C_%s))", name))
    end
  end
  -- v:lua, both as a call and as a funcref.
  local VLUA = {
    'v:lua.EvalsweepId(1)',
    'v:lua.EvalsweepId(v:null)',
    "v:lua.EvalsweepId('a')",
    'v:lua.EvalsweepId([1, 2])',
    "v:lua.EvalsweepId({'a': 1})",
    'v:lua.EvalsweepEcho()',
    "v:lua.EvalsweepEcho(1, 'a', v:true)",
    'v:lua.EvalsweepTable.nested.fn(1)',
    'v:lua.EvalsweepMissing(1)',
    "call(v:lua.EvalsweepId, ['x'])",
    "string(v:lua.EvalsweepId)",
  }
  for index, expr in ipairs(VLUA) do
    veval(string.format('vlua %02d %s', index, esc(expr)), 'string(' .. expr .. ')')
  end
end)

section('s2-apiconv', function()
  -- api/private/converter.rs from the other side: the Object types the
  -- api hands *out*, over answers whose shapes differ.
  local CALLS = {
    { 'get_mode', 'nvim_get_mode', {} },
    { 'list_bufs', 'nvim_list_bufs', {} },
    { 'get_current_buf', 'nvim_get_current_buf', {} },
    { 'buf_get_lines', 'nvim_buf_get_lines', { 0, 0, -1, true } },
    { 'get_all_options', 'nvim_get_option_value', { 'shiftwidth', {} } },

    { 'get_hl_id', 'nvim_get_hl_id_by_name', { 'Normal' } },
    { 'strwidth', 'nvim_strwidth', { 'abc' } },
    { 'parse_expression', 'nvim_parse_expression', { '1 + 2', '', true } },
    { 'parse_expression_bad', 'nvim_parse_expression', { '1 +', '', true } },
    { 'eval_statusline', 'nvim_eval_statusline', { 'ab%=cd', { maxwidth = 10 } } },
    { 'get_context', 'nvim_get_context', { { types = { 'jumps' } } } },
    { 'exec2', 'nvim_exec2', { 'echo 1', { output = true } } },
    { 'call_dict_function', 'nvim_call_dict_function', { { f = 'tr' }, 'f', { 'a', 'a', 'b' } } },
  }
  for _, call in ipairs(CALLS) do
    local name, fname, args = call[1], call[2], call[3]
    attempt('apiconv ' .. name, vim.api[fname], unpack(args))
  end
  -- nvim_get_color_map is 707 entries; dumping it would swamp the
  -- artifact without adding a distinct answer, so only its size and a
  -- few probes are recorded.
  local colors = vim.api.nvim_get_color_map()
  answer('apiconv color_map_size', vim.tbl_count(colors))
  for _, key in ipairs({ 'Black', 'White', 'NvimDarkBlue', 'X11Gray' }) do
    answer('apiconv color ' .. key, colors[key])
  end
  -- The Object -> typval direction: hand every corpus value to an api
  -- function that converts it back into a typval.
  for _, name in ipairs(NAMES) do
    local ok, value = false, nil
    if not LUA_SKIP[name] then
      ok, value = pcall(vim.api.nvim_eval, 'g:C_' .. name)
    end
    if LUA_SKIP[name] then
      emit('obj2tv ' .. name, '# SKIPPED (self-reference)')
    elseif ok then
      attempt('obj2tv ' .. name, vim.api.nvim_call_function, 'string', { value })
      attempt('obj2tv ' .. name .. ' type', vim.api.nvim_call_function, 'type', { value })
    else
      emit('obj2tv ' .. name, '! (eval)', esc(errtext(value)))
    end
  end
end)

-- ---------------------------------------------------------------------
-- §3 -- api validation error texts
-- ---------------------------------------------------------------------

section('s3-validate', function()
  -- Every case here must FAIL: the artifact is the error text.  A case
  -- that starts succeeding is a diff, which is the point.
  --
  -- The arms api/private/validate.rs picks between are named in the
  -- comments: `sp` = the name contains a space (so the format string
  -- drops the quotes), `n` = the numeric api_err_invalid arm, `exp` =
  -- api_err_exp with and without an actual, `req` = api_err_required,
  -- `conf` = api_err_conflict, `arr` = check_string_array's two.
  local CASES = {
    -- wrong types through the generated dispatch wrappers
    { 'buf-lines-badbuf', 'nvim_buf_get_lines', { 'x', 0, -1, true } },
    { 'buf-lines-nobuf', 'nvim_buf_get_lines', { 9999, 0, -1, true } },
    { 'buf-lines-oob', 'nvim_buf_get_lines', { 0, 5, 10, true } },
    { 'buf-lines-oob-soft', 'nvim_buf_get_lines', { 0, 5, 10, false } },
    { 'buf-lines-badstrict', 'nvim_buf_get_lines', { 0, 0, -1, 'yes' } },
    { 'buf-set-lines-notstr', 'nvim_buf_set_lines', { 0, 0, -1, true, { 1 } } }, -- arr
    { 'buf-set-lines-nl', 'nvim_buf_set_lines', { 0, 0, -1, true, { 'a\nb' } } }, -- arr
    { 'buf-set-lines-notlist', 'nvim_buf_set_lines', { 0, 0, -1, true, 'a' } },
    { 'buf-set-text-oob', 'nvim_buf_set_text', { 0, 0, 99, 0, 99, { 'x' } } },
    { 'buf-get-name-bad', 'nvim_buf_get_name', { -3 } },
    { 'buf-set-name-bad', 'nvim_buf_set_name', { 0, 42 } },
    { 'buf-get-var-missing', 'nvim_buf_get_var', { 0, 'nope' } },
    { 'buf-del-var-missing', 'nvim_buf_del_var', { 0, 'nope' } },
    { 'buf-get-mark-bad', 'nvim_buf_get_mark', { 0, 'zz' } }, -- sp
    { 'buf-get-mark-unset', 'nvim_buf_get_mark', { 0, 'z' } },
    { 'buf-add-hl-badns', 'nvim_buf_add_highlight', { 0, -2, 'Normal', 0, 0, -1 } },
    { 'buf-attach-badopts', 'nvim_buf_attach', { 0, false, { nope = 1 } } },
    -- windows and tabpages
    { 'win-set-cursor-badwin', 'nvim_win_set_cursor', { 9999, { 1, 0 } } },
    { 'win-set-cursor-oob', 'nvim_win_set_cursor', { 0, { 99, 0 } } },
    { 'win-set-cursor-short', 'nvim_win_set_cursor', { 0, { 1 } } }, -- exp
    { 'win-set-cursor-str', 'nvim_win_set_cursor', { 0, { 'a', 'b' } } },
    { 'win-set-cursor-neg', 'nvim_win_set_cursor', { 0, { 1, -2 } } },
    { 'win-set-height-bad', 'nvim_win_set_height', { 0, -1 } },
    { 'win-get-var-missing', 'nvim_win_get_var', { 0, 'nope' } },
    { 'tabpage-get-var-missing', 'nvim_tabpage_get_var', { 0, 'nope' } },
    -- options
    { 'opt-get-unknown', 'nvim_get_option_value', { 'nosuchoption', {} } },
    { 'opt-get-badscope', 'nvim_get_option_value', { 'shiftwidth', { scope = 'nope' } } }, -- sp
    { 'opt-get-badwin', 'nvim_get_option_value', { 'shiftwidth', { win = 9999 } } },
    { 'opt-set-badtype', 'nvim_set_option_value', { 'shiftwidth', 'x', {} } },
    { 'opt-set-conflict', 'nvim_set_option_value', { 'shiftwidth', 2, { win = 0, buf = 0 } } }, -- conf
    { 'opt-get-info-unknown', 'nvim_get_option_info2', { 'nosuchoption', {} } },
    -- variables and vimscript surfaces
    { 'get-var-missing', 'nvim_get_var', { 'nosuchvar' } },
    { 'get-vvar-missing', 'nvim_get_vvar', { 'nosuchvvar' } },
    { 'set-vvar-ro', 'nvim_set_vvar', { 'count', 1 } },
    { 'eval-syntax', 'nvim_eval', { '1 +' } },
    { 'eval-unknown-fn', 'nvim_eval', { 'NoSuchFunction()' } },
    { 'eval-throw', 'nvim_eval', { 'execute("throw \'boom\'")' } },
    { 'call-function-missing', 'nvim_call_function', { 'NoSuchFunction', {} } },
    { 'call-function-arity', 'nvim_call_function', { 'tr', { 'a' } } },
    { 'call-function-toomany', 'nvim_call_function', { 'strlen', { 'a', 'b', 'c' } } },
    { 'call-function-badargs', 'nvim_call_function', { 'strlen', 'notalist' } },
    { 'call-dict-fn-missing', 'nvim_call_dict_function', { { a = 1 }, 'a', {} } },
    { 'exec2-bad', 'nvim_exec2', { 'nosuchcommand', {} } },
    { 'exec2-badopt', 'nvim_exec2', { 'echo 1', { nope = true } } },
    { 'eval-luaeval-syntax', 'nvim_eval', { 'luaeval("return (")' } },
    { 'eval-luaeval-error', 'nvim_eval', { 'luaeval("error(\'boom\')")' } },
    { 'command-bad', 'nvim_command', { 'nosuchcommand' } },
    { 'parse-expr-badflags', 'nvim_parse_expression', { '1', 'zzz', false } }, -- sp
    { 'parse-cmd-bad', 'nvim_parse_cmd', { '', {} } },
    { 'parse-cmd-badopts', 'nvim_parse_cmd', { 'echo', { nope = 1 } } },
    -- keymaps, commands, autocmds
    { 'set-keymap-badmode', 'nvim_set_keymap', { 'zz', 'a', 'b', {} } },
    { 'set-keymap-badopt', 'nvim_set_keymap', { 'n', 'a', 'b', { nope = true } } },
    { 'del-keymap-missing', 'nvim_del_keymap', { 'n', 'nosuchlhs' } },
    { 'get-keymap-badmode', 'nvim_get_keymap', { 42 } },
    { 'create-cmd-badname', 'nvim_create_user_command', { 'lower', 'echo', {} } },
    { 'create-cmd-badnargs', 'nvim_create_user_command', { 'Xx', 'echo', { nargs = 'z' } } },
    { 'create-cmd-badopt', 'nvim_create_user_command', { 'Xx', 'echo', { nope = 1 } } },
    { 'del-cmd-missing', 'nvim_del_user_command', { 'NoSuchCommand' } },
    { 'create-au-badevent', 'nvim_create_autocmd', { 'NoSuchEvent', {} } },
    { 'create-au-noaction', 'nvim_create_autocmd', { 'BufEnter', {} } },
    { 'create-au-conflict', 'nvim_create_autocmd', { 'BufEnter', { command = 'echo', callback = 'x' } } }, -- conf
    { 'create-au-badgroup', 'nvim_create_autocmd', { 'BufEnter', { group = 'NoSuchGroup', command = 'echo' } } },
    { 'del-augroup-missing', 'nvim_del_augroup_by_name', { 'NoSuchGroup' } },
    { 'exec-au-badevent', 'nvim_exec_autocmds', { 'NoSuchEvent', {} } },
    { 'get-au-badopts', 'nvim_get_autocmds', { { nope = 1 } } },
    { 'get-au-badevent', 'nvim_get_autocmds', { { event = 42 } } },
    -- highlights, namespaces, extmarks
    -- ns_id <= 0 reaches `assert!(ns_id > 0)` in decoration_provider.rs
    -- and aborts the process (C asserts too, but C's assert vanishes
    -- under NDEBUG and this one does not).  Recorded in the divergence
    -- docket; a sweep case that kills the harness gates nothing, so the
    -- probe uses an unallocated positive namespace instead.
    { 'set-hl-badns', 'nvim_set_hl', { 999999, 'X', {} } },
    { 'set-hl-badopt', 'nvim_set_hl', { 0, 'X', { nope = 1 } } },
    { 'set-hl-badlink', 'nvim_set_hl', { 0, 'X', { link = 42 } } },
    { 'get-hl-badopts', 'nvim_get_hl', { 0, { nope = 1 } } },
    { 'buf-set-extmark-badns', 'nvim_buf_set_extmark', { 0, 9999, 0, 0, {} } },
    { 'buf-set-extmark-oob', 'nvim_buf_set_extmark', { 0, 1, 99, 0, {} } },
    { 'buf-set-extmark-badopt', 'nvim_buf_set_extmark', { 0, 1, 0, 0, { nope = 1 } } },
    { 'buf-get-extmark-badid', 'nvim_buf_get_extmark_by_id', { 0, 1, 9999, {} } },
    -- floats and misc
    { 'open-win-badcfg', 'nvim_open_win', { 0, false, {} } }, -- req
    { 'open-win-badrelative', 'nvim_open_win', { 0, false, { relative = 'nope', width = 1, height = 1, row = 0, col = 0 } } },
    { 'open-win-conflict', 'nvim_open_win', { 0, false, { relative = 'editor', external = true, width = 1, height = 1, row = 0, col = 0 } } },
    { 'echo-badchunks', 'nvim_echo', { { 'notalist' }, false, {} } },
    { 'echo-badopts', 'nvim_echo', { {}, false, { nope = 1 } } },
    { 'input-mouse-badbutton', 'nvim_input_mouse', { 'zz', 'press', '', 0, 0, 0 } },
    { 'input-mouse-badaction', 'nvim_input_mouse', { 'left', 'zz', '', 0, 0, 0 } },
    { 'select-pum-noitem', 'nvim_select_popupmenu_item', { -2, false, false, {} } },
    { 'ui-detach', 'nvim_ui_detach', {} },
    { 'ui-attach-bad', 'nvim_ui_attach', { 0, 0, {} } },
    { 'chan-send-bad', 'nvim_chan_send', { 9999, 'x' } },
    { 'get-proc-bad', 'nvim_get_proc', { -1 } },
    { 'strwidth-notstr', 'nvim_strwidth', { 42 } },
    { 'replace-termcodes-notstr', 'nvim_replace_termcodes', { 42, true, true, true } },
    { 'notify-bad', 'nvim_notify', { 'm', -1, {} } },
    { 'set-current-buf-bad', 'nvim_set_current_buf', { 9999 } },
    { 'set-current-win-bad', 'nvim_set_current_win', { 9999 } },
    { 'set-current-tab-bad', 'nvim_set_current_tabpage', { 9999 } },
    { 'win-close-bad', 'nvim_win_close', { 9999, true } },
    { 'del-mark-bad', 'nvim_del_mark', { 'zz' } },
    { 'get-mark-bad', 'nvim_get_mark', { 'zz', {} } },
  }
  for _, case in ipairs(CASES) do
    local name, fname, args = case[1], case[2], case[3]
    local fn = vim.api[fname]
    if fn == nil then
      emit('validate ' .. name, '! NO SUCH API', fname)
    else
      local ok, res = pcall(fn, unpack(args))
      if ok then
        emit('validate ' .. name, '= OK', esc(canon(res)))
        struct('validate ' .. name, { ok = res })
      else
        emit('validate ' .. name, '!', esc(errtext(res)))
        struct('validate ' .. name, { err = errtext(res) })
      end
    end
  end
end)

section('s3-arity', function()
  -- The generated dispatch wrappers' own arity and type errors, which
  -- are not validate.rs but are the layer immediately above it and move
  -- for the same reasons.
  local CASES = {
    { 'noargs', 'nvim_strwidth', {} },
    { 'toomany', 'nvim_strwidth', { 'a', 'b' } },
    { 'nil-arg', 'nvim_strwidth', { vim.NIL } },
    { 'bool-for-str', 'nvim_strwidth', { true } },
    { 'table-for-str', 'nvim_strwidth', { {} } },
    { 'float-for-int', 'nvim_buf_get_lines', { 1.5, 0, -1, true } },
    { 'str-for-int', 'nvim_buf_line_count', { 'x' } },
    { 'list-for-dict', 'nvim_exec2', { 'echo', { 1, 2 } } },
    { 'dict-for-list', 'nvim_call_function', { 'strlen', { a = 1 } } },
    { 'nil-for-dict', 'nvim_exec2', { 'echo', vim.NIL } },
    { 'huge-int', 'nvim_buf_line_count', { 2 ^ 62 } },
    { 'neg-handle', 'nvim_buf_line_count', { -1 } },
  }
  for _, case in ipairs(CASES) do
    local name, fname, args = case[1], case[2], case[3]
    local ok, res = pcall(vim.api[fname], unpack(args))
    if ok then
      emit('arity ' .. name, '= OK', esc(canon(res)))
    else
      emit('arity ' .. name, '!', esc(errtext(res)))
    end
    struct('arity ' .. name, ok and { ok = res } or { err = errtext(res) })
  end
end)

-- ---------------------------------------------------------------------
-- §4 -- eval/deprecated.rs
--
-- The four builtins upstream keeps only for compatibility --
-- rpcstart(), rpcstop(), last_buffer_nr() and termopen() -- had no
-- differential coverage at all until B18-10.  What lives in
-- deprecated.rs is the argument checking, the argv build and the
-- dictionary bookkeeping; the spawning itself is channel.rs and is
-- deliberately NOT exercised here.  Every case that reaches
-- channel_job_start() reaches it with a program that cannot be
-- executed, or with a buffer jobstart() refuses before spawning, so no
-- case leaves a process behind and no case depends on how fast one
-- exits.
--
-- These sections run LAST and they mutate global state (buffers, the
-- current window's buffer, the channel table).  Keep them last, and
-- keep this order.  last_buffer_nr() counts buffers, so it must be
-- asked before termopen() replaces one.  And rpcstop() must come before
-- rpcstart(): an rpcstart() whose spawn fails leaves state behind that
-- makes the *exit* of a process which still owns a live job segfault in
-- libuv's stream teardown.  That is inherited behaviour, identical on
-- both paired sides and on p0-2's, so it is not this sweep's to fix --
-- but a sweep that ends in a core dump reports nothing, so the order
-- here is load-bearing.  See b18-docket.md, F-B18-4.
-- ---------------------------------------------------------------------

section('s4-rpcstop', function()
  veval('rpcstop badtype', "rpcstop('x')")
  veval('rpcstop badtype2', 'rpcstop([])')
  veval('rpcstop badtype3', 'rpcstop(v:null)')
  -- Not a job and not a channel: channel_close() fails and its error is
  -- what the caller sees.
  veval('rpcstop unknown', 'rpcstop(12345)')
  veval('rpcstop negative', 'rpcstop(-1)')
  -- The find_job() arm: a live job is stopped through jobstop(), not
  -- closed as a channel.  This section runs BEFORE s4-rpcstart, and
  -- reaps the job before it returns -- see the note above.
  local err = command("let g:__job = jobstart(['/bin/sh', '-c', 'exec sleep 60'])")
  if err then
    emit('rpcstop job', '! SETUP', esc(err))
  else
    veval('rpcstop job', 'rpcstop(g:__job)')
    command('call jobwait([g:__job], 10000)')
  end
  command('silent! unlet! g:__job')
end)

section('s4-rpcstart', function()
  -- The two type tests.  The second argument may be a list or absent,
  -- and nothing else.
  veval('rpcstart badname', 'rpcstart(1)')
  veval('rpcstart badname2', 'rpcstart(v:null)')
  veval('rpcstart badname3', 'rpcstart({})')
  veval('rpcstart badargs', "rpcstart('x', 'notalist')")
  veval('rpcstart badargs2', "rpcstart('x', 42)")
  veval('rpcstart badargs3', "rpcstart('x', {})")
  -- The "every item is a string" walk reports the *index* of the first
  -- item that is not, so ask it at three positions.
  veval('rpcstart item0', "rpcstart('x', [1])")
  veval('rpcstart item1', "rpcstart('x', ['a', {}, 'c'])")
  veval('rpcstart item2', "rpcstart('x', ['a', 'b', []])")
  -- An empty program name is refused before anything is allocated.
  veval('rpcstart empty', "rpcstart('')")
  veval('rpcstart empty args', "rpcstart('', ['a'])")
  -- ... and these reach the argv build.  The exec fails, which is the
  -- point: the vector is constructed, handed over and freed.
  veval('rpcstart noargs', "rpcstart('nvim-no-such-program-xyz')")
  veval('rpcstart emptylist', "rpcstart('nvim-no-such-program-xyz', [])")
  veval('rpcstart argv', "rpcstart('nvim-no-such-program-xyz', ['a', 'b', 'c'])")
end)

section('s4-lastbufnr', function()
  -- last_buffer_nr() is a maximum over the buffer list, which is not
  -- the same answer as bufnr("$") once the highest-numbered buffer is
  -- wiped; ask both so a rewrite that reaches for the easy one shows.
  veval('lastbufnr start', 'last_buffer_nr()')
  veval('lastbufnr start dollar', "bufnr('$')")
  command('badd zed')
  command('badd alpha')
  veval('lastbufnr added', 'last_buffer_nr()')
  veval('lastbufnr added dollar', "bufnr('$')")
  command('silent! bwipeout! alpha')
  veval('lastbufnr wiped top', 'last_buffer_nr()')
  veval('lastbufnr wiped top dollar', "bufnr('$')")
  command('silent! bwipeout! zed')
  veval('lastbufnr wiped all', 'last_buffer_nr()')
end)

section('s1-strpayload', function()
  -- The null string against the empty one, through every reader that
  -- could tell them apart: the coercions, the comparisons, the
  -- encoders, the registers, the buffer, the API.  Then text with a
  -- newline (a NUL in a register or a line) and the function names a
  -- funcref carries.
  local N = 'v:_null_string'
  local EXPRS = {
    N .. " == ''",
    N .. " is ''",
    N .. ' == ' .. N,
    N .. " !=# 'a'",
    N .. " =~ '^$'",
    'type(' .. N .. ')',
    'string(' .. N .. ')',
    'empty(' .. N .. ')',
    'len(' .. N .. ')',
    'strlen(' .. N .. ')',
    'strchars(' .. N .. ')',
    '!' .. N,
    N .. ' + 0',
    N .. " . 'x'",
    N .. ' . ' .. N,
    'json_encode(' .. N .. ')',
    'json_encode([' .. N .. ', {"k": ' .. N .. '}])',
    'msgpackdump([' .. N .. '])',
    'msgpackparse(msgpackdump([' .. N .. ']))',
    "printf('%s|', " .. N .. ')',
    "printf('%5s|%-3s|', " .. N .. ', ' .. N .. ')',
    "execute('echo ' . 'v:_null_string')",
    "execute('echon v:_null_string . \"|\"')",
    'toupper(' .. N .. ')',
    'tr(' .. N .. ", 'a', 'b')",
    'substitute(' .. N .. ", '^', 'x', '')",
    'repeat(' .. N .. ', 3)',
    'split(' .. N .. ')',
    'join([' .. N .. ", 'a', " .. N .. "], ',')",
    'sort([' .. N .. ", '', 'a', " .. N .. '])',
    'uniq([' .. N .. ", ''])",
    "index(['', 'a'], " .. N .. ')',
    "count(['', ''], " .. N .. ')',
    "has_key({'': 1}, " .. N .. ')',
    '{' .. N .. ': 1}',
    'escape(' .. N .. ", 'a')",
    'fnameescape(' .. N .. ')',
    'shellescape(' .. N .. ')',
    'trim(' .. N .. ')',
    'matchstr(' .. N .. ", '.*')",
    'stridx(' .. N .. ", '')",
    'str2nr(' .. N .. ')',
    'str2float(' .. N .. ')',
    'iconv(' .. N .. ", 'utf-8', 'latin1')",
    'fnamemodify(' .. N .. ", ':t:r')",
    'expand(' .. N .. ')',
    'copy(' .. N .. ')',
    'deepcopy([' .. N .. '])',
    'get([' .. N .. '], 0, 1)',
    'call(' .. N .. ', [])',
    'function(' .. N .. ')',
    'funcref(' .. N .. ')',
    'exists(' .. N .. ')',
    'getreg(' .. N .. ')',
    'setreg(' .. N .. ", 'x')",
    'setreg(' .. "'a', " .. N .. ')',
    "getreg('a', 1, 1)",
    "getregtype('a')",
    "setreg('b', [" .. N .. ", 'x', " .. N .. '])',
    "getreg('b', 1, 1)",
    "getreg('b')",
    "setreg('c', \"a\\nb\\u00e9\")",
    "getreg('c', 1, 1)",
    "getreg('c')",
    "getregtype('c')",
    "setreg('d', [\"a\\nb\", \"\\u4e2d\\n\"])",
    "getreg('d', 1, 1)",
    "getreg('d')",
    "[setline(1, ['x', \"a\\nb\", " .. N .. ']), getline(1, \'$\')]',
    "[setline(1, " .. N .. "), getline(1)]",
    'luaeval("_A == nil and \'nil\' or type(_A)", ' .. N .. ')',
    'luaeval("#_A", ' .. N .. ')',
    -- function names
    "string(function('EvalSweepF'))",
    "string(funcref('EvalSweepF'))",
    "string(function('EvalSweepF', [1]))",
    "string(funcref('EvalSweepF', [1], {}))",
    "get(function('EvalSweepF'), 'name')",
    "get(funcref('EvalSweepF'), 'name')",
    "get(function('EvalSweepF', [1]), 'args')",
    "function('EvalSweepF') == function('EvalSweepF')",
    "function('EvalSweepF') is function('EvalSweepF')",
    "function('EvalSweepF') == function('EvalSweepF', [])",
    "function('EvalSweepF')(" .. N .. ", 'b')",
    "call(function('EvalSweepF'), [" .. N .. '])',
    "call('EvalSweepF', [" .. N .. '])',
    'g:C_fn_script()',
    'string(g:C_fn_script)',
    "get(g:C_fn_script, 'name') =~ '^<SNR>\\d\\+_SweepS$'",
    "exists('*EvalSweepF')",
    "exists('*' . get(g:C_fn_script, 'name'))",
    "function('tr') == function('tr')",
    "string(function('tr'))",
    "function('NoSuchSweepFn')",
    "string(function('NoSuchSweepFn'))",
    "funcref('NoSuchSweepFn')",
    "string(map(['a', " .. N .. "], 'v:val . \"!\"'))",
    "string(map([" .. N .. "], {_, v -> v}))",
  }
  for index, expr in ipairs(EXPRS) do
    veval(string.format('strpayload %03d %s', index, esc(expr)), 'string(' .. expr .. ')')
  end
  -- A variable holding the null string, appended to and assigned over.
  command('let g:__ns = v:_null_string')
  veval('strpayload let', 'string(g:__ns)')
  command("let g:__ns .= 'y'")
  veval('strpayload let append', 'string(g:__ns)')
  command('let g:__ns = v:_null_string')
  command("let g:__ns ..= ''")
  veval('strpayload let append empty', 'string([g:__ns, g:__ns is v:_null_string])')
  command('silent! unlet! g:__ns')
  -- The API side: what a null string becomes as an Object and back.
  attempt('strpayload nvim_eval', vim.api.nvim_eval, N)
  attempt('strpayload nvim_eval list', vim.api.nvim_eval, '[' .. N .. ", '']")
  attempt('strpayload nvim_call_function', vim.api.nvim_call_function, 'EvalSweepF', { '', 'x' })
  attempt('strpayload vim.fn', vim.fn.EvalSweepF, 'a', '')
  attempt('strpayload nvim_get_var', vim.api.nvim_get_var, 'C_str_null')
  attempt('strpayload vim.g', function()
    return vim.g.C_str_mbnl
  end)
end)

section('s1-builtins', function()
  -- The value-and-text builtins (eval/funcs/{call,regexp,strings,
  -- container,reduce,variables,math,context,input}.rs): the arms the
  -- suites reach rarely, and the ones that run user code while they
  -- hold a container -- reduce(), the match*() family's text_cb,
  -- substitute() with a Funcref, execute() over a List that edits
  -- itself, call() with a dict.
  vim.api.nvim_exec2(
    table.concat({
      'function! BiDictF(a) dict',
      '  return self.x + a:a',
      'endfunction',
      'function! BiMutate(m)',
      '  let g:bi_s = repeat("q", 50)',
      '  call add(g:bi_l, "z")',
      '  return "<" . a:m[0] . ">"',
      'endfunction',
      'function! BiTextCb(d)',
      '  if len(g:bi_fz) > 2 | call remove(g:bi_fz, -1) | endif',
      '  return a:d.t',
      'endfunction',
      'function! BiWatch(d, k, c)',
      '  call add(g:bi_log, [a:k, a:c])',
      'endfunction',
    }, '\n'),
    {}
  )
  command('let g:bi_s = "aXbXc"')
  command('let g:bi_l = ["a", "b"]')
  command('let g:bi_rl = [1]')
  command('call add(g:bi_rl, g:bi_rl)')
  command('let g:bi_rd = {"k": 1}')
  command('let g:bi_rd.self = g:bi_rd')
  command('let g:bi_fz = [{"t": "foo"}, {"t": "fob"}, {"t": "bar"}]')
  command('let g:bi_log = []')
  command('let g:bi_wd = {}')
  command('call setline(1, ["one two", "three", "", "two two"])')
  local EXPRS = {
    -- reduce
    'reduce(0z010203, {a, b -> a + b})',
    'reduce(0z, {a, b -> a + b})',
    'reduce(0z, {a, b -> a + b}, 5)',
    'reduce(v:_null_blob, {a, b -> a + b}, 5)',
    "reduce('abc', {a, b -> a .. b})",
    "reduce('', {a, b -> a .. b})",
    "reduce('', {a, b -> a .. b}, 'x')",
    "reduce('aéb', {a, b -> a .. '[' .. b .. ']'}, '')",
    "reduce(v:_null_string, {a, b -> a .. b}, 'n')",
    'reduce([1, 2, 3], {a, b -> a + b})',
    'reduce([1, 2], {a, b -> a + b}, 10)',
    'reduce([], {a, b -> a + b})',
    'reduce(v:_null_list, {a, b -> a + b}, 1)',
    'reduce([1, 2, 3], {a, b -> add(g:bi_rl2, b)}, 0)',
    "[execute('let g:bi_rl2 = [1, 2]'), reduce(g:bi_rl2, {a, b -> add(g:bi_rl2, b)}, 0)][1]",
    "reduce([1, 2], 'BiNoSuch')",
    'reduce(123, {a, b -> a})',
    'reduce([1, 2, 3], {a, b -> a > 1 ? execute("throw 7") : a + b})',
    "reduce([1, 2], '')",
    'max([3, 1, 2])',
    'min({"a": 3, "b": -1})',
    'max(v:_null_dict)',
    'max(5)',
    "min(['x'])",
    -- the match*() family
    "matchstrpos('testing', 'ing')",
    "matchstrpos('testing', 'ing', 2)",
    "matchstrpos('testing', 'ing', 5)",
    "matchstrpos('testing', 'ing', 99)",
    "matchstrpos(['a', 'ing', 'b'], 'ing')",
    "matchstrpos(['a', 'ing', 'b'], 'x')",
    "matchstrpos(['a', 'ing', 'xing'], 'ing', 0, 2)",
    "matchstrpos('abcabc', 'b', 0, 2)",
    "matchstrpos('', '')",
    "matchstrpos(v:_null_list, 'a')",
    "matchstrpos('aébé', 'é', 2)",
    "match('testing', 't', 1, 2)",
    "match('testing', '^t', 1)",
    "match('testing', '^t', 1, 1)",
    "matchend('aaa', 'a*', 0, 3)",
    "matchend('aaa', 'a', 0, 3)",
    "match(['a', 'b', 'a'], 'a', -1)",
    "match(['a', 'b', 'a'], 'a', 1)",
    "match(['a', 'b', 'a'], 'a', 9)",
    "match('abc', '', 5)",
    "match('abc', '', 3)",
    "match('aéb', '.', 1, 2)",
    "match('abc', '\\(')",
    "match('abc', 'b', 'x')",
    "match([1, [2], {'a': 3}], '2')",
    "match([1, [2], {'a': 3}], 'a')",
    "matchlist('acd', '\\(a\\)\\?\\(b\\)\\?\\(c\\)\\?\\(.*\\)')",
    "matchlist(['x', 'yab'], 'a\\(b\\)')",
    "matchlist('abc', 'z')",
    "matchstr(['x', 'ab'], 'b')",
    "matchstr('abc', 'b', 1)",
    "matchstr('abc', 'b', -3)",
    "matchstrlist(['ab', 'cab', 9, 'abab'], 'a\\(b\\)', {'submatches': v:true})",
    "matchstrlist(['ab'], 'a', {'submatches': 1})",
    "matchstrlist(v:_null_list, 'a')",
    "matchstrlist(['ab'], '\\(')",
    "matchbufline(bufnr(), 'two', 1, '$')",
    "matchbufline(bufnr(), '\\(t\\)\\(w\\)', 1, 2, {'submatches': v:true})",
    "matchbufline(bufnr(), 'x', 0, 1)",
    "matchbufline(bufnr(), 'x', 2, 1)",
    "matchbufline(9999, 'x', 1, 1)",
    "matchbufline(bufnr(), '^$', 1, '$')",
    "matchfuzzy(['foo', 'fob', 'bar', 'oof'], 'fo')",
    "matchfuzzy(['foo bar', 'bar foo', 'fbar'], 'foo bar')",
    "matchfuzzy(['foo bar', 'bar foo', 'fbar'], 'foo bar', {'matchseq': 1})",
    "matchfuzzypos(['foo', 'fob', 'bar'], 'fo')",
    "matchfuzzy(g:bi_fz, 'fo', {'key': 't'})",
    "matchfuzzy(g:bi_fz, 'fo', {'key': ''})",
    "matchfuzzy(g:bi_fz, 'fo', {'text_cb': {d -> d.t}, 'limit': 1})",
    "matchfuzzy(g:bi_fz, 'fo', {'text_cb': 'BiTextCb'})",
    'len(g:bi_fz)',
    "matchfuzzypos(g:bi_fz[0:3], 'fo', {'key': 't'})",
    "matchfuzzy(['a'], 'a', {'limit': 'x'})",
    "matchfuzzy(['a'], 'a', {'text_cb': 7})",
    "matchfuzzy('x', 'a')",
    "matchfuzzy(['a'], 7)",
    "matchfuzzy(['a', {'t': 'a'}, 3], 'a')",
    -- substitute() and submatch()
    "substitute('abc', 'b', {m -> m[0] .. '!'}, 'g')",
    "substitute(g:bi_s, 'X', function('BiMutate'), 'g')",
    'g:bi_s',
    'g:bi_l',
    "substitute('a.b', '\\.', '\\=submatch(0) .. submatch(0)', 'g')",
    "substitute('ab', '\\(a\\)\\(b\\)', '\\=string(submatch(2, 1))', '')",
    "substitute('ab', 'a', '\\=submatch(10)', '')",
    "substitute('aaa', 'a', 'b', 'g')",
    "substitute('abc', '\\(b\\)', '[\\1]', '')",
    "substitute('abc', 'b', {m -> toupper(m[0])}, '')",
    "substitute('abc', 'b', [], '')",
    "substitute('abc', '\\(', 'x', '')",
    'submatch(1)',
    'submatch(-1)',
    "submatch('x')",
    -- call(), function(), funcref(), eval(), execute(), exists()
    "call('BiDictF', [1], {'x': 5})",
    "call(function('BiDictF'), [2], {'x': 1})",
    "call('BiDictF', [1])",
    "call('BiDictF', [1], 1)",
    "call('BiDictF', [1], v:_null_dict)",
    "call('tr', ['abc', 'a', 'b'])",
    "call(function('BiDictF', {'x': 3}), [4])",
    "call(function('BiDictF', [1], {'x': 3}), [])",
    "call('BiDictF', v:_null_list)",
    "call('', [])",
    "call(123, [])",
    "call('s:nosuch', [])",
    "call({'x': 1}, [])",
    "string(function('tr', [1]))",
    "string(function('tr', {}))",
    "function('tr', 1)",
    "function('123')",
    "function('')",
    "function('tr x')",
    "string(funcref('tr'))",
    "string(function('BiDictF', [], {'x': 1}))",
    "string(function(function('BiDictF', [1]), [2]))",
    "string(function(function('BiDictF', {'x': 1}), [2]))",
    "string(funcref(function('BiDictF', [1]), [2]))",
    "function('tr', range(21))",
    "string(function('my#auto#fn'))",
    "string(function('s:Local'))",
    "eval('1 +')",
    "eval('1 2')",
    "eval('')",
    "eval('[1,')",
    "eval('  [1, 2]  ')",
    "execute(['let g:bi_e = 1', 'echo g:bi_e'])",
    "execute(['echo 1', 'echo 2'], '')",
    "execute('echo 1', 'silent!')",
    "execute(\"echo 'x'\", 'silent')",
    "execute('echoerr 5', 'silent!')",
    'execute([])',
    'execute(v:_null_list)',
    "[execute('let g:bi_el = [\"call add(g:bi_el, \\\"echo 9\\\")\", \"echo 1\"]'), execute(g:bi_el)][1]",
    "[execute('let g:bi_eu = [\"unlet g:bi_eu\", \"echo 7\"]'), execute(g:bi_eu)][1]",
    "execute([{}, 'echo 2'])",
    "execute(['echo 3', [], 'echo 4'])",
    "execute('echo 1', [])",
    "exists('$HOME')",
    "exists('$BI_NOPE_X_Y')",
    "exists('&tw')",
    "exists('+tw')",
    "exists('&nope')",
    "exists('&tw x')",
    "exists('*tr')",
    "exists('*v:lua.print')",
    "exists('*v:lua.nosuchfn')",
    "exists(':edit')",
    "exists(':e')",
    "exists(':Nope')",
    "exists('##BufEnter')",
    "exists('##NoSuchEvent')",
    "exists('#BufEnter')",
    "exists('g:nope')",
    "exists('g:bi_l[0]')",
    "exists('g:bi_l[9]')",
    "exists('$')",
    -- strings
    "char2nr('é')",
    "char2nr('')",
    "char2nr('a', 1)",
    "char2nr('a', [])",
    "escape('a.b', '.')",
    "escape('aéb', 'é')",
    "escape(v:_null_string, v:_null_string)",
    "fnameescape('a b%#')",
    'keytrans("\\<C-a>x\\<Esc>")',
    'keytrans(7)',
    'nr2char(233)',
    'nr2char(-1)',
    'nr2char(0x10ffff)',
    'nr2char(2147483648)',
    'nr2char(0)',
    "printf('%d %s %5.2f %x %c|', 1, 'a', 1.5, 255, 65)",
    "printf('%s')",
    "printf('%d', 1, 2)",
    "printf('%1$s %1$s', 'x')",
    "printf('%*d|%-*d|', 5, 1, 3, 2)",
    "printf('%S|%5S|', 'é', 'é')",
    "printf('%b %B %o %X', 5, 6, 8, 255)",
    "printf('%e %g %.3f', 1.0e10, 0.5, 1)",
    "printf('%d', [])",
    "printf('%s', [1, {'a': 2}])",
    "printf('%2$s %1$s', 'a', 'b')",
    "printf('%2$s %s', 'a', 'b')",
    "printf('%.2s|%5.1s|', 'abc', 'éé')",
    "printf('%c', 0x100)",
    'repeat([1], 3)',
    'repeat(0z01, 3)',
    "repeat('ab', 3)",
    "repeat('ab', -1)",
    "repeat('', 5)",
    'repeat([], 2)',
    'repeat(7, 2)',
    "sha256('abc')",
    'sha256(0z00ff)',
    "shellescape(\"a'b\")",
    "shellescape('a%b!', 1)",
    "shellescape(\"a\\nb\")",
    "soundfold('hello')",
    "split('a b  c')",
    "split('a,b,,c', ',', 1)",
    "split('aXbXc', 'X*', 1)",
    "split('aXbXc', 'X*')",
    "split('abc', '')",
    "split('', ',', 1)",
    "split('aéb', '\\zs')",
    "split('abc', '^')",
    "split('a b', '\\(')",
    "split(' a b ', ' ', 1)",
    "split('abc', 'b', [])",
    "strftime('%Y-%m-%d %H', 86400 * 365 * 30)",
    "strftime('', 0)",
    "strftime('%%|%j', 0)",
    "strptime('%Y-%m-%d %H', '2000-06-02 12') - strptime('%Y-%m-%d %H', '2000-06-01 12')",
    "strptime('%Y', 'x')",
    "strptime('%Y-%m-%d', '1970-01-01')",
    -- variables.rs
    'id([]) == id([])',
    'id(g:bi_l) == id(g:bi_l)',
    'id(v:_null_list) == id(v:_null_list)',
    'type(id(1))',
    "islocked('g:bi_l')",
    "islocked('g:bi_l[0]')",
    "islocked('g:bi_nope')",
    "islocked('g:bi_l[9]')",
    "islocked('g:bi_rd.k')",
    "islocked('g:bi_rd.nope')",
    "islocked('g:bi_l[0:1]')",
    "islocked('g:bi_l x')",
    "islocked('v:true')",
    "[execute('lockvar g:bi_rd'), islocked('g:bi_rd'), islocked('g:bi_rd.k'), execute('unlockvar g:bi_rd')][1:2]",
    "dictwatcheradd(g:bi_wd, 'x', function('BiWatch'))",
    "[execute('let g:bi_wd.x = 1'), execute('let g:bi_wd.y = 2'), execute('unlet g:bi_wd.x'), g:bi_log][3]",
    "dictwatcherdel(g:bi_wd, 'x', function('BiWatch'))",
    "dictwatcherdel(g:bi_wd, 'x', function('BiWatch'))",
    "dictwatcheradd(v:_null_dict, 'x', function('BiWatch'))",
    "dictwatcherdel(v:_null_dict, 'x', function('BiWatch'))",
    "dictwatcheradd([], 'x', function('BiWatch'))",
    "dictwatcheradd({}, [], function('BiWatch'))",
    "dictwatcheradd({}, 'x', 7)",
    -- containers, math
    'string(g:bi_rl)',
    'string(g:bi_rd)',
    "execute('echo g:bi_rl g:bi_rd')",
    'string([g:bi_rl, g:bi_rl])',
    'flatten([1, [2, [3, [4]]]], 1)',
    'flatten([1, [2, [3]]], -1)',
    'flattennew([1, [2]], 0)',
    'flatten(v:_null_list)',
    'flatten([1], -2)',
    '[execute("lockvar g:bi_l"), flatten(g:bi_l)][1]',
    'flattennew(g:bi_l)',
    'execute("unlockvar g:bi_l")',
    "get(function('BiDictF', [1], {'x': 3}), 'args')",
    "get(function('BiDictF', [1], {'x': 3}), 'dict')",
    "get(function('BiDictF', [1], {'x': 3}), 'func')",
    "get(function('BiDictF', [1]), 'arity')",
    "get(function('tr'), 'arity')",
    "get(function('tr'), 'nope', 'dflt')",
    "get(0z0102, -1)",
    "get(0z0102, 5, 'd')",
    "get([1, 2], -3, 'd')",
    "get({'a': 1}, 'b', 'd')",
    "get(7, 0)",
    "index([1, '1', 1.0], 1)",
    "index(['a', 'A'], 'A', 0, 1)",
    "index(0z0102, 2, -1)",
    'indexof([1, 2, 3], {i, v -> v == 2})',
    "indexof([1, 2, 3], 'v:val == 3', {'startidx': -2})",
    '[execute("let g:bi_il = [1, 2, 3]"), indexof(g:bi_il, {i, v -> len(remove(g:bi_il, 0)) && 0})][1]',
    'indexof(0z0a0b, {i, v -> v == 11})',
    'indexof({}, {i, v -> 1})',
    "indexof([1], 'x(')",
    "len({'a': 1})",
    'len(0z0102)',
    "len(function('tr'))",
    'empty(v:_null_blob)',
    'empty(function("tr"))',
    'type(v:_null_blob)',
    'copy(g:bi_rl)[0]',
    'deepcopy(g:bi_rd).k',
    'abs(-3.5)',
    'abs(v:_null_string)',
    'and(12, 10) . or(12, 10) . xor(12, 10) . invert(0)',
    'atan2(1, 1)',
    "fmod(7, 'x')",
    'pow(2, 0.5)',
    'float2nr(1.0e100)',
    'float2nr(-1.0e100)',
    'isinf(-1.0 / 0.0)',
    'isnan(0.0 / 0.0)',
    'srand([1, 2, 3, 4])',
    "srand('x')",
    'rand(srand(42))',
    'rand([1, 2, 3, 4])',
    'rand([1, 2, 3])',
    "rand([1, 2, 'x', 4])",
    'range(5)',
    'range(2, 9, 3)',
    'range(9, 2, -3)',
    'range(1, 2, 0)',
    'range(2, 0)',
    'range(3, 1)',
    "str2float('1e3x')",
    "str2float('-inf')",
    "str2float('0x10', 16)",
    "str2float('1,5')",
    "str2float('+1.5')",
    -- context
    "ctxpush(['regs', 'jumps', 'bufs']) + ctxsize()",
    "sort(keys(ctxget()))",
    "ctxpush(['gvars']) + ctxsize()",
    'type(ctxget())',
    'ctxset({})',
    "ctxset({'gvars': 1})",
    'ctxpop()',
    'ctxpop()',
    'ctxget(5)',
    "ctxpush(['nope'])",
    'ctxpush(1)',
    'ctxsize()',
  }
  for index, expr in ipairs(EXPRS) do
    veval(string.format('builtins %03d %s', index, esc(expr)), 'string(' .. expr .. ')')
  end
  -- Typeahead the prompts read: input(), inputlist(), inputsecret(),
  -- confirm(), and inputsave()/inputrestore() around them.
  local PROMPTS = {
    { 'abc\r', "input('p> ')" },
    { '\r', "input({'prompt': 'q', 'default': 'dflt'})" },
    { 'x\r', "inputsecret('s> ', 'def')" },
    { '2\r', "inputlist(['one', 'two', 'three'])" },
    { '9\r', "inputlist(['one', 'two'])" },
    { '\r', 'inputlist(v:_null_list)' },
    { '', 'inputlist(3)' },
    { 'y', "confirm('ok?', \"&yes\\n&no\")" },
    { 'n', "confirm('ok?', \"&yes\\n&no\", 1, 'Warning')" },
    { '', '[inputsave(), inputrestore(), inputrestore()]' },
    { 'zz\r', "[inputsave(), inputrestore(), input('r> ')]" },
  }
  for index, entry in ipairs(PROMPTS) do
    vim.fn.feedkeys(entry[1], 't')
    veval(string.format('builtins prompt %02d %s', index, esc(entry[2])), 'string(' .. entry[2] .. ')')
  end
  command('silent! unlet! g:bi_s g:bi_l g:bi_rl g:bi_rd g:bi_fz g:bi_log g:bi_wd g:bi_e g:bi_el g:bi_il g:bi_rl2')
end)

section('s1-editor', function()
  -- The editor-facing builtins (eval/funcs/{register,runtime,env,region,
  -- position,search,marks,screen}.rs, eval/{buffer,window}/, pos.rs,
  -- system.rs, provider.rs): every register type through setreg()/
  -- getreg(), getregion()/getregionpos() over multibyte and tab lines,
  -- search skips that move the cursor or throw, the mark and tag lists,
  -- the screen readers, the info dictionaries with scope variables in
  -- them, the line setters on hidden buffers, prompt buffers driven
  -- through typeahead, win_execute() across a local directory,
  -- system()/systemlist() with List and buffer input, and the clipboard
  -- provider driven through g:clipboard.  An entry starting with ':' is
  -- a command, reported as `ok` or its error.
  command('cd ' .. vim.fn.fnameescape(work))
  command('silent! %bwipe!')
  command('silent! tabonly!')
  command('silent! only!')
  command('set shiftwidth=4 tabstop=8 noexpandtab cmdheight=1')
  command('resize')
  vim.api.nvim_exec2(
    table.concat({
      'function! EdSkipMove()',
      '  call cursor(1, 1)',
      '  return 0',
      'endfunction',
      'function! EdSkipCount()',
      '  let g:ed_skips += 1',
      '  return g:ed_skips % 2',
      'endfunction',
      'function! EdPromptCb(text)',
      '  call add(g:ed_prompt, a:text)',
      '  call prompt_setcallback(bufnr(), {t -> add(g:ed_prompt, "second:" .. t)})',
      'endfunction',
      'function! EdPromptInt()',
      '  call add(g:ed_prompt, "interrupt")',
      'endfunction',
      'function! EdTagFn(pat, flags, info)',
      '  return [{"name": "edtag", "filename": "one.txt", "cmd": "1"}]',
      'endfunction',
    }, '\n'),
    {}
  )
  command('let g:ed_skips = 0')
  command('let g:ed_prompt = []')
  command('let g:ed_clip = []')
  -- The sandbox PATH has nothing on it but /bin/sh, so the reader is
  -- the shell's own builtins.
  command("let g:ed_cat = 'while IFS= read -r l; do printf \"[%s]\" \"$l\"; done; printf \"<%s>\" \"$l\"'")
  command('let g:loaded_python3_provider = 0')
  command('let g:loaded_perl_provider = 0')
  command('let g:loaded_ruby_provider = 0')
  command('let g:loaded_node_provider = 0')
  command(
    "let g:clipboard = {'name': 'edclip',"
      .. " 'copy': {'+': {lines, regtype -> add(g:ed_clip, [lines, regtype])},"
      .. " '*': {lines, regtype -> add(g:ed_clip, ['star', lines, regtype])}},"
      .. " 'paste': {'+': {-> [['pasted', 'two'], 'V']}, '*': {-> ['starred']}},"
      .. " 'cache_enabled': 0}"
  )
  local LINES = "['one two three', 'f(a(b)c)d', \"\\u03b1\\u03b2\\u03b3\\u03b4 x\", \"\\tsix\\tseven\", '', 'two again']"
  local CASES = {
    ':enew!',
    ':call setline(1, ' .. LINES .. ')',
    -- registers: every type, the info dictionary, append, the specials
    "setreg('a', 'abc')",
    "[getreg('a'), getregtype('a'), getreg('a', 1, 1)]",
    "setreg('a', ['x', 'y'], 'l')",
    "[getreg('a'), getregtype('a'), getreg('a', 1, 1)]",
    "setreg('a', ['p', 'qq', ''], 'b5')",
    "[getreg('a'), getregtype('a'), getreg('a', 1, 1)]",
    "setreg('a', ['p', \"\\u03b1\\u03b2\"], 'b')",
    "getregtype('a')",
    "setreg('a', 'tail', 'a')",
    "[getreg('a', 1, 1), getregtype('a')]",
    "setreg('A', 'more')",
    "[getreg('a', 1, 1), getregtype('a')]",
    "setreg('a', \"x\\ny\\n\")",
    "[getreg('a', 1, 1), getregtype('a')]",
    "setreg('a', \"x\\ny\", 'c')",
    "[getreg('a', 1, 1), getregtype('a')]",
    "setreg('a', ['n\\nl'], 'v')",
    "getreg('a', 1, 1)",
    "setreg('a', {'regcontents': ['d', 'e'], 'regtype': 'V', 'isunnamed': v:true})",
    "getreginfo('a')",
    "getreginfo('\"')",
    "getreginfo('z')",
    "getreginfo('=')",
    "setreg('a', {'regcontents': 'single'})",
    "getreginfo('a')",
    "setreg('a', {'regtype': 'b3'})",
    "getreginfo('a')",
    "setreg('a', {'points_to': 'a'})",
    "setreg('a', {'isunnamed': v:false})",
    "getreginfo('\"')",
    "setreg('a', v:_null_list)",
    "[getreg('a', 1, 1), getregtype('a')]",
    "setreg('a', [v:_null_string, 'x'])",
    "getreg('a', 1, 1)",
    "setreg('a', [1, 2])",
    "setreg('a', [[1]])",
    "setreg('a', 0z0102)",
    "setreg('a', 'x', 'q')",
    "setreg('ab', 'x')",
    "setreg('=', '1 + 2')",
    "[getreg('='), getreg('=', 1), getregtype('=')]",
    "setreg('/', 'two')",
    "[getreg('/'), getregtype('/')]",
    "setreg('_', 'x')",
    "[getreg('_'), getregtype('_')]",
    "setreg('\"', 'unnamed')",
    "[getreg(), getreg('0'), getregtype()]",
    "getreg('%')",
    "getreg('ab')",
    'getreg(v:null)',
    "getreg('a', 2, 3)",
    "getreg('1')",
    "[reg_executing(), reg_recording(), reg_recorded()]",
    ':normal! qbiz\x1bq',
    "[reg_recorded(), getreg('b'), getregtype('b')]",
    ':normal! u',
    ':call setline(1, ' .. LINES .. ')',
    "setreg('c', 'echo 1', 'l')",
    "setreg('a', '')",
    "getreginfo('a')",
    -- the clipboard provider, through g:clipboard
    "has('clipboard')",
    "getreg('+')",
    "getregtype('+')",
    "getreg('*', 1, 1)",
    "setreg('+', ['to', 'clip'], 'V')",
    "setreg('*', 'star')",
    'g:ed_clip',
    "getreginfo('+')",
    "has('python3')",
    "py3eval('1')",
    -- getregion() / getregionpos()
    "getregion([0, 1, 1, 0], [0, 1, 3, 0])",
    "getregion([0, 1, 5, 0], [0, 2, 3, 0])",
    "getregion([0, 2, 3, 0], [0, 1, 5, 0])",
    "getregion([0, 1, 5, 0], [0, 2, 3, 0], {'type': 'V'})",
    "getregion([0, 1, 2, 0], [0, 4, 4, 0], {'type': \"\\x16\"})",
    "getregion([0, 3, 3, 0], [0, 4, 3, 0], {'type': \"\\x16\"})",
    "getregion([0, 3, 2, 0], [0, 3, 6, 0])",
    "getregion([0, 3, 2, 0], [0, 3, 6, 0], {'exclusive': v:true})",
    "getregion([0, 4, 1, 0], [0, 4, 2, 0], {'type': \"\\x16\"})",
    "getregion([0, 4, 2, 3], [0, 4, 2, 5], {'type': \"\\x16\"})",
    "getregion([0, 5, 1, 0], [0, 5, 1, 0])",
    "getregion([0, 1, 99, 0], [0, 2, 1, 0])",
    "getregion([0, 99, 1, 0], [0, 99, 1, 0])",
    "getregion([0, 1, 1, 0], [0, 1, 1, 0], {'type': 'x'})",
    "getregion([0, 1, 1, 0], [bufnr() + 99, 1, 1, 0])",
    "getregion([0, 1, 1, 0], [0, 1, 1])",
    'getregion(1, 2)',
    "getregion([0, 1, 1, 0], [0, 2, 1, 0], {'type': \"\\x162\"})",
    "getregionpos([0, 1, 5, 0], [0, 2, 3, 0])",
    "getregionpos([0, 1, 5, 0], [0, 2, 3, 0], {'type': 'V'})",
    "getregionpos([0, 1, 2, 0], [0, 4, 4, 0], {'type': \"\\x16\"})",
    "getregionpos([0, 3, 2, 0], [0, 3, 6, 0], {'exclusive': v:true})",
    "getregionpos([0, 1, 1, 0], [0, 2, 99, 0], {'eol': v:true})",
    "getregionpos([0, 4, 1, 0], [0, 4, 2, 0], {'type': \"\\x16\", 'eol': v:true})",
    "getregionpos([0, 5, 1, 0], [0, 5, 1, 0], {'eol': v:true})",
    "getregionpos([0, 99, 1, 0], [0, 99, 1, 0])",
    -- positions
    "[line('.'), line('$'), line('w0'), line('w$'), line('v')]",
    "line('.', win_getid())",
    "line('.', 9999)",
    "[col('.'), col('$'), col([1, '$']), col([2, 3]), col([9, 1]), col([1, 99])]",
    "col([1, 2, 3])",
    "[charcol('.'), charcol([3, 5]), charcol([3, '$'])]",
    "[virtcol('.'), virtcol([4, 2]), virtcol([4, 2], 1), virtcol([4, '$'])]",
    "virtcol('.', 1, win_getid())",
    "virtcol([3, 5], 0, 9999)",
    'cursor(2, 4)',
    "[getpos('.'), getcurpos(), getcursorcharpos()]",
    'cursor([3, 5, 0, 7])',
    "[getpos('.'), getcharpos('.'), getcurpos()]",
    "setpos('.', [0, 4, 3, 0])",
    "getcurpos()",
    "setpos('.', [0, 4, 3, 2, 9])",
    "getcurpos()",
    "setpos(\"'a\", [0, 1, 5, 0])",
    "[getpos(\"'a\"), line(\"'a\"), col(\"'a\")]",
    "setpos(\"'A\", [bufnr(), 2, 3, 0])",
    "getpos(\"'A\")",
    "setpos(\"'A\", [bufnr() + 99, 2, 3, 0])",
    "setpos(\"'<\", [0, 1, 1, 0])",
    "setpos(\"'>\", [0, 2, 2, 0])",
    "[getpos(\"'<\"), getpos(\"'>\")]",
    "setpos('x', [0, 1, 1, 0])",
    "setpos('.', [0, 1])",
    "setcharpos('.', [0, 3, 3, 0])",
    "[getpos('.'), getcharpos('.')]",
    "setcharpos(\"'b\", [0, 3, 9, 0])",
    "getcharpos(\"'b\")",
    'setcursorcharpos(3, 4)',
    "getcursorcharpos()",
    'setcursorcharpos([3, 2, 1])',
    "getcurpos()",
    "setcursorcharpos('x')",
    "cursor('x')",
    'cursor(99, 99)',
    "getcurpos()",
    "cursor(1, 1, 3)",
    "getcurpos()",
    '[byte2line(1), byte2line(15), byte2line(9999), byte2line(-1)]',
    '[line2byte(1), line2byte(2), line2byte(7), line2byte(99), line2byte(-1)]',
    "getcharsearch()",
    "setcharsearch({'char': 'x', 'forward': 0, 'until': 1})",
    "getcharsearch()",
    "setcharsearch({'char': \"\\u03b2\"})",
    "getcharsearch()",
    "setcharsearch(1)",
    "getpos('x')",
    "getpos(\"'z\")",
    -- searches, with skips that move the cursor, count, or throw
    'cursor(1, 1)',
    "[searchpos('two'), getcurpos()[1:2]]",
    "[searchpos('t', 'bW'), getcurpos()[1:2]]",
    "searchpos('two', 'n')",
    "searchpos('\\(one\\)\\|\\(two\\)', 'pcn')",
    "searchpos('two', 'e')",
    "searchpos('two', 'ws')",
    "getpos(\"''\")",
    "searchpos('two', 'b', 1)",
    "searchpos('two', 'W', 0, 0, {-> 1})",
    "searchpos('two', 'w', 0, 0, 'EdSkipMove()')",
    'getcurpos()[1:2]',
    "search('two', 'w', 0, 0, 'EdSkipCount()')",
    'g:ed_skips',
    "search('two', 'w', 0, 0, {-> execute('throw \"skip\"')})",
    "search('two', 'w', 0, 0, 'nosuchvar')",
    "search('two', 'x')",
    "search('nosuchpat')",
    "search('\\(')",
    "search('two', 'z')",
    "search('two', 'cnw', 2)",
    'cursor(2, 2)',
    "searchpair('(', '', ')')",
    'getcurpos()[1:2]',
    'cursor(2, 4)',
    "searchpair('(', '', ')', 'bW')",
    'getcurpos()[1:2]',
    'cursor(2, 2)',
    "searchpairpos('(', '', ')', 'n')",
    "searchpairpos('(', '', ')', 'W', 'EdSkipMove()')",
    'getcurpos()[1:2]',
    'cursor(2, 2)',
    "searchpairpos('(', '', ')', 'W', {-> col('.') == 4})",
    'cursor(2, 2)',
    "searchpairpos('(', '', ')', 'Wr')",
    'cursor(2, 2)',
    "searchpairpos('(', '', ')', 'Wm')",
    'cursor(2, 2)',
    "searchpairpos('(', '', ')', 'W', {-> execute('throw \"sp\"')})",
    'cursor(2, 2)',
    "searchpairpos('(', 'b', ')', 'W', '', 2)",
    "searchpairpos('(', '', ')', 'x')",
    "searchpairpos('(', '', ')', 'W', 'EdSkipCount()', 0, 100)",
    "searchpair('\\(', '', ')')",
    'cursor(6, 1)',
    "searchdecl('two')",
    'getcurpos()[1:2]',
    "searchdecl('two', 1)",
    'getcurpos()[1:2]',
    "searchdecl('nosuch', 0, 1)",
    "searchdecl('')",
    -- marks, jumps, changes, tags
    ':mark c',
    "map(getmarklist(bufnr()), {_, m -> [m.mark, m.pos]})",
    "map(getmarklist(), {_, m -> [m.mark, m.pos[1:]]})",
    'getmarklist(9999)',
    "getmarklist('x')",
    'type(getjumplist())',
    'getjumplist(9999)',
    "getjumplist(0, 9999)",
    'getchangelist(9999)',
    'type(getchangelist())',
    'changenr() >= 0',
    'gettagstack()',
    'gettagstack(9999)',
    "settagstack(win_getid(), {'items': [{'tagname': 'x', 'from': [0, 1, 1, 0]}]}, 'r')",
    'gettagstack()',
    "settagstack(win_getid(), {'items': [{'tagname': 'y', 'from': [0, 2, 1, 0], 'matchnr': 2}]}, 'a')",
    "settagstack(win_getid(), {'curidx': 1})",
    'gettagstack()',
    "settagstack(win_getid(), {'items': [{'tagname': 'z', 'from': [0, 3, 1, 0]}]}, 't')",
    'gettagstack()',
    "settagstack(win_getid(), {'items': 1})",
    "settagstack(win_getid(), {}, 'q')",
    'settagstack(9999, {})',
    'settagstack(win_getid(), 1)',
    "settagstack(win_getid(), {'items': []}, 'r')",
    'tagfiles()',
    "taglist('x')",
    ':set tagfunc=EdTagFn',
    "taglist('edtag')",
    ':set tagfunc=',
    -- screen readers, after a redraw
    ':redraw!',
    '[screenrow() > 0, screencol() > 0]',
    '[screenchar(1, 1), screenchars(1, 1), screenstring(1, 1)]',
    '[screenchar(3, 1), screenchars(3, 2), screenstring(3, 2)]',
    '[screenchar(999, 1), screenchars(0, 0), screenstring(999, 999)]',
    'screenattr(1, 1) == screenattr(1, 2)',
    'screenattr(999, 1)',
    "[hlexists('Normal'), hlexists('EdNoSuchHl'), hlexists(''), hlexists('Error')]",
    "hlID('EdNoSuchHl')",
    "synIDattr(hlID('Comment'), 'name')",
    "synIDattr(hlID('Comment'), 'fg', 'gui') != ''",
    "synIDattr(hlID('Comment'), 'bold')",
    "synIDattr(synIDtrans(hlID('Comment')), 'name')",
    "synIDattr(hlID('Comment'), 'nosuch')",
    'synID(1, 1, 1)',
    'synID(99, 1, 0)',
    'synconcealed(1, 1)',
    'synconcealed(99, 1)',
    'synstack(1, 1)',
    'synstack(99, 1)',
    -- the runtime questions
    "[has('nvim'), has('nvim-0.9'), has('nvim-99'), has('patch-8.1.0'), has('patch-9.9.9999')]",
    "[has('win32'), has('unix'), has('ttyin'), has('vim_starting'), has('nosuch'), has('')]",
    "has('patch123')",
    "[did_filetype(), eventhandler(), foreground(), getfontname()]",
    "[mode(), mode(1), state(), state('a'), visualmode(), wildmenumode()]",
    '[nextnonblank(1), nextnonblank(5), nextnonblank(99), prevnonblank(5), prevnonblank(0)]',
    '[pum_getpos(), pumvisible()]',
    '[shiftwidth(), shiftwidth(3)]',
    '[tabpagebuflist() == [bufnr()], tabpagebuflist(99)]',
    'windowsversion()',
    'wordcount()',
    "type(api_info())",
    "sort(keys(api_info()))",
    'getpid() > 0',
    "hostname() != ''",
    -- environment and expansion
    "has_key(environ(), 'HOME')",
    "getenv('EDSWEEP_X')",
    "setenv('EDSWEEP_X', 'v1')",
    "[getenv('EDSWEEP_X'), $EDSWEEP_X, environ()['EDSWEEP_X']]",
    "setenv('EDSWEEP_X', v:null)",
    "getenv('EDSWEEP_X')",
    "setenv('EDSWEEP_Y', 12)",
    "getenv('EDSWEEP_Y')",
    "setenv('', 'x')",
    ':file edsweep.txt',
    "[expand('%'), expand('%:t:r'), expand('<cword>'), expand('<cWORD>')]",
    "expand('#')",
    "expand('$EDSWEEP_Y/x')",
    "expand('~/x') == $HOME .. '/x'",
    "expand('*.nonesuch', 0, 1)",
    "expand('*.nonesuch', 1, 1)",
    "expand('<sfile>')",
    "expand('<abuf>')",
    "expand('<nosuch>')",
    "expand('', 0, 1)",
    "expand(v:_null_string)",
    "expandcmd('echo %')",
    "expandcmd('x <cword> %:r')",
    "expandcmd('x #9')",
    "expandcmd('x #9', {'errmsg': v:true})",
    "expandcmd('$EDSWEEP_Y')",
    "expandcmd('')",
    "writefile(['x'], 'edperm.txt')",
    "setfperm('edperm.txt', 'rw-------')",
    "getfperm('edperm.txt')",
    "setfperm('edperm.txt', 'bad')",
    "setfperm('nosuchfile', 'rw-------')",
    "delete('edperm.txt')",
    "[type(stdpath('config')), type(stdpath('data_dirs')), stdpath('nosuch')]",
    'stdpath(1)',
    "swapname(bufnr())",
    "swapname(9999)",
    "swapinfo('nosuchfile')",
    "swapinfo('edsweep.txt')",
    'type(swapfilelist())',
    -- buffer lines, the current and a hidden buffer
    "[getline(1), getline(9), getline(2, 3), getline(3, 2), getline('$')]",
    "getbufline(bufnr(), 1, '$')",
    "[getbufline(9999, 1), getbufline(bufnr(), 99), getbufoneline(bufnr(), 2), getbufoneline(bufnr(), 99)]",
    "setline(7, 'seven')",
    "setline(9, 'nine')",
    "setline(1, ['L1', 'L2'])",
    "setline('$', v:_null_list)",
    "setline(0, 'x')",
    "setline(1, [[1]])",
    "append(0, 'top')",
    "append('$', ['b1', 'b2'])",
    "append(99, 'x')",
    "append(1, v:_null_list)",
    "append(1, [{}])",
    "getline(1, '$')",
    "setbufline(bufnr(), 2, 'sb')",
    "setbufline(bufnr(), '$', ['sb1', 'sb2'])",
    "appendbufline(bufnr(), 0, 'ab0')",
    "appendbufline(bufnr(), '$', ['ab1', 'ab2'])",
    "appendbufline(bufnr(), 999, 'x')",
    "deletebufline(bufnr(), 1)",
    "deletebufline(bufnr(), 2, 3)",
    "deletebufline(bufnr(), 99)",
    "deletebufline(bufnr(), 3, 2)",
    "getline(1, '$')",
    "bufadd('edhidden')",
    "[bufexists('edhidden'), buflisted('edhidden'), bufloaded('edhidden')]",
    "setbufline('edhidden', 1, ['h1', 'h2'])",
    "[bufloaded('edhidden'), getbufline('edhidden', 1, '$')]",
    "bufload('edhidden')",
    "[bufloaded('edhidden'), getbufline('edhidden', 1, '$')]",
    "setbufline('edhidden', 1, ['h1', 'h2'])",
    "appendbufline('edhidden', 1, ['h1.5'])",
    "deletebufline('edhidden', 3)",
    "getbufline('edhidden', 1, '$')",
    "setbufline(9999, 1, 'x')",
    "appendbufline(9999, 1, 'x')",
    "deletebufline(9999, 1)",
    ':call setbufvar("edhidden", "&modifiable", 0)',
    "setbufline('edhidden', 1, 'x')",
    "appendbufline('edhidden', 1, 'x')",
    "deletebufline('edhidden', 1)",
    ':call setbufvar("edhidden", "&modifiable", 1)',
    -- buffer lookups
    "[bufnr('edhidden') > 0, bufnr('ednosuch'), bufnr('ednew', 1) > 0, bufexists('ednew')]",
    "[bufname(), bufname('edhid'), bufname(9999), bufname('%'), bufname('#')]",
    "[bufwinnr(bufnr()), bufwinid(bufnr()) == win_getid(), bufwinnr('edhidden'), bufwinid(9999)]",
    "bufnr('$') > 0",
    "[bufexists(0), buflisted(9999), bufloaded(''), bufadd('')]",
    -- the info dictionaries, with scope variables in them
    ':let b:ed_var = [1, 2]',
    ':let w:ed_var = {"k": 1}',
    ':let t:ed_var = "tab"',
    "keys(getbufinfo(bufnr())[0])",
    "getbufinfo(bufnr())[0].variables.ed_var",
    "[getbufinfo(bufnr())[0].name, getbufinfo(bufnr())[0].changed, getbufinfo(bufnr())[0].hidden]",
    "len(getbufinfo({'buflisted': 1})) > 0",
    "map(getbufinfo('edhidden'), {_, b -> [b.loaded, b.listed, b.linecount]})",
    "getbufinfo(9999)",
    "getbufinfo({'bufmodified': 1, 'bufloaded': 1})[0].bufnr == bufnr()",
    "keys(getwininfo(win_getid())[0])",
    "getwininfo(win_getid())[0].variables",
    "getwininfo(9999)",
    "len(getwininfo())",
    "keys(gettabinfo(1)[0])",
    "gettabinfo(1)[0].variables",
    'gettabinfo(99)',
    "len(gettabinfo())",
    -- windows: layout, numbering, views, switching
    '[&lines, &columns, &cmdheight, winheight(0), winnr(\'$\'), tabpagenr(\'$\')]',
    ':split',
    ':vsplit',
    "winlayout()",
    "[winnr(), winnr('$'), winnr('#'), winnr('j'), winnr('2l'), winnr('3j')]",
    "winnr('x')",
    "[tabpagenr(), tabpagenr('$'), tabpagenr('#'), tabpagewinnr(1), tabpagewinnr(1, '$')]",
    "tabpagenr('x')",
    "[win_gettype(), win_gettype(9999), getcmdwintype()]",
    "win_findbuf(bufnr())",
    "win_id2tabwin(win_getid(2))",
    "[win_id2win(win_getid(3)), win_getid(9), win_getid(1, 9)]",
    "[winbufnr(0) == bufnr(), winbufnr(99)]",
    "win_screenpos(1)",
    'win_screenpos(99)',
    "[winheight(0) > 0, winwidth(99), wincol(), winline()]",
    "winrestcmd() =~ 'resize'",
    "winsaveview()",
    "winrestview({'lnum': 2, 'col': 1, 'topline': 1, 'leftcol': 0})",
    "winsaveview()",
    'winrestview(v:_null_dict)',
    "winrestview([])",
    "win_move_separator(1, 3)",
    "win_move_separator(99, 3)",
    "win_move_statusline(1, -1)",
    "win_splitmove(1, 2)",
    "win_splitmove(1, 3, {'vertical': v:true, 'rightbelow': v:true})",
    "win_splitmove(1, 99)",
    "win_splitmove(1, 1)",
    "winlayout()",
    "[getwinpos(), getwinposx(), getwinposy()]",
    ':2wincmd w',
    ':lcd ' .. vim.fn.fnameescape(work .. '/home'),
    ':1wincmd w',
    "win_execute(win_getid(2), 'let g:ed_wcwd = fnamemodify(getcwd(0), \":t\")')",
    "[g:ed_wcwd, fnamemodify(getcwd(), ':t'), haslocaldir()]",
    "win_execute(win_getid(2), ['let g:ed_wnr = winnr()', 'let g:ed_wbuf = bufnr()'])",
    '[g:ed_wnr, winnr()]',
    "win_execute(win_getid(2), 'echo winnr()')",
    "win_execute(win_getid(2), 'echo winnr()', 'silent')",
    "win_execute(9999, 'let g:x = 1')",
    "win_execute(win_getid(2), 'throw \"inwin\"')",
    "win_execute(win_getid(2), 'lcd ..')",
    "win_execute(win_getid(2), 'let g:ed_wcwd = fnamemodify(getcwd(0), \":t\")')",
    'g:ed_wcwd',
    "win_execute(win_getid(2), 'close')",
    "win_gotoid(win_getid(2))",
    'winnr()',
    "win_gotoid(9999)",
    ':only',
    -- system() and systemlist()
    "system('echo hi')",
    "system(g:ed_cat, 'input')",
    "system(g:ed_cat, ['a', 'b', ''])",
    "system(g:ed_cat, \"a\\nb\")",
    "system(g:ed_cat, ['n\\nl'])",
    "systemlist('printf \"a\\nb\\n\\nc\"')",
    "systemlist('printf \"a\\r\\nb\"')",
    "systemlist(g:ed_cat, ['x', 'y\\nz'])",
    "systemlist('printf \"a\\n\\n\"', '', 1)",
    "systemlist('printf \"a\\n\\n\"', '', 0)",
    "system(['/bin/sh', '-c', 'read -r l; echo \"$l|$0|$1\"', 'a b', 'c'], 'in')",
    "systemlist(['/bin/sh', '-c', 'echo \"$0\"; echo \"$1\"', 'x', 'y'])",
    "[system('exit 3'), v:shell_error]",
    "[system('true'), v:shell_error]",
    "system(g:ed_cat, bufnr())",
    "systemlist(g:ed_cat, bufnr())",
    "system(g:ed_cat, 9999)",
    "system(g:ed_cat, [1, 2])",
    "system(g:ed_cat, {})",
    'system([])',
    "system('')",
    "system(['/nonexistent/edsweep'])",
    "system('printf \"a\\000b\"')",
    "systemlist('printf \"a\\000b\\nc\"')",
    "system('echo hi', '', 1)",
    "system(v:_null_list)",
    "system(g:ed_cat, v:_null_list)",
    "system(g:ed_cat, v:_null_string)",
    -- prompt buffers, driven through typeahead
    ':new',
    ':setlocal buftype=prompt',
    "prompt_setprompt(bufnr(), 'ed> ')",
    'prompt_getprompt(bufnr())',
    "prompt_setcallback(bufnr(), 'EdPromptCb')",
    "prompt_setinterrupt(bufnr(), 'EdPromptInt')",
    "[prompt_getprompt(9999), prompt_setprompt(9999, 'x')]",
    "prompt_setcallback(bufnr(), 7)",
    "getline(1, '$')",
  }
  local function run(cases, tag)
    for index, entry in ipairs(cases) do
      local label = string.format('editor %s%03d %s', tag, index, esc(entry))
      if entry:sub(1, 1) == ':' then
        local err = command(entry:sub(2))
        if err then
          emit(label, '!', esc(err))
        else
          emit(label, 'ok')
        end
      else
        veval(label, 'string(' .. entry .. ')')
      end
    end
  end
  run(CASES, '')
  -- Typeahead into the prompt buffer: the callback replaces itself on the
  -- first line, so the second line runs the replacement.
  local TYPED = {
    { 'ifirst\r', 'g:ed_prompt' },
    { 'isecond\r', 'g:ed_prompt' },
    { 'ithird\x03', 'g:ed_prompt' },
    { '', "getline(1, '$')" },
    { '', "prompt_appendbuf(bufnr(), 'appended')" },
    { '', "prompt_appendbuf(bufnr(), ['l1', 'l2'])" },
    { '', "getline(1, '$')" },
    { '', "prompt_setcallback(bufnr(), '')" },
    { 'inone\r', "getline(1, '$')" },
    { '', "prompt_setinterrupt(bufnr(), '')" },
  }
  for index, entry in ipairs(TYPED) do
    if entry[1] ~= '' then
      vim.api.nvim_feedkeys(entry[1], 'xt', false)
      command('stopinsert')
    end
    veval(string.format('editor prompt %02d %s', index, esc(entry[2])), 'string(' .. entry[2] .. ')')
  end
  command('bwipe!')
  command('cd -')
  command('silent! %bwipe!')
  command('silent! unlet! g:ed_skips g:ed_prompt g:ed_clip g:clipboard g:ed_wcwd g:ed_wnr g:ed_wbuf g:ed_cat')
  command('set cmdheight&')
end)

section('s4-termopen', function()
  -- The second argument must be a dictionary or absent.
  veval('termopen baddict', "termopen('x', 'notadict')")
  veval('termopen baddict2', "termopen('x', [])")
  veval('termopen baddict3', "termopen('x', 0)")
  veval('termopen baddict4', "termopen('x', v:null)")
  -- With a dictionary from the caller, `term` is added to *that*
  -- dictionary and stays there after the call.  A modified buffer makes
  -- jobstart() refuse before it spawns, so this arm costs no process.
  command('silent! enew!')
  command("call setline(1, 'modified')")
  command('let g:__td = {}')
  veval('termopen dict', "termopen('/bin/sh', g:__td)")
  veval('termopen dict after', 'string(g:__td)')
  command("let g:__td2 = {'detach': 1}")
  veval('termopen dict2', "termopen('/bin/sh', g:__td2)")
  veval('termopen dict2 after', 'string(g:__td2)')
  -- Without one, the dictionary is allocated and freed here.
  veval('termopen nodict', "termopen('/bin/sh')")
  veval('termopen nodict list', "termopen(['/bin/sh', '-c', 'exit 0'])")
  command('silent! unlet! g:__td')
  command('silent! unlet! g:__td2')
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
vim.api.nvim_command('set noswapfile nomore shortmess=filnxtToOF report=9999')
vim.api.nvim_command('set encoding=utf-8 fileencoding= isprint=@,161-255')
vim.api.nvim_command('set ignorecase& smartcase& maxfuncdepth=100')
vim.api.nvim_command('language C')

build_corpus()

for _, entry in ipairs(SECTIONS) do
  if not only or entry.name:match(only) then
    emit('')
    emit('== ' .. entry.name .. ' ==')
    local ok, err = pcall(entry.fn)
    if not ok then
      emit('== ' .. entry.name .. ' ABORTED:', esc(errtext(err)))
    end
  end
end

emit('')
emit('== done ==')
structfd:close()
