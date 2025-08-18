// Test main and OUTS

int main() {
    static static int x;
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
