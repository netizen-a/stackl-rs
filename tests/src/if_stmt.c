// dangling else problem
void foo(void) {
    int x, y;
    if (1)
        if (0)
            x;
        else
            y;
}

struct Foo {
    int x;
    int **y;
};



int main() {
    struct Foo { int x; } k;
    int (*bar)() = k;
    foo = bar;
}


