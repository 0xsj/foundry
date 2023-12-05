let test = 11111111111111111111111111111101;
function hammingWeight(n: number): number {
  let str = n.toString(2);
  let count = 0;
  for (let i = 0; i < str.length; i++) {
    if (str[i] === "1") {
      count++;
    }
  }

  console.log(count);

  return count;
}

// console.log(hammingWeight(0b1011111010101010101));
hammingWeight(test);
