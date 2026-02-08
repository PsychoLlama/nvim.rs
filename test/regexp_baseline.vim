" regexp_baseline.vim - Generate expected results from the C regexp engine
"
" Usage: nvim --headless -S test/regexp_baseline.vim
"
" Reads pattern/input pairs from src/nvim-rs/test/regexp_patterns.txt
" Tests each pair with re=0 (automatic), re=1 (backtracking NFA), re=2 (NFA)
" Writes results to src/nvim-rs/test/regexp_corpus.json

let s:input_file = 'src/nvim-rs/test/regexp_patterns.txt'
let s:output_file = 'src/nvim-rs/test/regexp_corpus.json'

function! s:escape_json_string(s) abort
  let s = a:s
  let s = substitute(s, '\\', '\\\\', 'g')
  let s = substitute(s, '"', '\\"', 'g')
  let s = substitute(s, "\t", '\\t', 'g')
  let s = substitute(s, "\n", '\\n', 'g')
  let s = substitute(s, "\r", '\\r', 'g')
  let s = substitute(s, nr2char(8), '\\b', 'g')
  let s = substitute(s, nr2char(12), '\\f', 'g')
  " Escape control characters as \u00XX
  let s = substitute(s, '[\x00-\x1f]', '\=printf("\\u%04x", char2nr(submatch(0)))', 'g')
  return s
endfunction

function! s:test_pattern(pattern, input, re_val) abort
  " Returns a dict with match results for the given re engine
  let saved_re = &regexpengine
  let &regexpengine = a:re_val

  let result = {}
  let result.engine = a:re_val

  try
    let msp = matchstrpos(a:input, a:pattern)
    let result.matched = msp[0]
    let result.start = msp[1]
    let result.end = msp[2]

    " Get submatch groups via matchlist
    let ml = matchlist(a:input, a:pattern)
    let result.groups = ml
    let result.error = v:null
  catch
    let result.matched = ''
    let result.start = -1
    let result.end = -1
    let result.groups = []
    let result.error = v:exception
  endtry

  let &regexpengine = saved_re
  return result
endfunction

function! s:result_to_json(pattern, input, results) abort
  let json = '  {'
  let json .= '"pattern": "' . s:escape_json_string(a:pattern) . '"'
  let json .= ', "input": "' . s:escape_json_string(a:input) . '"'
  let json .= ', "engines": ['

  let engine_parts = []
  for r in a:results
    let part = '{'
    let part .= '"re": ' . r.engine
    if r.error isnot v:null
      let part .= ', "error": "' . s:escape_json_string(r.error) . '"'
      let part .= ', "matched": null'
      let part .= ', "start": -1'
      let part .= ', "end": -1'
      let part .= ', "groups": []'
    else
      let part .= ', "error": null'
      let part .= ', "matched": "' . s:escape_json_string(r.matched) . '"'
      let part .= ', "start": ' . r.start
      let part .= ', "end": ' . r.end
      let part .= ', "groups": ['
      let group_strs = []
      for g in r.groups
        call add(group_strs, '"' . s:escape_json_string(g) . '"')
      endfor
      let part .= join(group_strs, ', ')
      let part .= ']'
    endif
    let part .= '}'
    call add(engine_parts, part)
  endfor

  let json .= join(engine_parts, ', ')
  let json .= ']}'
  return json
endfunction

function! s:main() abort
  echomsg 'Reading patterns from ' . s:input_file

  if !filereadable(s:input_file)
    echoerr 'Cannot read ' . s:input_file
    cquit!
  endif

  let lines = readfile(s:input_file)
  let entries = []
  let skipped = 0
  let errors = 0

  for line in lines
    " Skip comments and blank lines
    if line =~# '^\s*#' || line =~# '^\s*$'
      continue
    endif

    " Split on first tab
    let parts = split(line, "\t", 1)
    let pattern = parts[0]
    let input = len(parts) > 1 ? parts[1] : ''

    " Unescape literal \n, \t, \r in the input (they are typed as two chars
    " in the patterns file but should be actual control chars for testing)
    let input = substitute(input, '\\n', "\n", 'g')
    let input = substitute(input, '\\t', "\t", 'g')
    let input = substitute(input, '\\r', "\r", 'g')

    let results = []
    for re_val in [0, 1, 2]
      call add(results, s:test_pattern(pattern, input, re_val))
    endfor

    call add(entries, s:result_to_json(pattern, input, results))
  endfor

  " Write JSON output
  let json_lines = ['[']
  for i in range(len(entries))
    let suffix = (i < len(entries) - 1) ? ',' : ''
    call add(json_lines, entries[i] . suffix)
  endfor
  call add(json_lines, ']')

  call writefile(json_lines, s:output_file)
  echomsg 'Wrote ' . len(entries) . ' entries to ' . s:output_file

  qall!
endfunction

call s:main()
