// Adapted from UAFuzz example
// https://github.com/strongcourage/uafuzz/blob/master/tests/example/example.c
# include <stdio.h>
# include <stdlib.h>
# include <string.h>
# include <unistd.h>
# include <fcntl.h>

int *p, *p_alias;
char buf[10];

void good() 
{
    if (buf[1] == 'F'){
        printf("Test\n");
    }
}

void bad_func() 
{
    void (*fp) (void);
    if(buf[0]  == 'F') {
  	  abort();
	  fp = &good;
    }
    else {
	  fp = &good;
    }
	(*fp)();
}

int main (int argc, char *argv[]) 
{
    int f = open(argv[1], O_RDONLY);
    read(f, buf, 10);
    p = malloc(sizeof(int));
    void (*fp) (void);

    if (buf[0] == 'A') {
        p_alias = malloc(sizeof(int));   
        p = p_alias;    
	    fp = &good;
    }
    else if (buf[1] == 'F')
        fp = &bad_func;
    else if (buf[2] == 'U')
        fp = &good;
    else {
	    fp = &good;
    }
    (*fp)();
}
