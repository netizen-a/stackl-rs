// dangling else problem

void foo(void) {
    int x, z;
    if (1 == 1)
        if (0)
            x;
        else
            z;
}


