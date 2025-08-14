// Test main and OUTS

int main() {
    char* static k = "foo";
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
