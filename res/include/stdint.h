// Copyright (c) 2024-2026 Jonathan A. Thomason

#ifndef _STDINT_H_
#define _STDINT_H_

// C99 Exact-width integer types
typedef signed char        int8_t;
typedef unsigned char      uint8_t;
typedef signed int         int32_t;
typedef unsigned int       uint32_t;
typedef signed long        int64_t;
typedef unsigned long      uint64_t;
typedef signed long long   int128_t;
typedef unsigned long long uint128_t;

// C99 Minimum-width integer types
typedef signed char        int_least8_t;
typedef unsigned char      uint_least8_t;
typedef signed int         int_least16_t;
typedef unsigned int       uint_least16_t;
typedef signed int         int_least32_t;
typedef unsigned int       uint_least32_t;
typedef signed long        int_least64_t;
typedef unsigned long      uint_least64_t;
typedef signed long long   int_least128_t;
typedef unsigned long long uint_least128_t;

// C99 Fastest minimum-width integer types
typedef signed char        int_fast8_t;
typedef unsigned char      uint_fast8_t;
typedef signed int         int_fast16_t;
typedef unsigned int       uint_fast16_t;
typedef signed int         int_fast32_t;
typedef unsigned int       uint_fast32_t;
typedef signed long        int_fast64_t;
typedef unsigned long      uint_fast64_t;
typedef signed long long   int_fast128_t;
typedef unsigned long long uint_fast128_t;

#endif
