" The expression parser's oracle: WHERE the parser stops and WHAT it says.
" Run headless (exprverify.sh does); every line goes to the file named by
" $DIFFOUT.  Nothing here may depend on the build, only on behaviour.
" $EXPRSWEEP_TRACE names a file each case's label is appended to before it
" runs: the way to find the case a wedged run stopped in.
"
" Three sections:
"
"   expr   every line of exprcorpus.txt, and EVERY PREFIX of it (the
"          truncation corpus: a parser that reads a slice where the C read
"          up to a NUL disagrees first on a cut-off line), through seven
"          entries -- eval(), nvim_eval(), `:let`, `:echo` followed by `| echon`, so the
"          report shows whether the rest of the line ran, `:execute`, `:call`
"          and `:elseif` in SKIP mode (parsed, not evaluated) followed by
"          `| endif | let`.  Each answer is the value or the exception, then
"          v:errmsg.
"   cmd    every line of exprcmds.txt and every prefix of it, run through
"          execute(): lvalues (`:let l[i]`, slices, dict keys, `$ENV`,
"          `&option`, `@r`, curly-brace names), `:unlet`, `:lockvar`,
"          `:function` argument lists and defaults, heredocs, `:for`, `:if`,
"          `:try`.  `<NL>` in a line is a newline.  The answer is the
"          captured output or the exception, v:errmsg, and the fixture's
"          state after the command.
"   reent  text the evaluator is reading, changed by the expression it is
"          evaluating: a 'foldexpr' that resets itself, an `<expr>` mapping
"          that unmaps itself, a status line that clears 'statusline', a
"          `:s///` whose `\=` runs another `:s`.
"
" The fixture is rebuilt before every single evaluation, so no answer
" depends on what an earlier case did.

let s:out = []
let s:dir = expand('<sfile>:p:h')

" A cut `v:` or `g:` is the whole scope, and `v:` holds what differs from
" run to run: the binary's path, the server address, the start time.
func! S(...) abort
  let line = join(a:000, "\t")
  let line = substitute(line, '[^'' ]*/target/debug/nvim', '<NVIM>', 'g')
  let line = substitute(line, '/tmp/nvim\.[^/'' ]*/[^/'' ]*/nvim\.[0-9.]*', '<SERVER>', 'g')
  let line = substitute(line, 'starttime''*: \d\+', 'starttime: <TIME>', 'g')
  call add(s:out, line)
endfunc

" One value, printed so that a type difference shows.
func! Show(v) abort
  try
    return type(a:v) . ':' . string(a:v)
  catch
    return 'unprintable ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', '')
  endtry
endfunc

func! Exc() abort
  return 'threw ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', '')
endfunc

func! g:Id(x) abort
  return a:x
endfunc

func! s:Sf(...) abort
  return a:000
endfunc

func! Fixture() abort
  " A cut `:lockvar g:l` is `:lockvar g:`, which locks the scope and every
  " global in it.
  unlockvar g:
  " Bounded, not `!`: a cut `:let g:x = g:` leaves a cycle behind.
  for name in ['l', 'd', 'n', 'K', 's', 'b', 'x']
    if exists('g:' . name)
      execute 'unlockvar 3 g:' . name
    endif
  endfor
  for name in ['x', 'y', 'a', 'b2', 'rest', 'K', 'e', 'f2', 'i', 'v', 'k',
        \ 'w', 'c', 'h', 'lam', 'dyn', 'curly42x', 'R', 'N', 'T1', 'T2']
    if exists('g:' . name)
      execute 'unlet g:' . name
    endif
  endfor
  for f in ['g:T1', 'g:T2', 'g:T3', 'g:T4', 'g:T5', 'g:T6', 's:T7', 's:T8', 'g:t6']
    if exists('*' . f)
      execute 'delfunction ' . f
    endif
  endfor
  let g:l = [1, [2, 3], {'k': 'v'}]
  let g:d = {'a': 1, 'b': [1, 2], 'c d': 3}
  let g:n = 42
  let g:f = 1.5
  let g:s = 'héllo wörld'
  let g:b = 0z0102
  let g:F = function('len')
  let g:P = function('add', [[]])
  let g:acc = 0
  let g:curly42x = 'curly'
  let g:cury = 'cury'
  let g:obj = {'v': 'ov'}
  let g:fd = {'f': function('len'), 'p': function('add', [[]])}
  func! g:obj.method() dict abort
    return self.v
  endfunc
  let $NVIMTEST = 'env'
  let @r = 'reg'
  let @" = 'unnamed'
  set tw& sw& ts&
  let v:errmsg = ''
endfunc

" The seven entries for one expression text.
func! Entries(e) abort
  let r = []
  call Fixture()
  try
    let v = eval(a:e)
    call add(r, 'eval ' . Show(v))
  catch
    call add(r, 'eval ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  call Fixture()
  try
    let v = nvim_eval(a:e)
    call add(r, 'api ' . Show(v))
  catch
    call add(r, 'api ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  call Fixture()
  try
    execute 'let g:R = ' . a:e
    call add(r, 'let ' . Show(g:R))
  catch
    call add(r, 'let ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  call Fixture()
  try
    let o = execute('echo ' . a:e . ' | echon "<NEXT>"')
    call add(r, 'echo ' . string(o))
  catch
    call add(r, 'echo ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  " The command is a comment once the value is joined to it, so whatever
  " the expression yields is never run as a command.
  call Fixture()
  try
    execute 'execute ''"'' ' . a:e . ' | let g:N = 1'
    call add(r, 'execute next=' . exists('g:N'))
  catch
    call add(r, 'execute ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  call Fixture()
  try
    execute 'call ' . a:e
    call add(r, 'call ok')
  catch
    call add(r, 'call ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))

  " Through execute(), not `:execute`: a skipped expression that loses the
  " rest of its line (a bare `&`, upstream) takes the `| endif` with it, and
  " an `:if` left open by `:execute` swallows the lines of this driver after
  " it; execute() keeps its conditionals to itself.
  call Fixture()
  try
    call execute('if 1 | elseif ' . a:e . ' | endif | let g:N = 1')
    call add(r, 'skip next=' . exists('g:N'))
  catch
    call add(r, 'skip ' . Exc())
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))
  return r
endfunc

" One command text, through execute() -- as a List, one line per item: a
" `:function` or a heredoc reads its body from the items after it.  (A
" String holding a newline is one command line to execute(), and a
" `:function` in one waits for its body from the user.)
func! Command(c) abort
  call Fixture()
  try
    let o = execute(split(a:c, "\n", 1))
    let r = ['out ' . string(o)]
  catch
    let r = [Exc()]
  endtry
  call add(r, '  errmsg ' . string(v:errmsg))
  call add(r, '  state ' . Show([get(g:, 'l', '-'), get(g:, 'd', '-'),
        \ get(g:, 'n', '-'), $NVIMTEST, @r, &tw, &sw,
        \ get(g:, 'x', '-'), get(g:, 'h', '-'), get(g:, 'K', '-'),
        \ get(g:, 'dyn', '-'), get(g:, 'curly42x', '-'), get(g:, 'acc', '-'),
        \ islocked('g:l'), islocked('g:d'), islocked('g:n'),
        \ exists('*g:T1'), exists('*g:T2')]))
  return r
endfunc

" Every prefix, by bytes: a cut may land inside a multibyte character, which
" is exactly the case a slice bound and a NUL test answer differently.
func! Sweep(file, Fn, decode) abort
  let n = 0
  for line in readfile(s:dir . '/' . a:file)
    let n += 1
    if line =~# '^\s*\(#.*\)\=$'
      continue
    endif
    let text = a:decode ? substitute(line, '<NL>', "\n", 'g') : line
    for i in range(strlen(text) + 1)
      let p = strpart(text, 0, i)
      " A command cut inside its NAME is another command -- `:c`, `:i` and
      " `:a` read lines from the user and never return in a headless
      " process -- one cut before its arguments lists everything (`:let`
      " would print this driver's own report), and the command-name parser
      " is not this oracle's.
      if a:decode && p =~# '\%(^\||\|\n\)\s*\a*!\=\s*$'
        continue
      endif
      " A `:function` header with no body would read one from the user too.
      if a:decode && p =~# '^function' && p !~# "\n"
        let p .= "\nendfunction"
      endif
      call S(a:file . ':' . n . ':' . i, string(p))
      if $EXPRSWEEP_TRACE !=# ''
        call writefile([a:file . ':' . n . ':' . i], $EXPRSWEEP_TRACE, 'a')
      endif
      try
        let answers = a:Fn(p)
      catch
        let answers = ['DRIVER ' . Exc()]
      endtry
      for r in answers
        call S('    ' . r)
      endfor
    endfor
  endfor
endfunc

" A re-entrant case: `cmds` run in a scratch buffer, then `probe` read.
func! Reent(name, cmds, probe) abort
  call Fixture()
  enew!
  call setline(1, ['xa one', 'two xa', 'three'])
  try
    for c in a:cmds
      execute c
    endfor
    call S(a:name, 'ok')
  catch
    call S(a:name, Exc())
  endtry
  call S('    errmsg ' . string(v:errmsg))
  try
    call S('    probe ' . Show(eval(a:probe)))
  catch
    call S('    probe ' . Exc())
  endtry
  setlocal foldmethod& foldexpr& indentexpr& statusline& includeexpr& formatexpr& foldtext&
  silent! normal! zE
  set statusline& laststatus&
  silent! mapclear
  silent! mapclear!
endfunc

call S('=== expr ===')
call Sweep('exprcorpus.txt', function('Entries'), 0)
call S('=== cmd ===')
call Sweep('exprcmds.txt', function('Command'), 1)

call S('=== reent ===')
call Reent('foldexpr resets itself',
      \ ['setlocal foldmethod=expr foldexpr=execute(''setlocal\ foldexpr=1'')',
      \  'normal! zx'],
      \ '[foldlevel(1), foldlevel(3), &l:foldexpr]')
call Reent('foldexpr deletes the line it folds',
      \ ['setlocal foldmethod=expr foldexpr=execute(''2delete'')',
      \  'normal! zx'],
      \ '[getline(1, ''$''), &l:foldexpr]')
call Reent('indentexpr resets itself',
      \ ['setlocal indentexpr=execute(''setlocal\ indentexpr='')',
      \  'normal! gg=G'],
      \ '[getline(1, ''$''), &l:indentexpr]')
call Reent('expr mapping unmaps itself',
      \ ['nnoremap <expr> Q execute(''nunmap Q'') .. ''Ax<Esc>''',
      \  'call feedkeys(''Q'', ''xt'')'],
      \ '[getline(1), maparg(''Q'', ''n'')]')
call Reent('expr mapping redefines itself',
      \ ['nnoremap <expr> Q execute(''nnoremap Q Ay<Esc>'') .. ''Ax<Esc>''',
      \  'call feedkeys(''QQ'', ''xt'')'],
      \ '[getline(1), maparg(''Q'', ''n'')]')
call Reent('statusline clears statusline',
      \ ['set laststatus=2',
      \  'set statusline=%{execute(''set\ statusline='')}',
      \  'redrawstatus!'],
      \ '[&statusline, nvim_eval_statusline(''%{execute("set stl=")}x'', {}).str]')
call Reent('statusline %! resets itself',
      \ ['set laststatus=2',
      \  'set statusline=%!execute(''set\ statusline=abc'')',
      \  'redrawstatus!'],
      \ '&statusline')
call Reent('includeexpr resets itself',
      \ ['setlocal includeexpr=execute(''setlocal\ includeexpr='')..''nosuch''',
      \  'normal! gf'],
      \ '&l:includeexpr')
call Reent('formatexpr resets itself',
      \ ['setlocal formatexpr=execute(''setlocal\ formatexpr='')',
      \  'normal! gqj'],
      \ '[getline(1, ''$''), &l:formatexpr]')
call Reent('foldtext resets itself',
      \ ['setlocal foldmethod=manual foldtext=execute(''setlocal\ foldtext=x'')..''ft''',
      \  '1,2fold'],
      \ '[foldtextresult(1), foldtextresult(1), &l:foldtext]')
call Reent('backtick file name changes its own command',
      \ ['badd `=execute(''let g:x = 7'')..''bt''`'],
      \ '[g:x, bufexists(''bt'')]')
call Reent('substitute expression runs a substitute',
      \ ['%s/x/\=execute(''s#a#b#'')/'],
      \ 'getline(1, ''$'')')
call Reent('substitute expression deletes lines',
      \ ['%s/x/\=execute(''$delete'')/'],
      \ 'getline(1, ''$'')')
call Reent('substitute expression substitutes globally',
      \ ['s/x/\=execute(''%s#a#A#g'') .. submatch(0)/g'],
      \ 'getline(1, ''$'')')

call writefile(s:out, $DIFFOUT)
qall!
