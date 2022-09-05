# include <stdio.h>
# include <stdlib.h>
# include <string.h>
# include <unistd.h>
# include <fcntl.h>

char buf[15];

void path_1(char buf[])
{
    if (buf[0] == 'o') {
        if (buf[1] == 'b') {
            if (buf[2] == 'f') {
                if (buf[3] == 'u') {
                    if (buf[4] == 's') {
                        if (buf[5] == 'c') {
                            if (buf[6] == 'a') {
                                if (buf[7] == 't') {
                                    if (buf[8] == 'i') {
                                        if (buf[9] == 'o') {
                                            if (buf[10] == 'n') {
                                                return;
                                            }
                                        }
                                    }
                                    else if (buf[8] == 'e') {
                                        if (buf[9] == 'd') {
                                            if (buf[10] == '!') {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
void path_2(char buf[])
{
    if (buf[0] == 'i') {
        if (buf[1] == 'c') {
            if (buf[2] == 'e') {
                if (buf[3] == 'b') {
                    if (buf[4] == 'e') {
                        if (buf[5] == 'r') {
                            if (buf[6] == 'g') {
                                return;
                            }
                        }
                    }
                    else if (buf[4] == 'o') {
                        if (buf[5] == 'a') {
                            if (buf[6] == 't') {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}
void path_3(char buf[])
{
    if (buf[0] == 'h') {
        if (buf[1] == 'a') {
            if (buf[2] == 'b') {
                if (buf[3] == 'i') {
                    if (buf[4] == 't') {
                        return;
                    }
                }
            }
        }
    }
    else if (buf[0] == 'e') {
        if (buf[1] == 'a') {
            if (buf[2] == 'g') {
                if (buf[3] == 'l') {
                    if (buf[4] == 'e') {
                        return;
                    }
                }
            }
        }
    }
}
void path_4(char buf[])
{
    if (buf[0] == 'a') {
        if (buf[1] == 'b') {
            if (buf[2] == 'b') {
                if (buf[3] == 'r') {
                    if (buf[4] == 'e') {
                        if (buf[5] == 'v') {
                            if (buf[6] == 'i') {
                                if (buf[7] == 'a') {
                                    if (buf[8] == 't') {
                                        if (buf[9] == 'i') {
                                            if (buf[10] == 'o') {
                                                if (buf[11] == 'n') {
                                                    if (buf[12] == 's') {
                                                        abort();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

int main(int argc, char *argv[])
{
    int f = open(argv[1], O_RDONLY);
    read(f, buf, 15);

    switch(strlen(buf)) {
        case 11: path_1(buf); break;
        case 7: path_2(buf); break;
        case 5: path_3(buf); break;
        case 13: path_4(buf); break;
        default: break;
    }
}

