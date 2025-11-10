def is_palindrome(x: int) -> bool:

    if x < 0:
        return False

    if x % 10 == 0 and x != 0:
        return False

    num_str = str(x)

    left = 0;
    right = len(num_str) - 1

    while left < right:
        if num_str[left] != num_str[right]:
            return False

        left += 1
        right -= 1

    return True