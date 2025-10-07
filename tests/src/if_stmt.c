// dangling else problem
void foo(int);

void foo(void) {
    int x, y;
    if (1 == 1)
        if (0)
            x;
        else
            y;
}


