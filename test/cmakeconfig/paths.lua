-- Static replacement for the values CMake used to interpolate into
-- paths.lua.in (kept for reference). There is no CMake build anymore: the
-- Rust port builds with cargo, straight into the repo, so the "build dir"
-- and the source dir are both the repo root. The nvim binary itself is
-- located via $NVIM_PRG (set by scripts/run-tests.sh), not through here.
local M = {}

local function repo_root()
  local source = debug.getinfo(1, 'S').source
  assert(type(source) == 'string' and vim.startswith(source, '@'), 'failed to resolve paths.lua')
  local path = assert(vim.uv.fs_realpath(source:sub(2)), 'failed to resolve paths.lua')
  -- …/test/cmakeconfig/paths.lua -> repo root
  return vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(path)))
end

local root = repo_root()

-- C header include paths for the unit tests' FFI preprocessor. Only system
-- headers still go through it (the crate's own declarations are generated
-- by tools/ffigen); the target/upstream entries below serve the remaining
-- consumers of the reconstructed v0.12.4 tree (unit-fixtures.so and
-- scripts/check-unit-cdefs.py).
local deps_prefix = os.getenv('NVIM_DEPS_PREFIX')
local upstream = root .. '/target/upstream'
M.include_paths = {} --- @type string[]
if deps_prefix then
  table.insert(M.include_paths, deps_prefix .. '/include/luajit-2.1')
  table.insert(M.include_paths, deps_prefix .. '/include')
end
table.insert(M.include_paths, upstream .. '/build/src/nvim/auto')
table.insert(M.include_paths, upstream .. '/build/include')
table.insert(M.include_paths, upstream .. '/build/cmake.config')
table.insert(M.include_paths, upstream .. '/src/src')
-- Specs cimport paths like "./src/nvim/foo.h"; relative to the extracted
-- upstream root, since the port's own src/ holds the transpiled Rust.
table.insert(M.include_paths, upstream .. '/src')
M.apple_sysroot = ''

M.translations_enabled = false
-- Exported by the `*-asan` just recipes; some specs (e.g. memory_usage_spec)
-- must skip under ASan because redzones/quarantine distort their measurements.
M.is_asan = os.getenv('NVIM_TEST_ASAN') == '1'
M.is_zig_build = false
-- Must match the VTERM_TEST_FILE define in scripts/build-unit-fixtures.sh;
-- the vterm fixture (unit-fixtures.so) writes there.
M.vterm_test_file = root .. '/target/vterm_test_output'
M.test_lua_prg = deps_prefix and (deps_prefix .. '/bin/luajit') or nil
M.test_luajit_prg = M.test_lua_prg
-- target/ is dressed up as a CMake-style build dir (runtime/, lib/nvim/) by
-- scripts/prep-test-tree.sh.
M.test_build_dir = root .. '/target'
M.test_source_path = root

return M
