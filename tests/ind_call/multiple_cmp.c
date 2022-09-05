// Adapted from
// https://stackoverflow.com/a/44699230
#include "stdlib.h"
#include "stdio.h"

void example_fun3(void)
{
    printf("Example Fun 3\n");
}

void example_fun2(void)
{
    printf("Example Fun 2\n");
}

void example_fun(int param)
{
    printf("Example Fun\n");

    void (*fp) (void);
    int param2 = 10;

    if(param == 10){
        fp = example_fun3;
    }

    if(param2 == 10){
        fp = example_fun2;
    }
    (*fp)();
}

int main(void)
{
    int i = 0;

    if(i == 0) {
     example_fun(10);
    }
}

