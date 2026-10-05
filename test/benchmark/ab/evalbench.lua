-- Timing canary for the eval substrate (B14-4).
--
--   nvim --headless -c 'luafile evalbench.lua' -c 'qa!'
--
-- Run it through `evalbench.sh`, which does the interleaved
-- A/B the drift rule requires.  Prints "EVALBENCH<TAB>phase<TAB>ms"
-- lines so the shell can pick them out of whatever else lands on stdout.
--
-- Why this exists: B14 rewrites the interpreter's hottest data structure
-- (`typval_T` and the list/dict allocators around it) and collapses
-- ~12,700 lines -- six hand-expanded instantiations of one 132-line
-- encoder macro -- into one generic walker behind a sink trait.  Six
-- monomorphised copies of a generic walker are not the same code as six
-- macro expansions: inlining decisions, the dispatch through the trait,
-- and the extra indirection of a `&mut dyn`/`impl Sink` argument all
-- move.  `inbench` cannot see any of it: it has no eval phase at all.
--
-- Why `--headless -c` and not `-l` (inbench's reason, kept for the same
-- shape): `-l` leaves `full_screen` false, which changes the message
-- path.  Nothing here echoes, but two canaries that disagree about the
-- process's mode are two canaries nobody can compare.
--
-- Phase map (which code each phase is the canary for):
--   ctl        nothing          -- noise floor; distrust any phase whose
--                                  swing is not clearly larger than this
--   tvbuild    eval/typval.rs   -- tv_list_alloc / tv_dict_alloc and the
--                                  per-item allocators, building a deep
--                                  mixed structure from Vimscript
--   tvcopy     eval/typval.rs   -- var_item_copy: copy() (shallow) and
--                                  deepcopy() over the same structure
--   tvequal    eval/typval.rs   -- tv_equal recursing through a deep
--                                  list/dict pair that compares equal
--   tvdict     eval/typval.rs   -- tv_dict_find/_add_*/_item_remove: the
--                                  hashtab half, 20k keyed operations
--   tvlist     eval/typval.rs   -- tv_list_find/_append/_remove and
--                                  tv_list_item_sort
--   tvclear    eval/typval/     -- tv_clear's deep free: the `nothing`
--                                  encoder sink, the one instantiation
--                                  no other phase reaches
--   strconv    eval/encode.rs   -- encode_vim_to_string over the corpus
--                                  (one of the six instantiations)
--   jsonenc    eval/encode.rs   -- encode_vim_to_json + its escaper
--   jsondec    eval/decode.rs   -- json_decode_string / parse_json_*
--   msgpack    eval/encode.rs   -- msgpackdump (the msgpack sink) and
--                                  msgpackparse back
--   luapush    lua/converter.rs -- nlua_push_typval: vim.fn.eval() of a
--                                  Vimscript value into Lua
--   luapop     lua/converter.rs -- nlua_pop_typval: vim.fn.string() of a
--                                  Lua value
--   objconv    api/private/converter.rs -- vim_to_object +
--                                  object_to_lua, via nvim_eval()
--   objpop     api/private/converter.rs -- nlua_pop_Object, via
--                                  nvim_call_function()
--   funccall   eval/userfunc.rs -- call_func / call_user_func over a
--                                  plain :function
--   funcref    eval/userfunc.rs -- the same through a funcref and a
--                                  partial with bound arguments
--
-- The parser phases (added with the expression cursor's rewrite in view;
-- before them only funccall/funcref/lambda reached `eval0` at all, and
-- through a one-line body):
--   exprparse  eval/expr/        -- eval0..eval7 over long lines that
--                                  mix every operator level, literals,
--                                  subscripts, method calls, lambdas and
--                                  `$"..{}.."`, re-parsed every iteration
--   letloop    eval/lval*, vars/ -- `:let` over every lvalue shape: plain,
--                                  `+=`/`..=`, list index and slice, dict
--                                  key and `.key`, `[a, b] =`, `$ENV`,
--                                  `&option`, `@r`
--   strindex   eval/expr/index.rs -- string subscripts and slices over
--                                  multibyte text, `strcharpart`,
--                                  `charidx`/`byteidx`: the multibyte
--                                  readers the cursor is moving onto
--   skipexpr   eval/expr/        -- the same parser with evaluation off:
--                                  `:if 0` bodies, a taken `:if`'s
--                                  `:elseif`, `0 && ..`/`1 || ..`, the
--                                  `skip_expr` path of `:while`/`:for`
--
-- Groups.  EVALBENCH_GROUP picks what runs: `old` is every phase above the
-- parser ones -- the set every number recorded before them measured, so
-- an `old` Ir stays comparable with those -- `parser` is the four parser
-- phases alone, and `all` (the default) is both.  `evalbench.sh
-- --cachegrind` reports `old` and `parser` as two numbers.
--
-- Noise of the `parser` group, measured: three cachegrind runs of one
-- codegen-units = 1 release binary came out 9,673.3M-9,676.7M Ir, a spread
-- of 355 ppm (0.036 %) -- a tenth of the `old` group's 0.36 %, since these
-- phases barely touch LuaJIT.  Read a `parser` difference under ~0.05 % as
-- none.
local NROUND = tonumber(os.getenv('EVALBENCH_SCALE') or '1')
local GROUP = os.getenv('EVALBENCH_GROUP') or 'all'

-- A quiet, fixed editor.  Every option a phase can be slowed down by is
-- set explicitly: a canary whose baseline moves when a default does is
-- not a canary.
vim.cmd([==[
  silent! set nomore noruler noshowcmd noshowmode shortmess=filnxtToOF
  silent! set noswapfile nobackup nowritebackup noundofile undolevels=-1
  silent! set report=9999 maxfuncdepth=200
]==])

-- ---------------------------------------------------------------- time
local function ms(f, rounds)
  local t0 = vim.uv.hrtime()
  for _ = 1, math.max(1, math.floor(rounds * NROUND)) do
    f()
  end
  return (vim.uv.hrtime() - t0) / 1e6
end

local out = {}
local current = 'old'
local function phase(name, rounds, f)
  if GROUP ~= 'all' and GROUP ~= current then
    return
  end
  out[#out + 1] = ('EVALBENCH\t%s\t%.1f'):format(name, ms(f, rounds))
end

-- ------------------------------------------------------------ fixtures
--
-- One corpus, built once, shared by every phase, so the two sides of an
-- A/B measure the same bytes.  It has to carry the shapes the six sinks
-- disagree about -- nested containers, floats, blobs, NUL-bearing and
-- non-ASCII strings, v:null/v:true/v:false -- because a walker that is
-- fast only on flat ASCII is not the walker this batch ships.
vim.cmd([==[
  let g:leaf = {'n': 42, 'f': 1.5, 's': "a\<NL>béc", 'b': v:true,
        \       'z': v:null, 'e': '', 'blob': 0z00112233ff}
  let g:row = [1, 'two', 3.5, v:null, v:true, v:false, [], {}, 0z00, g:leaf]
  let g:deep = g:row
  for s:i in range(6)
    let g:deep = [g:deep, {'k' . s:i: g:deep, 'n': s:i}]
  endfor
  let g:wide = []
  for s:i in range(200)
    call add(g:wide, deepcopy(g:row))
  endfor
  let g:widedict = {}
  for s:i in range(2000)
    let g:widedict['key' . s:i] = s:i
  endfor
  let g:flat = range(20000)
  let g:strs = []
  for s:i in range(2000)
    call add(g:strs, 'string number ' . s:i . " with a tab\t and a quote \" in it")
  endfor

  function! Plain(a, b) abort
    return a:a + a:b
  endfunction
  function! Dicty() dict abort
    return self.n
  endfunction
  let g:Ref = function('Plain')
  let g:Part = function('Plain', [1])
  let g:PartDict = function('Dicty', {'n': 7})
]==])

local eval = vim.fn.eval
local json_encode = vim.fn.json_encode
local json_decode = vim.fn.json_decode
local msgpackdump = vim.fn.msgpackdump
local msgpackparse = vim.fn.msgpackparse
local vstring = vim.fn.string

-- Pre-rendered texts for the decode phases, so the decoders are timed
-- and the encoders are not.
local JSON = vim.fn.json_encode(vim.fn.eval('g:wide'))
local MPACK = vim.fn.msgpackdump({ vim.fn.eval('g:wide') })
local LUAVAL = vim.fn.eval('g:wide')

-- --------------------------------------------------------------- ctl
--
-- The noise floor: a Vimscript call of comparable per-iteration cost
-- that touches none of B14's twenty files.
--
-- CORRECTION (B18-5).  This comment used to end "`strwidth` is
-- charset.rs", and every batch since read the `ctl` phase against that.
-- It is wrong.  `f_strwidth` is in eval.rs and reaches
-- `mb_string2cells` in mbyte/cells.rs; the only charset.rs entry point
-- on that path is `char2cells`, taken solely on the overlong-sequence
-- arm, which an ASCII argument never enters (B18 survey Section 4).
--
-- Consequence for B18, which owns charset.rs, charset/display.rs and
-- the eval remainder: this control is NOT the floor it was read as.
-- Read TOTAL as well as `ctl`, and judge charset/cursor by the
-- typeahead and motion phases rather than by `ctl` alone.  editbench
-- (`abs(-1.5)` -> eval/funcs/math.rs) and opbench (`bufnr('%')` ->
-- buffer.rs, B20's) ARE honest floors for B18.  The question is
-- per-batch, not per-bench: re-read it every batch.
phase('ctl', 400000, function()
  local _ = vim.fn.strwidth('edit files/f01234.txt')
end)

-- ------------------------------------------------------------- typval
phase('tvbuild', 600, function()
  vim.cmd([==[
    let s:l = []
    for s:i in range(100)
      call add(s:l, {'a': s:i, 'b': [s:i, s:i + 1, [s:i]], 'c': 'x' . s:i})
    endfor
  ]==])
end)

phase('tvcopy', 400, function()
  local _ = eval('deepcopy(g:wide)')
end)

phase('tvcopy-shallow', 2000, function()
  local _ = eval('copy(g:wide)')
end)

phase('tvequal', 800, function()
  local _ = eval('g:wide == deepcopy(g:wide)')
end)

phase('tvdict', 20, function()
  vim.cmd([==[
    let s:d = {}
    for s:i in range(2000)
      let s:d['k' . s:i] = s:i
    endfor
    for s:i in range(2000)
      let s:x = get(s:d, 'k' . s:i, -1)
    endfor
    for s:i in range(2000)
      call remove(s:d, 'k' . s:i)
    endfor
  ]==])
end)

phase('tvlist', 3, function()
  vim.cmd([==[
    let s:l = copy(g:flat)
    call sort(s:l, {a, b -> b - a})
    let s:u = uniq(sort(copy(g:flat)))
    let s:x = index(s:l, 19999)
  ]==])
end)

-- `tv_clear`'s deep free -- the seventh instantiation of the encoder
-- macro, the `nothing` sink.  Nothing else here isolates it: every other
-- phase frees only as a side effect of building.
--
-- Vimscript cannot free without first building, so roughly half of this
-- phase is the allocator (which `tvbuild`/`tvcopy` already watch) and
-- half is the walk.  A regression confined to `tvclear` is therefore
-- worth about twice what the number says.
--
-- The three shapes the walk distinguishes are all here: a deep tree of
-- unshared containers (the plain descent), a structure whose children
-- are shared (`lv_refcount > 1`, the frame surgery that nulls `li` and
-- stops), and one self-referencing list.  The self-referencing one is a
-- deliberate leak -- upstream drops the last reference to a cycle and
-- leaves it for `garbagecollect()` -- but it is the same leak on both
-- sides and it is one 1-item list per round.
phase('tvclear', 900, function()
  vim.cmd([==[
    let s:c = deepcopy(g:wide)
    unlet s:c
    let s:d = deepcopy(g:deep)
    let s:s = [g:leaf, g:leaf, g:row, g:row, g:deep]
    unlet s:d
    unlet s:s
    let s:r = []
    call add(s:r, s:r)
    unlet s:r
  ]==])
end)

-- ------------------------------------------------------- encode sinks
phase('strconv', 1000, function()
  local _ = eval('string(g:wide)')
end)

phase('strconv-deep', 2000, function()
  local _ = eval('string(g:deep)')
end)

phase('jsonenc', 400, function()
  local _ = json_encode(LUAVAL)
end)

phase('jsondec', 400, function()
  local _ = json_decode(JSON)
end)

phase('msgpack', 400, function()
  local _ = msgpackdump({ LUAVAL })
end)

phase('msgpackparse', 400, function()
  local _ = msgpackparse(MPACK)
end)

-- Strings, on their own: the escaper is per-byte and is the half of the
-- sinks a container corpus barely reaches.
phase('strescape', 1200, function()
  local _ = eval('string(g:strs)')
end)

phase('jsonescape', 300, function()
  local _ = eval('json_encode(g:strs)')
end)

-- ----------------------------------------------- lua <-> vimscript
--
-- The four conversion paths, one phase each, exactly as evalsweep §2
-- names them.
phase('luapush', 1600, function()
  local _ = eval('g:wide')
end)

phase('luapop', 400, function()
  local _ = vstring(LUAVAL)
end)

phase('objconv', 1200, function()
  local _ = vim.api.nvim_eval('g:wide')
end)

phase('objpop', 400, function()
  local _ = vim.api.nvim_call_function('string', { LUAVAL })
end)

-- --------------------------------------------------------- userfunc
phase('funccall', 1200, function()
  vim.cmd('let s:t = 0 | for s:i in range(100) | let s:t += Plain(s:i, 1) | endfor')
end)

phase('funcref', 1200, function()
  vim.cmd([==[
    let s:t = 0
    for s:i in range(50)
      let s:t += g:Ref(s:i, 1) + g:Part(s:i) + g:PartDict()
    endfor
  ]==])
end)

phase('lambda', 2000, function()
  vim.cmd([==[
    let s:f = {x -> x * 2}
    let s:t = 0
    for s:i in range(100)
      let s:t += s:f(s:i)
    endfor
  ]==])
end)

-- ----------------------------------------------------------- parser
current = 'parser'

-- Their fixture is built only when they run, so an `old` run is the same
-- process it was before they existed.
if GROUP ~= 'old' then
  vim.cmd([==[
  let g:pl = [10, 20, 30, 40, 50]
  let g:pd = {'alpha': 1, 'beta': [2, 3], 'gamma': {'delta': 4}}
  let g:mb = repeat('aé日😀b', 40)
  let g:ascii = repeat('abcdefghij', 40)
  let $EVALBENCH_ENV = 'env'
]==])
end

phase('exprparse', 300, function()
  vim.cmd([==[
    for s:i in range(40)
      let s:x = (s:i + 3) * 2 - s:i / 3 % 7 + (s:i > 10 ? -1 : 1) + len(g:pl) * g:pl[s:i % 5]
      let s:y = 'abc' . s:i .. "d	e" == 'x' || s:i >= 20 && s:i <= 30 || g:pd.gamma.delta == 4
      let s:z = [s:i, s:i + 1, {'k': s:i, 'v': [g:pd['beta'][1], g:pd.alpha]}][2].v[0] + 0x1F + 0b101
      let s:w = g:pl->copy()->map({_, v -> v + s:i})->filter({_, v -> v % 20 == 0})->len()
      let s:v = $"{s:i} and {g:pd.alpha + s:i} and {g:pl[1]}" . toupper('x') . string(1.5e2)
    endfor
  ]==])
end)

phase('letloop', 500, function()
  vim.cmd([==[
    let s:l = range(10)
    let s:d = {'a': 0, 'b': 0}
    let s:s = ''
    let s:n = 0
    for s:i in range(40)
      let s:n = s:i
      let s:n += 2
      let s:l[s:i % 10] = s:i
      let s:l[2:3] = [s:i, s:i]
      let s:d.a = s:i
      let s:d['b'] += 1
      let s:d['k' . s:i % 4] = s:i
      let [s:p, s:q] = [s:i, s:n]
      let [s:p; s:rest] = [s:i, 1, 2]
      let s:s ..= s:i % 10 ? '' : 'x'
      let $EVALBENCH_ENV = 'e' . s:i % 3
      let &l:textwidth = 70 + s:i % 2
      let @r = 'r' . s:i % 3
    endfor
  ]==])
end)

phase('strindex', 500, function()
  vim.cmd([==[
    for s:i in range(40)
      let s:a = g:mb[s:i] . g:mb[s:i : s:i + 7] . g:mb[-5 :]
      let s:b = strcharpart(g:mb, s:i, 6) . slice(g:mb, s:i, s:i + 4)
      let s:c = charidx(g:mb, s:i * 2) + byteidx(g:mb, s:i) + strchars(g:mb[: s:i * 3])
      let s:e = g:ascii[s:i] . g:ascii[s:i : s:i + 9] . strpart(g:ascii, s:i, 5)
      let s:f = char2nr(g:mb[s:i * 2 :]) + strdisplaywidth(g:mb[: s:i])
    endfor
  ]==])
end)

phase('skipexpr', 450, function()
  vim.cmd([==[
    for s:i in range(40)
      if s:i < 0
        let s:x = (s:i + 3) * 2 - s:i / 3 % 7 + (s:i > 10 ? -1 : 1) + len(g:pl) * g:pl[s:i % 5]
        let s:y = g:pl->copy()->map({_, v -> v + s:i})->filter({_, v -> v % 20 == 0})->len()
        echo $"{s:i} and {g:pd.alpha + s:i}" g:mb[s:i : s:i + 7] strcharpart(g:mb, s:i, 6)
      elseif s:i >= 0 || [s:i, {'k': s:i, 'v': [g:pd['beta'][1]]}][1].v[0] == 3
        let s:z = 0 && (s:i + 3) * 2 - g:pd.gamma.delta / len(g:pl) + strchars(g:mb)
        let s:w = 1 || g:pl->copy()->map({_, v -> v + s:i})->filter({_, v -> v % 20 == 0})
      elseif s:i == 99 && g:pd.gamma.delta == 4 && toupper('x') ==# 'X' && $"{s:i}" != ''
        let s:never = 1
      endif
      while s:i < 0 && g:pd.alpha + len(g:pl) * 2 > 0
      endwhile
    endfor
  ]==])
end)

io.write(table.concat(out, '\n'), '\n')
