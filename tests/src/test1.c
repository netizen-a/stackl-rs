// Test main and OUTS

int main() {
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
