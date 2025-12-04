// Test that Rust charset functions match C implementations
// Compile: cc -o /tmp/compare_charset src/nvim-rs/test/compare_charset.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_charset

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>
#include <stdint.h>
#include <stddef.h>

// Rust implementations
extern const char *rs_skipwhite(const char *p);
extern const char *rs_skipwhite_len(const char *p, size_t len);
extern const char *rs_skipdigits(const char *q);
extern const char *rs_skipbin(const char *q);
extern const char *rs_skiphex(const char *q);
extern const char *rs_skiptodigit(const char *q);
extern const char *rs_skiptobin(const char *q);
extern const char *rs_skiptohex(const char *q);
extern const char *rs_skiptowhite(const char *p);
extern const char *rs_skiptowhite_esc(const char *p);
extern intptr_t rs_getwhitecols(const char *p);
extern int rs_hex2nr(int c);
extern int rs_hexhex2nr(const char *p);

// C implementations
static inline bool ascii_iswhite(int c) {
    return c == ' ' || c == '\t';
}

static inline bool ascii_isdigit(int c) {
    return c >= '0' && c <= '9';
}

static inline bool ascii_isbdigit(int c) {
    return c == '0' || c == '1';
}

static inline bool ascii_isxdigit(int c) {
    return (c >= '0' && c <= '9') || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f');
}

static char *c_skipwhite(const char *p) {
    while (ascii_iswhite(*p)) {
        p++;
    }
    return (char *)p;
}

static char *c_skipwhite_len(const char *p, size_t len) {
    for (; len > 0 && ascii_iswhite(*p); len--) {
        p++;
    }
    return (char *)p;
}

static char *c_skipdigits(const char *q) {
    const char *p = q;
    while (ascii_isdigit(*p)) {
        p++;
    }
    return (char *)p;
}

static const char *c_skipbin(const char *q) {
    const char *p = q;
    while (ascii_isbdigit(*p)) {
        p++;
    }
    return p;
}

static char *c_skiphex(char *q) {
    char *p = q;
    while (ascii_isxdigit(*p)) {
        p++;
    }
    return p;
}

static char *c_skiptodigit(char *q) {
    char *p = q;
    while (*p != '\0' && !ascii_isdigit(*p)) {
        p++;
    }
    return p;
}

static const char *c_skiptobin(const char *q) {
    const char *p = q;
    while (*p != '\0' && !ascii_isbdigit(*p)) {
        p++;
    }
    return p;
}

static char *c_skiptohex(char *q) {
    char *p = q;
    while (*p != '\0' && !ascii_isxdigit(*p)) {
        p++;
    }
    return p;
}

static char *c_skiptowhite(const char *p) {
    while (*p != ' ' && *p != '\t' && *p != '\0') {
        p++;
    }
    return (char *)p;
}

#define Ctrl_V 22  // ASCII value for Ctrl-V

static char *c_skiptowhite_esc(const char *p) {
    while (*p != ' ' && *p != '\t' && *p != '\0') {
        if ((*p == '\\' || *p == Ctrl_V) && *(p + 1) != '\0') {
            p++;
        }
        p++;
    }
    return (char *)p;
}

static intptr_t c_getwhitecols(const char *p) {
    return c_skipwhite(p) - p;
}

static int c_hex2nr(int c) {
    if ((c >= 'a') && (c <= 'f')) {
        return c - 'a' + 10;
    }
    if ((c >= 'A') && (c <= 'F')) {
        return c - 'A' + 10;
    }
    return c - '0';
}

static int c_hexhex2nr(const char *p) {
    if (!ascii_isxdigit(p[0]) || !ascii_isxdigit(p[1])) {
        return -1;
    }
    return (c_hex2nr(p[0]) << 4) + c_hex2nr(p[1]);
}

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

void test_skipwhite(void) {
    printf("Testing skipwhite:\n");

    const char *test_strs[] = {
        "hello",
        "  hello",
        "\thello",
        " \t \thello",
        "",
        "   ",
        "\t\t\t",
        "no whitespace",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skipwhite(test_strs[i]);
        const char *rs_result = rs_skipwhite(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skipwhite test_%d: offset C=%td Rust=%td", i, c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_skipwhite_len(void) {
    printf("Testing skipwhite_len:\n");

    struct {
        const char *str;
        size_t len;
    } test_cases[] = {
        {"     hello", 3},
        {"     hello", 5},
        {"     hello", 10},
        {"  hello", 0},
        {"  hello", 1},
        {"hello", 5},
        {"", 0},
        {"   ", 2},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skipwhite_len(test_cases[i].str, test_cases[i].len);
        const char *rs_result = rs_skipwhite_len(test_cases[i].str, test_cases[i].len);

        ptrdiff_t c_offset = c_result - test_cases[i].str;
        ptrdiff_t rs_offset = rs_result - test_cases[i].str;

        char name[128];
        snprintf(name, sizeof(name), "skipwhite_len test_%d (len=%zu): offset C=%td Rust=%td",
                 i, test_cases[i].len, c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_skipdigits(void) {
    printf("Testing skipdigits:\n");

    const char *test_strs[] = {
        "12345abc",
        "abc123",
        "12345",
        "",
        "000",
        "99999xyz",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skipdigits(test_strs[i]);
        const char *rs_result = rs_skipdigits(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skipdigits(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_skipbin(void) {
    printf("Testing skipbin:\n");

    const char *test_strs[] = {
        "01010abc",
        "abc010",
        "0101",
        "",
        "00011100",
        "01012345",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skipbin(test_strs[i]);
        const char *rs_result = rs_skipbin(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skipbin(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_skiphex(void) {
    printf("Testing skiphex:\n");

    const char *test_strs[] = {
        "1a2b3cGHI",
        "ABCDEF123xyz",
        "xyz",
        "",
        "deadbeef",
        "0123456789abcdef",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        char *str_copy = strdup(test_strs[i]);
        const char *c_result = c_skiphex(str_copy);
        const char *rs_result = rs_skiphex(str_copy);

        ptrdiff_t c_offset = c_result - str_copy;
        ptrdiff_t rs_offset = rs_result - str_copy;

        char name[128];
        snprintf(name, sizeof(name), "skiphex(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
        free(str_copy);
    }
}

void test_skiptodigit(void) {
    printf("Testing skiptodigit:\n");

    const char *test_strs[] = {
        "abc123",
        "123",
        "abc",
        "",
        "---5---",
        "xyz",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        char *str_copy = strdup(test_strs[i]);
        const char *c_result = c_skiptodigit(str_copy);
        const char *rs_result = rs_skiptodigit(str_copy);

        ptrdiff_t c_offset = c_result - str_copy;
        ptrdiff_t rs_offset = rs_result - str_copy;

        char name[128];
        snprintf(name, sizeof(name), "skiptodigit(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
        free(str_copy);
    }
}

void test_skiptobin(void) {
    printf("Testing skiptobin:\n");

    const char *test_strs[] = {
        "abc0101",
        "0101",
        "abc",
        "",
        "---1---",
        "234567",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skiptobin(test_strs[i]);
        const char *rs_result = rs_skiptobin(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skiptobin(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_skiptohex(void) {
    printf("Testing skiptohex:\n");

    const char *test_strs[] = {
        "xyz1aF",
        "AbCd",
        "ghi",
        "",
        "---A---",
        "ghijkl",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        char *str_copy = strdup(test_strs[i]);
        const char *c_result = c_skiptohex(str_copy);
        const char *rs_result = rs_skiptohex(str_copy);

        ptrdiff_t c_offset = c_result - str_copy;
        ptrdiff_t rs_offset = rs_result - str_copy;

        char name[128];
        snprintf(name, sizeof(name), "skiptohex(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
        free(str_copy);
    }
}

void test_skiptowhite(void) {
    printf("Testing skiptowhite:\n");

    const char *test_strs[] = {
        "hello world",
        "hello\tworld",
        "hello",
        "",
        " leading",
        "nospace",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skiptowhite(test_strs[i]);
        const char *rs_result = rs_skiptowhite(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skiptowhite(\"%s\"): offset C=%td Rust=%td",
                 test_strs[i], c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }
}

void test_hex2nr(void) {
    printf("Testing hex2nr:\n");

    int test_chars[] = {
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f',
        'A', 'B', 'C', 'D', 'E', 'F',
    };
    int n = sizeof(test_chars) / sizeof(test_chars[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_hex2nr(test_chars[i]);
        int rs_result = rs_hex2nr(test_chars[i]);

        char name[64];
        snprintf(name, sizeof(name), "hex2nr('%c'): C=%d Rust=%d", test_chars[i], c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_hexhex2nr(void) {
    printf("Testing hexhex2nr:\n");

    const char *test_strs[] = {
        "00",
        "FF",
        "ff",
        "1a",
        "A1",
        "GG",  // invalid
        "1G",  // invalid
        "G1",  // invalid
        "ab",
        "CD",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_hexhex2nr(test_strs[i]);
        int rs_result = rs_hexhex2nr(test_strs[i]);

        char name[64];
        snprintf(name, sizeof(name), "hexhex2nr(\"%s\"): C=%d Rust=%d", test_strs[i], c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_skiptowhite_esc(void) {
    printf("Testing skiptowhite_esc:\n");

    const char *test_strs[] = {
        "hello world",
        "hello\tworld",
        "hello",
        "",
        " leading",
        "nospace",
        "hello\\ world",     // backslash escapes space, continues to end
        "hello\\x world",    // backslash escapes x, then hits space
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *c_result = c_skiptowhite_esc(test_strs[i]);
        const char *rs_result = rs_skiptowhite_esc(test_strs[i]);

        ptrdiff_t c_offset = c_result - test_strs[i];
        ptrdiff_t rs_offset = rs_result - test_strs[i];

        char name[128];
        snprintf(name, sizeof(name), "skiptowhite_esc test_%d: offset C=%td Rust=%td",
                 i, c_offset, rs_offset);
        TEST(name, c_offset == rs_offset);
    }

    // Test with Ctrl-V escape
    char ctrl_v_str[] = {'h', 'i', Ctrl_V, ' ', 'x', '\0'};
    const char *c_result = c_skiptowhite_esc(ctrl_v_str);
    const char *rs_result = rs_skiptowhite_esc(ctrl_v_str);
    ptrdiff_t c_offset = c_result - ctrl_v_str;
    ptrdiff_t rs_offset = rs_result - ctrl_v_str;
    char name[128];
    snprintf(name, sizeof(name), "skiptowhite_esc with Ctrl-V: offset C=%td Rust=%td",
             c_offset, rs_offset);
    TEST(name, c_offset == rs_offset);
}

void test_getwhitecols(void) {
    printf("Testing getwhitecols:\n");

    const char *test_strs[] = {
        "   hello",
        "\t\thello",
        " \t \thello",
        "hello",
        "",
        "   ",
        "\t\t\t",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        intptr_t c_result = c_getwhitecols(test_strs[i]);
        intptr_t rs_result = rs_getwhitecols(test_strs[i]);

        char name[128];
        snprintf(name, sizeof(name), "getwhitecols test_%d: C=%td Rust=%td",
                 i, (ptrdiff_t)c_result, (ptrdiff_t)rs_result);
        TEST(name, c_result == rs_result);
    }
}

int main(void) {
    printf("=== Comparing C and Rust charset implementations ===\n\n");

    test_skipwhite();
    test_skipwhite_len();
    test_skipdigits();
    test_skipbin();
    test_skiphex();
    test_skiptodigit();
    test_skiptobin();
    test_skiptohex();
    test_skiptowhite();
    test_skiptowhite_esc();
    test_getwhitecols();
    test_hex2nr();
    test_hexhex2nr();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
