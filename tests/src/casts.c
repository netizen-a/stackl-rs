void test_casts(void) {
    // Integer widening
    int a = (int)((short)1 + (char)2);
    unsigned long b = (unsigned long)((unsigned short)3 + (unsigned char)4);

    // Integer truncation
    short c = (short)1000;           // int → short
    unsigned int d = (unsigned int)10000000000UL; // long → unsigned int

    // Integer → float
    float f1 = 42;     // int → float
    double f2 = 123U;  // unsigned int → double

    // Float → integer
    int i1 = 3.14f;    // float → int
    unsigned long i2 = 2.718; // double → unsigned long

    // Float widening/truncation
    double d1 = 1.0f;    // float → double
    float f3 = 2.0;      // double → float

    // Bool ↔ integer
    int b2 = 1 < 2;   // bool → int
    _Bool b3 = 42;    // int → bool
}
