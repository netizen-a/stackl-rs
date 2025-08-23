// Test main and OUTS

struct { int x; } k;

int main() {
    int a;
    int *b;
    int *c[3];
    int (*d)[3];
    int *f();
    int (*g)(void);
    //int (*const h[])(unsigned int, ...) = {};
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
