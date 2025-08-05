// Test main and OUTS

int main() {
    label: thing:
    asm ("OUTS" :: "g" ("Hello world\n"));
    //char * s = "hello" "world";
    return 0;
}
