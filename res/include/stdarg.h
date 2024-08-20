// Copyright (c) 2024-2026 Jonathan A. Thomason

#ifndef _STDARG_H_
#define _STDARG_H_

// reference impl by Local
#include <stddef.h>
typedef int* va_list;

void* __va_arg_advance(va_list* v, size_t type_size) {
    void* result = *v;
    (*v) += type_size / sizeof(int);
    return result;
}

#define va_start(V, A) ((V) = &(A) + 1)
#define va_end(V) ((V) = (int*)0)
#define va_copy(V0, V1) ((V0) = (V1))
#define va_arg(V, T) (*(T*)(__va_arg_advance(V, sizeof(T))))

#endif
