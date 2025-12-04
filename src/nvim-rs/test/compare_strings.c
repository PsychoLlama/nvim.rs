// Test that Rust string functions match C implementations
// Compile: cc -o /tmp/compare_strings src/nvim-rs/test/compare_strings.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_strings

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>

// Rust implementations
extern int rs_vim_strnicmp(const char *s1, const char *s2, size_t len);
extern int rs_has_non_ascii(const char *s);
extern void rs_sort_strings(char **files, int count);

// C implementations (ASCII-only versions from nvim/strings.c)
static inline int TOLOWER_ASC(int c) {
    return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c;
}

static int c_vim_strnicmp_asc(const char *s1, const char *s2, size_t len) {
    int i = 0;
    while (len > 0) {
        i = TOLOWER_ASC(*s1) - TOLOWER_ASC(*s2);
        if (i != 0) {
            break;
        }
        if (*s1 == '\0') {
            break;
        }
        s1++;
        s2++;
        len--;
    }
    return i;
}

static bool c_has_non_ascii(const char *s) {
    if (s != NULL) {
        for (const char *p = s; *p != '\0'; p++) {
            if ((unsigned char)(*p) >= 128) {
                return true;
            }
        }
    }
    return false;
}

static int c_sort_compare(const void *s1, const void *s2) {
    return strcmp(*(char **)s1, *(char **)s2);
}

static void c_sort_strings(char **files, int count) {
    qsort((void *)files, (size_t)count, sizeof(char *), c_sort_compare);
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

void test_vim_strnicmp_asc(void) {
    printf("Testing vim_strnicmp_asc:\n");

    struct {
        const char *s1;
        const char *s2;
        size_t len;
    } test_cases[] = {
        {"Hello", "HELLO", 5},
        {"Hello", "HELLO", 3},
        {"Hello", "World", 5},
        {"abc", "ABC", 3},
        {"abc", "abd", 2},
        {"", "", 0},
        {"a", "b", 1},
        {"A", "a", 1},
        {"test123", "TEST123", 7},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        int c_result = c_vim_strnicmp_asc(test_cases[i].s1, test_cases[i].s2, test_cases[i].len);
        int rs_result = rs_vim_strnicmp(test_cases[i].s1, test_cases[i].s2, test_cases[i].len);

        // Compare signs (0, <0, >0) rather than exact values
        int c_sign = (c_result > 0) - (c_result < 0);
        int rs_sign = (rs_result > 0) - (rs_result < 0);

        char name[128];
        snprintf(name, sizeof(name), "strnicmp_asc(\"%s\", \"%s\", %zu) C=%d Rust=%d",
                 test_cases[i].s1, test_cases[i].s2, test_cases[i].len, c_sign, rs_sign);
        TEST(name, c_sign == rs_sign);
    }
}

void test_has_non_ascii(void) {
    printf("Testing has_non_ascii:\n");

    const char *test_strs[] = {
        "Hello, World!",
        "",
        "abc123",
        "Hello\x80World",  // Contains 0x80 (non-ASCII)
        "\xff",            // Contains 0xff (non-ASCII)
        "ASCII only",
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        bool c_result = c_has_non_ascii(test_strs[i]);
        bool rs_result = rs_has_non_ascii(test_strs[i]) != 0;

        char name[128];
        // Don't print non-ASCII chars directly
        snprintf(name, sizeof(name), "has_non_ascii(test_%d) C=%d Rust=%d", i, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }

    // Test NULL
    bool c_result = c_has_non_ascii(NULL);
    bool rs_result = rs_has_non_ascii(NULL) != 0;
    char name[64];
    snprintf(name, sizeof(name), "has_non_ascii(NULL) C=%d Rust=%d", c_result, rs_result);
    TEST(name, c_result == rs_result);
}

void test_sort_strings(void) {
    printf("Testing sort_strings:\n");

    // Test 1: Basic sorting
    {
        char *c_arr[] = {"zebra", "apple", "mango"};
        char *rs_arr[] = {"zebra", "apple", "mango"};

        c_sort_strings(c_arr, 3);
        rs_sort_strings(rs_arr, 3);

        bool match = true;
        for (int i = 0; i < 3; i++) {
            if (strcmp(c_arr[i], rs_arr[i]) != 0) {
                match = false;
                break;
            }
        }
        TEST("sort_strings basic (3 elements)", match);
    }

    // Test 2: Already sorted
    {
        char *c_arr[] = {"a", "b", "c"};
        char *rs_arr[] = {"a", "b", "c"};

        c_sort_strings(c_arr, 3);
        rs_sort_strings(rs_arr, 3);

        bool match = true;
        for (int i = 0; i < 3; i++) {
            if (strcmp(c_arr[i], rs_arr[i]) != 0) {
                match = false;
                break;
            }
        }
        TEST("sort_strings already sorted", match);
    }

    // Test 3: Reverse sorted
    {
        char *c_arr[] = {"z", "m", "a"};
        char *rs_arr[] = {"z", "m", "a"};

        c_sort_strings(c_arr, 3);
        rs_sort_strings(rs_arr, 3);

        bool match = true;
        for (int i = 0; i < 3; i++) {
            if (strcmp(c_arr[i], rs_arr[i]) != 0) {
                match = false;
                break;
            }
        }
        TEST("sort_strings reverse sorted", match);
    }

    // Test 4: Single element
    {
        char *c_arr[] = {"single"};
        char *rs_arr[] = {"single"};

        c_sort_strings(c_arr, 1);
        rs_sort_strings(rs_arr, 1);

        TEST("sort_strings single element", strcmp(c_arr[0], rs_arr[0]) == 0);
    }

    // Test 5: Empty (count=0)
    {
        char *arr[] = {"test"};
        rs_sort_strings(arr, 0);  // Should be a no-op
        TEST("sort_strings empty (count=0)", strcmp(arr[0], "test") == 0);
    }
}

int main(void) {
    printf("=== Comparing C and Rust string implementations ===\n\n");

    test_vim_strnicmp_asc();
    test_has_non_ascii();
    test_sort_strings();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
