function findNumbers(nums: number[]): number {
  let result = 0;

  // since numbers do not have a length property, convert into a string
  let str = nums.map((num) => num.toString().length);

  for (let i = 0; i < str.length; i++) {
    if (str[i] % 2 === 0) {
      result++;
    }
  }

  return result;
}

function findNumbers2(nums: number[]): number {
  let result = 0;
  const str = nums.map((num) => num.toString().length);
  let start = 0;
  let end = str.length - 1;

  while (start < end) {
    if (str[start] % 2 === 0) {
      result++;
    }
    if (str[end] % 2 === 0) {
      result++;
    }

    start++;
    end--;
  }

  if (start === end && str[start] % 2 === 0) {
    result++;
  }

  return result;
}

// try 3

// findNumbers([]); // 0
console.log(findNumbers2([773, 165, 42, 381, 123])); // 1
// findNumbers([555, 901, 482, 1771]); // 1
console.log(findNumbers2([12, 345, 2, 6, 7896])); // 2
