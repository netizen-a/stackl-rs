// Test main and OUTS

int main() {
    const char *s = "Hello world\n";
    asm ("PUSHVAR %0\n"
        "OUTS"
        :
        : "p" (s));
    return 0;
}
