// Test main and OUTS

typedef int Foo;

int main() {
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
