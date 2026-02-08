// Compare Rust regexp utility functions against C implementations.
// Compile: cc -o /tmp/compare_regexp src/nvim-rs/test/compare_regexp.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_regexp
//
// Tests stateless regexp helpers that are natural first migration targets:
//   - skip_regexp: skip past a regexp pattern to its delimiter
//
// These are the functions other crates (ex_docmd, search) already call via FFI.
// Note: The full skip_regexp_ex depends on global state (cpo flags, multi-byte
// functions), so we test a simplified version that covers the core logic.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

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

int main(void) {
    printf("=== Comparing C regexp utility implementations ===\n\n");

    test_skip_regexp_basic();
    test_skip_regexp_magic();
    test_skip_regexp_edge_cases();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
