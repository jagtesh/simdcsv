#ifndef PORTABILITY_H
#define PORTABILITY_H

#ifdef _MSC_VER
/* Microsoft C/C++-compatible compiler */
#include <intrin.h>
#include <iso646.h>
#include <cstdint>

static inline bool add_overflow(uint64_t value1, uint64_t value2, uint64_t *result) {
	return _addcarry_u64(0, value1, value2, reinterpret_cast<unsigned __int64 *>(result));
}

#pragma intrinsic(_umul128)
static inline bool mul_overflow(uint64_t value1, uint64_t value2, uint64_t *result) {
	uint64_t high;
	*result = _umul128(value1, value2, &high);
	return high;
}


static inline int trailingzeroes(uint64_t input_num) {
    return _tzcnt_u64(input_num);
}

static inline int leadingzeroes(uint64_t  input_num) {
    return _lzcnt_u64(input_num);
}

static inline int hamming(uint64_t input_num) {
#ifdef _WIN64  // highly recommended!!!
	return (int)__popcnt64(input_num);
#else  // if we must support 32-bit Windows
	return (int)(__popcnt((uint32_t)input_num) +
		__popcnt((uint32_t)(input_num >> 32)));
#endif
}

#else
#include <cstdint>
#include <cstdlib>

#if defined(__BMI2__) || defined(__POPCOUNT__) || defined(__AVX2__)
#include <x86intrin.h>
#endif

#ifdef __ARM_NEON
#include <arm_neon.h>

static inline uint16_t neonmovemask(uint8x16_t input) {
    const uint8x16_t bit_mask = {1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128};
    uint8x16_t t0 = vandq_u8(input, bit_mask);
    uint8x8_t low = vget_low_u8(t0);
    uint8x8_t high = vget_high_u8(t0);
    return (uint16_t)vaddv_u8(low) | ((uint16_t)vaddv_u8(high) << 8);
}

static inline uint64_t neonmovemask_bulk(uint8x16_t i0, uint8x16_t i1, uint8x16_t i2, uint8x16_t i3) {
    return (uint64_t)neonmovemask(i0) | ((uint64_t)neonmovemask(i1) << 16) | ((uint64_t)neonmovemask(i2) << 32) | ((uint64_t)neonmovemask(i3) << 48);
}
#endif

static inline bool add_overflow(uint64_t  value1, uint64_t  value2, uint64_t *result) {
	return __builtin_uaddll_overflow(value1, value2, (unsigned long long*)result);
}
static inline bool mul_overflow(uint64_t  value1, uint64_t  value2, uint64_t *result) {
	return __builtin_umulll_overflow(value1, value2, (unsigned long long *)result);
}

/* result might be undefined when input_num is zero */
static inline int trailingzeroes(uint64_t input_num) {
#ifdef __BMI2__
	return _tzcnt_u64(input_num);
#else
	return __builtin_ctzll(input_num);
#endif
}

/* result might be undefined when input_num is zero */
static inline int leadingzeroes(uint64_t  input_num) {
#ifdef __BMI2__
	return _lzcnt_u64(input_num);
#else
	return __builtin_clzll(input_num);
#endif
}

/* result might be undefined when input_num is zero */
static inline int hamming(uint64_t input_num) {
#ifdef __POPCOUNT__
	return _popcnt64(input_num);
#else
	return __builtin_popcountll(input_num);
#endif
}

#endif // _MSC_VER


#endif // _PORTABILITY_H
