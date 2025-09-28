union FooBar {
    int x;
    float y;
};

struct Foo {
    int x: 33;
    float y: 22;
};

struct Bar {
    struct Foo: 1;
    int k: x;
};
