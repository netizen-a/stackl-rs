// dangling else problem
void foo(int);

auto void foo(void) {
    int x, y;
    if (1 == 1)
        if (0)
            x;
        else
            y;
}
int z = 5;
int k = z + 4;


