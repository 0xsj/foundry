CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g

# Find all test files
TEST_SOURCES = $(wildcard */*_test.c)
TEST_BINS = $(TEST_SOURCES:.c=)

.PHONY: all test clean

all: test

test: $(TEST_BINS)
	@for test in $(TEST_BINS); do \
		echo "Running $$test..."; \
		./$$test || exit 1; \
	done

%_test: %_test.c
	$(CC) $(CFLAGS) -o $@ $

clean:
	rm -f $(TEST_BINS)
	find . -name "*.o" -delete
