prog: prog.o
	gcc -o prog prog.o

prog.o: prog.c lib.c
	gcc -c prog.c lib.c