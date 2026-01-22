struct Bar {
    int x;
};

struct Foo {
    struct Bar bar;
};

int foo(void) {
    struct Foo foo;
    foo.bar.x = 5;
}
