// Test for declarator syntax

// int a;
// int *b, *p;
// int *c[3];
// int (*d)[3];
// int *f();
// int (*g)(void);
// int (*const h[3])(unsigned int, ...);

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

// unsigned long long int p;
// signed long long int p1;

// unsigned _Bool k;
// _Bool i;
// _Bool signed kk;
// long _Bool ttt;
// _Bool long long rrr;

// long float v;
// long long long double;

struct Foo signed k;

unsigned struct Foo { int x; } kk;
unsigned struct Foo { int y; } ll;

struct Foo { unsigned char x; } signed k;

