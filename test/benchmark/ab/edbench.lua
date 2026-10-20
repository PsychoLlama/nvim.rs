-- Timing canary for the editing core: the operators, registers, folds, diff,
-- autocommands and marks that none of the other benches reaches.
--
--   nvim --clean --headless -l edbench.lua
--
-- Run it through edbench.sh, which does the interleaved A/B. Prints
-- "phase<TAB>ms" lines prefixed with EDBENCH.
--
-- Phase map (which code each one is the canary for); every phase builds its
-- own buffer before its clock starts, so a phase run alone measures the same
-- work as the phase run in sequence:
--
--   ctl      nothing             -- a Vimscript loop: the noise floor
--   yank     op_yank / yank_copy_line / block_prep
--                                -- `:%y`, `yG` and a 10-column blockwise yank
--                                   over 50k lines
--   put      do_put              -- `p`/`P` of a 2k-line register, `]p` and `p`
--                                   of a 200-line blockwise register
--   join     do_join             -- `:%j` over 20k lines; `J` over `//` comment
--                                   lines with 'formatoptions' j
--   shift    shift_line / op_reindent + cindent
--                                -- `:%>`/`:%<` and `gg=G` with 'cindent' over
--                                   generated C
--   fmt      internal_format / format_lines
--                                -- `gqG` of 20k words at 'textwidth' 72
--   textobj  current_word / current_block / current_par
--                                -- `diw`, `ci(` and `yap` once per line
--   fold     foldUpdate / foldmethod marker + indent, zj / zc
--                                -- markers then indent, edits between, a
--                                   `zj`/`zc` walk over the whole buffer
--   diff     ex_diffupdate / diff_buf_* / xdiff
--                                -- two 20k-line buffers in diff mode, edits
--                                   and `:diffupdate` between them
--   au       apply_autocmds_group / auto_next_pat
--                                -- 10k `User` and `BufEnter` firings over
--                                   20 patterns, one of them `nested`
--   marks    setmark / getmark / ex_marks / getpos
--                                -- `m{a-z}`, `'{a-z}`, `getpos()`, `:marks`
--
-- Every phase checks its effect after it runs and raises instead of
-- reporting when the effect is missing: a command that quietly did nothing
-- is invisible in a timing number.
--
-- The fixtures and the loops are Vimscript, one `nvim_exec2` each: Lua is
-- only the driver and the clock. A first draft built the buffers as Lua
-- tables and looped in Lua, and the Lua heap made `yank` and `marks` move 2 %
-- and 0.8 % between two cachegrind runs of one binary.
--
-- Measured cachegrind noise (two runs of one cgu1 release binary of
-- ec4efd1a85, each phase alone via EDBENCH_ONLY, setup included) is in the
-- edbench paragraph of README.md.
--
-- Env: EDBENCH_ONLY (a comma-separated subset of the phases; a name that
-- matches nothing measures the startup alone).

local ONLY = os.getenv('EDBENCH_ONLY')

local function wanted(name)
  if not ONLY then
    return true
  end
  for p in ONLY:gmatch('[^,]+') do
    if p == name then
      return true
    end
  end
  return false
end

local function exec(src)
  vim.api.nvim_exec2(src, {})
end

-- `setup` builds the phase's fixture outside the clock and `run` is timed,
-- both Vimscript; `checks` are `{expr, what}` pairs that must be true after.
local function phase(name, setup, run, checks)
  if not wanted(name) then
    return
  end
  exec('silent! %bwipeout!\nenew!')
  exec(setup)
  local t = vim.uv.hrtime()
  exec(run)
  local elapsed = (vim.uv.hrtime() - t) / 1e6
  for _, c in ipairs(checks) do
    if vim.api.nvim_eval(c[1]) == 0 then
      io.stderr:write(('edbench: %s: %s (%s)\n'):format(name, c[2], c[1]))
      os.exit(1)
    end
  end
  io.stdout:write(('EDBENCH\t%s\t%.3f\n'):format(name, elapsed))
end

exec([[
set noswapfile shada= nomore undolevels=-1 hidden
" Words with some multibyte, fixed and reproducible.
let g:ed_words = ['alpha', 'beta', 'gamma', "délta", 'epsilon', 'zeta',
      \ 'eta', "thêta", 'iota', 'kappa', 'lambda', 'mu']
function! EdText(i, n) abort
  return join(map(range(1, a:n), {_, k -> g:ed_words[(a:i * 7 + k * 3) % 12]}))
endfunction
" Lines `first`..`last` of `n` words each, each behind `prefix` (a printf
" format taking the line number, or plain text).
function! EdLines(first, last, n, prefix) abort
  let fmt = a:prefix =~ '%' ? a:prefix : escape(a:prefix, '%') . '%.0s'
  " EdText() depends on the line number modulo 12 only, so the twelve texts
  " are built once and the line is a printf and a lookup: the fixture is
  " setup, and setup is in every per-phase count.
  let texts = map(range(12), {_, i -> EdText(i, a:n)})
  return map(range(a:first, a:last), 'printf(' . string(fmt) . ', v:val) . texts[v:val % 12]')
endfunction
]])

phase(
  'ctl',
  'let g:x = 0',
  [[
for i in range(20000)
  let g:x = (g:x + i * 7) % 1000003
endfor
]],
  { { 'g:x > 0', 'the loop did not run' } }
)

phase(
  'yank',
  [[call setline(1, EdLines(1, 50000, 6, '%06d '))]],
  [[
for _ in range(3)
  silent %yank a
  silent normal! gg"byG
  silent execute "normal! gg0\<C-v>G9l\"cy"
endfor
]],
  {
    { "len(getreg('a', 1, 1)) == 50000", 'linewise yank missed lines' },
    { "len(getreg('b', 1, 1)) == 50000", 'yG missed lines' },
    {
      "len(getreg('c', 1, 1)) == 50000 && len(getreg('c', 1, 1)[0]) == 10",
      'blockwise yank is wrong',
    },
    { 'getregtype(\'c\') == "\\<C-v>10"', 'blockwise yank is not 10 wide' },
  }
)

phase(
  'put',
  [[
call setreg('a', EdLines(1, 2000, 8, ''), 'l')
call setreg('b', map(range(1, 200), {_, i -> printf('[%03d]', i)}), 'b')
call setline(1, EdLines(1, 400, 4, '    '))
]],
  [[
for i in range(1, 40)
  call cursor(i * 5, 1)
  silent normal! "ap
  silent normal! "aP
  silent normal! 4l"b]p
  silent normal! "bp
endfor
]],
  {
    { "line('$') == 400 + 40 * 4000", 'put left the wrong line count' },
    { "search('\\[001\\]', 'nw') > 0", 'blockwise put left no block' },
  }
)

phase(
  'join',
  [[
let g:ed_flat = EdLines(1, 20000, 3, '  ')
let g:ed_comments = EdLines(1, 4000, 3, '  // ')
]],
  [[
call setline(1, g:ed_flat)
%join
let g:ed_joined = line('$')
setlocal comments=s1:/*,mb:*,ex:*/,:// formatoptions=jcroql
call setline(1, g:ed_comments)
silent normal! gg2000J
silent normal! j2000J
]],
  {
    { 'g:ed_joined == 1', ':%join left more than one line' },
    { "line('$') == 2", 'J left the wrong line count' },
    { "stridx(getline(1), '//', 6) < 0", "'fo' j kept a comment leader" },
  }
)

phase(
  'shift',
  [[
let s:fn = ['static int fn%d(int a, int b)', '{', 'int r = 0;',
      \ 'for (int i = 0; i < a; i++) {', 'if (i % 3 == 0) {', 'r += b * i;',
      \ '} else {', 'r -= i;', '}', '}', 'switch (r) {', 'case 0:', 'return 1;',
      \ 'default:', 'break;', '}', 'return r;', '}']
let s:src = []
for f in range(1, 80)
  call extend(s:src, [printf(s:fn[0], f)] + s:fn[1:])
endfor
call setline(1, s:src)
setlocal shiftwidth=4 expandtab cindent
]],
  [[
for _ in range(4)
  silent %>
  silent %<
endfor
silent %>
silent normal! gg=G
]],
  {
    { "getline(3) == '    int r = 0;'", 'cindent did not indent the body' },
    { "getline(6) == '            r += b * i;'", 'cindent did not indent the nest' },
  }
)

phase(
  'fmt',
  [[
" 20k words: one short line per 5, a paragraph per 500.
let g:ed_para = []
for i in range(1, 4000)
  call add(g:ed_para, EdText(i, 5))
  if i % 100 == 0
    call add(g:ed_para, '')
  endif
endfor
setlocal textwidth=72 formatoptions=tq
]],
  [[
for _ in range(4)
  silent %delete _
  call setline(1, g:ed_para)
  silent normal! gggqG
endfor
]],
  {
    { "line('$') > 1000 && line('$') < 4040", 'gq did not reflow' },
    {
      'max(map(getline(1, 50), {_, l -> strdisplaywidth(l)})) <= 72',
      'gq left a line over textwidth',
    },
  }
)

phase(
  'textobj',
  [[
let s:obj = []
for i in range(1, 3000)
  call add(s:obj, printf('call(%s, (%s), x%d)', EdText(i, 2), EdText(i + 1, 2), i))
  if i % 4 == 0
    call add(s:obj, '')
  endif
endfor
call setline(1, s:obj)
let g:ed_lines = line('$')
]],
  [[
for l in range(1, line('$'))
  call cursor(l, 1)
  if getline(l) != ''
    silent normal! diw
    silent normal! ci(z
    silent normal! yap
  endif
endfor
]],
  {
    { "line('$') == g:ed_lines", 'textobj changed the line count' },
    { "getline(1) == '(z)'", 'diw/ci( did the wrong thing' },
  }
)

phase(
  'fold',
  [[
let s:kinds = ['section %d {{{1', '  sub %d {{{2', '    body %d ', '    end }}}2%.0s']
call setline(1, map(range(1, 4000),
      \ {_, i -> printf(s:kinds[i % 4], i) . (i % 4 == 2 ? EdText(i, 3) : '')}))
setlocal shiftwidth=2
]],
  [[
setlocal foldmethod=marker foldlevel=99
for i in range(1, 100)
  call cursor(i * 37, 1)
  silent normal! Oextra line {{{3
  silent normal! jdd
endfor
silent normal! gg
for _ in range(2000)
  silent normal! zjzc
endfor
setlocal foldmethod=indent foldlevel=0
silent normal! gg
for _ in range(2000)
  silent normal! zjzozc
endfor
for i in range(1, 100)
  call cursor(i * 37, 1)
  silent normal! >>
endfor
silent normal! zR
silent normal! zM
]],
  {
    { 'foldlevel(3) > 0', 'indent folds missing' },
    { 'foldclosed(3) != -1', 'zM did not close the folds' },
  }
)

phase(
  'diff',
  [[
let s:a = EdLines(1, 20000, 5, '%05d ')
let s:b = []
for i in range(1, 20000)
  if i % 37 == 0
    call add(s:b, 'inserted ' . i)
  endif
  if i % 53 != 0
    call add(s:b, i % 41 == 0 ? 'changed ' . s:a[i - 1] : s:a[i - 1])
  endif
endfor
call setline(1, s:a)
diffthis
vnew
call setline(1, s:b)
diffthis
]],
  [[
for i in range(1, 3)
  call cursor(i * 5000, 1)
  execute 'silent normal! Oedited ' . i
  diffupdate
endfor
silent normal! gg
for _ in range(200)
  silent normal! ]c
endfor
]],
  {
    { '&diff', 'diff mode is off' },
    { "line('.') > 2000", ']c did not walk the hunks' },
  }
)
exec('silent! diffoff!')

phase(
  'au',
  [[
augroup edbench | autocmd! | augroup END
let g:ed_au = 0
let g:ed_nested = 0
for i in range(1, 20)
  execute printf('autocmd edbench User Pat%d*,Other%d let g:ed_au += 1', i, i)
  execute printf('autocmd edbench BufEnter *.e%d,*.x let g:ed_au += 1', i)
endfor
autocmd edbench User Nest nested doautocmd <nomodeline> User Pat7x
autocmd edbench User Nest let g:ed_nested += 1
silent file edbench.e7
]],
  [[
for i in range(1, 5000)
  execute 'doautocmd <nomodeline> User Pat' . (i % 20 + 1)
  doautocmd <nomodeline> BufEnter
endfor
for _ in range(1000)
  doautocmd <nomodeline> User Nest
endfor
]],
  {
    { 'g:ed_nested == 1000', 'nested autocmd did not fire' },
    -- Each User PatN fires 250 times; `Pat1*` also matches Pat10..Pat19 and
    -- `Pat2*` matches Pat20, so 250 * 31; BufEnter matches only `*.e7`
    -- (5000); every Nest fires Pat7x once more (1000).
    { 'g:ed_au == 13750', 'User/BufEnter autocmds fired the wrong number of times' },
  }
)
exec('augroup edbench | autocmd! | augroup END')

phase(
  'marks',
  [[call setline(1, EdLines(1, 5000, 4, ''))]],
  [[
let s:letters = split('abcdefghijklmnopqrstuvwxyz', '\zs')
for r in range(1, 40)
  for k in range(26)
    let c = s:letters[k]
    call cursor((r * 97 + (k + 1) * 13) % 5000 + 1, (k + 1) % 5 + 1)
    execute 'silent normal! m' . c
    execute 'silent normal! m' . toupper(c)
  endfor
  for c in s:letters
    execute "silent normal! '" . c
    execute 'silent normal! `' . toupper(c)
    call getpos("'" . c)
  endfor
  call execute('marks')
  call getmarklist(0)
endfor
]],
  {
    { 'getpos("\'z")[1] > 0', 'mark z is not set' },
    { 'getpos("\'Z")[1] > 0', 'mark Z is not set' },
  }
)
