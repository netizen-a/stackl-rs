struct Bar {
    int x;
    int y;
};

struct Foo {
    struct Bar bar;
    struct {
        int k, v;
    } foobar;
};

int foo(void) {
    struct Foo foo;
    foo.bar.x = 5;
}
