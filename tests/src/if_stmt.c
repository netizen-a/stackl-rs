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
    const struct Foo { int x; } k;
    int (*const volatile bar)(int,int) = k;
    foo = bar;
    // int **ptr0;
    // const int *const *const ptr1;
    // ptr1 = ptr0;
}


