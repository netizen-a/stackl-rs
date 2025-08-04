// Test main and OUTS

int main() {
    asm("OUTS" :: "g" ("Hello world\n"));
    return 0;
}
