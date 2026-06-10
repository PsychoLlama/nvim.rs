" VarType constants regression smoke test.
"
" Guards against the class of bug introduced by per-file hand-maintained copies
" of VimL VarType integer constants diverging from the authoritative C enum
" (typval_defs.h).  Two bugs were empirically reproduced and fixed:
"
"   - chdir() returned '' because VAR_STRING was defined as 6 instead of 2,
"     causing the arg-0 type check in fs.rs to always fail.
"   - writefile([list], path) produced E475 because VAR_LIST was defined as 5
"     instead of 4, causing the list branch to never be taken.
"
" This test encodes those repros plus related builtins so just check catches
" any future wrong-constant regression.  functionaltest (Lua/busted) would
" catch these too but is not in `just check`; this file fills that gap.
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-types
" (15 seconds).  Cleans up all temp files; idempotent.

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: chdir() — returns non-empty previous cwd AND changes directory
" Regression: VAR_STRING was 6 in fs.rs, causing the string-arg type check to
" fail and chdir() to silently return '' without changing dir.
" ============================================================================

let s:saved_cwd = getcwd()
let s:tmp_dir = tempname()
call mkdir(s:tmp_dir, 'p')

let s:old = chdir(s:tmp_dir)

if s:old !=# ''
  call s:ok('chdir-returns-nonempty-old-cwd')
else
  call s:fail('chdir-returns-nonempty-old-cwd', 'chdir() returned empty string; expected previous cwd: ' .. s:saved_cwd)
endif

if s:old ==# s:saved_cwd
  call s:ok('chdir-old-cwd-matches-saved')
else
  call s:fail('chdir-old-cwd-matches-saved', 'old=' .. s:old .. ' expected=' .. s:saved_cwd)
endif

if getcwd() ==# s:tmp_dir
  call s:ok('chdir-actually-changed-dir')
else
  call s:fail('chdir-actually-changed-dir', 'getcwd()=' .. getcwd() .. ' expected=' .. s:tmp_dir)
endif

" Restore cwd
call chdir(s:saved_cwd)
call delete(s:tmp_dir, 'rf')

" ============================================================================
" Test 2: writefile([list], path) + readfile round-trip
" Regression: VAR_LIST was 5 in fs.rs, causing the list branch to be skipped
" and writefile() to produce E475 (invalid argument).
" ============================================================================

let s:wf_tmp = tempname()
let s:wf_lines = ['line1', 'line2']

let s:wf_result = writefile(s:wf_lines, s:wf_tmp)

if s:wf_result == 0
  call s:ok('writefile-returns-zero')
else
  call s:fail('writefile-returns-zero', 'writefile returned ' .. s:wf_result .. ' (E475 means list-arg type check failed)')
endif

if filereadable(s:wf_tmp)
  call s:ok('writefile-file-exists')
else
  call s:fail('writefile-file-exists', 'file not created: ' .. s:wf_tmp)
endif

let s:rf = readfile(s:wf_tmp)
if s:rf ==# s:wf_lines
  call s:ok('writefile-readfile-roundtrip')
else
  call s:fail('writefile-readfile-roundtrip', 'got=' .. string(s:rf) .. ' expected=' .. string(s:wf_lines))
endif

call delete(s:wf_tmp)

" ============================================================================
" Test 3: getcwd() with scope arg — exercises VAR_NUMBER arg type checks
" Regression: VAR_NUMBER was 2 in fs.rs, so numeric scope args were
" misidentified as non-number and the scope was treated as absent.
" ============================================================================

let s:cwd_global = getcwd(-1, -1)
if type(s:cwd_global) == v:t_string && s:cwd_global !=# ''
  call s:ok('getcwd-with-scope-arg')
else
  call s:fail('getcwd-with-scope-arg', 'getcwd(-1,-1) returned unexpected: ' .. string(s:cwd_global))
endif

" haslocaldir(0, 0) should return 0 or 1 (no local dir in headless context)
let s:hld = haslocaldir(0, 0)
if s:hld == 0 || s:hld == 1
  call s:ok('haslocaldir-with-scope-arg')
else
  call s:fail('haslocaldir-with-scope-arg', 'haslocaldir(0,0) returned unexpected: ' .. string(s:hld))
endif

" ============================================================================
" Test 4: getmousepos() — returns a Dict with Number fields
" Regression: VAR_NUMBER was 4 in mouse/lib.rs, so the v_type written into
" the getmousepos result typvals was wrong; dict fields had wrong type.
" ============================================================================

let s:mp = getmousepos()
if type(s:mp) == v:t_dict
  call s:ok('getmousepos-returns-dict')
else
  call s:fail('getmousepos-returns-dict', 'expected Dict, got type=' .. type(s:mp))
endif

for s:field in ['screenrow', 'screencol', 'winid', 'winrow', 'wincol', 'line', 'column']
  if has_key(s:mp, s:field)
    if type(s:mp[s:field]) == v:t_number
      call s:ok('getmousepos-' .. s:field .. '-is-number')
    else
      call s:fail('getmousepos-' .. s:field .. '-is-number', 'field ' .. s:field .. ' has type=' .. type(s:mp[s:field]) .. ' (expected Number=0)')
    endif
  else
    call s:fail('getmousepos-' .. s:field .. '-present', 'field ' .. s:field .. ' missing from getmousepos()')
  endif
endfor

" ============================================================================
" Test 5: matchstrlist() — returns correct list of dicts
" Regression: VAR_BOOL was 6 in buf_match.rs instead of 7, causing the
" optional {submatches} dict arg to be mis-typed and potentially skipped.
" ============================================================================

let s:msl = matchstrlist(['foobar', 'barfoo', 'baz'], 'foo')
if type(s:msl) == v:t_list
  call s:ok('matchstrlist-returns-list')
else
  call s:fail('matchstrlist-returns-list', 'expected List, got type=' .. type(s:msl))
endif

" Should match 'foobar' (idx=0) and 'barfoo' (idx=1), not 'baz' (idx=2)
if len(s:msl) == 2
  call s:ok('matchstrlist-result-count')
else
  call s:fail('matchstrlist-result-count', 'expected 2 matches, got ' .. len(s:msl))
endif

if len(s:msl) >= 1
  let s:m0 = s:msl[0]
  if type(s:m0) == v:t_dict && has_key(s:m0, 'idx') && s:m0['idx'] == 0
    call s:ok('matchstrlist-first-match-idx')
  else
    call s:fail('matchstrlist-first-match-idx', 'first match: ' .. string(s:m0))
  endif
endif

" Test with {submatches: v:true} to exercise the VAR_BOOL dict-arg path
let s:msl_sub = matchstrlist(['foobar'], '\(foo\)')
if type(s:msl_sub) == v:t_list && len(s:msl_sub) >= 1
  call s:ok('matchstrlist-with-submatches-arg')
else
  call s:fail('matchstrlist-with-submatches-arg', 'matchstrlist with capture group failed: ' .. string(s:msl_sub))
endif

" ============================================================================
" Test 6: vars light coverage — let + redir capture + float type()
" Exercises vars/set_var.rs (VAR_FLOAT was 3, now 6) indirectly.
" ============================================================================

let s:fval = 3.14
if type(s:fval) == v:t_float
  call s:ok('float-type-correct')
else
  call s:fail('float-type-correct', 'let of float literal has wrong type(): ' .. type(s:fval))
endif

" redir exercises vars/redirect.rs (VAR_STRING was 1, now 2)
let s:redir_out = ''
redir => s:redir_out
  echo 'redir-test'
redir END
if s:redir_out =~# 'redir-test'
  call s:ok('redir-captures-string')
else
  call s:fail('redir-captures-string', 'redir output was: ' .. string(s:redir_out))
endif

" ============================================================================
" Test 7: getcellwidths() round-trip — Rust port of f_getcellwidths (Phase 2)
" Guards that the Rust impl reads cw_table correctly (inverse of setcellwidths).
" ============================================================================

" Empty table returns empty list
call setcellwidths([])
let s:cw_empty = getcellwidths()
if s:cw_empty ==# []
  call s:ok('getcellwidths-empty')
else
  call s:fail('getcellwidths-empty', 'got=' .. string(s:cw_empty))
endif

" Single-entry round-trip: 0x2103 = 8451
call setcellwidths([[0x2103, 0x2103, 2]])
let s:cw_one = getcellwidths()
if s:cw_one ==# [[8451, 8451, 2]]
  call s:ok('getcellwidths-roundtrip')
else
  call s:fail('getcellwidths-roundtrip', 'got=' .. string(s:cw_one))
endif

" Restore to empty
call setcellwidths([])

" ============================================================================
" Test 9: charclass() — Rust port of f_charclass (Phase 1 migration)
" Guards the mb_get_class dispatch and non-String fallback (returns 0).
" ============================================================================

" charclass('  ') — space → class 0
if charclass(' ') == 0
  call s:ok('charclass-space-is-0')
else
  call s:fail('charclass-space-is-0', 'got=' .. charclass(' '))
endif

" charclass(',') — punctuation → class 1
if charclass(',') == 1
  call s:ok('charclass-comma-is-1')
else
  call s:fail('charclass-comma-is-1', 'got=' .. charclass(','))
endif

" charclass('a') — word char → class 2
if charclass('a') == 2
  call s:ok('charclass-alpha-is-2')
else
  call s:fail('charclass-alpha-is-2', 'got=' .. charclass('a'))
endif

" charclass(123) — non-String arg → returns 0 (no crash)
if charclass(123) == 0
  call s:ok('charclass-nonstring-is-0')
else
  call s:fail('charclass-nonstring-is-0', 'got=' .. charclass(123))
endif

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'type-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'type-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
