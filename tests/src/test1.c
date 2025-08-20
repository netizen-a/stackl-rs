// Test main and OUTS

struct { int x; } k;

int main() {
    const char* s = "Hello world\n";
    asm ("OUTS\n" :: "m" (s));
    return 0;
}
