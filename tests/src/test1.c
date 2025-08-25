// Test main and OUTS

int main() {
    asm ("PUSHVAR $1"
        "OUTS\n"
        :
        : "m" ("Hello world\n"));
    return 0;
}
