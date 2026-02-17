// Compare Rust regexp utility functions against C implementations.
// Compile: cc -o /tmp/compare_regexp src/nvim-rs/test/compare_regexp.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_regexp
//
// Tests stateless regexp helpers that are natural first migration targets:
//   - skip_regexp: skip past a regexp pattern to its delimiter
//
// These are the functions other crates (ex_docmd, search) already call via FFI.
// The Rust rs_skip_regexp calls into C stub functions defined below for
// standalone testing (utfc_ptr2len, nvim_get_p_cpo, etc.).

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>

// Magic values matching src/nvim/regexp_defs.h
typedef enum {
    MAGIC_NONE = 1,   // \V very unmagic
    MAGIC_OFF = 2,    // \M or 'magic' off
    MAGIC_ON = 3,     // \m or 'magic'
    MAGIC_ALL = 4,    // \v very magic
} magic_T;

static int tests_passed = 0;
static int tests_failed = 0;

#define TEST(name, condition) do { \
    if (condition) { \
        tests_passed++; \
        printf("  ✓ %s\n", name); \
    } else { \
        tests_failed++; \
        printf("  ✗ %s FAILED\n", name); \
    } \
} while(0)

// --- FFI stubs for standalone testing ---
// rs_skip_regexp_ex calls these C functions. We provide simplified
// implementations sufficient for ASCII-only test patterns.

// Returns cpoptions string — empty means no CPO_LITERAL
const char *nvim_get_p_cpo(void) { return ""; }

// ASCII-only: each byte is one character
int utfc_ptr2len(const char *p) { return (*p != '\0') ? 1 : 0; }

// Simplified get_char_class: check for [:name:] patterns
int nvim_regexp_get_char_class(char **pp) {
    char *p = *pp;
    if (p[0] == '[' && p[1] == ':') {
        // Look for closing ":]"
        char *q = p + 2;
        while (*q != '\0') {
            if (*q == ':' && q[1] == ']') {
                *pp = q + 2;
                return 0;  // Return non-CLASS_NONE (0 != 99)
            }
            q++;
        }
    }
    return 99;  // CLASS_NONE
}

// Simplified: no equivalence classes in test data
int nvim_regexp_get_equi_class(char **pp) {
    char *p = *pp;
    if (p[0] == '[' && p[1] == '=') {
        char *q = p + 2;
        while (*q != '\0') {
            if (*q == '=' && q[1] == ']') {
                *pp = q + 2;
                return 1;  // non-zero = found
            }
            q++;
        }
    }
    return 0;
}

// Simplified: no collating elements in test data
int nvim_regexp_get_coll_element(char **pp) {
    char *p = *pp;
    if (p[0] == '[' && p[1] == '.') {
        char *q = p + 2;
        while (*q != '\0') {
            if (*q == '.' && q[1] == ']') {
                *pp = q + 2;
                return 1;  // non-zero = found
            }
            q++;
        }
    }
    return 0;
}

// ASCII strchr
char *vim_strchr(const char *s, int c) {
    return strchr(s, c);
}

// Allocate and copy n bytes
char *xstrnsave(const char *s, size_t len) {
    char *r = malloc(len + 1);
    if (r) {
        memcpy(r, s, len);
        r[len] = '\0';
    }
    return r;
}

// --- Mock globals for parse state accessors ---
static char *mock_regparse = NULL;
static int mock_curchr = -1;

char *nvim_regexp_get_regparse(void) { return mock_regparse; }
void nvim_regexp_set_regparse(char *p) { mock_regparse = p; }
int nvim_regexp_get_curchr(void) { return mock_curchr; }
void nvim_regexp_set_curchr(int v) { mock_curchr = v; }
int nvim_regexp_get_prevchr_len(void) { return 0; }
void nvim_regexp_set_prevchr_len(int v) { (void)v; }
int nvim_regexp_get_prevchr(void) { return 0; }
void nvim_regexp_set_prevchr(int v) { (void)v; }
int nvim_regexp_get_prevprevchr(void) { return 0; }
void nvim_regexp_set_prevprevchr(int v) { (void)v; }
int nvim_regexp_get_nextchr(void) { return -1; }
void nvim_regexp_set_nextchr(int v) { (void)v; }
int nvim_regexp_get_at_start(void) { return 0; }
void nvim_regexp_set_at_start(int v) { (void)v; }
int nvim_regexp_get_prev_at_start(void) { return 0; }
void nvim_regexp_set_prev_at_start(int v) { (void)v; }
int nvim_regexp_get_regnpar(void) { return 0; }
void nvim_regexp_set_regnpar(int v) { (void)v; }
int nvim_regexp_get_reg_magic(void) { return 3; } // MAGIC_ON
void nvim_regexp_set_reg_magic(int v) { (void)v; }
int nvim_regexp_get_after_slash(void) { return 0; }
void nvim_regexp_set_after_slash(int v) { (void)v; }
int nvim_regexp_get_rex_reg_ic(void) { return 0; }
int nvim_regexp_get_rex_reg_icombine(void) { return 0; }
int nvim_regexp_emsg2_fail(const char *msg, int is_magic_all) { (void)msg; (void)is_magic_all; return 0; }

// --- Rust FFI declarations ---
extern char *rs_skip_regexp(char *startp, int delim, int magic);
extern int rs_no_magic(int x);
extern int rs_toggle_magic(int x);
extern int rs_re_multi_type(int c);
extern int rs_backslash_trans(int c);
extern void rs_init_class_tab(int16_t *out);
extern int re_multiline(const void *prog);
extern int64_t rs_gethexchrs(int maxinputlen);
extern int64_t rs_getdecchrs(void);
extern int64_t rs_getoctchrs(void);

// --- Reference C implementation of skip_regexp (ASCII-only, simplified) ---
// This mirrors the logic in src/nvim/regexp.c skip_regexp_ex() but without
// multi-byte support or cpo_flags dependency. Sufficient for testing the
// core delimiter-skipping logic that the Rust version must match.

static char *c_skip_anyof(char *p)
{
    if (*p == '^') {
        p++;
    }
    if (*p == ']' || *p == '-') {
        p++;
    }
    while (*p != '\0' && *p != ']') {
        if (*p == '\\' && p[1] != '\0') {
            p += 2;
        } else if (*p == '-') {
            p++;
            if (*p != ']' && *p != '\0') {
                p++;
            }
        } else if (*p == '[') {
            // Skip [:class:], [=equiv=], [.collation.] if present
            if (p[1] == ':' || p[1] == '=' || p[1] == '.') {
                char close = p[1];
                char *q = p + 2;
                while (*q != '\0') {
                    if (*q == close && q[1] == ']') {
                        p = q + 1;
                        break;
                    }
                    q++;
                }
                if (*q == '\0') {
                    p++;
                } else {
                    p++;
                }
            } else {
                p++;
            }
        } else {
            p++;
        }
    }
    return p;
}

// Simplified skip_regexp: skips past a regexp pattern to the delimiter.
// Returns pointer to the delimiter char (or to NUL if not found).
// magic=1 means 'magic' is on (brackets are special without backslash)
// magic=0 means 'magic' is off (brackets need backslash)
static char *c_skip_regexp(char *startp, int delim, int magic)
{
    magic_T mymagic = magic ? MAGIC_ON : MAGIC_OFF;
    char *p = startp;

    for (; p[0] != '\0'; p++) {
        if (p[0] == delim) {
            break;
        }
        if ((p[0] == '[' && mymagic >= MAGIC_ON)
            || (p[0] == '\\' && p[1] == '[' && mymagic <= MAGIC_OFF)) {
            p = c_skip_anyof(p + 1);
            if (p[0] == '\0') {
                break;
            }
        } else if (p[0] == '\\' && p[1] != '\0') {
            p++;  // skip next character
            if (*p == 'v') {
                mymagic = MAGIC_ALL;
            } else if (*p == 'V') {
                mymagic = MAGIC_NONE;
            }
        }
    }
    return p;
}

// --- Helper: compare C and Rust skip_regexp results ---
static void compare_skip_regexp(const char *desc, const char *input, int delim, int magic)
{
    // Make mutable copies since both functions take char*
    char c_buf[256];
    char r_buf[256];
    strncpy(c_buf, input, sizeof(c_buf) - 1);
    c_buf[sizeof(c_buf) - 1] = '\0';
    strncpy(r_buf, input, sizeof(r_buf) - 1);
    r_buf[sizeof(r_buf) - 1] = '\0';

    char *c_result = c_skip_regexp(c_buf, delim, magic);
    char *r_result = rs_skip_regexp(r_buf, delim, magic);

    int c_offset = (int)(c_result - c_buf);
    int r_offset = (int)(r_result - r_buf);

    char name[512];
    snprintf(name, sizeof(name), "C vs Rust: %s (offset C=%d R=%d)", desc, c_offset, r_offset);
    TEST(name, c_offset == r_offset);
}

// --- Test cases ---

void test_skip_regexp_basic(void) {
    printf("Testing skip_regexp basic delimiter finding:\n");

    // Test 1: Simple pattern with / delimiter
    {
        char buf[] = "abc/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("simple pattern 'abc/rest' finds /",
             result == buf + 3 && *result == '/');
    }

    // Test 2: Pattern with backslash escape
    {
        char buf[] = "a\\/b/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("escaped delimiter 'a\\\\/b/rest' skips \\/ finds second /",
             result == buf + 4 && *result == '/');
    }

    // Test 3: Pattern with collection (magic on)
    {
        char buf[] = "[abc]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("collection '[abc]/rest' with magic finds /",
             result == buf + 5 && *result == '/');
    }

    // Test 4: Collection with / inside (magic on)
    {
        char buf[] = "[a/b]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("collection '[a/b]/rest' with magic skips / in []",
             result == buf + 5 && *result == '/');
    }

    // Test 5: No delimiter found
    {
        char buf[] = "abc";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("no delimiter 'abc' returns NUL",
             *result == '\0');
    }

    // Test 6: Empty pattern
    {
        char buf[] = "/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("empty pattern '/rest' finds / at start",
             result == buf && *result == '/');
    }

    // Test 7: Collection with ] at start (special case)
    {
        char buf[] = "[]abc]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("collection '[]abc]/rest' handles ] at start",
             result != NULL && *result == '/');
    }

    // Test 8: Negated collection with ] at start
    {
        char buf[] = "[^]abc]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("negated collection '[^]abc]/rest' handles ^] at start",
             result != NULL && *result == '/');
    }

    // Test 9: ? delimiter
    {
        char buf[] = "abc?rest";
        char *result = c_skip_regexp(buf, '?', 1);
        TEST("? delimiter 'abc?rest' finds ?",
             result == buf + 3 && *result == '?');
    }

    // Test 10: Backslash at end of pattern
    {
        char buf[] = "abc\\";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("trailing backslash 'abc\\\\' returns NUL",
             *result == '\0');
    }
}

void test_skip_regexp_magic(void) {
    printf("Testing skip_regexp magic mode handling:\n");

    // Test 1: Magic off - brackets are NOT special
    {
        char buf[] = "[abc]/rest";
        char *result = c_skip_regexp(buf, '/', 0);
        TEST("magic off: '[abc]/rest' treats [ as literal, finds / at pos 5",
             result == buf + 5 && *result == '/');
    }

    // Test 2: Magic off - escaped bracket IS special
    {
        char buf[] = "\\[abc]/rest";
        char *result = c_skip_regexp(buf, '/', 0);
        // With magic off, \[ starts a collection. skip_anyof handles rest.
        TEST("magic off: '\\\\[abc]/rest' escaped [ starts collection",
             result != NULL && *result == '/');
    }

    // Test 3: \\v switches to very magic mid-pattern
    {
        char buf[] = "abc\\v[def]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        // \v is consumed as magic switch, then [def] is a collection (very magic)
        TEST("\\\\v switches to very magic: handles [def] as collection",
             result != NULL && *result == '/');
    }

    // Test 4: \\V switches to very nomagic mid-pattern
    {
        char buf[] = "abc\\V[def]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        // After \V, [ is literal in very nomagic mode
        TEST("\\\\V switches to very nomagic",
             result != NULL && *result == '/');
    }
}

void test_skip_regexp_edge_cases(void) {
    printf("Testing skip_regexp edge cases:\n");

    // Test 1: Multiple backslash escapes
    {
        char buf[] = "a\\\\b/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        // a, \\, b, then /
        TEST("double backslash 'a\\\\\\\\b/rest' finds /",
             result == buf + 4 && *result == '/');
    }

    // Test 2: Collection with range
    {
        char buf[] = "[a-z]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("range '[a-z]/rest' finds /",
             result == buf + 5 && *result == '/');
    }

    // Test 3: Collection with - at start
    {
        char buf[] = "[-az]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("'[-az]/rest' handles - at start",
             result != NULL && *result == '/');
    }

    // Test 4: Nested character class
    {
        char buf[] = "[[:alpha:]]/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("'[[:alpha:]]/rest' handles [:alpha:]",
             result != NULL && *result == '/');
    }

    // Test 5: Only delimiter
    {
        char buf[] = "/";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("only delimiter '/' finds it at pos 0",
             result == buf && *result == '/');
    }

    // Test 6: Only NUL
    {
        char buf[] = "";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("empty string returns NUL",
             result == buf && *result == '\0');
    }

    // Test 7: Backslash-escaped special chars
    {
        char buf[] = "a\\(b\\)c/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("escaped parens 'a\\\\(b\\\\)c/rest' finds /",
             result == buf + 7 && *result == '/');
    }

    // Test 8: Alternation with backslash
    {
        char buf[] = "a\\|b/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("alternation 'a\\\\|b/rest' finds /",
             result == buf + 4 && *result == '/');
    }

    // Test 9: Complex pattern
    {
        char buf[] = "\\v(foo|bar)[0-9]+/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        // After \v (very magic), parens and [] are special
        TEST("complex very magic pattern finds /",
             result != NULL && *result == '/');
    }

    // Test 10: Pattern with all magic modes
    {
        char buf[] = "a\\vb\\Vc\\md\\Me/rest";
        char *result = c_skip_regexp(buf, '/', 1);
        TEST("multiple magic switches finds /",
             result != NULL && *result == '/');
    }
}

void test_rust_vs_c_comparison(void) {
    printf("Testing Rust rs_skip_regexp vs C reference:\n");

    // Basic patterns
    compare_skip_regexp("simple 'abc/rest'", "abc/rest", '/', 1);
    compare_skip_regexp("escaped delimiter 'a\\/b/rest'", "a\\/b/rest", '/', 1);
    compare_skip_regexp("collection '[abc]/rest'", "[abc]/rest", '/', 1);
    compare_skip_regexp("collection with delim '[a/b]/rest'", "[a/b]/rest", '/', 1);
    compare_skip_regexp("no delimiter 'abc'", "abc", '/', 1);
    compare_skip_regexp("empty pattern '/rest'", "/rest", '/', 1);
    compare_skip_regexp("empty string", "", '/', 1);
    compare_skip_regexp("? delimiter 'abc?rest'", "abc?rest", '?', 1);
    compare_skip_regexp("trailing backslash 'abc\\'", "abc\\", '/', 1);

    // Collections
    compare_skip_regexp("'] at start []abc]/rest'", "[]abc]/rest", '/', 1);
    compare_skip_regexp("negated '] at start [^]abc]/rest'", "[^]abc]/rest", '/', 1);
    compare_skip_regexp("range '[a-z]/rest'", "[a-z]/rest", '/', 1);
    compare_skip_regexp("- at start '[-az]/rest'", "[-az]/rest", '/', 1);
    compare_skip_regexp("char class '[[:alpha:]]/rest'", "[[:alpha:]]/rest", '/', 1);

    // Magic modes
    compare_skip_regexp("magic off '[abc]/rest'", "[abc]/rest", '/', 0);
    compare_skip_regexp("magic off escaped '\\[abc]/rest'", "\\[abc]/rest", '/', 0);
    compare_skip_regexp("\\v very magic 'abc\\v[def]/rest'", "abc\\v[def]/rest", '/', 1);
    compare_skip_regexp("\\V very nomagic 'abc\\V[def]/rest'", "abc\\V[def]/rest", '/', 1);

    // Edge cases
    compare_skip_regexp("double backslash 'a\\\\b/rest'", "a\\\\b/rest", '/', 1);
    compare_skip_regexp("escaped parens 'a\\(b\\)c/rest'", "a\\(b\\)c/rest", '/', 1);
    compare_skip_regexp("alternation 'a\\|b/rest'", "a\\|b/rest", '/', 1);
    compare_skip_regexp("very magic complex '\\v(foo|bar)[0-9]+/rest'",
                        "\\v(foo|bar)[0-9]+/rest", '/', 1);
    compare_skip_regexp("magic switches 'a\\vb\\Vc\\md\\Me/rest'",
                        "a\\vb\\Vc\\md\\Me/rest", '/', 1);
    compare_skip_regexp("only delimiter '/'", "/", '/', 1);
}

// --- Magic macros (matching regexp.c) ---
#define Magic(x)        ((int)(x) - 256)
#define un_Magic(x)     ((x) + 256)
#define is_Magic(x)     ((x) < 0)

// Multi-type constants
#define NOT_MULTI       0
#define MULTI_ONE       1
#define MULTI_MULT      2

// Control character constants
#define C_BS    '\010'
#define C_TAB   '\011'
#define C_CAR   '\015'
#define C_ESC   '\033'

// --- C reference implementations ---

static int c_no_Magic(int x)
{
    if (is_Magic(x)) {
        return un_Magic(x);
    }
    return x;
}

static int c_toggle_Magic(int x)
{
    if (is_Magic(x)) {
        return un_Magic(x);
    }
    return Magic(x);
}

static int c_re_multi_type(int c)
{
    if (c == Magic('@') || c == Magic('=') || c == Magic('?')) {
        return MULTI_ONE;
    }
    if (c == Magic('*') || c == Magic('+') || c == Magic('{')) {
        return MULTI_MULT;
    }
    return NOT_MULTI;
}

static int c_backslash_trans(int c)
{
    switch (c) {
    case 'r': return C_CAR;
    case 't': return C_TAB;
    case 'e': return C_ESC;
    case 'b': return C_BS;
    }
    return c;
}

// --- Comparison tests for Phase 1 functions ---

void test_no_magic(void) {
    printf("Testing no_Magic (C vs Rust, exhaustive [-512, 512)):\n");
    int mismatches = 0;
    for (int i = -512; i < 512; i++) {
        int c_val = c_no_Magic(i);
        int r_val = rs_no_magic(i);
        if (c_val != r_val) {
            if (mismatches < 5) {
                printf("  MISMATCH at %d: C=%d Rust=%d\n", i, c_val, r_val);
            }
            mismatches++;
        }
    }
    char name[128];
    snprintf(name, sizeof(name), "no_Magic: %d/1024 values match", 1024 - mismatches);
    TEST(name, mismatches == 0);
}

void test_toggle_magic(void) {
    printf("Testing toggle_Magic (C vs Rust, exhaustive [-512, 512)):\n");
    int mismatches = 0;
    for (int i = -512; i < 512; i++) {
        int c_val = c_toggle_Magic(i);
        int r_val = rs_toggle_magic(i);
        if (c_val != r_val) {
            if (mismatches < 5) {
                printf("  MISMATCH at %d: C=%d Rust=%d\n", i, c_val, r_val);
            }
            mismatches++;
        }
    }
    char name[128];
    snprintf(name, sizeof(name), "toggle_Magic: %d/1024 values match", 1024 - mismatches);
    TEST(name, mismatches == 0);
}

void test_re_multi_type(void) {
    printf("Testing re_multi_type (C vs Rust, exhaustive [-512, 512)):\n");
    int mismatches = 0;
    for (int i = -512; i < 512; i++) {
        int c_val = c_re_multi_type(i);
        int r_val = rs_re_multi_type(i);
        if (c_val != r_val) {
            if (mismatches < 5) {
                printf("  MISMATCH at %d: C=%d Rust=%d\n", i, c_val, r_val);
            }
            mismatches++;
        }
    }
    char name[128];
    snprintf(name, sizeof(name), "re_multi_type: %d/1024 values match", 1024 - mismatches);
    TEST(name, mismatches == 0);
}

void test_backslash_trans(void) {
    printf("Testing backslash_trans (C vs Rust, all 256 byte values):\n");
    int mismatches = 0;
    for (int i = 0; i < 256; i++) {
        int c_val = c_backslash_trans(i);
        int r_val = rs_backslash_trans(i);
        if (c_val != r_val) {
            if (mismatches < 5) {
                printf("  MISMATCH at %d: C=%d Rust=%d\n", i, c_val, r_val);
            }
            mismatches++;
        }
    }
    char name[128];
    snprintf(name, sizeof(name), "backslash_trans: %d/256 values match", 256 - mismatches);
    TEST(name, mismatches == 0);
}

// --- RI_* constants (matching regexp.c) ---
#define RI_DIGIT    0x01
#define RI_HEX      0x02
#define RI_OCTAL    0x04
#define RI_WORD     0x08
#define RI_HEAD     0x10
#define RI_ALPHA    0x20
#define RI_LOWER    0x40
#define RI_UPPER    0x80
#define RI_WHITE    0x100

// Build the C reference class table using the same logic as the original
static void c_init_class_tab(int16_t *tab)
{
    for (int i = 0; i < 256; i++) {
        if (i >= '0' && i <= '7') {
            tab[i] = RI_DIGIT + RI_HEX + RI_OCTAL + RI_WORD;
        } else if (i >= '8' && i <= '9') {
            tab[i] = RI_DIGIT + RI_HEX + RI_WORD;
        } else if (i >= 'a' && i <= 'f') {
            tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
        } else if (i >= 'g' && i <= 'z') {
            tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
        } else if (i >= 'A' && i <= 'F') {
            tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
        } else if (i >= 'G' && i <= 'Z') {
            tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
        } else if (i == '_') {
            tab[i] = RI_WORD + RI_HEAD;
        } else {
            tab[i] = 0;
        }
    }
    tab[' '] |= RI_WHITE;
    tab['\t'] |= RI_WHITE;
}

void test_init_class_tab(void) {
    printf("Testing init_class_tab (C vs Rust, all 256 entries):\n");

    int16_t c_tab[256];
    int16_t r_tab[256];
    c_init_class_tab(c_tab);
    rs_init_class_tab(r_tab);

    int mismatches = 0;
    for (int i = 0; i < 256; i++) {
        if (c_tab[i] != r_tab[i]) {
            if (mismatches < 5) {
                printf("  MISMATCH at %d ('%c'): C=0x%03x Rust=0x%03x\n",
                       i, (i >= 32 && i < 127) ? i : '.', c_tab[i], r_tab[i]);
            }
            mismatches++;
        }
    }
    char name[128];
    snprintf(name, sizeof(name), "init_class_tab: %d/256 entries match", 256 - mismatches);
    TEST(name, mismatches == 0);
}

// --- Mock regprog_T for re_multiline testing ---
// Must match the layout of struct regprog in regexp.c:
//   regengine_T *engine;   (pointer)
//   unsigned regflags;
//   unsigned re_engine;
//   unsigned re_flags;
//   bool re_in_use;
struct mock_regprog {
    void *engine;
    unsigned int regflags;
    unsigned int re_engine;
    unsigned int re_flags;
    int re_in_use;
};

#define RF_HASNL 4

// Accessor stub — re_multiline calls this via FFI
unsigned int nvim_regexp_get_regflags(const void *prog) {
    const struct mock_regprog *p = (const struct mock_regprog *)prog;
    return p->regflags;
}

void test_re_multiline(void) {
    printf("Testing re_multiline (mock regprog_T):\n");

    // Test 1: RF_HASNL set
    {
        struct mock_regprog prog = { .engine = NULL, .regflags = RF_HASNL };
        int result = re_multiline(&prog);
        TEST("RF_HASNL set -> non-zero", result != 0);
    }

    // Test 2: RF_HASNL not set
    {
        struct mock_regprog prog = { .engine = NULL, .regflags = 0 };
        int result = re_multiline(&prog);
        TEST("no flags -> 0", result == 0);
    }

    // Test 3: RF_HASNL set among other flags
    {
        struct mock_regprog prog = { .engine = NULL, .regflags = 1 | RF_HASNL | 8 };
        int result = re_multiline(&prog);
        TEST("mixed flags with RF_HASNL -> non-zero", result != 0);
    }

    // Test 4: other flags without RF_HASNL
    {
        struct mock_regprog prog = { .engine = NULL, .regflags = 1 | 2 | 8 };
        int result = re_multiline(&prog);
        TEST("other flags without RF_HASNL -> 0", result == 0);
    }
}

// --- C reference number parsers (matching regexp.c originals) ---

static int c_ascii_isxdigit(int c) {
    return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}

static int c_hex2nr(int c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return 0;
}

static int64_t c_gethexchrs(char **pp, int maxinputlen) {
    int64_t nr = 0;
    int i;
    for (i = 0; i < maxinputlen; i++) {
        int c = (unsigned char)(*pp)[0];
        if (!c_ascii_isxdigit(c)) break;
        nr <<= 4;
        nr |= c_hex2nr(c);
        (*pp)++;
    }
    return (i == 0) ? -1 : nr;
}

static int64_t c_getdecchrs(char **pp) {
    int64_t nr = 0;
    int i;
    for (i = 0; ; i++) {
        int c = (unsigned char)(*pp)[0];
        if (c < '0' || c > '9') break;
        nr *= 10;
        nr += c - '0';
        (*pp)++;
    }
    return (i == 0) ? -1 : nr;
}

static int64_t c_getoctchrs(char **pp) {
    int64_t nr = 0;
    int i;
    for (i = 0; i < 3 && nr < 040; i++) {
        int c = (unsigned char)(*pp)[0];
        if (c < '0' || c > '7') break;
        nr <<= 3;
        nr |= c_hex2nr(c);
        (*pp)++;
    }
    return (i == 0) ? -1 : nr;
}

// --- Number parser comparison tests ---

void test_gethexchrs(void) {
    printf("Testing gethexchrs (C vs Rust):\n");

    struct { const char *input; int maxlen; } cases[] = {
        {"20", 2}, {"ff", 2}, {"FF", 2}, {"0a", 2},
        {"20AC", 4}, {"20AC", 2},  // clipping
        {"", 2}, {"gg", 2}, {"xyz", 4},  // no hex digits
        {"2g", 2}, {"a_rest", 4},  // partial
        {"12345678", 8},  // 8-digit
    };

    for (int t = 0; t < (int)(sizeof(cases)/sizeof(cases[0])); t++) {
        char c_buf[64], r_buf[64];
        strncpy(c_buf, cases[t].input, sizeof(c_buf) - 1);
        c_buf[sizeof(c_buf)-1] = '\0';
        strncpy(r_buf, cases[t].input, sizeof(r_buf) - 1);
        r_buf[sizeof(r_buf)-1] = '\0';

        char *c_ptr = c_buf;
        int64_t c_val = c_gethexchrs(&c_ptr, cases[t].maxlen);
        int c_consumed = (int)(c_ptr - c_buf);

        mock_regparse = r_buf;
        int64_t r_val = rs_gethexchrs(cases[t].maxlen);
        int r_consumed = (int)(mock_regparse - r_buf);

        char name[256];
        snprintf(name, sizeof(name), "gethexchrs(\"%s\", %d): C=%ld/%d Rust=%ld/%d",
                 cases[t].input, cases[t].maxlen, (long)c_val, c_consumed, (long)r_val, r_consumed);
        TEST(name, c_val == r_val && c_consumed == r_consumed);
    }
}

void test_getdecchrs(void) {
    printf("Testing getdecchrs (C vs Rust):\n");

    const char *cases[] = {
        "123", "0", "42rest", "", "abc", "999999", "0000", "1",
    };

    for (int t = 0; t < (int)(sizeof(cases)/sizeof(cases[0])); t++) {
        char c_buf[64], r_buf[64];
        strncpy(c_buf, cases[t], sizeof(c_buf) - 1);
        c_buf[sizeof(c_buf)-1] = '\0';
        strncpy(r_buf, cases[t], sizeof(r_buf) - 1);
        r_buf[sizeof(r_buf)-1] = '\0';

        char *c_ptr = c_buf;
        int64_t c_val = c_getdecchrs(&c_ptr);
        int c_consumed = (int)(c_ptr - c_buf);

        mock_regparse = r_buf;
        int64_t r_val = rs_getdecchrs();
        int r_consumed = (int)(mock_regparse - r_buf);

        char name[256];
        snprintf(name, sizeof(name), "getdecchrs(\"%s\"): C=%ld/%d Rust=%ld/%d",
                 cases[t], (long)c_val, c_consumed, (long)r_val, r_consumed);
        TEST(name, c_val == r_val && c_consumed == r_consumed);
    }
}

void test_getoctchrs(void) {
    printf("Testing getoctchrs (C vs Rust):\n");

    const char *cases[] = {
        "377", "210", "0", "7", "", "8", "9",
        "400", "370", "1234", "77", "01",
    };

    for (int t = 0; t < (int)(sizeof(cases)/sizeof(cases[0])); t++) {
        char c_buf[64], r_buf[64];
        strncpy(c_buf, cases[t], sizeof(c_buf) - 1);
        c_buf[sizeof(c_buf)-1] = '\0';
        strncpy(r_buf, cases[t], sizeof(r_buf) - 1);
        r_buf[sizeof(r_buf)-1] = '\0';

        char *c_ptr = c_buf;
        int64_t c_val = c_getoctchrs(&c_ptr);
        int c_consumed = (int)(c_ptr - c_buf);

        mock_regparse = r_buf;
        int64_t r_val = rs_getoctchrs();
        int r_consumed = (int)(mock_regparse - r_buf);

        char name[256];
        snprintf(name, sizeof(name), "getoctchrs(\"%s\"): C=%ld/%d Rust=%ld/%d",
                 cases[t], (long)c_val, c_consumed, (long)r_val, r_consumed);
        TEST(name, c_val == r_val && c_consumed == r_consumed);
    }
}

int main(void) {
    printf("=== Comparing C regexp utility implementations ===\n\n");

    test_skip_regexp_basic();
    test_skip_regexp_magic();
    test_skip_regexp_edge_cases();
    test_rust_vs_c_comparison();
    test_no_magic();
    test_toggle_magic();
    test_re_multi_type();
    test_backslash_trans();
    test_init_class_tab();
    test_re_multiline();
    test_gethexchrs();
    test_getdecchrs();
    test_getoctchrs();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
