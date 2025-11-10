#include <stdio.h>
#include <stdbool.h>
#include <string.h>

bool isPalindrome(int x) {
    if (x < 0) {
        return false;
    }

    if (x % 10 == 0 && x != 0) {
        return false;
    }

    char numStr[12];
    sprintf(numStr, "%d", x);

    int left = 0;
    int right = strlen(numStr) - 1;

    while (left < right) {
        if (numStr[left] != numStr[right]) {
            return false;
        }
        left++;
        right--;
    }

    return true;
}