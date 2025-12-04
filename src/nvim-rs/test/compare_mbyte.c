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
extern int rs_utf_ptr2len_len(const char *p, int size);
extern int rs_utf_valid_string(const char *s, const char *end);
extern int rs_utf_eat_space(int cc);
extern int rs_utf_allow_break_before(int cc);
extern int rs_utf_allow_break_after(int cc);

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

static int c_utf_ptr2len_len(const char *p, int size) {
    if (size < 1) return 1;
    int len = utf8len_tab[(uint8_t)(*p)];
    if (len == 1) return 1;
    int m = (len > size) ? size : len;
    for (int i = 1; i < m; i++) {
        if ((p[i] & 0xc0) != 0x80) return 1;
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

// UTF-8 length table with 0 for invalid lead bytes (continuation bytes, FE, FF)
static uint8_t utf8len_tab_zero[256] = {
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x80-0x8F (continuation bytes)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x90-0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xA0-0xAF
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xB0-0xBF
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 0, 0,  // FE and FF are invalid
};

static bool c_utf_valid_string(const char *s, const char *end) {
    const uint8_t *p = (uint8_t *)s;

    while (end == NULL ? *p != 0 : p < (uint8_t *)end) {
        int l = utf8len_tab_zero[*p];
        if (l == 0) {
            return false;  // invalid lead byte
        }
        if (end != NULL && p + l > (uint8_t *)end) {
            return false;  // incomplete byte sequence
        }
        p++;
        while (--l > 0) {
            if ((*p++ & 0xc0) != 0x80) {
                return false;  // invalid trail byte
            }
        }
    }
    return true;
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

void test_utf_ptr2len_len(void) {
    printf("Testing utf_ptr2len_len:\n");

    struct {
        const char *str;
        int size;
        const char *desc;
    } test_cases[] = {
        {"A", 1, "ASCII size=1"},
        {"AB", 2, "ASCII size=2"},
        {"\xC3\xA1", 2, "2-byte complete"},
        {"\xC3\xA1", 1, "2-byte truncated"},
        {"\xE2\x82\xAC", 3, "3-byte complete"},
        {"\xE2\x82\xAC", 2, "3-byte truncated"},
        {"\xE2\x82\xAC", 1, "3-byte size=1"},
        {"\xF0\x9F\x98\x80", 4, "4-byte complete"},
        {"\xF0\x9F\x98\x80", 2, "4-byte truncated"},
        {"\x80", 1, "continuation as first"},
        {"\xC3\x00", 2, "invalid continuation"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_utf_ptr2len_len(test_cases[i].str, test_cases[i].size);
        int rs_result = rs_utf_ptr2len_len(test_cases[i].str, test_cases[i].size);

        char name[128];
        snprintf(name, sizeof(name), "ptr2len_len(%s, %d) C=%d Rust=%d",
                 test_cases[i].desc, test_cases[i].size, c_result, rs_result);
        TEST(name, c_result == rs_result);
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

void test_utf_valid_string(void) {
    printf("Testing utf_valid_string:\n");

    struct {
        const char *str;
        int len;  // -1 for NUL-terminated, otherwise explicit length
        bool expected;
        const char *desc;
    } test_cases[] = {
        // Valid strings
        {"", -1, true, "empty NUL-terminated"},
        {"A", -1, true, "single ASCII"},
        {"hello", -1, true, "ASCII string"},
        {"\xC3\xA1", -1, true, "2-byte (á)"},
        {"\xE2\x82\xAC", -1, true, "3-byte (€)"},
        {"\xF0\x9F\x98\x80", -1, true, "4-byte (😀)"},
        {"hello\xC3\xA1world", -1, true, "mixed ASCII and 2-byte"},
        {"\xC3\xA1\xE2\x82\xAC\xF0\x9F\x98\x80", -1, true, "2+3+4 byte sequence"},

        // Invalid strings (NUL-terminated)
        {"\x80", -1, false, "continuation byte at start"},
        {"\xC3", -1, false, "incomplete 2-byte (missing continuation)"},
        {"\xC3X", -1, false, "invalid continuation byte (X)"},
        {"\xE2\x82", -1, false, "incomplete 3-byte (missing last)"},
        {"\xF0\x9F\x98", -1, false, "incomplete 4-byte (missing last)"},
        {"\xFE", -1, false, "invalid lead byte 0xFE"},
        {"\xFF", -1, false, "invalid lead byte 0xFF"},

        // With explicit length
        {"AB", 2, true, "explicit len=2"},
        {"\xC3\xA1", 2, true, "2-byte explicit len=2"},
        {"\xC3\xA1X", 2, true, "2-byte explicit len=2 (ignore trailing)"},
        {"\xC3\xA1", 1, false, "2-byte truncated len=1"},
        {"\xE2\x82\xAC", 2, false, "3-byte truncated len=2"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *str = test_cases[i].str;
        const char *end = (test_cases[i].len < 0) ? NULL : (str + test_cases[i].len);

        bool c_result = c_utf_valid_string(str, end);
        int rs_result = rs_utf_valid_string(str, end);
        bool rs_bool = (rs_result != 0);

        char name[128];
        snprintf(name, sizeof(name), "valid_string(%s) expected=%d C=%d Rust=%d",
                 test_cases[i].desc, test_cases[i].expected, c_result, rs_bool);
        TEST(name, c_result == rs_bool && c_result == test_cases[i].expected);
    }
}

// C implementation of utf_eat_space
static bool c_utf_eat_space(int cc) {
    return (cc >= 0x2000 && cc <= 0x206F)   // General punctuations
           || (cc >= 0x2e00 && cc <= 0x2e7f)   // Supplemental punctuations
           || (cc >= 0x3000 && cc <= 0x303f)   // CJK symbols and punctuations
           || (cc >= 0xff01 && cc <= 0xff0f)   // Full width ASCII punctuations
           || (cc >= 0xff1a && cc <= 0xff20)   // ..
           || (cc >= 0xff3b && cc <= 0xff40)   // ..
           || (cc >= 0xff5b && cc <= 0xff65);  // ..
}

// BOL prohibition punctuation (sorted for binary search)
static const int BOL_prohibition_punct[] = {
    '!', '%', ')', ',', ':', ';', '>', '?', ']', '}',
    0x2019, 0x201d, 0x2020, 0x2021, 0x2026, 0x2030, 0x2031, 0x203c,
    0x2047, 0x2048, 0x2049, 0x2103, 0x2109,
    0x3001, 0x3002, 0x3009, 0x300b, 0x300d, 0x300f, 0x3011, 0x3015,
    0x3017, 0x3019, 0x301b,
    0xff01, 0xff09, 0xff0c, 0xff0e, 0xff1a, 0xff1b, 0xff1f, 0xff3d, 0xff5d,
};

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

static bool c_utf_allow_break_before(int cc) {
    int first = 0;
    int last = ARRAY_SIZE(BOL_prohibition_punct) - 1;

    while (first < last) {
        const int mid = (first + last) / 2;
        if (cc == BOL_prohibition_punct[mid]) {
            return false;
        } else if (cc > BOL_prohibition_punct[mid]) {
            first = mid + 1;
        } else {
            last = mid - 1;
        }
    }
    return cc != BOL_prohibition_punct[first];
}

// EOL prohibition punctuation (sorted for binary search)
static const int EOL_prohibition_punct[] = {
    '(', '<', '[', '`', '{',
    0x2018, 0x201c,
    0x3008, 0x300a, 0x300c, 0x300e, 0x3010, 0x3014, 0x3016, 0x3018, 0x301a,
    0xff08, 0xff3b, 0xff5b,
};

static bool c_utf_allow_break_after(int cc) {
    int first = 0;
    int last = ARRAY_SIZE(EOL_prohibition_punct) - 1;

    while (first < last) {
        const int mid = (first + last) / 2;
        if (cc == EOL_prohibition_punct[mid]) {
            return false;
        } else if (cc > EOL_prohibition_punct[mid]) {
            first = mid + 1;
        } else {
            last = mid - 1;
        }
    }
    return cc != EOL_prohibition_punct[first];
}

void test_utf_eat_space(void) {
    printf("Testing utf_eat_space:\n");

    struct {
        int cc;
        bool expected;
        const char *desc;
    } test_cases[] = {
        // ASCII - should NOT eat space
        {'a', false, "ASCII 'a'"},
        {' ', false, "ASCII space"},
        {'.', false, "ASCII period"},
        // General punctuations (0x2000-0x206F)
        {0x2000, true, "general punct start"},
        {0x2014, true, "em dash"},
        {0x206F, true, "general punct end"},
        // Supplemental punctuations (0x2E00-0x2E7F)
        {0x2E00, true, "supplemental punct start"},
        {0x2E7F, true, "supplemental punct end"},
        // CJK symbols (0x3000-0x303F)
        {0x3000, true, "ideographic space"},
        {0x3001, true, "ideographic comma"},
        {0x303F, true, "CJK end"},
        // Full width ASCII punctuations
        {0xFF01, true, "fullwidth !"},
        {0xFF0F, true, "fullwidth /"},
        {0xFF1A, true, "fullwidth :"},
        {0xFF3B, true, "fullwidth ["},
        {0xFF5B, true, "fullwidth {"},
        // Outside ranges
        {0x1FFF, false, "below general punct"},
        {0x2070, false, "above general punct"},
        {0x2DFF, false, "below supplemental punct"},
        {0x2E80, false, "above supplemental punct"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int cc = test_cases[i].cc;
        bool c_result = c_utf_eat_space(cc);
        bool rs_result = (rs_utf_eat_space(cc) != 0);

        char name[128];
        snprintf(name, sizeof(name), "eat_space(0x%X %s) expected=%d C=%d Rust=%d",
                 cc, test_cases[i].desc, test_cases[i].expected, c_result, rs_result);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

void test_utf_allow_break_before(void) {
    printf("Testing utf_allow_break_before:\n");

    struct {
        int cc;
        bool expected;
        const char *desc;
    } test_cases[] = {
        // Regular characters - break allowed
        {'a', true, "ASCII 'a'"},
        {' ', true, "space"},
        {'(', true, "open paren"},
        // Prohibited characters - break NOT allowed before
        {'!', false, "exclamation"},
        {')', false, "close paren"},
        {',', false, "comma"},
        {':', false, "colon"},
        {';', false, "semicolon"},
        {'?', false, "question"},
        {']', false, "close bracket"},
        {'}', false, "close brace"},
        // CJK punctuation
        {0x3001, false, "ideographic comma"},
        {0x3002, false, "ideographic period"},
        {0xFF01, false, "fullwidth !"},
        {0xFF09, false, "fullwidth )"},
        // Allowed before
        {0x3000, true, "ideographic space"},
        {0x4E00, true, "CJK unified"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int cc = test_cases[i].cc;
        bool c_result = c_utf_allow_break_before(cc);
        bool rs_result = (rs_utf_allow_break_before(cc) != 0);

        char name[128];
        snprintf(name, sizeof(name), "break_before(0x%X %s) expected=%d C=%d Rust=%d",
                 cc, test_cases[i].desc, test_cases[i].expected, c_result, rs_result);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

void test_utf_allow_break_after(void) {
    printf("Testing utf_allow_break_after:\n");

    struct {
        int cc;
        bool expected;
        const char *desc;
    } test_cases[] = {
        // Regular characters - break allowed
        {'a', true, "ASCII 'a'"},
        {' ', true, "space"},
        {')', true, "close paren"},
        // Prohibited characters - break NOT allowed after
        {'(', false, "open paren"},
        {'<', false, "less than"},
        {'[', false, "open bracket"},
        {'`', false, "backtick"},
        {'{', false, "open brace"},
        // CJK opening brackets
        {0x3008, false, "left angle bracket"},
        {0x300A, false, "left double angle"},
        {0x300C, false, "left corner bracket"},
        {0xFF08, false, "fullwidth ("},
        {0xFF3B, false, "fullwidth ["},
        {0xFF5B, false, "fullwidth {"},
        // Allowed after
        {0x3009, true, "right angle bracket"},
        {0x4E00, true, "CJK unified"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int cc = test_cases[i].cc;
        bool c_result = c_utf_allow_break_after(cc);
        bool rs_result = (rs_utf_allow_break_after(cc) != 0);

        char name[128];
        snprintf(name, sizeof(name), "break_after(0x%X %s) expected=%d C=%d Rust=%d",
                 cc, test_cases[i].desc, test_cases[i].expected, c_result, rs_result);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

int main(void) {
    printf("=== Comparing C and Rust mbyte implementations ===\n\n");

    test_utf_char2len();
    test_utf_char2bytes();
    test_utf_byte2len();
    test_utf_ptr2len();
    test_utf_ptr2len_len();
    test_utf_ptr2char();
    test_roundtrip();
    test_utf_valid_string();
    test_utf_eat_space();
    test_utf_allow_break_before();
    test_utf_allow_break_after();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
