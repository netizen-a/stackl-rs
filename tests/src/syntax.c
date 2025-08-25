// Test for declarator syntax

int main() {
    int a;
    int *b;
    int *c[3];
    int (*d)[3];
    int *f();
    int (*g)(void);
    int (*const h[3])(unsigned int, ...);
}
