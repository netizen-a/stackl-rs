// dangling else problem
void foo(void) {
    int x, y;
    int y;
    if (1 == 1)
        if (0)
            x;
        else
            y;

}


