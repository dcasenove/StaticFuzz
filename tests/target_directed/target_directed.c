# include <stdio.h>
# include <stdlib.h>
# include <string.h>
# include <unistd.h>
# include <fcntl.h>

char buf[12];

void target_fun_1(char buf[])
{
    if (buf[1] == 'a') {
        abort();
    }
}
void target_fun_2(char buf[])
{
    if (buf[1] == 'b') {
        abort();
    }
}
void target_fun_3(char buf[])
{
    if (buf[1] == 'c') {
        abort();
    }
}
void target_fun_4(char buf[])
{
    if (buf[1] == 'd') {
        abort();
    }
}
void target_fun_5(char buf[])
{
    if (buf[1] == 'f') {
        abort();
    }
}

int main(int argc, char *argv[])
{
    int f = open(argv[1], O_RDONLY);
    read(f, buf, 12);

    switch(buf[0]) {
        case 'a': target_fun_1(buf); break;
        case 'b': target_fun_2(buf); break;
        case 'c': target_fun_3(buf); break;
        case 'd': target_fun_4(buf); break;
        case 'e': target_fun_5(buf); break;
        default: break;
    }
}

