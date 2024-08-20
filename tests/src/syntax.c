// Test for declarator syntax

// #include "example_header.h"

float ff = 5e5;
int a = 1 + 1;
int *b;
int *c[3];
int (*d)[*];
int *f();
int (*g)(void);
int (*const h[3])(unsigned int, ...);

// typedef int Foo;
// Foo bar;
// int foo;

// static static int foo(void);

// unsigned signed unsigned bar;

// signed float x;
// unsigned float y;

// signed double xx;
// unsigned double yy;

// float signed xxx;
// float unsigned yyy;

// double signed zz;
// double unsigned aa;

// void void x;

// int void y;

// long char z;
// short long j;

// char long long j;

// unsigned long long int *const p;
// signed long long int p1, *const *restrict k;

// unsigned _Bool k;
// _Bool i;
// _Bool signed kk;
// long _Bool ttt;
// _Bool long long rrr;

// long float v;
// long long long double;

// struct Bar {
//     int x;
//     int y[*];
// };

// struct Barz z;

// struct Fooz signed i;

// unsigned struct FooFoo { int x; } j;
// unsigned struct FooBar { int y; } k;

// struct Baz { unsigned char x; } signed xx;
// struct FooBaz { unsigned char x; } unsigned yy;

// union Bark {
//     int zz;
//     int hh[*];
// };

// union Blap s_bar;

// union Bloop signed bloop;

// unsigned union Car { int x; } car;
// unsigned union Cat { int y; } cat;

// union Food { unsigned char x; } signed food;
// union Farm { unsigned char x; } unsigned farm;

// long union Fun foo;
// struct Fuun long long bar;

// auto x;
// struct Foo {
//     const x: 8;
//     int x;
// } x;


// int bar(const a, int b)
// {
//     return a + b;
// }

// int (*foo(a, b))(int, int)
//     int a, b;
// {
//     return bar;
// }

// int foo {
//     return 0;
// }

// typedef invalid_type invalid_t;
// typedef float int;

// int foo(x)(x, y);

// int x[*];

// int foo(int x[*]) {}

// int bar(int x[*]);

// int baz(int (*x)(int y[*])) {}

// static void foo(void) {}
// extern void bar(void) {}
// auto void baz(void) {}
// typedef void bap(void) {}
// register void rap(void) {}

int far(int) {}



