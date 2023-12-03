// fibonnaci

function climbStairs(n: number): number {
  if (n === 1 || n === 0) {
    return 1;
  }
  let permutations: number[] = new Array(n + 1);
  permutations[0] = 1;
  permutations[1] = 1;

  for (let i = 2; i <= n; i++) {
    permutations[i] = permutations[i - 1] + permutations[i - 2];
  }

  return permutations[n];

  // number of steps can't be < 2;

  // we can call this function until n = 0
}

console.log(climbStairs(2)); // [1, 1], [2]
console.log(climbStairs(4)); // [1, 1, 1, 1], [1, 2, 1], [2, 1, 1], etc.
console.log(climbStairs(3)); //
