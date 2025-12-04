// Test that Rust path functions match C implementations
// Compile: cc -o /tmp/compare_path src/nvim-rs/test/compare_path.c -L target/release -lnvim_rs -lpthread -ldl -lm
// Run: /tmp/compare_path

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Rust implementations
extern int rs_vim_ispathsep(int c);
extern int rs_vim_ispathsep_nocolon(int c);
extern int rs_vim_ispathlistsep(int c);
extern int rs_path_head_length(void);
extern int rs_path_is_absolute(const char *path);
extern int rs_path_is_url(const char *p);

// C implementations (from nvim/path.c, for Unix)
static bool c_vim_ispathsep(int c) {
    return c == '/';
}

static bool c_vim_ispathsep_nocolon(int c) {
    return c_vim_ispathsep(c);  // Same on Unix
}

static bool c_vim_ispathlistsep(int c) {
    return c == ':';
}

static int c_path_head_length(void) {
    return 1;  // Unix value
}

static bool c_path_is_absolute(const char *fname) {
    return *fname == '/' || *fname == '~';
}

// path_is_url in C checks if string starts with ":/" or ":\\"
static int c_path_is_url(const char *p) {
    if (strncmp(p, ":/", 2) == 0) {
        return 1;  // URL_SLASH
    } else if (strncmp(p, ":\\\\", 3) == 0) {
        return 2;  // URL_BACKSLASH
    }
    return 0;
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

void test_ispathsep(void) {
    printf("Testing vim_ispathsep:\n");

    int test_chars[] = {'/', '\\', ':', 'a', ' ', 0, -1, 255};
    int n = sizeof(test_chars) / sizeof(test_chars[0]);

    for (int i = 0; i < n; i++) {
        int c = test_chars[i];
        bool c_result = c_vim_ispathsep(c);
        bool rs_result = rs_vim_ispathsep(c) != 0;

        char name[64];
        snprintf(name, sizeof(name), "ispathsep(%d) C=%d Rust=%d", c, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_ispathsep_nocolon(void) {
    printf("Testing vim_ispathsep_nocolon:\n");

    int test_chars[] = {'/', '\\', ':', 'a', ' ', 0};
    int n = sizeof(test_chars) / sizeof(test_chars[0]);

    for (int i = 0; i < n; i++) {
        int c = test_chars[i];
        bool c_result = c_vim_ispathsep_nocolon(c);
        bool rs_result = rs_vim_ispathsep_nocolon(c) != 0;

        char name[64];
        snprintf(name, sizeof(name), "ispathsep_nocolon(%d) C=%d Rust=%d", c, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_ispathlistsep(void) {
    printf("Testing vim_ispathlistsep:\n");

    int test_chars[] = {':', ';', '/', 'a', 0};
    int n = sizeof(test_chars) / sizeof(test_chars[0]);

    for (int i = 0; i < n; i++) {
        int c = test_chars[i];
        bool c_result = c_vim_ispathlistsep(c);
        bool rs_result = rs_vim_ispathlistsep(c) != 0;

        char name[64];
        snprintf(name, sizeof(name), "ispathlistsep(%d) C=%d Rust=%d", c, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_path_head_length(void) {
    printf("Testing path_head_length:\n");

    int c_result = c_path_head_length();
    int rs_result = rs_path_head_length();

    char name[64];
    snprintf(name, sizeof(name), "path_head_length C=%d Rust=%d", c_result, rs_result);
    TEST(name, c_result == rs_result);
}

void test_path_is_absolute(void) {
    printf("Testing path_is_absolute:\n");

    const char *test_paths[] = {
        "/home/user",
        "~/documents",
        "home/user",
        "./file",
        "../file",
        "file.txt",
        "",
        "/",
        "~",
    };
    int n = sizeof(test_paths) / sizeof(test_paths[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_paths[i];
        bool c_result = c_path_is_absolute(path);
        bool rs_result = rs_path_is_absolute(path) != 0;

        char name[128];
        snprintf(name, sizeof(name), "is_absolute(\"%s\") C=%d Rust=%d", path, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_path_is_url(void) {
    printf("Testing path_is_url:\n");

    const char *test_strs[] = {
        "://example.com",       // URL_SLASH
        ":\\\\server\\share",   // URL_BACKSLASH
        ":foo",                 // no match
        "/home/user",           // no match
        ":",                    // no match (needs at least :/)
        "",                     // no match
    };
    int n = sizeof(test_strs) / sizeof(test_strs[0]);

    for (int i = 0; i < n; i++) {
        const char *s = test_strs[i];
        int c_result = c_path_is_url(s);
        int rs_result = rs_path_is_url(s);

        char name[128];
        snprintf(name, sizeof(name), "path_is_url(\"%s\") C=%d Rust=%d", s, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

int main(void) {
    printf("=== Comparing C and Rust path implementations ===\n\n");

    test_ispathsep();
    test_ispathsep_nocolon();
    test_ispathlistsep();
    test_path_head_length();
    test_path_is_absolute();
    test_path_is_url();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
