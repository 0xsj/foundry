const digits = [1, 2, 3]; // [1, 2, 4]
const digits2 = [4, 3, 2, 1]; // [4, 3, 2, 2]
const digits3 = [1]; //[2]

/**
 * 1. initialize n to the full length of the digits array
 * 2. the carry, represents the value that needs to be varried over to the next higher order digit when adding 1 to the current
 * we have to thinkn in terms of math. 9 + 1 = 10, and when its 10, the 1 carries over to the next.
 * for example, lets say the number is 89. 9 + 1 = 10 (1, 0), we carry over the 1 to the 8, and the last digit becomes 0.
 * 3. we iterate over the loop. starting from the least significant digit (last element in array)
 * 4. the sum, is the calculated sum of the current digit and the carry value.
 * 5. we update the current digit digit[i] with the sum modulo 10.
 * 6. we check if the carry is 0, because we want to find out whether or not we need to carry over
 * break out if carry is 0
 * 7. if we have a carry left, insert it at the start of the array.
 */
function plusOne(digits: number[]): number[] {
  const n = digits.length;
  let carry = 1;

  for (let i = n - 1; i >= 0; i--) {
    const sum = digits[i] + carry;
    digits[i] = sum % 10;
    carry = Math.floor(sum / 10);

    if (carry === 0) {
      break;
    }
  }

  if (carry > 0) {
    digits.unshift(carry);
  }

  return digits;
}
console.log(plusOne(digits3));
