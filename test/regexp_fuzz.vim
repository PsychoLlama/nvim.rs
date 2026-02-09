" test/regexp_fuzz.vim - Fuzz test for regexp engine
" Run: VIMRUNTIME=runtime ./build/bin/nvim --headless --clean -S test/regexp_fuzz.vim
"
" Generates random regexp patterns and tests them against both BT and NFA
" engines. Pattern syntax errors are expected (caught by try/catch + silent!).
" Crashes (SIGSEGV) or infinite loops terminate the process — those are bugs.

let s:meta = split('a b c 1 2 . * + ? [ ] ( ) { } ^ $ | _ \\ d w s', ' ')
let s:meta_len = len(s:meta)
let s:chars = 'abcdefghijklmnopqrstuvwxyz 0123456789ABCDEFghij.,$!@#'
let s:chars_len = len(s:chars)
let s:iterations = 10000
let s:total = 0

" --- Phase 1: Adversarial patterns ---
" Known patterns that historically cause problems in regexp engines.

let s:adversarial = [
  \ '(a+)+b',
  \ '(a*)*b',
  \ '([a-z]+)*',
  \ '\v(a{1,100}){1,100}',
  \ '.*.*.*.*.*x',
  \ '\v(.{0,50}){0,50}',
  \ '[',
  \ '\_[',
  \ '\(',
  \ '\%(',
  \ '\z(',
  \ repeat('(', 100),
  \ repeat('\%(', 50) .. repeat('\)', 50),
  \ '\v' .. repeat('(', 50) .. repeat(')', 50),
  \ repeat('[', 50),
  \ '\%#=1\v(a+)+b',
  \ '\%#=2\v(a+)+b',
  \ '\v\C[\zs',
  \ ]

let s:adversarial_input = 'aaaaaaaaaaaaaaaaaaaab'

echomsg 'Fuzz: testing adversarial patterns...'
for pat in s:adversarial
  try | silent! call matchstrpos(s:adversarial_input, pat) | catch | endtry
  let s:total += 1
endfor
echomsg printf('Fuzz: %d adversarial patterns OK', len(s:adversarial))

" --- Phase 2: Random fuzzing ---

call srand(42)

for i in range(s:iterations)
  " Generate random pattern inline (avoid function call overhead)
  let plen = (rand() % 25) + 1
  let pat = ''
  for j in range(plen)
    let pat .= s:meta[rand() % s:meta_len]
  endfor

  " Generate random input inline
  let ilen = (rand() % 40) + 1
  let input = ''
  for j in range(ilen)
    let input .= s:chars[rand() % s:chars_len]
  endfor

  " Test with default engine
  try | silent! call matchstrpos(input, pat) | catch | endtry

  " Test with \v prefix (very magic)
  try | silent! call matchstrpos(input, '\v' .. pat) | catch | endtry

  " Test with \V prefix (very nomagic)
  try | silent! call matchstrpos(input, '\V' .. pat) | catch | endtry

  " Test with BT engine
  try | silent! call matchstrpos(input, '\%#=1' .. pat) | catch | endtry

  " Test with NFA engine
  try | silent! call matchstrpos(input, '\%#=2' .. pat) | catch | endtry

  let s:total += 5

  if (i + 1) % 2000 == 0
    echomsg printf('Fuzz: %d/%d iterations', i + 1, s:iterations)
  endif
endfor

echomsg printf('Fuzz complete: %d total regexp operations, no crashes', s:total)
qall!
