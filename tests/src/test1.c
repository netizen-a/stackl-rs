// Test main and OUTS

int main() {
    {
        typedef int Foo;
        Foo x;
    }
    int Foo;
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
