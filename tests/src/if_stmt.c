// dangling else problem
void foo(void) {
    int x, y;
    if (1)
        if (0)
            x;
        else
            y;
}
