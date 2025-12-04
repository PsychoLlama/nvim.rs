// Test that Rust mbyte functions match C implementations
// Compile: cc -o /tmp/compare_mbyte src/nvim-rs/test/compare_mbyte.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_mbyte

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>

// Rust implementations
extern int rs_utf_char2len(int c);
extern int rs_utf_char2bytes(int c, char *buf);
extern int rs_utf_byte2len(int b);
extern int rs_utf_ptr2char(const char *p);
extern int rs_utf_ptr2len(const char *p);

// C implementations (simplified from nvim/mbyte.c)
static uint8_t utf8len_tab[256] = {
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 1, 1,
};

static int c_utf_char2len(int c) {
    if (c < 0x80) return 1;
    else if (c < 0x800) return 2;
    else if (c < 0x10000) return 3;
    else if (c < 0x200000) return 4;
    else if (c < 0x4000000) return 5;
    else return 6;
}

static int c_utf_char2bytes(int c, char *buf) {
    if (c < 0x80) {
        buf[0] = (char)c;
        return 1;
    } else if (c < 0x800) {
        buf[0] = (char)(0xc0 + ((unsigned)c >> 6));
        buf[1] = (char)(0x80 + ((unsigned)c & 0x3f));
        return 2;
    } else if (c < 0x10000) {
        buf[0] = (char)(0xe0 + ((unsigned)c >> 12));
        buf[1] = (char)(0x80 + (((unsigned)c >> 6) & 0x3f));
        buf[2] = (char)(0x80 + ((unsigned)c & 0x3f));
        return 3;
    } else if (c < 0x200000) {
        buf[0] = (char)(0xf0 + ((unsigned)c >> 18));
        buf[1] = (char)(0x80 + (((unsigned)c >> 12) & 0x3f));
        buf[2] = (char)(0x80 + (((unsigned)c >> 6) & 0x3f));
        buf[3] = (char)(0x80 + ((unsigned)c & 0x3f));
        return 4;
    } else if (c < 0x4000000) {
        buf[0] = (char)(0xf8 + ((unsigned)c >> 24));
        buf[1] = (char)(0x80 + (((unsigned)c >> 18) & 0x3f));
        buf[2] = (char)(0x80 + (((unsigned)c >> 12) & 0x3f));
        buf[3] = (char)(0x80 + (((unsigned)c >> 6) & 0x3f));
        buf[4] = (char)(0x80 + ((unsigned)c & 0x3f));
        return 5;
    } else {
        buf[0] = (char)(0xfc + ((unsigned)c >> 30));
        buf[1] = (char)(0x80 + (((unsigned)c >> 24) & 0x3f));
        buf[2] = (char)(0x80 + (((unsigned)c >> 18) & 0x3f));
        buf[3] = (char)(0x80 + (((unsigned)c >> 12) & 0x3f));
        buf[4] = (char)(0x80 + (((unsigned)c >> 6) & 0x3f));
        buf[5] = (char)(0x80 + ((unsigned)c & 0x3f));
        return 6;
    }
}

static int c_utf_byte2len(int b) {
    return utf8len_tab[(uint8_t)b];
}

static int c_utf_ptr2len(const char *p) {
    uint8_t *pp = (uint8_t *)p;
    if (*pp == 0) return 0;
    int len = utf8len_tab[*pp];
    for (int i = 1; i < len; i++) {
        if ((pp[i] & 0xc0) != 0x80) return 1;
    }
    return len;
}

static int c_utf_ptr2char(const char *p) {
    uint8_t *pp = (uint8_t *)p;
    uint32_t v0 = pp[0];
    if (v0 < 0x80) return (int)v0;

    int len = utf8len_tab[v0];
    if (len < 2) return (int)v0;

    uint32_t v1 = pp[1];
    if ((v1 & 0xC0) != 0x80) return (int)v0;
    if (len == 2) return (int)((v0 << 6) + v1 - ((0xC0 << 6) + 0x80));

    uint32_t v2 = pp[2];
    if ((v2 & 0xC0) != 0x80) return (int)v0;
    if (len == 3) return (int)((v0 << 12) + (v1 << 6) + v2 - ((0xE0 << 12) + (0x80 << 6) + 0x80));

    uint32_t v3 = pp[3];
    if ((v3 & 0xC0) != 0x80) return (int)v0;
    if (len == 4) return (int)((v0 << 18) + (v1 << 12) + (v2 << 6) + v3
                               - ((0xF0 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80));

    uint32_t v4 = pp[4];
    if ((v4 & 0xC0) != 0x80) return (int)v0;
    if (len == 5) return (int)((v0 << 24) + (v1 << 18) + (v2 << 12) + (v3 << 6) + v4
                               - ((0xF8 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80));

    uint32_t v5 = pp[5];
    if ((v5 & 0xC0) != 0x80) return (int)v0;
    // len == 6
    return (int)((v0 << 30) + (v1 << 24) + (v2 << 18) + (v3 << 12) + (v4 << 6) + v5
                 - ((0x80 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80));
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

void test_utf_char2len(void) {
    printf("Testing utf_char2len:\n");

    int test_codepoints[] = {
        0x00, 0x7F,       // ASCII
        0x80, 0x7FF,      // 2-byte
        0x800, 0xFFFF,    // 3-byte
        0x10000, 0x1FFFF, // 4-byte
        0x1F600,          // Emoji
        0x10FFFF,         // Unicode max
    };
    int n = sizeof(test_codepoints) / sizeof(test_codepoints[0]);

    for (int i = 0; i < n; i++) {
        int c = test_codepoints[i];
        int c_result = c_utf_char2len(c);
        int rs_result = rs_utf_char2len(c);

        char name[64];
        snprintf(name, sizeof(name), "char2len(0x%X) C=%d Rust=%d", c, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_utf_char2bytes(void) {
    printf("Testing utf_char2bytes:\n");

    int test_codepoints[] = {
        0x41,     // 'A'
        0xE1,     // á (2-byte)
        0x20AC,   // € (3-byte)
        0x1F600,  // 😀 (4-byte)
    };
    int n = sizeof(test_codepoints) / sizeof(test_codepoints[0]);

    for (int i = 0; i < n; i++) {
        int c = test_codepoints[i];
        char c_buf[6] = {0};
        char rs_buf[6] = {0};

        int c_len = c_utf_char2bytes(c, c_buf);
        int rs_len = rs_utf_char2bytes(c, rs_buf);

        bool match = (c_len == rs_len) && (memcmp(c_buf, rs_buf, c_len) == 0);

        char name[64];
        snprintf(name, sizeof(name), "char2bytes(0x%X) len C=%d Rust=%d", c, c_len, rs_len);
        TEST(name, match);
    }
}

void test_utf_byte2len(void) {
    printf("Testing utf_byte2len:\n");

    int test_bytes[] = {
        0x00, 0x7F,    // ASCII (1-byte)
        0x80, 0xBF,    // Continuation (invalid as first byte, returns 1)
        0xC0, 0xDF,    // 2-byte lead
        0xE0, 0xEF,    // 3-byte lead
        0xF0, 0xF7,    // 4-byte lead
        0xF8, 0xFB,    // 5-byte lead
        0xFC, 0xFD,    // 6-byte lead
        0xFE, 0xFF,    // Invalid (returns 1)
    };
    int n = sizeof(test_bytes) / sizeof(test_bytes[0]);

    for (int i = 0; i < n; i++) {
        int b = test_bytes[i];
        int c_result = c_utf_byte2len(b);
        int rs_result = rs_utf_byte2len(b);

        char name[64];
        snprintf(name, sizeof(name), "byte2len(0x%02X) C=%d Rust=%d", b, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_utf_ptr2len(void) {
    printf("Testing utf_ptr2len:\n");

    struct {
        const char *str;
        const char *desc;
    } test_cases[] = {
        {"A", "ASCII"},
        {"\xC3\xA1", "2-byte (á)"},
        {"\xE2\x82\xAC", "3-byte (€)"},
        {"\xF0\x9F\x98\x80", "4-byte (😀)"},
        {"", "empty"},
        {"\x80", "continuation as first"},
        {"\xC3", "incomplete 2-byte"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_utf_ptr2len(test_cases[i].str);
        int rs_result = rs_utf_ptr2len(test_cases[i].str);

        char name[128];
        snprintf(name, sizeof(name), "ptr2len(%s) C=%d Rust=%d", test_cases[i].desc, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_utf_ptr2char(void) {
    printf("Testing utf_ptr2char:\n");

    struct {
        const char *str;
        const char *desc;
        int expected;
    } test_cases[] = {
        {"A", "ASCII 'A'", 0x41},
        {"\xC3\xA1", "2-byte (á)", 0xE1},
        {"\xE2\x82\xAC", "3-byte (€)", 0x20AC},
        {"\xF0\x9F\x98\x80", "4-byte (😀)", 0x1F600},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_utf_ptr2char(test_cases[i].str);
        int rs_result = rs_utf_ptr2char(test_cases[i].str);

        char name[128];
        snprintf(name, sizeof(name), "ptr2char(%s) C=0x%X Rust=0x%X expected=0x%X",
                 test_cases[i].desc, c_result, rs_result, test_cases[i].expected);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

void test_roundtrip(void) {
    printf("Testing roundtrip (char2bytes -> ptr2char):\n");

    int test_codepoints[] = {
        0x41,     // 'A'
        0xE1,     // á
        0x20AC,   // €
        0x1F600,  // 😀
        0x10FFFF, // Unicode max
    };
    int n = sizeof(test_codepoints) / sizeof(test_codepoints[0]);

    for (int i = 0; i < n; i++) {
        int c = test_codepoints[i];
        char buf[7] = {0};

        rs_utf_char2bytes(c, buf);
        int decoded = rs_utf_ptr2char(buf);

        char name[64];
        snprintf(name, sizeof(name), "roundtrip(0x%X) decoded=0x%X", c, decoded);
        TEST(name, c == decoded);
    }
}

int main(void) {
    printf("=== Comparing C and Rust mbyte implementations ===\n\n");

    test_utf_char2len();
    test_utf_char2bytes();
    test_utf_byte2len();
    test_utf_ptr2len();
    test_utf_ptr2char();
    test_roundtrip();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
