// Test for declarator syntax

const int a;
int *b, *p;
int *c[3];
int (*d)[3];
int *f();
int (*g)(void);
int (*const h[3])(unsigned int, ...);

int foo(restrict int x) {
    return 0;
}
