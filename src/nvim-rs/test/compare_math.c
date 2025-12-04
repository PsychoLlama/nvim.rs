// Compare Rust math functions against the original C implementation
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <limits.h>
#include <math.h>

// Rust implementations
extern int rs_xctz(uint64_t x);
extern unsigned int rs_xpopcount(uint64_t x);
extern int rs_xfpclassify(double d);
extern int rs_xisinf(double d);
extern int rs_xisnan(double d);
extern int rs_trim_to_int(int64_t x);
extern int rs_vim_append_digit_int(int *value, int digit);

// Original C implementations (copied from nvim/math.c)
int c_xfpclassify(double d) {
    uint64_t m;
    memcpy(&m, &d, sizeof(m));
    int e = 0x7ff & (m >> 52);
    m = 0xfffffffffffffULL & m;

    switch (e) {
    default:
        return FP_NORMAL;
    case 0x000:
        return m ? FP_SUBNORMAL : FP_ZERO;
    case 0x7ff:
        return m ? FP_NAN : FP_INFINITE;
    }
}

int c_xisinf(double d) {
    return FP_INFINITE == c_xfpclassify(d);
}

int c_xisnan(double d) {
    return FP_NAN == c_xfpclassify(d);
}

int c_xctz(uint64_t x) {
    if (x == 0) {
        return 8 * sizeof(x);
    }
#if defined(__clang__) || (defined(__GNUC__) && (__GNUC__ >= 4))
    return __builtin_ctzll(x);
#else
    int count = 0;
    x = (x ^ (x - 1)) >> 1;
    while (x) {
        count++;
        x >>= 1;
    }
    return count;
#endif
}

unsigned c_xpopcount(uint64_t x) {
#if defined(__clang__) || defined(__GNUC__)
    return (unsigned)__builtin_popcountll(x);
#else
    unsigned count = 0;
    for (; x != 0; x >>= 1) {
        if (x & 1) count++;
    }
    return count;
#endif
}

#define OK 1
#define FAIL 0

int c_vim_append_digit_int(int *value, int digit) {
    int x = *value;
    if (x > ((INT_MAX - digit) / 10)) {
        return FAIL;
    }
    *value = x * 10 + digit;
    return OK;
}

int c_trim_to_int(int64_t x) {
    return x > INT_MAX ? INT_MAX : x < INT_MIN ? INT_MIN : (int)x;
}

// Map our FP constants to libc's
int map_rs_fp(int rs_val) {
    // RS: FP_NAN=0, FP_INFINITE=1, FP_ZERO=2, FP_SUBNORMAL=3, FP_NORMAL=4
    // libc: FP_NAN, FP_INFINITE, FP_ZERO, FP_SUBNORMAL, FP_NORMAL (varies by system)
    switch(rs_val) {
        case 0: return FP_NAN;
        case 1: return FP_INFINITE;
        case 2: return FP_ZERO;
        case 3: return FP_SUBNORMAL;
        case 4: return FP_NORMAL;
        default: return -1;
    }
}

int main(void) {
    int errors = 0;
    printf("Comparing Rust implementations against C originals...\n\n");

    // Test xctz
    printf("Testing xctz...\n");
    for (int i = 0; i < 64; i++) {
        uint64_t val = 1ULL << i;
        int c_result = c_xctz(val);
        int rs_result = rs_xctz(val);
        if (c_result != rs_result) {
            printf("  MISMATCH: xctz(1<<%d) C=%d Rust=%d\n", i, c_result, rs_result);
            errors++;
        }
    }
    if (c_xctz(0) != rs_xctz(0)) {
        printf("  MISMATCH: xctz(0) C=%d Rust=%d\n", c_xctz(0), rs_xctz(0));
        errors++;
    }
    printf("  xctz: %s\n", errors == 0 ? "PASS" : "FAIL");

    // Test xpopcount
    int pop_errors = 0;
    printf("Testing xpopcount...\n");
    uint64_t pop_tests[] = {0, 1, 2, 0xFF, 0xFFFF, 0xFFFFFFFF, 0xFFFFFFFFFFFFFFFFULL, 0x5555555555555555ULL};
    for (size_t i = 0; i < sizeof(pop_tests)/sizeof(pop_tests[0]); i++) {
        unsigned c_result = c_xpopcount(pop_tests[i]);
        unsigned rs_result = rs_xpopcount(pop_tests[i]);
        if (c_result != rs_result) {
            printf("  MISMATCH: xpopcount(0x%llx) C=%u Rust=%u\n",
                   (unsigned long long)pop_tests[i], c_result, rs_result);
            pop_errors++;
        }
    }
    errors += pop_errors;
    printf("  xpopcount: %s\n", pop_errors == 0 ? "PASS" : "FAIL");

    // Test xfpclassify
    int fp_errors = 0;
    printf("Testing xfpclassify...\n");
    double fp_tests[] = {0.0, -0.0, 1.0, -1.0, 1e-310, 1.0/0.0, -1.0/0.0, 0.0/0.0};
    for (size_t i = 0; i < sizeof(fp_tests)/sizeof(fp_tests[0]); i++) {
        int c_result = c_xfpclassify(fp_tests[i]);
        int rs_result = map_rs_fp(rs_xfpclassify(fp_tests[i]));
        if (c_result != rs_result) {
            printf("  MISMATCH: xfpclassify(%g) C=%d Rust=%d\n", fp_tests[i], c_result, rs_result);
            fp_errors++;
        }
    }
    errors += fp_errors;
    printf("  xfpclassify: %s\n", fp_errors == 0 ? "PASS" : "FAIL");

    // Test xisinf
    int inf_errors = 0;
    printf("Testing xisinf...\n");
    for (size_t i = 0; i < sizeof(fp_tests)/sizeof(fp_tests[0]); i++) {
        int c_result = c_xisinf(fp_tests[i]);
        int rs_result = rs_xisinf(fp_tests[i]);
        if (c_result != rs_result) {
            printf("  MISMATCH: xisinf(%g) C=%d Rust=%d\n", fp_tests[i], c_result, rs_result);
            inf_errors++;
        }
    }
    errors += inf_errors;
    printf("  xisinf: %s\n", inf_errors == 0 ? "PASS" : "FAIL");

    // Test xisnan
    int nan_errors = 0;
    printf("Testing xisnan...\n");
    for (size_t i = 0; i < sizeof(fp_tests)/sizeof(fp_tests[0]); i++) {
        int c_result = c_xisnan(fp_tests[i]);
        int rs_result = rs_xisnan(fp_tests[i]);
        if (c_result != rs_result) {
            printf("  MISMATCH: xisnan(%g) C=%d Rust=%d\n", fp_tests[i], c_result, rs_result);
            nan_errors++;
        }
    }
    errors += nan_errors;
    printf("  xisnan: %s\n", nan_errors == 0 ? "PASS" : "FAIL");

    // Test trim_to_int
    int trim_errors = 0;
    printf("Testing trim_to_int...\n");
    int64_t trim_tests[] = {0, 1, -1, 100, -100, INT_MAX, INT_MIN,
                            (int64_t)INT_MAX + 1, (int64_t)INT_MIN - 1,
                            3000000000LL, -3000000000LL};
    for (size_t i = 0; i < sizeof(trim_tests)/sizeof(trim_tests[0]); i++) {
        int c_result = c_trim_to_int(trim_tests[i]);
        int rs_result = rs_trim_to_int(trim_tests[i]);
        if (c_result != rs_result) {
            printf("  MISMATCH: trim_to_int(%lld) C=%d Rust=%d\n",
                   (long long)trim_tests[i], c_result, rs_result);
            trim_errors++;
        }
    }
    errors += trim_errors;
    printf("  trim_to_int: %s\n", trim_errors == 0 ? "PASS" : "FAIL");

    // Test vim_append_digit_int
    int append_errors = 0;
    printf("Testing vim_append_digit_int...\n");
    struct { int start; int digit; } append_tests[] = {
        {0, 5}, {10, 3}, {100, 0}, {214748364, 7}, // Should succeed
        {214748365, 0}, {214748364, 8}, // Should fail (overflow)
    };
    for (size_t i = 0; i < sizeof(append_tests)/sizeof(append_tests[0]); i++) {
        int c_val = append_tests[i].start;
        int rs_val = append_tests[i].start;
        int c_result = c_vim_append_digit_int(&c_val, append_tests[i].digit);
        int rs_result = rs_vim_append_digit_int(&rs_val, append_tests[i].digit);
        if (c_result != rs_result || (c_result == OK && c_val != rs_val)) {
            printf("  MISMATCH: vim_append_digit_int(%d, %d) C=(%d,%d) Rust=(%d,%d)\n",
                   append_tests[i].start, append_tests[i].digit,
                   c_result, c_val, rs_result, rs_val);
            append_errors++;
        }
    }
    errors += append_errors;
    printf("  vim_append_digit_int: %s\n", append_errors == 0 ? "PASS" : "FAIL");

    printf("\n========================================\n");
    if (errors == 0) {
        printf("All comparisons PASSED! Rust matches C.\n");
        return 0;
    } else {
        printf("FAILED: %d mismatches found.\n", errors);
        return 1;
    }
}
