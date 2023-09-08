function kidsWithCandies_bruceforce(candies: number[], extraCandies: number) {
  const maxValue = Math.max(...candies);
  const results: boolean[] = [];

  const fn = (n: number) => {
    return n + extraCandies >= maxValue;
  };

  for (let i = 0; i < candies.length; i++) {
    results.push(fn(candies[i]));
  }

  return results;
}

// console.log(kidsWithCandies_bruceforce([2, 3, 5, 1, 3], 3));
// console.log(kidsWithCandies_bruceforce([4, 2, 1, 1, 2], 1));
// console.log(kidsWithCandies_bruceforce([12, 1, 12], 10));

function kidsWithCandies_reduce(candies: number[], extraCandies: number) {
  const maxValue = Math.max(...candies);
  const fn = (n: number) => {
    return n + extraCandies >= maxValue;
  };

  return candies.reduce<boolean[]>((result, currValue, index) => {
    result.push(fn(currValue));
    return result;
  }, []);
}

console.log(kidsWithCandies_reduce([2, 3, 5, 1, 3], 3));
console.log(kidsWithCandies_reduce([4, 2, 1, 1, 2], 1));
console.log(kidsWithCandies_reduce([12, 1, 12], 10));
