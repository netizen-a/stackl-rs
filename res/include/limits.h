// Copyright (c) 2024-2026 Jonathan A. Thomason
#ifndef _LIMITS_H_
#define _LIMITS_H_

#define CHAR_BIT   (8)
#define SCHAR_MIN  (-127)                                     // -(2^7 - 1)
#define SCHAR_MAX  (+127)                                     // 2^7 - 1
#define UCHAR_MAX  (255)                                      // 2^8 - 1
#define CHAR_MIN   SCHAR_MIN
#define CHAR_MAX   SCHAR_MAX
#define MB_LEN_MAX (1)
#define SHRT_MIN   (-2147483647)                              // −(2^31 − 1)
#define SHRT_MAX   (+2147483647)                              // 2^31 − 1
#define USHRT_MAX  (4294967295)                               // 2^32 − 1
#define INT_MIN    (-2147483647)                              // −(2^31 − 1)
#define INT_MAX    (+2147483647)                              // 2^31 − 1
#define UINT_MAX   (4294967295)                               // 2^32 − 1
#define LONG_MIN   (-9223372036854775807)                     // −(2^63 − 1)
#define LONG_MAX   (+9223372036854775807)                     // 2^63 − 1
#define ULONG_MAX  (18446744073709551615)                     // 2^64 - 1
#define LLONG_MIN  (-170141183460469231731687303715884105727) // −(2^127 − 1)
#define LLONG_MAX  (170141183460469231731687303715884105727)  // 2^127 − 1
#define ULLONG_MAX (340282366920938463463374607431768211455)  // 2^128 - 1

#endif
