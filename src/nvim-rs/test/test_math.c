// Test that Rust math functions can be called from C
#include <stdio.h>
#include <stdint.h>
#include <assert.h>

// Declare the Rust functions we want to test
extern int rs_xctz(uint64_t x);
extern unsigned int rs_xpopcount(uint64_t x);
extern int rs_xfpclassify(double d);
extern int rs_xisinf(double d);
extern int rs_xisnan(double d);
extern int rs_trim_to_int(int64_t x);

int main(void) {
    printf("Testing Rust math functions from C...\n");

    // Test rs_xctz
    assert(rs_xctz(0) == 64);
    assert(rs_xctz(1) == 0);
    assert(rs_xctz(2) == 1);
    assert(rs_xctz(8) == 3);
    printf("  rs_xctz: PASS\n");

    // Test rs_xpopcount
    assert(rs_xpopcount(0) == 0);
    assert(rs_xpopcount(1) == 1);
    assert(rs_xpopcount(0xFF) == 8);
    assert(rs_xpopcount(0xFFFFFFFFFFFFFFFFULL) == 64);
    printf("  rs_xpopcount: PASS\n");

    // Test rs_xfpclassify (FP_NORMAL=4, FP_ZERO=2, FP_INFINITE=1, FP_NAN=0)
    assert(rs_xfpclassify(1.0) == 4);  // FP_NORMAL
    assert(rs_xfpclassify(0.0) == 2);  // FP_ZERO
    printf("  rs_xfpclassify: PASS\n");

    // Test rs_xisinf
    assert(rs_xisinf(1.0) == 0);
    assert(rs_xisinf(1.0/0.0) == 1);
    printf("  rs_xisinf: PASS\n");

    // Test rs_xisnan
    assert(rs_xisnan(1.0) == 0);
    assert(rs_xisnan(0.0/0.0) == 1);
    printf("  rs_xisnan: PASS\n");

    // Test rs_trim_to_int
    assert(rs_trim_to_int(100) == 100);
    assert(rs_trim_to_int(-100) == -100);
    assert(rs_trim_to_int(3000000000LL) == 2147483647);  // INT_MAX
    assert(rs_trim_to_int(-3000000000LL) == -2147483648); // INT_MIN
    printf("  rs_trim_to_int: PASS\n");

    printf("\nAll Rust math function tests PASSED!\n");
    return 0;
}
