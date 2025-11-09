CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g

.PHONY: all test clean

all: test

test: arrays-and-hashing/two_sum_test
	./arrays-and-hashing/two_sum_test

arrays-and-hashing/two_sum_test: arrays-and-hashing/two_sum_test.c arrays-and-hashing/two_sum.c
	$(CC) $(CFLAGS) -o $@ $^

clean:
	rm -f arrays-and-hashing/two_sum_test
	find . -name "*.o" -delete
