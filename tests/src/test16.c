// This tests the type of logical operators.
int main()
{
    char *msg;
    int *localInt;
    int bp;
    int lp;

    if ((int)msg < 0 || (int)msg >= (lp - bp) || (int)localInt ) 
        lp = 1;

    asm("OUTS", "Bug in type of logical ops is fixed\n");

    return 0;
}
