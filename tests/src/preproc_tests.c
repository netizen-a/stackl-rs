// #undef __DATE__
// #undef __FILE__
// #undef __LINE__
// #undef __STDC__

// #define __DATE__
// #define __FILE__
// #define __LINE__
#define __STDC__

// #undef FOO bar baz 128 "so many extra tokens"
// #include <foo.h> and even more 128 right here ???
// #line 5 "hello world!"
// unsigned struct Foo {} foo;

// #line 5 "hello world!"

// unsigned struct Bar {} bar;

#line 8 "different source" "invalid string"

signed struct Baz {} baz;

// #error "this is an error. Hello world!" "hello"

// int foo () {

