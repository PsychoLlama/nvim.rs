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
extern int rs_is_path_head(const char *path);
extern const char *rs_get_past_head(const char *path);
extern int rs_path_is_absolute(const char *path);
extern int rs_path_is_url(const char *p);
extern const char *rs_path_tail(const char *fname);
extern int rs_path_has_drive_letter(const char *p, size_t path_len);
extern int rs_path_with_url(const char *fname);

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

// C implementation of is_path_head (Unix version)
static bool c_is_path_head(const char *path) {
    return c_vim_ispathsep(*path);
}

// C implementation of get_past_head (Unix version)
static const char *c_get_past_head(const char *path) {
    const char *retval = path;
    // On Unix, just skip past leading path separators
    while (c_vim_ispathsep(*retval)) {
        retval++;
    }
    return retval;
}

// C implementation of path_tail (Unix version, simplified)
static const char *c_path_tail(const char *fname) {
    static const char *empty = "";
    if (fname == NULL) {
        return empty;
    }
    // Skip leading slashes (simplified get_past_head for Unix)
    const char *tail = fname;
    while (*tail == '/') {
        tail++;
    }
    const char *p = tail;
    // Find last part of path
    while (*p != '\0') {
        if (*p == '/') {
            tail = p + 1;
        }
        p++;
    }
    return tail;
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

void test_is_path_head(void) {
    printf("Testing is_path_head:\n");

    const char *test_paths[] = {
        "/home/user",
        "/",
        "home/user",
        "./file",
        "../file",
        "file.txt",
        "",
        "~/file",
        "a",
    };
    int n = sizeof(test_paths) / sizeof(test_paths[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_paths[i];
        bool c_result = c_is_path_head(path);
        bool rs_result = rs_is_path_head(path) != 0;

        char name[128];
        snprintf(name, sizeof(name), "is_path_head(\"%s\") C=%d Rust=%d", path, c_result, rs_result);
        TEST(name, c_result == rs_result);
    }
}

void test_get_past_head(void) {
    printf("Testing get_past_head:\n");

    struct {
        const char *path;
        const char *expected;
    } test_cases[] = {
        {"/home/user", "home/user"},
        {"/", ""},
        {"///home", "home"},
        {"home/user", "home/user"},
        {"./file", "./file"},
        {"", ""},
        {"a", "a"},
        {"///", ""},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_cases[i].path;
        const char *expected = test_cases[i].expected;
        const char *c_result = c_get_past_head(path);
        const char *rs_result = rs_get_past_head(path);

        bool match = (strcmp(c_result, rs_result) == 0);
        char name[256];
        snprintf(name, sizeof(name), "get_past_head(\"%s\") C=\"%s\" Rust=\"%s\" expected=\"%s\"",
                 path, c_result, rs_result, expected);
        TEST(name, match && strcmp(c_result, expected) == 0);
    }
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

void test_path_tail(void) {
    printf("Testing path_tail:\n");

    struct {
        const char *path;
        const char *expected;
    } test_cases[] = {
        {"/home/user/file.txt", "file.txt"},
        {"/home/user/", ""},
        {"file.txt", "file.txt"},
        {"/", ""},
        {"", ""},
        {"///multiple/slashes///file", "file"},
        {"/a/b/c", "c"},
        {"relative/path/to/file", "file"},
        {".", "."},
        {"..", ".."},
        {"./file", "file"},
        {"../file", "file"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_cases[i].path;
        const char *expected = test_cases[i].expected;
        const char *c_result = c_path_tail(path);
        const char *rs_result = rs_path_tail(path);

        bool match = (strcmp(c_result, rs_result) == 0);
        char name[256];
        snprintf(name, sizeof(name), "path_tail(\"%s\") C=\"%s\" Rust=\"%s\" expected=\"%s\"",
                 path, c_result, rs_result, expected);
        TEST(name, match);
    }

    // Test NULL handling
    const char *null_c = c_path_tail(NULL);
    const char *null_rs = rs_path_tail(NULL);
    char name[128];
    snprintf(name, sizeof(name), "path_tail(NULL) C=\"%s\" Rust=\"%s\"", null_c, null_rs);
    TEST(name, strcmp(null_c, null_rs) == 0);
}

// C implementation of path_has_drive_letter
static bool c_path_has_drive_letter(const char *p, size_t path_len) {
    if (path_len < 2) return false;
    // ASCII_ISALPHA check
    char c0 = p[0];
    if (!((c0 >= 'A' && c0 <= 'Z') || (c0 >= 'a' && c0 <= 'z'))) return false;
    // Second char must be ':' or '|'
    if (p[1] != ':' && p[1] != '|') return false;
    // If only 2 chars, that's valid
    if (path_len == 2) return true;
    // Third char must be '/', '\', '?', or '#'
    return p[2] == '/' || p[2] == '\\' || p[2] == '?' || p[2] == '#';
}

// C implementation of path_with_url
static int c_path_with_url(const char *fname) {
    // First char must be alpha
    char c0 = fname[0];
    if (!((c0 >= 'A' && c0 <= 'Z') || (c0 >= 'a' && c0 <= 'z'))) {
        return 0;
    }

    // Check for drive letter
    if (c_path_has_drive_letter(fname, strlen(fname))) {
        return 0;
    }

    // Scan scheme body: alpha, digit, '+', '-', '.'
    const char *p = fname + 1;
    while ((*p >= 'A' && *p <= 'Z') || (*p >= 'a' && *p <= 'z') ||
           (*p >= '0' && *p <= '9') || *p == '+' || *p == '-' || *p == '.') {
        p++;
    }

    // Check last char is not '+', '-', or '.'
    if (p > fname + 1) {
        char last = p[-1];
        if (last == '+' || last == '-' || last == '.') {
            return 0;
        }
    }

    // Check for ":/" or ":\\"
    return c_path_is_url(p);
}

void test_path_has_drive_letter(void) {
    printf("Testing path_has_drive_letter:\n");

    struct {
        const char *path;
        bool expected;
        const char *desc;
    } test_cases[] = {
        // Valid drive letters
        {"C:/", true, "C:/"},
        {"D:\\", true, "D:\\"},
        {"c:/", true, "lowercase c:/"},
        {"Z:\\", true, "Z:\\"},
        {"C|/", true, "C|/ (pipe)"},
        {"C|\\", true, "C|\\ (pipe)"},
        {"C:?", true, "C:?"},
        {"C:#", true, "C:#"},
        {"C:", true, "C: (len=2)"},

        // Invalid drive letters
        {"C", false, "C (too short)"},
        {"1:/", false, "1:/ (not alpha)"},
        {"/home", false, "/home (starts with /)"},
        {"://", false, ":// (no alpha)"},
        {"C:x", false, "C:x (invalid third char)"},
        {"", false, "empty"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_cases[i].path;
        size_t len = strlen(path);
        bool c_result = c_path_has_drive_letter(path, len);
        bool rs_result = (rs_path_has_drive_letter(path, len) != 0);

        char name[128];
        snprintf(name, sizeof(name), "drive_letter(\"%s\") expected=%d C=%d Rust=%d",
                 test_cases[i].desc, test_cases[i].expected, c_result, rs_result);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

void test_path_with_url(void) {
    printf("Testing path_with_url:\n");

    struct {
        const char *path;
        int expected;  // 0, URL_SLASH (1), or URL_BACKSLASH (2)
        const char *desc;
    } test_cases[] = {
        // Valid URLs
        {"http://example.com", 1, "http://"},
        {"https://example.com", 1, "https://"},
        {"ftp://server/file", 1, "ftp://"},
        {"file:///path", 1, "file:///"},
        {"mailto://user@host", 1, "mailto://"},
        {"custom-scheme://foo", 1, "custom-scheme://"},
        {"a+b.c://", 1, "a+b.c://"},
        {"http:\\\\server", 2, "http:\\\\ (backslash)"},

        // Not URLs
        {"C:/path", 0, "C:/ (drive letter)"},
        {"C:\\path", 0, "C:\\ (drive letter)"},
        {"/home/user", 0, "/home (not alpha start)"},
        {"123://", 0, "123:// (not alpha start)"},
        {"http", 0, "http (no ://)"},
        {"http:", 0, "http: (no slash)"},
        {"http:/foo", 1, "http:/foo (single slash is still URL)"},
        {"+scheme://", 0, "+scheme (starts with +)"},
        {"scheme+://", 0, "scheme+ (ends with +)"},
        {"scheme-://", 0, "scheme- (ends with -)"},
        {"scheme.://", 0, "scheme. (ends with .)"},
    };
    int n = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < n; i++) {
        const char *path = test_cases[i].path;
        int c_result = c_path_with_url(path);
        int rs_result = rs_path_with_url(path);

        char name[128];
        snprintf(name, sizeof(name), "with_url(\"%s\") expected=%d C=%d Rust=%d",
                 test_cases[i].desc, test_cases[i].expected, c_result, rs_result);
        TEST(name, c_result == rs_result && c_result == test_cases[i].expected);
    }
}

int main(void) {
    printf("=== Comparing C and Rust path implementations ===\n\n");

    test_ispathsep();
    test_ispathsep_nocolon();
    test_ispathlistsep();
    test_path_head_length();
    test_is_path_head();
    test_get_past_head();
    test_path_is_absolute();
    test_path_is_url();
    test_path_tail();
    test_path_has_drive_letter();
    test_path_with_url();

    printf("\n=== Results ===\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
