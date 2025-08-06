// Test main and OUTS

int main() {
    asm ("OUTS" :: "m" ("Hello world\n"));
    return 0;
}
