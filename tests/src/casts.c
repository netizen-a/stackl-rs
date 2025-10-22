int foo(void) {
    int x = 5;
    int * y = &x;
    float t = *y;
    t += 4;
    return (int)t;
}
